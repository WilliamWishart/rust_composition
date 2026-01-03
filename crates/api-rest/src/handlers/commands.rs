use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

use crate::{dto::*, AppState};
use domain::commands::{RegisterUserCommand, RenameUserCommand};
use super::error::error_to_response;

/// Register a new user
/// 
/// Creates a new user with the provided ID and name.
/// Returns 201 Created on success.
#[utoipa::path(
    post,
    path = "/users",
    request_body = RegisterUserRequest,
    responses(
        (status = 201, description = "User registered successfully", body = SuccessResponse),
        (status = 409, description = "User with this ID already exists", body = ErrorResponse),
        (status = 422, description = "Invalid user data (ID must be > 0, name 1-255 chars)", body = ErrorResponse),
    ),
    tag = "Users"
)]
pub async fn register_user(
    State(state): State<AppState>,
    Json(payload): Json<RegisterUserRequest>,
) -> impl IntoResponse {
    state.logger.debug(&format!(
        "POST /users - register user {}",
        payload.user_id
    ));

    // Create command - validation happens in domain layer
    let command = match RegisterUserCommand::new(payload.user_id, payload.name.clone()) {
        Ok(cmd) => cmd,
        Err(err) => {
            state.logger.error(&format!("Invalid register command: {:?}", err));
            let (status, response) = error_to_response(&err);
            return (status, response).into_response();
        }
    };

    match state.command_handler.handle_register_user(command).await {
        Ok(_) => {
            state.logger.info(&format!(
                "User {} registered successfully",
                payload.user_id
            ));
            (
                StatusCode::CREATED,
                Json(SuccessResponse {
                    message: format!("User {} registered successfully", payload.user_id),
                }),
            )
                .into_response()
        }
        Err(err) => {
            state.logger.error(&format!("Failed to register user: {:?}", err));
            let (status, response) = error_to_response(&err);
            (status, response).into_response()
        }
    }
}

/// Rename an existing user
/// 
/// Updates the name of an existing user.
/// Returns 200 OK on success.
#[utoipa::path(
    put,
    path = "/users",
    request_body = RenameUserRequest,
    responses(
        (status = 200, description = "User renamed successfully", body = SuccessResponse),
        (status = 404, description = "User with this ID not found", body = ErrorResponse),
        (status = 409, description = "Concurrency violation (user was modified)", body = ErrorResponse),
        (status = 422, description = "Invalid user data (ID must be > 0, name 1-255 chars)", body = ErrorResponse),
    ),
    tag = "Users"
)]
pub async fn rename_user(
    State(state): State<AppState>,
    Json(payload): Json<RenameUserRequest>,
) -> impl IntoResponse {
    state.logger.debug(&format!(
        "PUT /users/{} - rename to {}",
        payload.user_id, payload.new_name
    ));

    // Create command - validation happens in domain layer
    let command = match RenameUserCommand::new(payload.user_id, payload.new_name.clone()) {
        Ok(cmd) => cmd,
        Err(err) => {
            state.logger.error(&format!("Invalid rename command: {:?}", err));
            let (status, response) = error_to_response(&err);
            return (status, response).into_response();
        }
    };

    match state.command_handler.handle_rename_user(command).await {
        Ok(_) => {
            state.logger.info(&format!(
                "User {} renamed successfully",
                payload.user_id
            ));
            (
                StatusCode::OK,
                Json(SuccessResponse {
                    message: format!("User {} renamed successfully", payload.user_id),
                }),
            )
                .into_response()
        }
        Err(err) => {
            state.logger.error(&format!("Failed to rename user: {:?}", err));
            let (status, response) = error_to_response(&err);
            (status, response).into_response()
        }
    }
}
