use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::AppState;
use domain::commands::{RegisterUserCommand, RenameUserCommand};

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterUserRequest {
    pub user_id: u32,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RenameUserRequest {
    pub user_id: u32,
    pub new_name: String,
}

#[derive(Debug, Serialize)]
pub struct SuccessResponse {
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

pub async fn register_user(
    State(state): State<AppState>,
    Json(payload): Json<RegisterUserRequest>,
) -> impl IntoResponse {
    state.logger.info(&format!(
        "REST API: POST /users - register user {}",
        payload.user_id
    ));

    let command = RegisterUserCommand {
        user_id: payload.user_id,
        name: payload.name.clone(),
    };

    match state.command_handler.handle_register_user(command).await {
        Ok(_) => {
            state.logger.info(&format!(
                "REST API: User {} registered successfully",
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
            let error_msg = format!("{:?}", err);
            state.logger.error(&format!("REST API: Failed to register user: {}", error_msg));
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: error_msg,
                }),
            )
                .into_response()
        }
    }
}

pub async fn rename_user(
    State(state): State<AppState>,
    Json(payload): Json<RenameUserRequest>,
) -> impl IntoResponse {
    state.logger.info(&format!(
        "REST API: PUT /users/{} - rename to {}",
        payload.user_id, payload.new_name
    ));

    let command = RenameUserCommand {
        user_id: payload.user_id,
        new_name: payload.new_name.clone(),
    };

    match state.command_handler.handle_rename_user(command).await {
        Ok(_) => {
            state.logger.info(&format!(
                "REST API: User {} renamed successfully",
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
            let error_msg = format!("{:?}", err);
            state.logger.error(&format!("REST API: Failed to rename user: {}", error_msg));
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: error_msg,
                }),
            )
                .into_response()
        }
    }
}
