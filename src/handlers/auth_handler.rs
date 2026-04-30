use crate::errors::AppError;
use crate::models::ConcreteAppState;
use crate::models::GithubCredentials;
use crate::services::AuthServiceTrait;
use actix_web::web::Data;
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};

#[get("/github")]
pub async fn github_auth(
    state: Data<ConcreteAppState>,
    query: web::Query<GithubCredentials>,
    _req: HttpRequest,
) -> Result<impl Responder, AppError> {
    let jwt = state
        .auth_service
        .authenticate_with_github(&query.code, &query.redirect_uri, &query.code_verifier)
        .await
        .map_err(|e| AppError::AuthenticationFailed(e.to_string()))?;
    let cookie = state.auth_service.create_cookies(jwt);

    Ok(HttpResponse::Ok()
        .cookie(cookie)
        .body("Authenticated with GitHub"))
}

#[post("/logout")]
pub async fn logout(state: Data<ConcreteAppState>, _req: HttpRequest) -> impl Responder {
    let cookie = state.auth_service.logout_cookies();
    HttpResponse::Ok().cookie(cookie).body("Logged out")
}
