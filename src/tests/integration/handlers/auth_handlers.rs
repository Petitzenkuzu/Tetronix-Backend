#[cfg(test)]
mod tests {
    use crate::handlers::{get_user, github_auth, logout};
    use crate::middleware::auth_middleware::Auth;
    use crate::models::User;
    use crate::services::UserServiceTrait;
    use crate::tests::integration::helper::HandlersFixture;
    use actix_web::dev::Service;
    use actix_web::{cookie::Cookie, http::StatusCode, test, web, App};

    #[actix_web::test]
    async fn test_github_auth_success() {
        let opts = mockito::ServerOpts {
            host: "127.0.0.1",
            port: 0,
            ..Default::default()
        };
        let mut server = mockito::Server::new_with_opts(opts);

        let _token_mock = server
            .mock("POST", "/login/oauth/access_token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"access_token": "test_access_token", "token_type": "Bearer"}"#)
            .expect(1)
            .create();

        let _user_mock = server.mock("GET", "/user")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"login": "test_user", "id": 1234567890, "name": "Test User", "avatar_url": "https://example.com/avatar.png"}"#)
            .expect(1)
            .create();

        let fixture = HandlersFixture::new(Some(server.url())).await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(fixture.app_state.clone()))
                .service(github_auth)
                .service(logout)
                .service(web::scope("").wrap(Auth).service(get_user)),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/github?code=test_code&redirect_uri=test_redirect_uri&code_verifier=test_code_verifier")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let cookie = resp.headers().get("Set-Cookie").unwrap();
        let cookie = Cookie::parse(cookie.to_str().unwrap()).unwrap();

        let req = test::TestRequest::get()
            .uri("/user")
            .cookie(cookie.clone())
            .to_request();

        let user: User = test::call_and_read_body_json(&app, req).await;
        assert_eq!(user.name, "test_user");
        assert_eq!(user.best_score, 0);
        assert_eq!(user.highest_level, 0);
        assert_eq!(user.number_of_games, 0);

        let req = test::TestRequest::post()
            .uri("/logout")
            .cookie(cookie.clone())
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let cookie = resp.headers().get("Set-Cookie").unwrap();
        let cookie = Cookie::parse(cookie.to_str().unwrap()).unwrap();

        let req = test::TestRequest::get()
            .uri("/user")
            .cookie(cookie)
            .to_request();
        let resp = app.call(req).await;
        assert!(resp.is_err());
        let error = resp.err().unwrap().error_response();
        assert_eq!(error.status(), StatusCode::UNAUTHORIZED);

        () = fixture
            .app_state
            .user_service
            .delete("test_user")
            .await
            .unwrap();
    }
}
