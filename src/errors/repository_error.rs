
#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("{what} not found")]
    NotFound{what: String},
    #[error("{what} already exists")]
    AlreadyExists{what: String},
    #[error("internal server error: {0}")]
    InternalServerError(String),
    #[error("{what} is invalid")]
    InvalidInput{what: String},
    #[error("limit must be greater or equal to {low} and less or equal to {high}")]
    InvalidLimit{low: i32, high: i32},
    #[error("serialization error: {0}")]
    SerializationError(String),
    #[error("deserialization error: {0}")]
    DeserializationError(String),
}