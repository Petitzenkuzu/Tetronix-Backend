use crate::{AppState, config::{AuthConfig, SessionConfig}, services::{AuthService, GameService, SessionService, UserService}, repository::{GameRepository, SessionRepository, UserRepository}};
use sqlx::{PgPool, postgres::PgPoolOptions};
use dotenv::dotenv;
use crate::tests::unit::helpers::common_helpers::GameBuilder;
use std::sync::OnceLock;
use std::env;
use uuid::Uuid;
use crate::models::Game;

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
    pub app_state: AppState,
}

impl HandlersFixture {
    pub async fn new() -> Self {
        let pool = get_pool().await;
        let app_state = AppState {
            auth_service: AuthService::new(UserRepository::new(pool.clone()), SessionRepository::new(pool.clone()), AuthConfig::from_env()),
            session_service: SessionService::new(SessionRepository::new(pool.clone()), SessionConfig::from_env()),
            game_service: GameService::new(GameRepository::new(pool.clone())),
            user_service: UserService::new(UserRepository::new(pool.clone())),
        };
        Self { app_state }
    }

    pub fn random_user_name(&self) -> String {
        format!("test_user_{}", Uuid::new_v4())
    }
    
    pub fn random_session_hash(&self) -> String {
        format!("test_session_{}", Uuid::new_v4())
    }

    pub async fn with_test_user_and_session<F, Fut, R> (&self, test_fn : F) -> R
    where 
        F: FnOnce(String, String, AppState) -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        let username = self.random_user_name();
        let _ =self.app_state.user_service.create(&username).await.expect("Failed to create test user");

        let session_hash = self.random_session_hash();
        let _ = self.app_state.session_service.create(&username, &session_hash).await.expect("Failed to create test session");

        let result = test_fn(username.clone(), session_hash.clone(), self.app_state.clone()).await;
        
        let _ = self.app_state.session_service.delete(&session_hash).await;
        let _ = self.app_state.user_service.delete(&username).await;
        result
    }

    pub async fn with_test_user_and_session_and_game<F, Fut, R> (&self, test_fn : F) -> R
    where 
        F: FnOnce(String, String, Game, AppState) -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        let username = self.random_user_name();
        let _ =self.app_state.user_service.create(&username).await.expect("Failed to create test user");

        let session_hash = self.random_session_hash();
        let _ = self.app_state.session_service.create(&username, &session_hash).await.expect("Failed to create test session");

        let game = GameBuilder::new(&username).build();
        let _ = self.app_state.game_service.upsert(&game).await.expect("Failed to create test game");

        let result = test_fn(username.clone(), session_hash.clone(), game, self.app_state.clone()).await;
        
        let _ = self.app_state.user_service.delete(&username).await;
        result
    }


}
