/// RegisterUserCommand - Command to register a new user
/// Commands are imperative - they express intent to change state
/// Commands can be rejected (validation)
#[derive(Debug, Clone)]
pub struct RegisterUserCommand {
    pub user_id: u32,
    pub name: String,
}

impl RegisterUserCommand {
    pub fn new(user_id: u32, name: String) -> Result<Self, String> {
        if name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        Ok(RegisterUserCommand { user_id, name })
    }
}
