use crate::models::{Piece, ClientAction, ClientActionType, PieceType};
use crate::game_logic::Grid;
use crate::models::{GameResult, Action};
use tokio::sync::mpsc::{Receiver, UnboundedSender};
use std::time::Instant;
use std::time::Duration;
use crate::game_logic::PieceRng;
use crate::models::ActionType;
use crate::models::ServerResponse;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct GameEngine {
    grid : Grid,
    current_piece : Piece,
    score : u32,
    level : i32,
    lines : u32,
    x : i32,
    y : i32,
    last_processed_action : u32,
    finished : bool,
    timestamp : u128,
}

impl GameEngine {
    pub fn start_engine(self, mut receiver : Receiver<ClientAction>, sender : UnboundedSender<ServerResponse>) {
        std::thread::spawn(move || {
            let mut engine = self;
            let mut action_queue : Vec<Action> = vec![];
            let start = Instant::now();
            let mut last_fall = 0_u128;
            let mut piece_rng = PieceRng::new();
            engine.current_piece = piece_rng.get_next_piece();

            let state = match serde_json::to_string(&engine) {
                Ok(state) => state,
                Err(e) => {
                    sender.send(ServerResponse::InternalServerError(e.to_string())).unwrap();
                    return;
                }
            };

            if let Err(e) = sender.send(ServerResponse::Start(state)) {
                tracing::error!("Error sending start: {}", e);
                return;
            }

            loop {
                let mut need_to_send_state = false;
                engine.timestamp = start.elapsed().as_millis();

                while let Ok(action) = receiver.try_recv() {
                    println!("Action received: {:?}", action);
                    let action = engine.process_action(action.action_type, engine.timestamp);
                    if let Some(action) = action {
                        action_queue.push(action);
                        need_to_send_state = true;
                    }
                }

                let (action, need_to_change_piece) = engine.process_fall(last_fall, engine.timestamp);
                if let Some(action) = action {
                    action_queue.push(action);
                    last_fall = engine.timestamp;
                    need_to_send_state = true;
                }
                if need_to_change_piece {
                    engine.change_piece(piece_rng.get_next_piece());
                }

                if need_to_send_state {
                    let state = match serde_json::to_string(&engine) {
                        Ok(state) => state,
                        Err(e) => {
                            tracing::error!("Error serializing grid: {}", e);
                            sender.send(ServerResponse::InternalServerError(e.to_string())).unwrap();
                            return;
                        }
                    };
                    if let Err(e) = sender.send(ServerResponse::State(state)) {
                        tracing::error!("Error sending state: {}", e);
                        return;
                    }
                }
                std::thread::sleep(Duration::from_millis(16)); // 60 tick per seconds
            }

        });
    }
}

impl GameEngine {
    pub fn new() -> Self {
        Self {
            grid: Grid::new(),
            current_piece: Piece {
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

    /// Returns a tuple containing an optional action and a boolean indicating if the we need to change current piece
    fn process_fall(&mut self, last_fall : u128, now : u128, ) -> (Option<Action>, bool) {
        // fall time formula 1000*(0.8**level.value)
        if now > last_fall + (1000_f32*(0.8_f32.powi(self.level as i32))) as u128 {
            if self.grid.is_placeable(&self.current_piece, (self.x+1 , self.y)) {
                self.move_down();
                return (Some(Action::new(ActionType::Fall, now, Some(self.current_piece.piece_type))), false);
            }
            else {
                return (None, true);
            }
        }
        (None, false)
    }

    fn process_action(&mut self, action : ClientActionType, now : u128) -> Option<Action> {
        match action {
            ClientActionType::Right => {
                if self.grid.is_placeable(&self.current_piece, (self.x , self.y+1)) {
                    self.move_right();
                    return Some(Action::new(ActionType::Right, now, Some(self.current_piece.piece_type)));
                }
                else {
                    return None;
                }
            },
            ClientActionType::Left => {
                if self.grid.is_placeable(&self.current_piece, (self.x , self.y-1)) {
                    self.move_left();
                    return Some(Action::new(ActionType::Left, now, Some(self.current_piece.piece_type)));
                }
                else {
                    return None;
                }
            },
            ClientActionType::Rotate => {
                let new_shape = self.current_piece.rotate();
                let piece = Piece { shape: new_shape, piece_type: self.current_piece.piece_type };
                if self.grid.is_placeable(&piece, (self.x , self.y)) {
                    self.current_piece = piece;
                    return Some(Action::new(ActionType::Rotate, now, Some(self.current_piece.piece_type)));
                }
                else {
                    return None;
                }
            },
            ClientActionType::HardDrop => None,
            _ => return None,
        }
    }

    fn move_right(&mut self) {
        self.y += 1;
    }

    fn move_left(&mut self) {
        self.y -= 1;
    }
    
    fn move_down(&mut self) {
        self.x += 1;
    }

    fn rotate(&mut self) {
        let new_shape = self.current_piece.rotate();
        if self.grid.is_placeable(&Piece { shape: new_shape.clone(), piece_type: self.current_piece.piece_type }, (self.x , self.y)) {
            self.current_piece.shape = new_shape;
        }
    }

    fn change_piece(&mut self, new_piece : Piece) -> Option<Action> {
        self.grid.place_piece(&self.current_piece, (self.x , self.y));
        let (lines_cleared, score) = self.grid.delete_full_rows(self.level);
        self.score += score;
        if self.lines%10 > (self.lines+lines_cleared)%10 {
            self.level += 1;
        }
        self.lines += lines_cleared;
        if self.grid.is_placeable(&new_piece, (self.x , self.y)) {
            self.current_piece = new_piece;
            self.x = 0;
            self.y = 4;
            return Some(Action::new(ActionType::Piece, self.timestamp, Some(self.current_piece.piece_type)));
        }
        else {
            self.finished = true;
            return None;
        }
    }
}


