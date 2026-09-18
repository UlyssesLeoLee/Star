//! `consumer.rs` — 4 消费者 (Graph / UI / Risk / Notification, per spec §4.5 + BD §13.2)
//!
//! 每 Consumer 关注一组事件类型:
//! - `GraphConsumer`: 全部 15 个 (图投影)
//! - `UiConsumer`: WorktreeCreated / WorktreeChanged / WorktreeDeleted / WorktreeMerged /
//!   BranchUpdated / MainUpdated / AgentStarted / AgentStopped /
//!   TaskChanged / RiskDetected / RiskResolved / HealthChanged /
//!   RelationCreated / RelationRemoved (UI 推送)
//! - `RiskConsumer`: WorktreeChanged / MainUpdated / TestCompleted (Risk 增量计算)
//! - `NotificationConsumer`: RiskDetected / HealthChanged (通知)
//!
//! 订阅关系 (per BD §13.1 消费者矩阵):
//!
//! | Event Type        | Graph | UI | Risk | Notification |
//! |-------------------|:-----:|:--:|:----:|:------------:|
//! | WorktreeCreated   |  Y    | Y  |      |              |
//! | WorktreeChanged   |  Y    | Y  |  Y   |              |
//! | WorktreeDeleted   |  Y    | Y  |      |              |
//! | WorktreeMerged    |  Y    | Y  |      |              |
//! | BranchUpdated     |  Y    | Y  |      |              |
//! | MainUpdated       |  Y    | Y  |  Y   |              |
//! | AgentStarted      |  Y    | Y  |      |              |
//! | AgentStopped      |  Y    | Y  |      |              |
//! | TaskChanged       |  Y    | Y  |      |              |
//! | TestCompleted     |  Y    |    |  Y   |              |
//! | RiskDetected      |  Y    | Y  |      |     Y        |
//! | RiskResolved      |  Y    | Y  |      |              |
//! | HealthChanged     |  Y    | Y  |      |     Y        |
//! | RelationCreated   |  Y    | Y  |      |              |
//! | RelationRemoved   |  Y    | Y  |      |              |

use serde::{Deserialize, Serialize};

use crate::events::EventKind;

/// 4 Consumer Group 名称 (per BD §13.2)
pub mod groups {
    /// Graph 投影消费者
    pub const GRAPH: &str = "graph";
    /// UI SSE 推送消费者
    pub const UI: &str = "ui";
    /// Risk 增量计算消费者
    pub const RISK: &str = "risk";
    /// Notification 通知消费者
    pub const NOTIFICATION: &str = "notification";
}

/// Consumer 类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Consumer {
    /// Graph 投影消费者 (全部 15 个)
    Graph,
    /// UI 推送消费者 (14 个, 除 TestCompleted)
    Ui,
    /// Risk 增量消费者 (3 个)
    Risk,
    /// Notification 通知消费者 (2 个)
    Notification,
}

impl Consumer {
    /// Consumer Group 名称
    pub fn group_name(&self) -> &'static str {
        match self {
            Self::Graph => groups::GRAPH,
            Self::Ui => groups::UI,
            Self::Risk => groups::RISK,
            Self::Notification => groups::NOTIFICATION,
        }
    }

    /// 关注的事件类型
    pub fn interested_kinds(&self) -> Vec<EventKind> {
        match self {
            Self::Graph => EventKind::ALL.to_vec(),
            Self::Ui => vec![
                EventKind::WorktreeCreated,
                EventKind::WorktreeChanged,
                EventKind::WorktreeDeleted,
                EventKind::WorktreeMerged,
                EventKind::BranchUpdated,
                EventKind::MainUpdated,
                EventKind::AgentStarted,
                EventKind::AgentStopped,
                EventKind::TaskChanged,
                EventKind::RiskDetected,
                EventKind::RiskResolved,
                EventKind::HealthChanged,
                EventKind::RelationCreated,
                EventKind::RelationRemoved,
            ],
            Self::Risk => vec![
                EventKind::WorktreeChanged,
                EventKind::MainUpdated,
                EventKind::TestCompleted,
            ],
            Self::Notification => vec![EventKind::RiskDetected, EventKind::HealthChanged],
        }
    }

    /// 4 Consumer 总数 (守门用)
    pub const COUNT: usize = 4;

    /// 全部 4 个 Consumer
    pub const ALL: [Consumer; Self::COUNT] =
        [Self::Graph, Self::Ui, Self::Risk, Self::Notification];
}

/// EventConsumer — 处理单条 event 的回调 (P2 落点实装)
pub trait EventConsumer: Send + Sync {
    /// 处理 event
    fn handle(&self, event_kind: EventKind, event_id: uuid::Uuid);
}

/// Consumer Group 配置 (per BD §13.2 Redis Streams consumer group)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumerGroup {
    /// Consumer 类型
    pub consumer: Consumer,
    /// Stream key
    pub stream: String,
    /// Consumer group 名称 (per `Consumer::group_name`)
    pub group: String,
}

impl ConsumerGroup {
    /// 构造
    pub fn new(consumer: Consumer, stream: impl Into<String>) -> Self {
        let stream = stream.into();
        let group = consumer.group_name().to_string();
        Self {
            consumer,
            stream,
            group,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_4_consumers_graph_ui_risk_notification() {
        // 守门 UT-2 (T9): 4 消费者完整定义 + 关注类型正确
        assert_eq!(Consumer::COUNT, 4);
        assert_eq!(Consumer::ALL.len(), 4);

        // 4 消费者 Group name 唯一
        let names: Vec<&str> = Consumer::ALL.iter().map(|c| c.group_name()).collect();
        assert_eq!(
            names.iter().collect::<std::collections::HashSet<_>>().len(),
            4
        );

        // Graph 关注全部 15 个
        assert_eq!(Consumer::Graph.interested_kinds().len(), 15);

        // Risk 关注 3 个
        let risk_kinds = Consumer::Risk.interested_kinds();
        assert_eq!(risk_kinds.len(), 3);
        assert!(risk_kinds.contains(&EventKind::WorktreeChanged));
        assert!(risk_kinds.contains(&EventKind::MainUpdated));
        assert!(risk_kinds.contains(&EventKind::TestCompleted));

        // UI 关注 14 个 (除 TestCompleted)
        let ui_kinds = Consumer::Ui.interested_kinds();
        assert_eq!(ui_kinds.len(), 14);
        assert!(!ui_kinds.contains(&EventKind::TestCompleted));

        // Notification 关注 2 个
        let notif_kinds = Consumer::Notification.interested_kinds();
        assert_eq!(notif_kinds.len(), 2);
        assert!(notif_kinds.contains(&EventKind::RiskDetected));
        assert!(notif_kinds.contains(&EventKind::HealthChanged));
    }
}
