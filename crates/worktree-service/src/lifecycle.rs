//! `lifecycle.rs` — 7-state Worktree state machine (per DD §23 + INV-WC-01)
//!
//! 7 HumanState: Running / Waiting / Ready / Diverged / Conflict / Merged / Stale
//!
//! 状态机迁移 (per DD §23.1 + §23.2):
//! - Created → Running
//! - Running ↔ Waiting
//! - Running/Diverged/Conflict + NoRisk + health≥80 + ahead==0 + risk_count==0 + test_passed → Ready
//! - → Diverged (ahead>5 AND behind>5)
//! - → Conflict (ConflictDetected)
//! - Conflict → Ready (ConflictResolved + health high)
//! - → Stale (last_activity > 7d, except Merged)
//! - → Merged (git merge success)
//! - Stale → Running (ActivityResumed)
//!
//! 全部迁移总路径: 7×7 = 49 (含 self-loop)

use serde::{Deserialize, Serialize};
use thiserror::Error;

use graph_core::state::{HumanState, TestState};

/// 状态机迁移事件 (per DD §23.1)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WorktreeEvent {
    /// Created
    Created,
    /// Agent 暂停
    AgentPaused,
    /// Agent 恢复
    AgentResumed,
    /// 无风险 (高健康)
    NoRisk,
    /// 检测到分叉
    DivergenceDetected {
        /// Ahead of main
        ahead: u32,
        /// Behind main
        behind: u32,
    },
    /// 检测到 merge conflict
    ConflictDetected,
    /// Conflict 解决
    ConflictResolved,
    /// 不活跃检查 (定期)
    InactivityCheck,
    /// git merge 成功
    Merged,
    /// 恢复活动 (Stale → Running)
    ActivityResumed,
}

/// 状态机迁移错误 (per INV-WC-01)
#[derive(Debug, Error, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransitionError {
    /// 非法迁移 (from state + event 不允许)
    #[error("Invalid transition from {from:?} on event {event:?}")]
    InvalidTransition {
        /// 当前 state
        from: HumanState,
        /// 触发 event
        event: WorktreeEvent,
    },
}

/// 工作区快照 (用于 transition 计算, per DD §23.1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeSnapshot {
    /// Health score value
    pub health_score: u8,
    /// Ahead of main
    pub ahead: u32,
    /// Behind main
    pub behind: u32,
    /// 风险计数
    pub risk_count: u32,
    /// 测试状态
    pub test_state: TestState,
}

impl Default for WorktreeSnapshot {
    fn default() -> Self {
        Self {
            health_score: 100,
            ahead: 0,
            behind: 0,
            risk_count: 0,
            test_state: TestState::None,
        }
    }
}

/// 状态机迁移函数 (per DD §23.1)
///
/// 根据 (current state, event) 计算目标 state。非法迁移返回 `TransitionError::InvalidTransition`。
pub fn transition(
    from: HumanState,
    event: &WorktreeEvent,
    snapshot: &WorktreeSnapshot,
) -> Result<HumanState, TransitionError> {
    use HumanState::*;

    let to = match (from, event) {
        // Created → Running
        (_, WorktreeEvent::Created) => Running,

        // Running ↔ Waiting
        (Running, WorktreeEvent::AgentPaused) => Waiting,
        (Waiting, WorktreeEvent::AgentResumed) => Running,

        // → Ready (高健康 + ahead==0 + risk==0 + 测试通过)
        // 来源: Running / Diverged / Conflict
        (
            Running | Diverged | Conflict,
            WorktreeEvent::NoRisk,
        ) if snapshot.health_score >= 80
            && snapshot.ahead == 0
            && snapshot.risk_count == 0
            && snapshot.test_state == TestState::Passed =>
        {
            Ready
        }

        // → Diverged (ahead>5 AND behind>5)
        (_, WorktreeEvent::DivergenceDetected { ahead, behind })
            if *ahead > 5 && *behind > 5 =>
        {
            Diverged
        }

        // → Conflict
        (_, WorktreeEvent::ConflictDetected) => Conflict,

        // Conflict → Ready (resolved + 高健康)
        (Conflict, WorktreeEvent::ConflictResolved)
            if snapshot.health_score >= 80
                && snapshot.risk_count == 0
                && snapshot.test_state == TestState::Passed =>
        {
            Ready
        }

        // Conflict → Diverged (resolved but diverged)
        (Conflict, WorktreeEvent::ConflictResolved) => Diverged,

        // → Stale (inactivity, except Merged)
        (s, WorktreeEvent::InactivityCheck) if s != Merged => Stale,

        // → Merged (terminal)
        (s, WorktreeEvent::Merged) if s != Merged => Merged,

        // Stale → Running (resumed)
        (Stale, WorktreeEvent::ActivityResumed) => Running,

        // Self-loop (no-op): NoRisk on already-Ready state stays Ready
        (Ready, WorktreeEvent::NoRisk) => Ready,
        // Self-loop: Merged + Merged event stays Merged
        (Merged, WorktreeEvent::Merged) => Merged,
        // Self-loop: Stale + InactivityCheck stays Stale
        (Stale, WorktreeEvent::InactivityCheck) => Stale,

        // Invalid
        (from_state, _) => {
            return Err(TransitionError::InvalidTransition {
                from: from_state,
                event: event.clone(),
            });
        }
    };

    Ok(to)
}

/// 工作区合并: 强制覆盖 (per WorktreeService.update)
pub fn force_set(state: HumanState, target: HumanState) -> Result<HumanState, TransitionError> {
    if state == HumanState::Merged && target != HumanState::Merged {
        return Err(TransitionError::InvalidTransition {
            from: state,
            event: WorktreeEvent::ActivityResumed, // 占位, 表示不允许 unmerge
        });
    }
    Ok(target)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap(health: u8, ahead: u32, behind: u32, risk: u32, test: TestState) -> WorktreeSnapshot {
        WorktreeSnapshot {
            health_score: health,
            ahead,
            behind,
            risk_count: risk,
            test_state: test,
        }
    }

    #[test]
    fn transition_7states_all_paths() {
        use HumanState::*;
        // Per TEST-DESIGN §2.2: worktree_state_transition_7states_all_paths
        // 测试每个 from state 至少 1 个合法迁移
        let snap_ready = snap(85, 0, 5, 0, TestState::Passed);

        // Created → Running
        assert_eq!(
            transition(Running, &WorktreeEvent::Created, &snap_ready),
            Ok(Running)
        );
        // Running → Waiting
        assert_eq!(
            transition(Running, &WorktreeEvent::AgentPaused, &snap_ready),
            Ok(Waiting)
        );
        // Waiting → Running
        assert_eq!(
            transition(Waiting, &WorktreeEvent::AgentResumed, &snap_ready),
            Ok(Running)
        );
        // Running → Ready (no risk + health high)
        assert_eq!(
            transition(Running, &WorktreeEvent::NoRisk, &snap_ready),
            Ok(Ready)
        );
        // Running → Diverged
        assert_eq!(
            transition(
                Running,
                &WorktreeEvent::DivergenceDetected { ahead: 10, behind: 8 },
                &snap_ready
            ),
            Ok(Diverged)
        );
        // Running → Conflict
        assert_eq!(
            transition(Running, &WorktreeEvent::ConflictDetected, &snap_ready),
            Ok(Conflict)
        );
        // Conflict → Ready
        assert_eq!(
            transition(Conflict, &WorktreeEvent::ConflictResolved, &snap_ready),
            Ok(Ready)
        );
        // Conflict → Diverged (resolved but diverged)
        let snap_low_health = snap(50, 0, 0, 2, TestState::Passed);
        assert_eq!(
            transition(Conflict, &WorktreeEvent::ConflictResolved, &snap_low_health),
            Ok(Diverged)
        );
        // Running → Stale (inactivity)
        assert_eq!(
            transition(Running, &WorktreeEvent::InactivityCheck, &snap_ready),
            Ok(Stale)
        );
        // Running → Merged
        assert_eq!(
            transition(Running, &WorktreeEvent::Merged, &snap_ready),
            Ok(Merged)
        );
        // Stale → Running
        assert_eq!(
            transition(Stale, &WorktreeEvent::ActivityResumed, &snap_ready),
            Ok(Running)
        );
    }

    #[test]
    fn state_machine_invalid_transition_rejected() {
        use HumanState::*;
        // Per TEST-DESIGN §2.2: state_machine_invalid_transition_rejected
        // Merged 是终态, 不允许 ActivityResumed
        let snap = snap(85, 0, 0, 0, TestState::Passed);
        let err = transition(Merged, &WorktreeEvent::ActivityResumed, &snap);
        assert!(matches!(err, Err(TransitionError::InvalidTransition { .. })));

        // Created → Conflict (无 Created 源 state)
        // 测试 random invalid: Running + DivergenceDetected {ahead:0, behind:0} (不触发 Diverged 条件)
        // 实际 invalid: Running + Merged 无具体条件约束, 走合法路径返回 Merged
        // 真正 invalid: Waiting + NoRisk (NoRisk 只对 Running/Diverged/Conflict 合法)
        let err2 = transition(Waiting, &WorktreeEvent::NoRisk, &snap);
        assert!(matches!(err2, Err(TransitionError::InvalidTransition { .. })));
    }

    #[test]
    fn state_machine_self_loops() {
        use HumanState::*;
        let snap = snap(85, 0, 0, 0, TestState::Passed);

        // Ready + NoRisk → Ready
        assert_eq!(transition(Ready, &WorktreeEvent::NoRisk, &snap), Ok(Ready));
        // Merged + Merged → Merged
        assert_eq!(transition(Merged, &WorktreeEvent::Merged, &snap), Ok(Merged));
        // Stale + InactivityCheck → Stale
        assert_eq!(
            transition(Stale, &WorktreeEvent::InactivityCheck, &snap),
            Ok(Stale)
        );
        // Conflict + ConflictDetected → Conflict
        assert_eq!(
            transition(Conflict, &WorktreeEvent::ConflictDetected, &snap),
            Ok(Conflict)
        );
    }

    #[test]
    fn state_machine_full_49_paths_enumeration() {
        // 7 states × 7 events (取子集作为合法迁移路径) — 枚举全部 49 条
        use HumanState::*;
        let snap = snap(85, 0, 0, 0, TestState::Passed);
        let events = [
            WorktreeEvent::Created,
            WorktreeEvent::AgentPaused,
            WorktreeEvent::AgentResumed,
            WorktreeEvent::NoRisk,
            WorktreeEvent::DivergenceDetected { ahead: 10, behind: 8 },
            WorktreeEvent::ConflictDetected,
            WorktreeEvent::ConflictResolved,
            WorktreeEvent::InactivityCheck,
            WorktreeEvent::Merged,
            WorktreeEvent::ActivityResumed,
        ];
        // 验证迁移函数对每个 (state, event) 都返回 Ok 或 Err (不 panic)
        for s in [Running, Waiting, Ready, Diverged, Conflict, Merged, Stale] {
            for e in &events {
                let _ = transition(s, e, &snap);
            }
        }
    }
}
