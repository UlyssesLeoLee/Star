/*
@cypher schema=1 source_sha256=913c1e508b4170695cdfb880121cfd72fbfb9017369c1041c8ce0cf19951cc27
MERGE (self:File {path:"crates/star-agent-runtime/src/rate_limiter.rs"})
@endcypher
*/
//! Token Bucket 速率控制 (per 02-BD §3.1 RateLimiter + SRS §42/§56)。
//!
//! per-tenant 独立桶 (per SRS §56: 不得允许单 Tenant 独占所有资源)。
//! 时间注入 (now 参数) 保证测试确定性。

use std::collections::HashMap;
use std::fmt;
use std::sync::Mutex;
use std::time::Instant;

use crate::id::TenantId;

/// TokenBucket 参数错误。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RateLimiterError {
    /// capacity 必须 >= 1。
    ZeroCapacity,
}

impl fmt::Display for RateLimiterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RateLimiterError::ZeroCapacity => write!(f, "capacity 必须 >= 1"),
        }
    }
}

impl std::error::Error for RateLimiterError {}

/// Token Bucket (per 02-BD §3.3 同构)。
#[derive(Debug, Clone)]
pub struct TokenBucket {
    capacity: u32,
    refill_per_sec: f64,
    tokens: f64,
    last: Instant,
}

impl TokenBucket {
    /// 新建。refill_per_sec = 0 表示不自动补充 (配额/纯限流)。
    pub fn new(capacity: u32, refill_per_sec: f64, now: Instant) -> Result<Self, RateLimiterError> {
        if capacity == 0 {
            return Err(RateLimiterError::ZeroCapacity);
        }
        Ok(Self {
            capacity,
            refill_per_sec,
            tokens: capacity as f64,
            last: now,
        })
    }

    fn refill(&mut self, now: Instant) {
        let elapsed = now.duration_since(self.last).as_secs_f64();
        if elapsed > 0.0 {
            self.tokens = (self.tokens + elapsed * self.refill_per_sec).min(self.capacity as f64);
            self.last = now;
        }
    }

    /// 获取 n 个 token (per 02-BD §3.1 acquire)。
    pub fn try_acquire(&mut self, n: u32, now: Instant) -> bool {
        self.refill(now);
        if self.tokens >= n as f64 {
            self.tokens -= n as f64;
            true
        } else {
            false
        }
    }

    /// 当前可用 token。
    pub fn available(&mut self, now: Instant) -> f64 {
        self.refill(now);
        self.tokens
    }
}

/// per-tenant Token Bucket 表 (per SRS §42 Tenant Quota 雏形)。线程安全。
#[derive(Debug)]
pub struct TenantRateLimiter {
    capacity: u32,
    refill_per_sec: f64,
    buckets: Mutex<HashMap<TenantId, TokenBucket>>,
}

impl TenantRateLimiter {
    /// 新建 (缺省桶参数)。
    pub fn new(capacity: u32, refill_per_sec: f64) -> Result<Self, RateLimiterError> {
        if capacity == 0 {
            return Err(RateLimiterError::ZeroCapacity);
        }
        Ok(Self {
            capacity,
            refill_per_sec,
            buckets: Mutex::new(HashMap::new()),
        })
    }

    /// acquire(tenant) (per 02-BD §3.1)。
    pub fn acquire(&self, tenant_id: TenantId, n: u32, now: Instant) -> bool {
        let mut buckets = self.buckets.lock().expect("rate limiter lock poisoned");
        let bucket = buckets.entry(tenant_id).or_insert_with(|| {
            TokenBucket::new(self.capacity, self.refill_per_sec, now)
                .expect("capacity validated in Self::new")
        });
        bucket.try_acquire(n, now)
    }

    /// release(tenant) (per 02-BD §3.1): 归还 token (不超 capacity)。
    pub fn release(&self, tenant_id: TenantId, n: u32, now: Instant) {
        let mut buckets = self.buckets.lock().expect("rate limiter lock poisoned");
        if let Some(bucket) = buckets.get_mut(&tenant_id) {
            bucket.refill(now);
            bucket.tokens = (bucket.tokens + n as f64).min(bucket.capacity as f64);
        }
    }

    /// 桶数量。
    pub fn bucket_count(&self) -> usize {
        self.buckets
            .lock()
            .expect("rate limiter lock poisoned")
            .len()
    }
}
