#[cfg(test)]
mod tests {
    use crate::tests::unit::helpers::service_helpers::ServiceTestFixture;
    use crate::{assert_service_already_exists, assert_service_not_found, assert_service_unable_to_delete};
    use crate::services::{SessionServiceTrait, UserServiceTrait};
    #[tokio::test]
    async fn test_create_session_success() {
        let fixture = ServiceTestFixture::new().await;
        fixture.with_test_user_and_session(|_username, _session_id, _user_service, _session_service| async move {
            // nothing to do here a session is created by the fixture
        }).await;
    }

    #[tokio::test]
    async fn test_create_session_duplicate() {
        let fixture = ServiceTestFixture::new().await;
        fixture.with_test_user_and_session(|username, session_id, _user_service, session_service| async move {
            assert_service_already_exists!(session_service.create(&username, &session_id).await);
        }).await;
    }

    #[tokio::test]
    async fn test_get_session_by_id_success() {
        let fixture = ServiceTestFixture::new().await;
        fixture.with_test_user_and_session(|username, session_id, _user_service, session_service| async move {
            let session = session_service.get_by_id(&session_id).await.unwrap();
            assert_eq!(session.name, username);
            assert_eq!(session.session_id, session_service.hash_session_id(&session_id));
        }).await;
    }

    #[tokio::test]
    async fn test_get_session_by_id_not_found() {
        let fixture = ServiceTestFixture::new().await;
        let session_id = fixture.random_session_hash();
        assert_service_not_found!(fixture.session_service.get_by_id(&session_id).await);
    }

    #[tokio::test]
    async fn test_delete_session_success() {
        let fixture = ServiceTestFixture::new().await;
        fixture.with_test_user_and_session(|_username, _session_id, _user_service, _session_service| async move {
            // nothing to do here a session is created and deletedby the fixture
        }).await;
    }

    #[tokio::test]
    async fn test_delete_session_not_found() {
        let fixture = ServiceTestFixture::new().await;
        let session_id = fixture.random_session_hash();
        assert_service_unable_to_delete!(fixture.session_service.delete(&session_id).await);
    }
}