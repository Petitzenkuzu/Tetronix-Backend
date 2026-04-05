<<<<<<< HEAD
use crate::models::{Piece, ClientAction, ClientActionType};
=======
use crate::models::{ClientAction, ClientActionType, Piece};
>>>>>>> origin/main

use crate::builder::game_builder::GameBuilder;
use crate::game_logic::PieceRng;
use crate::game_logic::State;
use crate::models::Ack;
use crate::models::Action;
use crate::models::ActionType;
use crate::models::ServerResponse;
use std::time::Duration;
use std::time::Instant;
use tokio::sync::mpsc::{Receiver, UnboundedSender};

pub struct GameEngine {
    state: State,
    action_queue: Vec<Action>,
    last_fall: u128,
    start: Instant,
    receiver: Receiver<ClientAction>,
    sender: UnboundedSender<ServerResponse>,
    need_to_send_state: bool,
    send_ack: bool,
    missing_actions_message_pending: Option<(u32, u128)>,
}

impl GameEngine {
    pub fn start_engine(self) {
        std::thread::spawn(move || {
            let mut engine = self;
            let mut piece_rng = PieceRng::new();
            engine.start = Instant::now();
            engine.last_fall = 0_u128;
            engine.state.set_current_piece(piece_rng.get_next_piece());
            engine.state.set_next_piece(piece_rng.get_next_piece());
            engine.last_fall = 3000_u128; // we put a 3s delay at the start of the game

            let state = match serde_json::to_string(&engine.state) {
                Ok(state) => state,
                Err(e) => {
                    engine
                        .sender
                        .send(ServerResponse::InternalServerError(e.to_string()))
                        .unwrap();
                    return;
                }
            };

            engine.sender.send(ServerResponse::Start(state)).unwrap();

            engine.action_queue.push(Action::new(
                ActionType::Start,
                engine.start.elapsed().as_millis(),
                Some(engine.state.current_piece.piece_type),
            ));

            loop {
                engine.state.timestamp = engine.start.elapsed().as_millis();

                while let Ok(action) = engine.receiver.try_recv() {
                    // if the action id is less than the last processed action, skip it
                    if action.id < engine.state.last_processed_action {
                        continue;
                    }
                    // if the actions id are not consecutive, send a missing action message
                    if action.id > engine.state.last_processed_action + 1 {
                        engine
                            .sender
                            .send(ServerResponse::MissingAction(action.id.to_string()))
                            .unwrap();
                        engine.missing_actions_message_pending = Some((
                            engine.state.last_processed_action + 1,
                            engine.state.timestamp,
                        ));
                        continue;
                    }
                    // check if there is a missing action message pending and resend it if it's been too long since we asked for the missing action
                    if let Some(missing_action) = engine.missing_actions_message_pending {
                        if action.id != missing_action.0 {
                            if missing_action.1 + 1000 < engine.state.timestamp {
                                engine
                                    .sender
                                    .send(ServerResponse::MissingAction(action.id.to_string()))
                                    .unwrap();
                                engine.missing_actions_message_pending = Some((
                                    engine.state.last_processed_action + 1,
                                    engine.state.timestamp,
                                ));
                            }
                            continue;
                        } else {
                            engine.missing_actions_message_pending = None;
                        }
                    }
                    // process the action
                    let piece_changed = engine.process_action(action.action_type);
                    if piece_changed {
                        engine.state.set_next_piece(piece_rng.get_next_piece());
                    }
                    engine.state.last_processed_action += 1;
                }

                let piece_changed = engine.process_fall();
                if piece_changed {
                    engine.state.set_next_piece(piece_rng.get_next_piece());
                }

                if engine.need_to_send_state {
                    engine.need_to_send_state = false;
                    engine.send_ack = false;
                    let state = serde_json::to_string(&engine.state).unwrap();
                    match engine.state.finished {
                        true => {
                            let game_builder = GameBuilder::new("")
                                .with_actions(std::mem::take(&mut engine.action_queue))
                                .with_score(engine.state.score)
                                .with_level(engine.state.level)
                                .with_lines(engine.state.lines);
                            engine.sender.send(ServerResponse::End(state)).unwrap();
                            engine
                                .sender
                                .send(ServerResponse::Game(game_builder))
                                .unwrap();
                        }
                        false => {
                            engine.sender.send(ServerResponse::State(state)).unwrap();
                        }
                    }
                } else if engine.send_ack {
                    engine.send_ack = false;
                    let ack = Ack::new(engine.state.last_processed_action);
                    let ack_str = serde_json::to_string(&ack).unwrap();
                    engine.sender.send(ServerResponse::Ack(ack_str)).unwrap();
                }
                std::thread::sleep(Duration::from_millis(16)); // 60 tick per seconds
            }
        });
    }
}

impl GameEngine {
    pub fn new(receiver: Receiver<ClientAction>, sender: UnboundedSender<ServerResponse>) -> Self {
        Self {
            state: State::new(),
            action_queue: vec![],
            last_fall: 0,
            start: Instant::now(),
            receiver,
            sender,
            need_to_send_state: false,
            send_ack: false,
            missing_actions_message_pending: None,
        }
    }

    /// Returns a boolean indicating if the piece has been changed
    fn process_fall(&mut self) -> bool {
        // fall time formula 1000*(0.8**level.value)
        if self.state.timestamp
            > self.last_fall + (1000_f32 * (0.8_f32.powi(self.state.level))) as u128
        {
            if self
                .state
                .grid
                .is_placeable(&self.state.current_piece, (self.state.x + 1, self.state.y))
            {
                self.move_down();
                self.action_queue.push(Action::new(
                    ActionType::Fall,
                    self.state.timestamp,
                    Some(self.state.current_piece.piece_type),
                ));
                self.last_fall = self.state.timestamp;
                return false;
            } else {
                self.last_fall = self.state.timestamp;
                self.need_to_send_state = true;
                self.change_piece();
                return true;
            }
        }
        false
    }

    /// Returns a boolean indicating if the piece has been changed
    fn process_action(&mut self, action: ClientActionType) -> bool {
        match action {
            ClientActionType::Right => {
                if self
                    .state
                    .grid
                    .is_placeable(&self.state.current_piece, (self.state.x, self.state.y + 1))
                {
                    self.move_right();
                    self.action_queue.push(Action::new(
                        ActionType::Right,
                        self.state.timestamp,
                        Some(self.state.current_piece.piece_type),
                    ));
                    self.send_ack = true;
                }
                false
            }
            ClientActionType::Left => {
                if self
                    .state
                    .grid
                    .is_placeable(&self.state.current_piece, (self.state.x, self.state.y - 1))
                {
                    self.move_left();
                    self.action_queue.push(Action::new(
                        ActionType::Left,
                        self.state.timestamp,
                        Some(self.state.current_piece.piece_type),
                    ));
                    self.send_ack = true;
                }
                false
            }
            ClientActionType::HardDrop => {
                self.action_queue.push(Action::new(
                    ActionType::HardDrop,
                    self.state.timestamp,
                    Some(self.state.current_piece.piece_type),
                ));
                let ghost_x = self
                    .state
                    .grid
                    .get_ghost_x(&self.state.current_piece, (self.state.x, self.state.y));
                let diff = ghost_x - self.state.x;
                if diff > 0 {
                    self.state.add_to_score(diff * (self.state.level * 10));
                }
                self.state.x = ghost_x;
                self.change_piece();
                self.need_to_send_state = true;
                self.last_fall = self.state.timestamp;
                true
            }
            ClientActionType::Rotate => {
                let new_shape = self.state.current_piece.rotate();
                let piece = Piece {
                    shape: new_shape,
                    piece_type: self.state.current_piece.piece_type,
                };
                if self
                    .state
                    .grid
                    .is_placeable(&piece, (self.state.x, self.state.y))
                {
                    self.state.set_current_piece(piece);
                    self.action_queue.push(Action::new(
                        ActionType::Rotate,
                        self.state.timestamp,
                        Some(self.state.current_piece.piece_type),
                    ));
                    self.send_ack = true;
                }
                false
            }
        }
    }

    fn move_right(&mut self) {
        self.state.y += 1;
    }

    fn move_left(&mut self) {
        self.state.y -= 1;
    }

    fn move_down(&mut self) {
        self.state.x += 1;
    }

    fn change_piece(&mut self) {
        let new_piece = std::mem::take(&mut self.state.next_piece);
        self.state
            .grid
            .place_piece(&self.state.current_piece, (self.state.x, self.state.y));
        let (lines_cleared, score) = self.state.grid.delete_full_rows(self.state.level);
        self.state.add_to_score(score);

        if self.state.lines % 10 > (self.state.lines + lines_cleared) % 10 {
            self.state.add_to_level(1);
        }
        self.state.add_to_lines(lines_cleared);

        self.state.x = 0;
        self.state.y = 4;
        if self
            .state
            .grid
            .is_placeable(&new_piece, (self.state.x, self.state.y))
        {
            self.state.set_current_piece(new_piece);
            self.action_queue.push(Action::new(
                ActionType::Piece,
                self.state.timestamp,
                Some(self.state.current_piece.piece_type),
            ));
        } else {
            self.state.finished = true;
            self.action_queue
                .push(Action::new(ActionType::End, self.state.timestamp, None));
            self.need_to_send_state = true;
        }
    }
}
