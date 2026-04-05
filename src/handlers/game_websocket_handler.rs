use crate::builder::user_builder::UserBuilder;
use crate::errors::AppError;
use crate::game_logic::GameEngine;
use crate::models::ClientAction;
use crate::models::ServerResponse;
use crate::models::{AuthenticatedUser, GameCloseReason};
use crate::services::GameServiceTrait;
use crate::services::UserServiceTrait;
use crate::ConcreteAppState;
use actix_web::{get, HttpRequest, HttpResponse};
use actix_web::{rt, web};
use actix_ws::AggregatedMessage;
use futures_util::StreamExt as _;

#[get("/start")]
async fn start_game(
    authenticated_user: AuthenticatedUser,
    state: web::Data<ConcreteAppState>,
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, AppError> {
    let (res, mut ws_session, stream) = actix_ws::handle(&req, stream)
        .map_err(|_| AppError::InternalServerError("Failed to start game".to_string()))?;

    let mut stream = stream
        .aggregate_continuations()
        .max_continuation_size(2_usize.pow(10));

    rt::spawn(async move {
        let user = match state
            .user_service
            .get_by_name(&authenticated_user.username)
            .await
        {
            Ok(user) => user,
            Err(_) => {
                ws_session
                    .close(Some(GameCloseReason::NoUserFound.to_close_reason()))
                    .await
                    .unwrap();
                return;
            }
        };

        let (sender, receiver) = tokio::sync::mpsc::channel(100);
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let game_engine = GameEngine::new(receiver, tx);
        game_engine.start_engine();

        loop {
            tokio::select! {
                Some(msg) = stream.next() => {
                    match msg {
                        Ok(AggregatedMessage::Binary(bin)) => {
                            // action type (1 byte) + id (4 bytes)
                            if bin.len() != 5 {
                                let _ = ws_session.close(Some(GameCloseReason::InvalidMessageLength.to_close_reason())).await;
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
                                let _ = ws_session.close(Some(GameCloseReason::InternalError.to_close_reason())).await;
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
                    if let ServerResponse::InternalServerError(e) = msg {
                        tracing::error!("Internal server error: {}", e);
                        let _ = ws_session.close(Some(GameCloseReason::InternalError.to_close_reason())).await;
                        break;
                    }
                    if let ServerResponse::Game(game_builder) = msg {
                        let game = game_builder.with_owner(&user.name).build();
                        let res = state.game_service.upsert(&game).await;
                        if let Err(e) = res {
                            tracing::error!("Error upserting game: {}", e);
                            let _ = ws_session.close(Some(GameCloseReason::InternalError.to_close_reason())).await;
                            break;
                        }
                        else {
                            let user = UserBuilder::new(&user.name)
                                        .with_score(std::cmp::max(user.best_score, game.game_score))
                                        .with_level(std::cmp::max(user.highest_level, game.game_level))
                                        .with_games(user.number_of_games.saturating_add(1))
                                        .build();
                            let _ = state.user_service.update(&user).await;
                            let _ = ws_session.close(Some(GameCloseReason::GameEnded.to_close_reason())).await;
                            break;
                        }
                    }
                    if ws_session.text(serde_json::to_string(&msg).unwrap()).await.is_err() {
                        let _ = ws_session.close(Some(GameCloseReason::InternalError.to_close_reason())).await;
                        break;
                    }
                }
            }
        }
    });

    Ok(res)
}
