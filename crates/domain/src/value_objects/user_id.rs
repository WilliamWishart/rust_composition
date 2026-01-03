// UserId Value Object
// Represents a user's unique identifier with validation

use std::fmt;
use serde::{Deserialize, Serialize};
use crate::errors::{AppError, DomainResult};

/// UserId - Strongly-typed user identifier
/// Ensures user IDs are always > 0
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(u32);

impl UserId {
    /// Create a new UserId with validation
    pub fn new(id: u32) -> DomainResult<Self> {
        if id == 0 {
            Err(AppError::Validation(
                "User ID must be greater than 0".to_string(),
            ))
        } else {
            Ok(UserId(id))
        }
    }

    /// Get the raw u32 value
    pub fn value(&self) -> u32 {
        self.0
    }
}

impl From<u32> for UserId {
    fn from(id: u32) -> Self {
        UserId(id)
    }
}

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_id_zero_is_invalid() {
        assert!(UserId::new(0).is_err());
    }

    #[test]
    fn test_user_id_positive_is_valid() {
        assert!(UserId::new(1).is_ok());
        assert!(UserId::new(u32::MAX).is_ok());
    }

    #[test]
    fn test_user_id_equality() {
        let id1 = UserId::new(1).unwrap();
        let id2 = UserId::new(1).unwrap();
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_user_id_value_getter() {
        let id = UserId::new(42).unwrap();
        assert_eq!(id.value(), 42);
    }

    #[test]
    fn test_user_id_from_u32() {
        let id: UserId = 100u32.into();
        assert_eq!(id.value(), 100);
    }
}
