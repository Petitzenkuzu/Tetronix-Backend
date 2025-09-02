#[cfg(test)]
mod tests {
    use crate::tests::unit::helpers::service_helpers::{ServiceTestFixture};
    use crate::{assert_service_invalid_input, assert_service_already_exists, assert_service_not_found, assert_service_unable_to_delete};
    use crate::models::User;
    use crate::services::{UserServiceTrait};
    #[tokio::test]
    async fn test_create_user_success() {
        let fixture = ServiceTestFixture::new().await;
        fixture.with_test_user(|username, user_service| async move {
            assert!(user_service.get_by_name(&username).await.is_ok());
        }).await;
    }

    #[tokio::test]
    async fn test_create_user_invalid_input() {
        let fixture = ServiceTestFixture::new().await;
        assert_service_invalid_input!(fixture.user_service.create("").await);
        assert_service_invalid_input!(fixture.user_service.create("a").await);
        assert_service_invalid_input!(fixture.user_service.create(&"a".repeat(51)).await);
    }

    #[tokio::test]
    async fn test_create_user_already_exists() {
        let fixture = ServiceTestFixture::new().await;
        fixture.with_test_user(|username, user_service| async move {
            assert_service_already_exists!(user_service.create(&username).await);
        }).await;
    }

    #[tokio::test]
    async fn test_get_user_by_name_success() {
        let fixture = ServiceTestFixture::new().await;
        fixture.with_test_user(|username, user_service| async move {
            let user = user_service.get_by_name(&username).await.unwrap();
            assert_eq!(user.name, username);
            assert_eq!(user.number_of_games, 0);
            assert_eq!(user.best_score, 0);
            assert_eq!(user.highest_level, 0);
        }).await;
    }

    #[tokio::test]
    async fn test_get_user_by_name_not_found() {
        let fixture = ServiceTestFixture::new().await;
        let username = fixture.random_user_name();
        assert_service_not_found!(fixture.user_service.get_by_name(&username).await);
    }

    #[tokio::test]
    async fn test_update_user_success() {
        let fixture = ServiceTestFixture::new().await;
        fixture.with_test_user(|username, user_service| async move {
            let mut user = user_service.get_by_name(&username).await.unwrap();
            user.number_of_games += 1;
            user.best_score += 100;
            user.highest_level += 10;
            assert!(user_service.update(&user).await.is_ok());
            let user = user_service.get_by_name(&username).await.unwrap();
            assert_eq!(user.number_of_games, 1);
            assert_eq!(user.best_score, 100);
            assert_eq!(user.highest_level, 10);
        }).await;
    }

    #[tokio::test]
    async fn test_update_user_not_found() {
        let fixture = ServiceTestFixture::new().await;
        let user = User { name: fixture.random_user_name(), number_of_games: 0, best_score: 0, highest_level: 0 };
        assert_service_not_found!(fixture.user_service.update(&user).await);
    }

    #[tokio::test]
    async fn test_update_user_invalid_input() {

        let fixture = ServiceTestFixture::new().await;
        let user = User { name: "".to_string(), number_of_games: 0, best_score: 0, highest_level: 0 };
        assert_service_invalid_input!(fixture.user_service.update(&user).await);
        let user = User { name: "a".repeat(51), number_of_games: 0, best_score: 0, highest_level: 0 };
        assert_service_invalid_input!(fixture.user_service.update(&user).await);
        let user = User { name: "a".to_string(), number_of_games: -1, best_score: 0, highest_level: 0 };
        assert_service_invalid_input!(fixture.user_service.update(&user).await);
        let user = User { name: "a".to_string(), number_of_games: 0, best_score: -1, highest_level: 0 };
        assert_service_invalid_input!(fixture.user_service.update(&user).await);
        let user = User { name: "a".to_string(), number_of_games: 0, best_score: 0, highest_level: -1 };
        assert_service_invalid_input!(fixture.user_service.update(&user).await);
    
    }

    #[tokio::test]
    async fn test_get_top_users_success() {
        let fixture = ServiceTestFixture::new().await;
        let username2 = fixture.random_user_name();
        let username3 = fixture.random_user_name();
        fixture.with_test_user(|username, user_service| async move {

            let _ = user_service.create(&username2).await.unwrap();
            let _ = user_service.create(&username3).await.unwrap();

            let user1 = User { name: username, number_of_games: 100, best_score: 10000, highest_level: 100 };
            let user2 = User { name: username2, number_of_games: 90, best_score: 900, highest_level: 90 };
            let user3 = User { name: username3, number_of_games: 80, best_score: 80, highest_level: 80 };

            let _ = user_service.update(&user1).await.unwrap();
            let _ = user_service.update(&user2).await.unwrap();
            let _ = user_service.update(&user3).await.unwrap();

            let users = user_service.get_top(3).await.unwrap();

            assert_eq!(users.len(), 3);
            assert_eq!(users[0], user1);
            assert_eq!(users[1], user2);
            assert_eq!(users[2], user3);

            assert!(user_service.delete(&user2.name).await.is_ok());
            assert!(user_service.delete(&user3.name).await.is_ok());
        }).await;
    }

    #[tokio::test]
    async fn test_get_top_users_invalid_input() {
        let fixture = ServiceTestFixture::new().await;
        assert_service_invalid_input!(fixture.user_service.get_top(-1).await);
        assert_service_invalid_input!(fixture.user_service.get_top(101).await);
    }

    #[tokio::test]
    async fn test_delete_user_success() {
        let fixture = ServiceTestFixture::new().await;
        fixture.with_test_user(|_username, _user_service| async move {
            // no need to create a user, it's already created and deleted in the fixture
        }).await;
    }

    #[tokio::test]
    async fn test_delete_user_not_found() {
        let fixture = ServiceTestFixture::new().await;
        let username = fixture.random_user_name();
        assert_service_unable_to_delete!(fixture.user_service.delete(&username).await);
    }
}