#[cfg(test)]
mod tests {
    use crate::services::{AuthServiceTrait, UserServiceTrait};
    use crate::tests::unit::helpers::service_helpers::ServiceTestFixture;
    #[tokio::test]
    async fn test_login_success() {
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

        let fixture = ServiceTestFixture::new(Some(server.url())).await;

        let jwt_token = fixture
            .auth_service
            .authenticate_with_github("test_code", "redirect_uri", "test_code_verifier")
            .await;
        assert!(jwt_token.is_ok());
        let jwt_token = jwt_token.unwrap();
        let user = fixture.user_service.get_by_name("test_user").await.unwrap();
        assert_eq!(user.name, "test_user");
        let verified_user = fixture
            .auth_service
            .verify_jwt(&jwt_token)
            .expect("Failed to verify JWT, should be a valid JWT");
        assert_eq!(verified_user, "test_user");

        assert!(fixture.user_service.delete("test_user").await.is_ok());
    }
}
