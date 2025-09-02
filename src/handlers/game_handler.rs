use actix_web::{web::Path, Responder, HttpRequest, get, HttpResponse};
use crate::models::{ConcreteAppState, Session};
use crate::errors::AppError;
use actix_web::web::Data;
use crate::services::GameServiceTrait;

#[get("/stats")]
pub async fn get_stats(session: Session, state: Data<ConcreteAppState>, _req: HttpRequest) -> Result<impl Responder, AppError> {
    let stats = state.game_service.get_stats(&session.name).await?;
    Ok(HttpResponse::Ok().json(stats))
}

#[get("/stats/{game_owner}")]
pub async fn get_stats_by_owner(_session: Session, game_owner : Path<String>, state: Data<ConcreteAppState>, _req: HttpRequest) -> Result<impl Responder, AppError> {
    let stats = state.game_service.get_stats(&game_owner).await?;
    Ok(HttpResponse::Ok().json(stats))
}

#[get("/replay/{game_owner}")]
pub async fn get_game(_session: Session, game_owner : Path<String>, state: Data<ConcreteAppState>, _req: HttpRequest) -> Result<impl Responder, AppError> {
    let game = state.game_service.get_by_owner(&game_owner).await?;
    Ok(HttpResponse::Ok().json(game))
}

