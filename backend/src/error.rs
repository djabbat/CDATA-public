use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use tracing::error;
use validator::ValidationErrors;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("Migration error: {0}")]
    Migration(String),
    
    #[error("Server error: {0}")]
    Server(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationErrors),
    
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    
    #[error("Bad request: {0}")]
    BadRequest(String),
    
    #[error("Internal server error")]
    Internal,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Configuration(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::Database(msg) => {
                error!("Database error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
            }
            AppError::Migration(msg) => {
                error!("Migration error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Migration error".to_string())
            }
            AppError::Server(msg) => {
                error!("Server error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Server error".to_string())
            }
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Validation(err) => {
                let errors = err.field_errors()
                    .into_iter()
                    .map(|(field, errors)| {
                        let messages: Vec<String> = errors
                            .iter()
                            .map(|e| e.message.clone().unwrap_or_default().to_string())
                            .collect();
                        json!({field: messages})
                    })
                    .collect::<Vec<_>>();
                return (StatusCode::BAD_REQUEST, Json(json!({"errors": errors}))).into_response();
            }
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Internal => {
                error!("Internal server error");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
        };

        let body = Json(json!({
            "error": error_message,
            "status": status.as_u16(),
        }));

        (status, body).into_response()
    }
}