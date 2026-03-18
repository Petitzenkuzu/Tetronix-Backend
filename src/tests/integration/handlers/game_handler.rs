#[cfg(test)]
mod tests {
    use actix_web::{test, web, App, http::StatusCode, cookie::Cookie};
    use crate::handlers::{get_stats, get_stats_by_owner, get_game};
    use crate::tests::integration::helper::HandlersFixture;
    use crate::models::{Game, GameStats};
    use crate::middleware::auth_middleware::Auth;
    use actix_web::dev::Service;

    #[actix_web::test]
    async fn test_get_stats_unauthorized() {
        let fixture = HandlersFixture::new().await;
        let app = test::init_service(App::new().app_data(web::Data::new(fixture.app_state)).service(get_stats)).await;
        let req = test::TestRequest::get().uri("/stats").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    async fn test_get_stats_success() {
        let fixture = HandlersFixture::new().await;
        fixture.with_test_user_and_game(|_username, jwt, game, app_state| async move {
            let app = test::init_service(App::new().app_data(web::Data::new(app_state)).wrap(Auth).service(get_stats)).await;
            let req = test::TestRequest::get().uri("/stats").cookie(Cookie::new("auth_token", jwt)).to_request();
            let resp : GameStats = test::call_and_read_body_json(&app, req).await;
            assert_eq!(resp.game_score, game.game_score);
            assert_eq!(resp.game_level, game.game_level);
        }).await;
    }

    #[actix_web::test]
    async fn test_get_stats_not_found() {
        let fixture = HandlersFixture::new().await;
        fixture.with_test_user(|_username, _jwt, app_state| async move {
            let other_jwt = app_state.auth_service.create_jwt("false_username".to_string()).expect("Failed to create JWT");
            let app = test::init_service(App::new().app_data(web::Data::new(app_state)).wrap(Auth).service(get_stats)).await;
            let req = test::TestRequest::get().uri("/stats").cookie(Cookie::new("auth_token", other_jwt)).to_request();
            let resp = test::call_service(&app, req).await;
            assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        }).await;
    }

    #[actix_web::test]
    async fn test_get_stats_by_owner_success() {
        let fixture = HandlersFixture::new().await;
        fixture.with_test_user_and_game(|username, jwt, game, app_state| async move {
            let app = test::init_service(App::new().app_data(web::Data::new(app_state)).wrap(Auth).service(get_stats_by_owner)).await;
            let req = test::TestRequest::get().uri(&format!("/stats/{}", username)).cookie(Cookie::new("auth_token", jwt)).to_request();
            let resp : GameStats = test::call_and_read_body_json(&app, req).await;
            assert_eq!(resp.game_score, game.game_score);
            assert_eq!(resp.game_level, game.game_level);
        }).await;
    }

    #[actix_web::test]
    async fn test_get_stats_by_owner_not_found() {
        let fixture = HandlersFixture::new().await;
        fixture.with_test_user_and_game(|_username, jwt, _game, app_state| async move {
            let app = test::init_service(App::new().app_data(web::Data::new(app_state)).wrap(Auth).service(get_stats_by_owner)).await;
            let req = test::TestRequest::get().uri(&format!("/stats/{}", "not_found")).cookie(Cookie::new("auth_token", jwt)).to_request();
            let resp = test::call_service(&app, req).await;
            assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        }).await;
    }

    #[actix_web::test]
    async fn test_get_stats_by_owner_unauthorized() {
        let fixture = HandlersFixture::new().await;
        let app = test::init_service(App::new().app_data(web::Data::new(fixture.app_state)).wrap(Auth).service(get_stats_by_owner)).await;
        let req = test::TestRequest::get().uri("/stats/not_found").to_request();
        let resp = app.call(req).await;
        assert!(resp.is_err());
        let error = resp.err().unwrap().error_response();
        assert_eq!(error.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    async fn test_get_game_success() {
        let fixture = HandlersFixture::new().await;
        fixture.with_test_user_and_game(|username, jwt, game, app_state| async move {
            let app = test::init_service(App::new().app_data(web::Data::new(app_state)).wrap(Auth).service(get_game)).await;
            let req = test::TestRequest::get().uri(&format!("/replay/{}", username)).cookie(Cookie::new("auth_token", jwt)).to_request();
            let resp : Game = test::call_and_read_body_json(&app, req).await;
            assert_eq!(resp, game);
        }).await;
    }

    #[actix_web::test]
    async fn test_get_game_not_found() {
        let fixture = HandlersFixture::new().await;
        fixture.with_test_user_and_game(|_username, jwt, _game, app_state| async move {
            let app = test::init_service(App::new().app_data(web::Data::new(app_state)).wrap(Auth).service(get_game)).await;
            let req = test::TestRequest::get().uri("/replay/not_found").cookie(Cookie::new("auth_token", jwt)).to_request();
            let resp = test::call_service(&app, req).await;
            assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        }).await;
    }

    #[actix_web::test]
    async fn test_get_game_unauthorized() {
        let fixture = HandlersFixture::new().await;
        let app = test::init_service(App::new().app_data(web::Data::new(fixture.app_state)).wrap(Auth).service(get_game)).await;
        let req = test::TestRequest::get().uri("/replay/not_found").to_request();
        let resp = app.call(req).await;
        assert!(resp.is_err());
        let error = resp.err().unwrap().error_response();
        assert_eq!(error.status(), StatusCode::UNAUTHORIZED);
    }
}