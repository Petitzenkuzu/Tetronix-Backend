use crate::services::{AuthService, SessionService, GameService, UserService};

#[derive(Clone)]
pub struct AppState {
    pub auth_service: AuthService,
    pub session_service: SessionService,
    pub game_service: GameService,
    pub user_service: UserService,
}
