// EmailAddress Value Object
// Represents an email with validation (optional but future-proofing)

use std::fmt::{self, Display, Formatter};
use serde::{Deserialize, Serialize};
use crate::errors::{AppError, DomainResult};

/// EmailAddress - Strongly-typed email with basic validation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmailAddress(String);

impl EmailAddress {
    /// Create a new EmailAddress with basic validation
    pub fn new(email: String) -> DomainResult<Self> {
        let trimmed = email.trim();

        if trimmed.is_empty() {
            return Err(AppError::Validation(
                "Email cannot be empty".to_string(),
            ));
        }

        // Simple email validation: must contain @
        if !trimmed.contains('@') {
            return Err(AppError::Validation(
                "Invalid email format: must contain @".to_string(),
            ));
        }

        let parts: Vec<&str> = trimmed.split('@').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err(AppError::Validation(
                "Invalid email format".to_string(),
            ));
        }

        Ok(EmailAddress(trimmed.to_string()))
    }

    /// Get the string value
    pub fn value(&self) -> &str {
        &self.0
    }

    /// Get the domain part of the email
    pub fn domain(&self) -> &str {
        self.0.split('@').nth(1).unwrap_or("")
    }

    /// Domain behavior: Does this email match another (case-insensitive)?
    pub fn equals_ignoring_case(&self, other: &EmailAddress) -> bool {
        self.0.to_lowercase() == other.0.to_lowercase()
    }
}

impl Display for EmailAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_valid() {
        assert!(EmailAddress::new("alice@example.com".to_string()).is_ok());
        assert!(EmailAddress::new("user+tag@domain.co.uk".to_string()).is_ok());
    }

    #[test]
    fn test_email_empty_is_invalid() {
        assert!(EmailAddress::new("".to_string()).is_err());
        assert!(EmailAddress::new("   ".to_string()).is_err());
    }

    #[test]
    fn test_email_missing_at_is_invalid() {
        assert!(EmailAddress::new("alice.example.com".to_string()).is_err());
    }

    #[test]
    fn test_email_missing_local_part_is_invalid() {
        assert!(EmailAddress::new("@example.com".to_string()).is_err());
    }

    #[test]
    fn test_email_missing_domain_is_invalid() {
        assert!(EmailAddress::new("alice@".to_string()).is_err());
    }

    #[test]
    fn test_email_trimmed() {
        let email = EmailAddress::new("  alice@example.com  ".to_string()).unwrap();
        assert_eq!(email.value(), "alice@example.com");
    }

    #[test]
    fn test_email_domain() {
        let email = EmailAddress::new("alice@example.com".to_string()).unwrap();
        assert_eq!(email.domain(), "example.com");
    }

    #[test]
    fn test_email_equals_ignoring_case() {
        let email1 = EmailAddress::new("Alice@Example.com".to_string()).unwrap();
        let email2 = EmailAddress::new("alice@example.com".to_string()).unwrap();
        assert!(email1.equals_ignoring_case(&email2));
    }
}
