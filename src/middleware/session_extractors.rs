use crate::models::Session;
use actix_web::{FromRequest, HttpRequest, dev::Payload, web::Data};
use crate::models::AppState;
use crate::errors::{AppError, ServicesError};


impl FromRequest for Session {
    type Error = AppError;
    type Future = futures_util::future::LocalBoxFuture<'static, Result<Self, AppError>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future{
        let cookie = req.cookie("session_id");
        // if session_id cookies provided, we try to get the session from the database
        if let Some(cookie) = cookie {
            let session_service = req.app_data::<Data<AppState>>().unwrap().session_service.clone();
            Box::pin(async move {
                let session = session_service.get_by_id(cookie.value()).await
                .map_err(|e| if let ServicesError::NotFound{what: _} = e {
                    AppError::Unauthorized
                } else {
                    AppError::InternalServerError("Something went wrong".to_string())
                })?;
                Ok(Session{session_id: cookie.value().to_string(), name: session.name})
            })
        }
        else{
            // if no session_id cookies provided, we return an unauthorized error
            Box::pin(async move {
                Err(AppError::Unauthorized)
            })
        }
    }

}