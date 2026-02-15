use crate::models::{PieceType};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct Action {
    action_type : ActionType,
    timestamp : u128,
    piece : Option<PieceType>	
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub enum ActionType {
    Fall = 0x00,
    Piece = 0x01,
    Rotate = 0x02,
    Right = 0x03,
    Left = 0x04,
    HardDrop = 0x05,
    Start = 0x06,
    End = 0x07,
}

impl Action {
    pub fn new(action_type: ActionType, timestamp: u128, piece: Option<PieceType>) -> Self {
        Self { action_type, timestamp, piece }
    }
}

pub enum ServerResponse {
    Start(String),
    State(String),
    End(String),
    MissingAction(String),
    InternalServerError(String),
}