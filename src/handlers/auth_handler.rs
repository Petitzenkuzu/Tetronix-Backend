use actix_web::{web, Responder, HttpRequest, get, HttpResponse};
use crate::models::GithubCredentials;
use actix_web::web::Data;
use crate::models::AppState;
use crate::errors::AppError;

#[get("/github")]
pub async fn github_auth(state: Data<AppState>, query: web::Query<GithubCredentials>, _req: HttpRequest) -> Result<impl Responder, AppError> {

    let session_id = state.auth_service.authenticate_with_github(&query.code, &query.redirect_uri).await?;
    let cookie = state.auth_service.create_cookies(&session_id);
    
    Ok(HttpResponse::Ok()
        .cookie(cookie)
        .body("Authenticated with GitHub"))
}