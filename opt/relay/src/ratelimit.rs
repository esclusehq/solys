use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Instant;

use dashmap::DashMap;

struct TokenBucket {
    tokens: f64,
    last_refill: Instant,
}

pub struct RateLimiter {
    buckets: DashMap<IpAddr, Mutex<TokenBucket>>,
    requests_per_minute: u32,
    #[allow(dead_code)]
    last_sweep_seq: AtomicU64,
}

impl RateLimiter {
    pub fn new(requests_per_minute: u32) -> Self {
        Self {
            buckets: DashMap::new(),
            requests_per_minute,
            last_sweep_seq: AtomicU64::new(0),
        }
    }

    /// Returns true if the request is allowed (token available), false if rate-limited.
    /// Refills the bucket proportionally to the elapsed time since last refill.
    pub fn check(&self, ip: IpAddr) -> bool {
        let now = Instant::now();
        let rate_per_sec = self.requests_per_minute as f64 / 60.0;
        let capacity = self.requests_per_minute as f64;

        let bucket_ref = self.buckets.entry(ip).or_insert_with(|| {
            Mutex::new(TokenBucket {
                tokens: capacity,
                last_refill: now,
            })
        });

        let mut bucket = bucket_ref.lock().unwrap();
        let elapsed = now.duration_since(bucket.last_refill).as_secs_f64();
        bucket.tokens = (bucket.tokens + elapsed * rate_per_sec).min(capacity);
        bucket.last_refill = now;

        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    /// Count of tracked IPs (for diagnostics/tests).
    #[allow(dead_code)]
    pub fn tracked(&self) -> usize {
        self.buckets.len()
    }

    /// Approximate summary for logging. Not used on the hot path.
    #[allow(dead_code)]
    pub fn summary(&self) -> HashMap<String, u64> {
        let mut m = HashMap::new();
        m.insert("tracked_ips".to_string(), self.buckets.len() as u64);
        m.insert("rate_per_minute".to_string(), self.requests_per_minute as u64);
        m.insert(
            "sweep_seq".to_string(),
            self.last_sweep_seq.fetch_add(0, Ordering::Relaxed),
        );
        m
    }
}
