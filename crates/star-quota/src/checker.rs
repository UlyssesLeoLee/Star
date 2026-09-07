// Quota 默认值 + LlmTokenBucket (per G.5, 守门 #13 d 派生 M SCD Type 2)
use uuid::Uuid;

/// Compute 默认配额 (CPU/内存秒, per 守门 #13 d)
pub fn compute_default_limit() -> u64 {
    3_600_000 // 1 小时
}

/// Storage 默认配额 (字节, 1 GB)
pub fn storage_default_limit() -> u64 {
    1_073_741_824 // 1 GiB
}

/// LLM Token 默认配额 (per 守门 #4 token-OLU 基线 STAR-OLU-001 v0.1)
pub fn token_default_limit() -> u64 {
    1_200_000 // 1 SRE · 周
}

/// LLM Token 桶 (per token-bucket 限流, 防 burst 超限)
pub struct LlmTokenBucket {
    /// 租户 ID
    pub tenant_id: Uuid,
    /// 上限
    pub limit: u64,
    /// 当前剩余
    pub remaining: u64,
    /// 速率 (tokens / sec, 0 = 不限速)
    pub refill_per_sec: u64,
    /// 上次 refill 时间戳 (ms)
    pub last_refill_ms: u64,
}

impl LlmTokenBucket {
    /// 创建新 token 桶
    pub fn new(tenant_id: Uuid, limit: u64, refill_per_sec: u64) -> Self {
        Self {
            tenant_id,
            limit,
            remaining: limit,
            refill_per_sec,
            last_refill_ms: now_ms(),
        }
    }

    /// 尝试扣减 tokens (返回 Ok 表示成功, Err 表示不够)
    pub fn try_consume(&mut self, amount: u64, now_ms: u64) -> Result<(), String> {
        // 1) refill
        if self.refill_per_sec > 0 {
            let elapsed_sec = now_ms.saturating_sub(self.last_refill_ms) / 1000;
            let refill = elapsed_sec * self.refill_per_sec;
            self.remaining = (self.remaining + refill).min(self.limit);
            self.last_refill_ms = now_ms;
        }
        // 2) consume
        if self.remaining < amount {
            return Err(format!(
                "insufficient tokens: tenant={}, requested={}, remaining={}",
                self.tenant_id, amount, self.remaining
            ));
        }
        self.remaining -= amount;
        Ok(())
    }
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn token_bucket_consume_within_limit() {
        let mut b = LlmTokenBucket::new(Uuid::new_v4(), 1000, 0);
        b.try_consume(500, 0).unwrap();
        assert_eq!(b.remaining, 500);
    }

    #[test]
    fn token_bucket_consume_exceeds() {
        let mut b = LlmTokenBucket::new(Uuid::new_v4(), 1000, 0);
        b.try_consume(500, 0).unwrap();
        assert!(b.try_consume(600, 0).is_err());
    }

    #[test]
    fn token_bucket_refill_over_time() {
        let mut b = LlmTokenBucket::new(Uuid::new_v4(), 1000, 10);
        b.try_consume(1000, 0).unwrap();
        assert_eq!(b.remaining, 0);
        // 5 秒后应 refill 50
        b.try_consume(50, 5000).unwrap();
        assert_eq!(b.remaining, 0);
    }

    #[test]
    fn default_limits_reasonable() {
        assert!(compute_default_limit() > 0);
        assert!(storage_default_limit() > 0);
        assert!(token_default_limit() > 0);
    }
}
