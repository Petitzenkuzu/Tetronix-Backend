use serde::{Deserialize, Serialize, Deserializer, Serializer};
use sqlx::FromRow;

#[derive(Deserialize, Serialize, Debug, FromRow, PartialEq)]
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

#[derive(Deserialize, Serialize, Debug, Clone, FromRow, PartialEq)]
pub struct GameStats {
    pub game_score : i32,
    pub game_level : i32,
    pub game_lines : i32,
}

#[derive(Deserialize, Serialize, Debug, Clone, FromRow, PartialEq)]
pub struct Piece {
    pub shape : Vec<Vec<bool>>,
    pub color : String
}

#[derive(Deserialize, Serialize, Debug, FromRow, PartialEq)]
pub struct Action {
    pub action_type : ActionType,
    pub piece : PieceType,
    pub timestamp : i64
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum ActionType {
    Start = 0x00,
    Rotate = 0x01,
    Right = 0x02,
    Left = 0x03,
    Fall = 0x04,
    HardDrop = 0x05,
    ChangePiece = 0x06,
    End = 0x07,
    Ping = 0xFF,
}

impl Serialize for ActionType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(self.to_u8())
    }
}

impl<'de> Deserialize<'de> for ActionType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        Ok(Self::from_u8(value))
    }
}

#[derive(Debug, Clone, PartialEq)]
#[repr(u8)]
pub enum PieceType {
    Cyan = 0x00,
    Blue = 0x01,
    Yellow = 0x02,
    Orange = 0x03,
    Purple = 0x04,
    Green = 0x05,
    Red = 0x06,
    Void = 0x07,
}

impl Serialize for PieceType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(self.to_u8())
    }
}

impl<'de> Deserialize<'de> for PieceType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        Ok(Self::from_u8(value))
    }
}

pub struct Grid {
    pub grid : Vec<Vec<bool>>
}

pub enum GameResult {
    Score(i32,i32,i32),
    IllegalMove(String),
}