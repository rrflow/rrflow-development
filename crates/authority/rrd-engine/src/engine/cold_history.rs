//! Engine-owned, derived cold-history publication. Source history remains
//! authoritative; publication never reclaims or replaces it.

use super::clock::{observe_required, HostSystemClock};
use super::*;
use rrd_contract::{
    ColdHistoryCompression, ColdHistoryFailure, ColdHistoryFailureCode,
    ColdHistoryGenerationSnapshot, ColdHistoryPolicy, ColdHistoryReadResult, ReadColdHistory,
};
use rrd_query::{
    decode_cold_history_page, encode_cold_history_page, ColdHistoryReadExpectation,
    ColdHistoryReadLimits, ParquetCompression, ParquetEncoding, ParquetExportLimits,
};
use rrd_store::filesystem::{AnchoredPath, OpenOptions};
use std::io::{Read, Write};
use std::sync::TryLockError;

const COLD_STATE_VERSION: u16 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ColdState {
    version: u16,
    generation: u64,
    through_cursor: u64,
    last_receipt_sha256: Option<String>,
    pending: Option<ColdPending>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    failure: Option<ColdHistoryFailure>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ColdPending {
    stamp: ReadStamp,
    requested_after: u64,
    through_cursor: u64,
    scan_limit: u64,
    source_sha256: String,
    policy_sha256: String,
    artifact_sha256: String,
    artifact_bytes: u64,
    rows: u64,
    row_groups: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ColdReceipt {
    version: u16,
    generation: u64,
    previous_receipt_sha256: Option<String>,
    pending: ColdPending,
}

impl RrdEngine {
    /// Read engine-owned cold-history status or one authenticated generation.
    /// Restricted row/field grants are denied because whole change records
    /// cannot yet be filtered by this operation.
    #[allow(clippy::too_many_arguments)]
    pub fn read_cold_history(
        &self,
        session: &CorrelationId,
        token: &CorrelationId,
        request: &ReadColdHistory,
        now: u64,
        request_id: &str,
        operation_id: &str,
    ) -> Result<ColdHistoryReadResult> {
        request
            .validate()
            .map_err(|error| ServiceError::Contract(error.to_string()))?;
        self.enforce_query_configuration("cold_history.read", &request.budget)?;
        self.authorize_resource_without_data_policy(
            session,
            token,
            SecurityAction::ChangefeedRead,
            &self.instance_resource(),
            now,
            request_id,
            operation_id,
        )?;
        let scope = self.query_scope(&request.scope)?;
        let deadline = Instant::now()
            .checked_add(Duration::from_millis(request.budget.max_elapsed_ms))
            .ok_or(ServiceError::DeadlineExceeded)?;
        let key = format!("server/state/{}/cold-history/head", self.instance);
        let state = match self.storage.control().get(&key)? {
            Some(bytes) => serde_json::from_slice::<ColdState>(&bytes).map_err(contract_json)?,
            None => ColdState {
                version: COLD_STATE_VERSION,
                generation: 0,
                through_cursor: 0,
                last_receipt_sha256: None,
                pending: None,
                failure: None,
            },
        };
        if state.version != COLD_STATE_VERSION
            || (state.generation == 0) != state.last_receipt_sha256.is_none()
            || (state.generation == 0 && state.through_cursor != 0)
        {
            return Err(ServiceError::Storage("invalid cold history state".into()));
        }
        let mut result = ColdHistoryReadResult {
            scope: request.scope.clone(),
            enabled: self.estate_configuration.storage.cold_history.is_some(),
            current_generation: state.generation,
            through_cursor: state.through_cursor,
            pending: state.pending.is_some(),
            failure: state.failure.clone(),
            generation: None,
            changes: Vec::new(),
        };
        let Some(generation) = request.generation else {
            let mut output = CountOutput {
                limit: request.budget.max_output_bytes,
                used: 0,
            };
            serde_json::to_writer(&mut output, &result).map_err(|_| {
                ServiceError::ConfigurationLimit(
                    "cold history response exceeds output-byte budget".into(),
                )
            })?;
            check_deadline(deadline)?;
            return Ok(result);
        };
        if generation > state.generation {
            return Err(ServiceError::Changefeed(
                "cold history generation is not published".into(),
            ));
        }
        let chain_reads = state
            .generation
            .checked_sub(generation)
            .and_then(|value| value.checked_add(1))
            .ok_or_else(|| {
                ServiceError::Storage("cold history generation range overflow".into())
            })?;
        if chain_reads > request.budget.max_storage_keys {
            return Err(ServiceError::ConfigurationLimit(
                "cold history receipt chain exceeds storage-key budget".into(),
            ));
        }
        let mut expected_sha = state
            .last_receipt_sha256
            .clone()
            .ok_or_else(|| ServiceError::Storage("cold history head digest is missing".into()))?;
        let mut expected_through = state.through_cursor;
        let mut selected = None;
        for current in (generation..=state.generation).rev() {
            check_deadline(deadline)?;
            if !valid_sha256(&expected_sha) {
                return Err(ServiceError::Storage(
                    "cold history receipt digest is invalid".into(),
                ));
            }
            let key = format!(
                "server/state/{}/cold-history/generation/{current:020}",
                self.instance
            );
            let bytes =
                self.storage.control().get(&key)?.ok_or_else(|| {
                    ServiceError::Storage("cold history receipt is missing".into())
                })?;
            if digest::sha256_hex(&bytes) != expected_sha {
                return Err(ServiceError::Storage(
                    "cold history receipt digest mismatch".into(),
                ));
            }
            let receipt: ColdReceipt = serde_json::from_slice(&bytes).map_err(contract_json)?;
            receipt.pending.stamp.validate().map_err(core_contract)?;
            if receipt.version != COLD_STATE_VERSION
                || receipt.generation != current
                || receipt.pending.stamp.scope != scope
                || receipt.pending.through_cursor != expected_through
                || receipt.pending.requested_after >= receipt.pending.through_cursor
                || receipt.pending.through_cursor > receipt.pending.stamp.commit_cursor
                || !valid_sha256(&receipt.pending.source_sha256)
                || !valid_sha256(&receipt.pending.policy_sha256)
                || !valid_sha256(&receipt.pending.artifact_sha256)
                || receipt.pending.artifact_bytes == 0
                || receipt.pending.scan_limit == 0
                || receipt.pending.scan_limit > 65_536
                || receipt.pending.rows > receipt.pending.scan_limit
                || receipt.pending.row_groups > receipt.pending.rows
                || (current == 1) != receipt.previous_receipt_sha256.is_none()
            {
                return Err(ServiceError::Storage(
                    "cold history receipt is invalid".into(),
                ));
            }
            if current == generation {
                selected = Some(receipt);
                break;
            }
            expected_through = receipt.pending.requested_after;
            expected_sha = receipt.previous_receipt_sha256.ok_or_else(|| {
                ServiceError::Storage("cold history receipt chain is incomplete".into())
            })?;
        }
        let receipt = selected
            .ok_or_else(|| ServiceError::Storage("cold history generation is missing".into()))?;
        if receipt.pending.artifact_bytes > request.max_artifact_bytes
            || receipt.pending.rows > request.budget.max_rows
        {
            return Err(ServiceError::ConfigurationLimit(
                "cold history generation exceeds the requested artifact or row budget".into(),
            ));
        }
        let store = self.storage.as_rrflow_kv().ok_or_else(|| {
            ServiceError::Storage("cold history requires persistent rrflowKV".into())
        })?;
        let path = store
            .anchored_path()
            .join("cold-history")
            .join(format!("{}.parquet", receipt.pending.artifact_sha256));
        let file = path.open_read().map_err(io_error)?;
        if file.metadata().map_err(io_error)?.len() != receipt.pending.artifact_bytes {
            return Err(ServiceError::Storage(
                "cold history artifact length mismatch".into(),
            ));
        }
        let length = native(receipt.pending.artifact_bytes)?;
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(length.saturating_add(1))
            .map_err(|_| {
                ServiceError::ConfigurationLimit("cold history artifact allocation failed".into())
            })?;
        file.take(receipt.pending.artifact_bytes.saturating_add(1))
            .read_to_end(&mut bytes)
            .map_err(io_error)?;
        if bytes.len() != length {
            return Err(ServiceError::Storage(
                "cold history artifact changed while reading".into(),
            ));
        }
        let expectation = ColdHistoryReadExpectation {
            stamp: receipt.pending.stamp.clone(),
            requested_after: receipt.pending.requested_after,
            through_cursor: receipt.pending.through_cursor,
            rows: receipt.pending.rows,
            artifact_sha256: receipt.pending.artifact_sha256.clone(),
        };
        let decoded = decode_cold_history_page(
            bytes,
            &expectation,
            ColdHistoryReadLimits {
                max_rows: native(request.budget.max_rows)?,
                max_encoded_bytes: native(request.max_artifact_bytes)?,
                max_decoded_bytes: native(request.budget.max_memory_bytes / 32)?,
                max_memory_bytes: native(request.budget.max_memory_bytes)?,
                max_batch_rows: native(request.budget.max_batch_rows)?,
                deadline,
            },
        )
        .map_err(ServiceError::from)?;
        if decoded.row_groups as u64 != receipt.pending.row_groups {
            return Err(ServiceError::Storage(
                "cold history row-group count differs from its receipt".into(),
            ));
        }
        if source_digest_parts(
            &receipt.pending.stamp,
            receipt.pending.requested_after,
            receipt.pending.through_cursor,
            receipt.pending.stamp.commit_cursor,
            &decoded.changes,
        ) != receipt.pending.source_sha256
        {
            return Err(ServiceError::Storage(
                "cold history decoded changes differ from the prepared source".into(),
            ));
        }
        result.changes = decoded
            .changes
            .iter()
            .map(public_runtime_change)
            .collect::<Result<Vec<_>>>()?;
        result.generation = Some(ColdHistoryGenerationSnapshot {
            generation,
            requested_after_cursor: receipt.pending.requested_after,
            through_cursor: receipt.pending.through_cursor,
            source_head_cursor: receipt.pending.stamp.commit_cursor,
            source_manifest_sha256: receipt.pending.stamp.manifest_id,
            policy_sha256: receipt.pending.policy_sha256,
            artifact_sha256: receipt.pending.artifact_sha256,
            artifact_bytes: receipt.pending.artifact_bytes,
            rows: receipt.pending.rows,
            row_groups: receipt.pending.row_groups,
            observed_arrow_bytes: decoded.observed_arrow_bytes as u64,
            declared_uncompressed_bytes: decoded.declared_uncompressed_bytes,
        });
        let mut output = CountOutput {
            limit: request.budget.max_output_bytes,
            used: 0,
        };
        serde_json::to_writer(&mut output, &result).map_err(|_| {
            ServiceError::ConfigurationLimit(
                "cold history response exceeds output-byte budget".into(),
            )
        })?;
        check_deadline(deadline)?;
        Ok(result)
    }

    /// Offer one configured cold-history quantum. The server coalesces offers;
    /// embedded callers own their own scheduling.
    pub fn maintain_cold_history(&self) -> Result<bool> {
        let Some(policy) = self.estate_configuration.storage.cold_history.as_ref() else {
            return Ok(false);
        };
        let _guard = match self.cold_history_gate.try_lock() {
            Ok(guard) => guard,
            Err(TryLockError::WouldBlock) => return Ok(false),
            Err(TryLockError::Poisoned(_)) => {
                return Err(ServiceError::Storage(
                    "cold history maintenance lock poisoned".into(),
                ))
            }
        };
        let now =
            observe_required(&HostSystemClock, "cold_history.maintenance")?.observed_at_unix_ms;
        match self.maintain_cold_history_at(policy, now) {
            Ok(progress) => Ok(progress),
            Err(error) => {
                if self.record_cold_history_failure(&error, now).is_err() {
                    tracing::warn!(target: "rrflow::parquet_compactor", "could not persist cold history failure status");
                }
                Err(error)
            }
        }
    }

    fn maintain_cold_history_at(&self, policy: &ColdHistoryPolicy, now: u64) -> Result<bool> {
        policy
            .validate()
            .map_err(|error| ServiceError::Contract(error.to_string()))?;
        let Some(store) = self.storage.as_rrflow_kv() else {
            return Ok(false);
        };
        let deadline = Instant::now()
            .checked_add(Duration::from_millis(policy.max_elapsed_ms))
            .ok_or(ServiceError::DeadlineExceeded)?;
        let policy_sha256 = digest::sha256_hex(&serde_json::to_vec(policy).map_err(contract_json)?);
        let key = format!("server/state/{}/cold-history/head", self.instance);
        let previous = self.storage.control().get(&key)?;
        let mut state = match previous.as_deref() {
            Some(bytes) => serde_json::from_slice::<ColdState>(bytes).map_err(contract_json)?,
            None => ColdState {
                version: COLD_STATE_VERSION,
                generation: 0,
                through_cursor: 0,
                last_receipt_sha256: None,
                pending: None,
                failure: None,
            },
        };
        if state.version != COLD_STATE_VERSION
            || (state.generation == 0) != state.last_receipt_sha256.is_none()
            || (state.generation == 0 && state.through_cursor != 0)
        {
            return Err(ServiceError::Storage("invalid cold history state".into()));
        }
        if state
            .last_receipt_sha256
            .as_ref()
            .is_some_and(|hash| !valid_sha256(hash))
        {
            return Err(ServiceError::Storage(
                "invalid cold history previous receipt digest".into(),
            ));
        }
        if state.generation > 0 && state.pending.is_some() {
            verify_head(self, store.anchored_path(), &state, deadline)?;
        }
        if let Some(pending) = &state.pending {
            let expected_scope =
                ScopeId::new(format!("instance:{}", self.instance)).map_err(core_contract)?;
            pending.stamp.validate().map_err(core_contract)?;
            if pending.stamp.scope != expected_scope
                || pending.requested_after != state.through_cursor
                || pending.through_cursor <= pending.requested_after
                || pending.through_cursor > pending.stamp.commit_cursor
                || pending.scan_limit == 0
                || pending.scan_limit > 65_536
                || pending.artifact_bytes == 0
                || pending.artifact_bytes > 1_073_741_824
                || pending.rows > pending.scan_limit
                || pending.row_groups > pending.rows
                || !valid_sha256(&pending.source_sha256)
                || !valid_sha256(&pending.policy_sha256)
                || !valid_sha256(&pending.artifact_sha256)
            {
                return Err(ServiceError::Storage(
                    "invalid pending cold history coordinates".into(),
                ));
            }
        }
        if state
            .pending
            .as_ref()
            .is_some_and(|pending| pending.policy_sha256 != policy_sha256)
        {
            // A newly authorized policy may change after a pending quantum.
            // Preserve the committed cursor and replan rather than reinterpret
            // old work under different limits or compression settings.
            state.pending = None;
            state.failure = None;
            self.commit_cold_control(transition(
                &key,
                previous,
                &state,
                now,
                "cold_history.replanned",
            )?)?;
            return Ok(true);
        }

        if let Some(pending) = state.pending.clone() {
            if pending.scan_limit > policy.max_changes
                || pending.artifact_bytes > policy.max_output_bytes
            {
                return Err(ServiceError::Storage(
                    "pending cold history exceeds its admitted policy".into(),
                ));
            }
            let page = match self.storage.runtime().read_changes_bounded_bytes(
                &pending.stamp,
                pending.requested_after,
                native(pending.scan_limit)?,
                policy.max_input_bytes,
                deadline,
            ) {
                Ok(page) => page,
                Err(
                    rrd_store::Error::ReadStampMismatch(_)
                    | rrd_store::Error::ReadStampUnavailable(_),
                ) => {
                    // No archive has been committed for this interval. Keep
                    // the previous head and plan again from the new stamp.
                    state.pending = None;
                    state.failure = None;
                    self.commit_cold_control(transition(
                        &key,
                        previous,
                        &state,
                        now,
                        "cold_history.source_replanned",
                    )?)?;
                    return Ok(true);
                }
                Err(error) => return Err(cold_source_error(error)),
            };
            if page.through_cursor != pending.through_cursor
                || source_digest(&pending.stamp, &page) != pending.source_sha256
            {
                return Err(ServiceError::StorageConflict(
                    "cold history source changed; prepared work cannot be published".into(),
                ));
            }
            let artifact = encode(&pending.stamp, &page, policy, deadline)?;
            if digest::sha256_hex(&artifact.artifact.bytes) != pending.artifact_sha256
                || artifact.artifact.bytes.len() as u64 != pending.artifact_bytes
                || page.changes.len() as u64 != pending.rows
                || artifact.artifact.row_groups as u64 != pending.row_groups
            {
                return Err(ServiceError::Storage(
                    "cold history replay does not match prepared artifact".into(),
                ));
            }
            let directory = store.anchored_path().join("cold-history");
            directory.create_dir_all().map_err(io_error)?;
            let final_path = directory.join(format!("{}.parquet", pending.artifact_sha256));
            if !final_path.try_exists().map_err(io_error)?
                && !store.try_admit_immutable_file(pending.artifact_bytes)?
            {
                return Ok(false);
            }
            publish_file(
                &directory,
                &final_path,
                &pending.artifact_sha256,
                &artifact.artifact.bytes,
                deadline,
            )?;

            let generation = state
                .generation
                .checked_add(1)
                .ok_or_else(|| ServiceError::Storage("cold history generation overflow".into()))?;
            let receipt = ColdReceipt {
                version: COLD_STATE_VERSION,
                generation,
                previous_receipt_sha256: state.last_receipt_sha256.clone(),
                pending,
            };
            let receipt_bytes = serde_json::to_vec(&receipt).map_err(contract_json)?;
            let receipt_sha256 = digest::sha256_hex(&receipt_bytes);
            let receipt_key = format!(
                "server/state/{}/cold-history/generation/{generation:020}",
                self.instance
            );
            state.generation = generation;
            state.through_cursor = receipt.pending.through_cursor;
            state.last_receipt_sha256 = Some(receipt_sha256);
            state.pending = None;
            state.failure = None;
            check_deadline(deadline)?;
            self.commit_cold_control_batch(&[
                transition_bytes(
                    &receipt_key,
                    None,
                    receipt_bytes,
                    now,
                    "cold_history.generation",
                ),
                transition(&key, previous, &state, now, "cold_history.published")?,
            ])?;
            tracing::info!(target: "rrflow::parquet_compactor", generation, rows = receipt.pending.rows, bytes = receipt.pending.artifact_bytes, "cold history generation published");
            return Ok(true);
        }

        let scope = ScopeId::new(format!("instance:{}", self.instance)).map_err(core_contract)?;
        let stamp = self.storage.runtime().read_stamp(&scope)?;
        if state.through_cursor > stamp.commit_cursor {
            return Err(ServiceError::Storage(
                "cold history cursor exceeds the source head".into(),
            ));
        }
        if state.through_cursor == stamp.commit_cursor {
            return Ok(false);
        }
        if state.generation > 0 {
            verify_head(self, store.anchored_path(), &state, deadline)?;
        }
        let mut scan_limit = policy.max_changes;
        let (page, artifact) = loop {
            check_deadline(deadline)?;
            let page = self
                .storage
                .runtime()
                .read_changes_bounded_bytes(
                    &stamp,
                    state.through_cursor,
                    native(scan_limit)?,
                    policy.max_input_bytes,
                    deadline,
                )
                .map_err(cold_source_error)?;
            if page.through_cursor <= state.through_cursor {
                return Err(ServiceError::Storage(
                    "cold history source did not advance".into(),
                ));
            }
            match encode_cold_history_page(
                &stamp,
                &page,
                limits(policy, deadline)?,
                encoding(policy),
            ) {
                Ok(artifact) => break (page, artifact),
                Err(rrd_query::Error::Budget(_)) if scan_limit > 1 => {
                    scan_limit = (scan_limit / 2).max(1)
                }
                Err(error) => return Err(cold_query_error(error, deadline)),
            }
        };
        let source_sha256 = source_digest(&stamp, &page);
        let pending = ColdPending {
            stamp,
            requested_after: page.requested_after,
            through_cursor: page.through_cursor,
            scan_limit,
            source_sha256,
            policy_sha256,
            artifact_sha256: digest::sha256_hex(&artifact.artifact.bytes),
            artifact_bytes: artifact.artifact.bytes.len() as u64,
            rows: page.changes.len() as u64,
            row_groups: artifact.artifact.row_groups as u64,
        };
        state.pending = Some(pending);
        state.failure = None;
        check_deadline(deadline)?;
        self.commit_cold_control(transition(
            &key,
            previous,
            &state,
            now,
            "cold_history.prepared",
        )?)?;
        Ok(true)
    }

    /// Publication runs beside foreground requests and appends to the same
    /// control journal, so it waits at the engine's transaction gate like
    /// every other writer. Two writers that skip it meet as a physical write
    /// conflict, which clients see as a runtime conflict at an unchanged
    /// cursor. The gate is held for the commit alone, never for encoding.
    fn commit_cold_control(
        &self,
        transition: rrd_store::ControlTransition,
    ) -> Result<rrd_store::ControlJournalEntry> {
        let _gate = self.admit_transaction(&transition, COLD_REQUEST_ID, COLD_OPERATION_ID)?;
        Ok(self.storage.control().commit(&transition)?)
    }

    fn commit_cold_control_batch(
        &self,
        transitions: &[rrd_store::ControlTransition],
    ) -> Result<Vec<rrd_store::ControlJournalEntry>> {
        let _gate = self.admit_transaction(transitions, COLD_REQUEST_ID, COLD_OPERATION_ID)?;
        Ok(self.storage.control().commit_batch(transitions)?)
    }

    fn record_cold_history_failure(&self, error: &ServiceError, now: u64) -> Result<()> {
        let code = match error.kind() {
            ServiceErrorKind::DeadlineExceeded => ColdHistoryFailureCode::DeadlineExceeded,
            ServiceErrorKind::ResourceExhausted => ColdHistoryFailureCode::ResourceLimit,
            ServiceErrorKind::Conflict => ColdHistoryFailureCode::SourceConflict,
            _ => ColdHistoryFailureCode::StorageOrOther,
        };
        let key = format!("server/state/{}/cold-history/head", self.instance);
        let previous = self.storage.control().get(&key)?;
        let mut state = match previous.as_deref() {
            Some(bytes) => serde_json::from_slice::<ColdState>(bytes).map_err(contract_json)?,
            None => ColdState {
                version: COLD_STATE_VERSION,
                generation: 0,
                through_cursor: 0,
                last_receipt_sha256: None,
                pending: None,
                failure: None,
            },
        };
        if state.version != COLD_STATE_VERSION
            || (state.generation == 0) != state.last_receipt_sha256.is_none()
            || (state.generation == 0 && state.through_cursor != 0)
        {
            return Err(ServiceError::Storage("invalid cold history state".into()));
        }
        if state
            .failure
            .as_ref()
            .is_some_and(|failure| failure.code == code)
        {
            return Ok(());
        }
        state.failure = Some(ColdHistoryFailure {
            code,
            since_unix_ms: now,
        });
        self.commit_cold_control(transition(
            &key,
            previous,
            &state,
            now,
            "cold_history.failure",
        )?)?;
        Ok(())
    }
}

/// Background publication is no client invocation; these name its gate reservation.
const COLD_REQUEST_ID: &str = "cold-history-maintenance";
const COLD_OPERATION_ID: &str = "cold-history-publication";

fn native(value: u64) -> Result<usize> {
    usize::try_from(value).map_err(|_| {
        ServiceError::ConfigurationLimit("cold history limit exceeds native address space".into())
    })
}

fn cold_source_error(error: rrd_store::Error) -> ServiceError {
    match error {
        rrd_store::Error::ColdHistorySourceDeadline => ServiceError::DeadlineExceeded,
        rrd_store::Error::ColdHistorySourceLimit { .. } => {
            ServiceError::ConfigurationLimit(error.to_string())
        }
        other => other.into(),
    }
}

fn cold_query_error(error: rrd_query::Error, deadline: Instant) -> ServiceError {
    if matches!(error, rrd_query::Error::Budget(_)) && Instant::now() >= deadline {
        ServiceError::DeadlineExceeded
    } else {
        error.into()
    }
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn limits(policy: &ColdHistoryPolicy, deadline: Instant) -> Result<ParquetExportLimits> {
    Ok(ParquetExportLimits {
        max_rows: native(policy.max_changes)?,
        max_columns: 14,
        max_input_bytes: native(policy.max_input_bytes)?,
        max_output_bytes: native(policy.max_output_bytes)?,
        row_group_rows: native(policy.row_group_rows)?,
        max_memory_bytes: native(policy.max_memory_bytes)?,
        deadline,
    })
}

fn encoding(policy: &ColdHistoryPolicy) -> ParquetEncoding {
    ParquetEncoding {
        compression: match policy.compression {
            ColdHistoryCompression::Uncompressed => ParquetCompression::Uncompressed,
            ColdHistoryCompression::Zstd { level } => ParquetCompression::Zstd {
                level: i32::from(level),
            },
        },
        dictionary: policy.dictionary,
        statistics: policy.statistics,
    }
}

fn encode(
    stamp: &ReadStamp,
    page: &rrd_core::RuntimeChangePage,
    policy: &ColdHistoryPolicy,
    deadline: Instant,
) -> Result<rrd_query::ColdHistoryPage> {
    encode_cold_history_page(stamp, page, limits(policy, deadline)?, encoding(policy))
        .map_err(|error| cold_query_error(error, deadline))
}

fn source_digest(stamp: &ReadStamp, page: &rrd_core::RuntimeChangePage) -> String {
    source_digest_parts(
        stamp,
        page.requested_after,
        page.through_cursor,
        page.head_cursor,
        &page.changes,
    )
}

fn source_digest_parts(
    stamp: &ReadStamp,
    requested_after: u64,
    through_cursor: u64,
    head_cursor: u64,
    changes: &[RuntimeChange],
) -> String {
    let mut hash = digest::Sha256::new();
    hash.update(b"rrflow-cold-history-page-v1\0");
    hash.update(stamp.manifest_id.as_bytes());
    for cursor in [requested_after, through_cursor, head_cursor] {
        hash.update(&cursor.to_be_bytes());
    }
    for change in changes {
        hash.update(&change.cursor.to_be_bytes());
        hash.update(change.digest.as_bytes());
    }
    hash.finalize_hex()
}

struct CountOutput {
    limit: u64,
    used: u64,
}
impl Write for CountOutput {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let next = self
            .used
            .checked_add(bytes.len() as u64)
            .ok_or_else(|| std::io::Error::other("output length overflow"))?;
        if next > self.limit {
            return Err(std::io::Error::other("output limit exceeded"));
        }
        self.used = next;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn transition<T: Serialize>(
    key: &str,
    expected: Option<Vec<u8>>,
    replacement: &T,
    now: u64,
    action: &str,
) -> Result<ControlTransition> {
    Ok(transition_bytes(
        key,
        expected,
        serde_json::to_vec(replacement).map_err(contract_json)?,
        now,
        action,
    ))
}

fn transition_bytes(
    key: &str,
    expected: Option<Vec<u8>>,
    replacement: Vec<u8>,
    now: u64,
    action: &str,
) -> ControlTransition {
    ControlTransition {
        key: key.into(),
        expected,
        replacement: Some(replacement),
        at: now,
        actor: "system:maintenance".into(),
        action: action.into(),
        request_id: "cold-history-maintenance".into(),
        operation_id: "cold-history-maintenance".into(),
    }
}

fn check_deadline(deadline: Instant) -> Result<()> {
    if Instant::now() >= deadline {
        Err(ServiceError::DeadlineExceeded)
    } else {
        Ok(())
    }
}

fn io_error(error: std::io::Error) -> ServiceError {
    ServiceError::Storage(format!("cold history file operation failed: {error}"))
}

fn verify_head(
    engine: &RrdEngine,
    root: &AnchoredPath,
    state: &ColdState,
    deadline: Instant,
) -> Result<()> {
    let key = format!(
        "server/state/{}/cold-history/generation/{:020}",
        engine.instance, state.generation
    );
    let bytes = engine
        .storage
        .control()
        .get(&key)?
        .ok_or_else(|| ServiceError::Storage("cold history head receipt is missing".into()))?;
    if Some(digest::sha256_hex(&bytes).as_str()) != state.last_receipt_sha256.as_deref() {
        return Err(ServiceError::Storage(
            "cold history head receipt digest mismatch".into(),
        ));
    }
    let receipt: ColdReceipt = serde_json::from_slice(&bytes).map_err(contract_json)?;
    if receipt.version != COLD_STATE_VERSION
        || receipt.generation != state.generation
        || receipt.pending.through_cursor != state.through_cursor
        || !valid_sha256(&receipt.pending.artifact_sha256)
        || receipt.pending.artifact_bytes == 0
        || receipt.pending.artifact_bytes > 1_073_741_824
        || (receipt.generation == 1) != receipt.previous_receipt_sha256.is_none()
        || receipt
            .previous_receipt_sha256
            .as_ref()
            .is_some_and(|hash| !valid_sha256(hash))
    {
        return Err(ServiceError::Storage(
            "cold history head receipt is invalid".into(),
        ));
    }
    let file = root
        .join("cold-history")
        .join(format!("{}.parquet", receipt.pending.artifact_sha256));
    verify_file(
        &file,
        &receipt.pending.artifact_sha256,
        native(receipt.pending.artifact_bytes)?,
        deadline,
    )
}

fn verify_file(
    path: &AnchoredPath,
    expected: &str,
    expected_bytes: usize,
    deadline: Instant,
) -> Result<()> {
    let mut file = path.open_read().map_err(io_error)?;
    if file.metadata().map_err(io_error)?.len() != expected_bytes as u64 {
        return Err(ServiceError::Storage(
            "cold history artifact length mismatch".into(),
        ));
    }
    let mut hash = digest::Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        check_deadline(deadline)?;
        let count = file.read(&mut buffer).map_err(io_error)?;
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
    }
    if hash.finalize_hex() != expected {
        return Err(ServiceError::Storage(
            "cold history artifact digest mismatch".into(),
        ));
    }
    Ok(())
}

fn publish_file(
    directory: &AnchoredPath,
    final_path: &AnchoredPath,
    digest: &str,
    bytes: &[u8],
    deadline: Instant,
) -> Result<()> {
    if final_path.try_exists().map_err(io_error)? {
        return verify_file(final_path, digest, bytes.len(), deadline);
    }
    let staging = directory.join(format!(".{digest}.tmp"));
    if staging.try_exists().map_err(io_error)? {
        staging.remove_file().map_err(io_error)?;
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&staging)
        .map_err(io_error)?;
    file.write_all(bytes).map_err(io_error)?;
    check_deadline(deadline)?;
    file.sync_all().map_err(io_error)?;
    drop(file);
    check_deadline(deadline)?;
    staging.rename_new_to(final_path).map_err(io_error)?;
    verify_file(final_path, digest, bytes.len(), deadline)
}

#[cfg(test)]
mod tests {
    use super::*;
    use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
    use rrd_contract::ColdHistoryCompression;
    use rrd_core::{RuntimeLogicalModel, RuntimeRecord, RuntimeSchemaRegistry, RuntimeTableSchema};

    fn policy() -> ColdHistoryPolicy {
        ColdHistoryPolicy {
            interval_ms: 1_000,
            max_changes: 16,
            max_input_bytes: 1024 * 1024,
            max_output_bytes: 1024 * 1024,
            max_memory_bytes: 64 * 1024 * 1024,
            max_elapsed_ms: 30_000,
            row_group_rows: 2,
            compression: ColdHistoryCompression::Zstd { level: 3 },
            dictionary: true,
            statistics: true,
        }
    }

    #[test]
    fn policy_minimum_equals_the_encoder_reservation_for_a_full_page() {
        let policy = ColdHistoryPolicy {
            interval_ms: 1_000,
            max_changes: 512,
            max_input_bytes: 8 * 1024 * 1024,
            max_output_bytes: 8 * 1024 * 1024,
            max_memory_bytes: 4_294_967_296,
            max_elapsed_ms: 30_000,
            row_group_rows: 64,
            compression: ColdHistoryCompression::Zstd { level: 3 },
            dictionary: true,
            statistics: true,
        };
        const COLUMNS: usize = 15;
        let reservation = rrd_query::parquet_working_set_reservation(
            policy.max_input_bytes as usize,
            policy.max_changes as usize * COLUMNS,
            policy.max_output_bytes as usize,
            COLUMNS,
        )
        .unwrap();
        assert_eq!(policy.minimum_working_memory_bytes(), reservation as u64);
    }

    fn open(path: &Path) -> RrdEngine {
        RrdEngine::open(path, CanonicalId::new("cold").unwrap(), [7; 32]).unwrap()
    }

    fn seed(engine: &RrdEngine) {
        let mut registry = RuntimeSchemaRegistry::empty(1, "cold maintenance fixture");
        registry.tables.insert(
            RuntimeType::new("note").unwrap(),
            RuntimeTableSchema::schemaless(RuntimeLogicalModel::Relational),
        );
        engine
            .storage
            .runtime()
            .commit(&RuntimeCommit {
                scope: ScopeId::new("instance:cold").unwrap(),
                at: 10,
                actor: "cold-test".into(),
                expected_cursor: 0,
                mutations: vec![
                    RuntimeMutation::Schema { registry },
                    RuntimeMutation::Record {
                        record: RuntimeRecord {
                            reference: RuntimeRef::new("note", "one").unwrap(),
                            valid_from: 10,
                            valid_to: None,
                            properties: BTreeMap::from([(
                                "body".into(),
                                RuntimeValue::String("before".into()),
                            )]),
                        },
                    },
                ],
            })
            .unwrap();
    }

    fn state(engine: &RrdEngine) -> ColdState {
        let bytes = engine
            .storage
            .control()
            .get("server/state/cold/cold-history/head")
            .unwrap()
            .unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[test]
    fn prepared_history_replays_after_restart_and_new_writes_then_detects_corruption() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("estate");
        let engine = open(&path);
        seed(&engine);
        let policy = policy();
        assert!(engine.maintain_cold_history_at(&policy, 100).unwrap());
        let prepared = state(&engine);
        let pending = prepared.pending.as_ref().unwrap();
        assert_eq!(prepared.through_cursor, 0);
        let original = engine
            .storage
            .runtime()
            .read_changes(
                &pending.stamp,
                pending.requested_after,
                pending.scan_limit as usize,
            )
            .unwrap()
            .changes;
        let directory = engine
            .storage
            .as_rrflow_kv()
            .unwrap()
            .anchored_path()
            .join("cold-history");
        directory.create_dir_all().unwrap();
        let stage = directory.join(format!(".{}.tmp", pending.artifact_sha256));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&stage)
            .unwrap();
        file.write_all(b"interrupted stage").unwrap();
        file.sync_all().unwrap();
        drop(file);
        engine
            .storage
            .runtime()
            .commit(&RuntimeCommit {
                scope: ScopeId::new("instance:cold").unwrap(),
                at: 20,
                actor: "cold-test".into(),
                expected_cursor: 2,
                mutations: vec![RuntimeMutation::Record {
                    record: RuntimeRecord {
                        reference: RuntimeRef::new("note", "two").unwrap(),
                        valid_from: 20,
                        valid_to: None,
                        properties: BTreeMap::from([(
                            "body".into(),
                            RuntimeValue::String("after".into()),
                        )]),
                    },
                }],
            })
            .unwrap();
        drop(engine);
        let engine = open(&path);
        assert!(engine.maintain_cold_history_at(&policy, 101).unwrap());
        let published = state(&engine);
        assert_eq!(published.generation, 1);
        assert_eq!(published.through_cursor, pending.through_cursor);
        assert!(published.pending.is_none());
        assert!(!stage.try_exists().unwrap());
        let key = "server/state/cold/cold-history/generation/00000000000000000001";
        let receipt: ColdReceipt =
            serde_json::from_slice(&engine.storage.control().get(key).unwrap().unwrap()).unwrap();
        let file = directory
            .join(format!("{}.parquet", receipt.pending.artifact_sha256))
            .open_read()
            .unwrap();
        let reader = ParquetRecordBatchReaderBuilder::try_new(file)
            .unwrap()
            .build()
            .unwrap();
        let decoded = reader
            .flat_map(|batch| rrd_query::record_batch_to_rows(&batch.unwrap()).unwrap())
            .map(|row| {
                let RuntimeValue::String(json) = &row.values["change_json"] else {
                    panic!("history value")
                };
                serde_json::from_str::<RuntimeChange>(json).unwrap()
            })
            .collect::<Vec<_>>();
        assert_eq!(decoded, original);
        assert!(engine.maintain_cold_history_at(&policy, 102).unwrap());
        assert!(engine.maintain_cold_history_at(&policy, 103).unwrap());
        assert_eq!(state(&engine).through_cursor, 3);
        let last = state(&engine);
        let key = format!(
            "server/state/cold/cold-history/generation/{:020}",
            last.generation
        );
        let receipt: ColdReceipt =
            serde_json::from_slice(&engine.storage.control().get(&key).unwrap().unwrap()).unwrap();
        let file = directory.join(format!("{}.parquet", receipt.pending.artifact_sha256));
        let mut writer = OpenOptions::new().write(true).open(&file).unwrap();
        writer.write_all(b"corrupt").unwrap();
        writer.sync_all().unwrap();
        engine
            .storage
            .runtime()
            .commit(&RuntimeCommit {
                scope: ScopeId::new("instance:cold").unwrap(),
                at: 30,
                actor: "cold-test".into(),
                expected_cursor: 3,
                mutations: vec![RuntimeMutation::Record {
                    record: RuntimeRecord {
                        reference: RuntimeRef::new("note", "three").unwrap(),
                        valid_from: 30,
                        valid_to: None,
                        properties: BTreeMap::new(),
                    },
                }],
            })
            .unwrap();
        assert!(engine.maintain_cold_history_at(&policy, 104).is_err());
        assert_eq!(state(&engine).through_cursor, 3);
    }

    #[test]
    fn changed_policy_replans_pending_work_without_rewinding_committed_cursor() {
        let root = tempfile::tempdir().unwrap();
        let engine = open(&root.path().join("estate"));
        seed(&engine);
        let first = policy();
        assert!(engine.maintain_cold_history_at(&first, 100).unwrap());
        let mut changed = first;
        changed.compression = ColdHistoryCompression::Zstd { level: 4 };
        assert!(engine.maintain_cold_history_at(&changed, 101).unwrap());
        assert!(state(&engine).pending.is_none());
        assert_eq!(state(&engine).through_cursor, 0);
        assert!(engine.maintain_cold_history_at(&changed, 102).unwrap());
        assert!(engine.maintain_cold_history_at(&changed, 103).unwrap());
        assert_eq!(state(&engine).through_cursor, 2);
    }

    #[test]
    fn changed_catalogue_replans_a_stale_stamp_without_losing_progress() {
        let root = tempfile::tempdir().unwrap();
        let engine = open(&root.path().join("estate"));
        seed(&engine);
        let policy = policy();
        assert!(engine.maintain_cold_history_at(&policy, 100).unwrap());
        let old_manifest = state(&engine).pending.unwrap().stamp.manifest_id;
        let scope = ScopeId::new("instance:cold").unwrap();
        engine
            .storage
            .control()
            .commit_catalog(
                &scope,
                &ControlTransition {
                    key: "server/state/cold/test-catalogue".into(),
                    expected: None,
                    replacement: Some(b"updated".to_vec()),
                    at: 101,
                    actor: "cold-test".into(),
                    action: "test.catalogue_changed".into(),
                    request_id: "cold-test".into(),
                    operation_id: "cold-test".into(),
                },
            )
            .unwrap();
        assert!(engine.maintain_cold_history_at(&policy, 102).unwrap());
        assert!(state(&engine).pending.is_none());
        assert_eq!(state(&engine).through_cursor, 0);
        assert!(engine.maintain_cold_history_at(&policy, 103).unwrap());
        assert_ne!(
            state(&engine).pending.unwrap().stamp.manifest_id,
            old_manifest
        );
        assert!(engine.maintain_cold_history_at(&policy, 104).unwrap());
        assert_eq!(state(&engine).through_cursor, 2);
    }

    #[test]
    fn authenticated_generation_readback_reopens_and_rejects_corruption_and_limits() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("estate");
        let mut engine = open(&path);
        seed(&engine);
        let policy = policy();
        engine.estate_configuration.storage.cold_history = Some(policy.clone());
        for _ in 0..64 {
            engine.maintain_storage().unwrap();
            if let Some(bytes) = engine
                .storage
                .control()
                .get("server/state/cold/cold-history/head")
                .unwrap()
            {
                let state: ColdState = serde_json::from_slice(&bytes).unwrap();
                if state.generation == 1 {
                    break;
                }
            }
        }
        assert_eq!(state(&engine).generation, 1);
        drop(engine);
        let mut engine = open(&path);
        engine.estate_configuration.storage.cold_history = Some(policy);
        let session = engine
            .create_session(
                &rrd_contract::CreateSession {
                    limits: rrd_contract::SessionLimits {
                        idle_timeout_ms: 5_000,
                        absolute_timeout_ms: 10_000,
                        max_open_transactions: 2,
                    },
                },
                &CorrelationId::new("cold-read-session").unwrap(),
                1_000,
                "create",
                "create",
            )
            .unwrap();
        let mut request = ReadColdHistory {
            scope: "instance:cold".into(),
            generation: None,
            max_artifact_bytes: 1024 * 1024,
            budget: rrd_contract::QueryBudget::default(),
        };
        let read = |request: &ReadColdHistory| {
            engine.read_cold_history(
                &session.session_id,
                &session.token,
                request,
                1_100,
                "read",
                "read",
            )
        };
        let status = read(&request).unwrap();
        assert!(status.enabled);
        assert_eq!(status.current_generation, 1);
        assert_eq!(status.through_cursor, 2);
        assert!(status.generation.is_none());
        request.generation = Some(1);
        let result = read(&request).unwrap();
        assert_eq!(result.changes.len(), 2);
        let artifact = result.generation.unwrap();
        assert_eq!(artifact.through_cursor, 2);
        let mut restricted = request.clone();
        restricted.max_artifact_bytes = 1;
        assert!(read(&restricted).is_err());
        restricted = request.clone();
        restricted.scope = "instance:other".into();
        assert!(read(&restricted).is_err());
        assert!(engine
            .read_cold_history(
                &session.session_id,
                &CorrelationId::new("wrong-token").unwrap(),
                &request,
                1_100,
                "read",
                "read",
            )
            .is_err());
        let path = engine
            .storage
            .as_rrflow_kv()
            .unwrap()
            .anchored_path()
            .join("cold-history")
            .join(format!("{}.parquet", artifact.artifact_sha256));
        let mut file = OpenOptions::new().write(true).open(&path).unwrap();
        file.write_all(b"corrupt").unwrap();
        file.sync_all().unwrap();
        assert!(read(&request).is_err());
        assert_eq!(state(&engine).through_cursor, 2);
    }

    #[test]
    fn publication_waits_for_a_foreground_transaction() {
        // A foreground commit and publication both append to the control
        // journal. Without the gate, they meet as a physical write conflict
        // that clients see as a retryable runtime conflict at an unchanged cursor.
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("estate");
        let mut engine = open(&path);
        seed(&engine);
        engine.estate_configuration.storage.cold_history = Some(policy());
        let before = engine.storage.control().sequence().unwrap();
        let foreground = engine
            .admit_transaction(&"foreground", "request-foreground", "operation-foreground")
            .unwrap();
        std::thread::scope(|scope| {
            let publication = scope.spawn(|| engine.maintain_cold_history());
            std::thread::sleep(std::time::Duration::from_millis(300));
            assert_eq!(
                engine.storage.control().sequence().unwrap(),
                before,
                "publication committed while a foreground transaction held the gate"
            );
            drop(foreground);
            publication.join().unwrap().unwrap();
        });
        assert!(
            engine.storage.control().sequence().unwrap() > before,
            "publication proceeds once the gate is free"
        );
    }

    #[test]
    fn maintenance_failure_is_durable_deduplicated_and_cleared_by_progress() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("estate");
        let mut engine = open(&path);
        seed(&engine);
        let mut too_small = policy();
        too_small.max_input_bytes = 1;
        engine.estate_configuration.storage.cold_history = Some(too_small.clone());
        for _ in 0..64 {
            if engine.maintain_storage().is_err() {
                break;
            }
        }
        assert_eq!(
            state(&engine).failure.unwrap().code,
            ColdHistoryFailureCode::ResourceLimit
        );
        let sequence = engine.storage.control().sequence().unwrap();
        assert!(engine.maintain_storage().is_err());
        assert_eq!(engine.storage.control().sequence().unwrap(), sequence);
        drop(engine);

        let mut engine = open(&path);
        engine.estate_configuration.storage.cold_history = Some(too_small);
        let session = engine
            .create_session(
                &rrd_contract::CreateSession {
                    limits: rrd_contract::SessionLimits {
                        idle_timeout_ms: 5_000,
                        absolute_timeout_ms: 10_000,
                        max_open_transactions: 2,
                    },
                },
                &CorrelationId::new("cold-failure-session").unwrap(),
                1_000,
                "create",
                "create",
            )
            .unwrap();
        let request = ReadColdHistory {
            scope: "instance:cold".into(),
            generation: None,
            max_artifact_bytes: 1024 * 1024,
            budget: rrd_contract::QueryBudget::default(),
        };
        let status = engine
            .read_cold_history(
                &session.session_id,
                &session.token,
                &request,
                1_100,
                "read",
                "read",
            )
            .unwrap();
        assert_eq!(
            status.failure.unwrap().code,
            ColdHistoryFailureCode::ResourceLimit
        );
        engine.estate_configuration.storage.cold_history = Some(policy());
        for _ in 0..64 {
            engine.maintain_storage().unwrap();
            if state(&engine).generation == 1 {
                break;
            }
        }
        assert_eq!(state(&engine).generation, 1);
        let recovered = engine
            .read_cold_history(
                &session.session_id,
                &session.token,
                &request,
                1_100,
                "read",
                "read",
            )
            .unwrap();
        assert!(recovered.failure.is_none());
    }

    #[test]
    fn encoded_output_limit_is_reported_as_a_resource_failure() {
        let root = tempfile::tempdir().unwrap();
        let mut engine = open(&root.path().join("estate"));
        seed(&engine);
        let mut limited = policy();
        limited.max_output_bytes = 1;
        engine.estate_configuration.storage.cold_history = Some(limited);

        let error = engine.maintain_cold_history().unwrap_err();
        assert_eq!(error.kind(), ServiceErrorKind::ResourceExhausted);
        let recorded = state(&engine);
        assert_eq!(recorded.generation, 0);
        assert!(recorded.pending.is_none());
        assert_eq!(
            recorded.failure.unwrap().code,
            ColdHistoryFailureCode::ResourceLimit
        );
    }
}
