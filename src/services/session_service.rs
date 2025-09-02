use crate::repository::SessionRepository;
use crate::errors::ServicesError;
use crate::errors::RepositoryError;
use crate::models::Session;
use sha2::{Sha256};
use hmac::{Hmac, Mac};
use crate::config::SessionConfig;
use crate::repository::SessionRepositoryTrait;
use crate::services::SessionServiceTrait;
type HmacSha256 = Hmac<Sha256>;

#[derive(Clone)]
pub struct SessionService<T: SessionRepositoryTrait> {
    session_repo: T,
    config: SessionConfig,
}

impl<T: SessionRepositoryTrait> SessionService<T> {
    pub fn new(session_repo: T, config: SessionConfig) -> Self {
        Self { session_repo, config }
    }
}

impl<T: SessionRepositoryTrait> SessionServiceTrait for SessionService<T> {
    /// Hash a session ID
    /// 
    /// # Arguments
    /// 
    /// * `session_id` - The original session ID to hash
    /// 
    /// # Returns
    /// 
    /// * `String` - The hashed session ID
    /// 
    fn hash_session_id(&self, session_id: &str) -> String {
        let mut mac = HmacSha256::new_from_slice(self.config.secret_key.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(session_id.as_bytes());
        let result : String = format!("{:x}", mac.finalize().into_bytes());
        result
    }
    /// Create a new session for a user
    /// 
    /// # Arguments
    /// 
    /// * `name` - The name of the user to create a session for
    /// * `session_id` - The id of the session to create
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - If the session has been created successfully
    /// * `Err(ServicesError::AlreadyExists{what})` - If the session already exists
    /// * `Err(ServicesError::InternalServerError(e))` - If there is an internal server error
    /// 
    /// # Examples
    /// 
    /// ```
    /// let session_service = SessionService::new(SessionRepository::new(db_pool));
    /// match session_service.create("john_doe", "123").await {
    ///     Ok(()) => println!("Session created successfully"),
    ///     Err(e) => eprintln!("Error creating session: {}", e),
    /// }
    /// ```
    async fn create(&self, name: &str, session_id: &str) -> Result<(), ServicesError> {
        let result = self.session_repo.create_session(name, &self.hash_session_id(session_id)).await;
        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                match e {
                    RepositoryError::AlreadyExists{what} => Err(ServicesError::AlreadyExists{what}),
                    _ => Err(ServicesError::InternalServerError(e.to_string())),
                }
            }
        }
    }

    /// Get a session by its id
    /// 
    /// # Arguments
    /// 
    /// * `session_id` - The id of the session to get
    /// 
    /// # Returns
    /// 
    /// * `Ok(Session)` - The session if found
    /// * `Err(ServicesError::NotFound{what})` - if the session is not found
    /// * `Err(ServicesError::InternalServerError(e))` - if there is an internal server error
    /// 
    /// # Example
    /// 
    /// ```
    /// let session = session_service.get_by_id("123").await;
    /// match session {
    ///     Ok(session) => println!("Session found: {:?}", session),
    ///     Err(e) => println!("Error: {:?}", e),
    /// }
    /// ```
    async fn get_by_id(&self, session_id: &str) -> Result<Session, ServicesError> {
        let result = self.session_repo.get_session_by_id(&self.hash_session_id(session_id)).await;
        match result {
            Ok(session) => Ok(session),
            Err(e) => {
                match e {
                    RepositoryError::NotFound{what} => Err(ServicesError::NotFound{what}),
                    _ => Err(ServicesError::InternalServerError(e.to_string())),
                }
            },
        }
    }

    /// Delete a session by its id
    /// 
    /// # Arguments
    /// 
    /// * `session_id` - The id of the session to delete
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - If the session has been deleted successfully
    /// * `Err(ServicesError::UnableToDelete{what})` - If the session could not be deleted
    /// * `Err(ServicesError::InternalServerError(e))` - If there is an internal server error
    /// 
    /// # Examples
    /// 
    /// ```
    /// let session_service = SessionService::new(SessionRepository::new(db_pool));
    /// match session_service.delete("123").await {
    ///     Ok(()) => println!("Session deleted successfully"),
    ///     Err(e) => eprintln!("Error deleting session: {}", e),
    /// }
    /// ```
    async fn delete(&self, session_id: &str) -> Result<(), ServicesError> {
        let result = self.session_repo.delete_session(&self.hash_session_id(session_id)).await;
        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                match e {
                    RepositoryError::NotFound{what} => Err(ServicesError::UnableToDelete{what}),
                    _ => Err(ServicesError::InternalServerError(e.to_string())),
                }
            },
        }
    }
}