mod auth_handler;
pub use auth_handler::github_auth;
pub use auth_handler::logout;

mod user_handler;
pub use user_handler::get_leaderboard;
pub use user_handler::get_user;

mod game_handler;
pub use game_handler::get_game;
pub use game_handler::get_stats;
pub use game_handler::get_stats_by_owner;

mod game_websocket_handler;
pub use game_websocket_handler::start_game;
