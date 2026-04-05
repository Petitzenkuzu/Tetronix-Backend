use crate::errors::ServicesError;
use crate::models::Game;
use crate::models::GameStats;
use crate::models::User;
use actix_web::cookie::Cookie;

#[allow(dead_code)]
pub trait UserServiceTrait: Clone {
    async fn create(&self, name: &str) -> Result<(), ServicesError>;
    async fn update(&self, user: &User) -> Result<(), ServicesError>;
    async fn get_by_name(&self, name: &str) -> Result<User, ServicesError>;
    async fn get_top(&self, limit: i32) -> Result<Vec<User>, ServicesError>;
    async fn delete(&self, name: &str) -> Result<(), ServicesError>;
}

pub trait GameServiceTrait: Clone {
    async fn upsert(&self, game: &Game) -> Result<(), ServicesError>;
    async fn get_by_owner(&self, owner: &str) -> Result<Game, ServicesError>;
    async fn get_stats(&self, owner: &str) -> Result<GameStats, ServicesError>;
}

pub trait AuthServiceTrait: Clone {
    async fn authenticate_with_github(
        &self,
        code: &str,
        redirect_uri: &str,
    ) -> Result<String, ServicesError>;
    fn create_cookies(&self, jwt: String) -> Cookie<'_>;
    fn logout_cookies(&self) -> Cookie<'_>;
    fn verify_jwt(&self, jwt: &str) -> Result<String, ServicesError>;
}
