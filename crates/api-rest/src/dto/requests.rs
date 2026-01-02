use serde::Deserialize;

/// RegisterUserRequest - Request payload for creating a new user
#[derive(Debug, Deserialize)]
pub struct RegisterUserRequest {
    pub user_id: u32,
    pub name: String,
}

/// RenameUserRequest - Request payload for renaming a user
#[derive(Debug, Deserialize)]
pub struct RenameUserRequest {
    pub user_id: u32,
    pub new_name: String,
}
