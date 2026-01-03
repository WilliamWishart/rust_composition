// UserName Value Object
// Represents a user's name with validation and domain behavior

use std::fmt::{self, Display, Formatter};
use serde::{Deserialize, Serialize};
use crate::errors::{AppError, DomainResult};

/// UserName - Strongly-typed user name with validation
/// Enforces non-empty and length constraints
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserName(String);

impl UserName {
    pub const MIN_LENGTH: usize = 1;
    pub const MAX_LENGTH: usize = 255;

    /// Create a new UserName with validation
    pub fn new(name: String) -> DomainResult<Self> {
        let trimmed = name.trim();

        if trimmed.is_empty() {
            return Err(AppError::Validation(
                format!(
                    "Name must be at least {} character",
                    Self::MIN_LENGTH
                ),
            ));
        }

        if trimmed.len() > Self::MAX_LENGTH {
            return Err(AppError::Validation(
                format!(
                    "Name cannot exceed {} characters",
                    Self::MAX_LENGTH
                ),
            ));
        }

        Ok(UserName(trimmed.to_string()))
    }

    /// Get the string value
    pub fn value(&self) -> &str {
        &self.0
    }

    /// Domain behavior: Can this user be renamed to another name?
    /// Returns true if names are different (case-insensitive)
    pub fn can_be_renamed_to(&self, other: &UserName) -> bool {
        self.0.to_lowercase() != other.0.to_lowercase()
    }

    /// Domain behavior: Does this name match another (case-insensitive)?
    pub fn equals_ignoring_case(&self, other: &UserName) -> bool {
        self.0.to_lowercase() == other.0.to_lowercase()
    }

    /// Domain behavior: Is the name suitable for registration?
    /// (Can add business rules here, e.g., reserved words)
    pub fn is_valid_for_registration(&self) -> bool {
        // No reserved names in this simple example
        // But you could add rules like:
        // !self.0.to_lowercase().contains("admin")
        true
    }
}

impl Display for UserName {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_name_empty_is_invalid() {
        assert!(UserName::new("".to_string()).is_err());
        assert!(UserName::new("   ".to_string()).is_err());
    }

    #[test]
    fn test_user_name_valid() {
        assert!(UserName::new("Alice".to_string()).is_ok());
        assert!(UserName::new("Bob Smith".to_string()).is_ok());
    }

    #[test]
    fn test_user_name_too_long_is_invalid() {
        let long_name = "a".repeat(256);
        assert!(UserName::new(long_name).is_err());
    }

    #[test]
    fn test_user_name_max_length() {
        let max_name = "a".repeat(255);
        assert!(UserName::new(max_name).is_ok());
    }

    #[test]
    fn test_user_name_trimmed() {
        let name = UserName::new("  Alice  ".to_string()).unwrap();
        assert_eq!(name.value(), "Alice");
    }

    #[test]
    fn test_can_be_renamed_to_different_name() {
        let alice = UserName::new("Alice".to_string()).unwrap();
        let bob = UserName::new("Bob".to_string()).unwrap();
        assert!(alice.can_be_renamed_to(&bob));
    }

    #[test]
    fn test_cannot_be_renamed_to_same_name() {
        let alice1 = UserName::new("Alice".to_string()).unwrap();
        let alice2 = UserName::new("Alice".to_string()).unwrap();
        assert!(!alice1.can_be_renamed_to(&alice2));
    }

    #[test]
    fn test_cannot_be_renamed_to_same_name_case_insensitive() {
        let alice = UserName::new("Alice".to_string()).unwrap();
        let alice_upper = UserName::new("ALICE".to_string()).unwrap();
        assert!(!alice.can_be_renamed_to(&alice_upper));
    }

    #[test]
    fn test_equals_ignoring_case() {
        let alice = UserName::new("Alice".to_string()).unwrap();
        let alice_upper = UserName::new("ALICE".to_string()).unwrap();
        assert!(alice.equals_ignoring_case(&alice_upper));
    }

    #[test]
    fn test_is_valid_for_registration() {
        let name = UserName::new("Alice".to_string()).unwrap();
        assert!(name.is_valid_for_registration());
    }
}
