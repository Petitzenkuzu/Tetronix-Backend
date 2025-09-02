use crate::errors::ServicesError;
use crate::models::User;
use crate::models::Session;
use crate::models::Game;
use crate::models::GameStats;
use crate::repository::UserRepositoryTrait;
use crate::repository::SessionRepositoryTrait;
use crate::config::SessionConfig;
use crate::repository::GameRepositoryTrait;
use crate::config::AuthConfig;
use actix_web::cookie::Cookie;

pub trait UserServiceTrait: Clone {
    async fn create(&self, name: &str) -> Result<(), ServicesError>;
    async fn update(&self, user: &User) -> Result<(), ServicesError>;
    async fn get_by_name(&self, name: &str) -> Result<User, ServicesError>;
    async fn get_top(&self, limit: i32) -> Result<Vec<User>, ServicesError>;
    async fn delete(&self, name: &str) -> Result<(), ServicesError>;
}

pub trait SessionServiceTrait: Clone {
    fn hash_session_id(&self, session_id: &str) -> String;
    async fn create(&self, name: &str, session_hash: &str) -> Result<(), ServicesError>;
    async fn get_by_id(&self, session_hash: &str) -> Result<Session, ServicesError>;
    async fn delete(&self, session_hash: &str) -> Result<(), ServicesError>;
}

pub trait GameServiceTrait: Clone {
    async fn upsert(&self, game: &Game) -> Result<(), ServicesError>;
    async fn get_by_owner(&self, owner: &str) -> Result<Game, ServicesError>;
    async fn get_stats(&self, owner: &str) -> Result<GameStats, ServicesError>;
}

pub trait AuthServiceTrait: Clone {
    async fn authenticate_with_github(&self, code: &str, redirect_uri: &str) -> Result<String, ServicesError>;
    fn create_cookies(&self, session_id: &str) -> Cookie;
    fn logout_cookies(&self) -> Cookie;
}
