use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct GithubCredentials {
    pub code: String,
    pub redirect_uri: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GithubTokenResponse {
    pub access_token: String,
    pub token_type: String
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GithubUser {
    pub login: String,
    pub id: Option<u64>,
    pub name: Option<String>,
    pub avatar_url: Option<String>,

}