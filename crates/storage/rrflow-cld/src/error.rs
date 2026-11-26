use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Binding(String),
    Budget(String),
    DeadlineExceeded(String),
    Execution(String),
    Integrity(String),
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (kind, detail) = match self {
            Self::Binding(value) => ("binding", value),
            Self::Budget(value) => ("budget", value),
            Self::DeadlineExceeded(value) => ("deadline", value),
            Self::Execution(value) => ("execution", value),
            Self::Integrity(value) => ("integrity", value),
        };
        write!(formatter, "RRFlow cold storage {kind} error: {detail}")
    }
}

impl std::error::Error for Error {}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::Execution(value.to_string())
    }
}

impl From<rrflow_arrow::Error> for Error {
    fn from(value: rrflow_arrow::Error) -> Self {
        match value {
            rrflow_arrow::Error::Binding(message) => Self::Binding(message),
            rrflow_arrow::Error::Budget(message) => Self::Budget(message),
            rrflow_arrow::Error::Execution(message) => Self::Execution(message),
            rrflow_arrow::Error::Integrity(message) => Self::Integrity(message),
        }
    }
}
