use crate::models::User;

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