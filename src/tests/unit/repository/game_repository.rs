#[cfg(test)]
mod tests {
    use crate::tests::unit::helpers::repository_helpers::RepositoryTestFixture;
    use crate::builder::game_builder::GameBuilder;
    use crate::models::{GameStats};
    use crate::{assert_repository_not_found};
    use crate::repository::GameRepositoryTrait;

    #[tokio::test]
    async fn test_upsert_get_game_success() {
        let fixture = RepositoryTestFixture::new().await;
        fixture.with_test_user_and_game(|username, _user_repo, game_repo| async move {
            // with_test_user_and_game already creates a game
            let game = game_repo.get_game_by_owner(&username).await.unwrap();
            assert_eq!(game, GameBuilder::new(&username).build());

            let game = GameBuilder::new(&username).with_score(100).with_level(10).with_lines(10).build();
            assert!(game_repo.upsert_game(&game).await.is_ok());

            let updated_game = game_repo.get_game_by_owner(&username).await.unwrap();
            assert_eq!(game, updated_game);
        }).await;
    }

    #[tokio::test]
    async fn test_get_game_by_owner_not_found() {
        let fixture = RepositoryTestFixture::new().await;
        assert_repository_not_found!(fixture.game_repo.get_game_by_owner(&fixture.random_user_name()).await);
    }

    #[tokio::test]
    async fn test_get_game_stats_by_owner_success() {
        let fixture = RepositoryTestFixture::new().await;
        fixture.with_test_user_and_game(|username, _user_repo, game_repo| async move {
            let game_stats = game_repo.get_game_stats_by_owner(&username).await.unwrap();
            assert_eq!(game_stats, GameStats{game_score: 0, game_level: 0, game_lines: 0});
        }).await;
    }

    #[tokio::test]
    async fn test_get_game_stats_by_owner_not_found() {
        let fixture = RepositoryTestFixture::new().await;
        assert_repository_not_found!(fixture.game_repo.get_game_stats_by_owner(&fixture.random_user_name()).await);
    }
}