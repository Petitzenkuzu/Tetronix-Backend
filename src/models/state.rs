use crate::services::{AuthService, SessionService, GameService, UserService};
use crate::repository::{UserRepository, SessionRepository, GameRepository};
use crate::config::{AuthConfig, SessionConfig};
use sqlx::{Pool, Postgres};

#[derive(Clone)]
pub struct AppState<A, S, G, U> 
where
    A: crate::services::AuthServiceTrait,
    S: crate::services::SessionServiceTrait,
    G: crate::services::GameServiceTrait,
    U: crate::services::UserServiceTrait,
{
    pub auth_service: A,
    pub session_service: S,
    pub game_service: G,
    pub user_service: U,
}

pub type ConcreteAppState = AppState<AuthService<UserRepository, SessionRepository>, SessionService<SessionRepository>, GameService<GameRepository>, UserService<UserRepository>>;

impl ConcreteAppState {
    pub fn new(pool: Pool<Postgres>, auth_config: AuthConfig, session_config: SessionConfig) -> ConcreteAppState {
        let user_repo = UserRepository::new(pool.clone());
        let session_repo = SessionRepository::new(pool.clone());
        let game_repo = GameRepository::new(pool);

        Self {
            auth_service: AuthService::new(user_repo.clone(), session_repo.clone(), auth_config),
            session_service: SessionService::new(session_repo.clone(), session_config),
            game_service: GameService::new(game_repo.clone()),
            user_service: UserService::new(user_repo.clone()),
        }
    }
}
