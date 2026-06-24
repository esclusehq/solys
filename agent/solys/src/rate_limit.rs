//! Rate Limiter - Token bucket rate limiting

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::RwLock;

pub struct RateLimiter {
    task_type_buckets: HashMap<String, TokenBucket>,
    user_buckets: HashMap<uuid::Uuid, TokenBucket>,
    agent_bucket: TokenBucket,
}

#[derive(Clone)]
pub struct RateLimiterHandle {
    inner: Arc<RwLock<RateLimiter>>,
}

#[derive(Debug, Clone)]
pub struct TokenBucket {
    capacity: u64,
    tokens: f64,
    refill_rate: f64,
    last_refill: Instant,
}

#[derive(Debug)]
pub enum RateLimitError {
    TaskTypeLimitExceeded(String),
    UserLimitExceeded(uuid::Uuid),
    AgentLimitExceeded,
    RetryAfter(Duration),
}

impl std::fmt::Display for RateLimitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RateLimitError::TaskTypeLimitExceeded(ty) => write!(f, "Task type '{}' rate limit exceeded", ty),
            RateLimitError::UserLimitExceeded(uid) => write!(f, "User {} rate limit exceeded", uid),
            RateLimitError::AgentLimitExceeded => write!(f, "Agent rate limit exceeded"),
            RateLimitError::RetryAfter(d) => write!(f, "Rate limited, retry after {:?}", d),
        }
    }
}

impl RateLimiterHandle {
    pub fn new(
        tasks_per_minute: u64,
        _user_per_minute: u64,
        agent_per_minute: u64,
    ) -> Self {
        let refill_rate = tasks_per_minute as f64 / 60.0;
        
        Self {
            inner: Arc::new(RwLock::new(RateLimiter {
                task_type_buckets: HashMap::new(),
                user_buckets: HashMap::new(),
                agent_bucket: TokenBucket::new(agent_per_minute, refill_rate),
            })),
        }
    }

    pub async fn check_rate_limit(
        &self,
        task_type: &str,
        user_id: Option<uuid::Uuid>,
    ) -> Result<(), RateLimitError> {
        let mut limiter = self.inner.write().await;

        // Check agent total limit
        if !limiter.agent_bucket.try_consume(1) {
            return Err(RateLimitError::AgentLimitExceeded);
        }

        // Check task type limit
        let task_bucket = limiter.task_type_buckets
            .entry(task_type.to_string())
            .or_insert_with(|| TokenBucket::new(30, 0.5));
        
        if !task_bucket.try_consume(1) {
            return Err(RateLimitError::TaskTypeLimitExceeded(task_type.to_string()));
        }

        // Check per-user limit
        if let Some(uid) = user_id {
            let user_bucket = limiter.user_buckets
                .entry(uid)
                .or_insert_with(|| TokenBucket::new(60, 1.0));
            
            if !user_bucket.try_consume(1) {
                return Err(RateLimitError::UserLimitExceeded(uid));
            }
        }

        Ok(())
    }
}

impl TokenBucket {
    pub fn new(capacity: u64, refill_rate: f64) -> Self {
        Self {
            capacity,
            tokens: capacity as f64,
            refill_rate,
            last_refill: Instant::now(),
        }
    }

    pub fn try_consume(&mut self, tokens: u64) -> bool {
        self.refill();
        
        if self.tokens >= tokens as f64 {
            self.tokens -= tokens as f64;
            true
        } else {
            false
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        let new_tokens = elapsed * self.refill_rate;
        self.tokens = (self.tokens + new_tokens).min(self.capacity as f64);
        self.last_refill = now;
    }
}

impl Default for RateLimiterHandle {
    fn default() -> Self {
        Self::new(30, 60, 60)
    }
}

lazy_static::lazy_static! {
    pub static ref RATE_LIMITER: RateLimiterHandle = RateLimiterHandle::default();
}

pub async fn check_rate_limit(task_type: &str, user_id: Option<uuid::Uuid>) -> Result<(), RateLimitError> {
    RATE_LIMITER.check_rate_limit(task_type, user_id).await
}
