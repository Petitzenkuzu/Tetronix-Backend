use actix_web::{get, web::{self, Data}, HttpResponse, Responder, HttpRequest, cookie::{Cookie, SameSite}};
use reqwest::Client;
use std::env;
use dotenv::dotenv;
use uuid::Uuid;
use sqlx;
use crate::AppState;
use crate::models::{GithubAuth,GithubTokenResponse,GithubUser,GithubAuthMobile};
use crate::data_base::{create_user,get_user_from_name,create_session,get_session_from_name};

/*
    Return un access token en fonction du code github
    Return un error si les variables d'environnement ne sont pas définies
*/
async fn get_access_token(code: &str) -> Result<String, reqwest::Error> {
    dotenv().ok();
    let client_id = env::var("GITHUB_CLIENT_ID").unwrap_or_else(|_| {
        println!("ATTENTION: Variable d'environnement GITHUB_CLIENT_ID non définie!");
        "".to_string()
    });

    let client_secret = env::var("GITHUB_CLIENT_SECRET").unwrap_or_else(|_| {
        println!("ATTENTION: Variable d'environnement GITHUB_CLIENT_SECRET non définie!");
        "".to_string()
    });

    let redirect_uri = env::var("BACKEND_URL").unwrap_or_else(|_| {
        println!("ATTENTION: Variable d'environnement BACKEND_URL non définie!");
        "httpl://localhost:8080".to_string()
    });
    
    let client = Client::new();
    let response = client.post("https://github.com/login/oauth/access_token")
        .form(&[
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("code", code),
            ("redirect_uri", format!("{}/auth/github", redirect_uri).as_str())
        ])
        .header("Accept", "application/json")
        .send()
        .await?;

    let token_data = response.json::<GithubTokenResponse>().await.unwrap();
    Ok(token_data.access_token)
}

/*
    Return un user en fonction de l'access token
    Return un error si la requête ne passe pas
*/
async fn get_user_info(token: &str) -> Result<GithubUser, reqwest::Error> {
    let client = Client::new();
    let response = client.get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "Tetris by Amaël")
        .send()
        .await.expect("Failed to send request");

    let user = response.json::<GithubUser>().await.expect("Failed to parse response");
    Ok(user)
}

#[get("/github")]
pub async fn github_auth(state: Data<AppState>, query: web::Query<GithubAuth>, _req: HttpRequest) -> impl Responder {
    dotenv().ok();
    
    let code = &query.code;
    let access_token = get_access_token(code).await.unwrap();
    let github_user = get_user_info(&access_token).await.unwrap();
    let front_url = std::env::var("FRONT_URL").unwrap_or_else(|_| {
        println!("ATTENTION: Variable d'environnement FRONT_URL non définie!");
        "http://localhost:3000".to_string()
    });

    // Récupération du nom d'utilisateur GitHub
    let username = github_user.login;
    
    let user = get_user_from_name(&state.db,&username).await;
    match user {
        Ok(_user) => {
            //user existant alors on cherche une session existante
            let session = get_session_from_name(&state.db,&username).await;
            match session {
                Ok(session) => {
                    //session existante
                    let mut cookie = Cookie::new("session_id", session.session_id);
                    cookie.set_path("/");
                    cookie.set_http_only(true);
                    cookie.set_secure(false);
                    cookie.set_same_site(SameSite::Lax);

                    let rep = HttpResponse::Found()
                    .cookie(cookie)
                    .append_header(("Location", front_url))
                    .finish();
                    return rep;
                }
                Err(e) => {
                    if let sqlx::Error::RowNotFound = e {
                        //session non existante alors on crée une session
                        let session_id = Uuid::new_v4().to_string();
                        let session = create_session(&state.db,&username,&session_id).await;
                        match session{
                            //session créée
                            Ok(_) => {
                                let mut cookie = Cookie::new("session_id", session_id);
                                cookie.set_path("/");
                                cookie.set_http_only(true);
                                cookie.set_secure(false);
                                let rep = HttpResponse::Found()
                                .cookie(cookie)
                                .append_header(("Location", front_url))
                                .finish();
                                return rep;
                            }
                            //session non créée
                            Err(e) => {
                                let error_message = format!("Internal server error : {}", e);
                                return HttpResponse::InternalServerError().body(error_message);
                            }
                        }
                    }
                    else {
                        let error_message = format!("Internal server error : {}", e);
                        return HttpResponse::InternalServerError().body(error_message);
                    }
                }
            }
        }
        Err(e) => {
            //user non existant alors on crée un user
            if let sqlx::Error::RowNotFound = e {
                let user = create_user(&state.db,&username).await;
                match user {
                    //user créé alors on crée une session
                    Ok(_) => {
                        let session_id = Uuid::new_v4().to_string();
                        let session = create_session(&state.db,&username,&session_id).await;
                        match session{
                            //session créée alors on fait le cookie et on redirige vers le front
                            Ok(_) => {
                                let mut cookie = Cookie::new("session_id", session_id);
                                cookie.set_path("/");
                                cookie.set_http_only(true);
                                cookie.set_secure(false);
                                cookie.set_same_site(SameSite::Lax);

                                let rep = HttpResponse::Found()
                                .cookie(cookie)
                                .append_header(("Location", front_url))
                                .finish();
                                return rep;
                            }
                            //session non créée alors on renvoie une erreur interne
                            Err(e) => {
                                let error_message = format!("Internal server error : {}", e);
                                return HttpResponse::InternalServerError().body(error_message);
                            }
                        }
                    }
                    //user non créé alors on renvoie une erreur interne
                    Err(e) => {
                        let error_message = format!("Internal server error : {}", e);
                        return HttpResponse::InternalServerError().body(error_message);
                    }
                }

            }
            else {
                //user non existant et erreur interne
                let error_message = format!("Internal server error : {}", e);
                return HttpResponse::InternalServerError().body(error_message);
            }
        }
    }
}


/*
    Return un access token en fonction du code github
    Return un error si les variables d'environnement ne sont pas définies
*/
async fn get_access_token_mobile(code: &str, redirect_uri: &str) -> Result<String, reqwest::Error> {
    dotenv().ok();
    let client_id = env::var("GITHUB_CLIENT_ID_MOBILE").unwrap_or_else(|_| {
        println!("ATTENTION: Variable d'environnement GITHUB_CLIENT_ID_MOBILE non définie!");
        "".to_string()
    });

    let client_secret = env::var("GITHUB_CLIENT_SECRET_MOBILE").unwrap_or_else(|_| {
        println!("ATTENTION: Variable d'environnement GITHUB_CLIENT_SECRET_MOBILE non définie!");
        "".to_string()
    });
    
    let client = Client::new();
    let response = client.post("https://github.com/login/oauth/access_token")
        .form(&[
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("code", code),
            ("redirect_uri", format!("{}", redirect_uri).as_str())
        ])
        .header("Accept", "application/json")
        .send()
        .await?;
    match  response.json::<GithubTokenResponse>().await {
        Ok(token_data) => {
            Ok(token_data.access_token)
        }
        Err(e) => {
            println!("Error: {}", e);
            Err(e)
        }
    }
}

/*
    Return un user en fonction de l'access token
    Return un error si la requête ne passe pas
*/
async fn get_user_info_mobile(token: &str) -> Result<GithubUser, reqwest::Error> {
    let client = Client::new();
    let response = client.get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "Tetronix by Amaël")
        .send()
        .await.expect("Failed to send request");

    let user = response.json::<GithubUser>().await;
    match user {
        Ok(user) => {
            Ok(user)
        }
        Err(e) => {
            println!("Error: {}", e);
            Err(e)
        }
    }
}


#[get("/github_mobile")]
pub async fn github_auth_mobile(state: Data<AppState>, query: web::Query<GithubAuthMobile>, _req: HttpRequest) -> impl Responder {
    dotenv().ok();
    
    let code = &query.code;
    let access_token = get_access_token_mobile(code, &query.redirect_uri).await;
    if let Err(e) = access_token {
        return HttpResponse::InternalServerError().body(format!("Internal server error : {}", e));
    }
    let access_token = access_token.unwrap();
    let github_user = get_user_info_mobile(&access_token).await;
    if let Err(e) = github_user {
        return HttpResponse::InternalServerError().body(format!("Internal server error : {}", e));
    }
    let github_user = github_user.unwrap();
    // Récupération du nom d'utilisateur GitHub
    let username = github_user.login;
    
    let user = get_user_from_name(&state.db,&username).await;
    match user {
        Ok(user) => {
            //user existant alors on cherche une session existante
            let session = get_session_from_name(&state.db,&user.name).await;
            match session {
                Ok(session) => {
                    //session existante
                    let mut cookie = Cookie::new("session_id", &session.session_id);
                    cookie.set_path("/");
                    cookie.set_http_only(true);
                    cookie.set_secure(false);
                    cookie.set_same_site(SameSite::Lax);
                    cookie.set_max_age(actix_web::cookie::time::Duration::days(7));
                    let rep = HttpResponse::Ok()
                        .cookie(cookie)
                        .append_header(("session_id", session.session_id))
                        .body("connected on existing session");
                    return rep;
                }
                Err(e) => {
                    if let sqlx::Error::RowNotFound = e {
                        //session non existante alors on crée une session
                        let session_id = Uuid::new_v4().to_string();
                        let session = create_session(&state.db,&username,&session_id).await;
                        match session{
                            //session créée
                            Ok(_) => {
                                let mut cookie = Cookie::new("session_id", &session_id);
                                cookie.set_path("/");
                                cookie.set_http_only(true);
                                cookie.set_secure(false);
                                cookie.set_same_site(SameSite::Lax);
                                cookie.set_max_age(actix_web::cookie::time::Duration::days(7));
                                let rep = HttpResponse::Ok()
                                .cookie(cookie)
                                .append_header(("session_id", session_id))
                                .body("connected on new session");
                                return rep;
                            }
                            //session non créée
                            Err(e) => {
                                let error_message = format!("Internal server error : {}", e);
                                return HttpResponse::InternalServerError().body(error_message);
                            }
                        }
                    }
                    else {
                        let error_message = format!("Internal server error : {}", e);
                        return HttpResponse::InternalServerError().body(error_message);
                    }
                }
            }
        }
        Err(e) => {
            //user non existant alors on crée un user
            if let sqlx::Error::RowNotFound = e {
                let user = create_user(&state.db,&username).await;
                match user {
                    //user créé alors on crée une session
                    Ok(_) => {
                        let session_id = Uuid::new_v4().to_string();
                        let session = create_session(&state.db,&username,&session_id).await;
                        match session{
                            //session créée alors on fait le cookie et on redirige vers le front
                            Ok(_) => {
                                let mut cookie = Cookie::new("session_id", &session_id);
                                cookie.set_path("/");
                                cookie.set_http_only(true);
                                cookie.set_secure(false);
                                cookie.set_same_site(SameSite::Lax);
                                cookie.set_max_age(actix_web::cookie::time::Duration::days(7));
                                let rep = HttpResponse::Ok()
                                .cookie(cookie)
                                .append_header(("session_id", session_id))
                                .body("connected to a new session with a new user");
                                return rep;
                            }
                            //session non créée alors on renvoie une erreur interne
                            Err(e) => {
                                let error_message = format!("Internal server error : {}", e);
                                return HttpResponse::InternalServerError().body(error_message);
                            }
                        }
                    }
                    //user non créé alors on renvoie une erreur interne
                    Err(e) => {
                        let error_message = format!("Internal server error : {}", e);
                        return HttpResponse::InternalServerError().body(error_message);
                    }
                }

            }
            else {
                //user non existant et erreur interne
                let error_message = format!("Internal server error : {}", e);
                return HttpResponse::InternalServerError().body(error_message);
            }
        }
    }
}