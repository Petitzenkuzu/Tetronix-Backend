mod client;
mod server;

pub use server::Action;
pub use server::ActionType;
pub use server::ServerResponse;

pub use client::ClientAction;
pub use client::ClientActionType;

use serde::{Deserialize, Serialize, Deserializer, Serializer};
use sqlx::FromRow;
use actix_ws::{CloseCode, CloseReason};

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
    pub piece_type : PieceType
}

impl From<PieceType> for Piece {
    fn from(piece_type : PieceType) -> Self {
        match piece_type {
            PieceType::Cyan => Piece { shape: vec![vec![false, true, false, false], vec![false, true, false, false], vec![false, true, false, false], vec![false, true, false, false]], piece_type: PieceType::Cyan },
            PieceType::Blue => Piece { shape: vec![vec![false, true, false], vec![false, true, false], vec![false, true, true]], piece_type: PieceType::Blue },
            PieceType::Yellow => Piece { shape: vec![vec![true, true], vec![true, true]], piece_type: PieceType::Yellow },
            PieceType::Orange => Piece { shape: vec![vec![false, true, false], vec![false, true, false], vec![true, true, false]], piece_type: PieceType::Orange },
            PieceType::Purple => Piece { shape: vec![vec![false, true, false], vec![true, true, true], vec![false, false, false]], piece_type: PieceType::Purple },
            PieceType::Green => Piece { shape: vec![vec![true, true, false], vec![false, true, true], vec![false, false, false]], piece_type: PieceType::Green },
            PieceType::Red => Piece { shape: vec![vec![false, true, true], vec![true, true, false], vec![false, false, false]], piece_type: PieceType::Red },
            PieceType::Empty => Piece { shape: vec![], piece_type: PieceType::Empty },
        }
    }
}

impl From<u8> for PieceType {
    fn from(value : u8) -> Self {
        match value {
            0x00 => PieceType::Cyan,
            0x01 => PieceType::Blue,
            0x02 => PieceType::Yellow,
            0x03 => PieceType::Orange,
            0x04 => PieceType::Purple,
            0x05 => PieceType::Green,
            0x06 => PieceType::Red,
            0x07 => PieceType::Empty,
            _ => PieceType::Empty,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
#[repr(u8)]
pub enum PieceType {
    Cyan = 0x00,
    Blue = 0x01,
    Yellow = 0x02,
    Orange = 0x03,
    Purple = 0x04,
    Green = 0x05,
    Red = 0x06,
    Empty = 0x07,
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

pub enum GameResult {
    Score(i32,i32,i32),
    IllegalMove,
}

pub enum GameCloseReason {  
    GameEnded,
    IllegalMove,
    InternalError,
    InvalidMessageLength,
    NoUserFound,
    Timeout,
}

impl GameCloseReason {
    pub fn to_close_reason(&self) -> CloseReason {
        match self {
            GameCloseReason::GameEnded => CloseReason {
                code: CloseCode::Normal,
                description: Some("Game ended".to_string()),
            },
            GameCloseReason::IllegalMove => CloseReason {
                code: CloseCode::Error,
                description: Some("Illegal move".to_string()),
            },
            GameCloseReason::InternalError => CloseReason {
                code: CloseCode::Error,
                description: Some("Internal error".to_string()),
            },
            GameCloseReason::InvalidMessageLength => CloseReason {
                code: CloseCode::Error,
                description: Some("Invalid message length".to_string()),
            },
            GameCloseReason::NoUserFound => CloseReason {
                code: CloseCode::Policy,
                description: Some("No user found".to_string()),
            },
            GameCloseReason::Timeout => CloseReason {
                code: CloseCode::Error,
                description: Some("Timeout".to_string()),
            },
        }
    }
}