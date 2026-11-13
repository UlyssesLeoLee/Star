/*
@cypher schema=1 source_sha256=b5c600e56047d640a3a7e2c59faa33f8f650029c8f43a0caa61bafac3ac4350d
MERGE (self:File {path:"crates/star-agent-runtime/src/config.rs"})
@endcypher
*/
//! RuntimeConfig (per SRS §100-§101 + §7.4 + §30 + 03-DD §3.2)。
//!
//! 所有资源控制参数配置化 (per SRS §100), 默认值 Fail Safe (per SRS §101):
//! 配置非法 → `validate()` 拒绝, 绝不静默退化为 Unlimited Concurrency / Queue。
//! YAML/JSON 解析 (serde) 待 P3-B 注册进根 workspace 后接线 (骨架 std-only)。

use std::fmt;
use std::time::Duration;

use crate::backpressure::OverflowPolicy;
use crate::runtime_mode::RuntimeModeConfig;

/// 配置错误 (per SRS §101 Fail Safe)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigError {
    /// 全部问题列表。
    pub problems: Vec<String>,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "配置非法: {}", self.problems.join("; "))
    }
}

impl std::error::Error for ConfigError {}

/// STAR Agent Runtime 资源控制参数 (per SRS §100)。
#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeConfig {
    /// 模式阈值 (per SRS §7)。
    pub runtime_mode: RuntimeModeConfig,
    /// HOT 槽位上限 (per SRS §30, 默认 100)。
    pub max_hot_agents: usize,
    /// 各类外部并发上限 (per SRS §44, 禁止 unlimited spawn)。
    pub max_llm_requests: usize,
    /// Tool 并发上限。
    pub max_tool_requests: usize,
    /// MCP 并发上限。
    pub max_mcp_requests: usize,
    /// RAG 并发上限。
    pub max_rag_requests: usize,
    /// Context 装载并发上限。
    pub max_context_loads: usize,
    /// 队列上限 (per SRS §47, 禁止无界)。
    pub max_queue_size: usize,
    /// 溢出策略 (per SRS §47)。
    pub overflow_policy: OverflowPolicy,
    /// HOT → WARM 超时 (per 03-DD §3.2, 默认 60s)。
    pub hot_to_warm_timeout: Duration,
    /// WARM → COLD idle 超时 (默认 600s)。
    pub warm_to_cold_idle_timeout: Duration,
    /// COLD 恢复超时 (默认 30s, per 03-DD §3.2)。
    pub restore_timeout: Duration,
    /// 驱逐批大小 (默认 100, per 03-DD §3.2)。
    pub eviction_batch_size: usize,
    /// 最大重试 (per SRS §51, 禁止无限)。
    pub max_retry: u32,
    /// Mailbox 上限 (per SRS §19, 禁止无界)。
    pub mailbox_max_size: usize,
    /// WARM agent 内存预算 (per SRS §62, 默认 100 KB)。
    pub warm_agent_memory_budget_bytes: u64,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            runtime_mode: RuntimeModeConfig::default(),
            max_hot_agents: 100,
            max_llm_requests: 16,
            max_tool_requests: 32,
            max_mcp_requests: 16,
            max_rag_requests: 8,
            max_context_loads: 8,
            max_queue_size: 10_000,
            overflow_policy: OverflowPolicy::default(),
            hot_to_warm_timeout: Duration::from_secs(60),
            warm_to_cold_idle_timeout: Duration::from_secs(600),
            restore_timeout: Duration::from_secs(30),
            eviction_batch_size: 100,
            max_retry: 3,
            mailbox_max_size: 256,
            warm_agent_memory_budget_bytes: 100 * 1024,
        }
    }
}

impl RuntimeConfig {
    /// Fail Safe 校验 (per SRS §101): 非法即拒 (ConfigError)。
    pub fn validate(&self) -> Result<(), ConfigError> {
        let mut problems = Vec::new();
        if let Err(e) = self.runtime_mode.validate() {
            problems.push(e.to_string());
        }
        for (name, value) in [
            ("max_hot_agents", self.max_hot_agents),
            ("max_llm_requests", self.max_llm_requests),
            ("max_tool_requests", self.max_tool_requests),
            ("max_mcp_requests", self.max_mcp_requests),
            ("max_rag_requests", self.max_rag_requests),
            ("max_context_loads", self.max_context_loads),
            ("max_queue_size", self.max_queue_size),
            ("eviction_batch_size", self.eviction_batch_size),
            ("mailbox_max_size", self.mailbox_max_size),
        ] {
            if value == 0 {
                problems.push(format!("{name} 必须 >= 1 (禁止无界, per SRS §44/§47)"));
            }
        }
        if self.hot_to_warm_timeout.is_zero() || self.warm_to_cold_idle_timeout.is_zero() {
            problems.push("lifecycle timeout 必须 > 0".to_string());
        }
        if problems.is_empty() {
            Ok(())
        } else {
            Err(ConfigError { problems })
        }
    }
}
