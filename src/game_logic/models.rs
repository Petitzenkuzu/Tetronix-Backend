use crate::models::PieceType;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Grid {
    pub grid : Vec<Vec<PieceType>>
}