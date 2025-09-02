use crate::models::Game;
use crate::services::{UserService, SessionService, GameService, AuthService};
use crate::repository::{UserRepository, SessionRepository, GameRepository};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::sync::OnceLock;
use dotenv::dotenv;
use std::env;
use uuid::Uuid;
use crate::builder::game_builder::GameBuilder;
use crate::config::{SessionConfig, AuthConfig};
use crate::services::{UserServiceTrait, SessionServiceTrait, GameServiceTrait, AuthServiceTrait};
static POOL: OnceLock<PgPool> = OnceLock::new();

async fn get_pool() -> &'static PgPool {
    if let Some(pool) = POOL.get() {
        pool
    } else {
        dotenv().ok();
        let database_url = env::var("TEST_DATABASE_URL")
            .expect("TEST_DATABASE_URL must be set for tests");
        let pool = PgPoolOptions::new()
            .max_connections(20)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database");
        POOL.get_or_init(|| pool)
    }
}



pub struct ServiceTestFixture
{   
    pub pool: &'static PgPool,
    pub user_service: UserService<UserRepository>,
    pub session_service: SessionService<SessionRepository>,
    pub game_service: GameService<GameRepository>,
    pub auth_service: AuthService<UserRepository, SessionRepository>,
}

impl ServiceTestFixture 
{
    pub async fn new() -> Self {
        let pool = get_pool().await;
        let user_repo = UserRepository::new(pool.clone());
        let session_repo = SessionRepository::new(pool.clone());
        let game_repo = GameRepository::new(pool.clone());
        Self {
            pool,
            user_service: UserService::new(user_repo.clone()),
            session_service: SessionService::new(session_repo.clone(), SessionConfig::from_env()),
            game_service: GameService::new(game_repo),
            auth_service: AuthService::new(user_repo, session_repo, AuthConfig::from_env()),
        }
    }

    pub fn random_user_name(&self) -> String {
        format!("test_user_{}", Uuid::new_v4())
    }
    
    pub fn random_session_hash(&self) -> String {
        format!("test_session_{}", Uuid::new_v4())
    }

    pub async fn with_test_user<F, Fut, R> (&self, test_fn : F) -> R
    where 
        F: FnOnce(String, UserService<UserRepository>) -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        let username = self.random_user_name();
        let _ =self.user_service.create(&username).await.expect("Failed to create test user");

        let result = test_fn(username.clone(), self.user_service.clone()).await;
        
        let _ = self.user_service.delete(&username).await;
        result
    }

    pub async fn with_test_user_and_session<F, Fut, R> (&self, test_fn : F) -> R
    where 
        F: FnOnce(String, String, UserService<UserRepository>, SessionService<SessionRepository>) -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        let username = self.random_user_name();
        let _ =self.user_service.create(&username).await.expect("Failed to create test user");

        let session_hash = self.random_session_hash();
        let _ = self.session_service.create(&username, &session_hash).await.expect("Failed to create test session");

        let result = test_fn(username.clone(), session_hash.clone(), self.user_service.clone(), self.session_service.clone()).await;
        
        let _ = self.session_service.delete(&session_hash).await;
        let _ = self.user_service.delete(&username).await;
        result
    }

    pub async fn with_test_user_and_game<F, Fut, R> (&self, test_fn : F) -> R
    where 
        F: FnOnce(String, Game, UserService<UserRepository>, GameService<GameRepository>) -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        let username = self.random_user_name();
        let _ =self.user_service.create(&username).await.expect("Failed to create test user");

        let game = GameBuilder::new(&username).build();
        let _ = self.game_service.upsert(&game).await.expect("Failed to create test game");

        let result = test_fn(username.clone(), game, self.user_service.clone(), self.game_service.clone()).await;
        
        let _ = self.user_service.delete(&username).await;
        result
    }


}

#[macro_export]
macro_rules! assert_service_not_found {
    ($result:expr) => {
        match $result {
            Err(crate::errors::ServicesError::NotFound { .. }) => {},
            Ok(_) => panic!("Expected ServicesError::NotFound, got success at {}", std::panic::Location::caller()),
            Err(e) => panic!("Expected ServicesError::NotFound, got {:?} at {}", e, std::panic::Location::caller()),
        }
    };
}

#[macro_export]
macro_rules! assert_service_already_exists {
    ($result:expr) => {
        match $result {
            Err(crate::errors::ServicesError::AlreadyExists { .. }) => {},
            Ok(_) => panic!("Expected ServicesError::AlreadyExists, got success at {}", std::panic::Location::caller()),
            Err(e) => panic!("Expected ServicesError::AlreadyExists, got {:?} at {}", e, std::panic::Location::caller()),
        }
    };
}

#[macro_export]
macro_rules! assert_service_invalid_input {
    ($result:expr) => {
        match $result {
            Err(crate::errors::ServicesError::InvalidInput { .. }) => {},
            Ok(_) => panic!("Expected ServicesError::InvalidInput, got success at {}", std::panic::Location::caller()),
            Err(e) => panic!("Expected ServicesError::InvalidInput, got {:?} at {}", e, std::panic::Location::caller()),
        }
    };
}

#[macro_export]
macro_rules! assert_service_unable_to_delete {
    ($result:expr) => {
        match $result {
            Err(crate::errors::ServicesError::UnableToDelete { .. }) => {},
            Ok(_) => panic!("Expected ServicesError::UnableToDelete, got success at {}", std::panic::Location::caller()),
            Err(e) => panic!("Expected ServicesError::UnableToDelete, got {:?} at {}", e, std::panic::Location::caller()),
        }
    };
}