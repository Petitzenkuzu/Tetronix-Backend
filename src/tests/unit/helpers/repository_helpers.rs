use crate::builder::game_builder::GameBuilder;
use crate::repository::{GameRepository, UserRepository};
use crate::repository::{GameRepositoryTrait, UserRepositoryTrait};
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

pub struct RepositoryTestFixture {
    pub pool: &'static PgPool,
    pub user_repo: UserRepository,
    pub game_repo: GameRepository,
}

impl RepositoryTestFixture {
    pub async fn new() -> Self {
        let pool = get_pool().await;
        Self {
            pool,
            user_repo: UserRepository::new(pool.clone()),
            game_repo: GameRepository::new(pool.clone()),
        }
    }

    pub fn random_user_name(&self) -> String {
        format!("test_user_{}", Uuid::new_v4())
    }

    pub async fn with_test_user<F, Fut, R>(&self, test_fn: F) -> R
    where
        F: FnOnce(String, UserRepository) -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        let username = self.random_user_name();

        self.user_repo
            .create_user(&username)
            .await
            .expect("Failed to create test user");

        let result = test_fn(username.clone(), self.user_repo.clone()).await;

        let _ = self.user_repo.delete_user(&username).await;

        result
    }

    pub async fn with_test_user_and_game<F, Fut, R>(&self, test_fn: F) -> R
    where
        F: FnOnce(String, UserRepository, GameRepository) -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        let username = self.random_user_name();

        self.user_repo
            .create_user(&username)
            .await
            .expect("Failed to create test user");

        let game = GameBuilder::new(&username).build();

        self.game_repo
            .upsert_game(&game)
            .await
            .expect("Failed to create test game");

        let result = test_fn(
            username.clone(),
            self.user_repo.clone(),
            self.game_repo.clone(),
        )
        .await;

        let _ = self.user_repo.delete_user(&username).await;

        result
    }
}

#[macro_export]
macro_rules! assert_repository_not_found {
    ($result:expr) => {
        match $result {
            Err(crate::errors::RepositoryError::NotFound { .. }) => {}
            Ok(_) => panic!(
                "Expected NotFound error, got success at {}",
                std::panic::Location::caller()
            ),
            Err(e) => panic!(
                "Expected NotFound error, got {:?} at {}",
                e,
                std::panic::Location::caller()
            ),
        }
    };
}

#[macro_export]
macro_rules! assert_repository_already_exists {
    ($result:expr) => {
        match $result {
            Err(crate::errors::RepositoryError::AlreadyExists { .. }) => {}
            Ok(_) => panic!(
                "Expected AlreadyExists error, got success at {}",
                std::panic::Location::caller()
            ),
            Err(e) => panic!(
                "Expected AlreadyExists error, got {:?} at {}",
                e,
                std::panic::Location::caller()
            ),
        }
    };
}

#[macro_export]
macro_rules! assert_repository_invalid_input {
    ($result:expr) => {
        match $result {
            Err(crate::errors::RepositoryError::InvalidInput { .. }) => {}
            Ok(_) => panic!(
                "Expected InvalidInput error, got success at {}",
                std::panic::Location::caller()
            ),
            Err(e) => panic!(
                "Expected InvalidInput error, got {:?} at {}",
                e,
                std::panic::Location::caller()
            ),
        }
    };
}
