//! 应用层: 4 Port trait (per docs/basic-design/charts-and-reports.md §6)
//!
//! - WorkItemQueryPort: 拉 work_item (per domain-work-item)
//! - SprintQueryPort: 拉 sprint (per domain-planning)
//! - UserQueryPort: 拉 user (per domain-identity)
//! - PermissionPort: 校验权限 (per domain-permission)

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::super::domain::c01_burndown::{CompletedIssue, SprintMeta};

/// WorkItem 查询 Port (阶段 1 仅 C01 用)
#[async_trait]
pub trait WorkItemQueryPort: Send + Sync {
    /// 列 Sprint 内所有 issue
    async fn list_in_sprint(
        &self,
        tenant_id: Uuid,
        sprint_id: Uuid,
    ) -> Result<Vec<CompletedIssue>, String>;

    /// 列 Sprint 内已完成 issue (per 时间窗)
    async fn list_completed_in_sprint(
        &self,
        tenant_id: Uuid,
        sprint_id: Uuid,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<CompletedIssue>, String>;
}

/// Sprint 查询 Port
#[async_trait]
pub trait SprintQueryPort: Send + Sync {
    async fn get_sprint(
        &self,
        tenant_id: Uuid,
        sprint_id: Uuid,
    ) -> Result<Option<SprintMeta>, String>;
}

/// User 查询 Port
#[async_trait]
pub trait UserQueryPort: Send + Sync {
    async fn get_user(
        &self,
        tenant_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<UserInfo>, String>;
}

#[derive(Debug, Clone)]
pub struct UserInfo {
    pub user_id: Uuid,
    pub name: String,
    pub avatar_url: Option<String>,
}

/// 权限校验 Port
#[async_trait]
pub trait PermissionPort: Send + Sync {
    async fn check(
        &self,
        actor_id: Uuid,
        tenant_id: Uuid,
        resource: &str,
        action: &str,
    ) -> Result<bool, String>;
}
