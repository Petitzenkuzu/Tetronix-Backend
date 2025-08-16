use crate::models::{Piece, Grid, PieceType};

impl Piece {
    pub fn rotate(&self) -> Vec<Vec<bool>> {
        let mut new_shape = vec![vec![false; self.shape.len()]; self.shape[0].len()];
        for i in 0..self.shape.len(){
            for j in 0..self.shape[i].len(){
                new_shape[j][self.shape.len() - i - 1] = self.shape[i][j];
            }
        }
        new_shape
    }

    pub fn from_u8(binary : u8) -> Self {
        match binary {
            0 => Self {
                shape: vec![
                    vec![false, true, false, false],
                    vec![false, true, false, false],
                    vec![false, true, false, false],
                    vec![false, true, false, false]
                ],
                color: "cyan".to_string(),
            },
            1 => Self {
                shape: vec![
                    vec![false, true, false],
                    vec![false, true, false],
                    vec![false, true, true]
                ],
                color: "blue".to_string(),
            },
            2 => Self {
                shape: vec![
                    vec![true, true], 
                    vec![true, true]
                ],
                color: "yellow".to_string(),
            },
            3 => Self {
                shape: vec![
                    vec![false, true, false],
                    vec![false, true, false],
                    vec![true, true, false]
                ],
                color: "orange".to_string(),
            },
            4 => Self {
                shape: vec![
                    vec![false, true, false],
                    vec![true, true, true],
                    vec![false, false, false]
                ],
                color: "purple".to_string(),
            },
            5 => Self {
                shape: vec![
                    vec![true, true, false],
                    vec![false, true, true],
                    vec![false, false, false]
                ],
                color: "green".to_string(),
            },
            6 => Self {
                shape: vec![
                    vec![false, true, true],
                    vec![true, true, false],
                    vec![false, false, false]
                ],
                color: "red".to_string(),
            },
            7 => Self {
                shape: vec![vec![false; 2]; 2],
                color: "void".to_string(),
            },
            _ => Self {
                shape: vec![vec![false; 2]; 2],
                color: "void".to_string(),
            },
        }
    }
}

use crate::models::ActionType;

impl ActionType {
    pub fn to_u8(&self) -> u8 {
        match self {
            ActionType::Start => 0x00,
            ActionType::Rotate => 0x01,
            ActionType::Right => 0x02,
            ActionType::Left => 0x03,
            ActionType::Fall => 0x04,
            ActionType::HardDrop => 0x05,
            ActionType::ChangePiece => 0x06,
            ActionType::End => 0x07,
            ActionType::Ping => 0xFF,
        }
    }
    pub fn from_u8(binary : u8) -> Self {
        match binary {
            0x00 => Self::Start,
            0x01 => Self::Rotate,
            0x02 => Self::Right,
            0x03 => Self::Left,
            0x04 => Self::Fall,
            0x05 => Self::HardDrop,
            0x06 => Self::ChangePiece,
            0x07 => Self::End,
            0xFF => Self::Ping,
            _ => Self::End,
        }
    }
}

impl PieceType {
    pub fn to_u8(&self) -> u8 {
        match self {
            PieceType::Cyan => 0x00,
            PieceType::Blue => 0x01,
            PieceType::Yellow => 0x02,
            PieceType::Orange => 0x03,
            PieceType::Purple => 0x04,
            PieceType::Green => 0x05,
            PieceType::Red => 0x06,
            PieceType::Void => 0x07,
        }
    }

    pub fn from_u8(binary : u8) -> Self {
        match binary {
            0x00 => Self::Cyan,
            0x01 => Self::Blue,
            0x02 => Self::Yellow,
            0x03 => Self::Orange,
            0x04 => Self::Purple,
            0x05 => Self::Green,
            0x06 => Self::Red,
            0x07 => Self::Void,
            _ => Self::Void,
        }
    }
}

impl Grid {
    pub fn new() -> Self {
        Self {
            grid: vec![vec![false; 10]; 20],
        }
    }

    pub fn is_placeable(&self, piece : &Piece, (x, y) : (i32, i32)) -> bool {
        for i in 0..piece.shape.len(){
            for j in 0..piece.shape[i].len(){
                if piece.shape[i][j] {
                    if (x + i as i32) < 0 || (y + j as i32) < 0 {
                        return false;
                    }
                    if (x + i as i32) >= self.grid.len() as i32 || (y + j as i32) >= self.grid[0].len() as i32 {
                        return false;
                    }
                    if self.grid[(x + i as i32) as usize][(y + j as i32) as usize] {
                        return false;
                    }
                }
            }
        }
        true
    }

    pub fn place_piece(&mut self, piece : &Piece, (x, y) : (i32, i32)) {
        for i in 0..piece.shape.len(){
            for j in 0..piece.shape[i].len(){
                if piece.shape[i][j] {
                    self.grid[x as usize + i][(y + j as i32) as usize] = true;
                }
            }
        }
    }

    pub fn get_ghost_x(&self, piece : &Piece, (x, y) : (i32, i32)) -> i32 {   
        let mut ghost_x = x+1;
        while self.is_placeable(piece, (ghost_x, y)) {
            ghost_x += 1;
        }
        ghost_x - 1
    }

    pub fn delete_full_rows(&mut self, level : i32) -> (i32, i32) {
        let mut lines_cleared = 0;
        let mut score = 0;
        let mut row = (self.grid.len()-1) as i32;
        let mut temp_grid = vec![vec![false; self.grid[0].len()]; self.grid.len()];
        for i in (0..self.grid.len()).rev() {
            if self.grid[i].iter().all(|&cell| cell) {
                lines_cleared += 1;
            }
            else {
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
