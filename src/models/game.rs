use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Deserialize, Serialize, Debug, Clone, FromRow)]
pub struct Game {
    pub game_owner : String,
    pub game_score : i32,
    pub game_level : i32,
    pub game_lines : i32,
    pub game_actions : Vec<Action>,
}

#[derive(Deserialize, Serialize, Debug, Clone, FromRow)]
pub struct GameJson {
    pub game_owner : String,
    pub game_score : i32,
    pub game_level : i32,
    pub game_lines : i32,
    pub game_actions : serde_json::Value,
}

#[derive(Deserialize, Serialize, Debug, Clone, FromRow)]
pub struct GameStats {
    pub game_score : i32,
    pub game_level : i32,
    pub game_lines : i32,
}

#[derive(Deserialize, Serialize, Debug, Clone, FromRow)]
pub struct Piece {
    pub shape : Vec<Vec<i32>>,
    pub color : String
}
#[derive(Deserialize, Serialize, Debug, Clone, FromRow)]
pub struct Action {
    pub action_type : String,
    pub piece : Option<Piece>,
    pub timestamp : i128
}