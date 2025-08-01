use actix_web::{web::{self, Data}, App ,HttpServer,middleware::Logger, HttpResponse};
use actix_cors::Cors;
mod auth;
mod services;
mod models;
mod data_base;
mod extractors;
mod repository;
mod handlers;
mod errors;
use handlers::{github_auth};
use services::{AuthService, SessionService, GameService};
use repository::{UserRepository, SessionRepository, GameRepository};
//use services::{get_user, get_leaderboard, post_user, logout, upsert_game, get_game, get_game_stats, get_game_stats_from_owner};
use dotenv::dotenv;
use std::env;
use sqlx::postgres::PgPoolOptions;
use models::AppState;
use env_logger::Env;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    
    dotenv().ok();
    let database_url = env::var("DB_URL").expect("DB_URL must be set");
    let front_url = env::var("FRONT_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
    env_logger::init_from_env(Env::default().default_filter_or("info"));
    
    let pool = PgPoolOptions::new()
    .max_connections(2)
    .connect(&database_url)
    .await
    .expect("Failed to connect to DB");
    
    HttpServer::new(move|| {
        // Configuration CORS - autoriser plusieurs origines
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .supports_credentials();

        let auth_service = AuthService::new(UserRepository::new(pool.clone()), SessionRepository::new(pool.clone()));
        let session_service = SessionService::new(SessionRepository::new(pool.clone()));
        let game_service = GameService::new(GameRepository::new(pool.clone()));
        
        let state = AppState { auth_service, session_service, game_service };
        
        App::new()
        .wrap(Logger::default())
        .wrap(cors)
        .app_data(Data::new(state))
        .service(
            web::scope("/auth")
            .service(github_auth)
        )
        /*.service(
            web::scope("/services")
            .default_service(web::route().to(|| async {HttpResponse::Unauthorized().body("Unauthorized") }))
            .service(
                web::scope("")
                .service(get_user)
                .service(get_leaderboard)
                .service(post_user)
                .service(logout)
                .service(upsert_game)
                .service(get_game)
                .service(get_game_stats)
                .service(get_game_stats_from_owner)
            )
        )*/
    }).workers(4)
    .bind("0.0.0.0:8080")?
    .run()
    .await
}