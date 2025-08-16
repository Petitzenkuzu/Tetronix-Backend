use sqlx::{Pool, Postgres};
use crate::models::User;
use crate::errors::RepositoryError;
#[derive(Clone)]
pub struct UserRepository {
    pub db: Pool<Postgres>,
}

impl UserRepository {
    pub fn new(db: Pool<Postgres>) -> Self {
        Self { db }
    }

    /// Create a new user
    /// 
    /// # Arguments
    /// 
    /// * `name` - The name of the user to create
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - If the user has been created successfully
    /// * `Err(RepositoryError::AlreadyExists)` - If a user with this name already exists
    /// * `Err(RepositoryError::InternalServerError)` - If there is a database error
    /// 
    /// # Examples
    /// 
    /// ```
    /// let repo = UserRepository::new(db_pool);
    /// match repo.create_user("john_doe").await {
    ///     Ok(()) => println!("User created successfully"),
    ///     Err(e) => eprintln!("Error creating user: {}", e),
    /// }
    /// ```
    pub async fn create_user(&self, name: &str) -> Result<(), RepositoryError> {
        let result = sqlx::query(
            "INSERT INTO users (name) VALUES ($1)",
        )
        .bind(name)
        .execute(&self.db)
        .await;
        
        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                match e {
                    sqlx::Error::Database(e) => {
                        if e.is_unique_violation() {
                            Err(RepositoryError::AlreadyExists{what: "User".into()})
                        } 
                        else if e.is_check_violation() {
                            Err(RepositoryError::InvalidInput{what: "User name".into()})
                        }
                        else {
                            Err(RepositoryError::InternalServerError(e.to_string()))
                        }
                    }
                    _ => Err(RepositoryError::InternalServerError(e.to_string())),
                }
            },
        }
    }

    /// Update a user
    /// 
    /// # Arguments
    /// 
    /// * `user` - The user to update
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - If the user has been updated successfully
    /// * `Err(RepositoryError::NotFound)` - If the user does not exist
    /// * `Err(RepositoryError::InternalServerError)` - If there is a database error
    /// 
    /// # Examples
    /// 
    /// ```
    /// let repo = UserRepository::new(db_pool);
    /// match repo.update_user(&user).await {
    ///     Ok(()) => println!("User updated successfully"),
    ///     Err(e) => eprintln!("Error updating user: {}", e),
    /// }
    /// ```
    pub async fn update_user(&self, user: &User) -> Result<(), RepositoryError> {
        let result = sqlx::query("UPDATE users SET best_score = $1, highest_level = $2, number_of_games = $3 WHERE name = $4")
            .bind(user.best_score)
            .bind(user.highest_level)
            .bind(user.number_of_games)
            .bind(&user.name)
            .execute(&self.db)
            .await;
    
        match result {
            Ok(result) => {
                if result.rows_affected() == 0 {
                    Err(RepositoryError::NotFound{what: "User".into()})
                } else {
                    Ok(())
                }
            },
            Err(e) => Err(RepositoryError::InternalServerError(e.to_string())),
        }
    }

    /// Get a user by name
    /// 
    /// # Arguments
    /// 
    /// * `name` - The name of the user to get
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
    /// let repo = UserRepository::new(db_pool);
    /// match repo.get_user_by_name("john_doe").await {
    ///     Ok(user) => println!("User found: {}", user.name),
    ///     Err(e) => eprintln!("Error getting user: {}", e),
    /// }
    /// ```
    pub async fn get_user_by_name(&self, name: &str) -> Result<User, RepositoryError> {
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE name = $1",
        )
        .bind(name)
        .fetch_optional(&self.db)
        .await;
        match user {
            Ok(Some(user)) => Ok(user),
            Ok(None) => Err(RepositoryError::NotFound{what: "User".into()}),
            Err(e) => Err(RepositoryError::InternalServerError(e.to_string())),
        }
    }

    /// Get the x top users
    /// 
    /// # Arguments
    /// 
    /// * `limit` - The number of users to get
    /// 
    /// # Returns
    /// 
    /// * `Ok(users)` - If the users have been found
    /// * `Err(RepositoryError::InvalidLimit)` - If the limit is not between 0 and 100
    /// * `Err(RepositoryError::InternalServerError)` - If there is a database error
    /// 
    /// # Examples
    /// 
    /// ```
    /// let repo = UserRepository::new(db_pool);
    /// match repo.get_top_users(10).await {
    ///     Ok(users) => println!("Users found: {:?}", users),
    ///     Err(e) => eprintln!("Error getting users: {}", e),
    /// }
    /// ```
    pub async fn get_top_users(&self, limit: i32) -> Result<Vec<User>, RepositoryError> {
        if limit <= 0 || limit > 100 {
            return Err(RepositoryError::InvalidLimit{low: 0, high: 100});
        }

        let leaderboard = sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY best_score DESC LIMIT $1")
            .bind(limit)
            .fetch_all(&self.db)
            .await;
        match leaderboard {
            Ok(leaderboard) => Ok(leaderboard),
            Err(e) => Err(RepositoryError::InternalServerError(e.to_string())),
        }
    }

    /// Delete a user
    /// 
    /// # Arguments
    /// 
    /// * `name` - The name of the user to delete
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - If the user has been deleted successfully
    /// * `Err(RepositoryError::NotFound)` - If the user does not exist
    /// * `Err(RepositoryError::InternalServerError)` - If there is a database error
    /// 
    /// # Examples
    /// 
    /// ```
    /// let repo = UserRepository::new(db_pool);
    /// match repo.delete_user("john_doe").await {
    ///     Ok(()) => println!("User deleted successfully"),
    ///     Err(e) => eprintln!("Error deleting user: {}", e),
    /// }
    /// ```
    pub async fn delete_user(&self, name: &str) -> Result<(), RepositoryError> {
        let result = sqlx::query("DELETE FROM users WHERE name = $1")
            .bind(name)
            .execute(&self.db)
            .await;
        match result {
            Ok(result) => {
                if result.rows_affected() == 0 {
                    Err(RepositoryError::NotFound{what: "User".into()})
                } else {
                    Ok(())
                }
            },
            Err(e) => Err(RepositoryError::InternalServerError(e.to_string())),
        }
    }
}