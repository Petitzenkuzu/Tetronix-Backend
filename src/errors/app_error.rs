use actix_web::{ error, http::{header::ContentType, StatusCode}, HttpResponse };
use thiserror::Error;
use crate::errors::ServicesError;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("internal server error: {0}")]
    InternalServerError(String),
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("Authentication required")]
    Unauthorized,
    #[error("not found: {0}")]
    NotFound(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("authentication failed: {0}")]
    AuthenticationFailed(String),
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
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
            ServicesError::NotFound { what } => {
                AppError::NotFound(format!("{} not found", what))
            },
            ServicesError::AlreadyExists { what } => {
                AppError::Conflict(format!("{} already exists", what))
            },
            ServicesError::UnableToDelete { what: _ } => {
                AppError::InternalServerError("Something went wrong".to_string())
            },
            ServicesError::UnableToSerialize { what: _ } => {
                AppError::InternalServerError("Data processing error".to_string())
            },
            ServicesError::UnableToDeserialize { what: _ } => {
                AppError::InternalServerError("Data processing error".to_string())
            },
            ServicesError::InternalServerError(_) => {
                AppError::InternalServerError("Something went wrong".to_string())
            },
        }
    }
}

impl error::ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::RateLimitExceeded => {
                HttpResponse::TooManyRequests()
                .insert_header(("X-RateLimit-Limit", "100"))
                .insert_header(("X-RateLimit-Remaining", "0"))
                .insert_header(("Retry-After", "10"))
                .json(serde_json::json!({
                    "error": "Rate limit exceeded"
                }))
            },
            _ => {
                HttpResponse::build(self.status_code())
                .insert_header(ContentType::json())
                .body(self.to_string())   
            }
        }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            AppError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::AuthenticationFailed(_) => StatusCode::UNAUTHORIZED,
            AppError::RateLimitExceeded => StatusCode::TOO_MANY_REQUESTS,
        }
    }
}