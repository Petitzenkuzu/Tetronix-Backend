use crate::game_logic::Grid;
use crate::models::Piece;
use crate::models::PieceType;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct State {
    pub grid: Grid,
    pub current_piece: Piece,
    pub next_piece: Piece,
    pub score: i32,
    pub level: i32,
    pub lines: i32,
    pub x: i32,
    pub y: i32,
    pub last_processed_action: u32,
    pub finished: bool,
    pub timestamp: u128,
}

impl State {
    pub fn new() -> Self {
        Self {
            grid: Grid::new(),
            current_piece: Piece {
                shape: vec![],
                piece_type: PieceType::Empty,
            },
            next_piece: Piece {
                shape: vec![],
                piece_type: PieceType::Empty,
            },
            score: 0,
            level: 1,
            lines: 0,
            x: 0,
            y: 4,
            finished: false,
            last_processed_action: 0,
            timestamp: 0,
        }
    }

    pub fn set_current_piece(&mut self, piece: Piece) {
        self.current_piece = piece;
    }

    pub fn set_next_piece(&mut self, piece: Piece) {
        self.next_piece = piece;
    }

    pub fn add_to_score(&mut self, score: i32) {
        self.score += score;
    }

    pub fn add_to_level(&mut self, level: i32) {
        self.level += level;
    }

    pub fn add_to_lines(&mut self, lines: i32) {
        self.lines += lines;
    }
}
