use actix_web::{get, web::Data, HttpResponse, Responder, HttpRequest, post, web::Json};
use crate::AppState;
use sqlx;
use crate::models::Session;
use crate::models::User;
use crate::models::Game;
use actix_web::cookie::{Cookie, SameSite};

#[get("/user")]
pub async fn get_user(session : Option<Session>,state: Data<AppState>, _req: HttpRequest) -> impl Responder {
    if let None = session {
        return HttpResponse::Unauthorized().body("Invalid session");
    }
    else{
        let user = crate::data_base::get_user_from_name(&state.db, &session.unwrap().name).await;
        match user{
            Ok(user) => {
                return HttpResponse::Ok().json(user);
            }
            Err(e) => {
                if let sqlx::Error::RowNotFound = e {
                    return HttpResponse::Unauthorized().body("Invalid session");
                }
                return HttpResponse::InternalServerError().body("Internal server error");
            }
        }
    }
}

#[post("/user")]
pub async fn post_user(session : Option<Session>, user : Json<User> ,state: Data<AppState>, _req: HttpRequest) -> impl Responder {
    if let None = session {
        return HttpResponse::Unauthorized().body("Invalid session");
    }
    if user.name != session.unwrap().name {
        return HttpResponse::Unauthorized().body("Invalid session");
    }
    let result = crate::data_base::update_user(&state.db, &user).await;
    match result {
        Ok(_) => {
            return HttpResponse::Ok().json(user);
        }
        Err(e) => {
            if let sqlx::Error::RowNotFound = e {
                return HttpResponse::Unauthorized().body("User not Found");
            }
            return HttpResponse::InternalServerError().body("Internal server error");
        }
    }
}

#[get("/leaderboard")]
pub async fn get_leaderboard(session : Option<Session>,state: Data<AppState>, _req: HttpRequest) -> impl Responder {
    if let None = session {
        return HttpResponse::Unauthorized().body("Invalid session");
    }
    else{
        let leaderboard = crate::data_base::get_leaderboard(&state.db).await;
        match leaderboard {
            Ok(leaderboard) => {
                return HttpResponse::Ok().json(leaderboard);
            }
            Err(_e) => {
                return HttpResponse::InternalServerError().body("Internal server error");
            }
        }
    }
}

#[post("/logout")]
pub async fn logout(session : Option<Session>, state: Data<AppState>, _req: HttpRequest) -> impl Responder {
    if let None = session {
        return HttpResponse::Unauthorized().body("Invalid session");
    }
    let result = crate::data_base::delete_session(&state.db, &session.unwrap().session_id).await;
    match result {
        Ok(_) => {
            let mut cookie = Cookie::new("session_id", "");
            cookie.set_path("/");
            cookie.set_http_only(true);
            cookie.set_secure(false);
            cookie.set_same_site(SameSite::Lax);
            // expire directement
            cookie.set_max_age(None);
            let rep = HttpResponse::Ok()
            .cookie(cookie)
            .finish();
            return rep;
        }
        Err(_e) => {
            return HttpResponse::InternalServerError().body("Internal server error");
        }
    }
}

#[get("/game")]
pub async fn get_game(session : Option<Session>, state: Data<AppState>, _req: HttpRequest) -> impl Responder {
    if let None = session {
        return HttpResponse::Unauthorized().body("Invalid session");
    }
    else{
        let game = crate::data_base::get_game_from_owner(&state.db, &session.unwrap().name).await;
        match game {
            Ok(game) => {
                return HttpResponse::Ok().json(game);
            }
            Err(e) => {
                if let sqlx::Error::RowNotFound = e {
                    return HttpResponse::NotFound().body("No game found");
                }
                return HttpResponse::InternalServerError().body("Internal server error");
            }
        }
    }
}

#[post("/game")]
pub async fn upsert_game(session : Option<Session>, game : Json<Game>, state: Data<AppState>, _req: HttpRequest) -> impl Responder {
    if let None = session {
        return HttpResponse::Unauthorized().body("Invalid session");
    }
    else{
        let result = crate::data_base::upsert_game(&state.db, &game).await;
        match result {
            Ok(_) => {
                return HttpResponse::Ok().body("Game upserted");
            }
            Err(_e) => {
                return HttpResponse::InternalServerError().body("Internal server error");
            }
        }
    }
}