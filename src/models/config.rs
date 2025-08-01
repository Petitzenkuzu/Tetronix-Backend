use crate::services::{AuthService, SessionService, GameService};

pub struct AppState {
    pub auth_service: AuthService,
    pub session_service: SessionService,
    pub game_service: GameService,
}
