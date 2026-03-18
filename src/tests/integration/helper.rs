use crate::{ConcreteAppState, config::AuthConfig};
use sqlx::{PgPool, postgres::PgPoolOptions};
use dotenv::dotenv;
use crate::builder::game_builder::GameBuilder;
use std::sync::OnceLock;
use std::env;
use uuid::Uuid;
use crate::models::Game;
use crate::services::{UserServiceTrait, GameServiceTrait};

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
pub struct HandlersFixture {
    pub app_state: ConcreteAppState,
}

impl HandlersFixture {
    pub async fn new() -> Self {
        let pool = get_pool().await;
        let app_state = ConcreteAppState::new(pool.clone(), AuthConfig::from_env());
        Self { app_state }
    }

    pub fn random_user_name(&self) -> String {
        format!("test_user_{}", Uuid::new_v4())
    }

    pub async fn with_test_user<F, Fut, R> (&self, test_fn : F) -> R
    where 
        F: FnOnce(String, String, ConcreteAppState) -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        let username = self.random_user_name();
        let _ =self.app_state.user_service.create(&username).await.expect("Failed to create test user");
        let jwt = self.app_state.auth_service.create_jwt(username.clone()).expect("Failed to create JWT");

        let result = test_fn(username.clone(), jwt, self.app_state.clone()).await;
        
        let _ = self.app_state.user_service.delete(&username).await;
        result
    }

    pub async fn with_test_user_and_game<F, Fut, R> (&self, test_fn : F) -> R
    where 
        F: FnOnce(String, String, Game, ConcreteAppState) -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        let username = self.random_user_name();
        let _ =self.app_state.user_service.create(&username).await.expect("Failed to create test user");
        let jwt = self.app_state.auth_service.create_jwt(username.clone()).expect("Failed to create JWT");

        let game = GameBuilder::new(&username).build();
        let _ = self.app_state.game_service.upsert(&game).await.expect("Failed to create test game");

        let result = test_fn(username.clone(), jwt, game, self.app_state.clone()).await;
        
        let _ = self.app_state.user_service.delete(&username).await;
        result
    }


}
