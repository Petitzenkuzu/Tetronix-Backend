use actix_web::{HttpRequest, get, HttpResponse};
use actix_web::{rt, web};
use crate::errors::AppError;
use actix_ws::{AggregatedMessage, CloseCode, CloseReason};
use futures_util::StreamExt as _;
use crate::models::{Session, User, Game};
use crate::models::{Action, ActionType, PieceType};
use crate::game_logic::GameEngine;
use crate::AppState;
use std::time::Instant;

#[get("/start")]
async fn start_game(session: Session, state: web::Data<AppState>, req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, AppError> {
    let (res, mut ws_session, stream) = actix_ws::handle(&req, stream).map_err(|_| AppError::InternalServerError("Failed to start game".to_string()))?;

    let mut stream = stream
        .aggregate_continuations()
        .max_continuation_size(2_usize.pow(10));

    rt::spawn(async move {
        let user = state.user_service.get_by_name(&session.name).await;
        if user.is_err() {
            let close_reason = CloseReason {
                code: CloseCode::Policy,
                description: Some("User not found".to_string()),
            };
            ws_session.close(Some(close_reason)).await.unwrap();
            return;
        }
        let user = user.unwrap();
        let mut game_engine = GameEngine::new();
        let mut game_actions : Vec<Action> = Vec::new();
        let mut last_ping_time = Instant::now();
        const TIMEOUT_SECONDS: u64 = 20000;
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(AggregatedMessage::Binary(bin)) => {
                    if bin.len() != 10 {
                        // code 1008 pas sensé arriver car requête authentifiée
                        let close_reason = CloseReason {
                            code: CloseCode::Invalid,
                            description: Some("Invalid message length".to_string()),
                        };
                        ws_session.close(Some(close_reason)).await.unwrap();
                        return;
                    }

                    let game_action = Action {
                        action_type: ActionType::from_u8(bin[0]),
                        piece: PieceType::from_u8(bin[1]),
                        timestamp: i64::from_be_bytes(bin[2..bin.len()].try_into().unwrap()),
                    };

                    if game_action.action_type == ActionType::Ping {
                        last_ping_time = Instant::now();
                        println!("Ping received");
                        let ack_response = vec![0x00];
                        ws_session.binary(ack_response).await.unwrap();
                        continue;
                    }

                    let result = game_engine.handle_action(&game_action);

                    game_actions.push(game_action);

                    if let Some((score,level,lines)) = result {

                        // code 1000 pour une fin de partie normale
                        let mut close_reason = CloseReason {
                            code: CloseCode::Normal,
                            description: Some("Game ended".to_string()),
                        };

                        let updated_user = User {
                            name: user.name.clone(),
                            number_of_games: user.number_of_games + 1,
                            best_score: user.best_score.max(score),
                            highest_level: user.highest_level.max(level),
                        };

                        let res = state.user_service.update(&updated_user).await;
                        // code 1011 pour une erreur de mise à jour de l'utilisateur
                        if res.is_err() {
                            close_reason.code = CloseCode::Error;
                            close_reason.description = Some("Error updating user".to_string());
                        }

                        if score > user.best_score && res.is_ok() {
                            let game = Game {
                                game_owner: user.name.clone(),
                                game_score: score,
                                game_level: level,
                                game_lines: lines,
                                game_actions: game_actions,
                            };

                            // code 1011 pour une erreur de mise à jour de la partie dans la base de données
                            let res = state.game_service.upsert(&game).await;
                            if res.is_err() {
                                close_reason.code = CloseCode::Error;
                                close_reason.description = Some("Error upserting game".to_string());
                            }
                        }
                        ws_session.close(Some(close_reason)).await.unwrap();
                        break;
                    }
                    
                    let ack_response = vec![0x00];
                    ws_session.binary(ack_response).await.unwrap();
                },
                Ok(AggregatedMessage::Close(_)) => {
                    break;
                }
                _ => {}
            }
        }
    });

    Ok(res)
}




