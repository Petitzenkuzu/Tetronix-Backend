use crate::errors::RepositoryError;
use crate::models::{Game, GameStats, User};

#[allow(dead_code)]
pub trait UserRepositoryTrait: Clone {
    async fn create_user(&self, name: &str) -> Result<(), RepositoryError>;
    async fn update_user(&self, user: &User) -> Result<(), RepositoryError>;
    async fn get_user_by_name(&self, name: &str) -> Result<User, RepositoryError>;
    async fn get_top_users(&self, limit: i32) -> Result<Vec<User>, RepositoryError>;
    async fn delete_user(&self, name: &str) -> Result<(), RepositoryError>;
}

pub trait GameRepositoryTrait: Clone {
    async fn upsert_game(&self, game: &Game) -> Result<(), RepositoryError>;
    async fn get_game_by_owner(&self, owner: &str) -> Result<Game, RepositoryError>;
    async fn get_game_stats_by_owner(&self, owner: &str) -> Result<GameStats, RepositoryError>;
}
