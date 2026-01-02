// Domain commands - express intent to change state
use crate::errors::{AppError, DomainResult};

/// RegisterUserCommand - Intent to create a new user
#[derive(Debug, Clone)]
pub struct RegisterUserCommand {
    pub user_id: u32,
    pub name: String,
}

impl RegisterUserCommand {
    pub fn new(user_id: u32, name: String) -> DomainResult<Self> {
        if name.trim().is_empty() {
            return Err(AppError::Validation(
                "Name cannot be empty".to_string(),
            ));
        }

        if name.len() > 255 {
            return Err(AppError::Validation(
                "Name cannot exceed 255 characters".to_string(),
            ));
        }

        if user_id == 0 {
            return Err(AppError::Validation(
                "User ID must be greater than 0".to_string(),
            ));
        }

        Ok(RegisterUserCommand { user_id, name })
    }
}

/// RenameUserCommand - Intent to rename an existing user
#[derive(Debug, Clone)]
pub struct RenameUserCommand {
    pub user_id: u32,
    pub new_name: String,
}

impl RenameUserCommand {
    pub fn new(user_id: u32, new_name: String) -> DomainResult<Self> {
        if new_name.is_empty() || new_name.trim().is_empty() {
            return Err(AppError::Validation(
                "New name cannot be empty".to_string(),
            ));
        }

        if new_name.len() > 255 {
            return Err(AppError::Validation(
                "New name cannot exceed 255 characters".to_string(),
            ));
        }

        if user_id == 0 {
            return Err(AppError::Validation(
                "User ID must be greater than 0".to_string(),
            ));
        }

        Ok(RenameUserCommand { user_id, new_name })
    }
}
