use crate::models::{Piece, Action, ActionType, Grid, PieceType};

pub struct GameEngine {
    grid : Grid,
    current_piece : Piece,
    score : i32,
    level : i32,
    lines : i32,
    x : i32,
    y : i32,
}

impl GameEngine {
    pub fn new() -> Self {
        Self {
            grid: Grid::new(),
            current_piece: Piece {
                shape: vec![vec![true, true], vec![true, true]],
                color: "yellow".to_string(),
            },
            score: 0,
            level: 1,
            lines: 0,
            x: 0,
            y: 4,
        }
    }

    pub fn handle_action(&mut self, action : &Action) -> Option<(i32, i32, i32)> {
        match action.action_type {
            ActionType::Start => {
                self.start(action.piece.clone());
            },
            ActionType::Left => {
                if self.grid.is_placeable(&self.current_piece, (self.x, self.y - 1)) {
                    self.move_left();
                }
            },
            ActionType::Right => {
                if self.grid.is_placeable(&self.current_piece, (self.x, self.y + 1)) {
                    self.move_right();
                }
            },
            ActionType::Fall => {
                if self.grid.is_placeable(&self.current_piece, (self.x + 1, self.y)) {
                    self.move_down();
                }
            },
            ActionType::HardDrop => {
                let ghost_x = self.grid.get_ghost_x(&self.current_piece, (self.x , self.y ));
                self.hard_drop(ghost_x);
            },
            ActionType::Rotate => {
                let new_piece = Piece { shape: self.current_piece.rotate(), color: self.current_piece.color.clone() };
                if self.grid.is_placeable(&new_piece, (self.x , self.y)) {
                    self.current_piece = new_piece;
                }
            },
            ActionType::ChangePiece => {
                self.change_piece(action.piece.clone());
            },
            ActionType::End => {
                return Some(self.end());
            },
            _ => {}
        }
        None
    }

    fn start(&mut self, piece : PieceType) {
        self.current_piece = Piece::from_u8(piece.to_u8());
        self.x = 0;
        self.y = 4;
        
    }

    fn move_left(&mut self) {
        self.y -= 1;
    }

    fn move_right(&mut self) {
        self.y += 1;
    }

    fn move_down(&mut self) {
        self.x += 1;
    }
    fn rotate(&mut self) {
        let new_shape = self.current_piece.rotate();
        if self.grid.is_placeable(&Piece { shape: new_shape.clone(), color: self.current_piece.color.clone() }, (self.x , self.y)) {
            self.current_piece.shape = new_shape;
        }
    }

    fn change_piece(&mut self, new_piece : PieceType) {
        self.grid.place_piece(&self.current_piece, (self.x , self.y));
        let (lines_cleared, score) = self.grid.delete_full_rows(self.level);
        self.score += score;
        if self.lines%10 > (self.lines+lines_cleared)%10 {
            self.level += 1;
        }
        self.lines += lines_cleared;
        self.current_piece = Piece::from_u8(new_piece.to_u8());
        self.x = 0;
        self.y = 4;
    }
    
    fn hard_drop(&mut self, ghost_x : i32) {
        let diff = ghost_x - self.x;
        self.x = ghost_x;
        self.score += diff * (self.level * 10);
    }
    fn end(&mut self) -> (i32, i32, i32) {
        return (self.score, self.level, self.lines);
    }

}


