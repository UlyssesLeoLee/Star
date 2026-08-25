//! Worktree 领域
//!
//! **crate**: `domain-worktree`
//! **上游 spec**: docs/specs/domain-worktree-spec.md §22 Worktree 生命周期
//! **基本设计**: docs/basic-design.md §2.1 / §4.1 / §7 状态机
//! **数据设计**: docs/data-design.md §4.2 (`worktree` schema)
//! **API 设计**: docs/api-design.md §3.6 (Worktree CRUD + 状态机)
//!
//! ## 职责
//!
//! 详细职责边界见 spec 文档第 1 节。骨架阶段仅声明 Port trait + Entity + Error,
//! 具体实现由 `crates/infrastructure` 中的 Adapter 提供。
//!
//! ## 关键不变量
//!
//! //! - Worktree Status 独立于 WorkItem Status(§22.2,REQ-WF-002)
//! - Worktree 17 状态机:Created/Initializing/Ready/Dirty/Cleaning/Conflict/.../Archived(§7)
//! - 1 Worktree 绑定 1 Runtime(Local Daemon);1 Runtime 可承载 N Worktree(§23)

//! ## 上游依赖(basic-design §2.3)
//!
//! 本 crate 依赖以下 domain-*(骨架阶段不实际 import,Cargo.toml 仅声明本 crate 自身需要的外部依赖):
//!
//!   - `domain-work-item`
//!   - `domain-scm`
//!   - `domain-development`
//!
//! **禁止反向依赖**(§2.3 禁线)。

//! ## 关键引用
//!
//! Worktree 17 状态机(§7.6):详见 lib.rs 注释

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// =====================================================================
// 实体(Entity / Aggregate Root)
// =====================================================================

/// Worktree (聚合根 / 实体)
///
/// 来源: docs/data-design.md §4.2 (`worktree` schema)
///
/// **骨架阶段**: 仅占位字段,完整字段与不变量留待 Phase 2。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Worktree {
    /// 主键 UUID
    pub id: Uuid,
    /// 租户隔离(13 类对象必带,§6.1)
    pub tenant_id: Uuid,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 更新时间
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// ConflictState (聚合根 / 实体)
///
/// 来源: docs/data-design.md §4.2 (`worktree` schema)
///
/// **骨架阶段**: 仅占位字段,完整字段与不变量留待 Phase 2。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictState {
    /// 主键 UUID
    pub id: Uuid,
    /// 租户隔离(13 类对象必带,§6.1)
    pub tenant_id: Uuid,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 更新时间
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// HealthState (聚合根 / 实体)
///
/// 来源: docs/data-design.md §4.2 (`worktree` schema)
///
/// **骨架阶段**: 仅占位字段,完整字段与不变量留待 Phase 2。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthState {
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

/// **WorktreeCommandPort**(命令端口)
///
/// 来源: docs/api-design.md §3.6 (Worktree CRUD + 状态机)
///
/// **骨架阶段**: 仅方法签名,无 body 实现。Phase 2 在
/// `crates/infrastructure/<adapter>.rs` 中提供 SQLx / NATS / SCM Adapter 实现。
#[async_trait]
pub trait WorktreeCommandPort: Send + Sync {
    async fn create_worktree(
        &self,
        cmd: CreateWorktreeCommand,
        actor: ActorContext,
    ) -> Result<WorktreeId, WorktreeError>;
    async fn update_worktree(
        &self,
        cmd: UpdateWorktreeCommand,
        actor: ActorContext,
    ) -> Result<Worktree, WorktreeError>;
    async fn delete_worktree(
        &self,
        cmd: WorktreeId,
        actor: ActorContext,
    ) -> Result<(), WorktreeError>;
    async fn transition_status(
        &self,
        cmd: TransitionWorktreeStatusCommand,
        actor: ActorContext,
    ) -> Result<Worktree, WorktreeError>;
    async fn attach_runtime(
        &self,
        cmd: AttachRuntimeCommand,
        actor: ActorContext,
    ) -> Result<Worktree, WorktreeError>;
    async fn register_observation(
        &self,
        cmd: RegisterObservationCommand,
        actor: ActorContext,
    ) -> Result<Worktree, WorktreeError>;
}


/// **WorktreeQueryPort**(查询端口)
///
/// 来源: docs/api-design.md §3.6 (Worktree CRUD + 状态机)
#[async_trait]
pub trait WorktreeQueryPort: Send + Sync {
    async fn get_by_id(
        &self,
        id: WorktreeId,
        viewer: ActorContext,
    ) -> Result<WorkItem, WorktreeError>;
    async fn list_by_work_item(
        &self,
        id: WorkItemId,
        viewer: ActorContext,
    ) -> Result<Vec<WorkItem>, WorktreeError>;
    async fn list_by_runtime(
        &self,
        id: RuntimeId,
        viewer: ActorContext,
    ) -> Result<Vec<WorkItem>, WorktreeError>;
    async fn list_observations(
        &self,
        id: WorktreeId,
        viewer: ActorContext,
    ) -> Result<Vec<RuntimeObservation>, WorktreeError>;
}

// =====================================================================
// Domain Events(CloudEvents 1.0,见 api-design §5)
// =====================================================================

/// Domain Event: `star.events.worktree.worktree.created.v1`
///
/// 来源: docs/api-design.md §5 (CloudEvents 1.0)
///
/// **骨架阶段**: 仅占位字段,Phase 2 补充完整 Payload 字段。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Worktree01Event {
    /// 事件唯一 ID(UUIDv7)
    pub event_id: Uuid,
    /// 租户 ID(必带)
    pub tenant_id: Uuid,
    /// 事件发生时间
    pub occurred_at: chrono::DateTime<chrono::Utc>,
}

/// Domain Event: `star.events.worktree.worktree.status_changed.v1`
///
/// 来源: docs/api-design.md §5 (CloudEvents 1.0)
///
/// **骨架阶段**: 仅占位字段,Phase 2 补充完整 Payload 字段。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Worktree02Event {
    /// 事件唯一 ID(UUIDv7)
    pub event_id: Uuid,
    /// 租户 ID(必带)
    pub tenant_id: Uuid,
    /// 事件发生时间
    pub occurred_at: chrono::DateTime<chrono::Utc>,
}

/// Domain Event: `star.events.worktree.worktree.observation_registered.v1`
///
/// 来源: docs/api-design.md §5 (CloudEvents 1.0)
///
/// **骨架阶段**: 仅占位字段,Phase 2 补充完整 Payload 字段。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Worktree03Event {
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

pub type ConflictStateId = Uuid;
pub type HealthStateId = Uuid;
pub type RuntimeId = Uuid;
pub type WorkItemId = Uuid;
pub type WorktreeId = Uuid;

/// **命令 / 查询 / 跨 crate 类型占位结构**(Phase 1 骨架:最小字段集)

/// Phase 2 由具体 spec 在 `domain-*` 内补全字段;`crates/application` 等
/// supporting crate 的占位则在 Phase 2 删除,改为 `use domain_xxx::*;` 引用。

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachRuntimeCommand {
    /// 主键 UUID
    pub id: Uuid,
    /// 租户 ID(13 类对象必带,§6.1)
    pub tenant_id: Uuid,
    // 其它字段在 Phase 2 由具体 spec 补充
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWorktreeCommand {
    /// 主键 UUID
    pub id: Uuid,
    /// 租户 ID(13 类对象必带,§6.1)
    pub tenant_id: Uuid,
    // 其它字段在 Phase 2 由具体 spec 补充
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterObservationCommand {
    /// 主键 UUID
    pub id: Uuid,
    /// 租户 ID(13 类对象必带,§6.1)
    pub tenant_id: Uuid,
    // 其它字段在 Phase 2 由具体 spec 补充
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeObservation {
    /// 主键 UUID
    pub id: Uuid,
    /// 租户 ID(13 类对象必带,§6.1)
    pub tenant_id: Uuid,
    // 其它字段在 Phase 2 由具体 spec 补充
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionWorktreeStatusCommand {
    /// 主键 UUID
    pub id: Uuid,
    /// 租户 ID(13 类对象必带,§6.1)
    pub tenant_id: Uuid,
    // 其它字段在 Phase 2 由具体 spec 补充
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateWorktreeCommand {
    /// 主键 UUID
    pub id: Uuid,
    /// 租户 ID(13 类对象必带,§6.1)
    pub tenant_id: Uuid,
    // 其它字段在 Phase 2 由具体 spec 补充
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkItem {
    /// 主键 UUID
    pub id: Uuid,
    /// 租户 ID(13 类对象必带,§6.1)
    pub tenant_id: Uuid,
    // 其它字段在 Phase 2 由具体 spec 补充
}


// =====================================================================
// Error
// =====================================================================

/// **Worktree 错误**
///
/// 来源: docs/api-design.md §8 (错误码)
/// 5 个标准变体;具体错误码在 Phase 2 由本 enum 派生 + 实现 `Into<ApiError>`。
#[derive(Debug, thiserror::Error)]
pub enum WorktreeError {
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
