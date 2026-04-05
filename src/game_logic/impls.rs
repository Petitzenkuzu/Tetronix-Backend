use crate::game_logic::Grid;
use crate::models::{Piece, PieceType};

impl Piece {
    pub fn rotate(&self) -> Vec<Vec<bool>> {
        let mut new_shape = vec![vec![false; self.shape.len()]; self.shape[0].len()];
        for i in 0..self.shape.len() {
            for (j, _) in self.shape[i].iter().enumerate() {
                new_shape[j][self.shape.len() - i - 1] = self.shape[i][j];
            }
        }
        new_shape
    }
}

impl PieceType {
    pub fn to_u8(self) -> u8 {
        match self {
            PieceType::Cyan => 0x00,
            PieceType::Blue => 0x01,
            PieceType::Yellow => 0x02,
            PieceType::Orange => 0x03,
            PieceType::Purple => 0x04,
            PieceType::Green => 0x05,
            PieceType::Red => 0x06,
            PieceType::Empty => 0x07,
        }
    }

    pub fn from_u8(binary: u8) -> Self {
        match binary {
            0x00 => Self::Cyan,
            0x01 => Self::Blue,
            0x02 => Self::Yellow,
            0x03 => Self::Orange,
            0x04 => Self::Purple,
            0x05 => Self::Green,
            0x06 => Self::Red,
            0x07 => Self::Empty,
            _ => Self::Empty,
        }
    }
}

impl Grid {
    pub fn new() -> Self {
        Self {
            grid: vec![vec![PieceType::Empty; 10]; 20],
        }
    }

    pub fn is_placeable(&self, piece: &Piece, (x, y): (i32, i32)) -> bool {
        for i in 0..piece.shape.len() {
            for j in 0..piece.shape[i].len() {
                if piece.shape[i][j] {
                    if (x + i as i32) < 0 || (y + j as i32) < 0 {
                        return false;
                    }
                    if (x + i as i32) >= self.grid.len() as i32
                        || (y + j as i32) >= self.grid[0].len() as i32
                    {
                        return false;
                    }
                    if self.grid[(x + i as i32) as usize][(y + j as i32) as usize]
                        != PieceType::Empty
                    {
                        return false;
                    }
                }
            }
        }
        true
    }

    pub fn place_piece(&mut self, piece: &Piece, (x, y): (i32, i32)) {
        for i in 0..piece.shape.len() {
            for j in 0..piece.shape[i].len() {
                if piece.shape[i][j] {
                    self.grid[x as usize + i][(y + j as i32) as usize] = piece.piece_type;
                }
            }
        }
    }

    pub fn get_ghost_x(&self, piece: &Piece, (x, y): (i32, i32)) -> i32 {
        let mut ghost_x = x + 1;
        while self.is_placeable(piece, (ghost_x, y)) {
            ghost_x += 1;
        }
        ghost_x - 1
    }
    pub fn delete_full_rows(&mut self, level: i32) -> (i32, i32) {
        let mut lines_cleared = 0_i32;
        let mut score = 0_i32;
        let mut row = (self.grid.len() - 1) as i32;
        let mut temp_grid = vec![vec![PieceType::Empty; self.grid[0].len()]; self.grid.len()];
        for i in (0..self.grid.len()).rev() {
            if self.grid[i].iter().all(|&cell| cell != PieceType::Empty) {
                lines_cleared += 1;
            } else {
                temp_grid[row as usize] = std::mem::take(&mut self.grid[i]);
                row -= 1;
            }
        }
        match lines_cleared {
            1 => score += 40 * level,
            2 => score += 100 * level,
            3 => score += 300 * level,
            4 => score += 1200 * level,
            _ => (),
        }
        self.grid = temp_grid;
        (lines_cleared, score)
    }
}
