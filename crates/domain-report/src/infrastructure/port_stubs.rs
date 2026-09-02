//! 基础设施: 4 Port in-memory stub (阶段 1, 真实实现待 V2 接 domain-work-item 等)

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::RwLock;
use uuid::Uuid;

use crate::application::ports::*;
use crate::domain::c01_burndown::{CompletedIssue, SprintMeta};

/// InMemory WorkItem Port (返回 1 个示例 Sprint 数据, 供阶段 1 验证)
pub struct InMemoryWorkItemPort {
    data: RwLock<HashMap<Uuid, Vec<CompletedIssue>>>,
}

impl InMemoryWorkItemPort {
    pub fn new() -> Self {
        Self { data: RwLock::new(HashMap::new()) }
    }

    /// 测试辅助: 注入 fixture
    pub fn seed(&self, sprint_id: Uuid, issues: Vec<CompletedIssue>) {
        let mut d = self.data.write().unwrap();
        d.insert(sprint_id, issues);
    }
}

impl Default for InMemoryWorkItemPort {
    fn default() -> Self { Self::new() }
}

#[async_trait]
impl WorkItemQueryPort for InMemoryWorkItemPort {
    async fn list_in_sprint(
        &self,
        _tenant_id: Uuid,
        sprint_id: Uuid,
    ) -> Result<Vec<CompletedIssue>, String> {
        let d = self.data.read().map_err(|e| e.to_string())?;
        Ok(d.get(&sprint_id).cloned().unwrap_or_default())
    }

    async fn list_completed_in_sprint(
        &self,
        _tenant_id: Uuid,
        sprint_id: Uuid,
        _from: DateTime<Utc>,
        _to: DateTime<Utc>,
    ) -> Result<Vec<CompletedIssue>, String> {
        let d = self.data.read().map_err(|e| e.to_string())?;
        // 阶段 1: 全部当作 completed, 真实场景按 completed_at 过滤
        Ok(d.get(&sprint_id).cloned().unwrap_or_default())
    }
}

/// InMemory Sprint Port (1 个示例 Sprint)
pub struct InMemorySprintPort {
    data: RwLock<HashMap<Uuid, SprintMeta>>,
}

impl InMemorySprintPort {
    pub fn new() -> Self {
        Self { data: RwLock::new(HashMap::new()) }
    }

    pub fn seed(&self, sprint: SprintMeta) {
        let mut d = self.data.write().unwrap();
        d.insert(sprint.sprint_id, sprint);
    }
}

impl Default for InMemorySprintPort {
    fn default() -> Self { Self::new() }
}

#[async_trait]
impl SprintQueryPort for InMemorySprintPort {
    async fn get_sprint(
        &self,
        _tenant_id: Uuid,
        sprint_id: Uuid,
    ) -> Result<Option<SprintMeta>, String> {
        let d = self.data.read().map_err(|e| e.to_string())?;
        Ok(d.get(&sprint_id).cloned())
    }
}

pub struct InMemoryUserPort;

impl InMemoryUserPort { pub fn new() -> Self { Self } }

#[async_trait]
impl UserQueryPort for InMemoryUserPort {
    async fn get_user(
        &self,
        _tenant_id: Uuid,
        _user_id: Uuid,
    ) -> Result<Option<UserInfo>, String> {
        Ok(None)
    }
}

pub struct InMemoryPermissionPort;

impl InMemoryPermissionPort { pub fn new() -> Self { Self } }

#[async_trait]
impl PermissionPort for InMemoryPermissionPort {
    async fn check(
        &self,
        _actor_id: Uuid,
        _tenant_id: Uuid,
        _resource: &str,
        _action: &str,
    ) -> Result<bool, String> {
        // 阶段 1: 全部放行, 真实权限接 domain-permission
        Ok(true)
    }
}
