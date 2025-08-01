use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Deserialize, Serialize, Debug, Clone, FromRow)]
pub struct User {
    pub name: String,
    pub number_of_games: i32,
    pub best_score: i32,
    pub highest_level: i32,
}

#[derive(Deserialize, Serialize, Debug, Clone, FromRow)]
pub struct Session {
    pub name: String,
    pub session_id: String,
}