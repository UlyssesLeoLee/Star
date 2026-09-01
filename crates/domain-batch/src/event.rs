//! 11 Domain Event (per BATCH-REQ-001 §3.5 + ADR-0040 §D36 + spec §5)
//!
//! 11 事件名 (NATS subject):
//! - `star.events.batch.task.created.v1`
//! - `star.events.batch.task.updated.v1`
//! - `star.events.batch.run.triggered.v1`
//! - `star.events.batch.run.started.v1`
//! - `star.events.batch.node.started.v1`
//! - `star.events.batch.node.succeeded.v1`
//! - `star.events.batch.node.failed.v1`
//! - `star.events.batch.run.completed.v1`
//! - `star.events.batch.run.cancelled.v1`
//! - `star.events.batch.alert.fired.v1`
//! - `star.events.batch.sla.breached.v1`

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::Event;
use crate::{EventId, RunId, TaskId, TenantId};

/// 事件 meta (NATS 消息头)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMeta {
    pub event_id: Uuid,
    pub tenant_id: TenantId,
    pub occurred_at: DateTime<Utc>,
}

impl EventMeta {
    pub fn new(tenant_id: TenantId) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            tenant_id,
            occurred_at: Utc::now(),
        }
    }
}

/// 11 事件类型 enum (per BATCH-REQ-001 §3.5 F-040~045 + spec §5)
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BatchEventKind {
    /// `task_created` (Task 创建)
    TaskCreated,
    /// `task_updated` (Task 修改)
    TaskUpdated,
    /// `run_triggered` (Run 触发)
    RunTriggered,
    /// `run_started` (Run 开始执行)
    RunStarted,
    /// `node_started` (节点开始)
    NodeStarted,
    /// `node_succeeded` (节点成功)
    NodeSucceeded,
    /// `node_failed` (节点失败)
    NodeFailed,
    /// `run_completed` (Run 完成 success/failed/partial)
    RunCompleted,
    /// `run_cancelled` (Run 取消)
    RunCancelled,
    /// `alert_fired` (告警触发)
    AlertFired,
    /// `sla_breached` (SLA 违反)
    SlaBreached,
}

impl BatchEventKind {
    /// NATS subject (per spec §5)
    pub fn subject(&self) -> &'static str {
        match self {
            Self::TaskCreated => "star.events.batch.task.created.v1",
            Self::TaskUpdated => "star.events.batch.task.updated.v1",
            Self::RunTriggered => "star.events.batch.run.triggered.v1",
            Self::RunStarted => "star.events.batch.run.started.v1",
            Self::NodeStarted => "star.events.batch.node.started.v1",
            Self::NodeSucceeded => "star.events.batch.node.succeeded.v1",
            Self::NodeFailed => "star.events.batch.node.failed.v1",
            Self::RunCompleted => "star.events.batch.run.completed.v1",
            Self::RunCancelled => "star.events.batch.run.cancelled.v1",
            Self::AlertFired => "star.events.batch.alert.fired.v1",
            Self::SlaBreached => "star.events.batch.sla.breached.v1",
        }
    }

    /// 事件名字符串 (per `batch_event.event_kind` 字段, 对齐 DB schema)
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::TaskCreated => "task_created",
            Self::TaskUpdated => "task_updated",
            Self::RunTriggered => "run_triggered",
            Self::RunStarted => "run_started",
            Self::NodeStarted => "node_started",
            Self::NodeSucceeded => "node_succeeded",
            Self::NodeFailed => "node_failed",
            Self::RunCompleted => "run_completed",
            Self::RunCancelled => "run_cancelled",
            Self::AlertFired => "alert_fired",
            Self::SlaBreached => "sla_breached",
        }
    }

    /// 从字符串解析 (反序列化, 用于 `batch_event.event_kind` DB 字段)
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "task_created" => Self::TaskCreated,
            "task_updated" => Self::TaskUpdated,
            "run_triggered" => Self::RunTriggered,
            "run_started" => Self::RunStarted,
            "node_started" => Self::NodeStarted,
            "node_succeeded" => Self::NodeSucceeded,
            "node_failed" => Self::NodeFailed,
            "run_completed" => Self::RunCompleted,
            "run_cancelled" => Self::RunCancelled,
            "alert_fired" => Self::AlertFired,
            "sla_breached" => Self::SlaBreached,
            _ => return None,
        })
    }
}

/// Batch 域事件 (11 事件 + meta)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchEvent {
    pub meta: EventMeta,
    pub kind: BatchEventKind,
    pub run_id: Option<RunId>,
    pub task_id: Option<TaskId>,
    pub payload: serde_json::Value,
}

impl BatchEvent {
    /// NATS subject (分发到对应 subject)
    pub fn subject(&self) -> &'static str {
        self.kind.subject()
    }

    /// 转换为 `batch_event` DB 实体 (per `Event` domain entity, per ADR-0040 §D36)
    pub fn into_db_event(self) -> Event {
        Event {
            id: EventId::from_uuid(self.meta.event_id),
            run_id: self.run_id,
            task_id: self.task_id,
            kind: self.kind,
            payload: self.payload,
            actor: "system".to_string(), // 后续可注入 actor
            ts: self.meta.occurred_at,
            causation_id: None,
            correlation_id: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_kind_subject_roundtrip() {
        let kinds = [
            BatchEventKind::TaskCreated,
            BatchEventKind::TaskUpdated,
            BatchEventKind::RunTriggered,
            BatchEventKind::RunStarted,
            BatchEventKind::NodeStarted,
            BatchEventKind::NodeSucceeded,
            BatchEventKind::NodeFailed,
            BatchEventKind::RunCompleted,
            BatchEventKind::RunCancelled,
            BatchEventKind::AlertFired,
            BatchEventKind::SlaBreached,
        ];
        assert_eq!(kinds.len(), 11);
        for kind in &kinds {
            let s = kind.as_str();
            let back = BatchEventKind::parse(s).unwrap();
            assert_eq!(*kind, back);
            assert!(kind.subject().starts_with("star.events.batch."));
        }
    }
}
