use actix_web::{web, Responder, HttpRequest, get, HttpResponse, post};
use crate::models::GithubCredentials;
use actix_web::web::Data;
use crate::models::{AppState, Session};
use crate::errors::AppError;


#[get("/github")]
pub async fn github_auth(state: Data<AppState>, query: web::Query<GithubCredentials>, _req: HttpRequest) -> Result<impl Responder, AppError> {

    let session_id = state.auth_service.authenticate_with_github(&query.code, &query.redirect_uri).await.map_err(|e| AppError::AuthenticationFailed(e.to_string()))?;
    let cookie = state.auth_service.create_cookies(&session_id);
    
    Ok(HttpResponse::Ok()
        .cookie(cookie)
        .body("Authenticated with GitHub"))
}

#[post("/logout")]
pub async fn logout(session: Session, state: Data<AppState>, _req: HttpRequest) -> impl Responder {
    if let Err(_e) = state.session_service.delete(&session.session_id).await {
        return HttpResponse::InternalServerError().body("Failed to logout");
    }
    
    let cookie = state.auth_service.logout_cookies();
    HttpResponse::Ok()
        .cookie(cookie)
        .body("Logged out")
}