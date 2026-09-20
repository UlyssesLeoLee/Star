//! `action.rs` — Action Trait + 18 Action enum + 3 类别 (per DD §12.1 + §18 + INV-WC-09)
//!
//! 18 Action 清单 (per BD §12.1 + DD §12.1 self-review 修正 Merge=Destructive):
//!
//! **Safe (6)** — 无副作用:
//! - Open, OpenInIDE, Compare, Focus, ExplainRisk, ListEvents
//!
//! **Warning (5)** — 有副作用但可撤销 (per issue description; BD 拍板 8,
//!   ULYS-57.3 自审调整到 5: SyncMain, Rebase, CreatePR, Archive, MarkSuperseded):
//! - SyncMain, Rebase, CreatePR, Archive, MarkSuperseded
//!
//! **Destructive (7)** — 高风险不可撤销 (per issue description: 7 = 5 baseline + Merge + ForceDelete):
//! - Delete, Cleanup, ForceMerge, ForceRebase, ForceDelete, Merge, RemoveDependency
//!
//! 注: 跟 BD §12.1 计数不一致是因为:
//! - BD §12.1 原本是 Safe 5 + Warning 8 + Destructive 5 = 18, 但 ULYS-57.3 任务描述
//!   明确"per spec §4.4 Merge=Destructive 修正", 即将 Merge 从 Warning 重分类到 Destructive
//! - Warning 的 8 项中 Lock/Unlock/SetDependency 跟 Destructive 边界模糊, 落到 Safe/Warning
//!   自审调整: 本期 Safe 6 + Warning 5 + Destructive 7 = 18 (Merge 单列为 Destructive)

use async_trait::async_trait;
use graph_core::types::{UserId, WorktreeId};
use serde::{Deserialize, Serialize};

use crate::error::ActionError;

/// Action 类别 (per DD §12.1)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionClass {
    /// Safe — 无副作用, 不需要确认
    Safe,
    /// Warning — 有副作用但可撤销, 走二次确认 (轻量)
    Warning,
    /// Destructive — 高风险不可撤销, 走二次确认 (typed, e.g. type "DELETE")
    Destructive,
}

/// ActionType — 18 Action 完整清单 (per ULYS-57.3 T8 acceptance criterion)
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    // ==== Safe (6) ====
    /// Open Worktree in file browser
    #[default]
    Open,
    /// Open in VSCode / Cursor
    OpenInIDE,
    /// Compare 2 Worktrees (read-only diff)
    Compare,
    /// Enter Focus Mode
    Focus,
    /// AI Explanation (read-only)
    ExplainRisk,
    /// List events
    ListEvents,
    // ==== Warning (5) ====
    /// git fetch + rebase onto main
    SyncMain,
    /// git rebase onto target branch
    Rebase,
    /// Open PR via GitHub/GitLab API
    CreatePR,
    /// Archive (not delete)
    Archive,
    /// Mark Worktree as Superseded
    MarkSuperseded,
    // ==== Destructive (7) ====
    /// git worktree remove + branch delete
    Delete,
    /// Batch delete STALE Worktrees
    Cleanup,
    /// Merge ignoring conflicts (force)
    ForceMerge,
    /// Rebase dropping commits (force)
    ForceRebase,
    /// Delete without confirmation (force)
    ForceDelete,
    /// Merge Worktree into main (per spec §4.4 自审修正: Warning → Destructive)
    Merge,
    /// Remove DEPENDS_ON Edge (irreversible)
    RemoveDependency,

    // ==== AI (3) -- ULYS-98-W4.2 (per docs/briefs/ulys-98-star-cursor-min-v1.md §Sub-task 4.2) ====
    /// AI chat completion (per W3.3 useChatStream hook).
    AiChat,
    /// AI inline code completion (per W2.3 inlineCompletion.ts).
    AiCompletion,
    /// AI multi-file composer diff (per W3.5 ComposerPanel.tsx).
    AiComposerEdit,
}

impl ActionType {
    /// 18 Action 总数 (per INV-WC-09 守门)
    pub const COUNT: usize = 21;

    /// 全部 18 个 Action (按 enum 顺序)
    pub const ALL: [ActionType; Self::COUNT] = [
        // Safe
        Self::Open,
        Self::OpenInIDE,
        Self::Compare,
        Self::Focus,
        Self::ExplainRisk,
        Self::ListEvents,
        // Warning
        Self::SyncMain,
        Self::Rebase,
        Self::CreatePR,
        Self::Archive,
        Self::MarkSuperseded,
        // Destructive
        Self::Delete,
        Self::Cleanup,
        Self::ForceMerge,
        Self::ForceRebase,
        Self::ForceDelete,
        Self::Merge,
        Self::RemoveDependency,
        // AI (per W4.2)
        Self::AiChat,
        Self::AiCompletion,
        Self::AiComposerEdit,
    ];
}

/// Action metadata (分类 + 描述 + 确认需求 + 可逆)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionMetadata {
    /// Action 类型
    pub action_type: ActionType,
    /// 分类 (Safe / Warning / Destructive)
    pub classification: ActionClass,
    /// 人类可读描述
    pub description: &'static str,
    /// 是否要求二次确认 (Destructive 必为 true)
    pub requires_confirm: bool,
    /// 是否可逆 (Delete = false; Archive = true)
    pub reversible: bool,
}

/// Action 请求 (入口参数)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRequest {
    /// Action 类型
    pub action_type: ActionType,
    /// 目标 Worktree ID
    pub worktree_id: WorktreeId,
    /// 操作人 (RBAC 决策)
    pub user_id: UserId,
    /// 客户端是否已二次确认 (per FR-ACTION-003, Warning/Destructive 必填 true)
    pub confirm: bool,
    /// 参数 (e.g. merge 策略, rebase target)
    #[serde(default)]
    pub params: serde_json::Value,
}

impl Default for ActionRequest {
    fn default() -> Self {
        Self {
            action_type: ActionType::default(),
            worktree_id: WorktreeId::nil(),
            user_id: UserId::nil(),
            confirm: false,
            params: serde_json::Value::Null,
        }
    }
}

/// Action 结果 (出口)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    /// 是否成功
    pub success: bool,
    /// Worktree ID
    pub worktree_id: WorktreeId,
    /// 新 HumanState (per graph-core::state::HumanState 简化版: stringly)
    pub new_state: Option<String>,
    /// 耗时 (ms)
    pub duration_ms: u64,
    /// 警告 (非致命)
    pub warnings: Vec<ActionWarning>,
    /// 错误 (致命; success=false 时至少 1 项)
    pub errors: Vec<String>,
}

/// Action 警告 (非致命)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionWarning {
    /// 警告码 (e.g. "WARN.SLOW_PATH")
    pub code: String,
    /// 人类可读消息
    pub message: String,
}

/// Action 上下文 (执行所需依赖, 由 `ActionEngine` 注入)
#[derive(Debug, Clone, Default)]
pub struct ActionContext {
    /// Action 请求
    pub request: ActionRequest,
    /// 当前用户角色 (per RBAC 决策)
    pub user_role: crate::rbac::Role,
    /// 调用方 trace id
    pub trace_id: String,
}

/// Action Trait (per DD §12.2 / §18.2)
#[async_trait]
pub trait Action: Send + Sync {
    /// Action metadata
    fn metadata(&self) -> ActionMetadata;

    /// 校验 (RBAC + pre-condition)
    async fn validate(&self, ctx: &ActionContext) -> Result<(), ActionError>;

    /// 执行 (git op + graph update + audit + event publish)
    async fn execute(&self, ctx: &ActionContext) -> Result<ActionResult, ActionError>;
}

/// 给 ActionType 返回默认 metadata (工厂表, 18 项一一对应)
///
/// 分类严格遵循 INV-WC-09:
///
/// - Safe: Open / OpenInIDE / Compare / Focus / ExplainRisk / ListEvents
/// - Warning: SyncMain / Rebase / CreatePR / Archive / MarkSuperseded
/// - Destructive: Delete / Cleanup / ForceMerge / ForceRebase / ForceDelete / Merge / RemoveDependency
pub fn default_metadata(action_type: ActionType) -> ActionMetadata {
    use ActionClass::*;
    use ActionType::*;
    let (classification, description, requires_confirm, reversible) = match action_type {
        // Safe
        Open => (Safe, "Open Worktree in file browser", false, true),
        OpenInIDE => (Safe, "Open in VSCode / Cursor", false, true),
        Compare => (Safe, "Compare 2 Worktrees (read-only diff)", false, true),
        Focus => (Safe, "Enter Focus Mode", false, true),
        ExplainRisk => (Safe, "AI Explanation (read-only)", false, true),
        ListEvents => (Safe, "List events for Worktree", false, true),
        // AI Actions (W4.2: Safe class, read-only by default)
        AiChat => (Safe, "AI chat stream (per W3.2 SSE handler)", false, true),
        AiCompletion => (Safe, "AI inline code completion (per W2.3)", false, true),
        AiComposerEdit => (Safe, "AI composer edit (per W3.5)", false, true),
        // Warning
        SyncMain => (Warning, "git fetch + rebase onto main", true, true),
        Rebase => (Warning, "git rebase onto target branch", true, true),
        CreatePR => (Warning, "Open PR via GitHub/GitLab API", true, true),
        Archive => (Warning, "Archive (not delete)", true, true),
        MarkSuperseded => (Warning, "Mark Worktree as Superseded", true, true),
        // Destructive
        Delete => (
            Destructive,
            "git worktree remove + branch delete",
            true,
            false,
        ),
        Cleanup => (Destructive, "Batch delete STALE Worktrees", true, false),
        ForceMerge => (Destructive, "Merge ignoring conflicts", true, false),
        ForceRebase => (Destructive, "Rebase dropping commits", true, false),
        ForceDelete => (Destructive, "Delete without confirmation", true, false),
        // per spec §4.4 自审修正: Warning → Destructive
        Merge => (Destructive, "Merge Worktree into main", true, false),
        RemoveDependency => (
            Destructive,
            "Remove DEPENDS_ON Edge (irreversible)",
            true,
            false,
        ),
    };
    ActionMetadata {
        action_type,
        classification,
        description,
        requires_confirm,
        reversible,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_18_types_register_correctly() {
        // 守门 UT-1: 18 Action 全枚举可达
        assert_eq!(ActionType::COUNT, 21);
        assert_eq!(ActionType::ALL.len(), 21);
        // 每项 metadata 可取
        for at in ActionType::ALL.iter().copied() {
            let m = default_metadata(at);
            assert_eq!(m.action_type, at);
            assert!(!m.description.is_empty());
        }
    }

    #[test]
    fn action_destructive_requires_confirmation() {
        // 守门 UT-2: Destructive 全部 requires_confirm=true (per FR-ACTION-003)
        for at in ActionType::ALL.iter().copied() {
            let m = default_metadata(at);
            if m.classification == ActionClass::Destructive {
                assert!(
                    m.requires_confirm,
                    "destructive {at:?} must require confirmation"
                );
            }
        }
        // 计数 (per ULYS-57.3 + ULYS-98 v1.0.1 AI Actions: Safe 9 + Warning 5 + Destructive 7 = 21)
        let mut safe = 0;
        let mut warning = 0;
        let mut destructive = 0;
        for at in ActionType::ALL.iter().copied() {
            match default_metadata(at).classification {
                ActionClass::Safe => safe += 1,
                ActionClass::Warning => warning += 1,
                ActionClass::Destructive => destructive += 1,
            }
        }
        assert_eq!(safe, 9);
        assert_eq!(warning, 5);
        assert_eq!(destructive, 7);
        assert_eq!(safe + warning + destructive, 21);
    }

    #[test]
    fn merge_is_destructive_per_self_review() {
        // 守门: per spec §4.4 / ULYS-57.3 自审修正 — Merge 必须 Destructive
        let m = default_metadata(ActionType::Merge);
        assert_eq!(m.classification, ActionClass::Destructive);
        assert!(m.requires_confirm);
    }
}
