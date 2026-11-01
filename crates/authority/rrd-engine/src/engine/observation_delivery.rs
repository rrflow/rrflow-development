//! Engine-owned metadata export. Delivery cannot execute or complete its source task.
use super::clock::{observe_required, ClockSource, HostSystemClock};
use super::*;
use rrd_contract::{
    DeliverObservation, ObservationDeliveryDisposition, ObservationDeliveryReadResult,
    ObservationDeliverySnapshot, ObservationDeliveryState, ObservationExportPolicy,
    ObservationSource, PrepareObservationDelivery, ReadObservationDelivery, TaskAttemptStatus,
    MAX_OBSERVATION_SOURCE_BYTES, OBSERVATION_DELIVERY_VERSION,
};
use rrd_observation::{EngineReferences, PreparedExport, TransportObservation};

mod persistence;
use persistence::*;

impl RrdEngine {
    pub fn prepare_observation_delivery(
        &self,
        session: &CorrelationId,
        token: &CorrelationId,
        request: &PrepareObservationDelivery,
        request_id: &str,
        operation_id: &str,
    ) -> Result<ObservationDeliverySnapshot> {
        let now = observe_required(&HostSystemClock, "observation-prepare")?.observed_at_unix_ms;
        self.prepare_observation_delivery_at(session, token, request, now, request_id, operation_id)
    }

    pub(super) fn prepare_observation_delivery_at(
        &self,
        session: &CorrelationId,
        token: &CorrelationId,
        request: &PrepareObservationDelivery,
        now: u64,
        request_id: &str,
        operation_id: &str,
    ) -> Result<ObservationDeliverySnapshot> {
        request.validate().map_err(contract)?;
        self.require_persistent_root("observation delivery")?;
        let context = EngineControlContext {
            now,
            session_id: session,
            action: "observation.prepared",
            request_id,
            operation_id,
        };
        let authority = self.observation_authority(
            session,
            token,
            SecurityAction::ObservationPrepare,
            &context,
        )?;
        let key = self.observation_record_key(&authority.principal, &request.delivery_id)?;
        let request_sha256 = operation_digest(request)?;
        let (mut index_bytes, mut index) = self.observation_index()?;
        if let Some(entry) = index.entries.get(&key) {
            let (_, record) = self.observation_record(&key, entry)?;
            self.authorize_observation_source(session, token, &record.snapshot.source, &context)?;
            if record.owner != authority.principal || record.request_sha256 != request_sha256 {
                return Err(ServiceError::IdempotencyConflict);
            }
            return Ok(record.snapshot);
        }
        let (configuration_bytes, configuration) = self.load_endpoint_configuration()?;
        if configuration.sha256().map_err(contract)? != request.expected_configuration_sha256 {
            return Err(ServiceError::StorageConflict(
                "observation endpoint configuration changed".into(),
            ));
        }
        let policy = policy(&configuration)?;
        if !policy.allow_network || !policy.allow_metadata_export {
            return Err(ServiceError::PermissionDenied);
        }
        let (source_json, mut references, source_guard) =
            self.observation_source(session, token, &request.source, &context)?;
        if policy
            .allowed_scopes
            .binary_search(&references.scope)
            .is_err()
        {
            return Err(ServiceError::PermissionDenied);
        }
        references.response_sha256 = digest::sha256_hex(source_json.as_bytes());
        let event_id =
            self.observation_uuid("event", &authority.principal, &request.delivery_id)?;
        let capture_id =
            self.observation_uuid("capture", &authority.principal, &request.delivery_id)?;
        let payload = rrd_observation::encode_event(
            &event_id,
            &capture_id,
            request.delivery_id.as_str(),
            now,
            &references,
        )
        .map_err(|message| ServiceError::Contract(message.into()))?;
        let binding = binding(&configuration, policy)?;
        PreparedExport::new(binding, policy, &payload, &event_id)
            .map_err(|message| ServiceError::Contract(message.into()))?;
        let expires = now
            .checked_add(policy.retention_ms)
            .ok_or(ServiceError::DeadlineExceeded)?;
        let snapshot = ObservationDeliverySnapshot {
            contract_version: OBSERVATION_DELIVERY_VERSION,
            delivery_id: request.delivery_id.clone(),
            source: request.source.clone(),
            configuration_sha256: request.expected_configuration_sha256.clone(),
            source_response_sha256: references.response_sha256.clone(),
            payload_sha256: digest::sha256_hex(&payload),
            event_id,
            scope: references.scope.clone(),
            state: ObservationDeliveryState::Prepared,
            attempts: 0,
            prepared_at_unix_ms: now,
            expires_at_unix_ms: expires,
            next_attempt_at_unix_ms: now,
            lease_expires_at_unix_ms: None,
            http_status: None,
            acknowledgement_sha256: None,
            disposition: None,
        };
        let record = DeliveryRecord {
            format_version: 1,
            owner: authority.principal.clone(),
            request_sha256,
            capture_id,
            references,
            source_json,
            payload_json: String::from_utf8(payload)
                .map_err(|_| ServiceError::Contract("invalid encoded observation".into()))?,
            snapshot,
        };
        // Validate the complete new record before any retirement can commit.
        let bytes = encode_record(&record)?;
        // Unresolved expiry requires a separate explicit configuration grant.
        // Retirement is audited; it never changes an unknown outcome to success.
        let expired: Vec<_> = index
            .entries
            .iter()
            .filter(|(_, entry)| {
                entry.expires_at_unix_ms <= now
                    && (policy.retire_expired_uncertain
                        || matches!(
                            entry.state,
                            ObservationDeliveryState::Prepared
                                | ObservationDeliveryState::NotDispatched
                                | ObservationDeliveryState::Acknowledged
                        ))
                    && entry
                        .lease_expires_at_unix_ms
                        .is_none_or(|expires| now >= expires)
            })
            .map(|(key, entry)| (key.clone(), entry.clone()))
            .collect();
        if index.entries.len() - expired.len() >= policy.max_retained_deliveries as usize {
            return Err(ServiceError::ConfigurationLimit(
                "observation history is full; unresolved deliveries remain visible".into(),
            ));
        }
        // The index contains at most 128 records. At most three batches retire
        // its eligible history when a configuration edit lowers the cap. Each
        // batch retains source/configuration/authority/index CAS guards. A
        // conflict can leave audited retirement progress, never a false receipt.
        let mut expired = expired.into_iter().peekable();
        loop {
            let mut writes = vec![source_guard.clone()];
            // 58 retirements + source + new record + index + three authority
            // and configuration guards fit the 64-transition control bound.
            for (old_key, entry) in expired.by_ref().take(58) {
                let (old_bytes, _) = self.observation_record(&old_key, &entry)?;
                writes.push((old_key.clone(), Some(old_bytes), None));
                index.entries.remove(&old_key);
            }
            let final_batch = expired.peek().is_none();
            if final_batch {
                index.entries.insert(
                    key.clone(),
                    DeliveryIndexEntry::from_record(&record, &bytes),
                );
                writes.push((key.clone(), None, Some(bytes.clone())));
            }
            let next_index = encode_index(&index)?;
            writes.push((
                self.observation_index_key(),
                index_bytes,
                Some(next_index.clone()),
            ));
            self.commit_observation(writes, &authority, configuration_bytes.clone(), &context)?;
            if final_batch {
                return Ok(record.snapshot);
            }
            index_bytes = Some(next_index);
        }
    }

    pub fn read_observation_delivery(
        &self,
        session: &CorrelationId,
        token: &CorrelationId,
        request: &ReadObservationDelivery,
        request_id: &str,
        operation_id: &str,
    ) -> Result<ObservationDeliveryReadResult> {
        let now = observe_required(&HostSystemClock, "observation-read")?.observed_at_unix_ms;
        self.read_observation_delivery_at(session, token, request, now, request_id, operation_id)
    }
    pub(super) fn read_observation_delivery_at(
        &self,
        session: &CorrelationId,
        token: &CorrelationId,
        request: &ReadObservationDelivery,
        now: u64,
        request_id: &str,
        operation_id: &str,
    ) -> Result<ObservationDeliveryReadResult> {
        request.validate().map_err(contract)?;
        let context = EngineControlContext {
            now,
            session_id: session,
            action: "observation.read",
            request_id,
            operation_id,
        };
        let authority =
            self.observation_authority(session, token, SecurityAction::ObservationRead, &context)?;
        let key = self.observation_record_key(&authority.principal, &request.delivery_id)?;
        let (_, index) = self.observation_index()?;
        let Some(entry) = index.entries.get(&key) else {
            return Ok(ObservationDeliveryReadResult {
                delivery: None,
                source_response_json: None,
            });
        };
        let (_, record) = self.observation_record(&key, entry)?;
        self.authorize_observation_source(session, token, &record.snapshot.source, &context)?;
        let mut snapshot = record.snapshot;
        if snapshot.state == ObservationDeliveryState::InFlight
            && snapshot
                .lease_expires_at_unix_ms
                .is_some_and(|expires| now >= expires)
        {
            snapshot.state = ObservationDeliveryState::Uncertain;
            snapshot.lease_expires_at_unix_ms = None;
            snapshot.disposition = Some(ObservationDeliveryDisposition::DeadlineExceeded);
        }
        snapshot.validate().map_err(contract)?;
        Ok(ObservationDeliveryReadResult {
            delivery: Some(snapshot),
            source_response_json: Some(record.source_json),
        })
    }

    pub fn deliver_observation(
        &self,
        session: &CorrelationId,
        token: &CorrelationId,
        request: &DeliverObservation,
        request_id: &str,
        operation_id: &str,
    ) -> Result<ObservationDeliverySnapshot> {
        self.deliver_observation_with(
            session,
            token,
            request,
            request_id,
            operation_id,
            &HostSystemClock,
            |prepared, secret, deadline| prepared.execute(secret, deadline),
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn deliver_observation_with(
        &self,
        session: &CorrelationId,
        token: &CorrelationId,
        request: &DeliverObservation,
        request_id: &str,
        operation_id: &str,
        clock: &dyn ClockSource,
        send: impl FnOnce(&PreparedExport<'_>, &str, Instant) -> TransportObservation,
    ) -> Result<ObservationDeliverySnapshot> {
        request.validate().map_err(contract)?;
        let now = observe_required(clock, "observation-delivery-admission")?.observed_at_unix_ms;
        let context = EngineControlContext {
            now,
            session_id: session,
            action: "observation.dispatch-admitted",
            request_id,
            operation_id,
        };
        let authority = self.observation_authority(
            session,
            token,
            SecurityAction::ObservationDeliver,
            &context,
        )?;
        let key = self.observation_record_key(&authority.principal, &request.delivery_id)?;
        let (index_bytes, mut index) = self.observation_index()?;
        let entry = index
            .entries
            .get(&key)
            .ok_or_else(|| ServiceError::Contract("observation delivery is absent".into()))?;
        let (record_bytes, mut record) = self.observation_record(&key, entry)?;
        self.authorize_observation_source(session, token, &record.snapshot.source, &context)?;
        if record.snapshot.payload_sha256 != request.expected_payload_sha256
            || record.snapshot.configuration_sha256 != request.expected_configuration_sha256
        {
            return Err(ServiceError::IdempotencyConflict);
        }
        // Readback of already accepted storage never repeats network work.
        if record.snapshot.state == ObservationDeliveryState::Acknowledged {
            return Ok(record.snapshot);
        }
        let (configuration_bytes, configuration) = self.load_endpoint_configuration()?;
        if configuration.sha256().map_err(contract)? != record.snapshot.configuration_sha256 {
            return Err(ServiceError::StorageConflict(
                "observation export configuration changed or was revoked".into(),
            ));
        }
        let policy = policy(&configuration)?;
        if !policy.allow_network
            || !policy.allow_metadata_export
            || policy
                .allowed_scopes
                .binary_search(&record.snapshot.scope)
                .is_err()
        {
            return Err(ServiceError::PermissionDenied);
        }
        if now < record.snapshot.prepared_at_unix_ms
            || now >= record.snapshot.expires_at_unix_ms
            || now < record.snapshot.next_attempt_at_unix_ms
        {
            return Err(ServiceError::DeadlineExceeded);
        }
        if record.snapshot.attempts >= policy.max_attempts {
            return Err(ServiceError::ConfigurationLimit(
                "observation attempts exhausted; remote outcome remains unresolved".into(),
            ));
        }
        if index.entries.values().any(|entry| {
            entry.state == ObservationDeliveryState::InFlight
                && entry
                    .lease_expires_at_unix_ms
                    .is_some_and(|expires| now < expires)
        }) {
            return Err(ServiceError::StorageConflict(
                "an observation dispatch is still admitted".into(),
            ));
        }
        let binding = binding(&configuration, policy)?;
        let secret = binding
            .credential
            .as_ref()
            .and_then(|reference| {
                self.endpoint_credentials
                    .as_ref()
                    .and_then(|credentials| credentials.resolve(reference))
            })
            .ok_or_else(|| {
                ServiceError::Contract("observation credential is unavailable".into())
            })?;
        let payload = record.payload_json.clone();
        let event_id = record.snapshot.event_id.clone();
        let prepared = PreparedExport::new(binding, policy, payload.as_bytes(), &event_id)
            .map_err(|message| ServiceError::Contract(message.into()))?;
        let uncertain_before = matches!(
            record.snapshot.state,
            ObservationDeliveryState::Uncertain | ObservationDeliveryState::InFlight
        );
        let timeout = binding
            .limits
            .timeout_ms
            .min(record.snapshot.expires_at_unix_ms - now);
        let deadline = Instant::now()
            .checked_add(Duration::from_millis(timeout))
            .ok_or(ServiceError::DeadlineExceeded)?;
        record.snapshot.attempts += 1;
        record.snapshot.state = ObservationDeliveryState::InFlight;
        record.snapshot.lease_expires_at_unix_ms = Some(
            now.checked_add(timeout)
                .ok_or(ServiceError::DeadlineExceeded)?,
        );
        record.snapshot.http_status = None;
        record.snapshot.disposition = None;
        let admitted = encode_record(&record)?;
        index.entries.insert(
            key.clone(),
            DeliveryIndexEntry::from_record(&record, &admitted),
        );
        let admitted_index = encode_index(&index)?;
        self.commit_observation(
            vec![
                (key.clone(), Some(record_bytes), Some(admitted.clone())),
                (
                    self.observation_index_key(),
                    index_bytes,
                    Some(admitted_index.clone()),
                ),
            ],
            &authority,
            configuration_bytes.clone(),
            &context,
        )?;
        let observed = send(&prepared, secret, deadline);
        let completed =
            observe_required(clock, "observation-delivery-receipt")?.observed_at_unix_ms;
        if completed < now
            || record
                .snapshot
                .lease_expires_at_unix_ms
                .is_some_and(|expires| completed >= expires)
        {
            return Err(ServiceError::DeadlineExceeded);
        }
        // A revoked authority/configuration or a competing fence cannot publish
        // this completion. The admitted record remains recoverably uncertain.
        let receipt_context = EngineControlContext {
            now: completed,
            session_id: session,
            action: "observation.receipt",
            request_id,
            operation_id,
        };
        let receipt_authority = self.observation_authority(
            session,
            token,
            SecurityAction::ObservationDeliver,
            &receipt_context,
        )?;
        self.authorize_observation_source(
            session,
            token,
            &record.snapshot.source,
            &receipt_context,
        )?;
        if receipt_authority != authority {
            return Err(ServiceError::PermissionDenied);
        }
        if self.load_endpoint_configuration()?.0 != configuration_bytes {
            return Err(ServiceError::StorageConflict(
                "observation export changed during dispatch".into(),
            ));
        }
        record.snapshot.lease_expires_at_unix_ms = None;
        record.snapshot.http_status = observed.http_status;
        record.snapshot.disposition = Some(observed.disposition);
        record.snapshot.next_attempt_at_unix_ms = completed
            .checked_add(policy.retry_delay_ms)
            .ok_or(ServiceError::DeadlineExceeded)?;
        if observed.disposition == ObservationDeliveryDisposition::Acknowledged {
            record.snapshot.state = ObservationDeliveryState::Acknowledged;
            record.snapshot.acknowledgement_sha256 = observed.reply_sha256;
        } else {
            record.snapshot.state = if uncertain_before || observed.request_may_have_been_sent {
                ObservationDeliveryState::Uncertain
            } else {
                ObservationDeliveryState::NotDispatched
            };
        }
        let bytes = encode_record(&record)?;
        index.entries.insert(
            key.clone(),
            DeliveryIndexEntry::from_record(&record, &bytes),
        );
        self.commit_observation(
            vec![
                (key, Some(admitted), Some(bytes)),
                (
                    self.observation_index_key(),
                    Some(admitted_index),
                    Some(encode_index(&index)?),
                ),
            ],
            &receipt_authority,
            configuration_bytes,
            &receipt_context,
        )?;
        Ok(record.snapshot)
    }
}

fn contract(error: rrd_contract::ContractError) -> ServiceError {
    ServiceError::Contract(error.to_string())
}
fn policy(configuration: &rrd_contract::EndpointConfiguration) -> Result<&ObservationExportPolicy> {
    configuration
        .profile
        .as_ref()
        .and_then(|profile| profile.observation_policy.as_ref())
        .ok_or_else(|| ServiceError::Contract("observation export is not configured".into()))
}
fn binding<'a>(
    configuration: &'a rrd_contract::EndpointConfiguration,
    policy: &ObservationExportPolicy,
) -> Result<&'a rrd_contract::EndpointBinding> {
    configuration
        .profile
        .as_ref()
        .and_then(|profile| {
            profile
                .bindings
                .iter()
                .find(|binding| binding.id == policy.binding_id)
        })
        .ok_or_else(|| ServiceError::Contract("observation export binding is absent".into()))
}
