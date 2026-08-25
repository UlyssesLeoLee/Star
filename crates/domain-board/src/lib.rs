//! Board 视图领域
//!
//! **crate**: `domain-board`
//! **上游 spec**: docs/specs/domain-board-spec.md §9 / §10 Board 视图
//! **基本设计**: docs/basic-design.md §2.1 / §4.10.1
//! **数据设计**: docs/data-design.md §4.11 (`board` schema)
//! **API 设计**: docs/api-design.md §3.14 (Board / Column / Swimlane)
//!
//! ## 职责
//!
//! 详细职责边界见 spec 文档第 1 节。骨架阶段仅声明 Port trait + Entity + Error,
//! 具体实现由 `crates/infrastructure` 中的 Adapter 提供。
//!
//! ## 关键不变量
//!
//! //! - Board 与 Sprint/Gantt 共享数据模型(§9,REQ-PLAN-003)
//! - Column 由 WorkflowDefinition 派生,不允许独立定义状态(§4.10.1)

//! ## 上游依赖(basic-design §2.3)
//!
//! 本 crate 依赖以下 domain-*(骨架阶段不实际 import,Cargo.toml 仅声明本 crate 自身需要的外部依赖):
//!
//!   - `domain-work-item`
//!   - `domain-planning`
//!
//! **禁止反向依赖**(§2.3 禁线)。

//! ## 关键引用
//!
//! Board/Sprint/Gantt 共享 WorkItem 投影(§9)

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// =====================================================================
// 实体(Entity / Aggregate Root)
// =====================================================================

/// Board (聚合根 / 实体)
///
/// 来源: docs/data-design.md §4.11 (`board` schema)
///
/// **骨架阶段**: 仅占位字段,完整字段与不变量留待 Phase 2。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    /// 主键 UUID
    pub id: Uuid,
    /// 租户隔离(13 类对象必带,§6.1)
    pub tenant_id: Uuid,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 更新时间
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Column (聚合根 / 实体)
///
/// 来源: docs/data-design.md §4.11 (`board` schema)
///
/// **骨架阶段**: 仅占位字段,完整字段与不变量留待 Phase 2。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    /// 主键 UUID
    pub id: Uuid,
    /// 租户隔离(13 类对象必带,§6.1)
    pub tenant_id: Uuid,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 更新时间
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Swimlane (聚合根 / 实体)
///
/// 来源: docs/data-design.md §4.11 (`board` schema)
///
/// **骨架阶段**: 仅占位字段,完整字段与不变量留待 Phase 2。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Swimlane {
    /// 主键 UUID
    pub id: Uuid,
    /// 租户隔离(13 类对象必带,§6.1)
    pub tenant_id: Uuid,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 更新时间
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

// =====================================================================
// 端口(Port / 抽象)
// =====================================================================

/// **BoardCommandPort**(命令端口)
///
/// 来源: docs/api-design.md §3.14 (Board / Column / Swimlane)
///
/// **骨架阶段**: 仅方法签名,无 body 实现。Phase 2 在
/// `crates/infrastructure/<adapter>.rs` 中提供 SQLx / NATS / SCM Adapter 实现。
#[async_trait]
pub trait BoardCommandPort: Send + Sync {
    async fn create_board(
        &self,
        cmd: CreateBoardCommand,
        actor: ActorContext,
    ) -> Result<BoardId, BoardError>;
    async fn add_column(
        &self,
        cmd: AddColumnCommand,
        actor: ActorContext,
    ) -> Result<Column, BoardError>;
    async fn add_swimlane(
        &self,
        cmd: AddSwimlaneCommand,
        actor: ActorContext,
    ) -> Result<Swimlane, BoardError>;
    async fn reorder_columns(
        &self,
        cmd: ReorderColumnsCommand,
        actor: ActorContext,
    ) -> Result<Board, BoardError>;
}


/// **BoardQueryPort**(查询端口)
///
/// 来源: docs/api-design.md §3.14 (Board / Column / Swimlane)
#[async_trait]
pub trait BoardQueryPort: Send + Sync {
    async fn get_board(
        &self,
        id: BoardId,
        viewer: ActorContext,
    ) -> Result<Board, BoardError>;
    async fn list_by_project(
        &self,
        id: ProjectId,
        viewer: ActorContext,
    ) -> Result<Vec<Board>, BoardError>;
}

// =====================================================================
// Domain Events(CloudEvents 1.0,见 api-design §5)
// =====================================================================

/// Domain Event: `star.events.board.board.created.v1`
///
/// 来源: docs/api-design.md §5 (CloudEvents 1.0)
///
/// **骨架阶段**: 仅占位字段,Phase 2 补充完整 Payload 字段。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board01Event {
    /// 事件唯一 ID(UUIDv7)
    pub event_id: Uuid,
    /// 租户 ID(必带)
    pub tenant_id: Uuid,
    /// 事件发生时间
    pub occurred_at: chrono::DateTime<chrono::Utc>,
}

/// Domain Event: `star.events.board.column.moved.v1`
///
/// 来源: docs/api-design.md §5 (CloudEvents 1.0)
///
/// **骨架阶段**: 仅占位字段,Phase 2 补充完整 Payload 字段。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board02Event {
    /// 事件唯一 ID(UUIDv7)
    pub event_id: Uuid,
    /// 租户 ID(必带)
    pub tenant_id: Uuid,
    /// 事件发生时间
    pub occurred_at: chrono::DateTime<chrono::Utc>,
}

// =====================================================================
// 类型别名与命令/查询/返回类型占位
// =====================================================================
/// **ID 类型别名**(Phase 1 骨架:均为 UUID 别名)
///
/// 真实使用应由 `domain-identity` 颁发强类型 ID(§23.2);
/// 骨架阶段以 `Uuid` 替代以避免跨 crate 编译依赖。

pub type BoardId = Uuid;
pub type ColumnId = Uuid;
pub type ProjectId = Uuid;
pub type SwimlaneId = Uuid;

/// **命令 / 查询 / 跨 crate 类型占位结构**(Phase 1 骨架:最小字段集)

/// Phase 2 由具体 spec 在 `domain-*` 内补全字段;`crates/application` 等
/// supporting crate 的占位则在 Phase 2 删除,改为 `use domain_xxx::*;` 引用。

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddColumnCommand {
    /// 主键 UUID
    pub id: Uuid,
    /// 租户 ID(13 类对象必带,§6.1)
    pub tenant_id: Uuid,
    // 其它字段在 Phase 2 由具体 spec 补充
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddSwimlaneCommand {
    /// 主键 UUID
    pub id: Uuid,
    /// 租户 ID(13 类对象必带,§6.1)
    pub tenant_id: Uuid,
    // 其它字段在 Phase 2 由具体 spec 补充
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBoardCommand {
    /// 主键 UUID
    pub id: Uuid,
    /// 租户 ID(13 类对象必带,§6.1)
    pub tenant_id: Uuid,
    // 其它字段在 Phase 2 由具体 spec 补充
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReorderColumnsCommand {
    /// 主键 UUID
    pub id: Uuid,
    /// 租户 ID(13 类对象必带,§6.1)
    pub tenant_id: Uuid,
    // 其它字段在 Phase 2 由具体 spec 补充
}


// =====================================================================
// Error
// =====================================================================

/// **Board 错误**
///
/// 来源: docs/api-design.md §8 (错误码)
/// 5 个标准变体;具体错误码在 Phase 2 由本 enum 派生 + 实现 `Into<ApiError>`。
#[derive(Debug, thiserror::Error)]
pub enum BoardError {
    #[error("not found: {0}")]
    NotFound(Uuid),
    #[error("invalid state: {0}")]
    InvalidState(String),
    #[error("permission denied")]
    PermissionDenied,
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("internal: {0}")]
    Internal(String),
}

// =====================================================================
// 共享类型
// =====================================================================

/// **Actor 上下文**(来自 `domain-identity` / `domain-permission` 的 JWT claim)
///
/// **骨架阶段**: 字段占位;Phase 2 由 `domain-identity` 颁发的 ActorContext 取代
/// 本 crate 内的占位定义(避免循环依赖)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorContext {
    /// 当前用户 ID
    pub user_id: Uuid,
    /// 当前租户 ID(13 类对象必带,§6.1)
    pub tenant_id: Uuid,
    /// 当前设备 ID(Local Runtime 三重绑定,§23.2)
    pub device_id: Option<Uuid>,
    /// 当前 Project IDs(用于 Project Policy 校验)
    pub project_ids: Vec<Uuid>,
    /// 当前用户角色(`tenant_admin` / `project_admin` / `developer` / `viewer`)
    pub roles: Vec<String>,
}

// =====================================================================
// 单元测试占位
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// **骨架阶段**: 最小冒烟测试,验证 crate 可编译、ActorContext 字段可达。
    /// Phase 2 由具体 spec 引入完整单元测试(状态机覆盖 / RLS 矩阵等)。
    #[test]
    fn actor_context_skeleton() {
        let actor = ActorContext {
            user_id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            device_id: None,
            project_ids: vec![],
            roles: vec!["developer".to_string()],
        };
        assert!(!actor.tenant_id.is_nil(), "tenant_id must be non-nil (§6.1,REQ-SEC-001)");
    }
}
