//! Durable outstanding calls are admission state in the task transaction.
//! Unknown remote execution keeps its entry; reopening never frees a permit.
use super::*;
use crate::engine::reasoning_budget::AttemptControlWrite;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::engine) struct CapacityLease {
    pub request: EndpointSelectionRequest,
    pub controller_binding_id: CanonicalId,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::engine) struct WorkerCapacityLease {
    pub scope: String,
    pub task_id: CanonicalId,
    pub attempt_id: CorrelationId,
    pub binding_id: CanonicalId,
}

impl WorkerCapacityLease {
    fn identity(&self) -> Result<String> {
        operation_digest(&(&self.scope, &self.task_id, &self.attempt_id))
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CapacityState {
    format_version: u16,
    outstanding: BTreeMap<String, CapacityLease>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    workers: BTreeMap<String, WorkerCapacityLease>,
}

impl RrdEngine {
    pub(in crate::engine) fn endpoint_capacity_transition(
        &self,
        lease: &CapacityLease,
        limits: Option<(u32, u32)>,
    ) -> Result<AttemptControlWrite> {
        let (key, expected, mut state) = self.load_endpoint_capacity()?;
        let id = capacity_identity(lease)?;
        if let Some((aggregate, binding)) = limits {
            if state.outstanding.contains_key(&id) || state.workers.contains_key(&id) {
                return Err(ServiceError::ReasoningAttemptRecorded {
                    outcome_sha256: None,
                });
            }
            if state.outstanding.len() >= aggregate as usize
                || state.len() >= rrd_contract::MAX_PROFILE_BINDINGS
                || state.binding_count(&lease.controller_binding_id) >= binding as usize
            {
                return Err(ServiceError::InferenceBusy);
            }
            state.outstanding.insert(id, lease.clone());
        } else {
            if state.outstanding.get(&id) != Some(lease) {
                return Err(ServiceError::StorageConflict(
                    "controller capacity ownership changed".into(),
                ));
            }
            state.outstanding.remove(&id);
        }
        capacity_write(key, expected, state)
    }

    pub(in crate::engine) fn worker_capacity_transition(
        &self,
        lease: &WorkerCapacityLease,
        limits: Option<(u32, u32)>,
    ) -> Result<AttemptControlWrite> {
        let (key, expected, mut state) = self.load_endpoint_capacity()?;
        let id = lease.identity()?;
        if let Some((aggregate, binding)) = limits {
            if state.outstanding.contains_key(&id) || state.workers.contains_key(&id) {
                return Err(ServiceError::ReasoningAttemptRecorded {
                    outcome_sha256: None,
                });
            }
            if state.workers.len() >= aggregate as usize
                || state.len() >= rrd_contract::MAX_PROFILE_BINDINGS
                || state.binding_count(&lease.binding_id) >= binding as usize
            {
                return Err(ServiceError::InferenceBusy);
            }
            state.workers.insert(id, lease.clone());
        } else {
            if state.workers.get(&id) != Some(lease) {
                return Err(ServiceError::StorageConflict(
                    "worker capacity ownership changed".into(),
                ));
            }
            state.workers.remove(&id);
        }
        capacity_write(key, expected, state)
    }

    fn load_endpoint_capacity(&self) -> Result<(String, Option<Vec<u8>>, CapacityState)> {
        let key = format!("server/state/{}/endpoint-proposal/capacity", self.instance);
        let expected = self.storage.control().get(&key)?;
        // Bound decoding independently of the policy that might have changed.
        if expected.as_ref().is_some_and(|bytes| bytes.len() > 262_144) {
            return Err(ServiceError::Storage(
                "controller capacity state is oversized".into(),
            ));
        }
        let state = match &expected {
            Some(bytes) => serde_json::from_slice::<CapacityState>(bytes).map_err(contract_json)?,
            None => CapacityState {
                format_version: 1,
                outstanding: BTreeMap::new(),
                workers: BTreeMap::new(),
            },
        };
        if state.format_version != 1 || state.len() > rrd_contract::MAX_PROFILE_BINDINGS {
            return Err(ServiceError::Storage(
                "controller capacity state is corrupt".into(),
            ));
        }
        for (id, stored) in &state.outstanding {
            stored
                .request
                .validate()
                .map_err(|e| ServiceError::Storage(e.to_string()))?;
            if *id != capacity_identity(stored)? {
                return Err(ServiceError::Storage(
                    "controller capacity identity is corrupt".into(),
                ));
            }
        }
        for (id, stored) in &state.workers {
            rrd_contract::ReadTaskAttempt {
                scope: stored.scope.clone(),
                task_id: stored.task_id.clone(),
                attempt_id: stored.attempt_id.clone(),
            }
            .validate()
            .map_err(|e| ServiceError::Storage(e.to_string()))?;
            if *id != stored.identity()? || state.outstanding.contains_key(id) {
                return Err(ServiceError::Storage(
                    "worker capacity identity is corrupt".into(),
                ));
            }
        }
        Ok((key, expected, state))
    }
}

impl CapacityState {
    fn len(&self) -> usize {
        self.outstanding.len() + self.workers.len()
    }
    fn binding_count(&self, binding_id: &CanonicalId) -> usize {
        self.outstanding
            .values()
            .filter(|entry| &entry.controller_binding_id == binding_id)
            .count()
            + self
                .workers
                .values()
                .filter(|entry| &entry.binding_id == binding_id)
                .count()
    }
}

fn capacity_write(
    key: String,
    expected: Option<Vec<u8>>,
    state: CapacityState,
) -> Result<AttemptControlWrite> {
    let replacement = serde_json::to_vec(&state).map_err(contract_json)?;
    if replacement.len() > 262_144 {
        return Err(ServiceError::ConfigurationLimit(
            "controller capacity state limit".into(),
        ));
    }
    Ok(AttemptControlWrite {
        key,
        expected,
        replacement,
    })
}

fn capacity_identity(lease: &CapacityLease) -> Result<String> {
    operation_digest(&(
        &lease.request.scope,
        &lease.request.task_id,
        &lease.request.attempt_id,
    ))
}
