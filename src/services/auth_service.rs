use crate::repository::{SessionRepository};
use crate::errors::ServicesError;
use crate::models::*;
use crate::errors::RepositoryError;
use crate::config::AuthConfig;
use reqwest::Client;
use uuid;
use actix_web::cookie::{SameSite, Cookie};
use sha2::{Sha256};
use hmac::{Hmac, Mac};
use crate::repository::{UserRepositoryTrait, SessionRepositoryTrait};
use crate::services::AuthServiceTrait;
type HmacSha256 = Hmac<Sha256>;

#[derive(Clone)]
pub struct AuthService<T: UserRepositoryTrait, S: SessionRepositoryTrait> {
    user_repository : T,
    session_repository : S,
    config: AuthConfig,
}


impl<T: UserRepositoryTrait, S: SessionRepositoryTrait> AuthService<T, S> {
    pub fn new(user_repository: T, session_repository: S, config: AuthConfig) -> Self {
        Self { user_repository, session_repository, config }
    }

    /// Hash a session ID
    /// 
    /// # Arguments
    /// 
    /// * `session_id` - The original session ID to hash
    /// 
    /// # Returns
    /// 
    /// * `String` - The hashed session ID
    /// 
    fn hash_session_id(&self, session_id: &str) -> String {
        let mut mac = HmacSha256::new_from_slice(self.config.session_secret_key.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(session_id.as_bytes());
        let result : String = format!("{:x}", mac.finalize().into_bytes());
        result
    }

    /// Exchange a code for an access token
    /// 
    /// # Arguments
    /// 
    /// * `code` - The code to exchange for an access token
    /// * `redirect_uri` - The redirect URI to use for the exchange
    /// 
    /// # Returns
    /// 
    /// * `Ok(access_token)` - If the code has been exchanged successfully
    /// * `Err(ServicesError::AuthenticationFailed)` - If the code exchange failed
    /// * `Err(ServicesError::UnableToDeserialize)` - If the response could not be deserialized
    /// 
    /// # Examples
    /// 
    /// ```
    /// let auth_service = AuthService::new(UserRepository::new(db_pool), SessionRepository::new(db_pool), config);
    /// match auth_service.exchange_code_for_access_token("code", "redirect_uri").await {
    ///     Ok(access_token) => println!("Access token: {}", access_token),
    ///     Err(e) => eprintln!("Error exchanging code: {}", e),
    /// }
    /// ```
    async fn exchange_code_for_access_token(&self, code: &str, redirect_uri: &str) -> Result<String, ServicesError> {

        let client_id = &self.config.github_client_id;
        let client_secret = &self.config.github_client_secret;
    
        let client = Client::new();
        let mut url = String::from("https://github.com/login/oauth/access_token");
        // test url for unit tests
        if let Some(test_url) = &self.config.github_test_url {
            url = format!("{}/login/oauth/access_token", test_url);
        }
        
        let response = client.post(url)
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

    /// Exchange an access token for user info
    /// 
    /// # Arguments
    /// 
    /// * `access_token` - The access token to exchange for user info
    /// 
    /// # Returns
    /// 
    /// * `Ok(user_info)` - If the access token has been exchanged successfully
    /// * `Err(ServicesError::AuthenticationFailed)` - If the access token exchange failed
    /// * `Err(ServicesError::UnableToDeserialize)` - If the response could not be deserialized
    /// 
    /// # Examples
    /// 
    /// ```
    /// let auth_service = AuthService::new(UserRepository::new(db_pool), SessionRepository::new(db_pool), config);
    /// match auth_service.exchange_access_token_for_user_info("access_token").await {
    ///     Ok(user_info) => println!("User info: {:?}", user_info),
    ///     Err(e) => eprintln!("Error exchanging access token: {}", e),
    /// }
    /// ```
    async fn exchange_access_token_for_user_info(&self, access_token: &str) -> Result<GithubUser, ServicesError> {
        let client = Client::new();
        let mut url = String::from("https://api.github.com/user");
        // test url for unit tests
        if let Some(test_url) = &self.config.github_test_url {
            url = format!("{}/user", test_url);
        }

        let response = client.get(url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("User-Agent", "Tetronix by Amaël")
            .send()
            .await.map_err(|e| ServicesError::AuthenticationFailed{reason: format!("user info exchange failed : {}", e)})?;
    
        let user = response.json::<GithubUser>().await.map_err(|_| ServicesError::UnableToDeserialize{what: "GithubUser".to_string()})?;
        Ok(user)
    }

    
    /// Ensure a user exists
    /// 
    /// # Arguments
    /// 
    /// * `username` - The name of the user to ensure exists
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - If the user exists or has been created successfully
    /// * `Err(ServicesError::AuthenticationFailed)` - If the user creation failed or retrieval failed
    /// 
    /// # Examples
    /// 
    /// ```
    /// let auth_service = AuthService::new(UserRepository::new(db_pool), SessionRepository::new(db_pool), config);
    /// match auth_service.ensure_user_exists("john_doe").await {
    ///     Ok(()) => println!("User exists or has been created successfully"),
    ///     Err(e) => eprintln!("Error ensuring user exists: {}", e),
    /// }
    /// ```
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

    /// Delete zombie sessions for a user
    /// 
    /// # Arguments
    /// 
    /// * `username` - The name of the user to delete the zombie sessions for
    /// 
    /// # Examples
    /// 
    /// ```
    /// let auth_service = AuthService::new(UserRepository::new(db_pool), SessionRepository::new(db_pool), config);
    /// auth_service.delete_zombie_session("john_doe").await;
    /// ```
    async fn delete_zombie_session(&self, username: &str) {
        let _ = self.session_repository.delete_session_by_name(&username).await;
    }
    
    /// Create a new session for a user
    /// 
    /// # Arguments
    /// 
    /// * `username` - The name of the user to create a session for
    /// 
    /// # Returns
    /// 
    /// * `Ok(session_id)` - If the session has been created successfully
    /// * `Err(ServicesError::AuthenticationFailed)` - If the session creation failed
    /// 
    /// # Examples
    /// 
    /// ```
    /// let auth_service = AuthService::new(UserRepository::new(db_pool), SessionRepository::new(db_pool), config);
    /// match auth_service.create_session_for_user("john_doe").await {
    ///     Ok(session_id) => println!("Session created successfully with id: {}", session_id),
    ///     Err(e) => eprintln!("Error creating session: {}", e),
    /// }
    /// ```
    async fn create_session_for_user(&self, username: &str) -> Result<String, ServicesError> {
        let session_id = uuid::Uuid::new_v4().to_string();
        let session_hash = self.hash_session_id(&session_id);
        self.session_repository.create_session(username, &session_hash).await
            .map_err(|e| ServicesError::AuthenticationFailed {
                reason: format!("Failed to create session: {}", e)
            })?;
        
        Ok(session_id)
    }


}

impl<T: UserRepositoryTrait, S: SessionRepositoryTrait> AuthServiceTrait for AuthService<T, S> {

    /// Create logout cookies
    /// 
    /// # Arguments
    /// 
    /// # Returns
    /// 
    /// * `cookie` - the logout cookies
    /// 
    /// # Examples
    /// 
    /// ```
    /// let auth_service = AuthService::new(UserRepository::new(db_pool), SessionRepository::new(db_pool), config);
    /// let cookie = auth_service.logout_cookies();
    /// println!("Cookie: {:?}", cookie);
    /// ```
    fn logout_cookies(&self) -> Cookie {
        let mut cookie = Cookie::new("session_id", "");
        cookie.set_path("/");
        cookie.set_http_only(true);
        cookie.set_secure(self.config.production);
        cookie.set_same_site(SameSite::Lax);
        cookie.set_max_age(None);
        cookie
    }

    /// Create authentication cookies
    /// 
    /// # Arguments
    /// 
    /// * `session_id` - The session id to create cookies for
    /// 
    /// # Returns
    /// 
    /// * `Ok(cookie)` - If the cookies have been created successfully
    /// 
    /// # Examples
    /// 
    /// ```
    /// let auth_service = AuthService::new(UserRepository::new(db_pool), SessionRepository::new(db_pool), config);
    /// let cookie = auth_service.create_cookies("session_id");
    /// println!("Cookie: {:?}", cookie);
    /// ```
    fn create_cookies(&self, session_id: &str) -> Cookie {
        let mut cookie = Cookie::new("session_id", session_id.to_string());
        cookie.set_path("/");
        cookie.set_http_only(true);
        cookie.set_secure(self.config.production);
        cookie.set_same_site(SameSite::Lax);
        cookie.set_max_age(actix_web::cookie::time::Duration::days(7));
        cookie
    }

    /// Authenticate with GitHub
    /// 
    /// # Arguments
    /// 
    /// * `code` - The code to exchange for an access token
    /// * `redirect_uri` - The redirect URI to use for the exchange
    /// 
    /// # Returns
    /// 
    /// * `Ok(session_id)` - If the user has been authenticated successfully
    /// * `Err(ServicesError::AuthenticationFailed)` - If the authentication failed
    /// * `Err(ServicesError::UnableToDeserialize)` - If the response could not be deserialized
    /// 
    /// # Examples
    /// 
    /// ```
    /// let service = AuthService::new(UserRepository::new(db_pool), SessionRepository::new(db_pool), config);
    /// match service.authenticate_with_github("code", "redirect_uri").await {
    ///     Ok(session_id) => println!("User authenticated successfully with session id: {}", session_id),
    ///     Err(e) => eprintln!("Error authenticating with GitHub: {}", e),
    /// }
    /// ```
    async fn authenticate_with_github(&self, code: &str, redirect_uri: &str) -> Result<String, ServicesError> {
        //Get GitHub user info
        let access_token = self.exchange_code_for_access_token(code, redirect_uri).await?;
        let github_user = self.exchange_access_token_for_user_info(&access_token).await?;
        
        //Ensure user exists, it will create it if it doesn't exist
        self.ensure_user_exists(&github_user.login).await?;

        //Ensure there is no zombie session for this user
        self.delete_zombie_session(&github_user.login).await;

        //Create new session
        let session_id = self.create_session_for_user(&github_user.login).await?;
        
        Ok(session_id)
    }

}