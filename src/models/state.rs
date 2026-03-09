use crate::services::{AuthService, GameService, UserService};
use crate::repository::{UserRepository, GameRepository};
use crate::config::{AuthConfig};
use sqlx::{Pool, Postgres};

#[derive(Clone)]
pub struct AppState<A, G, U> 
where
    A: crate::services::AuthServiceTrait,
    G: crate::services::GameServiceTrait,
    U: crate::services::UserServiceTrait,
{
    pub auth_service: A,
    pub game_service: G,
    pub user_service: U,
}

pub type ConcreteAppState = AppState<AuthService<UserRepository>, GameService<GameRepository>, UserService<UserRepository>>;

impl ConcreteAppState {
    pub fn new(pool: Pool<Postgres>, auth_config: AuthConfig) -> ConcreteAppState {
        let user_repo = UserRepository::new(pool.clone());
        let game_repo = GameRepository::new(pool);

        Self {
            auth_service: AuthService::new(user_repo.clone(), auth_config),
            game_service: GameService::new(game_repo.clone()),
            user_service: UserService::new(user_repo.clone()),
        }
    }
}
