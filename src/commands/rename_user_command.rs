use crate::infrastructure::{DomainError, DomainResult};

/// RenameUserCommand - Command to rename an existing user
/// Commands are imperative - they express intent to change state
/// Commands can be rejected (validation)
#[derive(Debug, Clone)]
pub struct RenameUserCommand {
    pub user_id: u32,
    pub new_name: String,
}

impl RenameUserCommand {
    pub fn new(user_id: u32, new_name: String) -> DomainResult<Self> {
        // Name validation
        if new_name.is_empty() || new_name.trim().is_empty() {
            return Err(DomainError::Validation(
                "New name cannot be empty".to_string(),
            ));
        }

        if new_name.len() > 255 {
            return Err(DomainError::Validation(
                "New name cannot exceed 255 characters".to_string(),
            ));
        }

        // User ID validation
        if user_id == 0 {
            return Err(DomainError::Validation(
                "User ID must be greater than 0".to_string(),
            ));
        }

        Ok(RenameUserCommand { user_id, new_name })
    }
}
