use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Lightpanda error: {0}")]
    Lightpanda(String),

    #[error("Search error: {0}")]
    Search(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Invalid request: {0}")]
    BadRequest(String),

    #[error("Timeout occurred: {0}")]
    Timeout(String),

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Timeout(msg) => (StatusCode::GATEWAY_TIMEOUT, msg),
            AppError::Lightpanda(msg) => (StatusCode::BAD_GATEWAY, msg),
            AppError::Search(msg) => (StatusCode::BAD_GATEWAY, msg),
            AppError::Parse(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg),
            AppError::Internal(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
        };

        let body = Json(json!({
            "error": {
                "message": error_message,
                "status": status.as_u16(),
            }
        }));

        (status, body).into_response()
    }
}
