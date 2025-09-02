

#[cfg(test)]
mod tests {
    use crate::tests::unit::helpers::repository_helpers::RepositoryTestFixture;
    use crate::models::User;
    use crate::{assert_repository_not_found, assert_repository_already_exists, assert_repository_invalid_input};
    use crate::repository::{UserRepositoryTrait, UserRepository};
    #[tokio::test]
    async fn test_create_user_success() {
        let fixture = RepositoryTestFixture::new().await;

        fixture.with_test_user(|username : String, user_repo: UserRepository| async move {
            let user = user_repo.get_user_by_name(&username).await;
            assert!(user.is_ok());
        }).await;
    }

    #[tokio::test]
    async fn test_create_user_duplicate_fails() {
        let fixture = RepositoryTestFixture::new().await;
        
        fixture.with_test_user(|username : String, user_repo: UserRepository| async move {
            let result = user_repo.create_user(&username).await;
            assert_repository_already_exists!(result);
        }).await;
    }

    #[tokio::test]
    async fn test_create_user_invalid_input() {
        let fixture = RepositoryTestFixture::new().await;

        let result = fixture.user_repo.create_user("").await;
        assert_repository_invalid_input!(result);
    }

    #[tokio::test]
    async fn test_get_user_by_name_success() {
        let fixture = RepositoryTestFixture::new().await;

        fixture.with_test_user(|username : String, user_repo: UserRepository| async move {
            let user = user_repo.get_user_by_name(&username).await;
            assert!(user.is_ok());
            let user = user.unwrap();
            assert_eq!(user.name, username);
            assert_eq!(user.number_of_games, 0);
            assert_eq!(user.best_score, 0);
            assert_eq!(user.highest_level, 0);
        }).await;
    }

    #[tokio::test]
    async fn test_get_user_by_name_not_found() {
        let fixture = RepositoryTestFixture::new().await;
        let username = fixture.random_user_name();
        let user = fixture.user_repo.get_user_by_name(&username).await;
        assert_repository_not_found!(user);
    }

    #[tokio::test]
    async fn test_update_user_success() {
        let fixture = RepositoryTestFixture::new().await;

        fixture.with_test_user(|username : String, user_repo: UserRepository| async move {
            let user = user_repo.get_user_by_name(&username).await.unwrap();
            
            assert_eq!(user.number_of_games, 0);
            assert_eq!(user.best_score, 0);
            assert_eq!(user.highest_level, 0);
            assert_eq!(user.name, username);

            let updated_user = User {
                name: username.clone(),
                number_of_games: 5,
                best_score: 1,
                highest_level: 3,
            };
            
            assert!(user_repo.update_user(&updated_user).await.is_ok());
            let user = user_repo.get_user_by_name(&username).await.unwrap();
            assert_eq!(user.number_of_games, 5);
            assert_eq!(user.best_score, 1);
            assert_eq!(user.highest_level, 3);
            assert_eq!(user.name, username);

        }).await;
    }

    #[tokio::test]
    async fn test_update_user_not_found() {
        let fixture = RepositoryTestFixture::new().await;
        let user = User {
            name: fixture.random_user_name(),
            number_of_games: 0,
            best_score: 0,
            highest_level: 0,
        };
        assert_repository_not_found!(fixture.user_repo.update_user(&user).await);
    }

    #[tokio::test]
    async fn test_get_top_users() {
        let fixture = RepositoryTestFixture::new().await;
        let username2 = fixture.random_user_name();
        let username3 = fixture.random_user_name();
        fixture.with_test_user(|username : String, user_repo: UserRepository| async move {
            assert!(user_repo.create_user(&username2).await.is_ok());
            assert!(user_repo.create_user(&username3).await.is_ok());

            assert!(user_repo.update_user(&User {
                name: username2.clone(),
                number_of_games: 1,
                best_score: 500,
                highest_level: 1,
            }).await.is_ok());

            assert!(user_repo.update_user(&User {
                name: username3.clone(),
                number_of_games: 2,
                best_score: 1000,
                highest_level: 2,
            }).await.is_ok());

            assert!(user_repo.update_user(&User {
                name: username.clone(),
                number_of_games: 3,
                best_score: 100,
                highest_level: 3,
            }).await.is_ok());

            let users = user_repo.get_top_users(3).await;
            assert!(users.is_ok());
            let users = users.unwrap();

            assert_eq!(users.len(), 3);
            
            assert_eq!(users[0].name, username3);
            assert_eq!(users[0].best_score, 1000);
            assert_eq!(users[1].name, username2);
            assert_eq!(users[1].best_score, 500);
            assert_eq!(users[2].name, username);
            assert_eq!(users[2].best_score, 100);
            assert!(user_repo.delete_user(&username2).await.is_ok());
            assert!(user_repo.delete_user(&username3).await.is_ok());
        }).await;
    }

    #[tokio::test]
    async fn test_delete_user_success() {
        let fixture = RepositoryTestFixture::new().await;
        fixture.with_test_user(|_ : String, _: UserRepository| async move {
            // Nothing to do here because the with_test_user will delete the user
        }).await;
    }
    #[tokio::test]
    async fn test_delete_user_not_found() {
        let fixture = RepositoryTestFixture::new().await;
        let username = fixture.random_user_name();
        assert_repository_not_found!(fixture.user_repo.delete_user(&username).await);
    }
}   