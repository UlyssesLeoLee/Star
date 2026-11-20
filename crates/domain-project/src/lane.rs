/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"crates/domain-project/src/lane.rs"})
MERGE (lane:Symbol {id:"crates/domain-project/src/lane.rs::Lane",kind:"struct"})
MERGE (laneId:Symbol {id:"crates/domain-project/src/lane.rs::LaneId",kind:"struct"})
MERGE (createCmd:Symbol {id:"crates/domain-project/src/lane.rs::CreateLaneCommand",kind:"struct"})
MERGE (updateCmd:Symbol {id:"crates/domain-project/src/lane.rs::UpdateLaneCommand",kind:"struct"})
MERGE (deleteCmd:Symbol {id:"crates/domain-project/src/lane.rs::DeleteLaneCommand",kind:"struct"})
MERGE (listQ:Symbol {id:"crates/domain-project/src/lane.rs::ListLanesQuery",kind:"struct"})
MERGE (getQ:Symbol {id:"crates/domain-project/src/lane.rs::GetLaneQuery",kind:"struct"})
MERGE (err:Symbol {id:"crates/domain-project/src/lane.rs::LaneError",kind:"enum"})
MERGE (cmdPort:Symbol {id:"crates/domain-project/src/lane.rs::LaneCommandPort",kind:"interface"})
MERGE (qPort:Symbol {id:"crates/domain-project/src/lane.rs::LaneQueryPort",kind:"interface"})
MERGE (repo:Symbol {id:"crates/domain-project/src/lane.rs::LaneRepository",kind:"interface"})
MERGE (svc:Symbol {id:"crates/domain-project/src/lane.rs::InMemoryLaneService",kind:"struct"})
MERGE (memRepo:Symbol {id:"crates/domain-project/src/lane.rs::InMemoryLaneRepository",kind:"struct"})
MERGE (self)-[:DEFINES]->(lane)
MERGE (self)-[:DEFINES]->(laneId)
MERGE (self)-[:DEFINES]->(createCmd)
MERGE (self)-[:DEFINES]->(updateCmd)
MERGE (self)-[:DEFINES]->(deleteCmd)
MERGE (self)-[:DEFINES]->(listQ)
MERGE (self)-[:DEFINES]->(getQ)
MERGE (self)-[:DEFINES]->(err)
MERGE (self)-[:DEFINES]->(cmdPort)
MERGE (self)-[:DEFINES]->(qPort)
MERGE (self)-[:DEFINES]->(repo)
MERGE (self)-[:DEFINES]->(svc)
MERGE (self)-[:DEFINES]->(memRepo)
MERGE (svc)-[:IMPLEMENTS]->(cmdPort)
MERGE (svc)-[:IMPLEMENTS]->(qPort)
MERGE (memRepo)-[:IMPLEMENTS]->(repo)
@endcypher
*/
//! Lane 域实体 (per 2026-11-15 用户发令: 导航统一 Project → Lane → Worktree → Branch)
//!
//! ## 职责
//!
//! Lane 是 Project 之下的管理层级实体。创建 Lane **不要求** 远端仓库或分支关联 —
//! 仅要求 Project 成员/所有者身份; 远端关联是可选的未来元数据, 不是前置条件。
//!
//! ## 关键不变量
//!
//! - INV-LANE-01: Lane 必属 Project + Tenant (tenant_id/project_id 必带)
//! - INV-LANE-02: 无远端仓库/分支也能创建、列出、授权、持久化 (P0 验收第一条)
//! - INV-LANE-03: 读 = Project 成员; 写 (创建/改名/删除) = Project 管理员或租户管理员
//! - INV-LANE-04: 授权是调用时判定 — 撤权后 list/get 立即不可见 (fail-closed)
//! - INV-LANE-05: 乐观锁 lock_version, 并发写冲突返回 Conflict

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::{ProjectId, TenantId, UserId};

crate::define_uuid_id!(LaneId);

// =====================================================================
// 实体
// =====================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Lane 聚合根: Project 之下的管理层级实体, 无远端关联也能存在 (INV-LANE-02)
pub struct Lane {
    /// Lane ID
    pub id: LaneId,
    /// 所属租户 ID (INV-LANE-01)
    pub tenant_id: TenantId,
    /// 所属 Project ID (INV-LANE-01)
    pub project_id: ProjectId,
    /// 展示名称 (Project 内唯一, 1..=128 字符)
    pub name: String,
    /// 描述 (可选)
    pub description: Option<String>,
    /// 创建者用户 ID
    pub created_by: UserId,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 最近更新时间
    pub updated_at: DateTime<Utc>,
    /// 乐观锁版本 (INV-LANE-05)
    pub lock_version: u32,
}

// =====================================================================
// 错误
// =====================================================================

#[derive(Debug, Error)]
/// Lane 领域错误类型
pub enum LaneError {
    #[error("not found: {0}")]
    /// Lane 未找到
    NotFound(String),
    #[error("permission denied")]
    /// 当前 Actor 不是 Project 成员/管理员, 或写操作缺少管理员角色 (INV-LANE-03/04)
    PermissionDenied,
    #[error("cross-tenant access denied: tenant {0} vs required {1}")]
    /// 跨租户访问被拒绝
    CrossTenantDenied(TenantId, TenantId),
    #[error("invalid name: {0}")]
    /// Lane 名称非法 (空/超长/控制字符)
    InvalidName(String),
    #[error("name already exists in project: {0}")]
    /// 同名 Lane 已存在于该 Project
    NameExists(String),
    #[error("conflict: {0}")]
    /// 乐观锁版本冲突 (INV-LANE-05)
    Conflict(String),
    #[error("internal: {0}")]
    /// 内部错误
    Internal(String),
}

// =====================================================================
// 命令 / 查询 DTO
// =====================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 创建 Lane 命令 — 不含任何远端仓库/分支字段 (INV-LANE-02)
pub struct CreateLaneCommand {
    /// 目标租户 ID
    pub tenant_id: TenantId,
    /// 目标 Project ID
    pub project_id: ProjectId,
    /// Lane 名称 (1..=128, 无控制字符)
    pub name: String,
    /// 描述 (可选)
    pub description: Option<String>,
    /// 发起操作的用户 ID
    pub actor_user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 更新 Lane 命令 (改名/改描述, 乐观锁)
pub struct UpdateLaneCommand {
    /// 目标租户 ID
    pub tenant_id: TenantId,
    /// 目标 Project ID
    pub project_id: ProjectId,
    /// 目标 Lane ID
    pub lane_id: LaneId,
    /// 新名称 (None = 不变)
    pub name: Option<String>,
    /// 新描述 (None = 不变)
    pub description: Option<String>,
    /// 期望的当前锁版本 (INV-LANE-05)
    pub expected_version: u32,
    /// 发起操作的用户 ID
    pub actor_user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 删除 Lane 命令
pub struct DeleteLaneCommand {
    /// 目标租户 ID
    pub tenant_id: TenantId,
    /// 目标 Project ID
    pub project_id: ProjectId,
    /// 目标 Lane ID
    pub lane_id: LaneId,
    /// 发起操作的用户 ID
    pub actor_user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 按 Project 列出 Lane 的查询
pub struct ListLanesQuery {
    /// 目标租户 ID
    pub tenant_id: TenantId,
    /// 目标 Project ID
    pub project_id: ProjectId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 查询单个 Lane 的请求
pub struct GetLaneQuery {
    /// 目标租户 ID
    pub tenant_id: TenantId,
    /// 目标 Project ID
    pub project_id: ProjectId,
    /// 目标 Lane ID
    pub lane_id: LaneId,
}

// =====================================================================
// 端口 (Port Traits)
// =====================================================================

#[async_trait]
/// Lane 命令端口 (创建 / 更新 / 删除)
pub trait LaneCommandPort: Send + Sync {
    /// 创建 Lane (不要求远端仓库/分支, INV-LANE-02)
    async fn create_lane(
        &self,
        cmd: CreateLaneCommand,
        actor: &crate::ActorContext,
    ) -> Result<Lane, LaneError>;

    /// 更新 Lane (改名/描述, 乐观锁 INV-LANE-05)
    async fn update_lane(
        &self,
        cmd: UpdateLaneCommand,
        actor: &crate::ActorContext,
    ) -> Result<Lane, LaneError>;

    /// 删除 Lane
    async fn delete_lane(
        &self,
        cmd: DeleteLaneCommand,
        actor: &crate::ActorContext,
    ) -> Result<(), LaneError>;
}

#[async_trait]
/// Lane 查询端口
pub trait LaneQueryPort: Send + Sync {
    /// 按 ID 查询单个 Lane
    async fn get_lane(
        &self,
        q: GetLaneQuery,
        actor: &crate::ActorContext,
    ) -> Result<Lane, LaneError>;

    /// 按 Project 列出 Lane (授权在调用时判定, INV-LANE-04)
    async fn list_by_project(
        &self,
        q: ListLanesQuery,
        actor: &crate::ActorContext,
    ) -> Result<Vec<Lane>, LaneError>;
}

#[async_trait]
/// Lane 仓储端口 (持久化抽象; 服务每次读写都走仓储, 重建服务实例后数据仍在)
pub trait LaneRepository: Send + Sync {
    /// 插入 Lane 记录
    async fn insert_lane(&self, lane: Lane) -> Result<(), LaneError>;
    /// 按 ID 获取 Lane
    async fn get_lane(&self, lane_id: LaneId) -> Result<Lane, LaneError>;
    /// 按 Project 列出 Lane (不判定授权, 授权在服务层)
    async fn list_by_project(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
    ) -> Result<Vec<Lane>, LaneError>;
    /// 更新 Lane 记录 (按 ID upsert)
    async fn update_lane(&self, lane: Lane) -> Result<(), LaneError>;
    /// 删除 Lane 记录
    async fn delete_lane(&self, lane_id: LaneId) -> Result<(), LaneError>;
}

// =====================================================================
// 校验
// =====================================================================

/// 校验 Lane 名称: 非空、去首尾空白后 1..=128 字符、无控制字符、不含 "://"
pub fn validate_lane_name(name: &str) -> Result<String, LaneError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(LaneError::InvalidName("name must not be empty".to_string()));
    }
    if trimmed.len() > 128 {
        return Err(LaneError::InvalidName(
            "name must be at most 128 chars".to_string(),
        ));
    }
    if trimmed.chars().any(char::is_control) || trimmed.contains("://") {
        return Err(LaneError::InvalidName(
            "name contains forbidden characters".to_string(),
        ));
    }
    Ok(trimmed.to_string())
}

/// 校验描述: 可选, 最大 2000 字符, 无控制字符
pub fn validate_lane_description(description: &Option<String>) -> Result<(), LaneError> {
    match description {
        None => Ok(()),
        Some(text) if text.len() <= 2000 && !text.chars().any(char::is_control) => Ok(()),
        Some(_) => Err(LaneError::InvalidName(
            "description must be at most 2000 chars without control characters".to_string(),
        )),
    }
}

// =====================================================================
// InMemoryLaneService — 全部读写走仓储 (持久化 port 层面, 服务重建后数据仍在)
// =====================================================================

/// 基于内存仓储的 Lane 服务实现 (测试/开发; 授权与乐观锁逻辑与生产一致)
pub struct InMemoryLaneService {
    repo: Arc<dyn LaneRepository>,
}

impl InMemoryLaneService {
    /// 使用默认内存仓储构造
    pub fn new() -> Self {
        Self {
            repo: Arc::new(InMemoryLaneRepository::new()),
        }
    }
    /// 使用共享仓储构造 — 重建服务实例共享同一仓储, 验证「持久化 port 层面」数据仍在
    pub fn with_repo(repo: Arc<dyn LaneRepository>) -> Self {
        Self { repo }
    }
    /// 取出底层仓储 (测试隔离/共享用)
    pub fn repo(&self) -> Arc<dyn LaneRepository> {
        self.repo.clone()
    }
}

impl Default for InMemoryLaneService {
    fn default() -> Self {
        Self::new()
    }
}

/// 授权判定: 租户一致 + Project 成员 (INV-LANE-03 读分支)
fn ensure_project_member(
    actor: &crate::ActorContext,
    tenant_id: TenantId,
    project_id: ProjectId,
) -> Result<(), LaneError> {
    if TenantId::from(actor.tenant_id) != tenant_id {
        return Err(LaneError::CrossTenantDenied(
            TenantId::from(actor.tenant_id),
            tenant_id,
        ));
    }
    if !actor.can_access_project(project_id.0) {
        return Err(LaneError::PermissionDenied);
    }
    Ok(())
}

/// 授权判定: 成员 + 管理员角色 (INV-LANE-03 写分支)
fn ensure_project_manager(
    actor: &crate::ActorContext,
    tenant_id: TenantId,
    project_id: ProjectId,
) -> Result<(), LaneError> {
    ensure_project_member(actor, tenant_id, project_id)?;
    if !actor.has_role("project_admin") && !actor.has_role("tenant_admin") {
        return Err(LaneError::PermissionDenied);
    }
    Ok(())
}

/// 同 Project 内重名校验
async fn ensure_name_unique(
    repo: &dyn LaneRepository,
    tenant_id: TenantId,
    project_id: ProjectId,
    name: &str,
    exclude: Option<LaneId>,
) -> Result<(), LaneError> {
    let existing = repo.list_by_project(tenant_id, project_id).await?;
    if existing
        .iter()
        .any(|lane| lane.name == name && exclude.map_or(true, |id| lane.id != id))
    {
        return Err(LaneError::NameExists(name.to_string()));
    }
    Ok(())
}

#[async_trait]
impl LaneCommandPort for InMemoryLaneService {
    async fn create_lane(
        &self,
        cmd: CreateLaneCommand,
        actor: &crate::ActorContext,
    ) -> Result<Lane, LaneError> {
        ensure_project_manager(actor, cmd.tenant_id, cmd.project_id)?;
        let name = validate_lane_name(&cmd.name)?;
        validate_lane_description(&cmd.description)?;
        ensure_name_unique(
            self.repo.as_ref(),
            cmd.tenant_id,
            cmd.project_id,
            &name,
            None,
        )
        .await?;
        let now = Utc::now();
        let lane = Lane {
            id: LaneId::new(),
            tenant_id: cmd.tenant_id,
            project_id: cmd.project_id,
            name,
            description: cmd.description,
            created_by: cmd.actor_user_id,
            created_at: now,
            updated_at: now,
            lock_version: 1,
        };
        self.repo.insert_lane(lane.clone()).await?;
        Ok(lane)
    }

    async fn update_lane(
        &self,
        cmd: UpdateLaneCommand,
        actor: &crate::ActorContext,
    ) -> Result<Lane, LaneError> {
        ensure_project_manager(actor, cmd.tenant_id, cmd.project_id)?;
        if cmd.expected_version < 1 {
            return Err(LaneError::Conflict(format!(
                "expected_version {} must be >= 1",
                cmd.expected_version
            )));
        }
        let mut lane = self.repo.get_lane(cmd.lane_id).await?;
        if lane.tenant_id != cmd.tenant_id {
            return Err(LaneError::CrossTenantDenied(lane.tenant_id, cmd.tenant_id));
        }
        if lane.project_id != cmd.project_id {
            return Err(LaneError::NotFound(format!(
                "lane:{} in project:{}",
                cmd.lane_id.as_uuid(),
                cmd.project_id.as_uuid()
            )));
        }
        if lane.lock_version != cmd.expected_version {
            return Err(LaneError::Conflict(format!(
                "lane:{} version {} != expected {}",
                cmd.lane_id.as_uuid(),
                lane.lock_version,
                cmd.expected_version
            )));
        }
        if let Some(raw_name) = &cmd.name {
            lane.name = validate_lane_name(raw_name)?;
        }
        validate_lane_description(&cmd.description)?;
        if cmd.description.is_some() {
            lane.description = cmd.description;
        }
        ensure_name_unique(
            self.repo.as_ref(),
            cmd.tenant_id,
            cmd.project_id,
            &lane.name,
            Some(lane.id),
        )
        .await?;
        lane.lock_version += 1;
        lane.updated_at = Utc::now();
        self.repo.update_lane(lane.clone()).await?;
        Ok(lane)
    }

    async fn delete_lane(
        &self,
        cmd: DeleteLaneCommand,
        actor: &crate::ActorContext,
    ) -> Result<(), LaneError> {
        ensure_project_manager(actor, cmd.tenant_id, cmd.project_id)?;
        let lane = self.repo.get_lane(cmd.lane_id).await?;
        if lane.tenant_id != cmd.tenant_id {
            return Err(LaneError::CrossTenantDenied(lane.tenant_id, cmd.tenant_id));
        }
        if lane.project_id != cmd.project_id {
            return Err(LaneError::NotFound(format!(
                "lane:{} in project:{}",
                cmd.lane_id.as_uuid(),
                cmd.project_id.as_uuid()
            )));
        }
        self.repo.delete_lane(cmd.lane_id).await
    }
}

#[async_trait]
impl LaneQueryPort for InMemoryLaneService {
    async fn get_lane(
        &self,
        q: GetLaneQuery,
        actor: &crate::ActorContext,
    ) -> Result<Lane, LaneError> {
        ensure_project_member(actor, q.tenant_id, q.project_id)?;
        let lane = self.repo.get_lane(q.lane_id).await?;
        if lane.tenant_id != q.tenant_id {
            return Err(LaneError::CrossTenantDenied(lane.tenant_id, q.tenant_id));
        }
        if lane.project_id != q.project_id {
            return Err(LaneError::NotFound(format!(
                "lane:{} in project:{}",
                q.lane_id.as_uuid(),
                q.project_id.as_uuid()
            )));
        }
        Ok(lane)
    }

    async fn list_by_project(
        &self,
        q: ListLanesQuery,
        actor: &crate::ActorContext,
    ) -> Result<Vec<Lane>, LaneError> {
        // INV-LANE-04: 授权在调用时判定 — 撤权后的 actor 直接 fail-closed,
        // 不会看到任何 Lane (REST 层映射为 403, 旧 Lane 数据不可见)。
        ensure_project_member(actor, q.tenant_id, q.project_id)?;
        Ok(self.repo.list_by_project(q.tenant_id, q.project_id).await?)
    }
}

// =====================================================================
// InMemoryLaneRepository
// =====================================================================

/// 基于内存 HashMap 的 LaneRepository 实现
pub struct InMemoryLaneRepository {
    lanes: RwLock<HashMap<LaneId, Lane>>,
}

impl InMemoryLaneRepository {
    /// 构造空仓储
    pub fn new() -> Self {
        Self {
            lanes: RwLock::new(HashMap::new()),
        }
    }
    /// 测试隔离: 清空全部记录
    pub fn reset(&self) {
        self.lanes.write().unwrap().clear();
    }
}

impl Default for InMemoryLaneRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LaneRepository for InMemoryLaneRepository {
    async fn insert_lane(&self, lane: Lane) -> Result<(), LaneError> {
        let mut store = self.lanes.write().expect("lock");
        if store.contains_key(&lane.id) {
            return Err(LaneError::Conflict(format!(
                "lane:{} already exists",
                lane.id.as_uuid()
            )));
        }
        store.insert(lane.id, lane);
        Ok(())
    }
    async fn get_lane(&self, lane_id: LaneId) -> Result<Lane, LaneError> {
        self.lanes
            .read()
            .expect("lock")
            .get(&lane_id)
            .cloned()
            .ok_or_else(|| LaneError::NotFound(format!("lane:{}", lane_id.as_uuid())))
    }
    async fn list_by_project(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
    ) -> Result<Vec<Lane>, LaneError> {
        Ok(self
            .lanes
            .read()
            .expect("lock")
            .values()
            .filter(|lane| lane.tenant_id == tenant_id && lane.project_id == project_id)
            .cloned()
            .collect())
    }
    async fn update_lane(&self, lane: Lane) -> Result<(), LaneError> {
        self.lanes.write().expect("lock").insert(lane.id, lane);
        Ok(())
    }
    async fn delete_lane(&self, lane_id: LaneId) -> Result<(), LaneError> {
        self.lanes.write().expect("lock").remove(&lane_id);
        Ok(())
    }
}

// =====================================================================
// 单元测试 (P0 验收: 无远端创建 / 授权 / 持久化 port / 撤权不可见)
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn member(tid: Uuid, project_id: ProjectId) -> crate::ActorContext {
        crate::ActorContext::new(Uuid::new_v4(), tid).with_project(project_id.0)
    }
    fn admin(tid: Uuid, project_id: ProjectId) -> crate::ActorContext {
        member(tid, project_id).with_role("project_admin")
    }
    fn create_cmd(tid: Uuid, project_id: ProjectId) -> CreateLaneCommand {
        CreateLaneCommand {
            tenant_id: TenantId(tid),
            project_id,
            name: "core-runtime".to_string(),
            description: None,
            actor_user_id: UserId::new(),
        }
    }

    #[tokio::test]
    async fn create_lane_without_remote_repo_or_branch() {
        // 验收第一条: 没有远端仓库/分支关联时也能创建 Lane。
        // CreateLaneCommand 根本不含仓库/分支字段 (INV-LANE-02)。
        let svc = InMemoryLaneService::new();
        let tid = Uuid::new_v4();
        let project = ProjectId::new();
        let actor = admin(tid, project);
        let lane = svc
            .create_lane(create_cmd(tid, project), &actor)
            .await
            .unwrap();
        assert_eq!(lane.name, "core-runtime");
        assert_eq!(lane.project_id, project);
        assert_eq!(lane.lock_version, 1);
        assert!(!lane.id.as_uuid().is_nil());
    }

    #[tokio::test]
    async fn create_lane_requires_manager_role() {
        let svc = InMemoryLaneService::new();
        let tid = Uuid::new_v4();
        let project = ProjectId::new();
        // 普通成员 (无 project_admin) 创建 → 拒绝
        let res = svc
            .create_lane(create_cmd(tid, project), &member(tid, project))
            .await;
        assert!(matches!(res, Err(LaneError::PermissionDenied)));
        // 非成员 → 拒绝
        let outsider = crate::ActorContext::new(Uuid::new_v4(), tid);
        let res = svc.create_lane(create_cmd(tid, project), &outsider).await;
        assert!(matches!(res, Err(LaneError::PermissionDenied)));
    }

    #[tokio::test]
    async fn create_lane_cross_tenant_denied() {
        let svc = InMemoryLaneService::new();
        let t1 = Uuid::new_v4();
        let t2 = Uuid::new_v4();
        let project = ProjectId::new();
        let actor = admin(t2, project);
        let res = svc.create_lane(create_cmd(t1, project), &actor).await;
        assert!(matches!(res, Err(LaneError::CrossTenantDenied(_, _))));
    }

    #[tokio::test]
    async fn create_lane_rejects_invalid_name_and_duplicate() {
        let svc = InMemoryLaneService::new();
        let tid = Uuid::new_v4();
        let project = ProjectId::new();
        let actor = admin(tid, project);
        let mut cmd = create_cmd(tid, project);
        cmd.name = "   ".to_string();
        assert!(matches!(
            svc.create_lane(cmd.clone(), &actor).await,
            Err(LaneError::InvalidName(_))
        ));
        cmd.name = "core-runtime".to_string();
        svc.create_lane(cmd.clone(), &actor).await.unwrap();
        assert!(matches!(
            svc.create_lane(cmd, &actor).await,
            Err(LaneError::NameExists(_))
        ));
    }

    #[tokio::test]
    async fn lane_persists_across_service_rebuild() {
        // 持久化 port 层面: 重建服务实例 (共享同一仓储) 后数据仍在。
        let repo: Arc<dyn LaneRepository> = Arc::new(InMemoryLaneRepository::new());
        let tid = Uuid::new_v4();
        let project = ProjectId::new();
        let actor = admin(tid, project);
        let first = InMemoryLaneService::with_repo(repo.clone());
        let lane = first
            .create_lane(create_cmd(tid, project), &actor)
            .await
            .unwrap();
        // 重建服务 (新实例, 新本地状态, 同一持久化 port)
        let second = InMemoryLaneService::with_repo(repo.clone());
        let listed = second
            .list_by_project(
                ListLanesQuery {
                    tenant_id: TenantId(tid),
                    project_id: project,
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, lane.id);
        assert_eq!(listed[0].name, "core-runtime");
        let fetched = second
            .get_lane(
                GetLaneQuery {
                    tenant_id: TenantId(tid),
                    project_id: project,
                    lane_id: lane.id,
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(fetched.lock_version, 1);
    }

    #[tokio::test]
    async fn revoked_member_cannot_see_lane() {
        // 撤权 = 调用时授权判定失败 (INV-LANE-04): 同一数据对撤权 actor 不可见。
        let repo: Arc<dyn LaneRepository> = Arc::new(InMemoryLaneRepository::new());
        let tid = Uuid::new_v4();
        let project = ProjectId::new();
        let manager = admin(tid, project);
        let svc = InMemoryLaneService::with_repo(repo.clone());
        let lane = svc
            .create_lane(create_cmd(tid, project), &manager)
            .await
            .unwrap();
        // 授权成员可见
        let authorized = member(tid, project);
        let list = svc
            .list_by_project(
                ListLanesQuery {
                    tenant_id: TenantId(tid),
                    project_id: project,
                },
                &authorized,
            )
            .await
            .unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, lane.id);
        // 撤权: project_ids 不再包含该项目 → fail-closed, 列表不可见 (REST 映射 403)
        let revoked = crate::ActorContext::new(Uuid::new_v4(), tid);
        let res = svc
            .list_by_project(
                ListLanesQuery {
                    tenant_id: TenantId(tid),
                    project_id: project,
                },
                &revoked,
            )
            .await;
        assert!(matches!(res, Err(LaneError::PermissionDenied)));
        let res = svc
            .get_lane(
                GetLaneQuery {
                    tenant_id: TenantId(tid),
                    project_id: project,
                    lane_id: lane.id,
                },
                &revoked,
            )
            .await;
        assert!(matches!(res, Err(LaneError::PermissionDenied)));
    }

    #[tokio::test]
    async fn update_lane_with_version_conflict() {
        let svc = InMemoryLaneService::new();
        let tid = Uuid::new_v4();
        let project = ProjectId::new();
        let actor = admin(tid, project);
        let lane = svc
            .create_lane(create_cmd(tid, project), &actor)
            .await
            .unwrap();
        // 错误版本 → 冲突
        let res = svc
            .update_lane(
                UpdateLaneCommand {
                    tenant_id: TenantId(tid),
                    project_id: project,
                    lane_id: lane.id,
                    name: Some("renamed".to_string()),
                    description: None,
                    expected_version: 2,
                    actor_user_id: UserId::new(),
                },
                &actor,
            )
            .await;
        assert!(matches!(res, Err(LaneError::Conflict(_))));
        // 正确版本 → 成功, lock_version +1
        let updated = svc
            .update_lane(
                UpdateLaneCommand {
                    tenant_id: TenantId(tid),
                    project_id: project,
                    lane_id: lane.id,
                    name: Some("renamed".to_string()),
                    description: Some("lane 描述".to_string()),
                    expected_version: 1,
                    actor_user_id: UserId::new(),
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(updated.name, "renamed");
        assert_eq!(updated.lock_version, 2);
        assert_eq!(updated.description.as_deref(), Some("lane 描述"));
    }

    #[tokio::test]
    async fn delete_lane_removes_it() {
        let svc = InMemoryLaneService::new();
        let tid = Uuid::new_v4();
        let project = ProjectId::new();
        let actor = admin(tid, project);
        let lane = svc
            .create_lane(create_cmd(tid, project), &actor)
            .await
            .unwrap();
        svc.delete_lane(
            DeleteLaneCommand {
                tenant_id: TenantId(tid),
                project_id: project,
                lane_id: lane.id,
                actor_user_id: UserId::new(),
            },
            &actor,
        )
        .await
        .unwrap();
        let res = svc
            .get_lane(
                GetLaneQuery {
                    tenant_id: TenantId(tid),
                    project_id: project,
                    lane_id: lane.id,
                },
                &actor,
            )
            .await;
        assert!(matches!(res, Err(LaneError::NotFound(_))));
    }

    #[tokio::test]
    async fn list_by_project_scopes_to_project() {
        let svc = InMemoryLaneService::new();
        let tid = Uuid::new_v4();
        let p1 = ProjectId::new();
        let p2 = ProjectId::new();
        let actor = crate::ActorContext::new(Uuid::new_v4(), tid)
            .with_project(p1.0)
            .with_project(p2.0)
            .with_role("project_admin");
        svc.create_lane(create_cmd(tid, p1), &actor).await.unwrap();
        svc.create_lane(create_cmd(tid, p2), &actor).await.unwrap();
        let list = svc
            .list_by_project(
                ListLanesQuery {
                    tenant_id: TenantId(tid),
                    project_id: p1,
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(list.len(), 1);
        assert!(list.iter().all(|lane| lane.project_id == p1));
    }
}
