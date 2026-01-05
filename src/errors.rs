use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    Database(sqlx::Error),
    Validation(String),
    Authentication(String),
    Authorization(String),
    NotFound(String),
    Conflict(String),
    Internal(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Database(e) => write!(f, "Database error: {}", e),
            AppError::Validation(msg) => write!(f, "Validation error: {}", msg),
            AppError::Authentication(msg) => write!(f, "Authentication error: {}", msg),
            AppError::Authorization(msg) => write!(f, "Authorization error: {}", msg),
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::Conflict(msg) => write!(f, "Conflict: {}", msg),
            AppError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    code: String,
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let (status, error_code) = match self {
            AppError::Database(_) => (actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, "database_error"),
            AppError::Validation(_) => (actix_web::http::StatusCode::BAD_REQUEST, "validation_error"),
            AppError::Authentication(_) => (actix_web::http::StatusCode::UNAUTHORIZED, "authentication_error"),
            AppError::Authorization(_) => (actix_web::http::StatusCode::FORBIDDEN, "authorization_error"),
            AppError::NotFound(_) => (actix_web::http::StatusCode::NOT_FOUND, "not_found"),
            AppError::Conflict(_) => (actix_web::http::StatusCode::CONFLICT, "conflict"),
            AppError::Internal(_) => (actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
        };

        let error_msg = match self {
            AppError::Database(_) => "Internal server error".to_string(),
            AppError::Internal(_) => "Internal server error".to_string(),
            _ => self.to_string(),
        };

        HttpResponse::build(status).json(ErrorResponse {
            error: error_msg,
            code: error_code.to_string(),
        })
    }
}

pub type AppResult<T> = Result<T, AppError>;

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => AppError::NotFound("Resource not found".to_string()),
            _ => AppError::Database(err),
        }
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        AppError::Authentication(format!("JWT error: {}", err))
    }
}

impl From<String> for AppError {
    fn from(err: String) -> Self {
        AppError::Internal(err)
    }
}

impl From<&str> for AppError {
    fn from(err: &str) -> Self {
        AppError::Internal(err.to_string())
    }
}
