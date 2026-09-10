//! A3.1 + A3.3 status sync + failed notification (per DD-CANVAS-AGENT-001 §3.1 + §5 5 状态机 + 守门 #13 a RLS + 守门 #19 v19 累积规).
//!
//! `StatusSync` 跟踪 1 个 agent 的 14 状态机 实时色卡同步 (A3.1 ≤ 200ms P95) +
//! failed 状态 notification 触发 (A3.3 ≤ 500ms 派生).
//! 主要业务方法: 状态色码 (CSS class) / failed notification 消息生成, 0 外部 I/O.

use crate::models::agent::AgentState;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// `agent-domain` status_sync 错误 (per DD-AGENT §4.7 + 守门 #13 a 100% RLS 13 类 tenant_id 必填).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StatusSyncError {
    /// tenant_id 未设置 (守门 #13 a RLS 13 类)
    TenantIdRequired,
}

/// A3.1 14 状态机 实时色卡同步 (per DD-CANVAS-AGENT-001 §5.2 + §3.1 + BD-CANVAS-AGENT-001 §3 A3.1).
///
/// 字段: agent_id + current_state (14 状态机当前态) + last_sync_at (≤ 200ms P95 频率).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusSync {
    /// agent ID
    pub agent_id: Uuid,
    /// 14 状态机当前态
    pub current_state: AgentState,
    /// 上次同步时间 (UTC, ≤ 200ms P95 per A3.1)
    pub last_sync_at: chrono::DateTime<chrono::Utc>,
    /// tenant_id (守门 #13 a 100% RLS 13 类 必携)
    pub tenant_id: Uuid,
}

impl StatusSync {
    /// A3.1 业务方法: 创建 status sync.
    ///
    /// 守门 #13 a: tenant_id 必填 (非 nil Uuid).
    pub fn new(
        agent_id: Uuid,
        current_state: AgentState,
        tenant_id: Uuid,
    ) -> Result<Self, StatusSyncError> {
        if tenant_id.is_nil() {
            return Err(StatusSyncError::TenantIdRequired);
        }
        Ok(Self {
            agent_id,
            current_state,
            last_sync_at: chrono::Utc::now(),
            tenant_id,
        })
    }

    /// A3.1 业务方法: 14 状态机 → CSS 色码 class (per frontend §4.6 StatusPill 60+ 派生).
    ///
    /// V0.4 业务方法: 返回 CSS class name 字符串, frontend 渲染色卡 ≤ 200ms P95.
    pub fn state_color_class(&self) -> &'static str {
        match self.current_state {
            AgentState::Initializing => "status-initializing",
            AgentState::Spawning => "status-spawning",
            AgentState::Running => "status-running",
            AgentState::Paused => "status-paused",
            AgentState::Stopping => "status-stopping",
            AgentState::Stopped => "status-stopped",
            AgentState::Completed => "status-completed",
            AgentState::Failed => "status-failed",
            AgentState::Archived => "status-archived",
            AgentState::SpawningSubagent => "status-spawning-subagent",
            AgentState::HandoffInProgress => "status-handoff-in-progress",
            AgentState::TrustScoreUpdated => "status-trust-score-updated",
            AgentState::ChallengeInProgress => "status-challenge-in-progress",
            AgentState::StandInActive => "status-stand-in-active",
        }
    }

    /// A3.3 业务方法: failed 状态 notification 触发 (≤ 500ms 派生).
    ///
    /// V0.4 业务方法: 返回 Some(message) 如果状态是 Failed (终态, 需通知 + 边框 + 闪烁 per A3.3),
    /// 否则 None.
    pub fn failed_notification(&self) -> Option<String> {
        if self.current_state == AgentState::Failed {
            Some(format!(
                "Agent {} failed at {} (tenant {})",
                self.agent_id, self.last_sync_at, self.tenant_id
            ))
        } else {
            None
        }
    }

    /// A3.1 业务方法: 更新当前状态 + last_sync_at (≤ 200ms P95).
    pub fn update_state(&mut self, new_state: AgentState) {
        self.current_state = new_state;
        self.last_sync_at = chrono::Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_sync_new_ok() {
        let agent = Uuid::new_v4();
        let tenant = Uuid::new_v4();
        let s = StatusSync::new(agent, AgentState::Running, tenant).expect("valid");
        assert_eq!(s.agent_id, agent);
        assert_eq!(s.current_state, AgentState::Running);
        assert_eq!(s.tenant_id, tenant);
    }

    #[test]
    fn test_status_sync_new_tenant_required() {
        let agent = Uuid::new_v4();
        let result = StatusSync::new(agent, AgentState::Running, Uuid::nil());
        assert_eq!(result.err(), Some(StatusSyncError::TenantIdRequired));
    }

    #[test]
    fn test_status_sync_state_color_class_running() {
        let tenant = Uuid::new_v4();
        let s = StatusSync::new(Uuid::new_v4(), AgentState::Running, tenant).unwrap();
        assert_eq!(s.state_color_class(), "status-running");
    }

    #[test]
    fn test_status_sync_state_color_class_failed() {
        let tenant = Uuid::new_v4();
        let s = StatusSync::new(Uuid::new_v4(), AgentState::Failed, tenant).unwrap();
        assert_eq!(s.state_color_class(), "status-failed");
    }

    #[test]
    fn test_status_sync_failed_notification_failed() {
        let agent = Uuid::new_v4();
        let tenant = Uuid::new_v4();
        let s = StatusSync::new(agent, AgentState::Failed, tenant).unwrap();
        let msg = s.failed_notification().expect("Some on Failed");
        assert!(msg.contains("failed"));
        assert!(msg.contains(&agent.to_string()));
    }

    #[test]
    fn test_status_sync_failed_notification_running() {
        let tenant = Uuid::new_v4();
        let s = StatusSync::new(Uuid::new_v4(), AgentState::Running, tenant).unwrap();
        assert_eq!(s.failed_notification(), None);
    }

    #[test]
    fn test_status_sync_update_state() {
        let tenant = Uuid::new_v4();
        let mut s = StatusSync::new(Uuid::new_v4(), AgentState::Initializing, tenant).unwrap();
        s.update_state(AgentState::Running);
        assert_eq!(s.current_state, AgentState::Running);
    }

    #[test]
    fn test_status_sync_all_14_states_color_class() {
        let tenant = Uuid::new_v4();
        let states = [
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
        for st in states {
            let s = StatusSync::new(Uuid::new_v4(), st, tenant).unwrap();
            assert!(s.state_color_class().starts_with("status-"));
        }
    }
}
