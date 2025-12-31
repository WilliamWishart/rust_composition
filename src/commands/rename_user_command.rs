/// RenameUserCommand - Command to rename an existing user
/// Commands are imperative - they express intent to change state
/// Commands can be rejected (validation)
#[derive(Debug, Clone)]
pub struct RenameUserCommand {
    pub user_id: u32,
    pub new_name: String,
}

impl RenameUserCommand {
    pub fn new(user_id: u32, new_name: String) -> Result<Self, String> {
        if new_name.is_empty() || new_name.trim().is_empty() {
            return Err("New name cannot be empty".to_string());
        }
        Ok(RenameUserCommand { user_id, new_name })
    }
}
