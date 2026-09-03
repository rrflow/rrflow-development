//! Engine-owned embedded and offline operator operations.
//!
//! This module is private: outward adapters receive [`crate::RrdEngine`] and
//! engine-owned value types, never a second storage-opening handle or an
//! `rrd_store::Engine` escape.

use crate::{load_or_create_token_key, InstanceBinding, RrdEngine, ServiceError};
use rrd_contract::CanonicalId;
use rrd_core::{RuntimeCommit, RuntimeMutation};
use rrd_store::Engine as _;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub use rrd_core::{
    digest, Claim, ClaimReader, Millis, Predicate, Producer, Reader, ReasoningPayload, ScopeId,
    Subject,
};
pub use rrd_store::{
    BackupCatalogue, BackupEntry, FormatMigrationEdge, FormatMigrationLedger,
    Invocation as OperatorInvocation, InvocationInput as OperatorInvocationInput,
    LogicalArchiveInventory, LogicalRestoreReport, MigrationReport, NativeApplicationFormat,
    Outcome, RemovalReport, Trigger,
};

pub type OperatorResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const LOCAL_TOKEN_KEY_FILE: &str = "RRD.SECRET";

impl RrdEngine {
    fn persistent_storage(&self) -> OperatorResult<&rrd_store::PersistentEngine> {
        self.storage
            .as_persistent()
            .ok_or_else(|| "operation requires a persistent RRD root".into())
    }

    /// Opens the canonical project-bound engine authority identified by an RRD
    /// store path. Only `<project>/.rrflow/rrd` is accepted.
    pub fn open_project_store(path: &Path) -> crate::Result<Self> {
        let absolute = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()
                .map_err(|error| ServiceError::Storage(error.to_string()))?
                .join(path)
        };
        let state_root = absolute
            .parent()
            .ok_or_else(|| ServiceError::Contract("RRD store path has no .rrflow parent".into()))?;
        if absolute.file_name().and_then(|value| value.to_str()) != Some("rrd")
            || state_root.file_name().and_then(|value| value.to_str()) != Some(".rrflow")
        {
            return Err(ServiceError::Contract(
                "embedded RRD must use the canonical <project>/.rrflow/rrd path".into(),
            ));
        }
        let project_root = state_root
            .parent()
            .ok_or_else(|| ServiceError::Contract("RRD store path has no project root".into()))?;
        let binding = InstanceBinding::discover(project_root)
            .map_err(|error| ServiceError::Contract(error.to_string()))?;
        binding
            .verify_store_path(&absolute)
            .map_err(|error| ServiceError::Contract(error.to_string()))?;
        Self::open_bound(&binding)
    }

    /// Opens the one embedded RRD authority bound to a discovered project.
    pub fn open_bound(binding: &InstanceBinding) -> crate::Result<Self> {
        binding
            .require_runtime_ready()
            .map_err(|error| ServiceError::Contract(error.to_string()))?;
        let instance = CanonicalId::new(binding.manifest.id.clone())
            .map_err(|error| ServiceError::Contract(error.to_string()))?;
        let database = binding.expected_store();
        let mut engine = Self::open(&database, instance, [0_u8; 32])?;
        engine.token_key = load_or_create_token_key(&database.join(LOCAL_TOKEN_KEY_FILE))
            .map_err(|error| ServiceError::Storage(error.to_string()))?;
        engine.bind_project_authority(binding, wall_clock_millis())?;
        Ok(engine)
    }

    /// Opens a bound engine with caller-supplied token material.
    pub fn open_bound_with_token_key(
        binding: &InstanceBinding,
        instance: CanonicalId,
        token_key: [u8; 32],
        at: u64,
    ) -> crate::Result<Self> {
        binding
            .require_runtime_ready()
            .map_err(|error| ServiceError::Contract(error.to_string()))?;
        binding
            .verify_store_path(&binding.expected_store())
            .map_err(|error| ServiceError::Contract(error.to_string()))?;
        let engine = Self::open(&binding.expected_store(), instance, token_key)?;
        engine.bind_project_authority(binding, at)?;
        Ok(engine)
    }

    pub fn path(&self) -> OperatorResult<&Path> {
        self.storage_root
            .as_deref()
            .ok_or_else(|| "in-memory RRD has no persistent path".into())
    }

    pub fn backend_name(&self) -> &'static str {
        self.storage.backend_name()
    }

    pub fn record_operator_invocation(
        &self,
        input: OperatorInvocationInput<'_>,
    ) -> OperatorResult<OperatorInvocation> {
        Ok(self.persistent_storage()?.record_invocation(input)?)
    }

    pub fn invocations_since(&self, since: Millis) -> OperatorResult<Vec<OperatorInvocation>> {
        Ok(self.persistent_storage()?.invocations_since(since)?)
    }

    pub fn invocation_count(&self) -> OperatorResult<u64> {
        Ok(self.persistent_storage()?.invocation_count()?)
    }

    pub fn access_count(&self) -> OperatorResult<usize> {
        Ok(self.persistent_storage()?.access_count()?)
    }

    pub fn sequence(&self) -> OperatorResult<u64> {
        Ok(self.storage.sequence()?)
    }

    pub fn observe(
        &self,
        reader: &Reader,
        subject: &Subject,
        predicate: &Predicate,
        at: Millis,
    ) -> OperatorResult<()> {
        Ok(self.storage.observe(reader, subject, predicate, at)?)
    }

    pub fn assert_claim(&self, claim: &Claim) -> OperatorResult<rrd_store::AppendOutcome> {
        claim.validate()?;
        let _gate = self
            .transaction_gate
            .lock()
            .map_err(|_| -> Box<dyn std::error::Error> {
                "transaction gate lock is poisoned".into()
            })?;
        let scope = ScopeId::new(format!("instance:{}", self.instance_id()))?;
        let read = self.storage.runtime_read_stamp(&scope)?;
        let previous = self
            .storage
            .as_of(&claim.subject, &claim.predicate, claim.valid_from)?;
        let claims = match previous {
            Some(previous) if previous.valid_from < claim.valid_from => {
                rrd_core::supersede(&previous, claim.clone())?.to_vec()
            }
            _ => vec![claim.clone()],
        };
        let mut mutations =
            Vec::with_capacity(claims.len() + usize::from(read.schema_revision.is_none()));
        if read.schema_revision.is_none() {
            let mut schema = rrd_core::RuntimeSchemaRegistry::empty(
                1,
                "install canonical reasoning claim model",
            );
            schema.tables.insert(
                rrd_core::RuntimeType::new("claim")?,
                rrd_core::RuntimeTableSchema::strict(rrd_core::RuntimeLogicalModel::ReasoningClaim),
            );
            mutations.push(RuntimeMutation::Schema { registry: schema });
        }
        mutations.extend(
            claims
                .iter()
                .cloned()
                .map(|claim| RuntimeMutation::Claim { claim }),
        );
        let outcome = self.storage.commit_runtime(&RuntimeCommit {
            scope,
            at: claim.tx_time,
            actor: claim.producer.actor.clone(),
            expected_cursor: read.commit_cursor,
            mutations,
        })?;
        Ok(rrd_store::AppendOutcome {
            first_sequence: outcome
                .first_claim_sequence
                .ok_or("canonical claim commit did not assign a claim sequence")?,
            last_sequence: outcome
                .last_claim_sequence
                .ok_or("canonical claim commit did not assign a claim sequence")?,
            count: claims.len(),
        })
    }

    pub fn claim_as_of(
        &self,
        subject: &Subject,
        predicate: &Predicate,
        at: Millis,
    ) -> OperatorResult<Option<Claim>> {
        Ok(self.storage.as_of(subject, predicate, at)?)
    }

    pub fn claim_history(
        &self,
        subject: &Subject,
        predicate: &Predicate,
    ) -> OperatorResult<Vec<Claim>> {
        Ok(self.storage.history(subject, predicate)?)
    }

    pub fn removal_report(
        &self,
        since: Millis,
        evaluated_at: Millis,
    ) -> OperatorResult<RemovalReport> {
        Ok(self
            .persistent_storage()?
            .removal_report(since, evaluated_at)?)
    }

    pub fn export_logical_archive(
        &self,
        archive: &Path,
    ) -> OperatorResult<LogicalArchiveInventory> {
        Ok(rrd_store::export_logical_archive(&self.storage, archive)?)
    }

    pub fn create_logical_backup(
        &self,
        catalogue: &Path,
        label: &str,
        now: Millis,
    ) -> OperatorResult<BackupEntry> {
        Ok(rrd_store::create_logical_backup(
            &self.storage,
            catalogue,
            label,
            now,
        )?)
    }

    pub fn verify_project_store(&self, root: &Path) -> OperatorResult<()> {
        let binding = InstanceBinding::discover(root)?;
        binding.require_runtime_ready()?;
        binding.verify_store_path(self.path()?)?;
        if binding.manifest.id != self.instance_id().as_str() {
            return Err("engine instance identity does not match the project manifest".into());
        }
        Ok(())
    }

    pub fn execute_operator_query(
        &self,
        scope: ScopeId,
        source: &str,
        parameters: &crate::Parameters,
        budget: &crate::ExecutionBudget,
        actor: &str,
        at: Millis,
    ) -> OperatorResult<crate::TracedQueryExecution> {
        crate::execute_traced_query(&self.storage, scope, source, parameters, budget, actor, at)
    }

    pub fn record_reasoning_event(
        &self,
        run: &str,
        now: Millis,
        actor: &str,
        payload: ReasoningPayload,
    ) -> OperatorResult<rrd_core::ReasoningEvent> {
        crate::record_reasoning(&self.storage, run, now, actor, payload)
    }

    pub fn reasoning_run_by_id(&self, run: &str) -> OperatorResult<Option<rrd_core::ReasoningRun>> {
        crate::reasoning_run(&self.storage, run)
    }

    pub fn active_reasoning(&self) -> OperatorResult<Option<rrd_core::ReasoningRun>> {
        crate::active_reasoning_run(&self.storage)
    }

    pub fn migrate_storage(db: &Path, now: Millis) -> OperatorResult<MigrationReport> {
        Ok(rrd_store::migrate_fjall_to_native(db, now)?)
    }

    pub fn storage_migration_status(db: &Path) -> OperatorResult<Option<MigrationReport>> {
        Ok(rrd_store::migration_status(db)?)
    }

    pub fn rollback_storage_migration(db: &Path) -> OperatorResult<MigrationReport> {
        Ok(rrd_store::rollback_fjall_migration(db)?)
    }

    pub fn inspect_logical_archive(archive: &Path) -> OperatorResult<LogicalArchiveInventory> {
        Ok(rrd_store::inspect_logical_archive(archive)?)
    }

    pub fn restore_logical_archive(
        archive: &Path,
        db: &Path,
        now: Millis,
    ) -> OperatorResult<LogicalRestoreReport> {
        Ok(rrd_store::restore_logical_archive_to_new_root(
            archive, db, now,
        )?)
    }

    pub fn verify_backup_catalogue(catalogue: &Path) -> OperatorResult<BackupCatalogue> {
        Ok(rrd_store::verify_backup_catalogue(catalogue)?)
    }

    pub fn restore_catalogued_backup(
        catalogue: &Path,
        backup_id: &str,
        db: &Path,
        now: Millis,
    ) -> OperatorResult<LogicalRestoreReport> {
        Ok(rrd_store::restore_catalogued_backup(
            catalogue, backup_id, db, now,
        )?)
    }

    pub fn migrate_native_format(db: &Path, now: Millis) -> OperatorResult<FormatMigrationLedger> {
        Ok(rrd_store::migrate_native_format(db, now)?)
    }

    pub fn supported_native_format_migrations() -> &'static [FormatMigrationEdge] {
        &rrd_store::SUPPORTED_NATIVE_FORMAT_MIGRATIONS
    }

    pub fn rollback_native_format(db: &Path) -> OperatorResult<FormatMigrationLedger> {
        Ok(rrd_store::rollback_native_format(db)?)
    }

    pub fn native_format_migration_status(
        db: &Path,
    ) -> OperatorResult<Option<FormatMigrationLedger>> {
        Ok(rrd_store::native_format_migration_status(db)?)
    }

    pub fn canonical_project_root_for_store(path: &Path) -> OperatorResult<PathBuf> {
        let absolute = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()?.join(path)
        };
        let state = absolute
            .parent()
            .ok_or("RRD store has no state directory")?;
        if absolute.file_name().and_then(|value| value.to_str()) != Some("rrd")
            || state.file_name().and_then(|value| value.to_str()) != Some(".rrflow")
        {
            return Err("RRD store must be <project>/.rrflow/rrd".into());
        }
        Ok(std::fs::canonicalize(
            state.parent().ok_or("RRD store has no project root")?,
        )?)
    }
}

fn wall_clock_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
