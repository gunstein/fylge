use std::net::IpAddr;
use std::time::{Duration, Instant};

use dashmap::DashMap;

/// Simple IP-based rate limiter.
pub struct RateLimiter {
    /// Map from IP to (last_request_time, request_count)
    requests: DashMap<IpAddr, (Instant, u32)>,
    /// Maximum requests per window
    max_requests: u32,
    /// Time window
    window: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window: Duration) -> Self {
        Self {
            requests: DashMap::new(),
            max_requests,
            window,
        }
    }

    /// Check if a request from this IP is allowed.
    /// Returns Ok(()) if allowed, Err(remaining_wait_time) if rate limited.
    pub fn check(&self, ip: IpAddr) -> Result<(), Duration> {
        let now = Instant::now();

        let mut entry = self.requests.entry(ip).or_insert((now, 0));
        let (last_time, count) = entry.value_mut();

        // Reset if window has passed
        if now.duration_since(*last_time) >= self.window {
            *last_time = now;
            *count = 1;
            return Ok(());
        }

        // Check if over limit
        if *count >= self.max_requests {
            let wait_time = self.window - now.duration_since(*last_time);
            return Err(wait_time);
        }

        *count += 1;
        Ok(())
    }

    /// Clean up old entries (call periodically).
    pub fn cleanup(&self) {
        let now = Instant::now();
        self.requests
            .retain(|_, (last_time, _)| now.duration_since(*last_time) < self.window * 2);
    }
}

/// Rate limiter for write operations (1 per second).
pub fn write_limiter() -> RateLimiter {
    RateLimiter::new(1, Duration::from_secs(1))
}

/// Rate limiter for read operations (10 per second).
pub fn read_limiter() -> RateLimiter {
    RateLimiter::new(10, Duration::from_secs(1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_allows_under_limit() {
        let limiter = RateLimiter::new(3, Duration::from_secs(1));
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

        assert!(limiter.check(ip).is_ok());
        assert!(limiter.check(ip).is_ok());
        assert!(limiter.check(ip).is_ok());
    }

    #[test]
    fn test_blocks_over_limit() {
        let limiter = RateLimiter::new(2, Duration::from_secs(1));
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

        assert!(limiter.check(ip).is_ok());
        assert!(limiter.check(ip).is_ok());
        assert!(limiter.check(ip).is_err());
    }

    #[test]
    fn test_different_ips_independent() {
        let limiter = RateLimiter::new(1, Duration::from_secs(1));
        let ip1 = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let ip2 = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 2));

        assert!(limiter.check(ip1).is_ok());
        assert!(limiter.check(ip2).is_ok());
        assert!(limiter.check(ip1).is_err());
        assert!(limiter.check(ip2).is_err());
    }
}
