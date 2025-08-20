#[cfg(test)]
mod tests {
    use actix_web::{test, web, App, http::StatusCode, cookie::Cookie};
    use crate::handlers::{get_user, get_leaderboard};
    use crate::tests::integration::helper::HandlersFixture;
    use crate::models::User;
    use crate::builder::user_builder::UserBuilder;
    #[actix_web::test]
    async fn test_get_user_unauthorized() {
        let fixture = HandlersFixture::new().await;
        let app = test::init_service(App::new().app_data(web::Data::new(fixture.app_state)).service(get_user)).await;
        let req = test::TestRequest::get().uri("/user").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    async fn test_get_user_success() {
        let fixture = HandlersFixture::new().await;
        fixture.with_test_user_and_session(|username, session_id, app_state| async move {
            let app = test::init_service(App::new().app_data(web::Data::new(app_state)).service(get_user)).await;
            let req = test::TestRequest::get().uri("/user").cookie(Cookie::new("session_id", session_id)).to_request();
            let resp : User = test::call_and_read_body_json(&app, req).await;
            assert_eq!(resp.name, username);
        }).await;
    }

    #[actix_web::test]
    async fn test_get_leaderboard_success() {
        let fixture = HandlersFixture::new().await;

        let username2 = fixture.random_user_name();
        let username3 = fixture.random_user_name();

        fixture.with_test_user_and_session(|username1, session_id, app_state| async move {

            let _ = app_state.user_service.create(&username2).await.expect("Failed to create test user");
            let _ = app_state.user_service.create(&username3).await.expect("Failed to create test user");

        let _ = app_state.user_service.update(&UserBuilder::new(&username1).with_score(10000).with_level(10).with_games(10).build()).await.expect("Failed to update test user score");

        let _ = app_state.user_service.update(&UserBuilder::new(&username2).with_score(20000).with_level(20).with_games(20).build()).await.expect("Failed to update test user score");

        let _ = app_state.user_service.update(&UserBuilder::new(&username3).with_score(30000).with_level(30).with_games(30).build()).await.expect("Failed to update test user score");

        let app = test::init_service(App::new().app_data(web::Data::new(app_state.clone())).service(get_leaderboard)).await;
        let req = test::TestRequest::get().uri("/leaderboard").cookie(Cookie::new("session_id", session_id)).to_request();
        let resp : Vec<User> = test::call_and_read_body_json(&app, req).await;

        assert_eq!(resp.len(), 3);
        assert_eq!(resp[0].name, username3);
        assert_eq!(resp[1].name, username2);
        assert_eq!(resp[2].name, username1);

            let _ = app_state.user_service.delete(&username2).await.expect("Failed to delete test user");
            let _ = app_state.user_service.delete(&username3).await.expect("Failed to delete test user");
        }).await;
    }

    #[actix_web::test]
    async fn test_get_leaderboard_single_user() {
        let fixture = HandlersFixture::new().await;
        fixture.with_test_user_and_session(|username, session_id, app_state| async move {
            let app = test::init_service(App::new().app_data(web::Data::new(app_state)).service(get_leaderboard)).await;
            let req = test::TestRequest::get().uri("/leaderboard").cookie(Cookie::new("session_id", session_id)).to_request();
            let resp : Vec<User> = test::call_and_read_body_json(&app, req).await;
            assert_eq!(resp.len(), 1);
            assert_eq!(resp[0].name, username);
        }).await;
    }

    #[actix_web::test]
    async fn test_get_leaderboard_unauthorized() {
        let fixture = HandlersFixture::new().await;
        let app = test::init_service(App::new().app_data(web::Data::new(fixture.app_state)).service(get_leaderboard)).await;
        let req = test::TestRequest::get().uri("/leaderboard").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }
}