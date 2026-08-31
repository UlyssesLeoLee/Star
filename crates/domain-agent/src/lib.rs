//! domain-agent crate
//!
//! 详细 spec: docs/specs/domain-agent-spec.md §4.2 / §24 Agent 适配层
//! 上游基本设计: docs/basic-design.md §2.1 / §4.2 / §7.4
//! 数据设计: docs/data-design.md §4.3 (`agent` schema)
//! API 设计: docs/api-design.md §3.7 (Agent CRUD + AgentSession 状态机)
//!
//! ## 职责
//!
//! Agent 一级领域对象 + AgentSession 14 状态机(§7.4,F-08 修正后)
//! - 适配 Codex / Claude Code / Gemini CLI / OpenAI Compatible / Local / Future
//! - Domain 不耦合 AI Provider SDK
//! - 12 强制点(§4.2.5,REQ-PERM-002)
//!
//! ## 关键不变量(INV-AGT-01~10)
//!
//! - INV-AGT-01:14 状态机严格迁移(§7.4 接口稳定承诺 #8)
//! - INV-AGT-02:1 AgentSession ↔ 1 Active Worktree(REQ-DEV-003)
//! - INV-AGT-03:1 Worktree → N AgentSession(REQ-DEV-002)
//! - INV-AGT-04:Domain 不耦合 Provider SDK(§4.2.4)
//! - INV-AGT-05:12 强制点由 Application 强加(§4.2.5,REQ-PERM-002)
//! - INV-AGT-07:Agent 必带 tenant_id(security §3.1)
//! - INV-AGT-08:CRASHED 由 Local Runtime 上报(§4.2.3)
//!
//! ## 状态机(14 状态,M07-AG-01 必做)
//!
//! - 启动:Created → Starting → Running
//! - Tool 循环:Running → WaitingTool → ToolRunning → ToolCompleted → Running
//! - Feedback 循环:Running → WaitingFeedback → FeedbackReceived → Running
//! - 验证:Running → Validating → Completed | Failed
//! - 异常:任何活跃态 → Aborted / Crashed / Timeout
//!
//! Lead 责任: agent Lead

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
pub use star_context::ActorContext;
use thiserror::Error;
use uuid::Uuid;

// =====================================================================
// ID 类型(本 crate 内强类型)
// =====================================================================

define_uuid_id!(AgentId);
define_uuid_id!(AgentSessionId);
define_uuid_id!(AgentPolicyTemplateId);
define_uuid_id!(TenantId);
define_uuid_id!(ProjectId);
define_uuid_id!(UserId);
define_uuid_id!(WorktreeId);
define_uuid_id!(WorkItemId);

// =====================================================================
// 14 状态机(§7.4,F-08 修正后)
// =====================================================================

/// AgentSession 14 状态(§7.4)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentSessionStatus {
    /// 0. 初始创建(尚未启动)
    Created,
    /// 1. 启动中(Local Runtime 接收指令)
    Starting,
    /// 2. 运行中(LLM 推理 / 工具调用循环)
    Running,
    /// 3. 等待工具执行(等子进程结果)
    WaitingTool,
    /// 4. 工具执行中
    ToolRunning,
    /// 5. 工具执行完成,等待 LLM 继续
    ToolCompleted,
    /// 6. 等待人工反馈(INV-N-07 关键突破抑制)
    WaitingFeedback,
    /// 7. 反馈已收到
    FeedbackReceived,
    /// 8. Validation 验证中
    Validating,
    /// 9. 验证通过,完成
    Completed,
    /// 10. 失败(Validation 不通过 / 业务错误)
    Failed,
    /// 11. 主动中止(用户/系统)
    Aborted,
    /// 12. 崩溃(INV-AGT-08:Local Runtime 上报)
    Crashed,
    /// 13. 超时
    Timeout,
}

impl AgentSessionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "CREATED",
            Self::Starting => "STARTING",
            Self::Running => "RUNNING",
            Self::WaitingTool => "WAITING_TOOL",
            Self::ToolRunning => "TOOL_RUNNING",
            Self::ToolCompleted => "TOOL_COMPLETED",
            Self::WaitingFeedback => "WAITING_FEEDBACK",
            Self::FeedbackReceived => "FEEDBACK_RECEIVED",
            Self::Validating => "VALIDATING",
            Self::Completed => "COMPLETED",
            Self::Failed => "FAILED",
            Self::Aborted => "ABORTED",
            Self::Crashed => "CRASHED",
            Self::Timeout => "TIMEOUT",
        }
    }

    /// 是否终态(Completed/Failed/Aborted/Crashed/Timeout)
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Completed | Self::Failed | Self::Aborted | Self::Crashed | Self::Timeout
        )
    }

    /// 是否活跃态(需要 Local Runtime 心跳)
    pub fn is_active(&self) -> bool {
        !self.is_terminal() && *self != Self::Created
    }

    /// 触发通知(§12,INV-N-07 关键突破点)
    pub fn triggers_notification(&self) -> bool {
        matches!(self, Self::Failed | Self::Crashed | Self::Timeout)
    }
}

/// 14 状态机迁移表(§7.4)
/// 严格迁移:任何不在表中的迁移返回 InvalidTransition
pub fn check_status_transition(
    from: AgentSessionStatus,
    to: AgentSessionStatus,
) -> Result<(), AgentError> {
    use AgentSessionStatus::*;
    let allowed = matches!(
        (from, to),
        // 启动序列
        (Created, Starting)
            | (Starting, Running)
            | (Starting, Crashed) // 启动失败崩溃
            // Tool 循环
            | (Running, WaitingTool)
            | (WaitingTool, ToolRunning)
            | (ToolRunning, ToolCompleted)
            | (ToolCompleted, Running)
            | (ToolRunning, Running) // 工具直接返回(无 completed 中间态)
            | (ToolRunning, Crashed) // 工具进程崩溃
            // Feedback 循环
            | (Running, WaitingFeedback)
            | (WaitingFeedback, FeedbackReceived)
            | (FeedbackReceived, Running)
            // 验证
            | (Running, Validating)
            | (Validating, Completed)
            | (Validating, Failed)
            // 异常路径
            | (Running, Failed)
            | (Running, Aborted)
            | (Running, Crashed)
            | (Running, Timeout)
            | (Starting, Failed)
            | (Starting, Aborted)
            | (Starting, Timeout)
            | (WaitingTool, Aborted)
            | (WaitingTool, Timeout)
            | (ToolRunning, Aborted)
            | (WaitingFeedback, Aborted)
            | (WaitingFeedback, Timeout)
            | (Validating, Aborted)
    );
    if !allowed {
        return Err(AgentError::InvalidTransition {
            from: from.as_str().to_string(),
            to: to.as_str().to_string(),
        });
    }
    Ok(())
}

// =====================================================================
// UUID 强类型 ID 宏
// =====================================================================

#[macro_export]
macro_rules! define_uuid_id {
    ($name:ident) => {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(pub Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }
            pub fn as_uuid(&self) -> Uuid {
                self.0
            }
        }

        impl From<Uuid> for $name {
            fn from(u: Uuid) -> Self {
                Self(u)
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

// =====================================================================
// 实体
// =====================================================================

/// Agent 适配器(§4.2.1,data-design §4.3)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: AgentId,
    pub tenant_id: TenantId,
    pub agent_type: AgentType,
    pub provider: String,
    pub version: String,
    pub capabilities: Vec<String>,
    pub policy_template_id: Option<AgentPolicyTemplateId>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Agent 类型(§4.2.1)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentType {
    Codex,
    ClaudeCode,
    GeminiCli,
    OpenAiCompatible,
    Local,
    Future,
}

impl AgentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::ClaudeCode => "claude_code",
            Self::GeminiCli => "gemini_cli",
            Self::OpenAiCompatible => "openai_compatible",
            Self::Local => "local",
            Self::Future => "future",
        }
    }
}

/// AgentSession 聚合根(§4.2.1,§24.1,data-design §4.3)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSession {
    pub id: AgentSessionId,
    pub tenant_id: TenantId,
    pub agent_id: AgentId,
    pub agent_type: AgentType,
    pub provider: String,
    pub version: String,
    pub worktree_id: WorktreeId,
    pub work_item_id: WorkItemId,
    pub status: AgentSessionStatus,
    pub intent: String,
    pub context_packet_id: Option<Uuid>,
    pub plan: Option<String>,
    pub decisions: Vec<Uuid>,
    pub tool_activity_summary: HashMap<String, u32>,
    pub change_set_ids: Vec<Uuid>,
    pub validation_result_ids: Vec<Uuid>,
    pub feedback_consumed_ids: Vec<Uuid>,
    pub result_summary: Option<String>,
    pub trace_reference: Option<String>,
    /// Token 统计(JSONB-like,简化为 HashMap)
    pub token_usage: HashMap<String, u64>,
    /// 成本摘要
    pub cost_summary: HashMap<String, f64>,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// 乐观锁
    pub lock_version: u32,
}

/// AgentPolicy(§4.2.5,§24.3,12 强制点)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPolicy {
    pub allowed_repositories: Vec<Uuid>,
    pub allowed_worktrees: Vec<WorktreeId>,
    pub allowed_paths: Vec<String>,
    pub forbidden_paths: Vec<String>,
    pub allowed_tools: Vec<String>,
    pub allowed_command_categories: Vec<String>,
    pub network_access: NetworkAccess,
    pub secret_access: SecretAccess,
    pub max_runtime_seconds: u32,
    pub max_context_tokens: u32,
    pub max_change_files: u32,
    pub max_change_lines: u32,
    pub require_review: bool,
    pub require_test: bool,
    pub require_approval: bool,
}

impl AgentPolicy {
    /// 默认保守策略
    pub fn conservative() -> Self {
        Self {
            allowed_repositories: vec![],
            allowed_worktrees: vec![],
            allowed_paths: vec![],
            forbidden_paths: vec!["**/.env".to_string(), "**/secrets/**".to_string()],
            allowed_tools: vec!["read_file".to_string(), "grep".to_string()],
            allowed_command_categories: vec!["query".to_string()],
            network_access: NetworkAccess::Deny,
            secret_access: SecretAccess::None,
            max_runtime_seconds: 300,
            max_context_tokens: 32_000,
            max_change_files: 10,
            max_change_lines: 500,
            require_review: true,
            require_test: true,
            require_approval: true,
        }
    }

    /// 12 强制点检查(REQ-PERM-002,§4.2.5)
    /// 由 Application 层在每次工具调用/操作前调用
    pub fn enforce(&self, check: &PolicyCheck) -> Result<(), AgentError> {
        // 1. path 必须在 allowed_paths(或 allowed_repositories 内)
        if !self.allowed_paths.is_empty() {
            if !self.allowed_paths.iter().any(|p| check.path.starts_with(p)) {
                return Err(AgentError::PolicyViolation(format!(
                    "path not allowed: {}",
                    check.path
                )));
            }
        }
        // 2. forbidden_paths 永远拒绝
        if self
            .forbidden_paths
            .iter()
            .any(|p| check.path.starts_with(p))
        {
            return Err(AgentError::PolicyViolation(format!(
                "forbidden path: {}",
                check.path
            )));
        }
        // 3. tool 必须在 allowed_tools
        if !self.allowed_tools.is_empty() && !self.allowed_tools.contains(&check.tool) {
            return Err(AgentError::PolicyViolation(format!(
                "tool not allowed: {}",
                check.tool
            )));
        }
        // 4. network 策略
        if matches!(self.network_access, NetworkAccess::Deny) && check.requires_network {
            return Err(AgentError::PolicyViolation(
                "network access denied".to_string(),
            ));
        }
        // 5. secret 策略
        if matches!(self.secret_access, SecretAccess::None) && check.requires_secret {
            return Err(AgentError::PolicyViolation(
                "secret access denied".to_string(),
            ));
        }
        // 6. runtime 上限
        if check.elapsed_seconds > self.max_runtime_seconds {
            return Err(AgentError::PolicyViolation(format!(
                "runtime exceeded: {} > {}",
                check.elapsed_seconds, self.max_runtime_seconds
            )));
        }
        // 7. change file 上限
        if check.changed_files > self.max_change_files {
            return Err(AgentError::PolicyViolation(format!(
                "change_files exceeded: {} > {}",
                check.changed_files, self.max_change_files
            )));
        }
        Ok(())
    }
}

/// 12 强制点的检查输入
#[derive(Debug, Clone)]
pub struct PolicyCheck {
    pub path: String,
    pub tool: String,
    pub requires_network: bool,
    pub requires_secret: bool,
    pub elapsed_seconds: u32,
    pub changed_files: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkAccess {
    Allow,
    Deny,
    Scoped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecretAccess {
    BrokerOnly,
    Scoped,
    None,
}

/// AgentPolicyTemplate(§4.2.5)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPolicyTemplate {
    pub id: AgentPolicyTemplateId,
    pub tenant_id: TenantId,
    pub name: String,
    pub policy: AgentPolicy,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// =====================================================================
// 错误
// =====================================================================

#[derive(Debug, Error)]
pub enum AgentError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("invalid state transition: {from} -> {to}")]
    InvalidTransition { from: String, to: String },
    #[error("permission denied")]
    PermissionDenied,
    #[error("cross-tenant access denied: tenant {0} vs required {1}")]
    CrossTenantDenied(TenantId, TenantId),
    #[error("policy violation: {0}")]
    PolicyViolation(String),
    #[error("agent already exists: {0}")]
    AgentAlreadyExists(AgentId),
    #[error("worktree mismatch: agent session must reference active worktree (INV-AGT-02)")]
    WorktreeMismatch,
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("internal: {0}")]
    Internal(String),
}

// =====================================================================
// 命令 / 查询 DTO
// =====================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterAgentCommand {
    pub tenant_id: TenantId,
    pub agent_type: AgentType,
    pub provider: String,
    pub version: String,
    pub capabilities: Vec<String>,
    pub policy_template_id: Option<AgentPolicyTemplateId>,
    pub actor_user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartSessionCommand {
    pub tenant_id: TenantId,
    pub agent_id: AgentId,
    pub worktree_id: WorktreeId,
    pub work_item_id: WorkItemId,
    pub intent: String,
    pub context_packet_id: Option<Uuid>,
    pub actor_user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionStatusCommand {
    pub tenant_id: TenantId,
    pub session_id: AgentSessionId,
    pub from: AgentSessionStatus,
    pub to: AgentSessionStatus,
    pub reason: Option<String>,
    pub actor_user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordToolActivityCommand {
    pub tenant_id: TenantId,
    pub session_id: AgentSessionId,
    pub tool: String,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitFeedbackCommand {
    pub tenant_id: TenantId,
    pub session_id: AgentSessionId,
    pub agent_instruction: String,
    pub actor_user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbortSessionCommand {
    pub tenant_id: TenantId,
    pub session_id: AgentSessionId,
    pub reason: String,
    pub actor_user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePolicyTemplateCommand {
    pub tenant_id: TenantId,
    pub name: String,
    pub policy: AgentPolicy,
    pub actor_user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSessionQuery {
    pub tenant_id: TenantId,
    pub session_id: AgentSessionId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListByWorktreeQuery {
    pub tenant_id: TenantId,
    pub worktree_id: WorktreeId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSessionSummary {
    pub id: AgentSessionId,
    pub tenant_id: TenantId,
    pub agent_id: AgentId,
    pub worktree_id: WorktreeId,
    pub status: AgentSessionStatus,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
}

// =====================================================================
// 端口(Port Traits)
// =====================================================================

#[async_trait]
pub trait AgentCommandPort: Send + Sync {
    async fn register_agent(
        &self,
        cmd: RegisterAgentCommand,
        actor: &ActorContext,
    ) -> Result<Agent, AgentError>;

    async fn start_session(
        &self,
        cmd: StartSessionCommand,
        actor: &ActorContext,
    ) -> Result<AgentSession, AgentError>;

    async fn transition_status(
        &self,
        cmd: TransitionStatusCommand,
        actor: &ActorContext,
    ) -> Result<AgentSession, AgentError>;

    async fn record_tool_activity(
        &self,
        cmd: RecordToolActivityCommand,
        actor: &ActorContext,
    ) -> Result<AgentSession, AgentError>;

    async fn submit_feedback(
        &self,
        cmd: SubmitFeedbackCommand,
        actor: &ActorContext,
    ) -> Result<AgentSession, AgentError>;

    async fn abort_session(
        &self,
        cmd: AbortSessionCommand,
        actor: &ActorContext,
    ) -> Result<AgentSession, AgentError>;

    async fn create_policy_template(
        &self,
        cmd: CreatePolicyTemplateCommand,
        actor: &ActorContext,
    ) -> Result<AgentPolicyTemplate, AgentError>;
}

#[async_trait]
pub trait AgentQueryPort: Send + Sync {
    async fn get_session(
        &self,
        q: GetSessionQuery,
        actor: &ActorContext,
    ) -> Result<AgentSession, AgentError>;

    async fn list_by_worktree(
        &self,
        q: ListByWorktreeQuery,
        actor: &ActorContext,
    ) -> Result<Vec<AgentSessionSummary>, AgentError>;

    async fn list_active_sessions(
        &self,
        tenant_id: TenantId,
        actor: &ActorContext,
    ) -> Result<Vec<AgentSessionSummary>, AgentError>;
}

#[async_trait]
pub trait AgentRepository: Send + Sync {
    async fn insert_agent(&self, agent: Agent) -> Result<(), AgentError>;
    async fn get_agent(&self, id: AgentId) -> Result<Agent, AgentError>;
    async fn list_agents(&self, tenant_id: TenantId) -> Result<Vec<Agent>, AgentError>;

    async fn insert_session(&self, session: AgentSession) -> Result<(), AgentError>;
    async fn get_session(&self, id: AgentSessionId) -> Result<AgentSession, AgentError>;
    async fn update_session(&self, session: AgentSession) -> Result<(), AgentError>;
    async fn list_sessions_by_worktree(
        &self,
        tenant_id: TenantId,
        worktree_id: WorktreeId,
    ) -> Result<Vec<AgentSession>, AgentError>;
    async fn list_active_sessions(
        &self,
        tenant_id: TenantId,
    ) -> Result<Vec<AgentSession>, AgentError>;

    async fn insert_policy_template(&self, template: AgentPolicyTemplate)
        -> Result<(), AgentError>;
    async fn get_policy_template(
        &self,
        id: AgentPolicyTemplateId,
    ) -> Result<AgentPolicyTemplate, AgentError>;
}

// =====================================================================
// InMemoryAgentService
// =====================================================================

pub struct InMemoryAgentService {
    repo: Arc<dyn AgentRepository>,
    agents: Arc<RwLock<HashMap<AgentId, Agent>>>,
    sessions: Arc<RwLock<HashMap<AgentSessionId, AgentSession>>>,
    policies: Arc<RwLock<HashMap<AgentPolicyTemplateId, AgentPolicyTemplate>>>,
}

impl InMemoryAgentService {
    pub fn new() -> Self {
        Self {
            repo: Arc::new(InMemoryAgentRepository::new()),
            agents: Arc::new(RwLock::new(HashMap::new())),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            policies: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    pub fn with_repo(repo: Arc<dyn AgentRepository>) -> Self {
        Self {
            repo,
            agents: Arc::new(RwLock::new(HashMap::new())),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            policies: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryAgentService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AgentCommandPort for InMemoryAgentService {
    async fn register_agent(
        &self,
        cmd: RegisterAgentCommand,
        actor: &ActorContext,
    ) -> Result<Agent, AgentError> {
        if TenantId::from(actor.tenant_id) != cmd.tenant_id {
            return Err(AgentError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                cmd.tenant_id,
            ));
        }
        if !actor.has_role("project_admin") && !actor.has_role("tenant_admin") {
            return Err(AgentError::PermissionDenied);
        }
        let now = Utc::now();
        let agent = Agent {
            id: AgentId::new(),
            tenant_id: cmd.tenant_id,
            agent_type: cmd.agent_type,
            provider: cmd.provider,
            version: cmd.version,
            capabilities: cmd.capabilities,
            policy_template_id: cmd.policy_template_id,
            enabled: true,
            created_at: now,
            updated_at: now,
        };
        // 同 tenant 下 name 不重复(name = provider:version)
        let key = format!("{}:{}", agent.provider, agent.version);
        let dup = {
            let agents = self.agents.read().unwrap();
            agents.values().any(|a| {
                format!("{}:{}", a.provider, a.version) == key && a.tenant_id == agent.tenant_id
            })
        };
        if dup {
            return Err(AgentError::Conflict(format!(
                "agent already exists for {}",
                key
            )));
        }
        self.repo.insert_agent(agent.clone()).await?;
        self.agents.write().unwrap().insert(agent.id, agent.clone());
        Ok(agent)
    }

    async fn start_session(
        &self,
        cmd: StartSessionCommand,
        actor: &ActorContext,
    ) -> Result<AgentSession, AgentError> {
        if TenantId::from(actor.tenant_id) != cmd.tenant_id {
            return Err(AgentError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                cmd.tenant_id,
            ));
        }
        let agent = {
            let agents = self.agents.read().unwrap();
            agents.get(&cmd.agent_id).cloned()
        }
        .ok_or_else(|| AgentError::NotFound(format!("agent:{}", cmd.agent_id.as_uuid())))?;
        if agent.tenant_id != cmd.tenant_id {
            return Err(AgentError::CrossTenantDenied(
                agent.tenant_id,
                cmd.tenant_id,
            ));
        }
        if !agent.enabled {
            return Err(AgentError::Conflict("agent disabled".to_string()));
        }
        let now = Utc::now();
        let session = AgentSession {
            id: AgentSessionId::new(),
            tenant_id: cmd.tenant_id,
            agent_id: cmd.agent_id,
            agent_type: agent.agent_type,
            provider: agent.provider.clone(),
            version: agent.version.clone(),
            worktree_id: cmd.worktree_id,
            work_item_id: cmd.work_item_id,
            status: AgentSessionStatus::Created,
            intent: cmd.intent,
            context_packet_id: cmd.context_packet_id,
            plan: None,
            decisions: vec![],
            tool_activity_summary: HashMap::new(),
            change_set_ids: vec![],
            validation_result_ids: vec![],
            feedback_consumed_ids: vec![],
            result_summary: None,
            trace_reference: None,
            token_usage: HashMap::new(),
            cost_summary: HashMap::new(),
            started_at: now,
            ended_at: None,
            created_at: now,
            updated_at: now,
            lock_version: 1,
        };
        self.repo.insert_session(session.clone()).await?;
        self.sessions
            .write()
            .unwrap()
            .insert(session.id, session.clone());
        Ok(session)
    }

    async fn transition_status(
        &self,
        cmd: TransitionStatusCommand,
        actor: &ActorContext,
    ) -> Result<AgentSession, AgentError> {
        if TenantId::from(actor.tenant_id) != cmd.tenant_id {
            return Err(AgentError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                cmd.tenant_id,
            ));
        }
        check_status_transition(cmd.from, cmd.to)?;
        let mut session = self
            .sessions
            .write()
            .unwrap()
            .get_mut(&cmd.session_id)
            .cloned()
            .ok_or_else(|| AgentError::NotFound(format!("session:{}", cmd.session_id.as_uuid())))?;
        if session.tenant_id != cmd.tenant_id {
            return Err(AgentError::CrossTenantDenied(
                session.tenant_id,
                cmd.tenant_id,
            ));
        }
        if session.status != cmd.from {
            return Err(AgentError::InvalidTransition {
                from: session.status.as_str().to_string(),
                to: cmd.to.as_str().to_string(),
            });
        }
        let now = Utc::now();
        session.status = cmd.to;
        if cmd.to.is_terminal() && session.ended_at.is_none() {
            session.ended_at = Some(now);
        }
        session.updated_at = now;
        session.lock_version += 1;
        self.repo.update_session(session.clone()).await?;
        self.sessions
            .write()
            .unwrap()
            .insert(session.id, session.clone());
        Ok(session)
    }

    async fn record_tool_activity(
        &self,
        cmd: RecordToolActivityCommand,
        actor: &ActorContext,
    ) -> Result<AgentSession, AgentError> {
        if TenantId::from(actor.tenant_id) != cmd.tenant_id {
            return Err(AgentError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                cmd.tenant_id,
            ));
        }
        let mut session = self
            .sessions
            .write()
            .unwrap()
            .get_mut(&cmd.session_id)
            .cloned()
            .ok_or_else(|| AgentError::NotFound(format!("session:{}", cmd.session_id.as_uuid())))?;
        if session.tenant_id != cmd.tenant_id {
            return Err(AgentError::CrossTenantDenied(
                session.tenant_id,
                cmd.tenant_id,
            ));
        }
        *session.tool_activity_summary.entry(cmd.tool).or_insert(0) += cmd.count;
        session.updated_at = Utc::now();
        session.lock_version += 1;
        self.repo.update_session(session.clone()).await?;
        self.sessions
            .write()
            .unwrap()
            .insert(session.id, session.clone());
        Ok(session)
    }

    async fn submit_feedback(
        &self,
        cmd: SubmitFeedbackCommand,
        actor: &ActorContext,
    ) -> Result<AgentSession, AgentError> {
        if TenantId::from(actor.tenant_id) != cmd.tenant_id {
            return Err(AgentError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                cmd.tenant_id,
            ));
        }
        // 隐式迁移:WaitingFeedback → FeedbackReceived
        let session = self
            .sessions
            .read()
            .unwrap()
            .get(&cmd.session_id)
            .cloned()
            .ok_or_else(|| AgentError::NotFound(format!("session:{}", cmd.session_id.as_uuid())))?;
        if session.status != AgentSessionStatus::WaitingFeedback {
            return Err(AgentError::InvalidTransition {
                from: session.status.as_str().to_string(),
                to: AgentSessionStatus::FeedbackReceived.as_str().to_string(),
            });
        }
        self.transition_status(
            TransitionStatusCommand {
                tenant_id: cmd.tenant_id,
                session_id: cmd.session_id,
                from: AgentSessionStatus::WaitingFeedback,
                to: AgentSessionStatus::FeedbackReceived,
                reason: Some(cmd.agent_instruction),
                actor_user_id: UserId::from(actor.user_id),
            },
            actor,
        )
        .await
    }

    async fn abort_session(
        &self,
        cmd: AbortSessionCommand,
        actor: &ActorContext,
    ) -> Result<AgentSession, AgentError> {
        let session = self
            .sessions
            .read()
            .unwrap()
            .get(&cmd.session_id)
            .cloned()
            .ok_or_else(|| AgentError::NotFound(format!("session:{}", cmd.session_id.as_uuid())))?;
        if session.tenant_id != cmd.tenant_id {
            return Err(AgentError::CrossTenantDenied(
                session.tenant_id,
                cmd.tenant_id,
            ));
        }
        if session.status.is_terminal() {
            return Err(AgentError::Conflict(format!(
                "session already terminal: {}",
                session.status.as_str()
            )));
        }
        self.transition_status(
            TransitionStatusCommand {
                tenant_id: cmd.tenant_id,
                session_id: cmd.session_id,
                from: session.status,
                to: AgentSessionStatus::Aborted,
                reason: Some(cmd.reason),
                actor_user_id: UserId::from(actor.user_id),
            },
            actor,
        )
        .await
    }

    async fn create_policy_template(
        &self,
        cmd: CreatePolicyTemplateCommand,
        actor: &ActorContext,
    ) -> Result<AgentPolicyTemplate, AgentError> {
        if TenantId::from(actor.tenant_id) != cmd.tenant_id {
            return Err(AgentError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                cmd.tenant_id,
            ));
        }
        if !actor.has_role("project_admin") && !actor.has_role("tenant_admin") {
            return Err(AgentError::PermissionDenied);
        }
        let now = Utc::now();
        let template = AgentPolicyTemplate {
            id: AgentPolicyTemplateId::new(),
            tenant_id: cmd.tenant_id,
            name: cmd.name,
            policy: cmd.policy,
            created_at: now,
            updated_at: now,
        };
        self.repo.insert_policy_template(template.clone()).await?;
        self.policies
            .write()
            .unwrap()
            .insert(template.id, template.clone());
        Ok(template)
    }
}

#[async_trait]
impl AgentQueryPort for InMemoryAgentService {
    async fn get_session(
        &self,
        q: GetSessionQuery,
        actor: &ActorContext,
    ) -> Result<AgentSession, AgentError> {
        if TenantId::from(actor.tenant_id) != q.tenant_id {
            return Err(AgentError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                q.tenant_id,
            ));
        }
        self.sessions
            .read()
            .unwrap()
            .get(&q.session_id)
            .cloned()
            .ok_or_else(|| AgentError::NotFound(format!("session:{}", q.session_id.as_uuid())))
    }

    async fn list_by_worktree(
        &self,
        q: ListByWorktreeQuery,
        actor: &ActorContext,
    ) -> Result<Vec<AgentSessionSummary>, AgentError> {
        if TenantId::from(actor.tenant_id) != q.tenant_id {
            return Err(AgentError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                q.tenant_id,
            ));
        }
        let sessions = self.sessions.read().unwrap();
        Ok(sessions
            .values()
            .filter(|s| s.tenant_id == q.tenant_id && s.worktree_id == q.worktree_id)
            .map(|s| AgentSessionSummary {
                id: s.id,
                tenant_id: s.tenant_id,
                agent_id: s.agent_id,
                worktree_id: s.worktree_id,
                status: s.status,
                started_at: s.started_at,
                ended_at: s.ended_at,
            })
            .collect())
    }

    async fn list_active_sessions(
        &self,
        tenant_id: TenantId,
        actor: &ActorContext,
    ) -> Result<Vec<AgentSessionSummary>, AgentError> {
        if TenantId::from(actor.tenant_id) != tenant_id {
            return Err(AgentError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                tenant_id,
            ));
        }
        let sessions = self.sessions.read().unwrap();
        Ok(sessions
            .values()
            .filter(|s| s.tenant_id == tenant_id && s.status.is_active())
            .map(|s| AgentSessionSummary {
                id: s.id,
                tenant_id: s.tenant_id,
                agent_id: s.agent_id,
                worktree_id: s.worktree_id,
                status: s.status,
                started_at: s.started_at,
                ended_at: s.ended_at,
            })
            .collect())
    }
}

// =====================================================================
// InMemoryAgentRepository
// =====================================================================

pub struct InMemoryAgentRepository {
    agents: RwLock<HashMap<AgentId, Agent>>,
    sessions: RwLock<HashMap<AgentSessionId, AgentSession>>,
    policies: RwLock<HashMap<AgentPolicyTemplateId, AgentPolicyTemplate>>,
}

impl InMemoryAgentRepository {
    pub fn new() -> Self {
        Self {
            agents: RwLock::new(HashMap::new()),
            sessions: RwLock::new(HashMap::new()),
            policies: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryAgentRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AgentRepository for InMemoryAgentRepository {
    async fn insert_agent(&self, agent: Agent) -> Result<(), AgentError> {
        self.agents.write().unwrap().insert(agent.id, agent);
        Ok(())
    }
    async fn get_agent(&self, id: AgentId) -> Result<Agent, AgentError> {
        self.agents
            .read()
            .unwrap()
            .get(&id)
            .cloned()
            .ok_or_else(|| AgentError::NotFound(format!("agent:{}", id.as_uuid())))
    }
    async fn list_agents(&self, tenant_id: TenantId) -> Result<Vec<Agent>, AgentError> {
        Ok(self
            .agents
            .read()
            .unwrap()
            .values()
            .filter(|a| a.tenant_id == tenant_id)
            .cloned()
            .collect())
    }
    async fn insert_session(&self, session: AgentSession) -> Result<(), AgentError> {
        self.sessions.write().unwrap().insert(session.id, session);
        Ok(())
    }
    async fn get_session(&self, id: AgentSessionId) -> Result<AgentSession, AgentError> {
        self.sessions
            .read()
            .unwrap()
            .get(&id)
            .cloned()
            .ok_or_else(|| AgentError::NotFound(format!("session:{}", id.as_uuid())))
    }
    async fn update_session(&self, session: AgentSession) -> Result<(), AgentError> {
        self.sessions.write().unwrap().insert(session.id, session);
        Ok(())
    }
    async fn list_sessions_by_worktree(
        &self,
        tenant_id: TenantId,
        worktree_id: WorktreeId,
    ) -> Result<Vec<AgentSession>, AgentError> {
        Ok(self
            .sessions
            .read()
            .unwrap()
            .values()
            .filter(|s| s.tenant_id == tenant_id && s.worktree_id == worktree_id)
            .cloned()
            .collect())
    }
    async fn list_active_sessions(
        &self,
        tenant_id: TenantId,
    ) -> Result<Vec<AgentSession>, AgentError> {
        Ok(self
            .sessions
            .read()
            .unwrap()
            .values()
            .filter(|s| s.tenant_id == tenant_id && s.status.is_active())
            .cloned()
            .collect())
    }
    async fn insert_policy_template(
        &self,
        template: AgentPolicyTemplate,
    ) -> Result<(), AgentError> {
        self.policies.write().unwrap().insert(template.id, template);
        Ok(())
    }
    async fn get_policy_template(
        &self,
        id: AgentPolicyTemplateId,
    ) -> Result<AgentPolicyTemplate, AgentError> {
        self.policies
            .read()
            .unwrap()
            .get(&id)
            .cloned()
            .ok_or_else(|| AgentError::NotFound(format!("policy:{}", id.as_uuid())))
    }
}

// =====================================================================
// 单元测试
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    fn make_actor(tenant_id: TenantId) -> ActorContext {
        ActorContext::new(Uuid::new_v4(), tenant_id.0).with_role("project_admin")
    }

    #[test]
    fn status_as_str() {
        assert_eq!(AgentSessionStatus::Created.as_str(), "CREATED");
        assert_eq!(AgentSessionStatus::Running.as_str(), "RUNNING");
        assert_eq!(AgentSessionStatus::Completed.as_str(), "COMPLETED");
    }

    #[test]
    fn status_is_terminal() {
        assert!(AgentSessionStatus::Completed.is_terminal());
        assert!(AgentSessionStatus::Failed.is_terminal());
        assert!(AgentSessionStatus::Aborted.is_terminal());
        assert!(AgentSessionStatus::Crashed.is_terminal());
        assert!(AgentSessionStatus::Timeout.is_terminal());
        assert!(!AgentSessionStatus::Running.is_terminal());
        assert!(!AgentSessionStatus::WaitingFeedback.is_terminal());
    }

    #[test]
    fn status_triggers_notification_inv_agt_n07() {
        // 关键突破抑制:Agent 中间步骤不触发,Failed/Crashed/Timeout 触发
        assert!(AgentSessionStatus::Failed.triggers_notification());
        assert!(AgentSessionStatus::Crashed.triggers_notification());
        assert!(AgentSessionStatus::Timeout.triggers_notification());
        assert!(!AgentSessionStatus::Running.triggers_notification());
        assert!(!AgentSessionStatus::WaitingTool.triggers_notification());
        assert!(!AgentSessionStatus::Validating.triggers_notification());
    }

    #[test]
    fn transition_start_sequence() {
        assert!(
            check_status_transition(AgentSessionStatus::Created, AgentSessionStatus::Starting)
                .is_ok()
        );
        assert!(
            check_status_transition(AgentSessionStatus::Starting, AgentSessionStatus::Running)
                .is_ok()
        );
    }

    #[test]
    fn transition_tool_loop() {
        assert!(check_status_transition(
            AgentSessionStatus::Running,
            AgentSessionStatus::WaitingTool
        )
        .is_ok());
        assert!(check_status_transition(
            AgentSessionStatus::WaitingTool,
            AgentSessionStatus::ToolRunning
        )
        .is_ok());
        assert!(check_status_transition(
            AgentSessionStatus::ToolRunning,
            AgentSessionStatus::ToolCompleted
        )
        .is_ok());
        assert!(check_status_transition(
            AgentSessionStatus::ToolCompleted,
            AgentSessionStatus::Running
        )
        .is_ok());
    }

    #[test]
    fn transition_feedback_loop() {
        assert!(check_status_transition(
            AgentSessionStatus::Running,
            AgentSessionStatus::WaitingFeedback
        )
        .is_ok());
        assert!(check_status_transition(
            AgentSessionStatus::WaitingFeedback,
            AgentSessionStatus::FeedbackReceived
        )
        .is_ok());
        assert!(check_status_transition(
            AgentSessionStatus::FeedbackReceived,
            AgentSessionStatus::Running
        )
        .is_ok());
    }

    #[test]
    fn transition_validation() {
        assert!(check_status_transition(
            AgentSessionStatus::Running,
            AgentSessionStatus::Validating
        )
        .is_ok());
        assert!(check_status_transition(
            AgentSessionStatus::Validating,
            AgentSessionStatus::Completed
        )
        .is_ok());
        assert!(check_status_transition(
            AgentSessionStatus::Validating,
            AgentSessionStatus::Failed
        )
        .is_ok());
    }

    #[test]
    fn transition_invalid_skipped() {
        // 跳跃:Created → Running 不允许
        assert!(
            check_status_transition(AgentSessionStatus::Created, AgentSessionStatus::Running)
                .is_err()
        );
        // 终态不可再迁移
        assert!(check_status_transition(
            AgentSessionStatus::Completed,
            AgentSessionStatus::Running
        )
        .is_err());
        assert!(
            check_status_transition(AgentSessionStatus::Aborted, AgentSessionStatus::Running)
                .is_err()
        );
    }

    #[test]
    fn policy_conservative_default() {
        let p = AgentPolicy::conservative();
        assert_eq!(p.network_access, NetworkAccess::Deny);
        assert_eq!(p.secret_access, SecretAccess::None);
        assert!(p.require_approval);
    }

    #[test]
    fn policy_enforce_forbidden_path() {
        let p = AgentPolicy {
            forbidden_paths: vec!["secrets/".to_string()],
            ..AgentPolicy::conservative()
        };
        let check = PolicyCheck {
            path: "secrets/api_key.txt".to_string(),
            tool: "read_file".to_string(),
            requires_network: false,
            requires_secret: false,
            elapsed_seconds: 10,
            changed_files: 0,
        };
        assert!(p.enforce(&check).is_err());
    }

    #[test]
    fn policy_enforce_tool_not_allowed() {
        let p = AgentPolicy::conservative();
        let check = PolicyCheck {
            path: "src/main.rs".to_string(),
            tool: "shell_exec".to_string(),
            requires_network: false,
            requires_secret: false,
            elapsed_seconds: 10,
            changed_files: 0,
        };
        assert!(p.enforce(&check).is_err());
    }

    #[test]
    fn policy_enforce_network_deny() {
        let p = AgentPolicy::conservative();
        let check = PolicyCheck {
            path: "src/main.rs".to_string(),
            tool: "read_file".to_string(),
            requires_network: true,
            requires_secret: false,
            elapsed_seconds: 10,
            changed_files: 0,
        };
        assert!(p.enforce(&check).is_err());
    }

    #[test]
    fn policy_enforce_runtime_exceeded() {
        let p = AgentPolicy::conservative();
        let check = PolicyCheck {
            path: "src/main.rs".to_string(),
            tool: "read_file".to_string(),
            requires_network: false,
            requires_secret: false,
            elapsed_seconds: 999,
            changed_files: 0,
        };
        assert!(p.enforce(&check).is_err());
    }

    #[test]
    fn policy_enforce_pass() {
        let p = AgentPolicy::conservative();
        let check = PolicyCheck {
            path: "src/main.rs".to_string(),
            tool: "read_file".to_string(),
            requires_network: false,
            requires_secret: false,
            elapsed_seconds: 10,
            changed_files: 0,
        };
        assert!(p.enforce(&check).is_ok());
    }

    #[tokio::test]
    async fn register_and_start_session() {
        let svc = InMemoryAgentService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_actor(tenant_id);
        let agent = svc
            .register_agent(
                RegisterAgentCommand {
                    tenant_id,
                    agent_type: AgentType::Codex,
                    provider: "openai".to_string(),
                    version: "1.0".to_string(),
                    capabilities: vec!["code".to_string()],
                    policy_template_id: None,
                    actor_user_id: UserId::from(actor.user_id),
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(agent.tenant_id, tenant_id);
        assert!(agent.enabled);
        let session = svc
            .start_session(
                StartSessionCommand {
                    tenant_id,
                    agent_id: agent.id,
                    worktree_id: WorktreeId::new(),
                    work_item_id: WorkItemId::new(),
                    intent: "fix bug".to_string(),
                    context_packet_id: None,
                    actor_user_id: UserId::from(actor.user_id),
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(session.status, AgentSessionStatus::Created);
    }

    #[tokio::test]
    async fn cross_tenant_register_denied() {
        let svc = InMemoryAgentService::new();
        let actor_tenant = uuid::Uuid::new_v4();
        let cmd_tenant = uuid::Uuid::new_v4();
        let actor = make_actor(actor_tenant);
        let res = svc
            .register_agent(
                RegisterAgentCommand {
                    tenant_id: cmd_tenant,
                    agent_type: AgentType::Codex,
                    provider: "openai".to_string(),
                    version: "1.0".to_string(),
                    capabilities: vec![],
                    policy_template_id: None,
                    actor_user_id: UserId::from(actor.user_id),
                },
                &actor,
            )
            .await;
        assert!(matches!(res, Err(AgentError::CrossTenantDenied(_, _))));
    }

    #[tokio::test]
    async fn full_session_lifecycle() {
        let svc = InMemoryAgentService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_actor(tenant_id);
        let agent = svc
            .register_agent(
                RegisterAgentCommand {
                    tenant_id,
                    agent_type: AgentType::Codex,
                    provider: "openai".to_string(),
                    version: "1.0".to_string(),
                    capabilities: vec![],
                    policy_template_id: None,
                    actor_user_id: UserId::from(actor.user_id),
                },
                &actor,
            )
            .await
            .unwrap();
        let session = svc
            .start_session(
                StartSessionCommand {
                    tenant_id,
                    agent_id: agent.id,
                    worktree_id: WorktreeId::new(),
                    work_item_id: WorkItemId::new(),
                    intent: "test".to_string(),
                    context_packet_id: None,
                    actor_user_id: UserId::from(actor.user_id),
                },
                &actor,
            )
            .await
            .unwrap();
        let id = session.id;
        // Created → Starting → Running → Validating → Completed
        let s = svc
            .transition_status(
                TransitionStatusCommand {
                    tenant_id,
                    session_id: id,
                    from: AgentSessionStatus::Created,
                    to: AgentSessionStatus::Starting,
                    reason: None,
                    actor_user_id: UserId::from(actor.user_id),
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(s.status, AgentSessionStatus::Starting);
        let s = svc
            .transition_status(
                TransitionStatusCommand {
                    tenant_id,
                    session_id: id,
                    from: AgentSessionStatus::Starting,
                    to: AgentSessionStatus::Running,
                    reason: None,
                    actor_user_id: UserId::from(actor.user_id),
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(s.status, AgentSessionStatus::Running);
        let s = svc
            .transition_status(
                TransitionStatusCommand {
                    tenant_id,
                    session_id: id,
                    from: AgentSessionStatus::Running,
                    to: AgentSessionStatus::Validating,
                    reason: None,
                    actor_user_id: UserId::from(actor.user_id),
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(s.status, AgentSessionStatus::Validating);
        let s = svc
            .transition_status(
                TransitionStatusCommand {
                    tenant_id,
                    session_id: id,
                    from: AgentSessionStatus::Validating,
                    to: AgentSessionStatus::Completed,
                    reason: None,
                    actor_user_id: UserId::from(actor.user_id),
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(s.status, AgentSessionStatus::Completed);
        assert!(s.ended_at.is_some());
    }

    #[tokio::test]
    async fn feedback_loop_transition() {
        let svc = InMemoryAgentService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_actor(tenant_id);
        let agent = svc
            .register_agent(
                RegisterAgentCommand {
                    tenant_id,
                    agent_type: AgentType::ClaudeCode,
                    provider: "anthropic".to_string(),
                    version: "1.0".to_string(),
                    capabilities: vec![],
                    policy_template_id: None,
                    actor_user_id: UserId::from(actor.user_id),
                },
                &actor,
            )
            .await
            .unwrap();
        let session = svc
            .start_session(
                StartSessionCommand {
                    tenant_id,
                    agent_id: agent.id,
                    worktree_id: WorktreeId::new(),
                    work_item_id: WorkItemId::new(),
                    intent: "test".to_string(),
                    context_packet_id: None,
                    actor_user_id: UserId::from(actor.user_id),
                },
                &actor,
            )
            .await
            .unwrap();
        let id = session.id;
        // Created → Starting → Running → WaitingFeedback
        svc.transition_status(
            TransitionStatusCommand {
                tenant_id,
                session_id: id,
                from: AgentSessionStatus::Created,
                to: AgentSessionStatus::Starting,
                reason: None,
                actor_user_id: UserId::from(actor.user_id),
            },
            &actor,
        )
        .await
        .unwrap();
        svc.transition_status(
            TransitionStatusCommand {
                tenant_id,
                session_id: id,
                from: AgentSessionStatus::Starting,
                to: AgentSessionStatus::Running,
                reason: None,
                actor_user_id: UserId::from(actor.user_id),
            },
            &actor,
        )
        .await
        .unwrap();
        svc.transition_status(
            TransitionStatusCommand {
                tenant_id,
                session_id: id,
                from: AgentSessionStatus::Running,
                to: AgentSessionStatus::WaitingFeedback,
                reason: None,
                actor_user_id: UserId::from(actor.user_id),
            },
            &actor,
        )
        .await
        .unwrap();
        // 提交反馈
        let s = svc
            .submit_feedback(
                SubmitFeedbackCommand {
                    tenant_id,
                    session_id: id,
                    agent_instruction: "请用 Redis 实现".to_string(),
                    actor_user_id: UserId::from(actor.user_id),
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(s.status, AgentSessionStatus::FeedbackReceived);
    }

    #[tokio::test]
    async fn abort_from_active() {
        let svc = InMemoryAgentService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_actor(tenant_id);
        let agent = svc
            .register_agent(
                RegisterAgentCommand {
                    tenant_id,
                    agent_type: AgentType::Codex,
                    provider: "openai".to_string(),
                    version: "1.0".to_string(),
                    capabilities: vec![],
                    policy_template_id: None,
                    actor_user_id: UserId::from(actor.user_id),
                },
                &actor,
            )
            .await
            .unwrap();
        let session = svc
            .start_session(
                StartSessionCommand {
                    tenant_id,
                    agent_id: agent.id,
                    worktree_id: WorktreeId::new(),
                    work_item_id: WorkItemId::new(),
                    intent: "test".to_string(),
                    context_packet_id: None,
                    actor_user_id: UserId::from(actor.user_id),
                },
                &actor,
            )
            .await
            .unwrap();
        let id = session.id;
        svc.transition_status(
            TransitionStatusCommand {
                tenant_id,
                session_id: id,
                from: AgentSessionStatus::Created,
                to: AgentSessionStatus::Starting,
                reason: None,
                actor_user_id: UserId::from(actor.user_id),
            },
            &actor,
        )
        .await
        .unwrap();
        svc.transition_status(
            TransitionStatusCommand {
                tenant_id,
                session_id: id,
                from: AgentSessionStatus::Starting,
                to: AgentSessionStatus::Running,
                reason: None,
                actor_user_id: UserId::from(actor.user_id),
            },
            &actor,
        )
        .await
        .unwrap();
        let s = svc
            .abort_session(
                AbortSessionCommand {
                    tenant_id,
                    session_id: id,
                    reason: "user requested".to_string(),
                    actor_user_id: UserId::from(actor.user_id),
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(s.status, AgentSessionStatus::Aborted);
        assert!(s.ended_at.is_some());
    }

    #[tokio::test]
    async fn list_by_worktree() {
        let svc = InMemoryAgentService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_actor(tenant_id);
        let agent = svc
            .register_agent(
                RegisterAgentCommand {
                    tenant_id,
                    agent_type: AgentType::Codex,
                    provider: "openai".to_string(),
                    version: "1.0".to_string(),
                    capabilities: vec![],
                    policy_template_id: None,
                    actor_user_id: UserId::from(actor.user_id),
                },
                &actor,
            )
            .await
            .unwrap();
        let wt = WorktreeId::new();
        svc.start_session(
            StartSessionCommand {
                tenant_id,
                agent_id: agent.id,
                worktree_id: wt,
                work_item_id: WorkItemId::new(),
                intent: "a".to_string(),
                context_packet_id: None,
                actor_user_id: UserId::from(actor.user_id),
            },
            &actor,
        )
        .await
        .unwrap();
        svc.start_session(
            StartSessionCommand {
                tenant_id,
                agent_id: agent.id,
                worktree_id: wt,
                work_item_id: WorkItemId::new(),
                intent: "b".to_string(),
                context_packet_id: None,
                actor_user_id: UserId::from(actor.user_id),
            },
            &actor,
        )
        .await
        .unwrap();
        let sessions = svc
            .list_by_worktree(
                ListByWorktreeQuery {
                    tenant_id,
                    worktree_id: wt,
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(sessions.len(), 2);
    }

    #[tokio::test]
    async fn create_policy_template() {
        let svc = InMemoryAgentService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_actor(tenant_id);
        let tpl = svc
            .create_policy_template(
                CreatePolicyTemplateCommand {
                    tenant_id,
                    name: "conservative".to_string(),
                    policy: AgentPolicy::conservative(),
                    actor_user_id: UserId::from(actor.user_id),
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(tpl.name, "conservative");
    }

    #[tokio::test]
    async fn record_tool_activity() {
        let svc = InMemoryAgentService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_actor(tenant_id);
        let agent = svc
            .register_agent(
                RegisterAgentCommand {
                    tenant_id,
                    agent_type: AgentType::Codex,
                    provider: "openai".to_string(),
                    version: "1.0".to_string(),
                    capabilities: vec![],
                    policy_template_id: None,
                    actor_user_id: UserId::from(actor.user_id),
                },
                &actor,
            )
            .await
            .unwrap();
        let session = svc
            .start_session(
                StartSessionCommand {
                    tenant_id,
                    agent_id: agent.id,
                    worktree_id: WorktreeId::new(),
                    work_item_id: WorkItemId::new(),
                    intent: "test".to_string(),
                    context_packet_id: None,
                    actor_user_id: UserId::from(actor.user_id),
                },
                &actor,
            )
            .await
            .unwrap();
        let s = svc
            .record_tool_activity(
                RecordToolActivityCommand {
                    tenant_id,
                    session_id: session.id,
                    tool: "read_file".to_string(),
                    count: 3,
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(s.tool_activity_summary.get("read_file"), Some(&3));
    }
}
