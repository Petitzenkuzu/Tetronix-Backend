use actix_web::{web, Responder, HttpRequest, get, HttpResponse};
use actix_web::web::Data;
use crate::models::{ConcreteAppState, Session};
use crate::errors::AppError;
use crate::services::UserServiceTrait;

#[get("/user")]
pub async fn get_user(session : Session, state: Data<ConcreteAppState>, _req: HttpRequest) -> Result<impl Responder, AppError> {
    let name = session.name;
    let user = state.user_service.get_by_name(&name).await?;
    Ok(HttpResponse::Ok().json(user))
}

#[get("/leaderboard")]
pub async fn get_leaderboard(_session : Session,state: Data<ConcreteAppState>, _req: HttpRequest) -> Result<impl Responder, AppError> {
    let leaderboard = state.user_service.get_top(3).await?;
    Ok(HttpResponse::Ok().json(leaderboard))
}