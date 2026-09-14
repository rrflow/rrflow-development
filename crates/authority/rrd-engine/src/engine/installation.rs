use super::*;
use crate::engine::memory_estate::build_memory_estate_plan;
use crate::engine::token_key::{
    create_api_key_document, create_token_key, read_api_key_document, read_token_key,
    ApiKeyDocument,
};
use rrd_contract::{
    attunement_plan_sha256, installation_plan_sha256, installation_result_sha256, AttunementJob,
    AttunementJobState, AttunementPhase, AttunementPhasePlan, AttunementPlan,
    InstallationActionDisposition, InstallationActionKind, InstallationActionResult,
    InstallationManagedPath, InstallationManagedPathKind, InstallationPlan, InstallationPlanAction,
    InstallationRemovalRule, InstallationResult, InstallationTargetKind, MemorySeatDefinition,
    PersistMemoryEstate, ATTUNEMENT_PHASES, INSTALL_ATTUNEMENT_CONTRACT_VERSION,
};
use rrd_core::digest::Sha256 as IncrementalSha256;
use rrd_security::{Principal, PrincipalKind, ResourceGrant, SecurityState, SECURITY_FORMAT};
use rrd_store::{ControlTransition, RrflowKvInspector};
use serde::de::DeserializeOwned;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};

const INSTALL_PROFILE_BYTES: &[u8] = include_bytes!("../../assets/install/default-profile-v1.json");
const INSTALL_PROFILE_FORMAT: u16 = 1;
const LOCATOR_FORMAT: u16 = 1;
const INSTALLED_RECORD_FORMAT: u16 = 1;
const INSTALL_CHECKPOINT_FORMAT: u16 = 1;
const LOCATOR_RELATIVE_PATH: &str = ".rrflow/config.toml";
const INSTALLED_RECORD_KEY: &str = "server/state/install/record";
const INSTALL_CHECKPOINT_KEY: &str = "server/state/install/checkpoint";
const MAX_LOCATOR_BYTES: u64 = 64 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DefaultInstallProfile {
    pub format_version: u16,
    pub id: CanonicalId,
    pub initial_seat: MemorySeatDefinition,
    pub initial_principal_id: CanonicalId,
    pub credential_bytes: u16,
    pub initial_grant_policy: CanonicalId,
    pub inactive_capabilities: Vec<CanonicalId>,
    pub attunement_phases: Vec<DefaultAttunementPhase>,
    pub estimated_min_duration_ms: u64,
    pub estimated_max_duration_ms: u64,
    pub inventory_max_files: u64,
    pub inventory_max_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DefaultAttunementPhase {
    pub phase: AttunementPhase,
    pub configuration: String,
    pub estimated_items: u64,
    pub estimated_input_bytes: u64,
}

impl DefaultInstallProfile {
    fn load(requested: &str) -> Result<(Self, String)> {
        if requested != "default" {
            return Err(ServiceError::Contract(format!(
                "unsupported installation profile {requested:?}"
            )));
        }
        let profile: Self = serde_json::from_slice(INSTALL_PROFILE_BYTES).map_err(contract_json)?;
        profile.validate()?;
        Ok((profile, digest::sha256_hex(INSTALL_PROFILE_BYTES)))
    }

    fn validate(&self) -> Result<()> {
        if self.format_version != INSTALL_PROFILE_FORMAT
            || self.id.as_str() != "default"
            || self.initial_grant_policy.as_str() != "local-owner-all-actions"
            || !(32..=4_096).contains(&self.credential_bytes)
            || self.inventory_max_files == 0
            || self.inventory_max_bytes == 0
            || self.estimated_min_duration_ms > self.estimated_max_duration_ms
        {
            return Err(ServiceError::Contract(
                "default installation profile has invalid bounds or identity".into(),
            ));
        }
        self.initial_seat
            .validate()
            .map_err(|error| ServiceError::Contract(error.to_string()))?;
        if self
            .inactive_capabilities
            .windows(2)
            .any(|capabilities| capabilities[0] >= capabilities[1])
            || self.attunement_phases.len() != ATTUNEMENT_PHASES.len()
        {
            return Err(ServiceError::Contract(
                "default installation profile catalogues are incomplete or non-canonical".into(),
            ));
        }
        for (phase, expected) in self.attunement_phases.iter().zip(ATTUNEMENT_PHASES) {
            if phase.phase != expected
                || phase.configuration.trim().is_empty()
                || phase.configuration.len() > 1_024
            {
                return Err(ServiceError::Contract(
                    "default installation profile attunement phases are invalid".into(),
                ));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InstallationPreview {
    pub installation: InstallationPlan,
    pub attunement: AttunementPlan,
}

impl InstallationPreview {
    pub fn validate(&self) -> Result<()> {
        self.installation
            .validate_attunement_plan(&self.attunement)
            .map_err(|error| ServiceError::Contract(error.to_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectLocator {
    pub format_version: u16,
    pub product_version: String,
    pub project_root: String,
    pub instance_id: CanonicalId,
    pub storage_root: String,
    pub token_key: String,
    pub operator_credential: String,
    pub plan_sha256: String,
    pub profile_id: CanonicalId,
    pub profile_sha256: String,
    pub executable_sha256: String,
    pub installed_record_sha256: String,
}

impl ProjectLocator {
    fn validate(&self) -> Result<()> {
        if self.format_version != LOCATOR_FORMAT
            || self.product_version != env!("CARGO_PKG_VERSION")
            || !is_sha256(&self.plan_sha256)
            || !is_sha256(&self.profile_sha256)
            || !is_sha256(&self.executable_sha256)
            || !is_sha256(&self.installed_record_sha256)
        {
            return Err(ServiceError::Contract(
                "project locator identity or digest is invalid".into(),
            ));
        }
        for path in [
            self.storage_root.as_str(),
            self.token_key.as_str(),
            self.operator_credential.as_str(),
        ] {
            validate_managed_relative(path)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InstalledEstateRecord {
    pub format_version: u16,
    pub installation: InstallationPlan,
    pub attunement_plan_sha256: String,
    pub credential_sha256: String,
    pub token_key_sha256: String,
    pub runtime_commit_sha256: String,
    pub expected_runtime_cursor: u64,
    pub bootstrap_control_sequence: u64,
    pub installed_at_unix_ms: u64,
    pub record_sha256: String,
}

impl InstalledEstateRecord {
    fn validate(&self) -> Result<()> {
        self.installation
            .validate()
            .map_err(|error| ServiceError::Contract(error.to_string()))?;
        if self.format_version != INSTALLED_RECORD_FORMAT
            || self.attunement_plan_sha256 != self.installation.attunement_plan_sha256
            || !is_sha256(&self.credential_sha256)
            || !is_sha256(&self.token_key_sha256)
            || !is_sha256(&self.runtime_commit_sha256)
            || self.expected_runtime_cursor == 0
            || self.bootstrap_control_sequence == 0
            || self.installed_at_unix_ms == 0
            || self.record_sha256 != installed_record_sha256(self)?
        {
            return Err(ServiceError::Contract(
                "installed estate record is inconsistent or corrupt".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct InstallationCheckpoint {
    format_version: u16,
    plan_sha256: String,
    runtime_commit_sha256: String,
    runtime_cursor: u64,
    control_sequence: u64,
    installed_record_sha256: String,
    at_unix_ms: u64,
    checkpoint_sha256: String,
}

impl InstallationCheckpoint {
    fn validate(&self) -> Result<()> {
        if self.format_version != INSTALL_CHECKPOINT_FORMAT
            || !is_sha256(&self.plan_sha256)
            || !is_sha256(&self.runtime_commit_sha256)
            || !is_sha256(&self.installed_record_sha256)
            || self.runtime_cursor == 0
            || self.control_sequence == 0
            || self.at_unix_ms == 0
            || self.checkpoint_sha256 != installation_checkpoint_sha256(self)?
        {
            return Err(ServiceError::Contract(
                "installation checkpoint is inconsistent or corrupt".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstallationVerificationStatus {
    Passed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InstallationVerificationCheck {
    pub id: CanonicalId,
    pub passed: bool,
    pub evidence_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InstallationVerificationReport {
    pub status: InstallationVerificationStatus,
    pub product_version: String,
    pub instance_id: CanonicalId,
    pub plan_sha256: String,
    pub project_inventory_sha256: String,
    pub attunement_source_current: bool,
    pub runtime_manifest_sha256: String,
    pub runtime_cursor: u64,
    pub control_journal_sequence: u64,
    pub physical: rrd_store::RrflowKvInspection,
    pub checks: Vec<InstallationVerificationCheck>,
}

impl RrdEngine {
    pub fn plan_installation(
        project: &Path,
        target_kind: InstallationTargetKind,
        profile_id: &str,
        executable: &Path,
    ) -> Result<InstallationPreview> {
        let project = canonical_project(project, target_kind)?;
        let (profile, profile_sha256) = DefaultInstallProfile::load(profile_id)?;
        let executable_sha256 = sha256_file(executable)?;
        let project_precondition_sha256 = project_inventory_sha256(&project, &profile)?;
        let project_root = utf8_project_path(&project)?.to_owned();
        let identity_seed = json_sha256(&(
            "rrflow-installed-estate-v1",
            &project_root,
            target_kind,
            &project_precondition_sha256,
            &profile_sha256,
            &executable_sha256,
        ))?;
        let install_id = canonical_prefixed("install", &identity_seed)?;
        let attunement_id = canonical_prefixed("attune", &identity_seed)?;
        let instance_id = canonical_prefixed("instance", &identity_seed)?;
        let estate_id = canonical_prefixed("estate", &identity_seed)?;
        let project_id = canonical_prefixed("project", &identity_seed)?;
        let target = ResourcePath {
            segments: vec![
                ResourceId::new(ResourceKind::Organization, "local")
                    .map_err(|error| ServiceError::Contract(error.to_string()))?,
                ResourceId::new(ResourceKind::Estate, estate_id.as_str())
                    .map_err(|error| ServiceError::Contract(error.to_string()))?,
                ResourceId::new(ResourceKind::Project, project_id.as_str())
                    .map_err(|error| ServiceError::Contract(error.to_string()))?,
                ResourceId::new(ResourceKind::Instance, instance_id.as_str())
                    .map_err(|error| ServiceError::Contract(error.to_string()))?,
            ],
        };
        let phases = profile
            .attunement_phases
            .iter()
            .map(|phase| {
                Ok(AttunementPhasePlan {
                    phase: phase.phase,
                    sequence: phase.phase.sequence(),
                    configuration_sha256: json_sha256(phase)?,
                    estimated_items: phase.estimated_items,
                    estimated_input_bytes: phase.estimated_input_bytes,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let mut attunement = AttunementPlan {
            contract_version: INSTALL_ATTUNEMENT_CONTRACT_VERSION,
            id: attunement_id,
            target: target.clone(),
            source_sha256: project_precondition_sha256.clone(),
            phases,
            estimated_min_duration_ms: profile.estimated_min_duration_ms,
            estimated_max_duration_ms: profile.estimated_max_duration_ms,
            plan_sha256: zero_digest(),
        };
        attunement.plan_sha256 = attunement_plan_sha256(&attunement)
            .map_err(|error| ServiceError::Contract(error.to_string()))?;

        let storage_relative = format!(".rrflow/rrd/roots/{install_id}");
        let token_relative = format!("{storage_relative}/RRD.TOKEN");
        let credential_relative =
            format!(".rrflow/credentials/{}.json", profile.initial_principal_id);
        let absent = digest::sha256_hex(b"rrflow-install-managed-path-absent-v1");
        let managed_paths = [
            (InstallationManagedPathKind::StorageRoot, storage_relative),
            (InstallationManagedPathKind::TokenKey, token_relative),
            (
                InstallationManagedPathKind::OperatorCredential,
                credential_relative,
            ),
            (
                InstallationManagedPathKind::ProjectLocator,
                LOCATOR_RELATIVE_PATH.into(),
            ),
        ]
        .into_iter()
        .map(|(kind, relative_path)| {
            let path = project.join(&relative_path);
            if std::fs::symlink_metadata(&path).is_ok() {
                return Err(ServiceError::ProjectBindingMismatch);
            }
            if let Some(parent) = path.parent() {
                reject_existing_symlink_path(&project, parent)?;
            }
            Ok(InstallationManagedPath {
                kind,
                relative_path,
                precondition_sha256: absent.clone(),
                removal_rule: InstallationRemovalRule::RemoveIfOwnedDigestMatches,
            })
        })
        .collect::<Result<Vec<_>>>()?;
        let configuration_sha256 = json_sha256(&(
            &profile,
            &target,
            &managed_paths,
            &project_precondition_sha256,
        ))?;
        let action_kinds = [
            InstallationActionKind::ValidateProject,
            InstallationActionKind::CreateStorageRoot,
            InstallationActionKind::PrepareCredentials,
            InstallationActionKind::InitializeInstance,
            InstallationActionKind::CreateAttunementJob,
            InstallationActionKind::PublishProjectLocator,
        ];
        let estimates = [
            0,
            4_096,
            u64::from(profile.credential_bytes) * 2 + 512,
            16_384,
            4_096,
            1_024,
        ];
        let actions = action_kinds
            .into_iter()
            .zip(estimates)
            .map(|(kind, estimated_write_bytes)| {
                Ok(InstallationPlanAction {
                    kind,
                    disposition: if kind == InstallationActionKind::ValidateProject {
                        InstallationActionDisposition::Unchanged
                    } else {
                        InstallationActionDisposition::Create
                    },
                    input_sha256: json_sha256(&(
                        kind,
                        &configuration_sha256,
                        &attunement.plan_sha256,
                    ))?,
                    estimated_write_bytes,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let mut installation = InstallationPlan {
            contract_version: INSTALL_ATTUNEMENT_CONTRACT_VERSION,
            id: install_id.clone(),
            target,
            target_kind,
            product_version: env!("CARGO_PKG_VERSION").into(),
            executable_sha256,
            profile_id: profile.id,
            profile_sha256,
            project_root,
            project_precondition_sha256,
            storage_root_id: install_id,
            configuration_sha256,
            initial_seat: profile.initial_seat,
            initial_principal_id: profile.initial_principal_id,
            initial_grants: SecurityAction::ALL.to_vec(),
            credential_bytes: profile.credential_bytes,
            inactive_capabilities: profile.inactive_capabilities,
            managed_paths,
            attunement_plan_id: attunement.id.clone(),
            attunement_plan_sha256: attunement.plan_sha256.clone(),
            actions,
            plan_sha256: zero_digest(),
        };
        installation.plan_sha256 = installation_plan_sha256(&installation)
            .map_err(|error| ServiceError::Contract(error.to_string()))?;
        let preview = InstallationPreview {
            installation,
            attunement,
        };
        preview.validate()?;
        Ok(preview)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn apply_installation(
        project: &Path,
        target_kind: InstallationTargetKind,
        preview: &InstallationPreview,
        expected_plan_sha256: &str,
        at_unix_ms: u64,
        executable: &Path,
    ) -> Result<InstallationResult> {
        preview.validate()?;
        if at_unix_ms == 0
            || preview.installation.target_kind != target_kind
            || preview.installation.plan_sha256 != expected_plan_sha256
        {
            return Err(ServiceError::Contract(
                "installation apply mode, time, or expected digest does not match the preview"
                    .into(),
            ));
        }
        let project = canonical_project_root(project)?;
        if preview.installation.project_root != utf8_project_path(&project)?
            || sha256_file(executable)? != preview.installation.executable_sha256
        {
            return Err(ServiceError::ProjectBindingMismatch);
        }
        let (_, profile_sha256) =
            DefaultInstallProfile::load(preview.installation.profile_id.as_str())?;
        if profile_sha256 != preview.installation.profile_sha256 {
            return Err(ServiceError::ProjectBindingMismatch);
        }

        let locator_path = project.join(LOCATOR_RELATIVE_PATH);
        if locator_path.exists() {
            return replay_installation_result(&project, preview, executable);
        }
        let canonical = Self::plan_installation(
            &project,
            target_kind,
            preview.installation.profile_id.as_str(),
            executable,
        )?;
        if &canonical != preview {
            return Err(ServiceError::ProjectBindingMismatch);
        }
        validate_installation_target(&project, target_kind)?;
        let (profile, _) = DefaultInstallProfile::load("default")?;
        if project_inventory_sha256(&project, &profile)?
            != preview.installation.project_precondition_sha256
        {
            return Err(ServiceError::ProjectBindingMismatch);
        }
        for managed in &preview.installation.managed_paths {
            if std::fs::symlink_metadata(project.join(&managed.relative_path)).is_ok() {
                return Err(ServiceError::ProjectBindingMismatch);
            }
        }

        let storage_relative = preview
            .installation
            .managed_path(InstallationManagedPathKind::StorageRoot)
            .relative_path
            .clone();
        let token_relative = preview
            .installation
            .managed_path(InstallationManagedPathKind::TokenKey)
            .relative_path
            .clone();
        let credential_relative = preview
            .installation
            .managed_path(InstallationManagedPathKind::OperatorCredential)
            .relative_path
            .clone();
        ensure_managed_parent(&project, &storage_relative)?;
        ensure_managed_parent(&project, &credential_relative)?;
        let storage_root = safe_join(&project, &storage_relative)?;
        let token_path = safe_join(&project, &token_relative)?;
        let credential_path = safe_join(&project, &credential_relative)?;
        let instance_id = target_instance(&preview.installation.target)?.clone();
        let mut engine = RrdEngine::create_new(&storage_root, instance_id.clone(), [0; 32])?;
        let token_key = create_token_key(&token_path)
            .map_err(|error| ServiceError::Storage(error.to_string()))?;
        engine.token_key = token_key;
        let credential = create_api_key_document(
            &credential_path,
            &preview.installation.plan_sha256,
            preview.installation.initial_principal_id.clone(),
            preview.installation.credential_bytes,
        )
        .map_err(|error| ServiceError::Storage(error.to_string()))?;
        let credential_sha256 = digest::sha256_hex(credential.credential().as_bytes());
        let token_key_sha256 = digest::sha256_hex(&token_key);

        let memory_plan = build_memory_estate_plan(
            &PersistMemoryEstate {
                scope: format!("instance:{instance_id}"),
                seat: preview.installation.initial_seat.clone(),
                representations: Vec::new(),
                valid_from: at_unix_ms,
            },
            None,
        )?;
        let bootstrap_session = CorrelationId::new("installation-bootstrap")
            .map_err(|error| ServiceError::Contract(error.to_string()))?;
        let runtime_mutations = memory_plan
            .mutations
            .iter()
            .map(|mutation| public_runtime_mutation(mutation, &bootstrap_session))
            .collect::<Result<Vec<_>>>()?;
        let runtime_commit = RuntimeCommit {
            scope: ScopeId::new(format!("instance:{instance_id}")).map_err(core_contract)?,
            at: at_unix_ms,
            actor: "rrflow:installer".into(),
            expected_cursor: 0,
            mutations: runtime_mutations,
        };
        runtime_commit.validate().map_err(core_contract)?;
        let runtime_commit_sha256 = runtime_commit.digest();
        let expected_runtime_cursor = runtime_commit.mutations.len() as u64;

        let resource = ResourcePath {
            segments: vec![
                ResourceId::new(ResourceKind::Instance, instance_id.as_str())
                    .map_err(|error| ServiceError::Contract(error.to_string()))?,
            ],
        };
        let grants = preview
            .installation
            .initial_grants
            .iter()
            .copied()
            .map(|action| ResourceGrant {
                action,
                resource_prefix: resource.clone(),
                data_policy: None,
            })
            .collect();
        let principal = Principal {
            id: preview.installation.initial_principal_id.clone(),
            kind: PrincipalKind::User,
            credential_sha256: credential_sha256.clone(),
            credential_revision: 1,
            not_before_unix_ms: at_unix_ms,
            expires_at_unix_ms: u64::MAX,
            disabled: false,
            role_ids: BTreeSet::new(),
            grants,
        };
        let security = SecurityState {
            format_version: SECURITY_FORMAT,
            revision: 1,
            principals: BTreeMap::from([(principal.id.clone(), principal)]),
            roles: BTreeMap::new(),
            identity_bindings: BTreeMap::new(),
            jwt_issuers: BTreeMap::new(),
        };
        let request_id = format!(
            "install-request-{}",
            &preview.installation.plan_sha256[..16]
        );
        let operation_id = format!(
            "install-operation-{}",
            &preview.installation.plan_sha256[..16]
        );
        let mut transitions = rrd_security::prepare_security_initialization(
            &instance_id,
            &security,
            at_unix_ms,
            "rrflow:installer",
            &request_id,
            &operation_id,
        )?;
        let bootstrap_control_sequence = transitions.len() as u64 + 4;
        let mut installed = InstalledEstateRecord {
            format_version: INSTALLED_RECORD_FORMAT,
            installation: preview.installation.clone(),
            attunement_plan_sha256: preview.attunement.plan_sha256.clone(),
            credential_sha256: credential_sha256.clone(),
            token_key_sha256,
            runtime_commit_sha256: runtime_commit_sha256.clone(),
            expected_runtime_cursor,
            bootstrap_control_sequence,
            installed_at_unix_ms: at_unix_ms,
            record_sha256: zero_digest(),
        };
        installed.record_sha256 = installed_record_sha256(&installed)?;
        installed.validate()?;
        let mut checkpoint = InstallationCheckpoint {
            format_version: INSTALL_CHECKPOINT_FORMAT,
            plan_sha256: preview.installation.plan_sha256.clone(),
            runtime_commit_sha256: runtime_commit_sha256.clone(),
            runtime_cursor: expected_runtime_cursor,
            control_sequence: bootstrap_control_sequence,
            installed_record_sha256: installed.record_sha256.clone(),
            at_unix_ms,
            checkpoint_sha256: zero_digest(),
        };
        checkpoint.checkpoint_sha256 = installation_checkpoint_sha256(&checkpoint)?;
        checkpoint.validate()?;
        let job = AttunementJob {
            contract_version: INSTALL_ATTUNEMENT_CONTRACT_VERSION,
            id: canonical_prefixed("job", &preview.attunement.plan_sha256)?,
            plan_id: preview.attunement.id.clone(),
            plan_sha256: preview.attunement.plan_sha256.clone(),
            revision: 1,
            state: AttunementJobState::Pending,
            current_phase: None,
            checkpoints: Vec::new(),
            lease: None,
            failure: None,
            created_at_unix_ms: at_unix_ms,
            updated_at_unix_ms: at_unix_ms,
        };
        job.validate(&preview.attunement)
            .map_err(|error| ServiceError::Contract(error.to_string()))?;
        transitions.extend([
            control_create(
                INSTALLED_RECORD_KEY,
                &installed,
                at_unix_ms,
                &request_id,
                &operation_id,
                "install.record",
            )?,
            control_create(
                INSTALL_CHECKPOINT_KEY,
                &checkpoint,
                at_unix_ms,
                &request_id,
                &operation_id,
                "install.checkpoint",
            )?,
            control_create(
                &attunement_plan_key(&instance_id, &preview.attunement.id),
                &preview.attunement,
                at_unix_ms,
                &request_id,
                &operation_id,
                "attunement.plan.created",
            )?,
            control_create(
                &attunement_job_key(&instance_id, &job.id),
                &job,
                at_unix_ms,
                &request_id,
                &operation_id,
                "attunement.job.created",
            )?,
        ]);
        let (outcome, entries) = engine
            .storage
            .runtime()
            .commit_with_control_transitions(&runtime_commit, &transitions)?;
        if outcome.commit_id != runtime_commit_sha256
            || outcome.last_cursor != expected_runtime_cursor
            || entries.last().map(|entry| entry.sequence) != Some(bootstrap_control_sequence)
            || engine.storage.control().get(INSTALLED_RECORD_KEY)?
                != Some(serde_json::to_vec(&installed).map_err(contract_json)?)
        {
            return Err(ServiceError::StorageConflict(
                "installation readback differs from the committed bootstrap".into(),
            ));
        }
        let read = engine.storage.runtime().read_stamp(&runtime_commit.scope)?;
        let locator = ProjectLocator {
            format_version: LOCATOR_FORMAT,
            product_version: env!("CARGO_PKG_VERSION").into(),
            project_root: utf8_project_path(&project)?.to_owned(),
            instance_id,
            storage_root: storage_relative,
            token_key: token_relative,
            operator_credential: credential_relative,
            plan_sha256: preview.installation.plan_sha256.clone(),
            profile_id: preview.installation.profile_id.clone(),
            profile_sha256: preview.installation.profile_sha256.clone(),
            executable_sha256: preview.installation.executable_sha256.clone(),
            installed_record_sha256: installed.record_sha256.clone(),
        };
        locator.validate()?;
        let locator_bytes = toml::to_string(&locator)
            .map_err(|error| ServiceError::Contract(error.to_string()))?
            .into_bytes();
        let locator_sha256 = digest::sha256_hex(&locator_bytes);
        drop(engine);
        publish_new_file(
            &locator_path,
            &preview.installation.plan_sha256,
            &locator_bytes,
        )?;
        build_installation_result(
            preview,
            &job.id,
            &installed,
            &read.manifest_id,
            read.commit_cursor,
            bootstrap_control_sequence,
            &credential_sha256,
            &locator_sha256,
            false,
        )
    }

    pub fn open_installed(project: &Path, executable: &Path) -> Result<Self> {
        let (project, locator, _) = read_locator(project)?;
        let (_, profile_sha256) = DefaultInstallProfile::load(locator.profile_id.as_str())?;
        if sha256_file(executable)? != locator.executable_sha256
            || profile_sha256 != locator.profile_sha256
        {
            return Err(ServiceError::ProjectBindingMismatch);
        }
        let token_path = safe_join(&project, &locator.token_key)?;
        let storage_root = safe_join(&project, &locator.storage_root)?;
        let token_key = read_token_key(&token_path)
            .map_err(|error| ServiceError::Storage(error.to_string()))?;
        let engine =
            RrdEngine::open_existing(&storage_root, locator.instance_id.clone(), token_key)?;
        let bytes = engine
            .storage
            .control()
            .get(INSTALLED_RECORD_KEY)?
            .ok_or_else(|| ServiceError::Storage("installed estate record is missing".into()))?;
        let installed: InstalledEstateRecord = decode_json(&bytes, "installed estate record")?;
        installed.validate()?;
        require_locator_record_match(&locator, &installed)?;
        if !engine.security_enforced()? {
            return Err(ServiceError::Storage(
                "installed security authority is missing".into(),
            ));
        }
        Ok(engine)
    }

    pub fn read_installed_api_key(project: &Path) -> Result<ApiKeyDocument> {
        let (project, locator, _) = read_locator(project)?;
        let document = read_api_key_document(&safe_join(&project, &locator.operator_credential)?)
            .map_err(|error| ServiceError::Storage(error.to_string()))?;
        if document.plan_sha256 != locator.plan_sha256 {
            return Err(ServiceError::ProjectBindingMismatch);
        }
        Ok(document)
    }

    /// Reads the public, project-local installed-engine locator. The locator
    /// contains identities and paths but never credential material.
    pub fn read_project_locator(project: &Path) -> Result<ProjectLocator> {
        read_locator(project).map(|(_, locator, _)| locator)
    }

    pub fn inspect_installed(
        project: &Path,
        executable: &Path,
    ) -> Result<InstallationVerificationReport> {
        let (project, locator, locator_bytes) = read_locator(project)?;
        let executable_sha256 = sha256_file(executable)?;
        if executable_sha256 != locator.executable_sha256 {
            return Err(ServiceError::ProjectBindingMismatch);
        }
        let (profile, profile_sha256) = DefaultInstallProfile::load(locator.profile_id.as_str())?;
        if profile_sha256 != locator.profile_sha256 {
            return Err(ServiceError::ProjectBindingMismatch);
        }
        let token_key = read_token_key(&safe_join(&project, &locator.token_key)?)
            .map_err(|error| ServiceError::Storage(error.to_string()))?;
        let credential = read_api_key_document(&safe_join(&project, &locator.operator_credential)?)
            .map_err(|error| ServiceError::Storage(error.to_string()))?;
        let storage_root = safe_join(&project, &locator.storage_root)?;
        super::core::validate_existing_object_layout(&storage_root)?;
        let inspector = RrflowKvInspector::open(&storage_root)?;
        let installed: InstalledEstateRecord =
            decode_control(&inspector, INSTALLED_RECORD_KEY, "installed estate record")?;
        installed.validate()?;
        require_locator_record_match(&locator, &installed)?;
        let project_inventory_sha256 = project_inventory_sha256(&project, &profile)?;
        let attunement_source_current =
            project_inventory_sha256 == installed.installation.project_precondition_sha256;
        if credential.plan_sha256 != installed.installation.plan_sha256
            || credential.principal_id != installed.installation.initial_principal_id
            || digest::sha256_hex(credential.credential().as_bytes()) != installed.credential_sha256
            || digest::sha256_hex(&token_key) != installed.token_key_sha256
            || inspector.inspection().runtime_cursor < installed.expected_runtime_cursor
            || inspector.inspection().control_journal_sequence
                < installed.bootstrap_control_sequence
        {
            return Err(ServiceError::ProjectBindingMismatch);
        }
        let security_key = format!("server/state/{}/security/policy", locator.instance_id);
        let security: SecurityState = decode_control(&inspector, &security_key, "security state")?;
        security.validate()?;
        let principal = security
            .principals
            .get(&installed.installation.initial_principal_id)
            .ok_or(ServiceError::Unauthenticated)?;
        if principal.credential_sha256 != installed.credential_sha256
            || principal
                .grants
                .iter()
                .map(|grant| grant.action)
                .collect::<Vec<_>>()
                != installed.installation.initial_grants
        {
            return Err(ServiceError::ProjectBindingMismatch);
        }
        let attunement: AttunementPlan = decode_control(
            &inspector,
            &attunement_plan_key(
                &locator.instance_id,
                &installed.installation.attunement_plan_id,
            ),
            "attunement plan",
        )?;
        installed
            .installation
            .validate_attunement_plan(&attunement)
            .map_err(|error| ServiceError::Contract(error.to_string()))?;
        let job_id = canonical_prefixed("job", &attunement.plan_sha256)?;
        let job: AttunementJob = decode_control(
            &inspector,
            &attunement_job_key(&locator.instance_id, &job_id),
            "attunement job",
        )?;
        job.validate(&attunement)
            .map_err(|error| ServiceError::Contract(error.to_string()))?;
        let checkpoint: InstallationCheckpoint = decode_control(
            &inspector,
            INSTALL_CHECKPOINT_KEY,
            "installation checkpoint",
        )?;
        checkpoint.validate()?;
        if checkpoint.installed_record_sha256 != installed.record_sha256
            || checkpoint.runtime_cursor != installed.expected_runtime_cursor
            || checkpoint.control_sequence != installed.bootstrap_control_sequence
        {
            return Err(ServiceError::ProjectBindingMismatch);
        }

        let runtime_manifest_sha256 = runtime_manifest_from_inspection(&inspector, &locator)?;
        let evidence = [
            ("locator", digest::sha256_hex(&locator_bytes)),
            ("profile", profile_sha256),
            ("executable", executable_sha256),
            ("project-inventory", project_inventory_sha256.clone()),
            ("physical", json_sha256(&inspector.inspection())?),
            ("installed-record", installed.record_sha256.clone()),
            ("credentials", installed.credential_sha256.clone()),
            ("security", json_sha256(&security)?),
            ("runtime", runtime_manifest_sha256.clone()),
            ("attunement", attunement.plan_sha256),
            ("checkpoint", checkpoint.checkpoint_sha256),
        ];
        let checks = evidence
            .into_iter()
            .map(|(id, evidence_sha256)| {
                Ok(InstallationVerificationCheck {
                    id: CanonicalId::new(id)
                        .map_err(|error| ServiceError::Contract(error.to_string()))?,
                    passed: true,
                    evidence_sha256,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(InstallationVerificationReport {
            status: InstallationVerificationStatus::Passed,
            product_version: locator.product_version,
            instance_id: locator.instance_id,
            plan_sha256: locator.plan_sha256,
            project_inventory_sha256,
            attunement_source_current,
            runtime_manifest_sha256,
            runtime_cursor: inspector.inspection().runtime_cursor,
            control_journal_sequence: inspector.inspection().control_journal_sequence,
            physical: inspector.inspection().clone(),
            checks,
        })
    }
}

fn canonical_project(project: &Path, target_kind: InstallationTargetKind) -> Result<PathBuf> {
    let project = canonical_project_root(project)?;
    validate_installation_target(&project, target_kind)?;
    Ok(project)
}

fn canonical_project_root(project: &Path) -> Result<PathBuf> {
    let link = std::fs::symlink_metadata(project)
        .map_err(|error| ServiceError::Storage(error.to_string()))?;
    if link.file_type().is_symlink() || !link.is_dir() {
        return Err(ServiceError::ProjectBindingMismatch);
    }
    let project =
        std::fs::canonicalize(project).map_err(|error| ServiceError::Storage(error.to_string()))?;
    Ok(project)
}

fn validate_installation_target(project: &Path, target_kind: InstallationTargetKind) -> Result<()> {
    if target_kind == InstallationTargetKind::FreshProject
        && std::fs::read_dir(project)
            .map_err(|error| ServiceError::Storage(error.to_string()))?
            .next()
            .is_some()
    {
        return Err(ServiceError::ProjectBindingMismatch);
    }
    Ok(())
}

fn project_inventory_sha256(project: &Path, profile: &DefaultInstallProfile) -> Result<String> {
    let mut entries = Vec::<(PathBuf, bool)>::new();
    collect_project_entries(project, project, &mut entries)?;
    entries.sort_by(|left, right| left.0.cmp(&right.0));
    if entries.len() as u64 > profile.inventory_max_files {
        return Err(ServiceError::Contract(
            "project inventory exceeds the configured file bound".into(),
        ));
    }
    let mut aggregate = IncrementalSha256::new();
    aggregate.update(b"rrflow-project-inventory-v1\0");
    let mut total = 0u64;
    for (path, directory) in entries {
        let relative = path
            .strip_prefix(project)
            .map_err(|error| ServiceError::Storage(error.to_string()))?;
        let relative = relative
            .to_str()
            .ok_or(ServiceError::ProjectBindingMismatch)?
            .replace('\\', "/");
        aggregate.update(if directory { b"d\0" } else { b"f\0" });
        aggregate.update(&(relative.len() as u64).to_be_bytes());
        aggregate.update(relative.as_bytes());
        if !directory {
            let metadata = std::fs::metadata(&path)
                .map_err(|error| ServiceError::Storage(error.to_string()))?;
            total = total.checked_add(metadata.len()).ok_or_else(|| {
                ServiceError::Contract("project inventory byte count overflowed".into())
            })?;
            if total > profile.inventory_max_bytes {
                return Err(ServiceError::Contract(
                    "project inventory exceeds the configured byte bound".into(),
                ));
            }
            aggregate.update(&metadata.len().to_be_bytes());
            aggregate.update(sha256_file(&path)?.as_bytes());
        }
    }
    Ok(aggregate.finalize_hex())
}

fn collect_project_entries(
    root: &Path,
    current: &Path,
    output: &mut Vec<(PathBuf, bool)>,
) -> Result<()> {
    let mut entries = std::fs::read_dir(current)
        .map_err(|error| ServiceError::Storage(error.to_string()))?
        .collect::<std::io::Result<Vec<_>>>()
        .map_err(|error| ServiceError::Storage(error.to_string()))?;
    entries.sort_by_key(std::fs::DirEntry::file_name);
    for entry in entries {
        let path = entry.path();
        let relative = path
            .strip_prefix(root)
            .map_err(|error| ServiceError::Storage(error.to_string()))?;
        if relative.components().next().is_some_and(|component| {
            matches!(component.as_os_str().to_str(), Some(".git" | ".rrflow"))
        }) {
            continue;
        }
        let metadata = std::fs::symlink_metadata(&path)
            .map_err(|error| ServiceError::Storage(error.to_string()))?;
        if metadata.file_type().is_symlink() {
            return Err(ServiceError::ProjectBindingMismatch);
        }
        if metadata.is_dir() {
            output.push((path.clone(), true));
            collect_project_entries(root, &path, output)?;
        } else if metadata.is_file() {
            output.push((path, false));
        } else {
            return Err(ServiceError::ProjectBindingMismatch);
        }
    }
    Ok(())
}

fn sha256_file(path: &Path) -> Result<String> {
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|error| ServiceError::Storage(error.to_string()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(ServiceError::ProjectBindingMismatch);
    }
    let mut file = File::open(path).map_err(|error| ServiceError::Storage(error.to_string()))?;
    let mut hasher = IncrementalSha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|error| ServiceError::Storage(error.to_string()))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hasher.finalize_hex())
}

fn ensure_managed_parent(project: &Path, relative: &str) -> Result<()> {
    validate_managed_relative(relative)?;
    let parent = Path::new(relative)
        .parent()
        .ok_or(ServiceError::ProjectBindingMismatch)?;
    let mut current = project.to_owned();
    for component in parent.components() {
        let std::path::Component::Normal(segment) = component else {
            return Err(ServiceError::ProjectBindingMismatch);
        };
        current.push(segment);
        match std::fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
                return Err(ServiceError::ProjectBindingMismatch);
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                std::fs::create_dir(&current)
                    .map_err(|error| ServiceError::Storage(error.to_string()))?;
                rrd_store::sync_directory_metadata(
                    current.parent().expect("managed directory has a parent"),
                )
                .map_err(|error| ServiceError::Storage(error.to_string()))?;
            }
            Err(error) => return Err(ServiceError::Storage(error.to_string())),
        }
    }
    Ok(())
}

fn reject_existing_symlink_path(project: &Path, path: &Path) -> Result<()> {
    let relative = path
        .strip_prefix(project)
        .map_err(|_| ServiceError::ProjectBindingMismatch)?;
    let mut current = project.to_owned();
    for component in relative.components() {
        current.push(component);
        match std::fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(ServiceError::ProjectBindingMismatch);
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => break,
            Err(error) => return Err(ServiceError::Storage(error.to_string())),
        }
    }
    Ok(())
}

fn validate_managed_relative(relative: &str) -> Result<()> {
    if relative.is_empty()
        || relative.len() > 4_096
        || relative.starts_with('/')
        || relative.contains('\\')
        || relative
            .split('/')
            .any(|segment| segment.is_empty() || matches!(segment, "." | ".."))
        || !relative.starts_with(".rrflow/")
    {
        return Err(ServiceError::ProjectBindingMismatch);
    }
    Ok(())
}

fn safe_join(project: &Path, relative: &str) -> Result<PathBuf> {
    validate_managed_relative(relative)?;
    let path = project.join(relative);
    reject_existing_symlink_path(project, &path)?;
    Ok(path)
}

fn read_locator(project: &Path) -> Result<(PathBuf, ProjectLocator, Vec<u8>)> {
    let project = canonical_project(project, InstallationTargetKind::ExistingProject)?;
    let path = project.join(LOCATOR_RELATIVE_PATH);
    reject_existing_symlink_path(&project, &path)?;
    let metadata = std::fs::symlink_metadata(&path)
        .map_err(|error| ServiceError::Storage(error.to_string()))?;
    if !metadata.is_file() || metadata.len() > MAX_LOCATOR_BYTES {
        return Err(ServiceError::ProjectBindingMismatch);
    }
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    File::open(&path)
        .map_err(|error| ServiceError::Storage(error.to_string()))?
        .take(MAX_LOCATOR_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| ServiceError::Storage(error.to_string()))?;
    if bytes.len() as u64 > MAX_LOCATOR_BYTES {
        return Err(ServiceError::ProjectBindingMismatch);
    }
    let locator: ProjectLocator = toml::from_str(
        std::str::from_utf8(&bytes).map_err(|error| ServiceError::Contract(error.to_string()))?,
    )
    .map_err(|error| ServiceError::Contract(error.to_string()))?;
    locator.validate()?;
    if locator.project_root != utf8_project_path(&project)? {
        return Err(ServiceError::ProjectBindingMismatch);
    }
    Ok((project, locator, bytes))
}

fn utf8_project_path(project: &Path) -> Result<&str> {
    project.to_str().ok_or(ServiceError::ProjectBindingMismatch)
}

fn publish_new_file(path: &Path, plan_sha256: &str, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().ok_or(ServiceError::ProjectBindingMismatch)?;
    let temporary = parent.join(format!(".config.{}.pending", &plan_sha256[..16]));
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt as _;
        options.mode(0o644);
    }
    let result = (|| -> std::io::Result<()> {
        let mut file = options.open(&temporary)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        std::fs::hard_link(&temporary, path)?;
        rrd_store::sync_directory_metadata(parent)?;
        std::fs::remove_file(&temporary)?;
        rrd_store::sync_directory_metadata(parent)
    })();
    if let Err(error) = result {
        let _ = std::fs::remove_file(&temporary);
        return Err(ServiceError::Storage(error.to_string()));
    }
    Ok(())
}

fn replay_installation_result(
    project: &Path,
    preview: &InstallationPreview,
    executable: &Path,
) -> Result<InstallationResult> {
    let report = RrdEngine::inspect_installed(project, executable)?;
    if report.plan_sha256 != preview.installation.plan_sha256 {
        return Err(ServiceError::IdempotencyConflict);
    }
    let (_, locator, locator_bytes) = read_locator(project)?;
    let inspector = RrflowKvInspector::open(&safe_join(project, &locator.storage_root)?)?;
    let installed: InstalledEstateRecord =
        decode_control(&inspector, INSTALLED_RECORD_KEY, "installed estate record")?;
    let job_id = canonical_prefixed("job", &preview.attunement.plan_sha256)?;
    build_installation_result(
        preview,
        &job_id,
        &installed,
        &report.runtime_manifest_sha256,
        report.runtime_cursor,
        installed.bootstrap_control_sequence,
        &installed.credential_sha256,
        &digest::sha256_hex(&locator_bytes),
        true,
    )
}

#[allow(clippy::too_many_arguments)]
fn build_installation_result(
    preview: &InstallationPreview,
    job_id: &CanonicalId,
    installed: &InstalledEstateRecord,
    runtime_manifest_sha256: &str,
    runtime_cursor: u64,
    control_journal_sequence: u64,
    credential_sha256: &str,
    locator_sha256: &str,
    idempotent_replay: bool,
) -> Result<InstallationResult> {
    let outputs = [
        preview.installation.project_precondition_sha256.clone(),
        json_sha256(&(
            &preview.installation.storage_root_id,
            preview
                .installation
                .managed_path(InstallationManagedPathKind::StorageRoot),
        ))?,
        credential_sha256.into(),
        installed.runtime_commit_sha256.clone(),
        preview.attunement.plan_sha256.clone(),
        locator_sha256.into(),
    ];
    let mut result = InstallationResult {
        contract_version: INSTALL_ATTUNEMENT_CONTRACT_VERSION,
        plan_id: preview.installation.id.clone(),
        plan_sha256: preview.installation.plan_sha256.clone(),
        attunement_job_id: job_id.clone(),
        action_results: preview
            .installation
            .actions
            .iter()
            .zip(outputs)
            .map(|(action, output_sha256)| InstallationActionResult {
                kind: action.kind,
                output_sha256,
            })
            .collect(),
        runtime_manifest_sha256: runtime_manifest_sha256.into(),
        runtime_cursor,
        control_journal_sequence,
        installed_record_sha256: installed.record_sha256.clone(),
        credential_sha256: credential_sha256.into(),
        locator_sha256: locator_sha256.into(),
        applied_at_unix_ms: installed.installed_at_unix_ms,
        idempotent_replay,
        result_sha256: zero_digest(),
    };
    result.result_sha256 = installation_result_sha256(&result)
        .map_err(|error| ServiceError::Contract(error.to_string()))?;
    result
        .validate_for(&preview.installation)
        .map_err(|error| ServiceError::Contract(error.to_string()))?;
    Ok(result)
}

fn control_create<T: Serialize>(
    key: &str,
    value: &T,
    at: u64,
    request_id: &str,
    operation_id: &str,
    action: &str,
) -> Result<ControlTransition> {
    let transition = ControlTransition {
        key: key.into(),
        expected: None,
        replacement: Some(serde_json::to_vec(value).map_err(contract_json)?),
        at,
        actor: "rrflow:installer".into(),
        action: action.into(),
        request_id: request_id.into(),
        operation_id: operation_id.into(),
    };
    transition.validate()?;
    Ok(transition)
}

fn decode_control<T: DeserializeOwned>(
    inspector: &RrflowKvInspector,
    key: &str,
    label: &str,
) -> Result<T> {
    let bytes = inspector
        .control_record(key)?
        .ok_or_else(|| ServiceError::Storage(format!("{label} is missing")))?;
    decode_json(&bytes, label)
}

fn decode_json<T: DeserializeOwned>(bytes: &[u8], label: &str) -> Result<T> {
    serde_json::from_slice(bytes)
        .map_err(|error| ServiceError::Storage(format!("{label} is invalid: {error}")))
}

fn require_locator_record_match(
    locator: &ProjectLocator,
    installed: &InstalledEstateRecord,
) -> Result<()> {
    if locator.plan_sha256 != installed.installation.plan_sha256
        || locator.profile_id != installed.installation.profile_id
        || locator.profile_sha256 != installed.installation.profile_sha256
        || locator.executable_sha256 != installed.installation.executable_sha256
        || locator.installed_record_sha256 != installed.record_sha256
        || locator.instance_id != *target_instance(&installed.installation.target)?
    {
        return Err(ServiceError::ProjectBindingMismatch);
    }
    Ok(())
}

fn target_instance(target: &ResourcePath) -> Result<&CanonicalId> {
    target
        .segments
        .last()
        .filter(|segment| segment.kind == ResourceKind::Instance)
        .map(|segment| &segment.id)
        .ok_or_else(|| ServiceError::Contract("installation target has no instance".into()))
}

fn attunement_plan_key(instance: &CanonicalId, plan: &CanonicalId) -> String {
    format!("server/state/{instance}/attunement/plan/{plan}")
}

fn attunement_job_key(instance: &CanonicalId, job: &CanonicalId) -> String {
    format!("server/state/{instance}/attunement/job/{job}")
}

fn runtime_manifest_from_inspection(
    inspector: &RrflowKvInspector,
    locator: &ProjectLocator,
) -> Result<String> {
    let scope = ScopeId::new(format!("instance:{}", locator.instance_id)).map_err(core_contract)?;
    let read = inspector.runtime_read_stamp(&scope)?;
    if read.commit_cursor != inspector.inspection().runtime_cursor {
        return Err(ServiceError::StorageConflict(
            "semantic read stamp and inspected runtime cursor differ".into(),
        ));
    }
    Ok(read.manifest_id)
}

fn installed_record_sha256(record: &InstalledEstateRecord) -> Result<String> {
    json_sha256(&(
        record.format_version,
        &record.installation,
        &record.attunement_plan_sha256,
        &record.credential_sha256,
        &record.token_key_sha256,
        &record.runtime_commit_sha256,
        record.expected_runtime_cursor,
        record.bootstrap_control_sequence,
        record.installed_at_unix_ms,
    ))
}

fn installation_checkpoint_sha256(checkpoint: &InstallationCheckpoint) -> Result<String> {
    json_sha256(&(
        checkpoint.format_version,
        &checkpoint.plan_sha256,
        &checkpoint.runtime_commit_sha256,
        checkpoint.runtime_cursor,
        checkpoint.control_sequence,
        &checkpoint.installed_record_sha256,
        checkpoint.at_unix_ms,
    ))
}

fn json_sha256<T: Serialize>(value: &T) -> Result<String> {
    serde_json::to_vec(value)
        .map(|bytes| digest::sha256_hex(&bytes))
        .map_err(contract_json)
}

fn canonical_prefixed(prefix: &str, sha256: &str) -> Result<CanonicalId> {
    if !is_sha256(sha256) {
        return Err(ServiceError::Contract(
            "identity seed is not SHA-256".into(),
        ));
    }
    CanonicalId::new(format!("{prefix}-{}", &sha256[..24]))
        .map_err(|error| ServiceError::Contract(error.to_string()))
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

fn zero_digest() -> String {
    "0".repeat(64)
}
