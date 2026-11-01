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

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CapacityState {
    format_version: u16,
    outstanding: BTreeMap<String, CapacityLease>,
}

impl RrdEngine {
    pub(in crate::engine) fn endpoint_capacity_transition(
        &self,
        lease: &CapacityLease,
        limits: Option<(u32, u32)>,
    ) -> Result<AttemptControlWrite> {
        let key = format!("server/state/{}/endpoint-proposal/capacity", self.instance);
        let expected = self.storage.control().get(&key)?;
        // Bound decoding independently of the policy that might have changed.
        if expected.as_ref().is_some_and(|bytes| bytes.len() > 262_144) {
            return Err(ServiceError::Storage(
                "controller capacity state is oversized".into(),
            ));
        }
        let mut state = match &expected {
            Some(bytes) => serde_json::from_slice::<CapacityState>(bytes).map_err(contract_json)?,
            None => CapacityState {
                format_version: 1,
                outstanding: BTreeMap::new(),
            },
        };
        if state.format_version != 1 || state.outstanding.len() > rrd_contract::MAX_PROFILE_BINDINGS
        {
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
        let id = capacity_identity(lease)?;
        if let Some((aggregate, binding)) = limits {
            if state.outstanding.contains_key(&id) {
                return Err(ServiceError::ReasoningAttemptRecorded {
                    outcome_sha256: None,
                });
            }
            if state.outstanding.len() >= aggregate as usize
                || state.outstanding.len() >= rrd_contract::MAX_PROFILE_BINDINGS
                || state
                    .outstanding
                    .values()
                    .filter(|entry| entry.controller_binding_id == lease.controller_binding_id)
                    .count()
                    >= binding as usize
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
}

fn capacity_identity(lease: &CapacityLease) -> Result<String> {
    operation_digest(&(
        &lease.request.scope,
        &lease.request.task_id,
        &lease.request.attempt_id,
    ))
}
