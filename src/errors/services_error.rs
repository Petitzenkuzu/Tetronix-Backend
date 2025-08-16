use thiserror::Error;
use crate::errors::RepositoryError;

#[derive(Error, Debug)]
pub enum ServicesError {
    #[error("internal server error: {0}")]
    InternalServerError(String),
    #[error("Invalid Input on field : {field} , {message}")]
    InvalidInput{field: String, message: String},
    #[error("{what} not found")]
    NotFound{what: String},
    #[error("{what} already exists")]
    AlreadyExists{what: String},
    #[error("unable to delete {what}")]
    UnableToDelete{what: String},
    #[error("unable to serialize {what}")]
    UnableToSerialize{what: String},
    #[error("unable to deserialize {what}")]
    UnableToDeserialize{what: String},
    #[error("Authentication Failed : {reason}")]
    AuthenticationFailed{reason: String},
}

impl From<RepositoryError> for ServicesError {
    fn from(err: RepositoryError) -> Self {
        match err {
            RepositoryError::NotFound { what } => {
                ServicesError::NotFound { what }
            }
            RepositoryError::AlreadyExists { what } => {
                ServicesError::AlreadyExists { what }
            }
            RepositoryError::InternalServerError(msg) => {
                ServicesError::InternalServerError(msg)
            }
            RepositoryError::InvalidInput { what } => {
                ServicesError::InvalidInput { field: what, message: "is invalid".to_string() }
            }
            RepositoryError::InvalidLimit { low, high } => {
                ServicesError::InvalidInput { field: "limit".to_string(), message: format!("must be greater or equal to {} and less or equal to {}", low, high) }
            }
            RepositoryError::SerializationError(msg) => {
                ServicesError::UnableToSerialize { what: msg }
            }
            RepositoryError::DeserializationError(msg) => {
                ServicesError::UnableToDeserialize { what: msg }
            }
        }
    }
}