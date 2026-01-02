use axum::http::StatusCode;
use axum::Json;

use crate::dto::ErrorResponse;
use domain::errors::AppError;

/// Maps AppError to appropriate HTTP status codes and error responses
pub fn error_to_response(err: &AppError) -> (StatusCode, Json<ErrorResponse>) {
    let (status, message) = match err {
        AppError::Validation(msg) => {
            (StatusCode::UNPROCESSABLE_ENTITY, msg.clone())
        }
        AppError::AggregateNotFound(id) => {
            (StatusCode::NOT_FOUND, format!("User {} not found", id))
        }
        AppError::ConcurrencyViolation {
            expected_version,
            actual_version,
        } => {
            (
                StatusCode::CONFLICT,
                format!(
                    "Version mismatch: expected {}, got {}",
                    expected_version, actual_version
                ),
            )
        }
        AppError::HandlerError {
            message,
            is_critical,
            ..
        } => {
            if *is_critical {
                (StatusCode::INTERNAL_SERVER_ERROR, message.clone())
            } else {
                (StatusCode::BAD_REQUEST, message.clone())
            }
        }
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal server error".to_string(),
        ),
    };
    (status, Json(ErrorResponse { error: message }))
}
