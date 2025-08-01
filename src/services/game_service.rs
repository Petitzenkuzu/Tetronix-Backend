use crate::repository::GameRepository;

pub struct GameService {
    game_repository: GameRepository,
}

impl GameService {
    pub fn new(game_repository: GameRepository) -> Self {
        Self { game_repository }
    }
}