use crate::models::PieceType;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Grid {
    pub grid: Vec<Vec<PieceType>>,
}
