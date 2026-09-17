//! `state.rs` — 跨域枚举类型 (per DD §2.1 + §2.3 + §2.4 + §7.3 + §7.5 + §7.6 + §7.7 + §8.1.10 + §8.1.13)
//!
//! 这些枚举由 graph-core 定义, 跨 crate 复用:
//! - `HumanState` (DD §2.1) — 7 态 Worktree UI 状态 (RUNNING / WAITING / READY / DIVERGED / CONFLICT / MERGED / STALE)
//! - `MachineState` (DD §2.1) — 12 态 Worktree 机器内部状态
//! - `MergeStrategy` (DD §8.1.13) — Merge / Squash / Rebase / FastForward
//! - `AgentStatus` (DD §2.3) — 14 态 (per SRS-STAR-AGENT-RUNTIME-001)
//! - `AgentResultState` (DD §2.3) — Success / Partial / Failed / Pending
//! - `TaskStatus` (DD §2.4) — 6 态 (Todo / InProgress / Review / Blocked / Done / Wontfix)
//! - `TestStatus` (DD §7.6) — Passed / Failed / Running / Skipped
//! - `PRState` (DD §7.3) — Open / Merged / Closed
//! - `IssueState` (DD §7.7) — Open / Closed
//! - `SymbolKind` (DD §7.5) — Function / Class / Variable / Type / Enum / Trait / Interface
//! - `TestState` (DD §2.1) — Passed / Failed / Running / None (UI 简化状态)
//! - `TokenUsage` (DD §2.3) — input/output/total
//!
//! 跨 crate 引用 (per WORKTREE-CANVAS-IMPL-PLAN-001 §3 集成策略):
//! - worktree-service 用 HumanState/MachineState/TestState
//! - risk-engine 用 MergeStrategy
//! - agent-bridge 用 AgentStatus/AgentResultState/TokenUsage
//! - canvas-renderer 用 PRState/IssueState
//! - file-indexer 用 SymbolKind

use serde::{Deserialize, Serialize};

/// 7 态 Worktree UI 状态 (per DD §2.1, SRS §9)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HumanState {
    /// Agent 正在执行
    Running,
    /// 等待人类输入
    Waiting,
    /// 已就绪, 可合并
    Ready,
    /// 与 main 分叉过大
    Diverged,
    /// Merge conflict
    Conflict,
    /// 已合并
    Merged,
    /// 长期无活动
    Stale,
}

/// 12 态 Worktree 机器内部状态 (per DD §2.1)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MachineState {
    /// Agent 正在执行
    AgentExecuting,
    /// Working tree clean
    GitClean,
    /// Working tree dirty
    GitDirty,
    /// Merge 进行中
    MergeInProgress,
    /// Rebase 进行中
    RebaseInProgress,
    /// CI 运行中
    CiRunning,
    /// 等待人类输入
    AwaitingHuman,
    /// 等待 Agent feedback
    AwaitingFeedback,
    /// 等待工具调用返回
    AwaitingTool,
    /// 干净合并后
    MergedClean,
    /// 已归档
    Archived,
    /// 已删除
    Deleted,
}

/// 4 档 Merge 策略 (per DD §8.1.13)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MergeStrategy {
    /// git merge (no-ff)
    Merge,
    /// git merge --squash
    Squash,
    /// git rebase
    Rebase,
    /// fast-forward
    FastForward,
}

/// 14 态 Agent status (per DD §2.3, SRS-STAR-AGENT-RUNTIME-001)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentStatus {
    /// 已注册但未启动
    Registered,
    /// 启动中
    Starting,
    /// 等待任务分配
    Idle,
    /// 任务已分配未开始
    Claimed,
    /// 思考中
    Thinking,
    /// 工具调用中
    ToolCalling,
    /// 等待工具返回
    AwaitingTool,
    /// 生成回复中
    Streaming,
    /// 等待人类反馈
    AwaitingFeedback,
    /// 任务完成
    Completed,
    /// 任务失败
    Failed,
    /// 已暂停
    Paused,
    /// 已停止
    Stopped,
    /// 异常退出
    Crashed,
}

/// Agent result state (per DD §2.3)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentResultState {
    /// 全部成功
    Success,
    /// 部分成功
    Partial,
    /// 失败
    Failed,
    /// 等待结果
    Pending,
}

/// 6 态 Task status (per DD §2.4)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    /// 待办
    Todo,
    /// 进行中
    InProgress,
    /// 评审中
    Review,
    /// 被阻塞
    Blocked,
    /// 完成
    Done,
    /// 不修复
    Wontfix,
}

/// Test status (per DD §7.6)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TestStatus {
    /// Passed
    Passed,
    /// Failed
    Failed,
    /// Running
    Running,
    /// Skipped
    Skipped,
}

/// UI 简化的 Test state (per DD §2.1) — 4 档 (passed / failed / running / none)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TestState {
    /// Passed
    Passed,
    /// Failed
    Failed,
    /// Running
    Running,
    /// No tests
    None,
}

/// PullRequest state (per DD §7.3)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PRState {
    /// Open
    Open,
    /// Merged
    Merged,
    /// Closed (without merge)
    Closed,
}

/// Issue state (per DD §7.7)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueState {
    /// Open
    Open,
    /// Closed
    Closed,
}

/// Symbol kind (per DD §7.5) — 7 类 (Function / Class / Variable / Type / Enum / Trait / Interface)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SymbolKind {
    /// Function
    Function,
    /// Class
    Class,
    /// Variable
    Variable,
    /// Type
    Type,
    /// Enum
    Enum,
    /// Trait
    Trait,
    /// Interface
    Interface,
}

/// Token usage (per DD §2.3)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Input tokens
    pub input: u64,
    /// Output tokens
    pub output: u64,
    /// Total tokens
    pub total: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn human_state_seven_variants() {
        assert_eq!(
            [
                HumanState::Running,
                HumanState::Waiting,
                HumanState::Ready,
                HumanState::Diverged,
                HumanState::Conflict,
                HumanState::Merged,
                HumanState::Stale,
            ]
            .len(),
            7
        );
    }

    #[test]
    fn merge_strategy_four_variants() {
        assert_eq!(
            [
                MergeStrategy::Merge,
                MergeStrategy::Squash,
                MergeStrategy::Rebase,
                MergeStrategy::FastForward,
            ]
            .len(),
            4
        );
    }

    #[test]
    fn symbol_kind_seven_variants() {
        assert_eq!(
            [
                SymbolKind::Function,
                SymbolKind::Class,
                SymbolKind::Variable,
                SymbolKind::Type,
                SymbolKind::Enum,
                SymbolKind::Trait,
                SymbolKind::Interface,
            ]
            .len(),
            7
        );
    }

    #[test]
    fn test_state_serializes_to_snake_case() {
        let json = serde_json::to_string(&TestState::Running).unwrap();
        assert_eq!(json, "\"running\"");
    }
}
