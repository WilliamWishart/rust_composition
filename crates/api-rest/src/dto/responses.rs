use serde::Serialize;
use persistence::projections::UserReadModel;
use utoipa::ToSchema;

/// UserResponse - API response for a user
#[derive(Debug, Serialize, ToSchema)]
pub struct UserResponse {
    /// User's unique identifier
    pub id: u32,
    /// User's name
    pub name: String,
    /// Timestamp when user was created (Unix timestamp in milliseconds)
    pub created_at: i64,
}

impl From<UserReadModel> for UserResponse {
    fn from(model: UserReadModel) -> Self {
        UserResponse {
            id: model.id,
            name: model.name,
            created_at: model.created_at,
        }
    }
}

/// SuccessResponse - Standard success response for mutations
#[derive(Debug, Serialize, ToSchema)]
pub struct SuccessResponse {
    /// Success message describing the operation result
    pub message: String,
}

/// ErrorResponse - Standard error response
#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    /// Error message describing what went wrong
    pub error: String,
}
