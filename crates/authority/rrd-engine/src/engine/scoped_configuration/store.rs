use super::super::token_key::{create_private_directory_anchored, pin_private_directory};
use super::*;
use rrd_store::filesystem::AnchoredPath;

const HEAD: &str = "server/state/user-configuration/head";

pub(super) struct UserStore {
    pub database: RrflowKvStore,
    pub head_bytes: Option<Vec<u8>>,
    pub head: Option<UserConfigurationRevision>,
}

impl UserStore {
    pub fn open(binding: &UserConfigurationBinding, create: bool) -> Result<Self> {
        checked(binding.validate())?;
        let path = AnchoredPath::from_path(Path::new(&binding.root)).map_err(|error| {
            unavailable(format!("user configuration root unavailable: {error}"))
        })?;
        let root = if create && !path.try_exists().map_err(io_error)? {
            create_private_directory_anchored(&path).map_err(io_error)?
        } else {
            pin_private_directory(&path).map_err(io_error)?
        };
        let data = root.join("configuration.rrkv");
        let database = if create && !data.try_exists().map_err(io_error)? {
            RrflowKvStore::create_new_anchored(&data, rrd_store::DatabaseOptions::default())
        } else {
            RrflowKvStore::open_existing_anchored(&data, rrd_store::DatabaseOptions::default())
        }
        .map_err(|error| {
            unavailable(format!(
                "required user configuration store unavailable: {error}"
            ))
        })?;
        let head_bytes = database.control().get(HEAD)?;
        let head = head_bytes.as_deref().map(decode).transpose()?;
        if let Some(head) = &head {
            if head.source_id != binding.source_id {
                return Err(ServiceError::ProjectBindingMismatch);
            }
        } else if !create {
            return Err(unavailable(
                "required user configuration has no published revision",
            ));
        }
        Ok(Self {
            database,
            head_bytes,
            head,
        })
    }

    pub fn select(
        &self,
        binding: &UserConfigurationBinding,
        valid_at: u64,
        recorded_revision: Option<u64>,
    ) -> Result<(u64, UserConfigurationRevision)> {
        let mut revision = self
            .head
            .clone()
            .ok_or_else(|| unavailable("user configuration has no published revision"))?;
        let known = recorded_revision.unwrap_or(revision.revision);
        if known == 0 || known > revision.revision {
            return Err(unavailable(
                "requested user configuration revision is unavailable",
            ));
        }
        for read in 0..binding.max_history_reads {
            if revision.revision <= known && revision.change.contains(valid_at) {
                return Ok((known, revision));
            }
            if revision.revision == 1 {
                return Err(unavailable(
                    "no user configuration covers the requested valid time",
                ));
            }
            if read + 1 == binding.max_history_reads {
                break;
            }
            let bytes = self
                .database
                .control()
                .get(&revision_key(revision.revision - 1))?
                .ok_or_else(|| unavailable("user configuration history is incomplete"))?;
            let previous = decode(&bytes)?;
            if previous.source_id != binding.source_id
                || previous.revision + 1 != revision.revision
                || Some(&previous.revision_sha256) != revision.previous_sha256.as_ref()
                || previous.recorded_at_unix_ms > revision.recorded_at_unix_ms
            {
                return Err(unavailable(
                    "user configuration history identity is inconsistent",
                ));
            }
            revision = previous;
        }
        Err(unavailable(
            "user configuration history read budget exhausted",
        ))
    }

    pub fn publish(&self, revision: &UserConfigurationRevision) -> Result<()> {
        checked(revision.validate())?;
        let bytes = serde_json::to_vec(revision).map_err(contract_json)?;
        let transition = |key, expected| ControlTransition {
            key,
            expected,
            replacement: Some(bytes.clone()),
            at: revision.recorded_at_unix_ms,
            actor: "local-os-owner".into(),
            action: "user.configuration.applied".into(),
            request_id: revision.applied_plan_sha256.clone(),
            operation_id: revision.applied_plan_sha256.clone(),
        };
        self.database.control().commit_batch(&[
            transition(HEAD.into(), self.head_bytes.clone()),
            transition(revision_key(revision.revision), None),
        ])?;
        Ok(())
    }
}

fn revision_key(revision: u64) -> String {
    format!("server/state/user-configuration/revision/{revision:020}")
}

fn decode(bytes: &[u8]) -> Result<UserConfigurationRevision> {
    if bytes.len() > 64 * 1024 {
        return Err(unavailable("oversized user configuration revision"));
    }
    let revision: UserConfigurationRevision =
        serde_json::from_slice(bytes).map_err(contract_json)?;
    checked(revision.validate())?;
    Ok(revision)
}

fn io_error(error: std::io::Error) -> ServiceError {
    unavailable(format!(
        "private user configuration directory unavailable: {error}"
    ))
}
