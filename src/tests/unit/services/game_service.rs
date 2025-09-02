#[cfg(test)]
mod tests {
    use crate::tests::unit::helpers::service_helpers::ServiceTestFixture;
    use crate::builder::game_builder::GameBuilder;
    use crate::assert_service_not_found;
    use crate::services::{GameServiceTrait, UserServiceTrait};
    #[tokio::test]
    async fn test_upsert_success() {
        let fixture = ServiceTestFixture::new().await;
        fixture.with_test_user_and_game(|username, game, _user_service, game_service| async move {
            let received_game = game_service.get_by_owner(&username).await.unwrap();
            assert_eq!(received_game, game);

            let new_game = GameBuilder::new(&username).with_score(100).with_level(10).with_lines(100).build();
            let _ = game_service.upsert(&new_game).await.unwrap();

            let received_game = game_service.get_by_owner(&username).await.unwrap();
            assert_eq!(received_game, new_game);
        }).await;
    }

    #[tokio::test]
    async fn test_get_by_owner_success() {
        let fixture = ServiceTestFixture::new().await;
        fixture.with_test_user_and_game(|username, game, _user_service, game_service| async move {
            let received_game = game_service.get_by_owner(&username).await.unwrap();
            assert_eq!(received_game, game);
        }).await;
    }

    #[tokio::test]
    async fn test_get_by_owner_not_found() {
        let fixture = ServiceTestFixture::new().await;
        let username = fixture.random_user_name();
        assert_service_not_found!(fixture.game_service.get_by_owner(&username).await);
    }

    #[tokio::test]
    async fn test_get_stats_success() {
        let fixture = ServiceTestFixture::new().await;
        fixture.with_test_user_and_game(|username, game, _user_service, game_service| async move {
            let stats = game_service.get_stats(&username).await.unwrap();
            assert_eq!(stats.game_score, game.game_score);
            assert_eq!(stats.game_level, game.game_level);
            assert_eq!(stats.game_lines, game.game_lines);
        }).await;
    }

    #[tokio::test]
    async fn test_get_stats_not_found() {
        let fixture = ServiceTestFixture::new().await;
        let username = fixture.random_user_name();
        assert_service_not_found!(fixture.game_service.get_stats(&username).await);
    }
}