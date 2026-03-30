use crate::builder::user_builder::UserBuilder;
use crate::config::AuthConfig;
use crate::errors::RepositoryError;
use crate::errors::ServicesError;
use crate::models::*;
use crate::repository::UserRepositoryTrait;
use crate::services::AuthServiceTrait;
use actix_web::cookie::{Cookie, SameSite};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use reqwest::Client;

#[derive(Clone)]
pub struct AuthService<T: UserRepositoryTrait> {
    user_repository: T,
    config: AuthConfig,
}

impl<T: UserRepositoryTrait> AuthService<T> {
    pub fn new(user_repository: T, config: AuthConfig) -> Self {
        Self {
            user_repository,
            config,
        }
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
    async fn exchange_code_for_access_token(
        &self,
        code: &str,
        redirect_uri: &str,
    ) -> Result<String, ServicesError> {
        let client_id = &self.config.github_client_id;
        let client_secret = &self.config.github_client_secret;

        let client = Client::new();
        let mut url = String::from("https://github.com/login/oauth/access_token");
        // test url for unit tests
        if let Some(test_url) = &self.config.github_test_url {
            url = format!("{}/login/oauth/access_token", test_url);
        }

        let response = client
            .post(url)
            .form(&[
                ("client_id", client_id.as_str()),
                ("client_secret", client_secret.as_str()),
                ("code", code),
                ("redirect_uri", redirect_uri),
            ])
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| ServicesError::AuthenticationFailed {
                reason: format!("access token exchange failed : {}", e),
            })?;

        let token_data = response.json::<GithubTokenResponse>().await.map_err(|_| {
            ServicesError::UnableToDeserialize {
                what: "GithubTokenResponse".to_string(),
            }
        })?;

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
    async fn exchange_access_token_for_user_info(
        &self,
        access_token: &str,
    ) -> Result<GithubUser, ServicesError> {
        let client = Client::new();
        let mut url = String::from("https://api.github.com/user");
        // test url for unit tests
        if let Some(test_url) = &self.config.github_test_url {
            url = format!("{}/user", test_url);
        }

        let response = client
            .get(url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("User-Agent", "Tetronix by Amaël")
            .send()
            .await
            .map_err(|e| ServicesError::AuthenticationFailed {
                reason: format!("user info exchange failed : {}", e),
            })?;

        let user = response.json::<GithubUser>().await.map_err(|_| {
            ServicesError::UnableToDeserialize {
                what: "GithubUser".to_string(),
            }
        })?;
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
                self.user_repository
                    .create_user(username)
                    .await
                    .map_err(|e| ServicesError::AuthenticationFailed {
                        reason: format!("Failed to create user: {}", e),
                    })
            }
            Err(e) => Err(ServicesError::AuthenticationFailed {
                reason: format!("Database error: {}", e),
            }),
        }
    }

    pub fn create_jwt(&self, username: String) -> Result<String, ServicesError> {
        let expiration = Utc::now()
            .checked_add_signed(Duration::days(7))
            .ok_or(ServicesError::InternalServerError(
                "Something went wrong".to_string(),
            ))?
            .timestamp() as usize;

        let claims = Claims {
            username,
            exp: expiration,
        };

        let jwt = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.config.session_secret_key.as_ref()),
        )
        .map_err(|_| ServicesError::InternalServerError("Something went wrong".to_string()))?;
        Ok(jwt)
    }
}

impl<T: UserRepositoryTrait> AuthServiceTrait for AuthService<T> {
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
    fn logout_cookies(&self) -> Cookie<'_> {
        let mut cookie = Cookie::new("auth_token", "");
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
    /// let cookie = auth_service.create_cookies("jwt");
    /// println!("Cookie: {:?}", cookie);
    /// ```
    fn create_cookies(&self, jwt: String) -> Cookie<'_> {
        let mut cookie = Cookie::new("auth_token", jwt);
        cookie.set_path("/");
        cookie.set_http_only(true);
        cookie.set_secure(self.config.production);
        cookie.set_same_site(SameSite::Lax);
        cookie.set_max_age(None);
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
    async fn authenticate_with_github(
        &self,
        code: &str,
        redirect_uri: &str,
    ) -> Result<String, ServicesError> {
        //Get GitHub user info
        let access_token = self
            .exchange_code_for_access_token(code, redirect_uri)
            .await?;
        let github_user = self
            .exchange_access_token_for_user_info(&access_token)
            .await?;

        UserBuilder::new(&github_user.login).build().validate()?;

        //Ensure user exists, it will create it if it doesn't exist
        self.ensure_user_exists(&github_user.login).await?;

        //Create JWT
        let jwt = self.create_jwt(github_user.login)?;

        Ok(jwt)
    }

    /// Verify a JWT
    ///
    /// # Arguments
    ///
    /// * `jwt` - The JWT to verify
    ///
    /// # Returns
    ///
    /// * `Ok(username)` - If the JWT has been verified successfully
    /// * `Err(ServicesError::InvalidJWT)` - If the JWT is invalid
    ///
    /// # Examples
    ///
    /// ```
    /// let auth_service = AuthService::new(UserRepository::new(db_pool), SessionRepository::new(db_pool), config);
    /// match auth_service.verify_jwt("jwt").await {
    ///     Ok(username) => println!("JWT verified successfully for user: {}", username),
    ///     Err(e) => eprintln!("Error verifying JWT: {}", e),
    /// }
    /// ```
    fn verify_jwt(&self, jwt: &str) -> Result<String, ServicesError> {
        let claims = decode::<Claims>(
            jwt,
            &DecodingKey::from_secret(self.config.session_secret_key.as_ref()),
            &Validation::default(),
        )
        .map_err(|_| ServicesError::InvalidJWT {
            reason: "Invalid JWT format".to_string(),
        })?;
        if claims.claims.exp < Utc::now().timestamp() as usize {
            return Err(ServicesError::InvalidJWT {
                reason: "JWT expired".to_string(),
            });
        }
        Ok(claims.claims.username)
    }
}
