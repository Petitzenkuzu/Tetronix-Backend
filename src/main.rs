use actix_web::{web::{self, Data}, App ,HttpServer,middleware::Logger, HttpResponse};
mod models;
mod errors;
mod services;
mod repository;
mod middleware;
mod handlers;
mod game_logic;
mod config;
mod tests;
use handlers::{github_auth, get_user, get_leaderboard, logout, get_stats, get_game, get_stats_by_owner, start_game};
use services::{AuthService, SessionService, GameService, UserService};
use repository::{UserRepository, SessionRepository, GameRepository};
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;
use models::AppState;
use env_logger::Env;
use config::{AuthConfig, SessionConfig, ServerConfig};



#[actix_web::main]
async fn main() -> std::io::Result<()> {
    
    dotenv().ok();
    env_logger::init_from_env(Env::default().default_filter_or("info"));
    
    let server_config = ServerConfig::from_env();

    let pool = PgPoolOptions::new()
    .max_connections(20)
    .connect(&server_config.database_url)
    .await
    .expect("Failed to connect to DB");

    HttpServer::new(move|| {

        let auth_config = AuthConfig::from_env();
        let session_config = SessionConfig::from_env();

        let auth_service = AuthService::new(UserRepository::new(pool.clone()), SessionRepository::new(pool.clone()), auth_config);
        let session_service = SessionService::new(SessionRepository::new(pool.clone()), session_config);
        let game_service = GameService::new(GameRepository::new(pool.clone()));
        let user_service = UserService::new(UserRepository::new(pool.clone()));
        let state = AppState { auth_service, session_service, game_service, user_service };
        
        App::new()
        .wrap(Logger::default())
        .app_data(Data::new(state))
        .default_service(web::route().to(|| async {HttpResponse::Unauthorized().body("Unauthorized")}))
        .service(
            web::scope("/auth")
            .service(github_auth)
            .service(logout)
        )
        .service(get_user)
        .service(get_leaderboard)
        .service(
            web::scope("/game")
            .default_service(web::route().to(|| async {HttpResponse::Unauthorized().body("Unauthorized")}))
            .service(get_stats)
            .service(get_stats_by_owner)
            .service(get_game)
            .service(start_game)
        )
    }).workers(4)
    .bind(("0.0.0.0", server_config.port))?
    .run()
    .await
}