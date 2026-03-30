use crate::errors::AppError;
use crate::models::AuthenticatedUser;
use actix_web::{dev::Payload, FromRequest, HttpMessage, HttpRequest};
use futures_util::future::{ready, Ready};

impl FromRequest for AuthenticatedUser {
    type Error = AppError;
    type Future = Ready<Result<Self, AppError>>;
    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        match req.extensions().get::<AuthenticatedUser>() {
            Some(user) => ready(Ok(user.clone())),
            None => ready(Err(AppError::Unauthorized)),
        }
    }
}
