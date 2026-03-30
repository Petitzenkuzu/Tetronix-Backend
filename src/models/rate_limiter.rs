use std::time::Instant;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub struct TokenBucket {
    pub token: u8,
    pub capacity: u8,
    pub refill_rate: u8,
    pub last_refill: Instant,
}

pub struct RateLimiter {
    pub clients: Arc<Mutex<HashMap<String, TokenBucket>>>,
}
