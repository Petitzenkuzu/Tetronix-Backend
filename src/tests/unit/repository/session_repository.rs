#[cfg(test)]
mod tests {
    use crate::tests::unit::helpers::repository_helpers::RepositoryTestFixture;
    use crate::{assert_repository_not_found, assert_repository_already_exists};


    #[tokio::test]
    async fn test_create_session_success() {
        let fixture = RepositoryTestFixture::new().await;
        fixture.with_test_user_and_session(|_username, _session_hash, _user_repo, _session_repo| async move {

        }).await;
    }
    #[tokio::test]
    async fn test_create_session_invalid_input() {
        let fixture = RepositoryTestFixture::new().await;
        fixture.with_test_user_and_session(|username, session_hash, _user_repo, session_repo| async move {
            assert_repository_already_exists!(session_repo.create_session(&username, &session_hash).await);
        }).await;
    }

    #[tokio::test]
    async fn test_get_session_by_id_not_found() {
        let fixture = RepositoryTestFixture::new().await;
        let session_hash = fixture.random_session_hash();
        assert_repository_not_found!(fixture.session_repo.get_session_by_id(&session_hash).await);
    }

    #[tokio::test]
    async fn test_get_session_by_id_success() {
        let fixture = RepositoryTestFixture::new().await;
        fixture.with_test_user_and_session(|username, session_hash, _user_repo, session_repo| async move {
            let session = session_repo.get_session_by_id(&session_hash).await.unwrap();
            assert_eq!(session.name, username);
            assert_eq!(session.session_id, session_hash);
        }).await;
    }

    #[tokio::test]
    async fn test_delete_session_success() {
        let fixture = RepositoryTestFixture::new().await;
        fixture.with_test_user_and_session(|_username, _session_hash, _user_repo, _session_repo| async move {
            // Nothing to do here because the with_test_user_and_session will delete the session
        }).await;
    }

    #[tokio::test]
    async fn test_delete_session_not_found() {
        let fixture = RepositoryTestFixture::new().await;
        let session_hash = fixture.random_session_hash();
        assert_repository_not_found!(fixture.session_repo.delete_session(&session_hash).await);
    }

    #[tokio::test]
    async fn test_delete_session_by_name_success() {
        let fixture = RepositoryTestFixture::new().await;
        let username = fixture.random_user_name();

        assert!(fixture.user_repo.create_user(&username).await.is_ok());

        let session_hash1 = fixture.random_session_hash();
        let session_hash2 = fixture.random_session_hash();
        let session_hash3 = fixture.random_session_hash();

        assert!(fixture.session_repo.create_session(&username, &session_hash1).await.is_ok());
        assert!(fixture.session_repo.create_session(&username, &session_hash2).await.is_ok());
        assert!(fixture.session_repo.create_session(&username, &session_hash3).await.is_ok());

        assert!(fixture.session_repo.delete_session_by_name(&username).await.is_ok());

        assert_repository_not_found!(fixture.session_repo.get_session_by_id(&session_hash1).await);
        assert_repository_not_found!(fixture.session_repo.get_session_by_id(&session_hash2).await);
        assert_repository_not_found!(fixture.session_repo.get_session_by_id(&session_hash3).await);

        assert!(fixture.user_repo.delete_user(&username).await.is_ok());
    }

    #[tokio::test]
    async fn test_delete_session_by_name_not_found() {
        let fixture = RepositoryTestFixture::new().await;
        let username = fixture.random_user_name();
        assert_repository_not_found!(fixture.session_repo.delete_session_by_name(&username).await);
    }
}