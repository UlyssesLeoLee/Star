/*
@cypher schema=1 source_sha256=ecfcd156024f01f7b07a8ab910f33f3d1524ee5192c97fdc62ebe650c6f5ad2f
MERGE (self:File {path:"crates/star-agent-runtime/src/components.rs"})
@endcypher
*/
//! ECS Component 定义 (per SRS §12-§20 + 03-DD §2.1)。
//!
//! 数据原则 (per SRS §70 REQ-ECS-DATA): Component 应 Small / Flat /
//! Reference-based / Cache-friendly。大型对象不直接进 Component —
//! Agent 只存 Ref (per SRS §8 不得存放)。

use std::fmt;

use crate::ecs::Component;
use crate::id::{
    AclId, CheckpointId, ContextId, MailboxId, MemoryId, PolicyId, TenantId, WorkflowId,
};

/// Context 分层 (per SRS §37 REQ-CTX-002)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContextTier {
    /// L1 Hot Context Cache (RAM, MB 级)。
    L1Hot,
    /// L2 Recent Context (Redis / Local Cache, GB 级)。
    L2Recent,
    /// L3 Full Context (DB / Object Store, TB 级)。
    L3Full,
}

/// 长期记忆类型 (per SRS §41 REQ-MEM-001)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryType {
    /// 语义记忆。
    Semantic,
    /// 情景记忆。
    Episodic,
    /// 用户记忆。
    User,
    /// 工作流记忆。
    Workflow,
    /// 知识引用。
    Knowledge,
}

/// 优先级 (per SRS §18 REQ-ECS-010), 数值越小越优先 (per 03-DD §2.1)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Priority {
    /// 危急。
    Critical = 0,
    /// 高。
    High = 1,
    /// 普通。
    Normal = 2,
    /// 低。
    Low = 3,
    /// 后台。
    Background = 4,
}

impl Component for Priority {}

/// 速率限制 (per 02-BD §3.1 RateLimiter, ToolPolicyRef 内嵌)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RateLimit {
    /// 每秒允许次数。
    pub per_sec: u32,
    /// 突发容量。
    pub burst: u32,
}

/// ContextRef (per SRS §12 REQ-ECS-004): 不含 Full Context。
#[derive(Debug, Clone)]
pub struct ContextRef {
    /// Context ID。
    pub context_id: ContextId,
    /// 分层 (per SRS §37)。
    pub tier: ContextTier,
    /// 是否已装载 (lazy load, per SRS §38)。
    pub loaded: bool,
    /// 共享 Context + Agent Delta 引用 (per SRS §40)。
    pub refs: Vec<ContextId>,
}

impl Component for ContextRef {}

/// MemoryRef (per SRS §13 REQ-ECS-005): Agent → MemoryRef → Memory Store。
#[derive(Debug, Clone)]
pub struct MemoryRef {
    /// Memory ID。
    pub memory_id: MemoryId,
    /// 记忆类型。
    pub memory_type: MemoryType,
}

impl Component for MemoryRef {}

/// ModelRef (per SRS §14 REQ-ECS-006): 模型配置, 实际连接走 Shared LLM Pool。
#[derive(Debug, Clone)]
pub struct ModelRef {
    /// Provider (如 "openai")。
    pub provider: String,
    /// Model (如 "gpt-4")。
    pub model: String,
    /// Profile。
    pub profile: String,
    /// 温度。
    pub temperature: f32,
    /// max tokens。
    pub max_tokens: u32,
}

impl Component for ModelRef {}

/// ToolPolicyRef (per SRS §15 REQ-ECS-007): 不持有 Tool 实例。
#[derive(Debug, Clone)]
pub struct ToolPolicyRef {
    /// Policy ID。
    pub policy_id: PolicyId,
    /// 工具白名单。
    pub tool_allowlist: Vec<String>,
    /// 速率限制。
    pub rate_limit: Option<RateLimit>,
}

impl Component for ToolPolicyRef {}

/// McpPolicyRef (per 03-DD §2.1): MCP 权限引用。
#[derive(Debug, Clone)]
pub struct McpPolicyRef {
    /// Policy ID。
    pub policy_id: PolicyId,
    /// MCP server 白名单。
    pub server_allowlist: Vec<String>,
}

impl Component for McpPolicyRef {}

/// PermissionRef (per SRS §16 REQ-ECS-008): ACL 引用, 不复制完整 ACL。
#[derive(Debug, Clone, Copy)]
pub struct PermissionRef {
    /// ACL ID。
    pub acl_id: AclId,
    /// Tenant ID。
    pub tenant_id: TenantId,
}

impl Component for PermissionRef {}

/// TokenBudget (per SRS §17 REQ-ECS-009)。
#[derive(Debug, Clone, Copy)]
pub struct TokenBudget {
    /// 最大 context tokens。
    pub max_context_tokens: u32,
    /// 最大 output tokens。
    pub max_output_tokens: u32,
    /// 剩余 tokens。
    pub remaining_tokens: u32,
    /// 成本预算 (cents)。
    pub cost_budget_cents: u32,
}

impl Component for TokenBudget {}

impl Default for TokenBudget {
    fn default() -> Self {
        Self {
            max_context_tokens: 8000,
            max_output_tokens: 2048,
            remaining_tokens: 8000,
            cost_budget_cents: 0,
        }
    }
}

/// MailboxRef (per SRS §19 REQ-ECS-011): 不存大消息。
#[derive(Debug, Clone, Copy)]
pub struct MailboxRef {
    /// Mailbox ID。
    pub mailbox_id: MailboxId,
    /// 未读数。
    pub unread_count: u32,
    /// 上限 (bounded, per SRS §44)。
    pub max_size: u32,
}

impl Component for MailboxRef {}

/// WorkflowRef (per SRS §20 REQ-ECS-012): 长生命周期 Workflow 状态不全嵌入 Agent。
#[derive(Debug, Clone, Copy)]
pub struct WorkflowRef {
    /// Workflow ID。
    pub workflow_id: WorkflowId,
    /// 当前步。
    pub step: u32,
    /// 总步数。
    pub total_steps: u32,
    /// Checkpoint ID (per SRS §42)。
    pub checkpoint_id: Option<CheckpointId>,
}

impl Component for WorkflowRef {}

/// 非法 ToolPolicyRef 的 RateLimit 错误 (0 per_sec)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RateLimitError;

impl fmt::Display for RateLimitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "非法 RateLimit: per_sec 必须 > 0")
    }
}

impl std::error::Error for RateLimitError {}

impl RateLimit {
    /// 校验 (per SRS §101 Fail Safe: 非法即拒)。
    pub fn validate(&self) -> Result<(), RateLimitError> {
        if self.per_sec == 0 {
            return Err(RateLimitError);
        }
        Ok(())
    }
}
