//! The single RRFlow engine composition root.
//!
//! Physical storage, query, vector, engram, and security components are
//! composed here. Daemon and embedded faces consume this API instead of
//! constructing those components independently.

/// The temporal values and invariants shared by the engine and its storage ports.
pub use rrflow_kernel as kernel;

#[cfg(feature = "full")]
mod capabilities;
pub mod edge;
#[cfg(feature = "crash-test")]
#[doc(hidden)]
pub use rrd_store::crash_test;
#[cfg(feature = "full")]
mod engine;
#[cfg(feature = "full")]
mod operator;
#[cfg(feature = "full")]
pub mod runtime;

#[cfg(feature = "full")]
pub use capabilities::product_capability_catalogue;
pub use edge::{OfflineDocument, OfflineEdgeConfig, OfflineEdgeIndex, OfflineQueryResult};
/// Internal entry for the primary executable's isolated parsing child.
/// The caller must start it with native resource and lifetime enforcement.
#[cfg(feature = "full")]
#[doc(hidden)]
pub fn run_internal_parse_worker() -> std::io::Result<()> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    rrd_attunement::worker::run_worker(&mut stdin.lock(), &mut stdout.lock())
}
#[cfg(feature = "full")]
pub use engine::{
    attunement_proposal_schema, load_attunement_proposal, load_engram_configuration,
    AttunementCommand, AttunementControlResult, AttunementProposal, AttunementProposalStep,
    AttunementReceiptView, AttunementReplanPreview, AttunementReplanReceiptView,
    AttunementReplanResult, AuthorizedInvocation, ClockAssessment, ClockAssessmentStatus,
    ClockObservation, ClockObservationFailure, ClockSourceKind, ClockTrustLevel,
    DefaultAttunementPhase, DefaultInstallProfile, DistributedAuthorityRead, DistributedReadRoute,
    EndpointCredentials, EngineServingStop, EngramAdminAction, EngramAdminResult,
    EngramBackupReconcileOutcome, EngramReconcileOutcome, FollowOutcome, FollowTick,
    InstallationPlanningOptions, InstallationPreview, InstallationVerificationCheck,
    InstallationVerificationReport, InstallationVerificationStatus, InstalledEngramRecord,
    InstalledTailRepairPlan, InstalledTailRepairReceipt, InstanceRegistration, InstanceRegistry,
    InstanceRegistryEntry, InstanceRegistryListing, InstanceStatus, Invocation,
    InvocationCompletion, InvocationCredential, NestedRepositoryDiscovery,
    PreparedDistributedAuthority, ProjectLocator, ProjectMapSummary, ReasoningBudgetStop, Result,
    RrdEngine, RrdOperation, ServiceError, ServiceErrorKind, ServingPublisher, ServingRecord,
    ServingRegistration, ServingStopRequest, ServingStopState, TaskEmbeddingAttempt,
    TaskReasoningBudget, TransactionAdmissionSnapshot, UnreadableInstanceEntry,
    ATTUNEMENT_PROPOSAL_FORMAT_VERSION, CLOCK_OBSERVATION_FORMAT_VERSION,
    DEFAULT_INSTANCE_REGISTRY_LOCK_WAIT_MS, DEFAULT_STOP_POLL_INTERVAL_MS, DEFAULT_STOP_WAIT_MS,
    INSTALLATION_PREVIEW_FORMAT_VERSION, INSTANCE_REGISTRY_ENVIRONMENT,
    INSTANCE_REGISTRY_LOCK_WAIT_ENVIRONMENT, MAX_AUDIT_PAGE_RECORDS, MAX_JWT_CREDENTIAL_BYTES,
    PROJECT_MAP_RELATIVE_PATH, TOKEN_KEY_BYTES,
};
#[cfg(feature = "full")]
pub use operator::{
    digest, BackupCatalogue, BackupEntry, Claim, ClaimReader, LogicalArchiveInventory,
    LogicalRestoreReport, Millis, OperatorInvocation, OperatorInvocationInput, OperatorResult,
    Outcome, Predicate, Producer, Reader, RemovalReport, ScopeId, Subject, Trigger,
};
#[cfg(feature = "full")]
pub use runtime::*;
