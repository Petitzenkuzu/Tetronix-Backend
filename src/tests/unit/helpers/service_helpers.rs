use crate::builder::game_builder::GameBuilder;
use crate::config::AuthConfig;
use crate::models::Game;
use crate::repository::{GameRepository, UserRepository};
use crate::services::{AuthService, GameService, UserService};
use crate::services::{GameServiceTrait, UserServiceTrait};
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::env;
use std::sync::OnceLock;
use uuid::Uuid;
static POOL: OnceLock<PgPool> = OnceLock::new();

async fn get_pool() -> &'static PgPool {
    if let Some(pool) = POOL.get() {
        pool
    } else {
        dotenv().ok();
        let database_url =
            env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set for tests");
        let pool = PgPoolOptions::new()
            .max_connections(20)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database");
        POOL.get_or_init(|| pool)
    }
}

pub struct ServiceTestFixture {
    pub user_service: UserService<UserRepository>,
    pub game_service: GameService<GameRepository>,
    pub auth_service: AuthService<UserRepository>,
}

impl ServiceTestFixture {
    pub async fn new(github_test_url: Option<String>) -> Self {
        let pool = get_pool().await;
        let user_repo = UserRepository::new(pool.clone());
        let game_repo = GameRepository::new(pool.clone());
        let auth_config = match github_test_url {
            Some(url) => AuthConfig::with_github_url(url),
            None => AuthConfig::from_env(),
        };
        Self {
            user_service: UserService::new(user_repo.clone()),
            game_service: GameService::new(game_repo),
            auth_service: AuthService::new(user_repo, auth_config),
        }
    }

    pub fn random_user_name(&self) -> String {
        format!("test_user_{}", Uuid::new_v4())
    }

    pub async fn with_test_user<F, Fut, R>(&self, test_fn: F) -> R
    where
        F: FnOnce(String, UserService<UserRepository>) -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        let username = self.random_user_name();
        () = self
            .user_service
            .create(&username)
            .await
            .expect("Failed to create test user");

        let result = test_fn(username.clone(), self.user_service.clone()).await;

        let _ = self.user_service.delete(&username).await;
        result
    }

    pub async fn with_test_user_and_game<F, Fut, R>(&self, test_fn: F) -> R
    where
        F: FnOnce(String, Game, UserService<UserRepository>, GameService<GameRepository>) -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        let username = self.random_user_name();
        () = self
            .user_service
            .create(&username)
            .await
            .expect("Failed to create test user");

        let game = GameBuilder::new(&username).build();
        () = self
            .game_service
            .upsert(&game)
            .await
            .expect("Failed to create test game");

        let result = test_fn(
            username.clone(),
            game,
            self.user_service.clone(),
            self.game_service.clone(),
        )
        .await;

        let _ = self.user_service.delete(&username).await;
        result
    }
}

#[macro_export]
macro_rules! assert_service_not_found {
    ($result:expr) => {
        match $result {
            Err($crate::errors::ServicesError::NotFound { .. }) => {}
            Ok(_) => panic!(
                "Expected ServicesError::NotFound, got success at {}",
                std::panic::Location::caller()
            ),
            Err(e) => panic!(
                "Expected ServicesError::NotFound, got {:?} at {}",
                e,
                std::panic::Location::caller()
            ),
        }
    };
}

#[macro_export]
macro_rules! assert_service_already_exists {
    ($result:expr) => {
        match $result {
            Err($crate::errors::ServicesError::AlreadyExists { .. }) => {}
            Ok(_) => panic!(
                "Expected ServicesError::AlreadyExists, got success at {}",
                std::panic::Location::caller()
            ),
            Err(e) => panic!(
                "Expected ServicesError::AlreadyExists, got {:?} at {}",
                e,
                std::panic::Location::caller()
            ),
        }
    };
}

#[macro_export]
macro_rules! assert_service_invalid_input {
    ($result:expr) => {
        match $result {
            Err($crate::errors::ServicesError::InvalidInput { .. }) => {}
            Ok(_) => panic!(
                "Expected ServicesError::InvalidInput, got success at {}",
                std::panic::Location::caller()
            ),
            Err(e) => panic!(
                "Expected ServicesError::InvalidInput, got {:?} at {}",
                e,
                std::panic::Location::caller()
            ),
        }
    };
}

#[macro_export]
macro_rules! assert_service_unable_to_delete {
    ($result:expr) => {
        match $result {
            Err($crate::errors::ServicesError::UnableToDelete { .. }) => {}
            Ok(_) => panic!(
                "Expected ServicesError::UnableToDelete, got success at {}",
                std::panic::Location::caller()
            ),
            Err(e) => panic!(
                "Expected ServicesError::UnableToDelete, got {:?} at {}",
                e,
                std::panic::Location::caller()
            ),
        }
    };
}
