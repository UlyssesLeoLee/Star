//! WorkItem 域值对象(Value Objects)
//!
//! 来源:
//! - `docs/data-design.md` §4.4 (work_item DDL:6 类型 / 6 状态 / 4 优先级 / 4 严重度)
//! - `docs/specs/domain-work-item-spec.md` §3 (基本类型 / 状态机)
//!
//! 本模块集中放置强类型 ID 与 6 个核心 enum,与 `entity` / `port` 解耦。

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// =====================================================================
// 强类型 ID(强类型 newtype,Phase 2 实现)
// =====================================================================

macro_rules! define_uuid_id {
    ($name:ident, $doc:expr) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub Uuid);

        impl $name {
            /// 创建新的强类型 ID(包装 `Uuid::new_v4`)。
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            /// 从已有 UUID 构造(由 infra 适配器在读取 DB 后使用)。
            pub fn from_uuid(id: Uuid) -> Self {
                Self(id)
            }

            /// 取内部 UUID 引用。
            pub fn as_uuid(&self) -> &Uuid {
                &self.0
            }

            /// 取出内部 UUID(consume)。
            pub fn into_uuid(self) -> Uuid {
                self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }

        impl std::ops::Deref for $name {
            type Target = Uuid;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl From<Uuid> for $name {
            fn from(u: Uuid) -> Self {
                Self(u)
            }
        }

        impl From<$name> for Uuid {
            fn from(id: $name) -> Self {
                id.0
            }
        }
    };
}

define_uuid_id!(WorkItemId, "WorkItem 主键(UUID newtype,继承 data-design §4.4.1)");
define_uuid_id!(RequirementId, "Requirement 主键 UUID(继承 data-design §4.4.2)");
define_uuid_id!(AcceptanceCriterionId, "AcceptanceCriterion 主键 UUID(继承 data-design §4.4.3)");
define_uuid_id!(BusinessGoalId, "BusinessGoal 主键 UUID(继承 data-design §4.4.4)");
define_uuid_id!(TenantId, "租户 ID(13 类对象必带,§6.1)");
define_uuid_id!(WorkspaceId, "Workspace ID(继承 workspace.workspace)");
define_uuid_id!(ProjectId, "Project ID(继承 project.project)");
define_uuid_id!(UserId, "用户 ID(继承 identity.user)");
define_uuid_id!(AgentId, "Agent ID(继承 agent.agent,AI 任务分配)");
define_uuid_id!(SprintId, "Sprint ID(继承 planning.sprint)");
define_uuid_id!(RepositoryId, "Repository ID(继承 scm.repository)");
define_uuid_id!(WorktreeId, "Worktree ID(继承 worktree.worktree)");

// =====================================================================
// Enum:WorkItemType(data-design §4.4.1 ck_work_item_type)
// =====================================================================

/// **WorkItem 类型**(`work_item.type` 列,CHECK 约束白名单)
///
/// 来源: docs/data-design.md §4.4.1 (`ck_work_item_type`)
/// 6 种合法类型:Epic / Story / Task / Bug / Subtask / AITask
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum WorkItemType {
    /// Epic — 大型业务主题,跨多个 Sprint
    Epic,
    /// Story — 用户故事,可独立交付
    Story,
    /// Task — 具体执行任务
    Task,
    /// Bug — 缺陷修复
    Bug,
    /// Subtask — Story/Task 的子项,**必须**带 `parent_work_item_id`(INV-WI-08)
    Subtask,
    /// AITask — AI 任务,**必须**先有 Repository Link + Agent Policy + Validation Policy(INV-WI-05)
    AITask,
}

impl WorkItemType {
    /// 该类型是否要求 `parent_work_item_id` 非空(INV-WI-08)。
    pub fn requires_parent(self) -> bool {
        matches!(self, Self::Subtask)
    }

    /// 该类型是否要求 Repository + Agent + Validation 链接(INV-WI-05)。
    pub fn requires_ai_prerequisites(self) -> bool {
        matches!(self, Self::AITask)
    }
}

impl std::fmt::Display for WorkItemType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Epic => "Epic",
            Self::Story => "Story",
            Self::Task => "Task",
            Self::Bug => "Bug",
            Self::Subtask => "Subtask",
            Self::AITask => "AITask",
        };
        f.write_str(s)
    }
}

// =====================================================================
// Enum:WorkItemStatus(data-design §4.4.1 ck_work_item_status)
// =====================================================================

/// **WorkItem 状态**(状态机:`work_item.status` 列)
///
/// 来源: docs/data-design.md §4.4.1 (`ck_work_item_status`)
/// 默认 3 态 TODO → IN_PROGRESS → DONE(INV-WI-01,REQ-WF-001)
/// 扩展 3 态 IN_REVIEW / BLOCKED / CANCELLED 由 Project Policy 显式启用
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WorkItemStatus {
    /// 待办(默认初始状态)
    TODO,
    /// 进行中
    IN_PROGRESS,
    /// 完成
    DONE,
    /// 评审中(扩展状态,需 Project Policy 启用)
    IN_REVIEW,
    /// 阻塞(扩展状态,需 Project Policy 启用)
    BLOCKED,
    /// 取消(扩展状态,需 Project Policy 启用)
    CANCELLED,
}

impl WorkItemStatus {
    /// 是否属于默认 3 态(INV-WI-01)。
    pub fn is_default_state(self) -> bool {
        matches!(self, Self::TODO | Self::IN_PROGRESS | Self::DONE)
    }

    /// 是否属于扩展 3 态(需 Project Policy 显式启用)。
    pub fn is_extended_state(self) -> bool {
        matches!(self, Self::IN_REVIEW | Self::BLOCKED | Self::CANCELLED)
    }

    /// 默认三态内的合法迁移(由 INV-WI-09 派生的兜底,完整迁移由 WorkflowDefinition 决定)。
    pub fn can_transition_default(self, target: Self) -> bool {
        use WorkItemStatus::*;
        match (self, target) {
            (TODO, IN_PROGRESS) => true,
            (IN_PROGRESS, DONE) => true,
            (IN_PROGRESS, TODO) => true, // 撤回
            (TODO, DONE) => true,        // 跳过 IN_PROGRESS(快速关闭)
            (DONE, IN_PROGRESS) => true, // 重开
            (DONE, TODO) => true,        // 撤回完成
            _ => false,
        }
    }
}

impl std::fmt::Display for WorkItemStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::TODO => "TODO",
            Self::IN_PROGRESS => "IN_PROGRESS",
            Self::DONE => "DONE",
            Self::IN_REVIEW => "IN_REVIEW",
            Self::BLOCKED => "BLOCKED",
            Self::CANCELLED => "CANCELLED",
        };
        f.write_str(s)
    }
}

impl std::str::FromStr for WorkItemStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "TODO" => Ok(Self::TODO),
            "IN_PROGRESS" => Ok(Self::IN_PROGRESS),
            "DONE" => Ok(Self::DONE),
            "IN_REVIEW" => Ok(Self::IN_REVIEW),
            "BLOCKED" => Ok(Self::BLOCKED),
            "CANCELLED" => Ok(Self::CANCELLED),
            other => Err(format!("invalid WorkItemStatus: {other}")),
        }
    }
}

// =====================================================================
// Enum:Priority(data-design §4.4.1 ck_work_item_priority)
// =====================================================================

/// **优先级**(`work_item.priority`,P0 最高 / P3 最低)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    /// P0 — 紧急
    P0,
    /// P1 — 高
    P1,
    /// P2 — 中
    P2,
    /// P3 — 低(默认)
    P3,
}

impl Default for Priority {
    fn default() -> Self {
        Self::P3
    }
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::P0 => "P0",
            Self::P1 => "P1",
            Self::P2 => "P2",
            Self::P3 => "P3",
        };
        f.write_str(s)
    }
}

impl std::str::FromStr for Priority {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "P0" => Ok(Self::P0),
            "P1" => Ok(Self::P1),
            "P2" => Ok(Self::P2),
            "P3" => Ok(Self::P3),
            other => Err(format!("invalid Priority: {other}")),
        }
    }
}

// =====================================================================
// Enum:Severity(data-design §4.4.1 ck_work_item_severity,Bug 专用)
// =====================================================================

/// **严重度**(`work_item.severity`,Bug 类型专用)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Severity {
    /// P0 — 致命
    Critical,
    /// P1 — 严重
    Major,
    /// P2 — 一般
    Normal,
    /// P3 — 提示
    Minor,
}

impl Default for Severity {
    fn default() -> Self {
        Self::Normal
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Critical => "Critical",
            Self::Major => "Major",
            Self::Normal => "Normal",
            Self::Minor => "Minor",
        };
        f.write_str(s)
    }
}

// =====================================================================
// Enum:RelationType(work_item_relation 数据设计 §4.4.7)
// =====================================================================

/// **WorkItem 关系类型**(`work_item_relation.relation_type`)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationType {
    /// 阻塞
    Blocks,
    /// 被阻塞
    IsBlockedBy,
    /// 复制
    Duplicates,
    /// 重复(被复制)
    IsDuplicatedBy,
    /// 关联
    Relates,
    /// 父子(已由 parent_work_item_id 覆盖,这里只用于横向关联)
    ParentChild,
    /// 子项
    ChildOf,
}

impl std::fmt::Display for RelationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Blocks => "blocks",
            Self::IsBlockedBy => "is_blocked_by",
            Self::Duplicates => "duplicates",
            Self::IsDuplicatedBy => "is_duplicated_by",
            Self::Relates => "relates",
            Self::ParentChild => "parent_child",
            Self::ChildOf => "child_of",
        };
        f.write_str(s)
    }
}

// =====================================================================
// 角色常量(便于测试与调用方使用)
// =====================================================================

/// 标准角色字符串(对应 `ActorContext.roles` 的常见取值)。
pub mod roles {
    /// 租户管理员
    pub const TENANT_ADMIN: &str = "tenant_admin";
    /// 项目管理员
    pub const PROJECT_ADMIN: &str = "project_admin";
    /// 开发
    pub const DEVELOPER: &str = "developer";
    /// 只读访客
    pub const VIEWER: &str = "viewer";
}
