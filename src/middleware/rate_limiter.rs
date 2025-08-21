use crate::models::RateLimiter;
use crate::config::TokenBucketConfig;
use crate::models::TokenBucket;
use std::time::Instant;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::future::{ready, Ready};
use crate::errors::AppError;
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures_util::future::LocalBoxFuture;
use std::time::Duration;

impl TokenBucket {
    pub fn new(config: TokenBucketConfig) -> Self {
        Self {
            token: config.capacity,
            capacity: config.capacity,
            refill_rate: config.refill_rate,
            last_refill: Instant::now(),
        }
    }

    pub fn refill(&mut self) {
        // refill the token bucket
        let now = Instant::now();

        let time_since_last_refill = now.duration_since(self.last_refill);
        let max_elapsed_time = Duration::from_secs(100);

        let elapsed_secs = time_since_last_refill.min(max_elapsed_time).as_secs() as u8;
        
        let tokens_to_add = elapsed_secs.saturating_mul(self.refill_rate);
        self.token = self.token.saturating_add(tokens_to_add).min(self.capacity);
        self.last_refill = now;
    }
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn check_limit(&self, client_ip: String) -> bool {
        let mut clients = self.clients.lock().unwrap();
        let client = clients
                                            .entry(client_ip)
                                            .or_insert(TokenBucket::new(TokenBucketConfig::new()));

        client.refill();

        // no token = no more requests, rate limit exceeded
        if client.token == 0 {
            return false;
        }

        // if there is a token, use it
        client.token -= 1;
        true
    }
}



pub struct RateLimiterTransform;

impl<S, B> Transform<S, ServiceRequest> for RateLimiterTransform
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RateLimiterMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RateLimiterMiddleware { service, rate_limiter: Arc::new(RateLimiter::new()) }))
    }
}

pub struct RateLimiterMiddleware<S> {
    service: S,
    rate_limiter: Arc<RateLimiter>,
}

impl<S, B> Service<ServiceRequest> for RateLimiterMiddleware<S>
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

        // get the client ip
        let client_ip = match req.connection_info().peer_addr() {
            Some(ip) => Some(ip.to_string()),
            None => None,
        };

        // check the limit
        if let Some(ip) = client_ip {
            // if the limit is exceeded, return a 429 error
            if !self.rate_limiter.check_limit(ip) {
                return Box::pin(async move {
                    Err(AppError::RateLimitExceeded.into())
                });
            }
            // if the limit is not exceeded, call the service
            else {
                let fut = self.service.call(req);

                return Box::pin(async move {
                    let res = fut.await?;
                    Ok(res)
                })
            }
        }

        // if no ip is found, return a 500 error
        Box::pin(async move {
            Err(AppError::InternalServerError("No IP found".to_string()).into())
        })
    }
}