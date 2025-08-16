use sqlx::{Pool, Postgres};
use crate::errors::RepositoryError;
use crate::models::Session;

#[derive(Clone)]
pub struct SessionRepository {
    pub db: Pool<Postgres>,
}

impl SessionRepository {
    pub fn new(db: Pool<Postgres>) -> Self {
        Self { db }
    }

    /// Create a new session
    /// 
    /// # Arguments
    /// 
    /// * `name` - The name of the user to create the session for
    /// * `session_hash` - The hash of the session to create
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - If the session has been created successfully
    /// * `Err(RepositoryError::AlreadyExists)` - If a session with this id already exists
    /// * `Err(RepositoryError::InternalServerError)` - If there is a database error
    /// 
    /// # Examples
    /// 
    /// ```
    /// let repo = SessionRepository::new(db_pool);
    /// match repo.create_session("john_doe", "12345678901234567890123456789012").await {
    ///     Ok(()) => println!("Session created successfully"),
    ///     Err(e) => eprintln!("Error creating session: {}", e),
    /// }
    /// ```
    pub async fn create_session(&self, name: &str , session_hash: &str) -> Result<(), RepositoryError> {
        let result = sqlx::query("INSERT INTO sessions (name, session_id) VALUES ($1, $2)")
            .bind(name)
            .bind(session_hash)
            .execute(&self.db)
            .await;

        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                match e {
                    sqlx::Error::Database(e) => {
                        if e.is_unique_violation() {
                            Err(RepositoryError::AlreadyExists{what: "Session".into()})
                        } else {
                            Err(RepositoryError::InternalServerError(e.to_string()))
                        }
                    }
                    _ => Err(RepositoryError::InternalServerError(e.to_string())),
                }
            },
        }
    
    }


    /// Get a session by its id
    /// 
    /// # Arguments
    /// 
    /// * `session_hash` - The hash of the session to get
    /// 
    /// # Returns
    /// 
    /// * `Ok(session)` - If the session has been found
    /// * `Err(RepositoryError::NotFound)` - If the session does not exist
    /// * `Err(RepositoryError::InternalServerError)` - If there is a database error
    /// 
    /// # Examples
    /// 
    /// ```
    /// let repo = SessionRepository::new(db_pool);
    /// match repo.get_session_by_id("12345678901234567890123456789012").await {
    ///     Ok(session) => println!("Session found: {:?}", session),
    ///     Err(e) => eprintln!("Error getting session: {}", e),
    /// }
    /// ```
    pub async fn get_session_by_id(&self, session_hash: &str) -> Result<Session, RepositoryError> {
        let session = sqlx::query_as::<_, Session>("SELECT * FROM sessions WHERE session_id = $1")
            .bind(session_hash)
            .fetch_optional(&self.db)
            .await;
    
        match session {
            Ok(Some(session)) => Ok(session),
            Ok(None) => Err(RepositoryError::NotFound{what: "Session".into()}),
            Err(e) => Err(RepositoryError::InternalServerError(e.to_string())),
        }
    }

    /// Delete a session
    /// 
    /// # Arguments
    /// 
    /// * `session_hash` - The hash of the session to delete
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - If the session has been deleted successfully
    /// * `Err(RepositoryError::NotFound)` - If the session does not exist
    /// * `Err(RepositoryError::InternalServerError)` - If there is a database error
    /// 
    /// # Examples
    /// 
    /// ```
    /// let repo = SessionRepository::new(db_pool);
    /// match repo.delete_session("1234567890").await {
    ///     Ok(()) => println!("Session deleted successfully"),
    ///     Err(e) => eprintln!("Error deleting session: {}", e),
    /// }
    /// ```
    pub async fn delete_session(&self, session_hash: &str) -> Result<(), RepositoryError> {
        let result = sqlx::query("DELETE FROM sessions WHERE session_id = $1")
            .bind(session_hash)
            .execute(&self.db)
            .await;

        match result {
            Ok(result) => {
                if result.rows_affected() == 0 {
                    Err(RepositoryError::NotFound{what: "Session".into()})
                } else {
                    Ok(())
                }
            },
            Err(e) => Err(RepositoryError::InternalServerError(e.to_string())),
        }
    }

    /// Delete session linked to a user
    /// 
    /// # Arguments
    /// 
    /// * `name` - The name of the user to delete the session for
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - If the sessions linked to the user have been deleted successfully
    /// * `Err(RepositoryError::NotFound)` - If no session is linked to the user
    /// * `Err(RepositoryError::InternalServerError)` - If there is a database error
    /// 
    /// # Examples
    /// 
    /// ```
    /// let repo = SessionRepository::new(db_pool);
    /// match repo.delete_session_by_name("john_doe").await {
    ///     Ok(()) => println!("Sessions deleted successfully"),
    ///     Err(e) => eprintln!("Error deleting sessions: {}", e),
    /// }
    /// ```
    pub async fn delete_session_by_name(&self, name: &str) -> Result<(), RepositoryError> {
        let result = sqlx::query("DELETE FROM sessions WHERE name = $1")
            .bind(name)
            .execute(&self.db)
            .await;
    
        match result {
            Ok(result) => {
                if result.rows_affected() == 0 {
                    Err(RepositoryError::NotFound{what: "Session".into()})
                } else {
                    Ok(())
                }
            },
            Err(e) => Err(RepositoryError::InternalServerError(e.to_string())),
        }
    }

}