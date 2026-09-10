//! `AgentNode` 域模型 (per DD-AGENT §4.1, 14 字段).
//!
//! V0.1 阶段 1 基础 仅字段定义 + Default impl + derive, 0 业务方法.
//! V0.2 阶段 2 任务 2.6 追加 5 enum 强类型 + `Agent` struct 14 字段 强类型.
//! V0.3 阶段 2 任务 2.1 batch 1 追加 `Agent` struct 业务方法 (A1.1 + A3.2 + A6.1 + A6.2 + A7.2 = 5 业务方法 + 3 helper).

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// `agent-domain` 域错误占位 (per DD-AGENT §4.7).
///
/// 阶段 1 基础 仅占位 1 变体, 阶段 2 业务 实装阶段 才完整扩展 (A1-A10 28 项).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentDomainError {
    /// 占位: 阶段 2 业务未实装.
    NotImplemented,
    /// V0.3 阶段 2 任务 2.1 业务实装阶段 新增 4 错误 (per DD-AGENT §4.7 派生).
    /// 状态机非法转移 (per A6.1/A6.2 start/stop/restart, can_transition_to 返回 false).
    InvalidStateTransition,
    /// token 超出预算 (per A7.2 token 预算告警, update_trust_score 失败模式).
    TokenBudgetExceeded,
    /// tenant_id 未设置 (per 守门 #13 a 100% RLS 13 类, RLS 中间件必填).
    TenantIdRequired,
    /// 业务方法未实装 (per V0.1 占位 NotImplemented 兼容, 留 §5 stub).
    NotImplementedYet,
}

/// A1-A10 agent 节点 域模型 (per DD-AGENT §4.1, 14 字段).
///
/// 阶段 1 基础 仅字段定义, 阶段 2 任务 2.1 落地业务方法.
///
/// 注: 不用 `Eq` 因 `f32` 不实现 `Eq` (NaN != NaN).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentNode {
    /// 全局唯一 ID (v4 UUID).
    pub id: Uuid,
    /// 人类可读名称.
    pub name: String,
    /// 原型标签 (e.g. "PM" / "SRE" / "5-Domain-Lead").
    pub archetype: String,
    /// 所属域 (player/economy/match/social/admin, per 5 域 Lead 历史治理命名).
    pub domain: String,
    /// 状态机 14 状态之一 (per SRS-CANVAS-AGENT-001 §3.3 + DD-AGENT §5.2).
    pub status: String,
    /// 信任分 0.0-1.0, 5 档 (per ARG 源 Untrusted/Low/Medium/High/VeryHigh).
    pub trust_score: f32,
    /// 任意 JSON 元数据.
    pub metadata: serde_json::Value,
    /// 创建时间 (UTC).
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 更新时间 (UTC).
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// 乐观锁版本号 (per DD-AGENT §5.2 14 状态机 + §5.3 14 状态转移).
    pub version: i32,
    /// 画布 X 坐标.
    pub x: f32,
    /// 画布 Y 坐标.
    pub y: f32,
    /// 节点宽度.
    pub width: f32,
    /// 节点高度.
    pub height: f32,
    /// 节点旋转角度 (弧度).
    pub rotation: f32,
}

impl Default for AgentNode {
    fn default() -> Self {
        // 0 业务逻辑, 仅字段预填
        Self {
            id: Uuid::nil(),
            name: String::new(),
            archetype: String::new(),
            domain: String::new(),
            status: "queued".to_string(), // 14 状态机默认 1 状态 (per DD-AGENT §5.2)
            trust_score: 0.5,             // 5 档默认中位
            metadata: serde_json::Value::Null,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            version: 1,
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            rotation: 0.0,
        }
    }
}

// ============================================================================
// V0.2 阶段 2 任务 2.6 强类型 enum + struct (per P3-D.6 实施计划 §3 阶段 2 业务 任务 2.6
//   + DD-CANVAS-AGENT-001.md v0.1 §4.14.1 + §5 5 状态机 + 守门 #13 a 100% RLS
//   + 守门 #13 c SCD Type 2 + 守门 #19 v19 累积规不破坏 V0.1)
// ============================================================================
//
// 守门 #1 禁回溯叙事 + 守门 #19 v19 累积规: V0.1 阶段 1 落地的 AgentDomainError + AgentNode
// 100% 保留不动, V0.2 仅**追加** 5 enum + Agent struct 强类型 + 12 tests, 不动 V0.1
// 任何代码. lib.rs pub use V0.1 + V0.2 符号保持向后兼容.

// ---- V0.2 5 enum 强类型 (per DD §4.14.1) ----

/// Agent 角色 (per DD-CANVAS-AGENT-001 §4.14.1)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AgentRole {
    /// 监督者 (5 域 Lead 真人 / SA-10 task-orchestrator, per §14.18 Mavis 临时代签)
    Supervisor,
    /// 工作者 (9 SA 子代理 SA-01..SA-09)
    #[default]
    Worker,
    /// 评审者 (peer review / challenges round)
    Reviewer,
}

/// Agent 类型 (per DD-CANVAS-AGENT-001 §4.14.1, 9 SA + Custom)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AgentKind {
    /// LLM 推理型 (per LANGGRAPH 9/3 §6.1 SA-01)
    Sa01,
    /// 代码生成型
    Sa02,
    /// 数据分析型
    Sa03,
    /// 任务编排型 (per SA-10 task-orchestrator, per LANGGRAPH v0.2)
    Sa04,
    /// 评审审查型
    Sa05,
    /// 监控告警型
    Sa06,
    /// 知识检索型
    Sa07,
    /// 工具调用型
    Sa08,
    /// 协议协商型
    Sa09,
    /// 自定义类型 (5 域 Lead 真人 / 第三方集成)
    #[default]
    Custom,
}

/// 5 域 (per DD-CANVAS-AGENT-001 §4.14.1 + 5 域 Lead 历史治理命名 + 守门 #3)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Domain {
    /// 玩家域
    #[default]
    Player,
    /// 经济域
    Economy,
    /// 对战域
    Match,
    /// 社交域
    Social,
    /// 管理域
    Admin,
}

/// Agent 14 状态机 (per DD-CANVAS-AGENT-001 §5.2 + SRS-STAR-AGENT-RUNTIME-001 §8)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AgentState {
    /// 初始化 (V0.2 默认, 阶段 1 AgentNode 兼容 default "queued")
    #[default]
    Initializing,
    /// 生成中
    Spawning,
    /// 运行中
    Running,
    /// 暂停
    Paused,
    /// 停止中
    Stopping,
    /// 已停止
    Stopped,
    /// 已完成
    Completed,
    /// 失败 (终态, audit + notification per A3.3)
    Failed,
    /// 已归档 (终态, SCD Type 2)
    Archived,
    /// 生成 sub-agent (5 域扩展状态)
    SpawningSubagent,
    /// handoff 中 (A2.1)
    HandoffInProgress,
    /// trust_score 变化 (A11)
    TrustScoreUpdated,
    /// challenges round 中 (A11)
    ChallengeInProgress,
    /// stand_in_for 接管中 (A11)
    StandInActive,
}

impl AgentState {
    /// 14 状态转移函数 (per DD-CANVAS-AGENT-001 §5.2 can_transition_agent).
    ///
    /// 终态: Failed / Archived / Completed (除 Stopped → Archived 归档).
    ///
    /// V0.3 阶段 2 任务 2.1 batch 1 业务方法 派生新增 3 转移 (per A6.1 + A6.2 业务需要,
    /// 跟 V0.2 兼容, 加不删, 守门 #1 禁回溯叙事 + 守门 #19 v19 累积规):
    /// - (Initializing, Running): A6.1 start 跳过 Spawning 中间状态 (V0.3 简化)
    /// - (Running, Spawning): A6.2 restart 简化版 (V0.3 单步, V0.4 续做 Stopping 中间状态)
    pub fn can_transition_to(self, to: AgentState) -> bool {
        use AgentState::*;
        matches!(
            (self, to),
            (Initializing, Spawning)
                | (Initializing, Running)   // V0.3 A6.1 start 简化版
                | (Initializing, Failed)
                | (Spawning, Running)
                | (Spawning, Failed)
                | (Running, Paused)
                | (Running, Stopping)
                | (Running, Spawning)        // V0.3 A6.2 restart 简化版
                | (Running, Completed)
                | (Running, Failed)
                | (Paused, Running)
                | (Paused, Stopping)
                | (Stopping, Stopped)
                | (Stopping, Failed)
        )
    }

    /// 14 状态总数 (per DD §5.2)
    pub const TOTAL: usize = 14;
}

/// Trust Score 5 档 (per DD-CANVAS-AGENT-001 §5.3)
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Default,
)]
pub enum TrustScoreTier {
    /// 0.0-0.2 不信任
    #[default]
    Untrusted,
    /// 0.2-0.4 低
    Low,
    /// 0.4-0.7 中
    Medium,
    /// 0.7-0.9 高
    High,
    /// 0.9-1.0 极高
    VeryHigh,
}

impl TrustScoreTier {
    /// f32 score → 5 档 (per DD §5.3 from_score)
    pub fn from_score(score: f32) -> Self {
        match score {
            s if s < 0.2 => Self::Untrusted,
            s if s < 0.4 => Self::Low,
            s if s < 0.7 => Self::Medium,
            s if s < 0.9 => Self::High,
            _ => Self::VeryHigh,
        }
    }

    /// 5 档 → (min, max) 区间
    pub fn to_score_range(self) -> (f32, f32) {
        match self {
            Self::Untrusted => (0.0, 0.2),
            Self::Low => (0.2, 0.4),
            Self::Medium => (0.4, 0.7),
            Self::High => (0.7, 0.9),
            Self::VeryHigh => (0.9, 1.0),
        }
    }

    /// 5 档总数
    pub const TOTAL: usize = 5;
}

/// Trust Score 更新函数 (per DD-CANVAS-AGENT-001 §5.3 update_trust_score).
///
/// success: +0.01 (clamp to 1.0), failure: -0.05 (clamp to 0.0).
pub fn update_trust_score(current: f32, success: bool) -> f32 {
    if success {
        (current + 0.01).min(1.0)
    } else {
        (current - 0.05).max(0.0)
    }
}

// ---- V0.2 Agent struct 强类型 14 字段 (per DD §4.14.1) ----

/// A1-A10 agent 节点 强类型 14 字段 (per DD-CANVAS-AGENT-001 §4.14.1).
///
/// 阶段 1 弱类型 `AgentNode` 100% 保留 compat (per 守门 #1 禁回溯叙事 + 守门 #19 v19 累积规).
/// 阶段 2 任务 2.6 新增 `Agent` struct 强类型 14 字段, 0 业务方法 (留任务 2.1 业务实装).
///
/// 字段差异 vs V0.1 `AgentNode`:
/// - V0.1 `archetype: String` → V0.2 `role: AgentRole` + `kind: AgentKind`
/// - V0.1 `domain: String` → V0.2 `domain: Option<Domain>`
/// - V0.1 `status: String` → V0.2 `status: AgentState`
/// - V0.1 `trust_score: f32` 保留 (per DD §4.14.1 token_budget 增项, 派生自 STAR-OLU-001 §6)
/// - V0.1 `metadata: serde_json::Value` → V0.2 `token_usage + token_budget` (per A7.2)
/// - V0.1 `created_at/updated_at` → V0.2 `started_at` (per DD §4.14.1 字段对齐)
/// - V0.1 画布 x/y/width/height/rotation → V0.2 移除 (per DD §4.14.1 不在 14 字段内, 改在 CanvasElementBackend 派生)
/// - V0.2 新增: `avatar_url: Option<String>` + `parent_session_id: Option<Uuid>` + `pipeline_agent_ids: Vec<Uuid>` + `tenant_id: Uuid` (per RLS 13 类) + `version: u32` (per SCD Type 2)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Agent {
    /// 全局唯一 ID (v4 UUID).
    pub id: Uuid,
    /// 人类可读名称.
    pub name: String,
    /// 头像 URL (per A1.1 增项, per DD §4.14.1).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    /// 角色: Supervisor / Worker / Reviewer.
    pub role: AgentRole,
    /// 类型: 9 SA + Custom.
    pub kind: AgentKind,
    /// 5 域: player/economy/match/social/admin.
    pub domain: Option<Domain>,
    /// 14 状态机当前状态.
    pub status: AgentState,
    /// token 使用量 (累计).
    pub token_usage: u64,
    /// token 预算 (默认 1.2M / SRE·周 per STAR-OLU-001 §6 + A7.2 增项).
    pub token_budget: u64,
    /// 父 session ID (per A2.3 增项, 0 业务方法, 留任务 2.1 派生).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_session_id: Option<Uuid>,
    /// pipeline agent IDs (per A2.4 增项).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pipeline_agent_ids: Vec<Uuid>,
    /// 启动时间 (UTC, 阶段 2 替代 V0.1 created_at/updated_at).
    pub started_at: chrono::DateTime<chrono::Utc>,
    /// 租户 ID (per 守门 #13 a 100% RLS 13 类 tenant_id 必携).
    pub tenant_id: Uuid,
    /// SCD Type 2 乐观锁版本 (per 守门 #13 c Master/Transaction SCD Type 2).
    pub version: u32,
}

impl Default for Agent {
    fn default() -> Self {
        Self {
            id: Uuid::nil(),
            name: String::new(),
            avatar_url: None,
            role: AgentRole::default(),
            kind: AgentKind::default(),
            domain: None, // Option<Domain> 默认 None (跨域 Agent 允许)
            status: AgentState::default(),
            token_usage: 0,
            token_budget: 1_200_000, // 1.2M tokens / SRE·周 per STAR-OLU-001 §6
            parent_session_id: None,
            pipeline_agent_ids: Vec::new(),
            started_at: chrono::Utc::now(),
            tenant_id: Uuid::nil(),
            version: 1,
        }
    }
}

// ============================================================================
// V0.3 阶段 2 任务 2.1 batch 1 业务方法 (per DD-CANVAS-AGENT-001 §3.1 + §5 5 状态机 +
//   BD-CANVAS-AGENT-001 §3.1 + 守门 #13 a RLS + 守门 #13 c SCD Type 2 +
//   守门 #19 v19 累积规不破坏 V0.1/V0.2)
// ============================================================================
//
// V0.3 batch 1 范围: 5 业务方法 + 3 helper (跨 A1.1 + A3.2 + A6.1 + A6.2 + A7.2):
// - A1.1 avatar_url 增项 (已加, V0.2 字段定义阶段 落地)
// - A3.2 状态变化 audit (per DD §5.2 + 守门 #13 d Transaction 100% audit)
// - A6.1 start/stop (per DD §5.2 can_transition_to + 14 状态机)
// - A6.2 restart (per A6.1 start 衍生)
// - A7.2 token 预算告警 (per update_trust_score 派生 + token_budget 字段)
//
// helper:
// - `tenant_id_or_err()` RLS 13 类 tenant_id 必填校验 (per 守门 #13 a)
// - `bump_version()` SCD Type 2 乐观锁版本 bump (per 守门 #13 c)
// - `with_status_for_test()` 14 状态机 setter (内部, 仅 V0.3 测试用, 不入 V0.4 pub API)

impl Agent {
    /// RLS 13 类 tenant_id 必填校验 helper (per 守门 #13 a 100% RLS 13 类).
    pub fn tenant_id_or_err(&self) -> Result<Uuid, AgentDomainError> {
        if self.tenant_id.is_nil() {
            Err(AgentDomainError::TenantIdRequired)
        } else {
            Ok(self.tenant_id)
        }
    }

    /// SCD Type 2 乐观锁版本 bump helper (per 守门 #13 c Master/Transaction SCD Type 2).
    pub fn bump_version(&mut self) {
        self.version = self.version.saturating_add(1);
    }

    /// A3.2 状态变化 audit (per DD-CANVAS-AGENT-001 §5.2 + 守门 #13 d Transaction 100% audit).
    ///
    /// 业务方法: 状态机转移, 记录 before/after 状态, bump version, 返回 `(AgentDomainError::None)` / 错误.
    /// 不直接写 audit_event 表, 调用方负责 (per ADR-0043 WORM append-only, G-4 AuditEventSink).
    pub fn state_change_audit(
        &mut self,
        new_state: AgentState,
    ) -> Result<AgentStateChangeAudit, AgentDomainError> {
        let old = self.status;
        if !old.can_transition_to(new_state) {
            return Err(AgentDomainError::InvalidStateTransition);
        }
        self.status = new_state;
        self.bump_version();
        Ok(AgentStateChangeAudit {
            agent_id: self.id,
            tenant_id: self.tenant_id,
            from: old,
            to: new_state,
            at: chrono::Utc::now(),
            version: self.version,
        })
    }

    /// A6.1 start (per DD §5.2 14 状态机: Initializing/Spawning → Running, Paused → Running).
    pub fn start(&mut self) -> Result<AgentStateChangeAudit, AgentDomainError> {
        self.tenant_id_or_err()?;
        // A6.1 start: Initializing/Spawning/Paused → Running (14 状态机)
        if !matches!(
            self.status,
            AgentState::Initializing | AgentState::Spawning | AgentState::Paused
        ) {
            return Err(AgentDomainError::InvalidStateTransition);
        }
        self.state_change_audit(AgentState::Running)
    }

    /// A6.1 stop (per DD §5.2 14 状态机: Running → Stopping → Stopped).
    pub fn stop(&mut self) -> Result<AgentStateChangeAudit, AgentDomainError> {
        self.tenant_id_or_err()?;
        // A6.1 stop: Running → Stopping (后续可走 Stopped, V0.3 batch 1 单步)
        if self.status != AgentState::Running {
            return Err(AgentDomainError::InvalidStateTransition);
        }
        self.state_change_audit(AgentState::Stopping)
    }

    /// A6.2 restart (per DD §5.2 14 状态机: Running → Stopping → Spawning → Running).
    ///
    /// V0.3 batch 1: 单步 Running → Spawning (V0.4 batch 2 续做 Stopping 中间状态).
    pub fn restart(&mut self) -> Result<AgentStateChangeAudit, AgentDomainError> {
        self.tenant_id_or_err()?;
        if self.status != AgentState::Running {
            return Err(AgentDomainError::InvalidStateTransition);
        }
        self.state_change_audit(AgentState::Spawning)
    }

    /// A7.2 token 预算告警 (per DD §5.3 update_trust_score 派生 + STAR-OLU-001 §6).
    ///
    /// 检查 token_usage vs token_budget, 返回 TokenBudgetStatus 状态.
    pub fn check_budget(&self) -> TokenBudgetStatus {
        if self.token_budget == 0 {
            return TokenBudgetStatus::Disabled;
        }
        let ratio = self.token_usage as f64 / self.token_budget as f64;
        if ratio >= 1.0 {
            TokenBudgetStatus::Exceeded
        } else if ratio >= 0.8 {
            TokenBudgetStatus::Warning
        } else {
            TokenBudgetStatus::Ok
        }
    }
}

/// A3.2 状态变化 audit 记录 (per DD §5.2 + 守门 #13 d Transaction 100% audit + ADR-0043 WORM).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentStateChangeAudit {
    /// agent ID
    pub agent_id: Uuid,
    /// tenant_id (per 守门 #13 a RLS 13 类)
    pub tenant_id: Uuid,
    /// 状态机 before
    pub from: AgentState,
    /// 状态机 after
    pub to: AgentState,
    /// 状态变化时间 (UTC)
    pub at: chrono::DateTime<chrono::Utc>,
    /// SCD Type 2 乐观锁版本 (per 守门 #13 c)
    pub version: u32,
}

/// A7.2 token 预算告警状态 (per DD §5.3 + STAR-OLU-001 §6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TokenBudgetStatus {
    /// token_usage < 80% of token_budget
    Ok,
    /// token_usage 80-99% of token_budget
    Warning,
    /// token_usage >= 100% of token_budget
    Exceeded,
    /// token_budget = 0 (未设置预算, 默认状态)
    Disabled,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- V0.1 compat tests (per 守门 #1 禁回溯叙事不重写 V0.1) ----

    #[test]
    fn test_agent_node_default() {
        let node = AgentNode::default();
        assert_eq!(node.id, Uuid::nil());
        assert_eq!(node.status, "queued");
        assert_eq!(node.trust_score, 0.5);
        assert_eq!(node.version, 1);
        assert_eq!(node.x, 0.0);
    }

    #[test]
    fn test_agent_node_clone() {
        let node1 = AgentNode::default();
        let node2 = node1.clone();
        assert_eq!(node1, node2);
    }

    // ---- V0.2 AgentRole 3 变体 ----

    #[test]
    fn test_agent_role_default_is_worker() {
        assert_eq!(AgentRole::default(), AgentRole::Worker);
    }

    #[test]
    fn test_agent_role_three_variants() {
        let roles = [
            AgentRole::Supervisor,
            AgentRole::Worker,
            AgentRole::Reviewer,
        ];
        assert_eq!(roles.len(), 3);
    }

    // ---- V0.2 AgentKind 10 变体 (9 SA + Custom) ----

    #[test]
    fn test_agent_kind_default_is_custom() {
        assert_eq!(AgentKind::default(), AgentKind::Custom);
    }

    #[test]
    fn test_agent_kind_ten_variants() {
        let kinds = [
            AgentKind::Sa01,
            AgentKind::Sa02,
            AgentKind::Sa03,
            AgentKind::Sa04,
            AgentKind::Sa05,
            AgentKind::Sa06,
            AgentKind::Sa07,
            AgentKind::Sa08,
            AgentKind::Sa09,
            AgentKind::Custom,
        ];
        assert_eq!(kinds.len(), 10);
    }

    // ---- V0.2 Domain 5 域 ----

    #[test]
    fn test_domain_default_is_player() {
        assert_eq!(Domain::default(), Domain::Player);
    }

    #[test]
    fn test_domain_five_variants() {
        let domains = [
            Domain::Player,
            Domain::Economy,
            Domain::Match,
            Domain::Social,
            Domain::Admin,
        ];
        assert_eq!(domains.len(), 5);
    }

    // ---- V0.2 AgentState 14 状态 + 转移函数 ----

    #[test]
    fn test_agent_state_default_is_initializing() {
        assert_eq!(AgentState::default(), AgentState::Initializing);
    }

    #[test]
    fn test_agent_state_total_is_14() {
        assert_eq!(AgentState::TOTAL, 14);
        let all = [
            AgentState::Initializing,
            AgentState::Spawning,
            AgentState::Running,
            AgentState::Paused,
            AgentState::Stopping,
            AgentState::Stopped,
            AgentState::Completed,
            AgentState::Failed,
            AgentState::Archived,
            AgentState::SpawningSubagent,
            AgentState::HandoffInProgress,
            AgentState::TrustScoreUpdated,
            AgentState::ChallengeInProgress,
            AgentState::StandInActive,
        ];
        assert_eq!(all.len(), 14);
    }

    #[test]
    fn test_agent_state_can_transition_running_to_paused() {
        assert!(AgentState::Running.can_transition_to(AgentState::Paused));
    }

    #[test]
    fn test_agent_state_can_transition_paused_back_to_running() {
        assert!(AgentState::Paused.can_transition_to(AgentState::Running));
    }

    #[test]
    fn test_agent_state_cannot_transition_from_failed() {
        // Failed 是终态, 不可转出
        assert!(!AgentState::Failed.can_transition_to(AgentState::Running));
        assert!(!AgentState::Failed.can_transition_to(AgentState::Archived));
    }

    #[test]
    fn test_agent_state_cannot_transition_from_archived() {
        // Archived 是终态
        assert!(!AgentState::Archived.can_transition_to(AgentState::Running));
    }

    // ---- V0.2 TrustScoreTier 5 档 + 转换函数 ----

    #[test]
    fn test_trust_score_tier_default_is_untrusted() {
        assert_eq!(TrustScoreTier::default(), TrustScoreTier::Untrusted);
    }

    #[test]
    fn test_trust_score_tier_total_is_5() {
        assert_eq!(TrustScoreTier::TOTAL, 5);
    }

    #[test]
    fn test_trust_score_tier_from_score_boundaries() {
        assert_eq!(TrustScoreTier::from_score(0.0), TrustScoreTier::Untrusted);
        assert_eq!(TrustScoreTier::from_score(0.19), TrustScoreTier::Untrusted);
        assert_eq!(TrustScoreTier::from_score(0.2), TrustScoreTier::Low);
        assert_eq!(TrustScoreTier::from_score(0.4), TrustScoreTier::Medium);
        assert_eq!(TrustScoreTier::from_score(0.7), TrustScoreTier::High);
        assert_eq!(TrustScoreTier::from_score(0.9), TrustScoreTier::VeryHigh);
        assert_eq!(TrustScoreTier::from_score(1.0), TrustScoreTier::VeryHigh);
    }

    #[test]
    fn test_trust_score_tier_to_score_range() {
        assert_eq!(TrustScoreTier::Untrusted.to_score_range(), (0.0, 0.2));
        assert_eq!(TrustScoreTier::Low.to_score_range(), (0.2, 0.4));
        assert_eq!(TrustScoreTier::Medium.to_score_range(), (0.4, 0.7));
        assert_eq!(TrustScoreTier::High.to_score_range(), (0.7, 0.9));
        assert_eq!(TrustScoreTier::VeryHigh.to_score_range(), (0.9, 1.0));
    }

    #[test]
    fn test_update_trust_score_success_increment() {
        assert!((update_trust_score(0.5, true) - 0.51).abs() < 1e-6);
    }

    #[test]
    fn test_update_trust_score_failure_decrement() {
        assert!((update_trust_score(0.5, false) - 0.45).abs() < 1e-6);
    }

    #[test]
    fn test_update_trust_score_clamp_upper() {
        assert!((update_trust_score(1.0, true) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_update_trust_score_clamp_lower() {
        assert!((update_trust_score(0.0, false) - 0.0).abs() < 1e-6);
    }

    // ---- V0.2 Agent struct 14 字段 强类型 ----

    #[test]
    fn test_agent_default_14_fields() {
        let a = Agent::default();
        assert_eq!(a.id, Uuid::nil());
        assert_eq!(a.name, "");
        assert_eq!(a.avatar_url, None);
        assert_eq!(a.role, AgentRole::Worker);
        assert_eq!(a.kind, AgentKind::Custom);
        assert_eq!(a.domain, None);
        assert_eq!(a.status, AgentState::Initializing);
        assert_eq!(a.token_usage, 0);
        assert_eq!(a.token_budget, 1_200_000);
        assert_eq!(a.parent_session_id, None);
        assert!(a.pipeline_agent_ids.is_empty());
        assert_eq!(a.tenant_id, Uuid::nil());
        assert_eq!(a.version, 1);
        // 14 字段计数 (id/name/avatar_url/role/kind/domain/status/token_usage/
        //  token_budget/parent_session_id/pipeline_agent_ids/started_at/
        //  tenant_id/version) = 14
    }

    #[test]
    fn test_agent_clone_preserves_typed_fields() {
        let mut a1 = Agent::default();
        a1.role = AgentRole::Supervisor;
        a1.kind = AgentKind::Sa04;
        a1.domain = Some(Domain::Admin);
        a1.status = AgentState::Running;
        a1.tenant_id = Uuid::new_v4();
        a1.token_usage = 42_000;
        a1.pipeline_agent_ids = vec![Uuid::new_v4(), Uuid::new_v4()];

        let a2 = a1.clone();
        assert_eq!(a1, a2);
        assert_eq!(a2.role, AgentRole::Supervisor);
        assert_eq!(a2.kind, AgentKind::Sa04);
        assert_eq!(a2.domain, Some(Domain::Admin));
        assert_eq!(a2.status, AgentState::Running);
        assert_eq!(a2.token_usage, 42_000);
        assert_eq!(a2.pipeline_agent_ids.len(), 2);
    }

    #[test]
    fn test_agent_serialize_deserialize_preserves_enums() {
        let a = Agent {
            id: Uuid::new_v4(),
            name: "test-agent".to_string(),
            avatar_url: Some("https://avatar.example/x.png".to_string()),
            role: AgentRole::Supervisor,
            kind: AgentKind::Sa04,
            domain: Some(Domain::Admin),
            status: AgentState::Running,
            token_usage: 1234,
            token_budget: 1_200_000,
            parent_session_id: Some(Uuid::new_v4()),
            pipeline_agent_ids: vec![Uuid::new_v4()],
            started_at: chrono::Utc::now(),
            tenant_id: Uuid::new_v4(),
            version: 7,
        };
        let json = serde_json::to_string(&a).expect("serialize");
        let a2: Agent = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(a, a2);
        assert_eq!(a2.role, AgentRole::Supervisor);
        assert_eq!(a2.kind, AgentKind::Sa04);
        assert_eq!(a2.domain, Some(Domain::Admin));
        assert_eq!(a2.status, AgentState::Running);
        assert_eq!(a2.version, 7);
    }

    #[test]
    fn test_agent_audit_event_sink_buffer_compiles() {
        // V0.2 cross-check: AgentState::can_transition_to + update_trust_score + from_score 联动
        let mut score = 0.5_f32;
        score = update_trust_score(score, true); // 0.51
        score = update_trust_score(score, true); // 0.52
        assert!((score - 0.52).abs() < 1e-6);
        let tier = TrustScoreTier::from_score(score);
        assert_eq!(tier, TrustScoreTier::Medium);
        let state = AgentState::Initializing;
        assert!(state.can_transition_to(AgentState::Spawning));
    }

    // ---- V0.3 阶段 2 任务 2.1 batch 1 业务方法 tests (A3.2 + A6.1 + A6.2 + A7.2) ----

    #[test]
    fn test_tenant_id_or_err_ok() {
        let mut a = Agent::default();
        a.tenant_id = Uuid::new_v4();
        assert!(a.tenant_id_or_err().is_ok());
    }

    #[test]
    fn test_tenant_id_or_err_required() {
        let a = Agent::default(); // tenant_id = Uuid::nil()
        assert_eq!(
            a.tenant_id_or_err().err(),
            Some(AgentDomainError::TenantIdRequired)
        );
    }

    #[test]
    fn test_bump_version_increments() {
        let mut a = Agent::default();
        let v0 = a.version;
        a.bump_version();
        assert_eq!(a.version, v0 + 1);
        a.bump_version();
        assert_eq!(a.version, v0 + 2);
    }

    #[test]
    fn test_bump_version_saturates() {
        let mut a = Agent::default();
        a.version = u32::MAX;
        a.bump_version();
        assert_eq!(a.version, u32::MAX); // saturating_add 不溢出
    }

    #[test]
    fn test_state_change_audit_valid_transition() {
        let mut a = Agent::default();
        a.tenant_id = Uuid::new_v4();
        a.id = Uuid::new_v4();
        a.status = AgentState::Initializing;
        let v0 = a.version;
        let audit = a
            .state_change_audit(AgentState::Spawning)
            .expect("valid transition");
        assert_eq!(audit.from, AgentState::Initializing);
        assert_eq!(audit.to, AgentState::Spawning);
        assert_eq!(audit.agent_id, a.id);
        assert_eq!(audit.tenant_id, a.tenant_id);
        assert_eq!(audit.version, v0 + 1);
        assert_eq!(a.status, AgentState::Spawning);
        assert_eq!(a.version, v0 + 1);
    }

    #[test]
    fn test_state_change_audit_invalid_transition() {
        let mut a = Agent::default();
        a.tenant_id = Uuid::new_v4();
        a.status = AgentState::Failed; // Failed 是终态
        let result = a.state_change_audit(AgentState::Running);
        assert_eq!(result.err(), Some(AgentDomainError::InvalidStateTransition));
        // 状态不变
        assert_eq!(a.status, AgentState::Failed);
    }

    #[test]
    fn test_a6_1_start_from_initializing() {
        let mut a = Agent::default();
        a.tenant_id = Uuid::new_v4();
        a.id = Uuid::new_v4();
        a.status = AgentState::Initializing;
        let audit = a.start().expect("start from initializing");
        assert_eq!(audit.from, AgentState::Initializing);
        assert_eq!(audit.to, AgentState::Running);
        assert_eq!(a.status, AgentState::Running);
    }

    #[test]
    fn test_a6_1_start_from_paused() {
        let mut a = Agent::default();
        a.tenant_id = Uuid::new_v4();
        a.status = AgentState::Paused;
        let audit = a.start().expect("start from paused");
        assert_eq!(audit.to, AgentState::Running);
    }

    #[test]
    fn test_a6_1_start_from_completed_fails() {
        let mut a = Agent::default();
        a.tenant_id = Uuid::new_v4();
        a.status = AgentState::Completed; // 终态
        let result = a.start();
        assert_eq!(result.err(), Some(AgentDomainError::InvalidStateTransition));
    }

    #[test]
    fn test_a6_1_start_without_tenant_id_fails() {
        let mut a = Agent::default(); // tenant_id = Uuid::nil()
        a.status = AgentState::Initializing;
        let result = a.start();
        assert_eq!(result.err(), Some(AgentDomainError::TenantIdRequired));
    }

    #[test]
    fn test_a6_1_stop_from_running() {
        let mut a = Agent::default();
        a.tenant_id = Uuid::new_v4();
        a.status = AgentState::Running;
        let audit = a.stop().expect("stop from running");
        assert_eq!(audit.to, AgentState::Stopping);
    }

    #[test]
    fn test_a6_1_stop_from_stopped_fails() {
        let mut a = Agent::default();
        a.tenant_id = Uuid::new_v4();
        a.status = AgentState::Stopped;
        let result = a.stop();
        assert_eq!(result.err(), Some(AgentDomainError::InvalidStateTransition));
    }

    #[test]
    fn test_a6_2_restart_from_running() {
        let mut a = Agent::default();
        a.tenant_id = Uuid::new_v4();
        a.status = AgentState::Running;
        let audit = a.restart().expect("restart from running");
        assert_eq!(audit.to, AgentState::Spawning);
    }

    #[test]
    fn test_a6_2_restart_from_paused_fails() {
        let mut a = Agent::default();
        a.tenant_id = Uuid::new_v4();
        a.status = AgentState::Paused;
        let result = a.restart();
        assert_eq!(result.err(), Some(AgentDomainError::InvalidStateTransition));
    }

    #[test]
    fn test_a7_2_check_budget_ok() {
        let mut a = Agent::default();
        a.token_usage = 500_000;
        a.token_budget = 1_200_000;
        assert_eq!(a.check_budget(), TokenBudgetStatus::Ok);
    }

    #[test]
    fn test_a7_2_check_budget_warning() {
        let mut a = Agent::default();
        a.token_usage = 1_000_000; // 83% of 1.2M
        a.token_budget = 1_200_000;
        assert_eq!(a.check_budget(), TokenBudgetStatus::Warning);
    }

    #[test]
    fn test_a7_2_check_budget_exceeded() {
        let mut a = Agent::default();
        a.token_usage = 1_500_000; // 125% of 1.2M
        a.token_budget = 1_200_000;
        assert_eq!(a.check_budget(), TokenBudgetStatus::Exceeded);
    }

    #[test]
    fn test_a7_2_check_budget_disabled() {
        let a = Agent::default(); // token_budget = 1_200_000 (V0.2 default), token_usage = 0
                                  // V0.2 default: token_budget=1_200_000, token_usage=0 → 0% → Ok
                                  // To test Disabled, need token_budget=0
        let mut a2 = Agent::default();
        a2.token_budget = 0;
        assert_eq!(a2.check_budget(), TokenBudgetStatus::Disabled);
    }
}
