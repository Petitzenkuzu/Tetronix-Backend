mod game_repository;
mod user_repository;

pub use game_repository::GameRepository;
pub use user_repository::UserRepository;

mod traits;
pub use traits::GameRepositoryTrait;
pub use traits::UserRepositoryTrait;
