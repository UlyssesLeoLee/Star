//! Star Local Runtime — CLI Session 状态机与持久化实体 (ULYS-156)
//!
//! 实现 [ULYS-156](https://app.multica.ai/issue/01a0bf6b-486c-71b8-9bf4-92bde909c7f8)
//! §A 单进程持久化层适配 路线下的 "session 状态机 + 实体":
//!
//! - `CliSession` = 跨 runtime 升级 / 重启可恢复的 agent 会话单元(per FR-ORCA-001 AC-1)
//! - `CliSessionState` = 7 态状态机:Created → Running → Paused / Restart / Orphaned → Terminated → Archived
//! - `CliSessionTransition` = 状态机迁移表 + 合法动作校验
//!
//! **不动 process.rs** —— 那里 `ProcessHandle` + `ProcessState` 是一次性进程句柄;
//! 一个 `CliSession` 可以跨多次 `ProcessHandle` (parent runtime 重启 → agent CLI 子进程残留 →
//! 重新 attach 到新 runtime), 这是不同抽象层级。
//!
//! **不在本模块**:
//! - process_supervisor / graceful_shutdown / health_self_test —— 等 spec ACs 落地
//!   (上游 `docs/ecosystem-survey/orca-design-survey.md` 在本 worktree 不可达, D-Boy 9/22
//!   §A 锁定本路线但具体 AC 切片由后续轮次补)
//! - 真实 OS 平台适配 (Linux/macOS `setsid(2)` + Windows Job Object) —— AC-1 子进程存活
//!   已在 PR #63 (`efa501a3`) 落地, AC-3 单进程模型下不满足是 D-Boy 已知 trade-off
//! - star-eventbus 通道 —— 状态写入优先保证本地持久化原子, bus 是后续切片

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{CliSessionId, TenantId, WorktreeId};

// =====================================================================
// 1. value_object
// =====================================================================

/// **CLI Session 7 状态机**(per ULYS-156 §A 单进程持久化层适配 锁定路径)
///
/// 迁移图(Mermaid):
///
/// ```text
/// Created
///   ├─ start() ─→ Running
///   └─ cancel() ─→ Terminated
/// Running
///   ├─ pause() ─→ Paused
///   ├─ cancel() ─→ Terminated
///   └─ (parent 失联 / restart 超时) ─→ Orphaned
/// Paused
///   ├─ resume() ─→ Running
///   └─ cancel() ─→ Terminated
/// Restart
///   └─ (重启完成) ─→ Running
/// Orphaned
///   ├─ adopt() ─→ Running
///   └─ cancel() ─→ Terminated
/// Terminated
///   └─ archive() ─→ Archived
/// Archived
///   └─ (终态)
/// ```
///
/// **INV-CLI-SESSION-01**: Archived 是唯一终态, 其余状态可达 Running(终态不可达 Running)。
/// **INV-CLI-SESSION-02**: 显式迁移经 `try_transition()` 校验, 不允许跨级跳变。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CliSessionState {
    /// 已创建, 尚未 start
    Created,
    /// 运行中(关联 ProcessHandle 活跃)
    Running,
    /// 已暂停(用户手动 pause, ProcessHandle 仍在)
    Paused,
    /// 重启中(parent runtime 重启, 子进程残留待 attach)
    Restart,
    /// 失联(parent runtime 重启超时未完成, 子进程孤立)
    Orphaned,
    /// 已终止(显式 cancel / 进程退出)
    Terminated,
    /// 已归档(终态, 仅供审计 / 历史回溯)
    Archived,
}

impl CliSessionState {
    /// 转为 snake_case 字符串(与 serde rename_all 对齐, 用于 SQL DDL 列存值)
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Running => "running",
            Self::Paused => "paused",
            Self::Restart => "restart",
            Self::Orphaned => "orphaned",
            Self::Terminated => "terminated",
            Self::Archived => "archived",
        }
    }

    /// 是否终态(仅 `Archived`)
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Archived)
    }
}

// =====================================================================
// 2. entity
// =====================================================================

/// **CLI Session 实体**(per ULYS-156 §A)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CliSession {
    /// 唯一 ID
    pub id: CliSessionId,
    /// 跨租户(per INV-RT-01)
    pub tenant_id: TenantId,
    /// 关联 worktree(INV-RT-02 派生)
    pub worktree_id: WorktreeId,
    /// 当前状态
    pub state: CliSessionState,
    /// 执行命令(如 "claude" / "codex")
    pub command: String,
    /// 命令参数
    pub args: Vec<String>,
    /// 重启时累积的 stdout 字节数(用于 AC-3 单进程模式下的部分 scrollback 恢复;
    /// 单进程模型下 parent 关闭期间字节为 0 是已知 trade-off, D-Boy 9/22 接受)
    pub scrollback_bytes: u64,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 最近状态变更时间
    pub updated_at: DateTime<Utc>,
    /// 状态变更历史(append-only, 用于审计 + UI 时间线)
    pub state_history: Vec<CliSessionTransition>,
    /// 元数据(host_name / os / arch, 与 LocalRuntime.metadata 对齐)
    pub metadata: std::collections::HashMap<String, String>,
}

/// **状态机迁移记录**(append-only)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CliSessionTransition {
    /// 旧状态(`None` 表示 Created 起点)
    pub from: Option<CliSessionState>,
    /// 新状态
    pub to: CliSessionState,
    /// 触发原因
    pub reason: String,
    /// 变更时间
    pub at: DateTime<Utc>,
}

impl CliSession {
    /// 构造一个新 CliSession(初始 state = Created)
    pub fn new(
        tenant_id: TenantId,
        worktree_id: WorktreeId,
        command: String,
        args: Vec<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: CliSessionId::new(),
            tenant_id,
            worktree_id,
            state: CliSessionState::Created,
            command,
            args,
            scrollback_bytes: 0,
            created_at: now,
            updated_at: now,
            state_history: vec![CliSessionTransition {
                from: None,
                to: CliSessionState::Created,
                reason: "session created".to_string(),
                at: now,
            }],
            metadata: std::collections::HashMap::new(),
        }
    }

    /// 尝试状态机迁移(校验合法性 + 写 updated_at + append transition)
    ///
    /// 返回 Err 时不修改 self —— 既保证原子, 也方便调用方决定是否需要回滚。
    pub fn try_transition(
        &mut self,
        to: CliSessionState,
        reason: impl Into<String>,
    ) -> Result<(), CliSessionTransitionError> {
        let from = self.state;
        // no-op 必须先判: 否则 is_valid_transition(from, from) 永远是 false
        // → 返回 IllegalTransition, 跟 NoOpTransition 在调用方语义不一致
        if from == to {
            return Err(CliSessionTransitionError::NoOpTransition { state: from });
        }
        if !is_valid_transition(from, to) {
            return Err(CliSessionTransitionError::IllegalTransition { from, to });
        }
        let now = Utc::now();
        self.state = to;
        self.updated_at = now;
        self.state_history.push(CliSessionTransition {
            from: Some(from),
            to,
            reason: reason.into(),
            at: now,
        });
        Ok(())
    }
}

/// **状态机迁移表** —— 单一事实源; 与 `CliSessionState` 文档迁移图对应
///
/// 合法迁移全集:
/// - Created → Running, Terminated
/// - Running → Paused, Terminated, Orphaned
/// - Paused → Running, Terminated
/// - Restart → Running
/// - Orphaned → Running, Terminated
/// - Terminated → Archived
/// - Archived → 无(终态)
pub fn is_valid_transition(from: CliSessionState, to: CliSessionState) -> bool {
    use CliSessionState::*;
    matches!(
        (from, to),
        (Created, Running)
            | (Created, Terminated)
            | (Running, Paused)
            | (Running, Terminated)
            | (Running, Orphaned)
            | (Paused, Running)
            | (Paused, Terminated)
            | (Restart, Running)
            | (Orphaned, Running)
            | (Orphaned, Terminated)
            | (Terminated, Archived)
    )
}

// =====================================================================
// 3. error
// =====================================================================

/// 状态机迁移错误
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum CliSessionTransitionError {
    /// 非法迁移(状态表不允许 from → to)
    #[error("illegal transition: {from:?} -> {to:?}")]
    IllegalTransition {
        /// 当前状态
        from: CliSessionState,
        /// 试图迁移到的状态
        to: CliSessionState,
    },
    /// 同状态迁移(no-op)
    #[error("no-op transition: already in {state:?}")]
    NoOpTransition {
        /// 当前状态
        state: CliSessionState,
    },
}

// =====================================================================
// 4. tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn make_session() -> CliSession {
        CliSession::new(
            Uuid::new_v4().into(),
            Uuid::new_v4().into(),
            "claude".to_string(),
            vec!["--flag".to_string()],
        )
    }

    #[test]
    fn state_as_str_roundtrip() {
        for s in [
            CliSessionState::Created,
            CliSessionState::Running,
            CliSessionState::Paused,
            CliSessionState::Restart,
            CliSessionState::Orphaned,
            CliSessionState::Terminated,
            CliSessionState::Archived,
        ] {
            let parsed: CliSessionState =
                serde_json::from_str(&format!("\"{}\"", s.as_str())).unwrap();
            assert_eq!(parsed, s);
        }
    }

    #[test]
    fn terminal_only_archived() {
        for s in [
            CliSessionState::Created,
            CliSessionState::Running,
            CliSessionState::Paused,
            CliSessionState::Restart,
            CliSessionState::Orphaned,
            CliSessionState::Terminated,
        ] {
            assert!(!s.is_terminal(), "{s:?} should not be terminal");
        }
        assert!(CliSessionState::Archived.is_terminal());
    }

    #[test]
    fn happy_path_created_running_terminated_archived() {
        let mut s = make_session();
        s.try_transition(CliSessionState::Running, "spawn").unwrap();
        s.try_transition(CliSessionState::Terminated, "user cancel")
            .unwrap();
        s.try_transition(CliSessionState::Archived, "audit archive")
            .unwrap();
        assert!(s.state.is_terminal());
        // history length: 1 (created) + 3 = 4
        assert_eq!(s.state_history.len(), 4);
    }

    #[test]
    fn pause_resume_roundtrip() {
        let mut s = make_session();
        s.try_transition(CliSessionState::Running, "spawn").unwrap();
        s.try_transition(CliSessionState::Paused, "user pause")
            .unwrap();
        s.try_transition(CliSessionState::Running, "user resume")
            .unwrap();
        assert_eq!(s.state, CliSessionState::Running);
    }

    #[test]
    fn orphaned_then_adopt() {
        let mut s = make_session();
        s.try_transition(CliSessionState::Running, "spawn").unwrap();
        s.try_transition(CliSessionState::Orphaned, "parent restart timeout")
            .unwrap();
        s.try_transition(CliSessionState::Running, "adopt on next startup")
            .unwrap();
        assert_eq!(s.state, CliSessionState::Running);
    }

    #[test]
    fn illegal_skip_state_rejected() {
        let mut s = make_session();
        // Created → Archived 不在迁移表里
        let err = s
            .try_transition(CliSessionState::Archived, "skip")
            .unwrap_err();
        assert!(matches!(
            err,
            CliSessionTransitionError::IllegalTransition { .. }
        ));
        // Created state must be unchanged
        assert_eq!(s.state, CliSessionState::Created);
        // History should still be the 1-entry created record
        assert_eq!(s.state_history.len(), 1);
    }

    #[test]
    fn noop_transition_rejected() {
        let mut s = make_session();
        let err = s
            .try_transition(CliSessionState::Created, "noop")
            .unwrap_err();
        assert!(matches!(
            err,
            CliSessionTransitionError::NoOpTransition { .. }
        ));
    }

    #[test]
    fn archived_is_terminal_no_further_transitions() {
        let mut s = make_session();
        s.try_transition(CliSessionState::Running, "spawn").unwrap();
        s.try_transition(CliSessionState::Terminated, "cancel")
            .unwrap();
        s.try_transition(CliSessionState::Archived, "archive")
            .unwrap();
        let err = s
            .try_transition(CliSessionState::Running, "wake up?")
            .unwrap_err();
        assert!(matches!(
            err,
            CliSessionTransitionError::IllegalTransition { .. }
        ));
    }

    #[test]
    fn transition_table_matches_doc() {
        // 校验迁移表:每一对合法迁移都已经显式列出
        let legal = [
            (CliSessionState::Created, CliSessionState::Running),
            (CliSessionState::Created, CliSessionState::Terminated),
            (CliSessionState::Running, CliSessionState::Paused),
            (CliSessionState::Running, CliSessionState::Terminated),
            (CliSessionState::Running, CliSessionState::Orphaned),
            (CliSessionState::Paused, CliSessionState::Running),
            (CliSessionState::Paused, CliSessionState::Terminated),
            (CliSessionState::Restart, CliSessionState::Running),
            (CliSessionState::Orphaned, CliSessionState::Running),
            (CliSessionState::Orphaned, CliSessionState::Terminated),
            (CliSessionState::Terminated, CliSessionState::Archived),
        ];
        for (from, to) in legal {
            assert!(
                is_valid_transition(from, to),
                "expected {from:?} -> {to:?} to be legal"
            );
        }
        // 完全非法样例
        assert!(!is_valid_transition(
            CliSessionState::Created,
            CliSessionState::Archived
        ));
        assert!(!is_valid_transition(
            CliSessionState::Archived,
            CliSessionState::Running
        ));
        assert!(!is_valid_transition(
            CliSessionState::Running,
            CliSessionState::Created
        ));
    }
}
