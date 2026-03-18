use actix_web::{FromRequest, HttpMessage, HttpRequest, dev::Payload};
use crate::models::AuthenticatedUser;
use crate::errors::AppError;
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