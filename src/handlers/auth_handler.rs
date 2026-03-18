use actix_web::{web, Responder, HttpRequest, get, HttpResponse, post};
use crate::models::GithubCredentials;
use actix_web::web::Data;
use crate::models::{ConcreteAppState};
use crate::errors::AppError;
use crate::services::AuthServiceTrait;


#[get("/github")]
pub async fn github_auth(state: Data<ConcreteAppState>, query: web::Query<GithubCredentials>, _req: HttpRequest) -> Result<impl Responder, AppError> {

    let jwt = state.auth_service.authenticate_with_github(&query.code, &query.redirect_uri).await.map_err(|e| AppError::AuthenticationFailed(e.to_string()))?;
    let cookie = state.auth_service.create_cookies(jwt);
    
    Ok(HttpResponse::Ok()
        .cookie(cookie)
        .body("Authenticated with GitHub"))
}

#[post("/logout")]
pub async fn logout(state: Data<ConcreteAppState>, _req: HttpRequest) -> impl Responder {
    let cookie = state.auth_service.logout_cookies();
    HttpResponse::Ok()
        .cookie(cookie)
        .body("Logged out")
}