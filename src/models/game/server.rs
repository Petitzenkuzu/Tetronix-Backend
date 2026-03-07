use crate::models::{PieceType};
use serde::{Deserialize, Serialize, Serializer, Deserializer};
use crate::builder::game_builder::GameBuilder;
#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct Action {
    action_type : ActionType,
    timestamp : u128,
    piece : Option<PieceType>	
}

#[derive(Debug, PartialEq)]
#[repr(u8)]
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

impl ActionType {
    pub fn to_u8(&self) -> u8 {
        match self {
            ActionType::Fall => 0x00,
            ActionType::Piece => 0x01,
            ActionType::Rotate => 0x02,
            ActionType::Right => 0x03,
            ActionType::Left => 0x04,
            ActionType::HardDrop => 0x05,
            ActionType::Start => 0x06,
            ActionType::End => 0x07,
        }
    }
    pub fn from_u8(binary : u8) -> Self {
        match binary {
            0x00 => Self::Fall,
            0x01 => Self::Piece,
            0x02 => Self::Rotate,
            0x03 => Self::Right,
            0x04 => Self::Left,
            0x05 => Self::HardDrop,
            0x06 => Self::Start,
            0x07 => Self::End,
            _ => Self::HardDrop,
        }
    }
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