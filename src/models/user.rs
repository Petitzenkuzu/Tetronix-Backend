use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use crate::errors::ServicesError;

#[derive(Deserialize, Serialize, Debug, Clone, FromRow, PartialEq)]
pub struct User {
    pub name: String,
    pub number_of_games: i32,
    pub best_score: i32,
    pub highest_level: i32,
}

impl User {
    /// Validate the user data
    /// 
    /// # Arguments
    /// 
    /// * `self` - The user to validate
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - If the user data is valid
    /// * `Err(ServicesError::InvalidInput{field, message})` - If the user data is invalid
    /// 
    /// # Examples
    /// 
    /// ```
    /// let user = User { name: "john_doe".to_string(), number_of_games: 10, best_score: 100, highest_level: 10 };
    /// match user.validate() {
    ///     Ok(()) => println!("User data is valid"),
    ///     Err(e) => eprintln!("Error validating user data: {}", e),
    /// }
    /// ```
    pub fn validate(&self) -> Result<(), ServicesError> {

        if self.name.len() < 2 {
            return Err(ServicesError::InvalidInput{
                field: "name".to_string(), 
                message: "Name must be at least 2 characters long".to_string()
            });
        }

        if self.name.len() > 50 {
            return Err(ServicesError::InvalidInput{
                field: "name".to_string(), 
                message: "Name cannot exceed 50 characters".to_string()
            });
        }

        if self.number_of_games < 0 {
            return Err(ServicesError::InvalidInput{
                field: "number_of_games".to_string(), 
                message: "Number of games cannot be negative".to_string()
            });
        }

        if self.best_score < 0 {
            return Err(ServicesError::InvalidInput{
                field: "best_score".to_string(), 
                message: "Best score cannot be negative".to_string()
            });
        }

        if self.highest_level < 0 {
            return Err(ServicesError::InvalidInput{
                field: "highest_level".to_string(), 
                message: "Highest level cannot be negative".to_string()
            });
        }
        
        Ok(())
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, FromRow)]
pub struct Session {
    pub name: String,
    pub session_id: String,
}