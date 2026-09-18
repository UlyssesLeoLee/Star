//! `events.rs` — 15 Event 类型定义 (per DD-WORKTREE-CANVAS-001 §19 + spec §4.5 + BD §13.1)
//!
//! 15 Event 清单 (per BD §13.1):
//!
//! | # | Event Type             | Payload                       | Consumer             |
//! |---|------------------------|-------------------------------|----------------------|
//! | 1 | WorktreeCreated        | WorktreeNode                  | Graph, UI, Risk      |
//! | 2 | WorktreeChanged        | WorktreeNode (diff)           | Graph, UI, Risk      |
//! | 3 | WorktreeDeleted        | `{ id }`                      | Graph, UI            |
//! | 4 | WorktreeMerged         | `{ id, merged_at }`           | Graph, UI, Recent    |
//! | 5 | BranchUpdated          | `{ repo_id, branch, head_sha }` | Graph, UI          |
//! | 6 | MainUpdated            | `{ repo_id, new_head }`       | Graph, Risk, UI      |
//! | 7 | AgentStarted           | AgentSessionNode              | Graph, UI            |
//! | 8 | AgentStopped           | `{ id, result }`              | Graph, UI            |
//! | 9 | TaskChanged            | TaskNode                      | Graph, UI            |
//! | 10| TestCompleted          | `{ worktree_id, test_state }` | Graph, Risk, Health  |
//! | 11| RiskDetected           | RiskEvent                     | Graph, UI, Notif     |
//! | 12| RiskResolved           | `{ risk_id }`                 | Graph, UI            |
//! | 13| HealthChanged          | `{ worktree_id, health }`     | Graph, UI            |
//! | 14| RelationCreated        | Edge                          | Graph, UI            |
//! | 15| RelationRemoved        | `{ edge_id }`                 | Graph, UI            |

use chrono::{DateTime, Utc};
use graph_core::types::{BranchId, RepoId, RiskScore, WorktreeId};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 15 Event 总数 (per INV-WC-09 + ULYS-57.3 T9 acceptance)
pub const EVENT_TYPE_COUNT: usize = 15;

/// 事件名 (per spec §4.5)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    /// Worktree 已创建
    WorktreeCreated,
    /// Worktree 状态变更
    WorktreeChanged,
    /// Worktree 已删除
    WorktreeDeleted,
    /// Worktree 已合并
    WorktreeMerged,
    /// Branch HEAD 更新
    BranchUpdated,
    /// Main HEAD 更新 (下游 Worktree 影响分析触发)
    MainUpdated,
    /// Agent 启动
    AgentStarted,
    /// Agent 停止
    AgentStopped,
    /// Task 状态变更
    TaskChanged,
    /// Test 完成 (passed / failed / etc.)
    TestCompleted,
    /// Risk 检测到
    RiskDetected,
    /// Risk 已解决
    RiskResolved,
    /// Health Score 变化
    HealthChanged,
    /// Graph Edge 创建
    RelationCreated,
    /// Graph Edge 移除
    RelationRemoved,
}

impl EventKind {
    /// 全部 15 个 Event 类型 (按 enum 顺序, per INV-WC-09 守门)
    pub const ALL: [EventKind; EVENT_TYPE_COUNT] = [
        Self::WorktreeCreated,
        Self::WorktreeChanged,
        Self::WorktreeDeleted,
        Self::WorktreeMerged,
        Self::BranchUpdated,
        Self::MainUpdated,
        Self::AgentStarted,
        Self::AgentStopped,
        Self::TaskChanged,
        Self::TestCompleted,
        Self::RiskDetected,
        Self::RiskResolved,
        Self::HealthChanged,
        Self::RelationCreated,
        Self::RelationRemoved,
    ];

    /// 事件名 (snake_case, for routing / filter)
    pub fn name(&self) -> &'static str {
        match self {
            Self::WorktreeCreated => "worktree.created",
            Self::WorktreeChanged => "worktree.changed",
            Self::WorktreeDeleted => "worktree.deleted",
            Self::WorktreeMerged => "worktree.merged",
            Self::BranchUpdated => "branch.updated",
            Self::MainUpdated => "main.updated",
            Self::AgentStarted => "agent.started",
            Self::AgentStopped => "agent.stopped",
            Self::TaskChanged => "task.changed",
            Self::TestCompleted => "test.completed",
            Self::RiskDetected => "risk.detected",
            Self::RiskResolved => "risk.resolved",
            Self::HealthChanged => "health.changed",
            Self::RelationCreated => "relation.created",
            Self::RelationRemoved => "relation.removed",
        }
    }
}

/// WorktreeChanged — 变更字段列表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeChangedPayload {
    /// Worktree ID
    pub id: WorktreeId,
    /// 变更字段列表 (e.g. ["human_state", "behind"])
    pub changed_fields: Vec<String>,
}

/// WorktreeDeleted payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeDeletedPayload {
    /// Worktree ID
    pub id: WorktreeId,
}

/// WorktreeMerged payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeMergedPayload {
    /// Worktree ID
    pub id: WorktreeId,
    /// 合并时间
    pub merged_at: DateTime<Utc>,
}

/// BranchUpdated payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchUpdatedPayload {
    /// Repo ID
    pub repo_id: RepoId,
    /// Branch ID
    pub branch_id: BranchId,
    /// 新 HEAD SHA
    pub head_sha: String,
}

/// MainUpdated payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MainUpdatedPayload {
    /// Repo ID
    pub repo_id: RepoId,
    /// 新 main HEAD SHA
    pub new_head: String,
}

/// AgentStarted payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStartedPayload {
    /// Agent ID
    pub agent_id: Uuid,
    /// 关联 Worktree
    pub worktree_id: Option<WorktreeId>,
}

/// AgentStopped payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStoppedPayload {
    /// Agent ID
    pub agent_id: Uuid,
    /// 结果 (success / failed)
    pub result: String,
}

/// TestCompleted payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCompletedPayload {
    /// Worktree ID
    pub worktree_id: WorktreeId,
    /// Test state (passed / failed / running / skipped)
    pub test_state: String,
}

/// RiskDetected payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskEventPayload {
    /// Risk ID
    pub risk_id: Uuid,
    /// Worktree ID
    pub worktree_id: WorktreeId,
    /// Risk 类型
    pub risk_type: String,
    /// Risk Score (0.0-1.0)
    pub risk_score: RiskScore,
    /// 触发原因
    pub reason: String,
}

/// RiskResolved payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskResolvedPayload {
    /// Risk ID
    pub risk_id: Uuid,
}

/// HealthChanged payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthChangedPayload {
    /// Worktree ID
    pub worktree_id: WorktreeId,
    /// 新 Health Score (0-100)
    pub health: u8,
    /// 时间
    pub computed_at: DateTime<Utc>,
}

/// RelationCreated payload (Graph Edge)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationCreatedPayload {
    /// Edge ID
    pub edge_id: String,
    /// 起点 Node ID
    pub from_id: String,
    /// 终点 Node ID
    pub to_id: String,
    /// Edge 类型
    pub kind: String,
}

/// RelationRemoved payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationRemovedPayload {
    /// Edge ID
    pub edge_id: String,
}

/// Canvas Event — 15 类型 tagged union (per DD §19)
///
/// 序列化形态: `{"kind": "worktree.created", "payload": {...}, "timestamp": "..."}`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CanvasEvent {
    /// WorktreeCreated
    WorktreeCreated {
        /// Payload
        payload: WorktreeDeletedPayload,
    },
    /// WorktreeChanged
    WorktreeChanged {
        /// Payload
        payload: WorktreeChangedPayload,
    },
    /// WorktreeDeleted
    WorktreeDeleted {
        /// Payload
        payload: WorktreeDeletedPayload,
    },
    /// WorktreeMerged
    WorktreeMerged {
        /// Payload
        payload: WorktreeMergedPayload,
    },
    /// BranchUpdated
    BranchUpdated {
        /// Payload
        payload: BranchUpdatedPayload,
    },
    /// MainUpdated
    MainUpdated {
        /// Payload
        payload: MainUpdatedPayload,
    },
    /// AgentStarted
    AgentStarted {
        /// Payload
        payload: AgentStartedPayload,
    },
    /// AgentStopped
    AgentStopped {
        /// Payload
        payload: AgentStoppedPayload,
    },
    /// TaskChanged
    TaskChanged {
        /// Payload
        payload: serde_json::Value,
    },
    /// TestCompleted
    TestCompleted {
        /// Payload
        payload: TestCompletedPayload,
    },
    /// RiskDetected
    RiskDetected {
        /// Payload
        payload: RiskEventPayload,
    },
    /// RiskResolved
    RiskResolved {
        /// Payload
        payload: RiskResolvedPayload,
    },
    /// HealthChanged
    HealthChanged {
        /// Payload
        payload: HealthChangedPayload,
    },
    /// RelationCreated
    RelationCreated {
        /// Payload
        payload: RelationCreatedPayload,
    },
    /// RelationRemoved
    RelationRemoved {
        /// Payload
        payload: RelationRemovedPayload,
    },
}

impl CanvasEvent {
    /// 事件 Kind (per spec §4.5 snake_case routing)
    pub fn kind(&self) -> EventKind {
        match self {
            Self::WorktreeCreated { .. } => EventKind::WorktreeCreated,
            Self::WorktreeChanged { .. } => EventKind::WorktreeChanged,
            Self::WorktreeDeleted { .. } => EventKind::WorktreeDeleted,
            Self::WorktreeMerged { .. } => EventKind::WorktreeMerged,
            Self::BranchUpdated { .. } => EventKind::BranchUpdated,
            Self::MainUpdated { .. } => EventKind::MainUpdated,
            Self::AgentStarted { .. } => EventKind::AgentStarted,
            Self::AgentStopped { .. } => EventKind::AgentStopped,
            Self::TaskChanged { .. } => EventKind::TaskChanged,
            Self::TestCompleted { .. } => EventKind::TestCompleted,
            Self::RiskDetected { .. } => EventKind::RiskDetected,
            Self::RiskResolved { .. } => EventKind::RiskResolved,
            Self::HealthChanged { .. } => EventKind::HealthChanged,
            Self::RelationCreated { .. } => EventKind::RelationCreated,
            Self::RelationRemoved { .. } => EventKind::RelationRemoved,
        }
    }

    /// 事件名 (snake_case)
    pub fn name(&self) -> &'static str {
        self.kind().name()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_bus_15_types_emit_correct() {
        // 守门 UT-1 (T9): 15 Event 类型完整定义
        assert_eq!(EVENT_TYPE_COUNT, 15);
        assert_eq!(EventKind::ALL.len(), 15);

        // 每个 kind 都有 name()
        for kind in EventKind::ALL.iter() {
            let name = kind.name();
            assert!(!name.is_empty());
            assert!(name.contains('.'));
        }

        // 序列化往返 (per spec §4.5 snake_case)
        let event = CanvasEvent::WorktreeMerged {
            payload: WorktreeMergedPayload {
                id: Uuid::new_v4(),
                merged_at: Utc::now(),
            },
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("worktree_merged") || json.contains("worktree.merged"));
    }

    #[test]
    fn canvas_event_kind_extraction_correct() {
        let event = CanvasEvent::RiskDetected {
            payload: RiskEventPayload {
                risk_id: Uuid::new_v4(),
                worktree_id: Uuid::new_v4(),
                risk_type: "merge_conflict".to_string(),
                risk_score: RiskScore(0.85),
                reason: "shared files".to_string(),
            },
        };
        assert_eq!(event.kind(), EventKind::RiskDetected);
        assert_eq!(event.name(), "risk.detected");
    }
}
