use sqlx::{Pool, Postgres};
use crate::errors::RepositoryError;
use crate::models::Session;
use crate::models::User;

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
    /// * `session_id` - The id of the session to create
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
    /// match repo.create_session("john_doe", "1234567890").await {
    ///     Ok(()) => println!("Session created successfully"),
    ///     Err(e) => eprintln!("Error creating session: {}", e),
    /// }
    /// ```
    pub async fn create_session(&self, name: &str , session_id: &str) -> Result<(), RepositoryError> {
        let result = sqlx::query("INSERT INTO sessions (name, session_id) VALUES ($1, $2)")
            .bind(name)
            .bind(session_id)
            .execute(&self.db)
            .await;

        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                match e {
                    sqlx::Error::Database(e) => {
                        if e.is_unique_violation() {
                            Err(RepositoryError::AlreadyExists{what: "Session".into(), identifier: name.into()})
                        } else {
                            Err(RepositoryError::InternalServerError(e.to_string()))
                        }
                    }
                    _ => Err(RepositoryError::InternalServerError(e.to_string())),
                }
            },
        }
    
    }

    /// Get a session by name
    /// 
    /// # Arguments
    /// 
    /// * `name` - The name of the user to get the session for
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
    /// match repo.get_session_by_name("john_doe").await {
    ///     Ok(session) => println!("Session found: {:?}", session),
    ///     Err(e) => eprintln!("Error getting session: {}", e),
    /// }
    /// ```
    pub async fn get_session_by_name(&self, name: &str) -> Result<Session, RepositoryError> {
        let session = sqlx::query_as::<_, Session>("SELECT * FROM sessions WHERE name = $1;")
            .bind(name)
            .fetch_optional(&self.db)
            .await;

        match session {
            Ok(Some(session)) => Ok(session),
            Ok(None) => Err(RepositoryError::NotFound{what: "Session".into(), identifier: name.into()}),
            Err(e) => Err(RepositoryError::InternalServerError(e.to_string())),
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
    /// * `Ok(session)` - If the session has been found
    /// * `Err(RepositoryError::NotFound)` - If the session does not exist
    /// * `Err(RepositoryError::InternalServerError)` - If there is a database error
    /// 
    /// # Examples
    /// 
    /// ```
    /// let repo = SessionRepository::new(db_pool);
    /// match repo.get_session_by_id("1234567890").await {
    ///     Ok(session) => println!("Session found: {:?}", session),
    ///     Err(e) => eprintln!("Error getting session: {}", e),
    /// }
    /// ```
    pub async fn get_session_by_id(&self, session_id: &str) -> Result<Session, RepositoryError> {
        let session = sqlx::query_as::<_, Session>("SELECT * FROM sessions WHERE session_id = $1")
            .bind(session_id)
            .fetch_optional(&self.db)
            .await;
    
        match session {
            Ok(Some(session)) => Ok(session),
            Ok(None) => Err(RepositoryError::NotFound{what: "Session".into(), identifier: session_id.into()}),
            Err(e) => Err(RepositoryError::InternalServerError(e.to_string())),
        }
    }

    /// Get a user by its session
    /// 
    /// # Arguments
    /// 
    /// * `session` - The id of the session to get the user for
    /// 
    /// # Returns
    /// 
    /// * `Ok(user)` - If the user has been found
    /// * `Err(RepositoryError::NotFound)` - If the user does not exist
    /// * `Err(RepositoryError::InternalServerError)` - If there is a database error
    /// 
    /// # Examples
    /// 
    /// ```
    /// let repo = SessionRepository::new(db_pool);
    /// match repo.get_user_by_session("1234567890").await {
    ///     Ok(user) => println!("User found: {:?}", user),
    ///     Err(e) => eprintln!("Error getting user: {}", e),
    /// }
    /// ```
    pub async fn get_user_by_session(&self, session: &str) -> Result<User, RepositoryError> {
    
        let user = sqlx::query_as::<_, User>("SELECT u.* FROM Sessions s NATURAL JOIN users u WHERE s.session_id = $1;")
            .bind(session)
            .fetch_optional(&self.db)
            .await;
    
        match user {
            Ok(Some(user)) => Ok(user),
            Ok(None) => Err(RepositoryError::NotFound{what: "User".into(), identifier: session.into()}),
            Err(e) => Err(RepositoryError::InternalServerError(e.to_string())),
        }
    }

    /// Delete a session
    /// 
    /// # Arguments
    /// 
    /// * `session_id` - The id of the session to delete
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
    pub async fn delete_session(&self, session_id: &str) -> Result<(), RepositoryError> {
        let result = sqlx::query("DELETE FROM sessions WHERE session_id = $1")
            .bind(session_id)
            .execute(&self.db)
            .await;
    
        match result {
            Ok(result) => {
                if result.rows_affected() == 0 {
                    Err(RepositoryError::NotFound{what: "Session".into(), identifier: session_id.into()})
                } else {
                    Ok(())
                }
            },
            Err(e) => Err(RepositoryError::InternalServerError(e.to_string())),
        }
    }

}