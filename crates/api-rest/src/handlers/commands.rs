use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

use crate::{dto::*, AppState};
use domain::commands::{RegisterUserCommand, RenameUserCommand};
use super::error::error_to_response;

/// POST /users - Register a new user
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

/// PUT /users - Rename an existing user
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
