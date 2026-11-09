use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

pub enum AppError {
    BadRequest(String),
    NotFound(String),
    Conflict(String),
    Internal,
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        if let Some(db) = err.as_database_error() {
            if db.is_unique_violation() {
                return AppError::Conflict("el recurso ya existe".to_string());
            }
        }

        eprintln!("error de base de datos: {err}");
        AppError::Internal
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            AppError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, "error interno".to_string()),
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}