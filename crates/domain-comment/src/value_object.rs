//! Comment 域值对象

use serde::{Deserialize, Serialize};

use crate::define_uuid_id;

define_uuid_id!(CommentId);
define_uuid_id!(MentionId);
define_uuid_id!(AttachmentId);
define_uuid_id!(ReactionId);
define_uuid_id!(TenantId);
define_uuid_id!(UserId);
define_uuid_id!(AgentId);
define_uuid_id!(ProjectId);

/// 强类型 WorkItem ID(避免依赖 domain-work-item)
define_uuid_id!(WorkItemId);

/// 强类型 PullRequest ID(避免依赖 domain-scm)
define_uuid_id!(PullRequestId);

/// 强类型 Discussion ID(避免依赖 domain-collaboration)
define_uuid_id!(DiscussionId);

// =====================================================================
// 枚举:ParentType
// =====================================================================

/// **Comment 父类型**
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ParentType {
    /// 父为 WorkItem
    WorkItem,
    /// 父为 PullRequest
    PullRequest,
    /// 父为 Discussion
    Discussion,
}

impl Default for ParentType {
    fn default() -> Self {
        Self::WorkItem
    }
}

impl std::fmt::Display for ParentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::WorkItem => "WORK_ITEM",
            Self::PullRequest => "PULL_REQUEST",
            Self::Discussion => "DISCUSSION",
        };
        f.write_str(s)
    }
}

// =====================================================================
// 枚举:CommentStatus
// =====================================================================

/// **Comment 状态**(spec §2)
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CommentStatus {
    /// 正常
    Open,
    /// 已编辑
    Edited,
    /// 已删除(软删除)
    Deleted,
}

impl Default for CommentStatus {
    fn default() -> Self {
        Self::Open
    }
}

impl std::fmt::Display for CommentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Open => "OPEN",
            Self::Edited => "EDITED",
            Self::Deleted => "DELETED",
        };
        f.write_str(s)
    }
}

// =====================================================================
// 角色
// =====================================================================

pub mod roles {
    pub const TENANT_ADMIN: &str = "tenant_admin";
    pub const PROJECT_ADMIN: &str = "project_admin";
    pub const DEVELOPER: &str = "developer";
    pub const VIEWER: &str = "viewer";
}
