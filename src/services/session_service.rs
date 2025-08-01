use crate::repository::SessionRepository;
use crate::errors::ServicesError;
use crate::errors::RepositoryError;
use crate::models::Session;
use crate::models::User;

pub struct SessionService {
    session_repo: SessionRepository,
}

impl SessionService {
    pub fn new(session_repo: SessionRepository) -> Self {
        Self { session_repo }
    }

    pub async fn create(&self, name: &str, session_id: &str) -> Result<(), ServicesError> {
        let result = self.session_repo.create_session(name, session_id).await;
        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                match e {
                    RepositoryError::AlreadyExists{what, identifier} => Err(ServicesError::AlreadyExists{what, identifier}),
                    _ => Err(ServicesError::InternalServerError(e.to_string())),
                }
            }
        }
    }

    pub async fn get_by_name(&self, name: &str) -> Result<Session, ServicesError> {
        let result = self.session_repo.get_session_by_name(name).await;
        match result {
            Ok(session) => Ok(session),
            Err(e) => {
                match e {
                    RepositoryError::NotFound{what, identifier} => Err(ServicesError::NotFound{what, identifier}),
                    _ => Err(ServicesError::InternalServerError(e.to_string())),
                }
            },
        }
    }

    pub async fn get_by_id(&self, session_id: &str) -> Result<Session, ServicesError> {
        let result = self.session_repo.get_session_by_id(session_id).await;
        match result {
            Ok(session) => Ok(session),
            Err(e) => {
                match e {
                    RepositoryError::NotFound{what, identifier} => Err(ServicesError::NotFound{what, identifier}),
                    _ => Err(ServicesError::InternalServerError(e.to_string())),
                }
            },
        }
    }

    pub async fn get_user_by_id(&self, session_id: &str) -> Result<User, ServicesError> {
        let result = self.session_repo.get_user_by_session(session_id).await;
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

    pub async fn delete(&self, session_id: &str) -> Result<(), ServicesError> {
        let result = self.session_repo.delete_session(session_id).await;
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