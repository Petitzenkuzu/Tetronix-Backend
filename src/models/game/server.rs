use crate::models::{PieceType};
use serde::{Deserialize, Serialize};
use crate::builder::game_builder::GameBuilder;
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

#[derive(Deserialize, Serialize)]
pub struct Ack {
    id : u32,
}

impl Ack {
    pub fn new(id: u32) -> Self {
        Self { id }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum ServerResponse {
    Start(String),
    State(String),
    Ack(String),
    End(String),
    Game(GameBuilder),
    MissingAction(String),
    InternalServerError(String),
}