use crate::repository::{UserRepository, SessionRepository};
use crate::errors::ServicesError;
use crate::models::*;
use crate::errors::RepositoryError;
use dotenv::dotenv;
use std::env;
use reqwest::Client;
use uuid;
use actix_web::cookie::{SameSite, Cookie};

pub struct AuthService {
    user_repository : UserRepository,
    session_repository : SessionRepository,
}


impl AuthService {
    pub fn new(user_repository: UserRepository, session_repository: SessionRepository) -> Self {
        Self { user_repository, session_repository }
    }

    async fn exchange_code_for_access_token(&self, code: &str, redirect_uri: &str) -> Result<String, ServicesError> {
        dotenv().ok();

        let client_id = env::var("GITHUB_CLIENT_ID").unwrap_or_else(|_| {
            println!("WARNING: you must define the GITHUB_CLIENT_ID environment variable");
            "".to_string()
        });

        let client_secret = env::var("GITHUB_CLIENT_SECRET").unwrap_or_else(|_| {
            println!("WARNING: you must define the GITHUB_CLIENT_SECRET environment variable");
            "".to_string()
        });
    
        let client = Client::new();
        let response = client.post("https://github.com/login/oauth/access_token")
            .form(&[
                ("client_id", client_id.as_str()),
                ("client_secret", client_secret.as_str()),
                ("code", code),
                ("redirect_uri", format!("{}", redirect_uri).as_str())
            ])
            .header("Accept", "application/json")
            .send()
            .await.map_err(|e| ServicesError::AuthenticationFailed{reason: format!("access token exchange failed : {}", e)})?;

        let token_data = response.json::<GithubTokenResponse>().await.map_err(|_| ServicesError::UnableToDeserialize{what: "GithubTokenResponse".to_string()})?;

        Ok(token_data.access_token)
    }

    async fn exchange_access_token_for_user_info(&self, access_token: &str) -> Result<GithubUser, ServicesError> {
        let client = Client::new();
        let response = client.get("https://api.github.com/user")
            .header("Authorization", format!("Bearer {}", access_token))
            .header("User-Agent", "Tetronix by Amaël")
            .send()
            .await.map_err(|e| ServicesError::AuthenticationFailed{reason: format!("user info exchange failed : {}", e)})?;
    
        let user = response.json::<GithubUser>().await.map_err(|_| ServicesError::UnableToDeserialize{what: "GithubUser".to_string()})?;
        Ok(user)
    }

    pub async fn authenticate_with_github(&self, code: &str, redirect_uri: &str) -> Result<String, ServicesError> {
        //Get GitHub user info
        let access_token = self.exchange_code_for_access_token(code, redirect_uri).await?;
        let github_user = self.exchange_access_token_for_user_info(&access_token).await?;
        
        //Check existing session
        if let Ok(session) = self.session_repository.get_session_by_name(&github_user.login).await {
            return Ok(session.session_id);
        }
        
        //Ensure user exists, it will create it if it doesn't exist
        self.ensure_user_exists(&github_user.login).await?;
        
        //Create new session
        let session_id = self.create_session_for_user(&github_user.login).await?;
        
        Ok(session_id)
    }
    
    async fn ensure_user_exists(&self, username: &str) -> Result<(), ServicesError> {
        match self.user_repository.get_user_by_name(username).await {
            //User exists
            Ok(_) => Ok(()),
            Err(RepositoryError::NotFound { .. }) => {
                //User doesn't exist, we create it
                self.user_repository.create_user(username).await
                    .map_err(|e| ServicesError::AuthenticationFailed {
                        reason: format!("Failed to create user: {}", e)
                    })
            },
            Err(e) => Err(ServicesError::AuthenticationFailed {
                reason: format!("Database error: {}", e)
            })
        }
    }
    
    async fn create_session_for_user(&self, username: &str) -> Result<String, ServicesError> {
        let session_id = uuid::Uuid::new_v4().to_string();
        
        self.session_repository.create_session(username, &session_id).await
            .map_err(|e| ServicesError::AuthenticationFailed {
                reason: format!("Failed to create session: {}", e)
            })?;
        
        Ok(session_id)
    }

    pub fn create_cookies(&self, session_id: &str) -> Cookie {
        dotenv().ok();
        let production = env::var("PRODUCTION").unwrap_or_else(|_| "false".to_string());
        let mut cookie = Cookie::new("session_id", session_id.to_string());
        cookie.set_path("/");
        cookie.set_http_only(true);
        cookie.set_secure(production == "true");
        cookie.set_same_site(SameSite::Lax);
        cookie.set_max_age(actix_web::cookie::time::Duration::days(7));
        cookie
    }
}