use crate::infrastructure::{DomainError, DomainResult};

/// RegisterUserCommand - Command to register a new user
/// Commands are imperative - they express intent to change state
/// Commands can be rejected (validation)
#[derive(Debug, Clone)]
pub struct RegisterUserCommand {
    pub user_id: u32,
    pub name: String,
}

impl RegisterUserCommand {
    pub fn new(user_id: u32, name: String) -> DomainResult<Self> {
        if name.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "Name cannot be empty".to_string(),
            ));
        }

        if user_id == 0 {
            return Err(DomainError::ValidationError(
                "User ID must be greater than 0".to_string(),
            ));
        }

        Ok(RegisterUserCommand { user_id, name })
    }
}
