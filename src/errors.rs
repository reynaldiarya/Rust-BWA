use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use serde_json::json;
use thiserror::Error;
use validator::ValidationErrors;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("SQLx Error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("Not found")]
    NotFound,

    #[error("Validation error: {0}")]
    Validation(#[from] ValidationErrors),

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

fn format_validation_errors(errors: &ValidationErrors) -> String {
    let mut error_messages = Vec::new();

    for (field, field_errors) in errors.field_errors() {
        for error in field_errors {
            if let Some(message) = &error.message {
                error_messages.push(format!("{}: {}", field, message));
            } else {
                error_messages.push(format!("{}: validation failed", field));
            }
        }
    }

    if error_messages.is_empty() {
        "Validation failed".to_string()
    } else {
        error_messages.join(", ")
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Sqlx(sqlx::Error::RowNotFound) | AppError::NotFound => {
                (StatusCode::NOT_FOUND, "Not found".to_string())
            }

            AppError::Sqlx(e) => {
                tracing::error!("SQLx error: {:?}", e); // Log error untuk debugging
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }

            AppError::Validation(e) => {
                tracing::warn!("Validation error: {:?}", e); // Log warning
                (StatusCode::BAD_REQUEST, format_validation_errors(&e))
            }

            AppError::Internal(e) => {
                tracing::error!("Internal error: {:?}", e); // Log error untuk debugging
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
        };

        (
            status,
            Json(json!({
                "error": error_message,
            })),
        )
            .into_response()
    }
}

pub type AppResponse<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body;
    use axum::response::IntoResponse;
    use serde_json::Value;

    #[tokio::test]
    async fn validation_error_should_return_400() {
        let mut errors = ValidationErrors::new();
        errors.add("name", validator::ValidationError::new("invalid"));
        let err = AppError::Validation(errors);

        let response = err.into_response();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = response.into_body();
        let bytes = body::to_bytes(body, 1024 * 1024).await.unwrap();
        let json: Value = serde_json::from_slice(&bytes).unwrap();

        assert!(json.get("error").is_some());
    }

    #[test]
    fn not_found_error_should_return_404() {
        let response = AppError::NotFound.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
