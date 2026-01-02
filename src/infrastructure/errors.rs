use std::fmt;

/// DomainError - Type-safe error handling for the domain layer
/// Replaces String-based errors with a structured enum
/// Enables pattern matching and specific error handling at call sites
#[derive(Debug, Clone, PartialEq)]
pub enum DomainError {
    /// Concurrency violation: optimistic lock version mismatch
    ConcurrencyViolation {
        expected_version: i32,
        actual_version: i32,
    },

    /// Aggregate not found in repository
    AggregateNotFound(u32),

    /// Command validation failed
    ValidationError(String),

    /// Invalid or malformed command
    InvalidCommand(String),

    /// Event sourcing/reconstruction failed
    EventReconstructionFailed(String),

    /// Repository operation failed
    RepositoryError(String),
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DomainError::ConcurrencyViolation {
                expected_version,
                actual_version,
            } => {
                write!(
                    f,
                    "Concurrency violation: expected version {}, but aggregate version is {}",
                    expected_version, actual_version
                )
            }
            DomainError::AggregateNotFound(id) => {
                write!(f, "Aggregate not found: {}", id)
            }
            DomainError::ValidationError(msg) => {
                write!(f, "Validation error: {}", msg)
            }
            DomainError::InvalidCommand(msg) => {
                write!(f, "Invalid command: {}", msg)
            }
            DomainError::EventReconstructionFailed(msg) => {
                write!(f, "Event reconstruction failed: {}", msg)
            }
            DomainError::RepositoryError(msg) => {
                write!(f, "Repository error: {}", msg)
            }
        }
    }
}

impl std::error::Error for DomainError {}

/// Convenience type alias for Results in the domain layer
pub type DomainResult<T> = Result<T, DomainError>;
