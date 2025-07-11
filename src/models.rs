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

#[derive(Deserialize)]
pub struct GithubAuth {
    pub code: String,
}

#[derive(Deserialize)]
pub struct GithubAuthMobile {
    pub code: String,
    pub redirect_uri: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GithubTokenResponse {
    pub access_token: String,
    pub token_type: String
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GithubUser {
    pub login: String,
    pub id: Option<u64>,
    pub name: Option<String>,
    pub avatar_url: Option<String>,

}
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
    pub game_actions : String,
}

#[derive(Deserialize, Serialize, Debug, Clone, FromRow)]
pub struct Piece {
    pub shape : Vec<Vec<i32>>,
    pub color : String
}
#[derive(Deserialize, Serialize, Debug, Clone, FromRow)]
pub struct Action {
    pub action_type : String,
    pub piece : Piece,
    pub times_tamp : i32
}