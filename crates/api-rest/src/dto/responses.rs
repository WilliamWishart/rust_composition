use serde::Serialize;
use persistence::projections::UserReadModel;

/// UserResponse - API response for a user
#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: u32,
    pub name: String,
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
#[derive(Debug, Serialize)]
pub struct SuccessResponse {
    pub message: String,
}

/// ErrorResponse - Standard error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}
