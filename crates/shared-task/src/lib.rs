//! shared-task — 3 view 共享 Task Schema Rust 整合层 (R9 阶段 2) 🟢
//!
//! Per DD-SHARED-TASK-001 v0.1 §2 §3 §4: 跨 Jira/Miro/MS Project/agent 4 view 共享 Task entity
//! 表达, 整合 6 view crate (star-task/star-workflow/star-canvas/star-scheduler/star-game/canvas-collab).
//!
//! 整合方式 (per DD §3.2 + Rust orphan rule):
//! - shared-task crate 拥有 TaskId/TaskState/Priority/SharedTask (本地类型)
//! - 5 view crate 各自 impl `From<TheirType> for SharedTask` (per orphan rule, SharedTask 本地 + 5 view 各自的类型本地)
//! - 这样不会产生 6 view crate ↔ shared-task cycle
//!
//! WBS 集成 (per ADR-0027 §2.3.2 共享 Task schema 跨 3 view):
//! - WBS row = SharedTask (8 字段, per DD §2.1)
//! - 3 view 字段映射: per DD §3.1 (Jira/Miro/MS Project/agent 4 view 字段对照)
//! - 7 crate 集成表: per DD §4 (star-task 7→5 态 + 4 view 1:1 映射 + star-registry 间接 + canvas-collab 间接)
//!
//! 守门合规 (per 守门 #1 v25 cargo test 单 crate 实证 + 守门 #7 0 unsafe + 守门 #11 缺标比错标)

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use std::time::SystemTime;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// §1 TaskId newtype (per Multica opaque ID 1:1 派生)
// ============================================================================

/// Task ID (per Multica opaque ID, 1:1 派生, 6 view crate 各自的 TaskId/IssueKey/NodeId/TaskId 共享此 newtype)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(pub Uuid);

impl From<Uuid> for TaskId {
    fn from(id: Uuid) -> Self {
        Self(id)
    }
}

// ============================================================================
// §2 TaskState enum (5 态, per Jira 默认 workflow + plan-032 R6 line 99-108)
// ============================================================================

/// Task 状态 (5 态状态机, per Jira 默认 workflow, 跨 4 view 共享)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskState {
    /// 打开 (WBS pending, Jira "To Do"/"Open")
    Open,
    /// 进行中 (WBS in_progress, Jira "In Progress")
    InProgress,
    /// 评审中 (WBS in_review, Jira "In Review"/"Code Review")
    InReview,
    /// 完成 (WBS completed, Jira "Done"/"Resolved")
    Done,
    /// 关闭 (WBS closed, Jira "Closed")
    Closed,
}

impl TaskState {
    /// 从 star-task 7 态派生 (per DD §4.1)
    /// 7 态 (Pending/Claimed/Queued/Running/Review/Completed/Failed/Cancelled) → 5 态 (Open/InProgress/InReview/Done/Closed)
    pub fn from_star_task_state(s: &str) -> Self {
        match s {
            "Pending" | "Claimed" | "Queued" | "Failed" => Self::Open,
            "Running" | "InProgress" => Self::InProgress,
            "Review" | "PendingReview" => Self::InReview,
            "Completed" => Self::Done,
            "Cancelled" => Self::Closed, // 人工取消 = 业务关闭
            _ => Self::Open,             // 未知状态默认 Open (per 守门 #11 缺标比错标)
        }
    }
    /// 从 star-game Quest 4 态派生 (per DD §4.5)
    /// 4 态 (Available/InProgress/Completed/Failed) → 5 态
    pub fn from_quest_state(s: &str) -> Self {
        match s {
            "Available" => Self::Open,
            "InProgress" => Self::InProgress,
            "Completed" => Self::Done,
            "Failed" => Self::Closed, // Failed → Closed (业务关闭)
            _ => Self::Open,
        }
    }
    /// 字符串表达 (e.g. "Open")
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "Open",
            Self::InProgress => "InProgress",
            Self::InReview => "InReview",
            Self::Done => "Done",
            Self::Closed => "Closed",
        }
    }
}

// ============================================================================
// §3 Priority enum (4 档, per Jira priority scheme)
// ============================================================================

/// Task 优先级 (4 档, per Jira priority scheme)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum Priority {
    /// 低
    Low,
    /// 中 (default, per Jira)
    #[default]
    Medium,
    /// 高
    High,
    /// 紧急
    Critical,
}

impl Priority {
    /// 字符串表达
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
            Self::Critical => "Critical",
        }
    }
}

// ============================================================================
// §4 SharedTask struct (8 字段, per DD §2.1)
// ============================================================================

/// 共享 Task entity (WBS row 跨 3 view 表达, per DD §2.1)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SharedTask {
    /// 1️⃣ Task ID (per Multica opaque ID 1:1 派生)
    pub id: TaskId,
    /// 2️⃣ 标题 (e.g. "R5 阶段 1 实装", per Jira Issue.summary / Miro Node.content / MS Project Task.name)
    pub title: String,
    /// 3️⃣ 描述 (per Jira Issue.description / Miro Node.content 复用)
    pub description: String,
    /// 4️⃣ 状态 (5 态, per Jira 默认 workflow)
    pub state: TaskState,
    /// 5️⃣ 负责人 (e.g. "Mavis" / "5 域 Lead 名" / "subagent-1", per 守门 #3 + 9/11 23:11 JST 强化)
    pub assignee: Option<String>,
    /// 6️⃣ 优先级 (per Jira priority scheme)
    pub priority: Priority,
    /// 7️⃣ Issue Key (业务主键, e.g. "STAR-001", WBS row 跨 3 view 表达)
    pub issue_key: Option<String>,
    /// 8️⃣ 创建时间
    pub created_at: SystemTime,
}

impl SharedTask {
    /// 新建 SharedTask (默认 Open + Medium + now)
    pub fn new(id: TaskId, title: impl Into<String>) -> Self {
        Self {
            id,
            title: title.into(),
            description: String::new(),
            state: TaskState::Open,
            assignee: None,
            priority: Priority::default(),
            issue_key: None,
            created_at: SystemTime::now(),
        }
    }
}

// ============================================================================
// §5 Tests (R9 阶段 2 PoC 验证, 6 UT)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_id_from_uuid() {
        let uuid = Uuid::new_v4();
        let id: TaskId = uuid.into();
        assert_eq!(id.0, uuid);
    }

    #[test]
    fn task_state_5_variants() {
        // 验证 5 态完整
        assert_eq!(TaskState::Open.as_str(), "Open");
        assert_eq!(TaskState::InProgress.as_str(), "InProgress");
        assert_eq!(TaskState::InReview.as_str(), "InReview");
        assert_eq!(TaskState::Done.as_str(), "Done");
        assert_eq!(TaskState::Closed.as_str(), "Closed");
    }

    #[test]
    fn task_state_from_star_task_7_to_5_mapping() {
        // 7 态 → 5 态映射 (per DD §4.1)
        assert_eq!(TaskState::from_star_task_state("Pending"), TaskState::Open);
        assert_eq!(TaskState::from_star_task_state("Claimed"), TaskState::Open);
        assert_eq!(TaskState::from_star_task_state("Queued"), TaskState::Open);
        assert_eq!(
            TaskState::from_star_task_state("Running"),
            TaskState::InProgress
        );
        assert_eq!(
            TaskState::from_star_task_state("Review"),
            TaskState::InReview
        );
        assert_eq!(
            TaskState::from_star_task_state("Completed"),
            TaskState::Done
        );
        assert_eq!(TaskState::from_star_task_state("Failed"), TaskState::Open);
    }

    #[test]
    fn task_state_from_quest_4_to_5_mapping() {
        // 4 态 → 5 态映射 (per DD §4.5)
        assert_eq!(TaskState::from_quest_state("Available"), TaskState::Open);
        assert_eq!(
            TaskState::from_quest_state("InProgress"),
            TaskState::InProgress
        );
        assert_eq!(TaskState::from_quest_state("Completed"), TaskState::Done);
        assert_eq!(TaskState::from_quest_state("Failed"), TaskState::Closed);
    }

    #[test]
    fn priority_default_is_medium() {
        assert_eq!(Priority::default(), Priority::Medium);
        assert_eq!(Priority::Medium.as_str(), "Medium");
    }

    #[test]
    fn shared_task_new_defaults() {
        let id = TaskId(Uuid::new_v4());
        let task = SharedTask::new(id, "test task");
        assert_eq!(task.id, id);
        assert_eq!(task.title, "test task");
        assert_eq!(task.state, TaskState::Open);
        assert_eq!(task.priority, Priority::Medium);
        assert_eq!(task.assignee, None);
        assert_eq!(task.issue_key, None);
        assert_eq!(task.description, "");
    }
}
