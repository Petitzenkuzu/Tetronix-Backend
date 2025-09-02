use crate::errors::ServicesError;
use crate::errors::RepositoryError;
use crate::models::User;
use crate::repository::UserRepositoryTrait;
use crate::services::UserServiceTrait;

#[derive(Clone)]
pub struct UserService<T: UserRepositoryTrait> {
    user_repo : T,
}

impl<T: UserRepositoryTrait> UserService<T> {
    pub fn new(user_repo: T) -> Self {
        Self { user_repo }
    }
}

impl<T: UserRepositoryTrait> UserServiceTrait for UserService<T> {
    /// Create a new user
    /// 
    /// # Arguments
    /// 
    /// * `name` - The name of the user to create
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - If the user has been created successfully
    /// * `Err(ServicesError::AlreadyExists{what})` - If the user already exists
    /// * `Err(ServicesError::InvalidInput{field, message})` - If the user name is invalid
    /// * `Err(ServicesError::InternalServerError(e))` - If there is an internal server error
    /// 
    /// # Examples
    /// 
    /// ```
    /// let user_service = UserService::new(UserRepository::new(db_pool));
    /// match user_service.create("john_doe").await {
    ///     Ok(()) => println!("User created successfully"),
    ///     Err(e) => eprintln!("Error creating user: {}", e),
    /// }
    /// ```
    async fn create(&self, name: &str) -> Result<(), ServicesError> {
        if name.len() < 2 {
            return Err(ServicesError::InvalidInput{field: "name".into(), message: "must be at least 2 characters long".to_string()});
        }
        if name.len() > 50 {
            return Err(ServicesError::InvalidInput{field: "name".into(), message: "must be less than 50 characters long".to_string()});
        }

        let result = self.user_repo.create_user(name).await;
        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                match e {
                    RepositoryError::AlreadyExists{what} => Err(ServicesError::AlreadyExists{what}),
                    _ => Err(ServicesError::InternalServerError(e.to_string())),
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
    /// * `Err(ServicesError::NotFound{what})` - If the user is not found
    /// * `Err(ServicesError::InvalidInput{field, message})` - If the user data is invalid
    /// * `Err(ServicesError::InternalServerError(e))` - If there is an internal server error
    /// 
    /// # Examples
    /// 
    /// ```
    /// let user_service = UserService::new(UserRepository::new(db_pool));
    /// let user = User { name: "john_doe".to_string(), number_of_games: 10, best_score: 100, highest_level: 10 };
    /// match user_service.update(&user).await {
    ///     Ok(()) => println!("User updated successfully"),
    ///     Err(e) => eprintln!("Error updating user: {}", e),
    /// }
    /// ```
    async fn update(&self, user: &User) -> Result<(), ServicesError> {
        user.validate()?;
        let result = self.user_repo.update_user(user).await;
        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                match e {
                    RepositoryError::NotFound{what} => Err(ServicesError::NotFound{what}),
                    _ => Err(ServicesError::InternalServerError(e.to_string())),
                }
            },
        }
    }

    /// Get a user by its name
    /// 
    /// # Arguments
    /// 
    /// * `name` - The name of the user to get
    /// 
    /// # Returns
    /// 
    /// * `Ok(User)` - If the user has been found
    /// * `Err(ServicesError::NotFound{what})` - If the user is not found
    /// * `Err(ServicesError::InternalServerError(e))` - If there is an internal server error
    /// 
    /// # Examples
    /// 
    /// ```
    /// let user_service = UserService::new(UserRepository::new(db_pool));
    /// match user_service.get_by_name("john_doe").await {
    ///     Ok(user) => println!("User found: {:?}", user),
    ///     Err(e) => eprintln!("Error getting user: {}", e),
    /// }
    /// ```
    async fn get_by_name(&self, name: &str) -> Result<User, ServicesError> {
        let result =self.user_repo.get_user_by_name(name).await;
        match result {
            Ok(user) => Ok(user),
            Err(e) => {
                match e {
                    RepositoryError::NotFound{what} => Err(ServicesError::NotFound{what}),
                    _ => Err(ServicesError::InternalServerError(e.to_string())),
                }
            },
        }
    }

    /// Get the top users
    /// 
    /// # Arguments
    /// 
    /// * `limit` - The limit of users to get
    /// 
    /// # Returns
    /// 
    /// * `Ok(Vec<User>)` - If the users have been found
    /// * `Err(ServicesError::InvalidInput{field, message})` - If the limit is invalid
    /// * `Err(ServicesError::InternalServerError(e))` - If there is an internal server error
    /// 
    /// # Examples
    /// 
    /// ```
    /// let user_service = UserService::new(UserRepository::new(db_pool));
    /// match user_service.get_top(10).await {
    ///     Ok(users) => println!("Users found: {:?}", users),
    ///     Err(e) => eprintln!("Error getting users: {}", e),
    /// }
    /// ```
    async fn get_top(&self, limit: i32) -> Result<Vec<User>, ServicesError> {
        let result = self.user_repo.get_top_users(limit).await;
        
        match result {
            Ok(users) => Ok(users),
            Err(e) => {
                match e {
                    RepositoryError::InvalidLimit{low, high} => Err(ServicesError::InvalidInput{field: "limit".into(), message: format!("Limit must be greater or equal to {} and less or equal to {}", low, high)}),
                    _ => Err(ServicesError::InternalServerError(e.to_string())),
                }
            },
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
    /// * `Err(ServicesError::UnableToDelete{what})` - If the user could not be deleted
    /// * `Err(ServicesError::InternalServerError(e))` - If there is an internal server error
    /// 
    /// # Examples
    /// 
    /// ```
    /// let user_service = UserService::new(UserRepository::new(db_pool));
    /// match user_service.delete("john_doe").await {
    ///     Ok(()) => println!("User deleted successfully"),
    ///     Err(e) => eprintln!("Error deleting user: {}", e),
    /// }
    /// ```
    async fn delete(&self, name: &str) -> Result<(), ServicesError> {
        let result = self.user_repo.delete_user(name).await;
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