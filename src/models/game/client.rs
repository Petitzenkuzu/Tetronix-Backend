use actix_web::web::Bytes;
use serde::{Deserialize, Serialize, Deserializer, Serializer};
use sqlx::FromRow;

#[derive(Deserialize, Serialize, Debug, FromRow, PartialEq)]
pub struct ClientAction {
    pub action_type : ClientActionType,
    pub id : u32
}

impl TryFrom<Bytes> for ClientAction {
    type Error = &'static str;
    fn try_from(value: Bytes) -> Result<Self, Self::Error> {
        let action_type = ClientActionType::from_u8(value[0]);
        let id = u32::from_be_bytes(value[1..5].try_into().map_err(|_| "Invalid message length")?);
        Ok(Self { action_type, id })
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum ClientActionType {
    Right = 0x00,
    Left = 0x01,
    Rotate = 0x02,
    HardDrop = 0x03,
}

impl Serialize for ClientActionType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(self.to_u8())
    }
}

impl<'de> Deserialize<'de> for ClientActionType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        Ok(Self::from_u8(value))
    }
}

impl ClientActionType {
    pub fn to_u8(&self) -> u8 {
        match self {
            ClientActionType::Right => 0x00,
            ClientActionType::Left => 0x01,
            ClientActionType::Rotate => 0x02,
            ClientActionType::HardDrop => 0x03,
        }
    }
    pub fn from_u8(binary : u8) -> Self {
        match binary {
            0x00 => Self::Right,
            0x01 => Self::Left,
            0x02 => Self::Rotate,
            0x03 => Self::HardDrop,
            _ => Self::HardDrop,
        }
    }
}