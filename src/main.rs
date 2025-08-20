use actix_web::{rt, web::{self, Data}, App ,HttpServer,middleware::Logger, HttpResponse};
mod models;
mod errors;
mod services;
mod repository;
mod middleware;
mod handlers;
mod game_logic;
mod config;
mod tests;
mod builder;
use handlers::{github_auth, get_user, get_leaderboard, logout, get_stats, get_game, get_stats_by_owner, start_game};
use services::{AuthService, SessionService, GameService, UserService};
use repository::{UserRepository, SessionRepository, GameRepository};
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;
use models::AppState;
use env_logger::Env;
use config::{AuthConfig, SessionConfig, ServerConfig};
use actix_web_prom::PrometheusMetricsBuilder;
use prometheus::Gauge;
use systemstat::{Platform, System};



#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let sys = System::new();
    dotenv().ok();
    env_logger::init_from_env(Env::default().default_filter_or("info"));
    
    let server_config = ServerConfig::from_env();

    let pool = PgPoolOptions::new()
    .max_connections(20)
    .connect(&server_config.database_url)
    .await
    .expect("Failed to connect to DB");

    let prometheus = PrometheusMetricsBuilder::new("api")
    .endpoint("/metrics")
    .build()
    .unwrap();

    let mem_gauge=Gauge::new("memory_usage", "Memory usage in bytes").unwrap();
    let cpu_gauge=Gauge::new("cpu_usage", "CPU usage in percentage").unwrap();

    prometheus.registry.register(Box::new(mem_gauge.clone())).unwrap();
    prometheus.registry.register(Box::new(cpu_gauge.clone())).unwrap();

    rt::spawn(async move {
        loop {
            match sys.cpu_load_aggregate() {
                Ok(cpu) => {
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    let cpu = cpu.done().unwrap();
                    cpu_gauge.set(f64::trunc(((cpu.user + cpu.system) * 100.0).into()));
                },
                Err(e) => {
                    eprintln!("Error getting CPU load: {}", e);
                }
            }
            match sys.memory() {
                Ok(mem) => {
                    let memory_used = mem.total.0 - mem.free.0;
                    let pourcentage_used = (memory_used as f64 / mem.total.0 as f64) * 100.0;
                    mem_gauge.set(f64::trunc(pourcentage_used));
                }
                Err(x) => println!("\nMemory: error: {}", x),
            }
        }
    });

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
        .wrap(prometheus.clone())
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