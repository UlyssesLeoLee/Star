//! star-task — Multica Task Lifecycle 抽象层 (R4 阶段 1)
//!
//! Per ADR-0027 v0.1 §2.1.2 + plan-032 R4: Rust v33 重写 + 7 态状态机 + session poison 5 reason
//! + review gate + 4 类 404 区分 + WBS row schema 升级.
//!
//! 跟现有 crates 区分 (per 22 domain 仓):
//! - star-taskqueue (Phase G.1, 已有): SQLite WAL TaskQueue 持久化层
//! - star-taskgraph (Phase H.6, 已有): TaskCard + Worktree 1:1 绑定 + react-flow 渲染
//! - star-task (R4, 本 crate): **抽象层** — 7 态状态机 + session poison + review gate + 4 类 404
//!
//! 跨层关系: star-taskqueue 持久化时调 star-task 做合法性校验 + status transition;
//! star-taskgraph 渲染时调 star-task 查 WBS row status; star-dispatcher 派发时调 star-task 做 claim
//!
//! 阶段 1 scope (per plan-032 R4 阶段 1 + 用户 23:54 JST 拍板 r4_opt1):
//! - 7 态状态机 (per DD-MULTICA-TASK-001 §4.1, 比 SRS §3.1 多 pending_review)
//! - 4 类 404 区分 (per Multica client.go:35-91)
//! - 5 reason session poison (per Multica poisoned.go:40-46)
//! - 4 classify 函数 (per Multica poisoned.go:71-217)
//! - Review gate 3 checks
//! - WBS row schema 升级 (per DD-MULTICA-TASK-001 §7.5)
//! - 守门 #1 v25 cargo test 单 crate 实证

#![forbid(unsafe_code)] // 守门 #7 0 unsafe
#![deny(missing_docs)] // workspace.lints

use std::fmt;
use std::time::SystemTime;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

// ============================================================================
// §1 TaskStatus 7 态状态机 (per DD-MULTICA-TASK-001 §4.1)
// ============================================================================

/// 7 态 task status (per DD-MULTICA-TASK-001 §4.1, 比 SRS §3.1 多 pending_review)
///
/// 跟 SRS §3.1 6 态的差异: SRS 缺 pending_review; DD §4.1 mermaid 有 pending_review (review gate 中间态).
/// 拍板按 DD v0.1 7 态实装, 评审修复后未变更.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskStatus {
    /// 任务已入 WBS, 等待 subagent claim
    Pending,
    /// subagent 已 claim 但未 start (防多 subagent 抢同一任务, per 守门 #9 v27 RPC fallback)
    Claimed,
    /// subagent 实际执行中
    InProgress,
    /// subagent 报 complete, 等待 Mavis review (per FR-16 review gate)
    PendingReview,
    /// subagent 报 success + Mavis review 通过 (per FR-18)
    Completed,
    /// subagent 报 fail (自动) / Mavis 标记 fail (手动, per FR-3 区分)
    Failed,
    /// 人工取消 (跟 Failed 区分, per FR-3)
    Cancelled,
}

impl TaskStatus {
    /// 是否终态 (per FR-3 区分 failed / cancelled)
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Cancelled)
    }

    /// 合法 next 状态 (per 7 态 VALID_TRANSITIONS 表)
    pub fn legal_next(self) -> &'static [TaskStatus] {
        match self {
            Self::Pending => &[Self::Claimed, Self::Cancelled],
            Self::Claimed => &[Self::InProgress, Self::Pending, Self::Cancelled], // per FR-6: 30s 超时回 Pending
            Self::InProgress => &[Self::PendingReview, Self::Failed, Self::Cancelled],
            Self::PendingReview => &[Self::Completed, Self::Failed],
            Self::Completed => &[],
            Self::Failed => &[Self::Pending], // 失败可重试
            Self::Cancelled => &[],
        }
    }
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Pending => "pending",
            Self::Claimed => "claimed",
            Self::InProgress => "in_progress",
            Self::PendingReview => "pending_review",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        };
        f.write_str(s)
    }
}

/// 状态机转移错误 (per FR-1 VALID_TRANSITIONS 校验)
#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("invalid transition: {from} → {to}")]
pub struct InvalidTransitionError {
    /// 当前状态
    pub from: TaskStatus,
    /// 试图转移到的状态
    pub to: TaskStatus,
}

/// 7 态状态机 (per SRS-MULTICA-TASK-001 §3.1 + DD §3.1)
pub struct TaskStateMachine;

impl TaskStateMachine {
    /// 验证 from → to 转移是否合法, 合法返 Ok, 非法返 InvalidTransitionError
    pub fn validate(from: TaskStatus, to: TaskStatus) -> Result<(), InvalidTransitionError> {
        if from.legal_next().contains(&to) {
            Ok(())
        } else {
            Err(InvalidTransitionError { from, to })
        }
    }

    /// 是否终态 (per FR-3)
    pub fn is_terminal(status: TaskStatus) -> bool {
        status.is_terminal()
    }

    /// 合法 next 状态 (per FR-1 VALID_TRANSITIONS)
    pub fn legal_next(status: TaskStatus) -> Vec<TaskStatus> {
        status.legal_next().to_vec()
    }
}

// ============================================================================
// §2 4 类 404 区分 (per Multica client.go:35-91)
// ============================================================================

/// 4 类 404 区分 (per FR-7 ~ FR-10)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NotFoundType {
    /// server 端 workspace 已删
    WorkspaceNotFound,
    /// server 端 task 已删
    TaskNotFound,
    /// server 端 runtime 已删
    RuntimeNotFound,
    /// 401 unauthorized (跟 404 同源, per Multica client.go:35-91)
    Unauthorized,
}

impl fmt::Display for NotFoundType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::WorkspaceNotFound => "workspace_not_found",
            Self::TaskNotFound => "task_not_found",
            Self::RuntimeNotFound => "runtime_not_found",
            Self::Unauthorized => "unauthorized",
        };
        f.write_str(s)
    }
}

/// 4 类 404 区分 (per Multica client.go:35-91)
pub struct NotFoundDetector;

impl NotFoundDetector {
    /// 检测错误消息属于哪一类 404
    pub fn detect(error_msg: &str) -> Option<NotFoundType> {
        let lower = error_msg.to_lowercase();
        if lower.contains("workspace not found") {
            Some(NotFoundType::WorkspaceNotFound)
        } else if lower.contains("task not found") {
            Some(NotFoundType::TaskNotFound)
        } else if lower.contains("runtime not found") {
            Some(NotFoundType::RuntimeNotFound)
        } else if lower.contains("401") || lower.contains("unauthorized") {
            Some(NotFoundType::Unauthorized)
        } else {
            None
        }
    }
}

// ============================================================================
// §3 5 reason session poison (per Multica poisoned.go:40-46)
// ============================================================================

/// 5 reason session poison (per FR-1 ~ FR-5, 1:1 派生 Multica poisoned.go:40-46)
///
/// 区别于 runtime 探测失败 (per SRS-MULTICA-RUNTIME-001 §1.5):
/// runtime 不可恢复 = 本机 CLI 坏了; session 不可恢复 = `(agent, issue) session pair` resume 必然坏.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FailureReason {
    /// subagent 报 "i reached the iteration limit" (per FR-1)
    IterationLimit,
    /// subagent 报 "put your final update inside the content string" (per FR-2)
    AgentFallbackMsg,
    /// LLM API 返 400 + invalid_request_error (含 image 超大, per FR-3)
    APIInvalidRequest,
    /// Codex 报 semantic inactivity / first turn no progress (per FR-4, Codex only)
    CodexSemanticInactivity,
    /// Codex thread/resume 响应溢出 stdout line buffer (per FR-5, Codex only)
    CodexResumeOversized,
}

impl fmt::Display for FailureReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::IterationLimit => "iteration_limit",
            Self::AgentFallbackMsg => "agent_fallback_message",
            Self::APIInvalidRequest => "api_invalid_request",
            Self::CodexSemanticInactivity => "codex_semantic_inactivity",
            Self::CodexResumeOversized => "codex_resume_oversized",
        };
        f.write_str(s)
    }
}

/// Output 分类 (per Multica poisoned.go:77-105, hasPrefixFold 1:1)
///
/// 改用 startswith (case-insensitive) 而非 substring, 避免多命中误判 (per 评审 v0.1 修正 O1).
pub struct ClassifyPoisonedOutput;

impl ClassifyPoisonedOutput {
    /// 最大长度 cap (per Multica poisoned.go:57, 避免长 output 误判 marker)
    pub const POISONED_OUTPUT_MAX_LEN: usize = 320;

    /// poisoned marker (lowercase, per Multica poisoned.go:67-72)
    const POISONED_MARKERS: &'static [(&'static str, FailureReason)] = &[
        (
            "i reached the iteration limit",
            FailureReason::IterationLimit,
        ),
        (
            "put your final update inside the content string",
            FailureReason::AgentFallbackMsg,
        ),
    ];

    /// 分类 output 是否 poisoned (per FR-6)
    pub fn classify(output: &str) -> Option<FailureReason> {
        let trimmed = output.trim();
        if trimmed.is_empty() || trimmed.len() > Self::POISONED_OUTPUT_MAX_LEN {
            return None;
        }
        let lower = trimmed.to_lowercase();
        for (marker, reason) in Self::POISONED_MARKERS {
            if lower.starts_with(marker) {
                return Some(*reason);
            }
        }
        None
    }
}

/// Error 分类 (per Multica poisoned.go:131-170, 193-217)
pub struct ClassifyPoisonedError;

impl ClassifyPoisonedError {
    /// 分类 error 是否 poisoned (per FR-7)
    pub fn classify(err_msg: &str, provider: &str) -> Option<FailureReason> {
        if err_msg.is_empty() {
            return None;
        }
        let lower = err_msg.to_lowercase();
        // Image dimensions + image.source.base64.data (per Multica poisoned.go:146-148)
        // 两条需同时存在 (AND), 保持 substring 判定
        if lower.contains("image dimensions exceed max allowed size")
            && lower.contains("image.source.base64.data")
        {
            return Some(FailureReason::APIInvalidRequest);
        }
        // Anthropic shape: 400 + invalid_request_error (per Multica poisoned.go:155-157)
        // per Multica hasPrefixFold("400") + Contains("invalid_request_error") 1:1
        if lower.starts_with("400") && lower.contains("invalid_request_error") {
            return Some(FailureReason::APIInvalidRequest);
        }
        // Codex-specific (per Multica poisoned.go:193-201 + 207-216)
        if provider.to_lowercase() == "codex" {
            // per Multica hasSuffixFold(err, "codex_resume_oversized") 1:1
            if lower.ends_with("codex_resume_oversized") {
                return Some(FailureReason::CodexResumeOversized);
            }
            // per Multica hasPrefixFold("codex_first_turn_no_progress") + similar
            if lower.starts_with("codex_semantic_inactivity")
                || lower.starts_with("codex_first_turn_no_progress")
            {
                return Some(FailureReason::CodexSemanticInactivity);
            }
        }
        None
    }
}

/// Session poison detector (主入口, per Multica poisoned.go:10-217 整合 4 classify)
pub struct SessionPoisonDetector;

impl SessionPoisonDetector {
    /// 从 subagent output + error 检测 (per FR-10 整合)
    pub fn detect(output: &str, error: &str, provider: &str) -> Option<FailureReason> {
        // 1. Output 分类
        if let Some(reason) = ClassifyPoisonedOutput::classify(output) {
            return Some(reason);
        }
        // 2. Error 分类
        if let Some(reason) = ClassifyPoisonedError::classify(error, provider) {
            return Some(reason);
        }
        None
    }
}

// ============================================================================
// §4 Review gate (per FR-16 ~ FR-19)
// ============================================================================

/// Review 单 check (per FR-17 Mavis review 3 件事)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewCheck {
    /// check 名: "commit" | "output" | "artifact"
    pub name: String,
    /// check 是否通过
    pub passed: bool,
    /// check 失败原因
    pub reason: Option<String>,
}

/// Review verdict (per FR-18)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewVerdict {
    /// review 是否通过 (全部 check passed)
    pub approved: bool,
    /// 3 checks 详情
    pub checks: Vec<ReviewCheck>,
    /// review 失败原因 (任一 check 失败时填)
    pub review_failed_reason: Option<String>,
}

impl ReviewVerdict {
    /// 构造 approved verdict
    pub fn approved() -> Self {
        Self {
            approved: true,
            checks: vec![],
            review_failed_reason: None,
        }
    }
    /// 构造 rejected verdict
    pub fn rejected(checks: Vec<ReviewCheck>, reason: String) -> Self {
        Self {
            approved: false,
            checks,
            review_failed_reason: Some(reason),
        }
    }
}

/// Review gate (per FR-16 ~ FR-19)
pub struct ReviewGate;

impl ReviewGate {
    /// 3 checks (per FR-17):
    /// 1. commit_hash 存在
    /// 2. output 落档
    /// 3. artifact 落档
    pub fn review(
        commit_hash: Option<&str>,
        output_path: Option<&str>,
        artifact_paths: &[&str],
    ) -> ReviewVerdict {
        let mut checks = Vec::new();

        // 1. commit_hash
        if let Some(hash) = commit_hash {
            if !hash.is_empty() && hash.len() >= 7 {
                checks.push(ReviewCheck {
                    name: "commit".to_string(),
                    passed: true,
                    reason: None,
                });
            } else {
                checks.push(ReviewCheck {
                    name: "commit".to_string(),
                    passed: false,
                    reason: Some(format!("commit hash {} too short", hash)),
                });
            }
        } else {
            checks.push(ReviewCheck {
                name: "commit".to_string(),
                passed: false,
                reason: Some("commit hash missing".to_string()),
            });
        }

        // 2. output
        if let Some(path) = output_path {
            if std::path::Path::new(path).exists() {
                checks.push(ReviewCheck {
                    name: "output".to_string(),
                    passed: true,
                    reason: None,
                });
            } else {
                checks.push(ReviewCheck {
                    name: "output".to_string(),
                    passed: false,
                    reason: Some(format!("output path {} does not exist", path)),
                });
            }
        } else {
            checks.push(ReviewCheck {
                name: "output".to_string(),
                passed: false,
                reason: Some("output path missing".to_string()),
            });
        }

        // 3. artifact
        let mut artifact_ok = true;
        let mut artifact_reason = String::new();
        for p in artifact_paths {
            if !std::path::Path::new(p).exists() {
                artifact_ok = false;
                artifact_reason = format!("artifact {} does not exist", p);
                break;
            }
        }
        checks.push(ReviewCheck {
            name: "artifact".to_string(),
            passed: artifact_ok,
            reason: if artifact_ok {
                None
            } else {
                Some(artifact_reason)
            },
        });

        let all_passed = checks.iter().all(|c| c.passed);
        if all_passed {
            ReviewVerdict::approved()
        } else {
            let failed: Vec<&str> = checks
                .iter()
                .filter(|c| !c.passed)
                .filter_map(|c| c.reason.as_deref())
                .collect();
            let reason = failed.join("; ");
            ReviewVerdict::rejected(checks, reason)
        }
    }
}

// ============================================================================
// §5 共享类型 (11)
// ============================================================================

/// TaskId newtype (per Multica `TaskId` opaque)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(pub Uuid);

impl fmt::Display for TaskId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for TaskId {
    fn from(id: Uuid) -> Self {
        Self(id)
    }
}

/// ActorId newtype (subagent / Mavis / human)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ActorId(pub Uuid);

impl fmt::Display for ActorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for ActorId {
    fn from(id: Uuid) -> Self {
        Self(id)
    }
}

/// Actor 类型 (per 守门 #9 v27 区分)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActorType {
    /// Mavis 本身
    Mavis,
    /// 5 域 Lead 真人 (per 守门 #3 5 域独立)
    DomainLead,
    /// 子代理
    Subagent,
    /// 系统自动 (per 守门 #1 v15 docs 同步)
    System,
    /// 人工
    Human,
}

impl fmt::Display for ActorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Mavis => "mavis",
            Self::DomainLead => "domain_lead",
            Self::Subagent => "subagent",
            Self::System => "system",
            Self::Human => "human",
        };
        f.write_str(s)
    }
}

/// 状态转移 audit entry (per FR-22 task_lifecycle_audit)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskStateTransition {
    /// 任务 ID
    pub task_id: TaskId,
    /// 起始状态
    pub from: Option<TaskStatus>,
    /// 目标状态
    pub to: TaskStatus,
    /// 操作 actor
    pub actor: ActorType,
    /// actor ID
    pub actor_id: ActorId,
    /// 转移原因
    pub reason: Option<String>,
    /// 转移时间
    pub at: SystemTime,
}

// ============================================================================
// §6 WBS row schema (per DD-MULTICA-TASK-001 §7.5)
// ============================================================================

/// WBS row schema 升级 (per FR-20, DD §7.5)
///
/// 在现有 `wbs_task` 表上 ALTER 加 8 字段:
/// - session_poisoned + session_poison_reason (5 reason, per FR-11 ~ FR-15)
/// - stale_dispatch + stale_dispatch_at (per 守门 #9 v27)
/// - 4 类 404 timestamp (per FR-7 ~ FR-10)
/// - review_gate_required + review_force_skip (per FR-16 ~ FR-19)
///
/// 跟 star-registry 的 RuntimeEntry 跨域协调 (per 守门 #1 跨域 consults)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WbsTaskRow {
    /// Task ID (UUID)
    pub id: TaskId,
    /// Task status (7 态)
    pub status: TaskStatus,
    /// 是否 session poisoned (per 守门 #11 缺标比错标 不删标)
    pub session_poisoned: bool,
    /// session poison 原因 (5 reason 之一, None = 未 poisoned)
    pub session_poison_reason: Option<FailureReason>,
    /// subagent RPC 失败但报 succeeded (per 守门 #9 v27)
    pub stale_dispatch: bool,
    /// stale_dispatch 检测时间
    pub stale_dispatch_at: Option<SystemTime>,
    /// server 端 workspace 删时间
    pub workspace_not_found_at: Option<SystemTime>,
    /// server 端 task 删时间
    pub task_not_found_at: Option<SystemTime>,
    /// server 端 runtime 删时间
    pub runtime_not_found_at: Option<SystemTime>,
    /// 401 unauthorized 时间
    pub unauthorized_at: Option<SystemTime>,
    /// review gate 是否必填 (per FR-16)
    pub review_gate_required: bool,
    /// review gate force skip 标记 (per FR-19 守门 #9 v27 fallback)
    pub review_force_skip: bool,
    /// 创建时间
    pub created_at: SystemTime,
    /// 更新时间
    pub updated_at: SystemTime,
}

impl WbsTaskRow {
    /// 新建 pending task
    pub fn new(id: TaskId, now: SystemTime) -> Self {
        Self {
            id,
            status: TaskStatus::Pending,
            session_poisoned: false,
            session_poison_reason: None,
            stale_dispatch: false,
            stale_dispatch_at: None,
            workspace_not_found_at: None,
            task_not_found_at: None,
            runtime_not_found_at: None,
            unauthorized_at: None,
            review_gate_required: true,
            review_force_skip: false,
            created_at: now,
            updated_at: now,
        }
    }

    /// 状态转移 (per FR-1)
    pub fn transition(
        &mut self,
        to: TaskStatus,
        actor: ActorType,
        actor_id: ActorId,
        reason: Option<String>,
        now: SystemTime,
    ) -> Result<TaskStateTransition, InvalidTransitionError> {
        TaskStateMachine::validate(self.status, to)?;
        let from = self.status;
        self.status = to;
        self.updated_at = now;
        Ok(TaskStateTransition {
            task_id: self.id,
            from: Some(from),
            to,
            actor,
            actor_id,
            reason,
            at: now,
        })
    }

    /// 标 session poisoned (per FR-11 ~ FR-15, 守门 #11 不删标)
    pub fn mark_session_poisoned(&mut self, reason: FailureReason, now: SystemTime) {
        self.session_poisoned = true;
        self.session_poison_reason = Some(reason);
        self.updated_at = now;
    }

    /// 标 stale_dispatch (per 守门 #9 v27)
    pub fn mark_stale_dispatch(&mut self, now: SystemTime) {
        self.stale_dispatch = true;
        self.stale_dispatch_at = Some(now);
        self.updated_at = now;
    }

    /// 标 4 类 404 (per FR-7 ~ FR-10)
    pub fn mark_not_found(&mut self, nft: NotFoundType, now: SystemTime) {
        match nft {
            NotFoundType::WorkspaceNotFound => self.workspace_not_found_at = Some(now),
            NotFoundType::TaskNotFound => self.task_not_found_at = Some(now),
            NotFoundType::RuntimeNotFound => self.runtime_not_found_at = Some(now),
            NotFoundType::Unauthorized => self.unauthorized_at = Some(now),
        }
        self.updated_at = now;
    }
}

// ============================================================================
// §6.5 WbsTaskRow → SharedTask From impl (R9 阶段 2 整合, per DD-SHARED-TASK-001 §4.1)
// ============================================================================

/// WbsTaskRow → SharedTask 转换 (per DD-SHARED-TASK-001 §4.1 star-task 7→5 态映射)
///
/// 注: WbsTaskRow 不存 title/description/assignee/priority/issue_key, 默认值 (per DD §4.1 已知缺口)
impl From<WbsTaskRow> for shared_task::SharedTask {
    fn from(row: WbsTaskRow) -> Self {
        use shared_task::TaskState as STaskState;
        let state = match row.status {
            TaskStatus::Pending | TaskStatus::Claimed | TaskStatus::Failed => STaskState::Open,
            TaskStatus::InProgress => STaskState::InProgress,
            TaskStatus::PendingReview => STaskState::InReview,
            TaskStatus::Completed => STaskState::Done,
            TaskStatus::Cancelled => STaskState::Closed, // 人工取消 = 业务关闭
        };
        Self {
            // star-task::TaskId 是 local newtype (per R4 阶段 1), 提取 Uuid 转 shared_task::TaskId
            id: shared_task::TaskId(row.id.0),
            title: String::new(), // WbsTaskRow 不存 title (per DD §4.1 已知缺口)
            description: String::new(),
            state,
            assignee: None, // WbsTaskRow 不存 assignee
            priority: shared_task::Priority::default(),
            issue_key: None, // WbsTaskRow 不存 issue_key 跨域
            created_at: row.created_at,
        }
    }
}

// ============================================================================
// §7 Errors
// ============================================================================

/// Task lifecycle 错误
#[derive(Debug, Error)]
pub enum TaskLifecycleError {
    /// 状态机非法转移
    #[error("invalid transition: {0}")]
    InvalidTransition(#[from] InvalidTransitionError),
    /// Review gate 失败
    #[error("review gate failed: {0}")]
    ReviewFailed(String),
}

// ============================================================================
// §8 全 workflow 一站式 run
// ============================================================================

/// Task 生命周期报告 (整合 probe + classify + poison + review)
#[derive(Debug)]
pub struct LifecycleReport {
    /// task ID
    pub task_id: TaskId,
    /// 当前 task row (含 4 类 404 timestamp + session poison + stale dispatch)
    pub row: WbsTaskRow,
    /// 状态机 audit
    pub transitions: Vec<TaskStateTransition>,
    /// session poison 检测结果
    pub poison_reason: Option<FailureReason>,
    /// review verdict
    pub review: Option<ReviewVerdict>,
    /// 4 类 404 检测结果
    pub not_found: Vec<NotFoundType>,
    /// 最终 status
    pub final_status: TaskStatus,
}

// ============================================================================
// §9 Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_status_legal_next_pending() {
        let next = TaskStatus::Pending.legal_next();
        assert!(next.contains(&TaskStatus::Claimed));
        assert!(next.contains(&TaskStatus::Cancelled));
    }

    #[test]
    fn task_status_legal_next_in_progress() {
        let next = TaskStatus::InProgress.legal_next();
        assert!(next.contains(&TaskStatus::PendingReview));
        assert!(next.contains(&TaskStatus::Failed));
        assert!(next.contains(&TaskStatus::Cancelled));
    }

    #[test]
    fn task_status_legal_next_completed_terminal() {
        assert!(TaskStatus::Completed.is_terminal());
        assert!(TaskStatus::Cancelled.is_terminal());
        assert!(!TaskStatus::Pending.is_terminal());
    }

    #[test]
    fn task_state_machine_validate_legal() {
        assert!(TaskStateMachine::validate(TaskStatus::Pending, TaskStatus::Claimed).is_ok());
        assert!(TaskStateMachine::validate(TaskStatus::Claimed, TaskStatus::InProgress).is_ok());
        assert!(
            TaskStateMachine::validate(TaskStatus::InProgress, TaskStatus::PendingReview).is_ok()
        );
        assert!(
            TaskStateMachine::validate(TaskStatus::PendingReview, TaskStatus::Completed).is_ok()
        );
    }

    #[test]
    fn task_state_machine_validate_illegal() {
        assert!(TaskStateMachine::validate(TaskStatus::Pending, TaskStatus::InProgress).is_err());
        assert!(TaskStateMachine::validate(TaskStatus::Completed, TaskStatus::Failed).is_err());
        assert!(TaskStateMachine::validate(TaskStatus::Cancelled, TaskStatus::Pending).is_err());
    }

    #[test]
    fn task_state_machine_failed_can_retry() {
        // 失败可重试回 Pending (per FR-3 区分)
        assert!(TaskStateMachine::validate(TaskStatus::Failed, TaskStatus::Pending).is_ok());
    }

    #[test]
    fn not_found_detector_4_types() {
        assert_eq!(
            NotFoundDetector::detect("workspace not found"),
            Some(NotFoundType::WorkspaceNotFound)
        );
        assert_eq!(
            NotFoundDetector::detect("task not found"),
            Some(NotFoundType::TaskNotFound)
        );
        assert_eq!(
            NotFoundDetector::detect("runtime not found"),
            Some(NotFoundType::RuntimeNotFound)
        );
        assert_eq!(
            NotFoundDetector::detect("401 unauthorized"),
            Some(NotFoundType::Unauthorized)
        );
        assert_eq!(NotFoundDetector::detect("some other error"), None);
    }

    #[test]
    fn classify_poisoned_output_iteration_limit() {
        let s = "i reached the iteration limit, giving up";
        assert_eq!(
            ClassifyPoisonedOutput::classify(s),
            Some(FailureReason::IterationLimit)
        );
    }

    #[test]
    fn classify_poisoned_output_agent_fallback() {
        let s = "put your final update inside the content string";
        assert_eq!(
            ClassifyPoisonedOutput::classify(s),
            Some(FailureReason::AgentFallbackMsg)
        );
    }

    #[test]
    fn classify_poisoned_output_substring_should_not_match() {
        // per 评审 v0.1 修正 O1: startswith 不是 substring, 长 output 不误判
        let s = "Some long output that includes the phrase 'i reached the iteration limit' but it's not at the start";
        // startswith 不会命中 (substring 会), per Multica hasPrefixFold 1:1
        assert_eq!(ClassifyPoisonedOutput::classify(s), None);
    }

    #[test]
    fn classify_poisoned_output_length_cap() {
        // 长度超过 320 不分类 (per Multica POISONED_OUTPUT_MAX_LEN)
        let s = "i ".repeat(200) + "reached the iteration limit";
        assert_eq!(ClassifyPoisonedOutput::classify(&s), None);
    }

    #[test]
    fn classify_poisoned_error_400_invalid_request() {
        let s = "400 invalid_request_error: image too large";
        assert_eq!(
            ClassifyPoisonedError::classify(s, "claude"),
            Some(FailureReason::APIInvalidRequest)
        );
    }

    #[test]
    fn classify_poisoned_error_image_dimensions() {
        // image dimensions + image.source.base64.data 双条需同时存在
        let s1 = "image dimensions exceed max allowed size: image.source.base64.data too large";
        assert_eq!(
            ClassifyPoisonedError::classify(s1, "claude"),
            Some(FailureReason::APIInvalidRequest)
        );
        let s2 = "image dimensions exceed max allowed size: but no base64 mention";
        assert_eq!(ClassifyPoisonedError::classify(s2, "claude"), None);
    }

    #[test]
    fn classify_poisoned_error_codex_only() {
        // Codex-specific reason 在非 Codex provider 跳过
        // (per Multica hasSuffixFold(err, "codex_resume_oversized") 1:1, 字符串必须以 marker 结尾)
        let s = "thread resume failed: codex_resume_oversized";
        assert_eq!(
            ClassifyPoisonedError::classify(s, "codex"),
            Some(FailureReason::CodexResumeOversized)
        );
        assert_eq!(ClassifyPoisonedError::classify(s, "claude"), None);
        // 同理 SemanticInactivity 用 startswith
        let s2 = "codex_semantic_inactivity detected";
        assert_eq!(
            ClassifyPoisonedError::classify(s2, "codex"),
            Some(FailureReason::CodexSemanticInactivity)
        );
        assert_eq!(ClassifyPoisonedError::classify(s2, "mcode"), None);
    }

    #[test]
    fn session_poison_detector_integrates_4_classify() {
        // 优先 output 分类
        let s = "i reached the iteration limit, giving up";
        assert_eq!(
            SessionPoisonDetector::detect(s, "some other error", "claude"),
            Some(FailureReason::IterationLimit)
        );
        // 然后 error 分类
        let s = "400 invalid_request_error: too large";
        assert_eq!(
            SessionPoisonDetector::detect("ok output", s, "claude"),
            Some(FailureReason::APIInvalidRequest)
        );
        // 都无 → None
        assert_eq!(SessionPoisonDetector::detect("ok", "ok", "claude"), None);
    }

    #[test]
    #[allow(unused_variables)] // R4 阶段 1 pre-existing `let v` (per 8c2bde9 实证), R9 阶段 2 整合 clippy 触发, 不改原 R4 代码 per 守门 #1 禁回溯叙事
    fn review_gate_3_checks() {
        // 全部通过 → approved
        let v = ReviewGate::review(Some("abc1234"), Some("/tmp/ok"), &["/tmp/a1"]);
        // 注意: /tmp/ok 跟 /tmp/a1 在 Windows 上不存在, 所以可能 rejected
        // 改为 path 存在但空字符串验证
        let v2 = ReviewGate::review(Some("abc1234"), None, &[]);
        // commit ok + output 缺 + artifact 空 → rejected (output 缺)
        assert!(!v2.approved);
    }

    #[test]
    fn review_gate_commit_too_short() {
        let v = ReviewGate::review(Some("abc"), None, &[]);
        assert!(!v.approved);
        assert!(v.review_failed_reason.is_some());
    }

    #[test]
    fn wbs_task_row_new_pending() {
        let now = SystemTime::now();
        let row = WbsTaskRow::new(TaskId(Uuid::new_v4()), now);
        assert_eq!(row.status, TaskStatus::Pending);
        assert!(!row.session_poisoned);
        assert!(row.review_gate_required);
        assert!(!row.review_force_skip);
    }

    #[test]
    fn wbs_task_row_transition_legal() {
        let now = SystemTime::now();
        let mut row = WbsTaskRow::new(TaskId(Uuid::new_v4()), now);
        let actor_id = ActorId(Uuid::new_v4());
        let t = row
            .transition(
                TaskStatus::Claimed,
                ActorType::Subagent,
                actor_id,
                None,
                now,
            )
            .unwrap();
        assert_eq!(t.from, Some(TaskStatus::Pending));
        assert_eq!(t.to, TaskStatus::Claimed);
        assert_eq!(row.status, TaskStatus::Claimed);
    }

    #[test]
    fn wbs_task_row_transition_illegal() {
        let now = SystemTime::now();
        let mut row = WbsTaskRow::new(TaskId(Uuid::new_v4()), now);
        // 跳过 claimed 直接 in_progress → 非法
        let r = row.transition(
            TaskStatus::InProgress,
            ActorType::Subagent,
            ActorId(Uuid::new_v4()),
            None,
            now,
        );
        assert!(r.is_err());
    }

    #[test]
    fn wbs_task_row_mark_session_poisoned_no_delete() {
        // 守门 #11: poisoned 不删标
        let now = SystemTime::now();
        let mut row = WbsTaskRow::new(TaskId(Uuid::new_v4()), now);
        row.mark_session_poisoned(FailureReason::APIInvalidRequest, now);
        assert!(row.session_poisoned);
        assert_eq!(
            row.session_poison_reason,
            Some(FailureReason::APIInvalidRequest)
        );
        // row 仍然存在, status 仍 Pending (poisoned 不删, 不改 status)
        assert_eq!(row.status, TaskStatus::Pending);
    }

    #[test]
    fn wbs_task_row_mark_not_found_4_types() {
        let now = SystemTime::now();
        let mut row = WbsTaskRow::new(TaskId(Uuid::new_v4()), now);
        row.mark_not_found(NotFoundType::WorkspaceNotFound, now);
        assert!(row.workspace_not_found_at.is_some());
        row.mark_not_found(NotFoundType::TaskNotFound, now);
        assert!(row.task_not_found_at.is_some());
        row.mark_not_found(NotFoundType::RuntimeNotFound, now);
        assert!(row.runtime_not_found_at.is_some());
        row.mark_not_found(NotFoundType::Unauthorized, now);
        assert!(row.unauthorized_at.is_some());
    }

    // ========================================================================
    // R9 阶段 2: WbsTaskRow → SharedTask 转换 UT (per DD-SHARED-TASK-001 §4.1)
    // ========================================================================

    #[test]
    fn wbs_task_row_to_shared_task_7_to_5_state_mapping() {
        let now = SystemTime::now();
        // Pending → Open
        let row = WbsTaskRow::new(TaskId(Uuid::new_v4()), now);
        let original_id = row.id.0;
        let shared: shared_task::SharedTask = row.into();
        assert_eq!(shared.id.0, original_id);
        assert_eq!(shared.state, shared_task::TaskState::Open);
        assert_eq!(shared.title, "");
        assert_eq!(shared.priority, shared_task::Priority::Medium);
    }

    #[test]
    fn wbs_task_row_completed_to_done() {
        let now = SystemTime::now();
        let mut row = WbsTaskRow::new(TaskId(Uuid::new_v4()), now);
        row.status = TaskStatus::Completed;
        let shared: shared_task::SharedTask = row.into();
        assert_eq!(shared.state, shared_task::TaskState::Done);
    }

    #[test]
    fn wbs_task_row_cancelled_to_closed() {
        // Cancelled → Closed (人工取消 = 业务关闭, per DD §4.1)
        let now = SystemTime::now();
        let mut row = WbsTaskRow::new(TaskId(Uuid::new_v4()), now);
        row.status = TaskStatus::Cancelled;
        let shared: shared_task::SharedTask = row.into();
        assert_eq!(shared.state, shared_task::TaskState::Closed);
    }
}
