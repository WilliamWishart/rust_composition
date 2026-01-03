use axum::{extract::{State, Path}, http::StatusCode, response::IntoResponse, Json};

use crate::{dto::*, AppState};
use domain::errors::AppError;
use super::error::error_to_response;

/// Get a user by ID
/// 
/// Retrieves a single user by their unique identifier.
/// Returns 200 OK if found, 404 Not Found otherwise.
#[utoipa::path(
    get,
    path = "/users/{user_id}",
    params(
        ("user_id" = u32, Path, description = "The user's unique identifier")
    ),
    responses(
        (status = 200, description = "User found", body = UserResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
    ),
    tag = "Users"
)]
pub async fn get_user(
    State(state): State<AppState>,
    Path(user_id): Path<u32>,
) -> impl IntoResponse {
    state.logger.debug(&format!("GET /users/{}", user_id));

    match state.projection.get_user(user_id) {
        Some(user) => {
            state.logger.debug(&format!("User {} found", user_id));
            (StatusCode::OK, Json(UserResponse::from(user))).into_response()
        }
        None => {
            state.logger.debug(&format!("User {} not found", user_id));
            let err = AppError::AggregateNotFound(user_id);
            let (status, response) = error_to_response(&err);
            (status, response).into_response()
        }
    }
}

/// Fetch all users
/// 
/// Retrieves a list of all registered users.
/// Returns 200 OK with an array of users (may be empty).
#[utoipa::path(
    get,
    path = "/users",
    responses(
        (status = 200, description = "List of all users", body = Vec<UserResponse>),
    ),
    tag = "Users"
)]
pub async fn get_all_users(
    State(state): State<AppState>,
) -> impl IntoResponse {
    state.logger.debug("GET /users - fetch all users");

    let users = state.projection.get_all_users();
    let response: Vec<UserResponse> = users.into_iter().map(UserResponse::from).collect();
    
    state.logger.debug(&format!("Returning {} users", response.len()));
    (StatusCode::OK, Json(response)).into_response()
}

/// Search for a user by name
/// 
/// Finds a user by their exact name.
/// Returns 200 OK if found, 404 Not Found otherwise.
#[utoipa::path(
    get,
    path = "/users/search/{name}",
    params(
        ("name" = String, Path, description = "The user's name to search for")
    ),
    responses(
        (status = 200, description = "User found", body = UserResponse),
        (status = 404, description = "User with that name not found", body = ErrorResponse),
    ),
    tag = "Users"
)]
pub async fn find_user_by_name(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    state.logger.debug(&format!("GET /users/search/{}", name));

    match state.projection.find_by_name(&name) {
        Some(user) => {
            state.logger.debug(&format!("User '{}' found", name));
            (StatusCode::OK, Json(UserResponse::from(user))).into_response()
        }
        None => {
            state.logger.debug(&format!("User '{}' not found", name));
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: format!("User '{}' not found", name),
                }),
            )
                .into_response()
        }
    }
}
