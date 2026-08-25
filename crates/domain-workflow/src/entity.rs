//! Workflow 域实体(Entity / Aggregate Root)
//!
//! 来源:
//! - `docs/data-design.md` §4.5 (`workflow` schema)
//! - `docs/specs/domain-workflow-spec.md` §2 (实体清单)
//!
//! 包含 3 个核心实体 + 1 个 system_default seed:
//! - `WorkflowDefinition` — 主聚合根(13 字段)
//! - `State` — 工作流状态(9 字段)
//! - `Transition` — 状态迁移(8 字段)
//! - `SystemDefault` — system_default seed 标识(spec §2)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::value_object::{
    ProjectId, StateCategory, StateId, TenantId, TransitionId, UserId, WorkflowId,
};

// =====================================================================
// WorkflowDefinition 聚合根
// =====================================================================

/// **WorkflowDefinition 聚合根**(继承 `data-design §4.5` DDL)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    /// 主键 UUID
    pub id: WorkflowId,

    /// 租户 ID(必带,§6.1,REQ-SEC-001)
    pub tenant_id: TenantId,

    /// Project ID(可空 → system_default)
    pub project_id: Option<ProjectId>,

    /// 名称
    pub name: String,

    /// 描述
    pub description: Option<String>,

    /// 版本号(单版本,future: 多版本支持;见 J-WF-01)
    pub version: u32,

    /// 是否为平台级 system_default(只读)
    pub is_system_default: bool,

    /// 初始 State ID(INV-WF-02:必须存在且唯一)
    pub initial_state_id: StateId,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 更新时间
    pub updated_at: DateTime<Utc>,

    /// 乐观锁版本号
    pub lock_version: u32,

    /// 创建者
    pub created_by_user_id: UserId,
}

impl WorkflowDefinition {
    /// 字段数(用于 §4.5 DDL 对齐审计)
    pub const FIELD_COUNT: usize = 13;

    /// 升级乐观锁版本号
    pub fn bump_version(&mut self) {
        self.lock_version = self.lock_version.saturating_add(1);
        self.updated_at = Utc::now();
    }

    /// 是否为 system_default(INV-WF-01:只读保护)
    pub fn is_read_only(&self) -> bool {
        self.is_system_default
    }
}

// =====================================================================
// State 实体
// =====================================================================

/// **State**(Workflow 的状态,9 字段)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    /// 主键
    pub id: StateId,

    /// Workflow ID
    pub workflow_id: WorkflowId,

    /// 租户 ID(必带,§6.1)
    pub tenant_id: TenantId,

    /// 状态名(如 TODO / IN_PROGRESS / DONE / IN_REVIEW)
    pub name: String,

    /// 类别(Initial / Intermediate / Terminal)
    pub category: StateCategory,

    /// 显示颜色(HEX 字符串)
    pub display_color: Option<String>,

    /// 显示顺序
    pub display_order: u32,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

impl State {
    /// 字段数
    pub const FIELD_COUNT: usize = 9;
}

// =====================================================================
// Transition 实体
// =====================================================================

/// **Transition**(状态迁移定义,8 字段)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transition {
    /// 主键
    pub id: TransitionId,

    /// Workflow ID
    pub workflow_id: WorkflowId,

    /// 租户 ID(必带)
    pub tenant_id: TenantId,

    /// 源 State
    pub from_state_id: StateId,

    /// 目标 State
    pub to_state_id: StateId,

    /// 所需权限字符串(如 "workflow:transition:approve")
    pub required_permission: Option<String>,

    /// 所需角色(如 "project_admin")
    pub required_role: Option<String>,

    /// 触发事件(可选)
    pub trigger_event: Option<String>,
}

impl Transition {
    /// 字段数
    pub const FIELD_COUNT: usize = 8;
}

// =====================================================================
// SystemDefault seed 引用
// =====================================================================

/// **SystemDefault**(平台级只读 seed 标识)
///
/// spec §2:`预置 3 态 TODO → IN_PROGRESS → DONE,所有 Tenant 共享`。
/// 实际 State / Transition 数据由 `service.rs` 的 `seed_system_default()` 函数初始化。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemDefault {
    /// 平台级 Workflow ID(所有 Tenant 共享同一 workflow_id)
    pub workflow_id: WorkflowId,
    /// 初始 State ID(TODO)
    pub initial_state_id: StateId,
    /// system_default marker(由数据库触发器或 RLS 强制)
    pub is_system_default: bool,
}
