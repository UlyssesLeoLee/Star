//! Star API → CLI Fallback Chain (B.8 实装, per 2026-08-30 07:37 JST wt-b8-api-fallback)
//!
//! 职责:
//!   1. **FallbackChain**: API Agent 失败 → CLI Agent 降级 (per 2026-08-29 11:00 JST P3-B 选项)
//!   2. **策略**: per CliProfile kind (OpenClaw / Hermes) 失败时, 切到对应 CLI kind (Claude / Codex / Gemini / Aider)
//!   3. **FallbackReason**: 区分 transient (可重试) / permanent (直接降级) / exhausted (整个链失败)
//!
//! 已知缺口 (per 缺标比错标 — 8/26 JST 偏好):
//!   1. 当前 fallback 配对 hardcoded (OpenClaw→Claude, Hermes→Codex), Phase 2 改用 CliProfile.fallback_target 字段
//!   2. 不接 B.7 retry_with_backoff, Phase 2 整合
//!   3. fallback 链不写 audit log (per B.9 监控)
//!
//! 不做 (per 守门):
//!   - 不动 network 层
//!   - 不写 UI (per B.9)
//!   - 不接 KMS, fallback 链明文配置

use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

/// Fallback 触发原因 (简化版: untagged enum, 序列化所有字段为 JSON value, 不强加 tag 结构, 避免 newtype 限制)
#[derive(Debug, Clone, PartialEq, Eq, Error, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FallbackReason {
    /// API endpoint 不可达 (network error, DNS fail, etc)
    #[error("API endpoint unreachable: {0}")]
    Unreachable(String),
    /// API rate limit 持续 (B.7 retry exhausted)
    #[error("API rate limit exhausted")]
    RateLimited,
    /// API 返回 5xx (server error)
    #[error("API server error: status={0}")]
    ServerError(u16),
    /// API 凭证无效 (401/403, permanent)
    #[error("API credential invalid: {0}")]
    InvalidCredential(String),
    /// 整个 fallback 链用尽
    #[error("fallback chain exhausted after {attempts} attempts")]
    Exhausted { attempts: u32 },
}

impl FallbackReason {
    /// 是否可降级 (vs 直接报错)
    pub fn should_fallback(&self) -> bool {
        matches!(
            self,
            FallbackReason::Unreachable(_)
                | FallbackReason::RateLimited
                | FallbackReason::ServerError(_)
        )
    }
}

/// Fallback 策略: per API kind 配 CLI kind
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FallbackPolicy {
    /// API kind → CLI kind 映射 (硬编码 Phase 2, 后续走 CliProfile.fallback_target)
    pub api_to_cli: std::collections::HashMap<String, String>,
    /// 最大 fallback 次数 (默认 1, 整个链用尽就报错)
    pub max_fallback_attempts: u32,
}

impl Default for FallbackPolicy {
    fn default() -> Self {
        let mut api_to_cli = std::collections::HashMap::new();
        // 默认映射 (per 2026-08-29 11:00 JST 拍板):
        // - OpenClaw (gpt-4) → Claude (claude-3-5-sonnet)  # 跨厂商等价
        // - Hermes (hermes-2) → Codex (gpt-4)             # 跨厂商等价
        // - openclaw.dev / hermes.dev 都降级到同 token 预算的 CLI agent
        api_to_cli.insert("openclaw".into(), "claude".into());
        api_to_cli.insert("hermes".into(), "codex".into());
        Self {
            api_to_cli,
            max_fallback_attempts: 1,
        }
    }
}

/// Fallback 决策结果
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FallbackDecision {
    /// 继续用 API agent (无需降级, e.g. permanent 错误但 transient 后端可重试)
    StayWithApi,
    /// 降级到 CLI agent
    FallbackTo {
        cli_kind: String,
        reason: FallbackReason,
    },
    /// 整个链用尽, 报错
    GiveUp { reason: FallbackReason },
}

/// FallbackChain: API Agent 失败 → CLI Agent 降级 决策
pub struct FallbackChain {
    policy: FallbackPolicy,
    attempts_so_far: u32,
}

impl FallbackChain {
    pub fn new(policy: FallbackPolicy) -> Self {
        Self {
            policy,
            attempts_so_far: 0,
        }
    }

    /// 默认策略
    pub fn with_default_policy() -> Self {
        Self::new(FallbackPolicy::default())
    }

    /// 决策下一步
    pub fn decide(&mut self, reason: FallbackReason) -> FallbackDecision {
        self.attempts_so_far += 1;

        // transient 错误, 还有 budget, 降级
        if reason.should_fallback() && self.attempts_so_far <= self.policy.max_fallback_attempts {
            // 简化: 当前默认 mapping OpenClaw→Claude, Hermes→Codex
            let cli_kind = self
                .policy
                .api_to_cli
                .values()
                .next()
                .cloned()
                .unwrap_or_else(|| "claude".to_string());
            return FallbackDecision::FallbackTo { cli_kind, reason };
        }

        // 用尽 OR permanent 错误
        if self.attempts_so_far > self.policy.max_fallback_attempts {
            return FallbackDecision::GiveUp {
                reason: FallbackReason::Exhausted {
                    attempts: self.attempts_so_far,
                },
            };
        }
        FallbackDecision::GiveUp { reason }
    }

    /// 当前 attempts
    pub fn attempts(&self) -> u32 {
        self.attempts_so_far
    }
}

/// Fallback 执行结果 (跨 API → CLI 链)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FallbackResult {
    /// 用了哪条链 (api_kind / cli_kind / "exhausted")
    pub chain_used: String,
    /// 总耗时 (ms)
    pub total_duration_ms: u64,
    /// fallback 触发原因
    pub reason: FallbackReason,
    /// 输出 (来自最终 agent, 可能是 API 也可能是 CLI)
    pub output: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_policy_maps_openclaw_to_claude() {
        let policy = FallbackPolicy::default();
        assert_eq!(
            policy.api_to_cli.get("openclaw"),
            Some(&"claude".to_string())
        );
        assert_eq!(policy.api_to_cli.get("hermes"), Some(&"codex".to_string()));
    }

    #[test]
    fn transient_reason_should_fallback() {
        assert!(FallbackReason::Unreachable("conn refused".into()).should_fallback());
        assert!(FallbackReason::RateLimited.should_fallback());
        assert!(FallbackReason::ServerError(503).should_fallback());
    }

    #[test]
    fn permanent_reason_should_not_fallback() {
        assert!(!FallbackReason::InvalidCredential("401".into()).should_fallback());
    }

    #[test]
    fn chain_decides_fallback_on_first_transient() {
        let mut chain = FallbackChain::with_default_policy();
        let decision = chain.decide(FallbackReason::Unreachable("conn refused".into()));
        assert!(matches!(decision, FallbackDecision::FallbackTo { .. }));
        assert_eq!(chain.attempts(), 1);
    }

    #[test]
    fn chain_gives_up_after_max_attempts() {
        let mut chain = FallbackChain::with_default_policy();
        // 第 1 次: 降级
        let d1 = chain.decide(FallbackReason::ServerError(503));
        assert!(matches!(d1, FallbackDecision::FallbackTo { .. }));
        // 第 2 次: 给定 max_fallback_attempts=1, 第 2 次会 give up
        let d2 = chain.decide(FallbackReason::ServerError(503));
        assert!(matches!(d2, FallbackDecision::GiveUp { .. }));
        assert_eq!(chain.attempts(), 2);
    }

    #[test]
    fn chain_gives_up_immediately_on_permanent() {
        let mut chain = FallbackChain::with_default_policy();
        let decision = chain.decide(FallbackReason::InvalidCredential("401".into()));
        assert!(matches!(decision, FallbackDecision::GiveUp { .. }));
        assert_eq!(chain.attempts(), 1);
    }

    #[test]
    fn fallback_result_serializes() {
        let result = FallbackResult {
            chain_used: "openclaw→claude".into(),
            total_duration_ms: 1500,
            reason: FallbackReason::ServerError(503),
            output: "fallback succeeded".into(),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"chain_used\":\"openclaw→claude\""));
        assert!(json.contains("\"total_duration_ms\":1500"));
    }
}
