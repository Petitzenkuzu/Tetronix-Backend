use std::env;

#[derive(Clone)]
pub struct AuthConfig {
    pub production: bool,
    pub github_client_id: String,
    pub github_client_secret: String,
    pub session_secret_key: String,
    pub github_test_url: Option<String>,
}

impl AuthConfig {
    pub fn from_env() -> Self {
        let production = env::var("PRODUCTION")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);

        let github_client_id = env::var("GITHUB_CLIENT_ID")
            .unwrap_or_else(|_| {
                eprintln!("WARNING: GITHUB_CLIENT_ID environment variable not set");
                String::new()
            });

        let github_client_secret = env::var("GITHUB_CLIENT_SECRET")
            .unwrap_or_else(|_| {
                eprintln!("WARNING: GITHUB_CLIENT_SECRET environment variable not set");
                String::new()
            });

        let session_secret_key = env::var("SESSION_SECRET_KEY")
            .unwrap_or_else(|_| {
                eprintln!("WARNING: SESSION_SECRET_KEY environment variable not set");
                String::new()
            });

        let github_test_url = env::var("GITHUB_TEST_URL").ok();

        Self {
            production,
            github_client_id,
            github_client_secret,
            session_secret_key,
            github_test_url,
        }
    }
}

#[derive(Clone)]
pub struct SessionConfig {
    pub secret_key: String
}

impl SessionConfig {
    pub fn from_env() -> Self {
        let secret_key = env::var("SESSION_SECRET_KEY")
            .unwrap_or_else(|_| {
                eprintln!("WARNING: SESSION_SECRET_KEY environment variable not set");
                String::new()
            });

        Self { secret_key }
    }
}

pub struct ServerConfig {
    pub port: u16,
    pub database_url: String
}

impl ServerConfig {
    pub fn from_env() -> Self {
        let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string()).parse::<u16>().expect("PORT must be a number");
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        Self { port, database_url }
    }
}

pub struct TokenBucketConfig {
    pub capacity: u8,
    pub refill_rate: u8,
}

impl TokenBucketConfig {
    pub fn new() -> Self {
        Self { capacity : 100 , refill_rate : 1 }
    }
}
