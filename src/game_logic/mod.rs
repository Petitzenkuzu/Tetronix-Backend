mod game_engine;
pub use game_engine::GameEngine;

mod impls;

mod models;
pub use models::Grid;

mod piece_rng;
pub use piece_rng::PieceRng;

mod state;
pub use state::State;

#[macro_export]
macro_rules! return_if_sender_closed {
    ($result:expr) => {
        if let Err(_) = $result {
            return;
        }
    };
}
