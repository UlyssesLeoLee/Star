//! domain-batch crate (第 23 个 domain crate, per BATCH-REQ-001 v0.1.2 + ADR-0040 commit aeaf213)
//!
//! 详细 spec: docs/specs/domain-batch-spec.md
//! 上游基本设计: docs/basic-design.md §5 (待 v0.16 增补 §5.16 batch)
//! 上游 ADR: docs/architecture/2026-08-26-upgrade/adr/0040-domain-batch.md
//! 上游需求: docs/requirements/batch-001.md
//! 数据设计: docs/data-design.md §4 (待 v0.16 增补 §4.16 batch 8 schema W/T/M)
//! API 设计: docs/api-design.md §3 (待 v0.16 增补 §3.16 batch 6 MCP tool)
//!
//! ## 职责
//!
//! 批处理任务调度 + DAG 编排 + 状态机 + 节点执行 (per ADR-0040 §D33-D39)。
//! MVP 不强制 DAG 拖拽编辑 (per 9/1 18:43 JST Ulysses 拍板 C, v0 phase 1 仅 JSON/YAML 导入 + 只读画布)。
//! 5 节点类型: domain-service / mcp-tool / http / shell / sql (per ADR-0040 §D35)。
//! 8 实体 3 分类 W/T/M: Master 4 + Work 3 + Transaction 1 (per ADR-0040 §D36)。
//!
//! ## 关键不变量 (INV-BA-01~12, 共 12 条)
//!
//! - **INV-BA-01** 必带 `tenant_id` + `domain` (5 域视图), 跨 tenant/跨域拒绝 (per NFR-006)
//! - **INV-BA-02** 节点执行是异步的 (Worker 拉取, 不阻塞业务事务)
//! - **INV-BA-03** DAG 拓扑无环, 创建时拓扑校验, 环检测 422 BA-006
//! - **INV-BA-04** 节点执行幂等, 失败重试用同一 `idempotency_key` (NodeId + RunId + RetryIdx)
//! - **INV-BA-05** 节点类型注册需架构师代签 SRE Lead 审批 (per 9/1 18:43 拍板 A)
//! - **INV-BA-06** `batch_event` append-only, 不可修改/删除 (审计用, 冷热分层)
//! - **INV-BA-07** 节点超时自动 kill + 标 `failed`, 超时时间 per-node 可配
//! - **INV-BA-08** 节点 shell 执行走 non-root user + 白名单命令 + 资源限制 (per ADR-0025)
//! - **INV-BA-09** batch 引擎 crash 后, running 节点可 resume (per ADR-0030 Lease 30s heartbeat)
//! - **INV-BA-10** 5 域 (player/economy/match/social/admin) 视图隔离 (per 8/21 JST 拒绝兼任)
//! - **INV-BA-11** 节点 sql 写操作走 per-tenant db role + 写操作审计
//! - **INV-BA-12** DAG 定义 schema 升版走 SCD Type 2, 老 run 用老 version
//!
//! Lead 责任: ⏳ 5 域 Lead + SRE Lead 缺位, 阶段性架构师代签 (per 9/1 18:43 JST 拍板 A)

#![warn(missing_docs)]

pub mod domain;
pub mod error;
pub mod event;
pub mod invariant;
pub mod port;

// =====================================================================
// 强类型 ID 宏 + 10 ID (inline to lib.rs per domain-automation 模式, 避免 mod reexport private)
// =====================================================================

/// 强类型 ID 宏 (per domain-automation::define_uuid_id!)
#[macro_export]
macro_rules! define_uuid_id {
    ($name:ident) => {
        #[allow(missing_docs)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub uuid::Uuid);

        impl $name {
            #[allow(dead_code)]
            pub fn new() -> Self {
                Self(uuid::Uuid::new_v4())
            }
            #[allow(dead_code)]
            pub fn from_uuid(id: uuid::Uuid) -> Self {
                Self(id)
            }
            #[allow(dead_code)]
            pub fn as_uuid(&self) -> uuid::Uuid {
                self.0
            }
            #[allow(dead_code)]
            pub fn into_uuid(self) -> uuid::Uuid {
                self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl std::ops::Deref for $name {
            type Target = uuid::Uuid;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<uuid::Uuid> for $name {
            fn from(id: uuid::Uuid) -> Self {
                Self(id)
            }
        }
    };
}

define_uuid_id!(TaskId);
define_uuid_id!(RunId);
define_uuid_id!(NodeId);
define_uuid_id!(NodeTypeId);
define_uuid_id!(EventId);
define_uuid_id!(AlertRuleId);
define_uuid_id!(SlaId);
define_uuid_id!(TenantId);
define_uuid_id!(UserId);
define_uuid_id!(WorkerId);

// =====================================================================
// Re-exports
// =====================================================================

pub use domain::{
    AlertRule, BatchDomain, Dag, DagNode, Event, Log, LogChunk, LogOffset, LogStream, Node,
    NodeExecutionResult, NodeStatus, NodeType, RegisterNodeTypeCommand, Run, RunStatus, Sla, Task,
    TaskStatus, TriggerType,
};
pub use error::{BatchError, BatchErrorCode};
pub use event::{BatchEvent, BatchEventKind, EventMeta};
pub use invariant::{
    check_all_invariants, validate_dag_topology, validate_node_type_approved, InvariantCheck,
    ALL_INVARIANT_CHECKS,
};
pub use port::{
    BatchCommandPort, BatchQueryPort, CreateTaskCommand, DagOrchestrator, ListEventQuery,
    ListNodeTypeQuery, ListRunQuery, ListTaskQuery, NodeExecutor, Scheduler, TriggerTaskCommand,
    UpdateTaskCommand, UpsertAlertRuleCommand, UpsertSlaCommand,
};
pub use star_context::ActorContext;

// =====================================================================
// ID 单测 (per 守门 #1 派生 v3 至少 1 test)
// =====================================================================

#[cfg(test)]
mod id_tests {
    use super::*;

    #[test]
    fn task_id_roundtrip() {
        let id = TaskId::new();
        let uuid = id.as_uuid();
        let back = TaskId::from_uuid(uuid);
        assert_eq!(id, back);
        assert!(!uuid.is_nil());
    }

    #[test]
    fn task_id_display() {
        let id = TaskId::new();
        let s = format!("{id}");
        assert_eq!(s, id.as_uuid().to_string());
    }
}
