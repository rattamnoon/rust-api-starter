use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

#[derive(Clone)]
pub struct RateLimiter {
    inner: Arc<Mutex<HashMap<String, VecDeque<Instant>>>>,
    limit: usize,
    window: Duration,
}

pub struct RateLimitDecision {
    pub allowed: bool,
    pub retry_after_seconds: u64,
}

impl RateLimiter {
    pub fn new(limit: usize, window: Duration) -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
            limit,
            window,
        }
    }

    pub fn check(&self, key: &str) -> RateLimitDecision {
        let now = Instant::now();
        let mut store = self
            .inner
            .lock()
            .expect("rate limiter mutex should not be poisoned");

        let entries = store.entry(key.to_string()).or_default();
        while let Some(front) = entries.front() {
            if now.duration_since(*front) >= self.window {
                entries.pop_front();
            } else {
                break;
            }
        }

        if entries.len() >= self.limit {
            let retry_after_seconds = entries
                .front()
                .map(|oldest| {
                    self.window
                        .saturating_sub(now.duration_since(*oldest))
                        .as_secs()
                        .max(1)
                })
                .unwrap_or(1);

            return RateLimitDecision {
                allowed: false,
                retry_after_seconds,
            };
        }

        entries.push_back(now);
        RateLimitDecision {
            allowed: true,
            retry_after_seconds: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_requests_within_limit() {
        let limiter = RateLimiter::new(2, Duration::from_secs(60));
        assert!(limiter.check("127.0.0.1").allowed);
        assert!(limiter.check("127.0.0.1").allowed);
    }

    #[test]
    fn blocks_requests_beyond_limit() {
        let limiter = RateLimiter::new(1, Duration::from_secs(60));
        assert!(limiter.check("127.0.0.1").allowed);

        let blocked = limiter.check("127.0.0.1");
        assert!(!blocked.allowed);
        assert!(blocked.retry_after_seconds >= 1);
    }
}
