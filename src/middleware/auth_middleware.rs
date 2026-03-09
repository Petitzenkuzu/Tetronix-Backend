use std::future::{ready, Ready};

use actix_web::{
    Error, HttpMessage, dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready}
};
use futures_util::future::LocalBoxFuture;
use actix_web::web::Data;
use crate::models::ConcreteAppState;
use crate::errors::AppError;
use crate::services::AuthServiceTrait;
use crate::models::AuthenticatedUser;
use tracing::info;

pub struct Auth;

impl<S, B> Transform<S, ServiceRequest> for Auth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddleware { service }))
    }
}

pub struct AuthMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let app_state = match req.app_data::<Data<ConcreteAppState>>() {
            Some(app_state) => app_state,
            None => return Box::pin(async move {
                Err(AppError::InternalServerError("Something went wrong".to_string()).into())
            }),
        };

        let auth_token = match req.cookie("auth_token") {
            Some(auth_token) => auth_token,
            None => return Box::pin(async move {
                Err(AppError::Unauthorized.into())
            }),
        };

        let username = match app_state.auth_service.verify_jwt(&auth_token.value()) {
            Ok(username) => username,
            Err(_) => return Box::pin(async move {
                Err(AppError::Unauthorized.into())
            })
        };

        info!("User {} authenticated", &username);
        req.extensions_mut().insert(AuthenticatedUser { username });
        
        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;

            Ok(res)
        })
    }
}