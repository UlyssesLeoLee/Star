//! # Exclusion Policy Loader
//!
//! per `docs/architecture/2026-09-07-exclusion-idempotency/03-detailed-design.md` §2.3.1
//!
//! 从 `exclusion_policy_master` 表加载策略 (per EX-01 SQL migration, Master 表 SCD Type 2)
//!
//! 守门合规:
//! - 守门 #13 c: Master 100% RLS 13 类必携
//! - 守门 #13 c: Master SCD Type 2 必携 (valid_from/valid_to)

use std::collections::HashMap;

/// 锁策略 (per `exclusion_policy_master.lock_strategy`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockStrategy {
    /// PG advisory lock.
    Advisory,
    /// Lease + heartbeat.
    Lease,
    /// Version CAS 乐观锁.
    Optimistic,
}

impl LockStrategy {
    /// 从 SQL `lock_strategy` 字符串解析.
    pub fn parse(s: &str) -> Result<Self, String> {
        match s {
            "advisory" => Ok(Self::Advisory),
            "lease" => Ok(Self::Lease),
            "optimistic" => Ok(Self::Optimistic),
            other => Err(format!("unknown lock_strategy: {}", other)),
        }
    }

    /// 序列化为 SQL 字符串.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Advisory => "advisory",
            Self::Lease => "lease",
            Self::Optimistic => "optimistic",
        }
    }
}

/// 锁策略配置.
#[derive(Debug, Clone)]
pub struct ExclusionPolicy {
    /// 策略名.
    pub name: String,
    /// 锁策略类型.
    pub strategy: LockStrategy,
    /// 超时 (ms).
    pub timeout_ms: u32,
    /// 重试最大次数.
    pub retry_max: u32,
}

impl Default for ExclusionPolicy {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            strategy: LockStrategy::Advisory,
            timeout_ms: 5000,
            retry_max: 3,
        }
    }
}

/// 策略加载器 (mock 实现 - per H2 v18 实证).
///
/// 实际生产从 `exclusion_policy_master` 表按 policy_name + valid_to IS NULL 查询当前活跃策略.
pub struct ExclusionPolicyLoader {
    /// 模拟策略存储.
    policies: HashMap<String, ExclusionPolicy>,
}

impl ExclusionPolicyLoader {
    /// 创建新 loader.
    pub fn new() -> Self {
        let mut policies = HashMap::new();
        policies.insert(
            "default".to_string(),
            ExclusionPolicy {
                name: "default".to_string(),
                strategy: LockStrategy::Advisory,
                timeout_ms: 5000,
                retry_max: 3,
            },
        );
        policies.insert(
            "domain_row_lock".to_string(),
            ExclusionPolicy {
                name: "domain_row_lock".to_string(),
                strategy: LockStrategy::Optimistic,
                timeout_ms: 5000,
                retry_max: 3,
            },
        );
        policies.insert(
            "task_card_lease".to_string(),
            ExclusionPolicy {
                name: "task_card_lease".to_string(),
                strategy: LockStrategy::Lease,
                timeout_ms: 300_000, // 5 min
                retry_max: 1,
            },
        );
        Self { policies }
    }

    /// 加载策略 by name.
    pub fn load(&self, name: &str) -> Result<ExclusionPolicy, String> {
        self.policies
            .get(name)
            .cloned()
            .ok_or_else(|| format!("policy not found: {}", name))
    }

    /// 测用: 注册新策略.
    pub fn register(&mut self, policy: ExclusionPolicy) {
        self.policies.insert(policy.name.clone(), policy);
    }
}

impl Default for ExclusionPolicyLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lock_strategy_parse() {
        assert_eq!(
            LockStrategy::parse("advisory").unwrap(),
            LockStrategy::Advisory
        );
        assert_eq!(LockStrategy::parse("lease").unwrap(), LockStrategy::Lease);
        assert_eq!(
            LockStrategy::parse("optimistic").unwrap(),
            LockStrategy::Optimistic
        );
        assert!(LockStrategy::parse("unknown").is_err());
    }

    #[test]
    fn test_lock_strategy_as_str() {
        assert_eq!(LockStrategy::Advisory.as_str(), "advisory");
        assert_eq!(LockStrategy::Lease.as_str(), "lease");
        assert_eq!(LockStrategy::Optimistic.as_str(), "optimistic");
    }

    #[test]
    fn test_load_default_policy() {
        let loader = ExclusionPolicyLoader::new();
        let policy = loader.load("default").unwrap();
        assert_eq!(policy.name, "default");
        assert_eq!(policy.strategy, LockStrategy::Advisory);
        assert_eq!(policy.timeout_ms, 5000);
    }

    #[test]
    fn test_load_domain_row_lock_policy() {
        let loader = ExclusionPolicyLoader::new();
        let policy = loader.load("domain_row_lock").unwrap();
        assert_eq!(policy.strategy, LockStrategy::Optimistic);
    }

    #[test]
    fn test_load_task_card_lease_policy() {
        let loader = ExclusionPolicyLoader::new();
        let policy = loader.load("task_card_lease").unwrap();
        assert_eq!(policy.strategy, LockStrategy::Lease);
        assert_eq!(policy.timeout_ms, 300_000);
    }

    #[test]
    fn test_load_not_found() {
        let loader = ExclusionPolicyLoader::new();
        assert!(loader.load("nonexistent").is_err());
    }

    #[test]
    fn test_register_policy() {
        let mut loader = ExclusionPolicyLoader::new();
        loader.register(ExclusionPolicy {
            name: "custom".to_string(),
            strategy: LockStrategy::Optimistic,
            timeout_ms: 1000,
            retry_max: 5,
        });
        let policy = loader.load("custom").unwrap();
        assert_eq!(policy.timeout_ms, 1000);
    }
}
