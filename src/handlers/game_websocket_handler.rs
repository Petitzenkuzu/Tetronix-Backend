use actix_web::{HttpRequest, get, HttpResponse};
use actix_web::{rt, web};
use crate::errors::AppError;
use actix_ws::{AggregatedMessage};
use futures_util::StreamExt as _;
use crate::models::{Session, GameResult, GameCloseReason};
use crate::models::{Action, ActionType, PieceType};
use crate::game_logic::GameEngine;
use crate::ConcreteAppState;
use crate::builder::{game_builder::GameBuilder, user_builder::UserBuilder};
use crate::services::UserServiceTrait;
use crate::services::GameServiceTrait;
use crate::models::ClientAction;
use crate::models::ServerResponse;

#[get("/start")]
async fn start_game(session: Session, state: web::Data<ConcreteAppState>, req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, AppError> {
    let (res, mut ws_session, stream) = actix_ws::handle(&req, stream).map_err(|_| AppError::InternalServerError("Failed to start game".to_string()))?;

    let mut stream = stream
        .aggregate_continuations()
        .max_continuation_size(2_usize.pow(10));


    rt::spawn(async move {
        let user = match state.user_service.get_by_name(&session.name).await {
            Ok(user) => user,
            Err(e) => {
                ws_session.close(Some(GameCloseReason::NoUserFound.to_close_reason())).await.unwrap();
                return;
            }
        };

        let game_engine = GameEngine::new();
        let (sender, receiver) = tokio::sync::mpsc::channel(100);
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        game_engine.start_engine(receiver, tx);

        loop {
            tokio::select! {
                Some(msg) = stream.next() => {
                    match msg {
                        Ok(AggregatedMessage::Binary(bin)) => {
                            // action type (1 byte) + id (4 bytes)
                            if bin.len() != 5 {
                                ws_session.close(Some(GameCloseReason::InvalidMessageLength.to_close_reason())).await.unwrap();
                                return;
                            }
                            let action = match ClientAction::try_from(bin) {
                                Ok(action) => action,
                                Err(_e) => {
                                    continue;
                                }
                            };

                            if let Err(_e) = sender.send(action).await {
                                tracing::error!("Game engine channel closed unexpectedly");
                                ws_session.close(Some(GameCloseReason::InternalError.to_close_reason())).await.unwrap();
                                break;
                            }
                        }
                        Ok(AggregatedMessage::Close(_)) => {
                            break;
                        }
                        _ => {}
                    }
                }
                Some(msg) = rx.recv() => {
                    match msg {
                        ServerResponse::Start(state) => {
                            ws_session.text(state).await.unwrap();
                        }
                        ServerResponse::State(state) => {
                            ws_session.text(state).await.unwrap();
                        }
                        ServerResponse::MissingAction(id) => {
                            ws_session.text(id).await.unwrap();
                        }
                        ServerResponse::InternalServerError(e) => {
                            tracing::error!("Internal server error: {}", e);
                            ws_session.close(Some(GameCloseReason::InternalError.to_close_reason())).await.unwrap();
                            break;
                        }
                    }
                }
            }
        }
    });

    Ok(res)
}