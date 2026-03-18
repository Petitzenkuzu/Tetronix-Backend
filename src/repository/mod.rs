mod user_repository;
mod game_repository;

pub use user_repository::UserRepository;
pub use game_repository::GameRepository;

mod traits;
pub use traits::UserRepositoryTrait;
pub use traits::GameRepositoryTrait;