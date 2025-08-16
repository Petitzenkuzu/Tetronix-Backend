use crate::models::{Game, Action, ActionType, PieceType, User};
pub struct GameBuilder {
    pub game : Game
}

impl GameBuilder {
    pub fn new(owner : &str) -> Self {
        Self { 
            game: Game {
                game_owner: owner.to_string(),
                game_score: 0,
                game_level: 0,
                game_lines: 0,
                game_actions: vec![Action{action_type: ActionType::Start, piece: PieceType::Cyan, timestamp: 0},
                    Action{action_type: ActionType::Fall, piece: PieceType::Cyan, timestamp: 1},
                    Action{action_type: ActionType::End, piece: PieceType::Void, timestamp: 2},
                ],
            }
         }
    }

    pub fn with_score(mut self, score : i32) -> Self {
        self.game.game_score = score;
        self
    }

    pub fn with_level(mut self, level : i32) -> Self {
        self.game.game_level = level;
        self
    }

    pub fn with_lines(mut self, lines : i32) -> Self {
        self.game.game_lines = lines;
        self
    }

    pub fn build(self) -> Game {
        self.game
    }
}

pub struct UserBuilder {
    pub user : User
}

impl UserBuilder {
    pub fn new(name : &str) -> Self {
        Self { user: User { name: name.to_string(), best_score: 0, highest_level: 0, number_of_games: 0 } }
    }

    pub fn with_score(mut self, score : i32) -> Self {
        self.user.best_score = score;
        self
    }

    pub fn with_level(mut self, level : i32) -> Self {
        self.user.highest_level = level;
        self
    }

    pub fn with_games(mut self, games : i32) -> Self {
        self.user.number_of_games = games;
        self
    }

    pub fn build(self) -> User {
        self.user
    }
}