use actix_web::{HttpRequest, get, HttpResponse};
use actix_web::{rt, web};
use crate::errors::AppError;
use actix_ws::{AggregatedMessage};
use futures_util::StreamExt as _;
use crate::models::{Session, GameResult, GameCloseReason};
use crate::models::{Action, ActionType, PieceType};
use crate::game_logic::GameEngine;
use crate::AppState;
use crate::builder::{game_builder::GameBuilder, user_builder::UserBuilder};

#[get("/start")]
async fn start_game(session: Session, state: web::Data<AppState>, req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, AppError> {
    let (res, mut ws_session, stream) = actix_ws::handle(&req, stream).map_err(|_| AppError::InternalServerError("Failed to start game".to_string()))?;

    let mut stream = stream
        .aggregate_continuations()
        .max_continuation_size(2_usize.pow(10));


    rt::spawn(async move {
        let user = state.user_service.get_by_name(&session.name).await;
        if user.is_err() {
            ws_session.close(Some(GameCloseReason::NoUserFound.to_close_reason())).await.unwrap();
             return;
        }
        let user = user.unwrap();
        let mut game_engine = GameEngine::new();
        let mut game_actions : Vec<Action> = Vec::new();

        while let Some(msg) = stream.next().await {
            match msg {
                Ok(AggregatedMessage::Binary(bin)) => {
                    if bin.len() != 10 {
                        // code 1008 should not happen because the request is authenticated
                        ws_session.close(Some(GameCloseReason::InvalidMessageLength.to_close_reason())).await.unwrap();
                        return;
                    }

                    // deserialize the game action
                    let game_action = Action {
                        action_type: ActionType::from_u8(bin[0]),
                        piece: PieceType::from_u8(bin[1]),
                        timestamp: i64::from_be_bytes(bin[2..bin.len()].try_into().unwrap()),
                    };

                    let result = game_engine.handle_action(&game_action);

                    game_actions.push(game_action);

                    // cheater detected
                    if let Some(GameResult::IllegalMove) = result {
                        println!("Illegal move");
                        ws_session.close(Some(GameCloseReason::IllegalMove.to_close_reason())).await.unwrap();
                        break;
                    }

                    // game ended, update the user and the game
                    if let Some(GameResult::Score(score,level,lines)) = result {

                        let updated_user = UserBuilder::new(&user.name)
                            .with_games(user.number_of_games + 1)
                            .with_score(user.best_score.max(score))
                            .with_level(user.highest_level.max(level))
                            .build();

                        let res = state.user_service.update(&updated_user).await;
                        if res.is_err() {
                            ws_session.close(Some(GameCloseReason::InternalError.to_close_reason())).await.unwrap();
                            break;
                        }

                        // if the score is higher than the user's best score, insert or update the replay game in the database
                        if score > user.best_score {
                            let game = GameBuilder::new(&user.name)
                                .with_score(score)
                                .with_level(level)
                                .with_lines(lines)
                                .with_actions(game_actions)
                                .build();

                            let res = state.game_service.upsert(&game).await;
                            if res.is_err() {
                                ws_session.close(Some(GameCloseReason::InternalError.to_close_reason())).await.unwrap();
                                break;
                            }
                        }
                        ws_session.close(Some(GameCloseReason::GameEnded.to_close_reason())).await.unwrap();
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




/*
while let Some(msg) = stream.next().await {
                match msg {
                    Ok(AggregatedMessage::Binary(bin)) => {
                        if bin.len() != 10 {
                            // code 1008 pas sensé arriver car requête authentifiée
                            ws_session.close(Some(GameCloseReason::InvalidMessageLength.to_close_reason())).await.unwrap();
                            return;
                        }
    
                        let game_action = Action {
                            action_type: ActionType::from_u8(bin[0]),
                            piece: PieceType::from_u8(bin[1]),
                            timestamp: i64::from_be_bytes(bin[2..bin.len()].try_into().unwrap()),
                        };
    
                        if game_action.action_type == ActionType::Ping {
                            *last_ping.write().await = Instant::now();
                            ws_session.binary(vec![0x00]).await.unwrap();
                            continue;
                        }
    
                        let result = game_engine.handle_action(&game_action);
    
                        game_actions.push(game_action);
    
                        // cheater detected
                        if let Some(GameResult::IllegalMove) = result {
                            ws_session.close(Some(GameCloseReason::IllegalMove.to_close_reason())).await.unwrap();
                            break;
                        }
    
                        // game ended
                        if let Some(GameResult::Score(score,level,lines)) = result {
    
                            let updated_user = UserBuilder::new(&user.name)
                                .with_games(user.number_of_games + 1)
                                .with_score(user.best_score.max(score))
                                .with_level(user.highest_level.max(level))
                                .build();
    
                            let res = state.user_service.update(&updated_user).await;
                            if res.is_err() {
                                ws_session.close(Some(GameCloseReason::InternalError.to_close_reason())).await.unwrap();
                                break;
                            }
    
                            if score > user.best_score && res.is_ok() {
                                let game = GameBuilder::new(&user.name)
                                    .with_score(score)
                                    .with_level(level)
                                    .with_lines(lines)
                                    .with_actions(game_actions)
                                    .build();
    
                                let res = state.game_service.upsert(&game).await;
                                if res.is_err() {
                                    ws_session.close(Some(GameCloseReason::InternalError.to_close_reason())).await.unwrap();
                                    break;
                                }
                            }
                            ws_session.close(Some(GameCloseReason::GameEnded.to_close_reason())).await.unwrap();
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

*/