use super::*;

pub type Result<T> = std::result::Result<T, ServiceError>;

#[derive(Debug)]
pub enum ServiceError {
    Contract(String),
    Engram(String),
    Storage(String),
    StorageConflict(String),
    /// A proven-absent transaction commit whose read cursor is no longer current.
    TransactionSnapshotStale {
        expected: u64,
        actual: u64,
    },
    StorageAdmission(String),
    AdmissionCancelled,
    SessionNotFound,
    Unauthenticated,
    SessionExpired,
    TransactionNotFound,
    TransactionExpired,
    TransactionClosed,
    TransactionQuota,
    RenewalQuota,
    IdempotencyConflict,
    InvocationInProgress,
    CommitInProgress,
    WrongScope,
    OperationDigestMismatch,
    Query(String),
    Inference(String),
    InferenceBusy,
    ReasoningBudgetStopped(ReasoningBudgetStop),
    ReasoningAttemptRecorded {
        outcome_sha256: Option<String>,
    },
    TaskAttemptInvalidated(rrd_contract::TaskAttemptInvalidation),
    EndpointCallFailed {
        reason: &'static str,
        execution_may_continue: bool,
    },
    InferenceCancelled {
        execution_may_continue: bool,
    },
    InferenceDeadlineExceeded {
        execution_may_continue: bool,
    },
    Vector(String),
    VectorPressure(String),
    Changefeed(String),
    Subscription(String),
    SubscriptionNotFound,
    SubscriptionExpired {
        requested: u64,
        retention_floor: u64,
        head: u64,
    },
    SubscriptionLeaseExpired,
    SubscriptionClosed,
    SubscriptionConnectionReplaced,
    SubscriptionBackpressure,
    Backup(String),
    Function(String),
    FunctionLimit(String),
    ConfigurationLimit(String),
    ConfigurationUnavailable(String),
    ConfigurationConflict(String),
    FunctionNotFound,
    FunctionCatalogueRevisionNotFound,
    MemoryTargetNotFound,
    FlightNotFound,
    TaskNotFound,
    TaskClosed,
    SeatNotRepresented,
    Runtime(String),
    PermissionDenied,
    DeadlineExceeded,
    ProjectBindingMismatch,
    /// The currently observed project precondition differs from the accepted plan.
    AttunementSourceChanged,
    /// Another process holds the installed engram's single writer lock.
    EngramInUse(String),
    ClockUnavailable(ClockObservationFailure),
    ClockRollback {
        observed_at_unix_ms: u64,
        anchor_unix_ms: u64,
        maximum_rollback_ms: u64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceErrorKind {
    InvalidArgument,
    NotFound,
    Unauthenticated,
    PermissionDenied,
    Conflict,
    FailedPrecondition,
    ResourceExhausted,
    DeadlineExceeded,
    Internal,
}

impl ServiceError {
    pub fn kind(&self) -> ServiceErrorKind {
        match self {
            Self::Contract(_)
            | Self::Changefeed(_)
            | Self::Subscription(_)
            | Self::Query(_)
            | Self::Inference(_)
            | Self::Function(_)
            | Self::Runtime(_)
            | Self::Vector(_)
            | Self::OperationDigestMismatch
            | Self::WrongScope => ServiceErrorKind::InvalidArgument,
            Self::SessionNotFound
            | Self::TransactionNotFound
            | Self::SubscriptionNotFound
            | Self::FunctionNotFound
            | Self::MemoryTargetNotFound
            | Self::FlightNotFound
            | Self::TaskNotFound => ServiceErrorKind::NotFound,
            Self::Unauthenticated => ServiceErrorKind::Unauthenticated,
            Self::PermissionDenied => ServiceErrorKind::PermissionDenied,
            Self::IdempotencyConflict
            | Self::InvocationInProgress
            | Self::ConfigurationConflict(_) => ServiceErrorKind::Conflict,
            Self::SessionExpired
            | Self::TransactionExpired
            | Self::TransactionClosed
            | Self::CommitInProgress
            | Self::SubscriptionExpired { .. }
            | Self::SubscriptionLeaseExpired
            | Self::SubscriptionClosed
            | Self::TaskClosed
            | Self::SubscriptionConnectionReplaced
            | Self::FunctionCatalogueRevisionNotFound
            | Self::SeatNotRepresented
            | Self::ClockUnavailable(_)
            | Self::ClockRollback { .. }
            | Self::AdmissionCancelled
            | Self::InferenceCancelled { .. }
            | Self::ReasoningBudgetStopped(_)
            | Self::ReasoningAttemptRecorded { .. }
            | Self::TaskAttemptInvalidated(_)
            | Self::EndpointCallFailed { .. }
            | Self::ProjectBindingMismatch
            | Self::AttunementSourceChanged
            | Self::EngramInUse(_)
            | Self::ConfigurationUnavailable(_) => ServiceErrorKind::FailedPrecondition,
            Self::TransactionQuota
            | Self::StorageAdmission(_)
            | Self::InferenceBusy
            | Self::RenewalQuota
            | Self::SubscriptionBackpressure
            | Self::FunctionLimit(_)
            | Self::ConfigurationLimit(_)
            | Self::VectorPressure(_) => ServiceErrorKind::ResourceExhausted,
            Self::DeadlineExceeded | Self::InferenceDeadlineExceeded { .. } => {
                ServiceErrorKind::DeadlineExceeded
            }
            Self::StorageConflict(_) | Self::TransactionSnapshotStale { .. } => {
                ServiceErrorKind::Conflict
            }
            Self::Storage(_) | Self::Backup(_) | Self::Engram(_) => ServiceErrorKind::Internal,
        }
    }

    pub fn retryable(&self) -> bool {
        matches!(
            self,
            Self::StorageConflict(_) | Self::InvocationInProgress | Self::InferenceBusy
        )
    }
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ClockUnavailable(failure) => {
                write!(f, "host clock is unavailable: {failure:?}")
            }
            Self::ClockRollback {
                observed_at_unix_ms,
                anchor_unix_ms,
                maximum_rollback_ms,
            } => write!(
                f,
                "host clock rollback exceeds policy: observed {observed_at_unix_ms}, installation anchor {anchor_unix_ms}, maximum rollback {maximum_rollback_ms} ms"
            ),
            Self::EngramInUse(detail) => {
                write!(f, "the installed engram is already open in another process ({detail})")
            }
            _ => write!(f, "{self:?}"),
        }
    }
}

impl std::error::Error for ServiceError {}

impl From<rrflow_db::Error> for ServiceError {
    fn from(value: rrflow_db::Error) -> Self {
        let message = value.to_string();
        match value {
            // Owner exhaustion or source-page pressure can obstruct an already
            // published commit. Only a proven-absent outcome permits rejection.
            rrflow_db::Error::TransactionLimit { .. }
            | rrflow_db::Error::SegmentHandleLimit { .. }
            | rrflow_db::Error::SourcePageLimit { .. }
            | rrflow_db::Error::ProjectedReadLimit { .. } => Self::Storage(message),
            rrflow_db::Error::WriteBatchLimit { .. }
            | rrflow_db::Error::MutableLimit { .. }
            | rrflow_db::Error::MaintenanceLimit { .. }
            | rrflow_db::Error::DiskAdmission { .. }
            | rrflow_db::Error::CheckpointLimit { .. }
            | rrflow_db::Error::RecoveryLimit { .. }
            | rrflow_db::Error::ReclamationPending
            | rrflow_db::Error::ReclamationLimit { .. } => Self::StorageAdmission(message),
            rrflow_db::Error::RuntimeReadBudgetExceeded { .. } => Self::ConfigurationLimit(message),
            rrflow_db::Error::IndexConstraint(_) => Self::Query(message),
            rrflow_db::Error::ActiveWriter(_) => Self::EngramInUse(message),
            rrflow_db::Error::RuntimeConflict { .. }
            | rrflow_db::Error::RuntimeSchemaConflict { .. }
            | rrflow_db::Error::IdempotencyConflict(_)
            | rrflow_db::Error::ControlConflict(_)
            | rrflow_db::Error::ReadStampMismatch(_) => Self::StorageConflict(message),
            _ => Self::Storage(message),
        }
    }
}

impl From<rrflow_engram::Error> for ServiceError {
    fn from(value: rrflow_engram::Error) -> Self {
        Self::Engram(value.to_string())
    }
}

impl From<rrflow_security::Error> for ServiceError {
    fn from(value: rrflow_security::Error) -> Self {
        match value {
            rrflow_security::Error::Unauthenticated | rrflow_security::Error::PrincipalNotFound => {
                Self::Unauthenticated
            }
            rrflow_security::Error::PermissionDenied | rrflow_security::Error::NotInitialized => {
                Self::PermissionDenied
            }
            rrflow_security::Error::Invalid(message) => Self::Contract(message),
            rrflow_security::Error::IdempotencyConflict
            | rrflow_security::Error::AlreadyInitialized => Self::IdempotencyConflict,
            rrflow_security::Error::Store(error) => Self::from(error),
        }
    }
}

impl From<rrflow_cld::Error> for ServiceError {
    fn from(error: rrflow_cld::Error) -> Self {
        match error {
            rrflow_cld::Error::Budget(message) => Self::ConfigurationLimit(message),
            rrflow_cld::Error::DeadlineExceeded(_) => Self::DeadlineExceeded,
            other => Self::Storage(other.to_string()),
        }
    }
}

impl From<rrflow_ql::Error> for ServiceError {
    fn from(error: rrflow_ql::Error) -> Self {
        match error {
            rrflow_ql::Error::StorageAdmission(message) => Self::StorageAdmission(message),
            rrflow_ql::Error::Budget(message) => Self::ConfigurationLimit(message),
            rrflow_ql::Error::DeadlineExceeded(_) => Self::DeadlineExceeded,
            other => Self::Query(other.to_string()),
        }
    }
}

/// Preserve the existing query error surface while carrying native admission
/// through storage reads that precede the query pipeline.
pub(super) fn query_storage_error(error: rrflow_db::Error) -> ServiceError {
    match error {
        rrflow_db::Error::SegmentHandleLimit { .. }
        | rrflow_db::Error::SourcePageLimit { .. }
        | rrflow_db::Error::ProjectedReadLimit { .. } => {
            ServiceError::StorageAdmission(error.to_string())
        }
        other => ServiceError::Query(other.to_string()),
    }
}

/// Known read-only acquisition can report pressure without asserting a durable
/// effect was rejected. Durable callers must use the ordinary conversion.
pub(super) fn storage_read_error(error: rrflow_db::Error) -> ServiceError {
    match error {
        rrflow_db::Error::SegmentHandleLimit { .. }
        | rrflow_db::Error::SourcePageLimit { .. }
        | rrflow_db::Error::ProjectedReadLimit { .. } => {
            ServiceError::StorageAdmission(error.to_string())
        }
        other => ServiceError::from(other),
    }
}
