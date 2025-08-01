use actix_web::{
    error, get,
    http::{header::ContentType, StatusCode},
    App, HttpResponse,
};
use thiserror::Error;
use crate::errors::ServicesError;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("internal server error: {0}")]
    InternalServerError(String),
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("unauthorized: {0}")]
    Unauthorized(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("authentication failed: {0}")]
    AuthenticationFailed(String),
}

impl From<ServicesError> for AppError {
    fn from(err: ServicesError) -> AppError {
        match err {
            ServicesError::AuthenticationFailed { reason } => {
                AppError::AuthenticationFailed(reason)
            },
            ServicesError::InvalidInput { field, message } => {
                AppError::BadRequest(format!("Invalid {}: {}", field, message))
            },
            ServicesError::NotFound { what, identifier } => {
                AppError::NotFound(format!("{} '{}' not found", what, identifier))
            },
            ServicesError::AlreadyExists { what, identifier } => {
                AppError::Conflict(format!("{} '{}' already exists", what, identifier))
            },
            ServicesError::UnableToDelete { what, identifier } => {
                AppError::InternalServerError(format!("Failed to delete {} '{}'", what, identifier))
            },
            ServicesError::UnableToSerialize { what } => {
                AppError::InternalServerError(format!("Serialization failed for {}", what))
            },
            ServicesError::UnableToDeserialize { what } => {
                AppError::InternalServerError(format!("Deserialization failed for {}", what))
            },
            ServicesError::InternalServerError(msg) => {
                AppError::InternalServerError(msg)
            },
        }
    }
}

impl error::ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match self {
            AppError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::AuthenticationFailed(_) => StatusCode::UNAUTHORIZED,
        }
    }
}