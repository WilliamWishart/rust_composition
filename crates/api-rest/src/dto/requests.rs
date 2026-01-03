use serde::Deserialize;
use utoipa::ToSchema;

/// RegisterUserRequest - Request payload for creating a new user
#[derive(Debug, Deserialize, ToSchema)]
#[schema(example = json!({"user_id": 1, "name": "Alice"}))]
pub struct RegisterUserRequest {
    /// Unique user identifier (must be > 0)
    pub user_id: u32,
    /// User's name (must be 1-255 characters)
    pub name: String,
}

/// RenameUserRequest - Request payload for renaming a user
#[derive(Debug, Deserialize, ToSchema)]
#[schema(example = json!({"user_id": 1, "new_name": "Bob"}))]
pub struct RenameUserRequest {
    /// User identifier to rename
    pub user_id: u32,
    /// New user name (must be 1-255 characters)
    pub new_name: String,
}
