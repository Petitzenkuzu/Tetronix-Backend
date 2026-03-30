use crate::errors::AppError;
use crate::models::AuthenticatedUser;
use crate::models::ConcreteAppState;
use crate::services::UserServiceTrait;
use actix_web::web::Data;
use actix_web::{get, HttpRequest, HttpResponse, Responder};

#[get("/user")]
pub async fn get_user(
    authenticated_user: AuthenticatedUser,
    state: Data<ConcreteAppState>,
    _req: HttpRequest,
) -> Result<impl Responder, AppError> {
    let user = state
        .user_service
        .get_by_name(&authenticated_user.username)
        .await?;
    Ok(HttpResponse::Ok().json(user))
}

#[get("/leaderboard")]
pub async fn get_leaderboard(
    state: Data<ConcreteAppState>,
    _req: HttpRequest,
) -> Result<impl Responder, AppError> {
    let leaderboard = state.user_service.get_top(3).await?;
    Ok(HttpResponse::Ok().json(leaderboard))
}
