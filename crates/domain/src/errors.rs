use std::fmt;
use std::error::Error;

/// AppError - Unified error type across the entire application
/// Consolidates domain, handler, and publish errors into a single type
#[derive(Debug, Clone, PartialEq)]
pub enum AppError {
    /// Command or input validation failed
    Validation(String),

    /// Concurrency violation: optimistic lock version mismatch
    ConcurrencyViolation {
        expected_version: i32,
        actual_version: i32,
    },

    /// Aggregate not found in repository
    AggregateNotFound(u32),

    /// Event sourcing/reconstruction failed
    EventReconstructionFailed(String),

    /// Repository operation failed
    RepositoryError(String),

    /// Handler execution failed
    HandlerError {
        handler_name: String,
        message: String,
        is_critical: bool,
    },

    /// Event publishing failed
    PublishError(String),

    /// Lock poisoned (internal error)
    LockPoisoned,
}

/// Legacy alias for backward compatibility
pub type DomainError = AppError;
pub type DomainResult<T> = Result<T, AppError>;

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Validation(msg) => {
                write!(f, "Validation error: {}", msg)
            }
            AppError::ConcurrencyViolation {
                expected_version,
                actual_version,
            } => {
                write!(
                    f,
                    "Concurrency violation: expected version {}, but aggregate version is {}",
                    expected_version, actual_version
                )
            }
            AppError::AggregateNotFound(id) => {
                write!(f, "Aggregate not found: {}", id)
            }
            AppError::EventReconstructionFailed(msg) => {
                write!(f, "Event reconstruction failed: {}", msg)
            }
            AppError::RepositoryError(msg) => {
                write!(f, "Repository error: {}", msg)
            }
            AppError::HandlerError {
                handler_name,
                message,
                is_critical,
            } => {
                let severity = if *is_critical { "CRITICAL" } else { "NON-CRITICAL" };
                write!(f, "Handler '{}' failed [{}]: {}", handler_name, severity, message)
            }
            AppError::PublishError(msg) => {
                write!(f, "Publish error: {}", msg)
            }
            AppError::LockPoisoned => {
                write!(f, "Internal error: lock poisoned")
            }
        }
    }
}

impl Error for AppError {}
