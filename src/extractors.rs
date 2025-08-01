/*use crate::models::Session;
use crate::data_base::get_session_from_id;
use actix_web::{FromRequest, HttpRequest, Error, dev::Payload, web::Data};
use crate::AppState;


impl FromRequest for Session {
    type Error = Error;
    type Future = futures_util::future::LocalBoxFuture<'static, Result<Self, Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future{
        let cookie = req.cookie("session_id");
        let pool = req.app_data::<Data<AppState>>().unwrap().db.clone();
    
        if let Some(cookie) = cookie {
            Box::pin(async move {
                let session = get_session_from_id(&pool, cookie.value()).await;
                match session {
                    Ok(session) => {
                        Ok(session)
                    }
                    Err(_e) => {
                        Err(actix_web::error::ErrorUnauthorized("Invalid session"))
                    }
                }
            })
        }
        else{
            Box::pin(async move {
                Err(actix_web::error::ErrorUnauthorized("No session found"))
            })
        }
    }

}*/