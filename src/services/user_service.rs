use crate::repository::UserRepository;
use crate::errors::ServicesError;
use crate::errors::RepositoryError;
use crate::models::User;

pub struct UserService {
    user_repo : UserRepository,
}

impl UserService {
    pub fn new(user_repo: UserRepository) -> Self {
        Self { user_repo }
    }

    pub async fn create(&self, name: &str) -> Result<(), ServicesError> {
        let result = self.user_repo.create_user(name).await;
        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                match e {
                    RepositoryError::AlreadyExists{what, identifier} => Err(ServicesError::AlreadyExists{what, identifier}),
                    _ => Err(ServicesError::InternalServerError(e.to_string())),
                }
            },
        }
    }

    pub async fn update(&self, user: &User) -> Result<(), ServicesError> {
        let result = self.user_repo.update_user(user).await;
        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                match e {
                    RepositoryError::NotFound{what, identifier} => Err(ServicesError::NotFound{what, identifier}),
                    _ => Err(ServicesError::InternalServerError(e.to_string())),
                }
            },
        }
    }

    pub async fn get_by_name(&self, name: &str) -> Result<User, ServicesError> {
        let result =self.user_repo.get_user_by_name(name).await;
        match result {
            Ok(user) => Ok(user),
            Err(e) => {
                match e {
                    RepositoryError::NotFound{what, identifier} => Err(ServicesError::NotFound{what, identifier}),
                    _ => Err(ServicesError::InternalServerError(e.to_string())),
                }
            },
        }
    }

    pub async fn get_top(&self, limit: i32) -> Result<Vec<User>, ServicesError> {
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

    pub async fn delete(&self, name: &str) -> Result<(), ServicesError> {
        let result = self.user_repo.delete_user(name).await;
        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                match e {
                    RepositoryError::NotFound{what, identifier} => Err(ServicesError::UnableToDelete{what, identifier}),
                    _ => Err(ServicesError::InternalServerError(e.to_string())),
                }
            },
        }
    }
}