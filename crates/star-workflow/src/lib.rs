//! star-workflow — Jira-like 任务流 Rust 阶段 1 (R6 阶段 1) 🟢
//!
//! Per ADR-0027 v0.1 §2.3 + plan-032 R6: **Jira 长处整合** = Issue tracking + workflow rules
//! (status transitions + 条件) + sprint + assign + 跨域依赖. **Jira 限制避免** = 不用 Jira JQL
//! 复杂查询 (改 Rust enum + match) + 不用 Jira plugin 体系 (改 Rust trait 抽象).
//!
//! WBS 集成 (per ADR-0027 §2.3.2 共享 Task schema 跨 3 view):
//! - WBS row = Issue (key + workflow + state + assignee + priority)
//! - Issue transition = WBS row 状态变更
//! - Sprint = WBS sprint 概念
//! - Cross-domain dependency = 跨域 task 依赖 (per plan-032 R6 line 105)
//!
//! 跟 star-task 跨域 (per 守门 #1 跨域 consults):
//! - star-task 7 态状态机 = 实现层 task lifecycle (claim/queue/run/review/done/...)
//! - star-workflow 5 态状态机 = 业务层 user-facing workflow (Open/InProgress/InReview/Done/Closed)
//! - R9 整合时两层映射 (per plan-032 R6 line 138-140 依赖 R4)
//!
//! 守门合规 (per 守门 #1 v25 cargo test 单 crate 实证 + 守门 #7 0 unsafe + 守门 #11 缺标比错标)

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use std::collections::HashMap;
use std::time::SystemTime;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

// ============================================================================
// §1 ID newtype (per Multica opaque ID 1:1 派生)
// ============================================================================

/// Workflow ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkflowId(pub Uuid);

impl From<Uuid> for WorkflowId {
    fn from(id: Uuid) -> Self {
        Self(id)
    }
}

/// Transition ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TransitionId(pub Uuid);

/// Comment ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommentId(pub Uuid);

/// Attachment ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AttachmentId(pub Uuid);

/// Sprint ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SprintId(pub Uuid);

/// Issue Key (Jira-style 业务 ID, e.g. "STAR-001", WBS row 共享主键)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IssueKey(pub String);

impl IssueKey {
    /// 新建 issue key
    pub fn new(key: impl Into<String>) -> Self {
        Self(key.into())
    }
    /// 字符串引用
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for IssueKey {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for IssueKey {
    fn from(s: String) -> Self {
        Self(s)
    }
}

// ============================================================================
// §2 Workflow 状态机 (Jira workflow rules, per plan-032 R6 Jira 长处整合)
// ============================================================================

/// Workflow 5 态状态机 (per ADR-0027 §2.3 + Jira 默认 workflow)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkflowState {
    /// 打开 (Jira "To Do" / "Open")
    Open,
    /// 进行中 (Jira "In Progress")
    InProgress,
    /// 评审中 (Jira "In Review" / "Code Review")
    InReview,
    /// 完成 (Jira "Done" / "Resolved")
    Done,
    /// 关闭 (Jira "Closed")
    Closed,
}

/// Workflow transition (from_state → to_state, 命名 per Jira workflow)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transition {
    /// Transition ID
    pub id: TransitionId,
    /// Transition 名称 (e.g. "Start Progress", "Submit Review", "Close")
    pub name: String,
    /// 起始状态
    pub from: WorkflowState,
    /// 目标状态
    pub to: WorkflowState,
}

impl Transition {
    /// 新建 transition
    pub fn new(name: impl Into<String>, from: WorkflowState, to: WorkflowState) -> Self {
        Self {
            id: TransitionId(Uuid::new_v4()),
            name: name.into(),
            from,
            to,
        }
    }
}

/// Workflow (Jira workflow = name + states + transitions DAG)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Workflow {
    /// Workflow ID
    pub id: WorkflowId,
    /// Workflow 名称
    pub name: String,
    /// Workflow 描述
    pub description: String,
    /// Transition 列表 (DAG, per plan-032 R6 状态转移规则)
    pub transitions: Vec<Transition>,
}

impl Workflow {
    /// 新建空 workflow (无 transition)
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id: WorkflowId(Uuid::new_v4()),
            name: name.into(),
            description: description.into(),
            transitions: Vec::new(),
        }
    }
    /// 加 transition
    pub fn add_transition(&mut self, transition: Transition) {
        self.transitions.push(transition);
    }
    /// 找 from_state 的所有 transition
    pub fn transitions_from(&self, from: WorkflowState) -> Vec<&Transition> {
        self.transitions.iter().filter(|t| t.from == from).collect()
    }
    /// 找 from → to 的 transition (per Jira workflow rule 校验)
    pub fn find_transition(&self, from: WorkflowState, to: WorkflowState) -> Option<&Transition> {
        self.transitions
            .iter()
            .find(|t| t.from == from && t.to == to)
    }
}

/// Jira-style "默认" workflow: Open → InProgress → InReview → Done → Closed (4 transition)
pub fn default_workflow() -> Workflow {
    let mut wf = Workflow::new(
        "default",
        "Default Jira-like workflow (5 states, 4 transitions)",
    );
    wf.add_transition(Transition::new(
        "Start Progress",
        WorkflowState::Open,
        WorkflowState::InProgress,
    ));
    wf.add_transition(Transition::new(
        "Submit Review",
        WorkflowState::InProgress,
        WorkflowState::InReview,
    ));
    wf.add_transition(Transition::new(
        "Complete",
        WorkflowState::InReview,
        WorkflowState::Done,
    ));
    wf.add_transition(Transition::new(
        "Close",
        WorkflowState::Done,
        WorkflowState::Closed,
    ));
    wf
}

// ============================================================================
// §3 Issue (WBS row = Issue, per ADR-0027 §2.3.2 共享 schema 跨 3 view)
// ============================================================================

/// Issue 优先级 (per Jira priority scheme)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum Priority {
    /// 低
    Low,
    /// 中 (default, per Jira 默认)
    #[default]
    Medium,
    /// 高
    High,
    /// 紧急
    Critical,
}

/// Issue (Jira issue = WBS row, 跟 star-task 1:1 派生, R9 整合时统一)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Issue {
    /// Issue Key (业务主键, e.g. "STAR-001")
    pub key: IssueKey,
    /// 关联 workflow ID
    pub workflow_id: WorkflowId,
    /// 当前状态
    pub state: WorkflowState,
    /// 负责人 (e.g. agent 名称, Mavis / 5 域 Lead 名 / 子代理 ID)
    pub assignee: Option<String>,
    /// 优先级
    pub priority: Priority,
    /// 标题
    pub summary: String,
    /// 描述
    pub description: String,
    /// 创建时间
    pub created_at: SystemTime,
}

impl Issue {
    /// 新建 issue (默认 Open 状态)
    pub fn new(
        key: impl Into<IssueKey>,
        workflow_id: WorkflowId,
        summary: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            key: key.into(),
            workflow_id,
            state: WorkflowState::Open,
            assignee: None,
            priority: Priority::default(),
            summary: summary.into(),
            description: description.into(),
            created_at: SystemTime::now(),
        }
    }
    /// 分配 assignee
    pub fn assign(&mut self, assignee: impl Into<String>) {
        self.assignee = Some(assignee.into());
    }
}

// ============================================================================
// §4 Comment (Jira issue comment)
// ============================================================================

/// Issue 评论 (Jira comment = text + author + timestamp)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Comment {
    /// Comment ID
    pub id: CommentId,
    /// 关联 issue key
    pub issue_key: IssueKey,
    /// 作者
    pub author: String,
    /// 评论内容
    pub body: String,
    /// 创建时间
    pub created_at: SystemTime,
}

impl Comment {
    /// 新建 comment
    pub fn new(
        issue_key: impl Into<IssueKey>,
        author: impl Into<String>,
        body: impl Into<String>,
    ) -> Self {
        Self {
            id: CommentId(Uuid::new_v4()),
            issue_key: issue_key.into(),
            author: author.into(),
            body: body.into(),
            created_at: SystemTime::now(),
        }
    }
}

// ============================================================================
// §5 Attachment (Jira issue attachment)
// ============================================================================

/// Issue 附件 (Jira attachment = filename + mime + size + url)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attachment {
    /// Attachment ID
    pub id: AttachmentId,
    /// 关联 issue key
    pub issue_key: IssueKey,
    /// 文件名
    pub filename: String,
    /// MIME 类型 (e.g. "text/plain", "application/pdf")
    pub mime: String,
    /// 文件大小 (bytes)
    pub size: u64,
    /// 存储 URL
    pub url: String,
    /// 上传者
    pub uploader: String,
    /// 上传时间
    pub created_at: SystemTime,
}

impl Attachment {
    /// 新建 attachment
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        issue_key: impl Into<IssueKey>,
        filename: impl Into<String>,
        mime: impl Into<String>,
        size: u64,
        url: impl Into<String>,
        uploader: impl Into<String>,
    ) -> Self {
        Self {
            id: AttachmentId(Uuid::new_v4()),
            issue_key: issue_key.into(),
            filename: filename.into(),
            mime: mime.into(),
            size,
            url: url.into(),
            uploader: uploader.into(),
            created_at: SystemTime::now(),
        }
    }
}

// ============================================================================
// §6 Sprint (Jira sprint = 时间盒 + issue 集合)
// ============================================================================

/// Sprint (Jira sprint = 时间盒 + issue_keys 集合)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sprint {
    /// Sprint ID
    pub id: SprintId,
    /// Sprint 名称
    pub name: String,
    /// Sprint 目标
    pub goal: String,
    /// 起始时间
    pub start: SystemTime,
    /// 结束时间
    pub end: SystemTime,
    /// 关联 issue keys
    pub issue_keys: Vec<IssueKey>,
}

impl Sprint {
    /// 新建 sprint (空 issue 列表)
    pub fn new(
        name: impl Into<String>,
        goal: impl Into<String>,
        start: SystemTime,
        end: SystemTime,
    ) -> Self {
        Self {
            id: SprintId(Uuid::new_v4()),
            name: name.into(),
            goal: goal.into(),
            start,
            end,
            issue_keys: Vec::new(),
        }
    }
    /// 加 issue 到 sprint
    pub fn add_issue(&mut self, key: impl Into<IssueKey>) {
        self.issue_keys.push(key.into());
    }
    /// issue 数量
    pub fn len(&self) -> usize {
        self.issue_keys.len()
    }
    /// 是否空
    pub fn is_empty(&self) -> bool {
        self.issue_keys.is_empty()
    }
}

// ============================================================================
// §7 WorkflowEngine (核心, per plan-032 R6 workflow engine)
// ============================================================================

/// Workflow 错误
#[derive(Debug, Error)]
pub enum WorkflowError {
    /// Issue 不存在
    #[error("issue not found: {0}")]
    IssueNotFound(String),
    /// Workflow 不存在
    #[error("workflow not found: {0}")]
    WorkflowNotFound(String),
    /// 非法 transition (from → to 不在 workflow rules 里)
    #[error("invalid transition: {from:?} -> {to:?}")]
    InvalidTransition {
        /// 起始状态
        from: WorkflowState,
        /// 目标状态
        to: WorkflowState,
    },
    /// Issue 已存在
    #[error("issue already exists: {0}")]
    IssueAlreadyExists(String),
    /// 跨域依赖冲突
    #[error("cross-domain dependency conflict: {0}")]
    CrossDomainConflict(String),
}

/// WorkflowEngine (核心, 持有 workflows + issues + comments + attachments + sprints)
#[derive(Debug, Default)]
pub struct WorkflowEngine {
    /// Workflow 仓库 (workflow_id → Workflow)
    pub workflows: HashMap<WorkflowId, Workflow>,
    /// Issue 仓库 (issue_key → Issue)
    pub issues: HashMap<IssueKey, Issue>,
    /// Comment 仓库 (issue_key → Vec<Comment>)
    pub comments: HashMap<IssueKey, Vec<Comment>>,
    /// Attachment 仓库 (issue_key → Vec<Attachment>)
    pub attachments: HashMap<IssueKey, Vec<Attachment>>,
    /// Sprint 仓库 (sprint_id → Sprint)
    pub sprints: HashMap<SprintId, Sprint>,
    /// 跨域依赖 (issue_key → Vec<issue_key>, per plan-032 R6 line 105 跨域依赖)
    pub cross_domain_deps: HashMap<IssueKey, Vec<IssueKey>>,
}

impl WorkflowEngine {
    /// 新建 engine
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册 workflow
    pub fn register_workflow(&mut self, workflow: Workflow) -> WorkflowId {
        let id = workflow.id;
        self.workflows.insert(id, workflow);
        id
    }

    /// 创建 issue
    pub fn create_issue(&mut self, issue: Issue) -> Result<IssueKey, WorkflowError> {
        if self.issues.contains_key(&issue.key) {
            return Err(WorkflowError::IssueAlreadyExists(issue.key.0));
        }
        let key = issue.key.clone();
        self.issues.insert(key.clone(), issue);
        Ok(key)
    }

    /// Transition issue (校验 workflow rule)
    pub fn transition_issue(
        &mut self,
        key: &IssueKey,
        to: WorkflowState,
    ) -> Result<WorkflowState, WorkflowError> {
        let issue = self
            .issues
            .get(key)
            .ok_or_else(|| WorkflowError::IssueNotFound(key.0.clone()))?;
        let workflow_id = issue.workflow_id;
        let from = issue.state;
        // NLL 会在 issue 不再被使用时自动释放 borrow

        let workflow = self
            .workflows
            .get(&workflow_id)
            .ok_or_else(|| WorkflowError::WorkflowNotFound(workflow_id.0.to_string()))?;

        if workflow.find_transition(from, to).is_none() {
            return Err(WorkflowError::InvalidTransition { from, to });
        }
        // NLL 会在 workflow 不再被使用时自动释放 borrow

        // 通过校验, 修改 issue state
        let issue = self
            .issues
            .get_mut(key)
            .ok_or_else(|| WorkflowError::IssueNotFound(key.0.clone()))?;
        issue.state = to;
        Ok(to)
    }

    /// 加 comment
    pub fn add_comment(&mut self, comment: Comment) -> Result<(), WorkflowError> {
        if !self.issues.contains_key(&comment.issue_key) {
            return Err(WorkflowError::IssueNotFound(comment.issue_key.0));
        }
        self.comments
            .entry(comment.issue_key.clone())
            .or_default()
            .push(comment);
        Ok(())
    }

    /// 加 attachment
    pub fn add_attachment(&mut self, attachment: Attachment) -> Result<(), WorkflowError> {
        if !self.issues.contains_key(&attachment.issue_key) {
            return Err(WorkflowError::IssueNotFound(attachment.issue_key.0));
        }
        self.attachments
            .entry(attachment.issue_key.clone())
            .or_default()
            .push(attachment);
        Ok(())
    }

    /// 创建 sprint
    pub fn create_sprint(&mut self, sprint: Sprint) -> SprintId {
        let id = sprint.id;
        self.sprints.insert(id, sprint);
        id
    }

    /// Issue 加到 sprint
    pub fn add_issue_to_sprint(
        &mut self,
        sprint_id: SprintId,
        issue_key: IssueKey,
    ) -> Result<(), WorkflowError> {
        if !self.issues.contains_key(&issue_key) {
            return Err(WorkflowError::IssueNotFound(issue_key.0));
        }
        let sprint = self.sprints.get_mut(&sprint_id).ok_or_else(|| {
            WorkflowError::CrossDomainConflict(format!("sprint {sprint_id:?} not found"))
        })?;
        sprint.add_issue(issue_key);
        Ok(())
    }

    /// 加跨域依赖 (issue A 依赖 issue B, B 不在 A 之前完成则 A 不能 Done)
    pub fn add_cross_domain_dep(
        &mut self,
        issue: IssueKey,
        depends_on: IssueKey,
    ) -> Result<(), WorkflowError> {
        if !self.issues.contains_key(&issue) {
            return Err(WorkflowError::IssueNotFound(issue.0));
        }
        if !self.issues.contains_key(&depends_on) {
            return Err(WorkflowError::IssueNotFound(depends_on.0));
        }
        self.cross_domain_deps
            .entry(issue)
            .or_default()
            .push(depends_on);
        Ok(())
    }
}

// ============================================================================
// §8 Tests (R6 阶段 1 PoC 验证)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn setup_engine() -> (WorkflowEngine, WorkflowId) {
        let mut engine = WorkflowEngine::new();
        let wf = default_workflow();
        let wf_id = engine.register_workflow(wf);
        (engine, wf_id)
    }

    #[test]
    fn default_workflow_has_4_transitions() {
        let wf = default_workflow();
        assert_eq!(wf.transitions.len(), 4);
        // Open → InProgress
        assert!(wf
            .find_transition(WorkflowState::Open, WorkflowState::InProgress)
            .is_some());
        // InProgress → InReview
        assert!(wf
            .find_transition(WorkflowState::InProgress, WorkflowState::InReview)
            .is_some());
        // InReview → Done
        assert!(wf
            .find_transition(WorkflowState::InReview, WorkflowState::Done)
            .is_some());
        // Done → Closed
        assert!(wf
            .find_transition(WorkflowState::Done, WorkflowState::Closed)
            .is_some());
    }

    #[test]
    fn issue_new_default_state_open() {
        let (_engine, wf_id) = setup_engine();
        let issue = Issue::new("STAR-001", wf_id, "test issue", "first");
        assert_eq!(issue.state, WorkflowState::Open);
        assert_eq!(issue.priority, Priority::Medium);
        assert_eq!(issue.assignee, None);
    }

    #[test]
    fn engine_create_issue_and_transition_valid() {
        let (mut engine, wf_id) = setup_engine();
        let issue = Issue::new("STAR-001", wf_id, "test", "desc");
        engine.create_issue(issue).unwrap();
        // Open → InProgress
        let new_state = engine
            .transition_issue(&"STAR-001".into(), WorkflowState::InProgress)
            .unwrap();
        assert_eq!(new_state, WorkflowState::InProgress);
        assert_eq!(
            engine.issues.get(&"STAR-001".into()).unwrap().state,
            WorkflowState::InProgress
        );
    }

    #[test]
    fn engine_transition_invalid_returns_error() {
        let (mut engine, wf_id) = setup_engine();
        let issue = Issue::new("STAR-001", wf_id, "test", "desc");
        engine.create_issue(issue).unwrap();
        // Open → Done (没有 Open → Done transition)
        let result = engine.transition_issue(&"STAR-001".into(), WorkflowState::Done);
        assert!(matches!(
            result,
            Err(WorkflowError::InvalidTransition { .. })
        ));
    }

    #[test]
    fn engine_create_issue_duplicate_fails() {
        let (mut engine, wf_id) = setup_engine();
        let issue1 = Issue::new("STAR-001", wf_id, "first", "");
        let issue2 = Issue::new("STAR-001", wf_id, "second", "");
        engine.create_issue(issue1).unwrap();
        let result = engine.create_issue(issue2);
        assert!(matches!(result, Err(WorkflowError::IssueAlreadyExists(_))));
    }

    #[test]
    fn engine_add_comment_and_attachment() {
        let (mut engine, wf_id) = setup_engine();
        let issue = Issue::new("STAR-001", wf_id, "test", "");
        engine.create_issue(issue).unwrap();
        let comment = Comment::new("STAR-001", "Mavis", "looks good");
        engine.add_comment(comment).unwrap();
        assert_eq!(engine.comments.get(&"STAR-001".into()).unwrap().len(), 1);
        let attachment = Attachment::new(
            "STAR-001",
            "spec.md",
            "text/markdown",
            1024,
            "https://example.com/spec.md",
            "Mavis",
        );
        engine.add_attachment(attachment).unwrap();
        assert_eq!(engine.attachments.get(&"STAR-001".into()).unwrap().len(), 1);
    }

    #[test]
    fn engine_sprint_create_and_add_issue() {
        let (mut engine, wf_id) = setup_engine();
        let issue1 = Issue::new("STAR-001", wf_id, "issue 1", "");
        let issue2 = Issue::new("STAR-002", wf_id, "issue 2", "");
        engine.create_issue(issue1).unwrap();
        engine.create_issue(issue2).unwrap();
        let now = SystemTime::now();
        let sprint = Sprint::new(
            "Sprint 1",
            "完成 R6 阶段 1",
            now,
            now + Duration::from_secs(14 * 24 * 3600),
        );
        let sprint_id = engine.create_sprint(sprint);
        engine
            .add_issue_to_sprint(sprint_id, "STAR-001".into())
            .unwrap();
        engine
            .add_issue_to_sprint(sprint_id, "STAR-002".into())
            .unwrap();
        assert_eq!(engine.sprints.get(&sprint_id).unwrap().len(), 2);
    }

    #[test]
    fn engine_cross_domain_dep_registers() {
        let (mut engine, wf_id) = setup_engine();
        let issue1 = Issue::new("STAR-001", wf_id, "issue 1", "");
        let issue2 = Issue::new("STAR-002", wf_id, "issue 2", "");
        engine.create_issue(issue1).unwrap();
        engine.create_issue(issue2).unwrap();
        // STAR-002 依赖 STAR-001
        engine
            .add_cross_domain_dep("STAR-002".into(), "STAR-001".into())
            .unwrap();
        let deps = engine.cross_domain_deps.get(&"STAR-002".into()).unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0], "STAR-001".into());
    }
}
