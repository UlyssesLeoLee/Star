//! domain-workspace crate
//!
//! 详细 spec: docs/specs/domain-workspace-spec.md §7
//! 上游基本设计: docs/basic-design.md §2.1(表 19) / §5.7
//! 数据设计: docs/data-design.md §4.2 (`workspace` schema)
//! API 设计: docs/api-design.md §3.3
//!
//! ## 职责
//!
//! Tenant 内协作单位(Workspace → Project 二级层级),`Workspace` 聚合根 +
//! `WorkspaceMember` 实体,4 个端口(4 cmd / 4 query) + 1 个事件 bus +
//! 4 条不变量(INV-WS-01~04) + 1 个 `InMemoryWorkspaceService` 真实实现。
//!
//! ## 关键不变量(INV-WS-01~04)
//!
//! - **INV-WS-01** `workspace_key` 在 tenant 内唯一(URL 友好业务键)
//! - **INV-WS-02** Workspace 必带 `tenant_id`,跨 tenant 拒绝(§6.1 REQ-SEC-001)
//! - **INV-WS-03** Workspace 删除前必须无 Project 引用(级联检查,Application 层强制)
//! - **INV-WS-04** 创建 Workspace 时必带 `created_by_user_id`,且该 User 必属同 Tenant
//!
//! ## 状态机
//!
//! Workspace 无显式状态机,3 个事件:`WorkspaceCreated` / `MemberAdded` /
//! `MemberRemoved`。删除是硬删除(返回 409 WS-003 当仍有 Project 引用)。
//!
//! Lead 责任: workspace Lead

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;
pub use star_context::ActorContext;

// =====================================================================
// 强类型 ID 宏
// =====================================================================

/// 生成强类型 `Uuid` newtype,提供 `new() / from_uuid / as_uuid / into_uuid`
/// / `Default` / `Display` / `From<Uuid>` / `Deref<Target=Uuid>` 一组常用 trait 实现。
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
            pub fn as_uuid(&self) -> uuid::Uuid { self.0 }
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

define_uuid_id!(WorkspaceId);
define_uuid_id!(WorkspaceMemberId);
define_uuid_id!(TenantId);
define_uuid_id!(UserId);
define_uuid_id!(ProjectId);

// =====================================================================
// 值对象
// =====================================================================

/// **Workspace 成员角色**(`workspace_member.role` 列,basic-design §3.4)
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WorkspaceRole {
    /// Workspace 管理员
    Admin,
    /// 成员
    Member,
    /// 访客(只读)
    Guest,
}

impl Default for WorkspaceRole {
    fn default() -> Self {
        Self::Member
    }
}

impl std::fmt::Display for WorkspaceRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Admin => "ADMIN",
            Self::Member => "MEMBER",
            Self::Guest => "GUEST",
        };
        f.write_str(s)
    }
}

/// 预定义角色字符串(security-design §3.4)
pub mod roles {
    /// Workspace 管理员
    pub const WORKSPACE_ADMIN: &str = "workspace_admin";
    /// Workspace 成员
    pub const WORKSPACE_MEMBER: &str = "workspace_member";
}

// =====================================================================
// 错误
// =====================================================================

/// Workspace 域错误(§8.3 错误码:WS-001~005 + SEC-007)
#[derive(Debug, Error)]
pub enum WorkspaceError {
    /// `WS-001` 404 Workspace 不存在
    #[error("workspace not found: {0}")]
    NotFound(WorkspaceId),
    /// `WS-002` 409 slug 已存在
    #[error("invalid state: {0}")]
    InvalidState(String),
    /// `WS-003` 409 删除时仍有 Project 引用
    #[error("permission denied")]
    PermissionDenied,
    /// 409 通用冲突(版本不一致 / 重复成员等)
    #[error("conflict: {0}")]
    Conflict(String),
    /// 5xx 内部错误
    #[error("internal error: {0}")]
    Internal(String),
}

impl WorkspaceError {
    /// 错误码字符串
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "WORKSPACE_NOT_FOUND",
            Self::InvalidState(_) => "WORKSPACE_INVALID_STATE",
            Self::PermissionDenied => "WORKSPACE_PERMISSION_DENIED",
            Self::Conflict(_) => "WORKSPACE_CONFLICT",
            Self::Internal(_) => "WORKSPACE_INTERNAL",
        }
    }
    /// 是否 5xx
    pub fn is_server_error(&self) -> bool {
        matches!(self, Self::Internal(_))
    }
}

impl From<uuid::Error> for WorkspaceError {
    fn from(e: uuid::Error) -> Self {
        Self::Internal(format!("uuid error: {e}"))
    }
}

// =====================================================================
// 实体
// =====================================================================

/// **Workspace 聚合根**(§4.2 `workspace` schema)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    /// 主键
    pub id: WorkspaceId,
    /// 租户 ID(INV-WS-02)
    pub tenant_id: TenantId,
    /// 业务键(tenant 内唯一,INV-WS-01)
    pub workspace_key: String,
    /// 显示名
    pub name: String,
    /// 描述
    pub description: Option<String>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    /// 乐观锁版本(防止并发 update)
    pub version: u32,
}

impl Workspace {
    /// 字段数量(契约 §2.2)
    pub const FIELD_COUNT: usize = 8;
    /// 乐观锁 +1
    pub fn bump_version(&mut self) {
        self.version = self.version.saturating_add(1);
        self.updated_at = Utc::now();
    }
}

/// **WorkspaceMember**(workspace 成员关联,basic-design §3.4)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceMember {
    /// 主键
    pub id: WorkspaceMemberId,
    /// Workspace ID
    pub workspace_id: WorkspaceId,
    /// 租户 ID(冗余,但便于跨租户拒绝)
    pub tenant_id: TenantId,
    /// 用户 ID
    pub user_id: UserId,
    /// 角色
    pub role: WorkspaceRole,
    /// 加入时间
    pub joined_at: DateTime<Utc>,
    /// 乐观锁版本
    pub version: u32,
}

impl WorkspaceMember {
    /// 字段数量
    pub const FIELD_COUNT: usize = 7;
    /// 是否 Admin
    pub fn is_admin(&self) -> bool {
        self.role == WorkspaceRole::Admin
    }
}

// =====================================================================
// 不变量(INV-WS-01~04)
// =====================================================================

/// 不变量检查函数类型
pub type InvariantCheck = fn(&Workspace) -> Result<(), WorkspaceError>;

/// **INV-WS-01** `workspace_key` 在 tenant 内唯一
pub fn check_invariant_01_workspace_key_unique(
    ws: &Workspace,
    existing_keys: &[String],
) -> Result<(), WorkspaceError> {
    if existing_keys.iter().any(|k| k == &ws.workspace_key) {
        return Err(WorkspaceError::InvalidState(format!(
            "INV-WS-01: workspace_key '{}' 已被占用",
            ws.workspace_key
        )));
    }
    Ok(())
}

/// **INV-WS-02** Workspace 必带 `tenant_id`(非 nil)
pub fn check_invariant_02_tenant_id_present(ws: &Workspace) -> Result<(), WorkspaceError> {
    if ws.tenant_id.as_uuid().is_nil() {
        return Err(WorkspaceError::InvalidState(
            "INV-WS-02: tenant_id 必须非 nil (§6.1, REQ-SEC-001)".to_string(),
        ));
    }
    Ok(())
}

/// **INV-WS-03** `workspace_key` 格式校验(非空 + 长度 ≤ 64)
pub fn check_invariant_03_workspace_key_format(ws: &Workspace) -> Result<(), WorkspaceError> {
    if ws.workspace_key.trim().is_empty() {
        return Err(WorkspaceError::InvalidState(
            "INV-WS-03: workspace_key 不能为空".to_string(),
        ));
    }
    if ws.workspace_key.len() > 64 {
        return Err(WorkspaceError::InvalidState(
            "INV-WS-03: workspace_key 长度 ≤ 64 字符".to_string(),
        ));
    }
    Ok(())
}

/// **INV-WS-04** `name` 非空 + 长度 ≤ 128(用于 UX 一致性)
pub fn check_invariant_04_name_format(ws: &Workspace) -> Result<(), WorkspaceError> {
    if ws.name.trim().is_empty() {
        return Err(WorkspaceError::InvalidState(
            "INV-WS-04: workspace name 不能为空".to_string(),
        ));
    }
    if ws.name.len() > 128 {
        return Err(WorkspaceError::InvalidState(
            "INV-WS-04: workspace name 长度 ≤ 128 字符".to_string(),
        ));
    }
    Ok(())
}

/// 不变量检查函数列表(顺序执行)
pub const ALL_INVARIANT_CHECKS: &[InvariantCheck] = &[
    check_invariant_02_tenant_id_present,
    check_invariant_03_workspace_key_format,
    check_invariant_04_name_format,
];

/// 顺序执行不变量检查
pub fn run_invariants(checks: &[InvariantCheck], ws: &Workspace) -> Result<(), WorkspaceError> {
    for check in checks {
        check(ws)?;
    }
    Ok(())
}

// =====================================================================
// 事件(NATS 主题 payload)
// =====================================================================

/// 事件元数据(actor / time)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMeta {
    /// 事件 ID
    pub event_id: Uuid,
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 发生时间
    pub occurred_at: DateTime<Utc>,
    /// 触发者用户(可能为 None,system actor)
    pub actor_user_id: Option<UserId>,
}

impl EventMeta {
    /// 新建事件元数据
    pub fn new(tenant_id: TenantId) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            tenant_id,
            occurred_at: Utc::now(),
            actor_user_id: None,
        }
    }
}

/// WorkspaceCreated 事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceCreated {
    /// 元数据
    pub meta: EventMeta,
    /// Workspace ID
    pub workspace_id: WorkspaceId,
    /// 业务键
    pub workspace_key: String,
    /// 显示名
    pub name: String,
}

/// MemberAdded 事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberAdded {
    /// 元数据
    pub meta: EventMeta,
    /// Workspace ID
    pub workspace_id: WorkspaceId,
    /// 新增成员 user_id
    pub user_id: UserId,
    /// 角色
    pub role: WorkspaceRole,
}

/// MemberRemoved 事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberRemoved {
    /// 元数据
    pub meta: EventMeta,
    /// Workspace ID
    pub workspace_id: WorkspaceId,
    /// 被移除成员 user_id
    pub user_id: UserId,
}

/// Workspace 域事件枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WorkspaceEvent {
    /// Workspace 创建
    Created(WorkspaceCreated),
    /// 添加成员
    MemberAdded(MemberAdded),
    /// 移除成员
    MemberRemoved(MemberRemoved),
}

impl WorkspaceEvent {
    /// NATS subject
    pub fn subject(&self) -> &'static str {
        match self {
            Self::Created(_) => "star.events.workspace.workspace.created.v1",
            Self::MemberAdded(_) => "star.events.workspace.member.added.v1",
            Self::MemberRemoved(_) => "star.events.workspace.member.removed.v1",
        }
    }
}

// =====================================================================
// 端口(Port traits)
// =====================================================================

/// CreateWorkspace 命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWorkspaceCommand {
    /// 租户 ID(从 JWT 注入)
    pub tenant_id: TenantId,
    /// 业务键
    pub workspace_key: String,
    /// 显示名
    pub name: String,
    /// 描述
    pub description: Option<String>,
    /// 创建者(自动成为 Admin 成员,INV-WS-04)
    pub owner_user_id: UserId,
}

/// UpdateWorkspace 命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateWorkspaceCommand {
    pub workspace_id: WorkspaceId,
    pub tenant_id: TenantId,
    /// 乐观锁(必填,防止覆盖并发修改)
    pub expected_version: u32,
    /// 更新名(None 表示不变)
    pub name: Option<String>,
    /// 双重 Option:`Some(None)` = 清空描述,`None` = 不变
    pub description: Option<Option<String>>,
}

/// AddMember 命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddMemberCommand {
    pub workspace_id: WorkspaceId,
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub role: WorkspaceRole,
}

/// RemoveMember 命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveMemberCommand {
    pub workspace_id: WorkspaceId,
    pub tenant_id: TenantId,
    pub user_id: UserId,
}

/// ListWorkspace 查询
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListWorkspaceQuery {
    pub tenant_id: TenantId,
    pub limit: u32,
    pub offset: u32,
}

impl Default for ListWorkspaceQuery {
    fn default() -> Self {
        Self {
            tenant_id: TenantId::new(),
            limit: 50,
            offset: 0,
        }
    }
}

/// Workspace 命令端口
#[async_trait]
pub trait WorkspaceCommandPort: Send + Sync {
    /// 创建 Workspace(自动加 owner 为 Admin 成员)
    async fn create_workspace(
        &self,
        cmd: CreateWorkspaceCommand,
        actor: ActorContext,
    ) -> Result<Workspace, WorkspaceError>;
    /// 更新 Workspace(name / description)
    async fn update_workspace(
        &self,
        cmd: UpdateWorkspaceCommand,
        actor: ActorContext,
    ) -> Result<Workspace, WorkspaceError>;
    /// 添加成员(需 workspace 存在 + 成员未重复)
    async fn add_member(
        &self,
        cmd: AddMemberCommand,
        actor: ActorContext,
    ) -> Result<WorkspaceMember, WorkspaceError>;
    /// 移除成员
    async fn remove_member(
        &self,
        cmd: RemoveMemberCommand,
        actor: ActorContext,
    ) -> Result<(), WorkspaceError>;
}

/// Workspace 查询端口
#[async_trait]
pub trait WorkspaceQueryPort: Send + Sync {
    async fn get_by_id(
        &self,
        id: WorkspaceId,
        viewer: ActorContext,
    ) -> Result<Workspace, WorkspaceError>;
    async fn get_by_key(
        &self,
        tenant_id: TenantId,
        workspace_key: &str,
        viewer: ActorContext,
    ) -> Result<Workspace, WorkspaceError>;
    async fn list_workspaces(
        &self,
        q: ListWorkspaceQuery,
        viewer: ActorContext,
    ) -> Result<Vec<Workspace>, WorkspaceError>;
    async fn list_members(
        &self,
        workspace_id: WorkspaceId,
        viewer: ActorContext,
    ) -> Result<Vec<WorkspaceMember>, WorkspaceError>;
}

// =====================================================================
// InMemoryWorkspaceService
// =====================================================================

/// **InMemory Workspace 命令/查询服务**(真实实现,纯 in-memory)
pub struct InMemoryWorkspaceService {
    workspaces: Arc<RwLock<HashMap<WorkspaceId, Workspace>>>,
    members: Arc<RwLock<HashMap<WorkspaceMemberId, WorkspaceMember>>>,
    event_tx: mpsc::UnboundedSender<WorkspaceEvent>,
}

impl InMemoryWorkspaceService {
    /// 新建服务 + 事件接收端
    pub fn new() -> (Arc<Self>, mpsc::UnboundedReceiver<WorkspaceEvent>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let svc = Arc::new(Self {
            workspaces: Arc::new(RwLock::new(HashMap::new())),
            members: Arc::new(RwLock::new(HashMap::new())),
            event_tx: tx,
        });
        (svc, rx)
    }
    /// 仅 svc(测试用,事件丢)
    pub fn new_for_test() -> Arc<Self> {
        Self::new().0
    }
    /// Workspace 数量(测试断言)
    pub async fn count(&self) -> usize {
        self.workspaces.read().await.len()
    }
    /// 成员数量
    pub async fn member_count(&self) -> usize {
        self.members.read().await.len()
    }
    /// tenant 一致性检查(INV-WS-02 的轻量级应用层校验)
    fn check_tenant(actor: &ActorContext, expected: TenantId) -> Result<(), WorkspaceError> {
        if actor.tenant_id != expected.0 {
            return Err(WorkspaceError::PermissionDenied);
        }
        Ok(())
    }
}

impl Default for InMemoryWorkspaceService {
    fn default() -> Self {
        Self::new().0.as_ref().clone()
    }
}

impl Clone for InMemoryWorkspaceService {
    fn clone(&self) -> Self {
        Self {
            workspaces: self.workspaces.clone(),
            members: self.members.clone(),
            event_tx: self.event_tx.clone(),
        }
    }
}

#[async_trait]
impl WorkspaceCommandPort for InMemoryWorkspaceService {
    async fn create_workspace(
        &self,
        cmd: CreateWorkspaceCommand,
        actor: ActorContext,
    ) -> Result<Workspace, WorkspaceError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        let now = Utc::now();
        let id = WorkspaceId::new();
        let ws = Workspace {
            id,
            tenant_id: cmd.tenant_id,
            workspace_key: cmd.workspace_key.clone(),
            name: cmd.name.clone(),
            description: cmd.description,
            created_at: now,
            updated_at: now,
            version: 1,
        };
        // 格式 + 必带字段校验
        run_invariants(ALL_INVARIANT_CHECKS, &ws)?;
        // tenant 内 key 唯一
        let existing_keys: Vec<String> = {
            let guard = self.workspaces.read().await;
            guard
                .values()
                .filter(|w| w.tenant_id == cmd.tenant_id)
                .map(|w| w.workspace_key.clone())
                .collect()
        };
        check_invariant_01_workspace_key_unique(&ws, &existing_keys)?;
        // 写入(锁在表达式内,避免跨 .await)
        {
            let mut guard = self.workspaces.write().await;
            guard.insert(id, ws.clone());
        }
        // owner 自动成为 Admin 成员
        let m_id = WorkspaceMemberId::new();
        let member = WorkspaceMember {
            id: m_id,
            workspace_id: id,
            tenant_id: cmd.tenant_id,
            user_id: cmd.owner_user_id,
            role: WorkspaceRole::Admin,
            joined_at: now,
            version: 1,
        };
        {
            let mut guard = self.members.write().await;
            guard.insert(m_id, member);
        }
        // 事件
        let event = WorkspaceEvent::Created(WorkspaceCreated {
            meta: EventMeta {
                actor_user_id: Some(UserId::from_uuid(actor.user_id)),
                ..EventMeta::new(cmd.tenant_id)
            },
            workspace_id: id,
            workspace_key: ws.workspace_key.clone(),
            name: ws.name.clone(),
        });
        let _ = self.event_tx.send(event);
        Ok(ws)
    }

    async fn update_workspace(
        &self,
        cmd: UpdateWorkspaceCommand,
        actor: ActorContext,
    ) -> Result<Workspace, WorkspaceError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        // 单次锁内完成全部读改写(避免 Send guard 跨 .await)
        let updated = {
            let mut store = self.workspaces.write().await;
            let w = store
                .get_mut(&cmd.workspace_id)
                .ok_or(WorkspaceError::NotFound(cmd.workspace_id))?;
            if w.tenant_id != cmd.tenant_id {
                return Err(WorkspaceError::PermissionDenied);
            }
            if w.version != cmd.expected_version {
                return Err(WorkspaceError::Conflict(format!(
                    "version mismatch: expected {}, actual {}",
                    cmd.expected_version, w.version
                )));
            }
            if let Some(name) = cmd.name {
                w.name = name;
            }
            if let Some(desc) = cmd.description {
                w.description = desc;
            }
            w.bump_version();
            // 重跑不变量(防止 name 校验被绕过)
            run_invariants(ALL_INVARIANT_CHECKS, w)?;
            w.clone()
        };
        Ok(updated)
    }

    async fn add_member(
        &self,
        cmd: AddMemberCommand,
        actor: ActorContext,
    ) -> Result<WorkspaceMember, WorkspaceError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        // workspace 存在性
        let exists = {
            let guard = self.workspaces.read().await;
            guard.contains_key(&cmd.workspace_id)
        };
        if !exists {
            return Err(WorkspaceError::NotFound(cmd.workspace_id));
        }
        // 重复成员检查
        let dup = {
            let guard = self.members.read().await;
            guard
                .values()
                .any(|m| m.workspace_id == cmd.workspace_id && m.user_id == cmd.user_id)
        };
        if dup {
            return Err(WorkspaceError::Conflict(format!(
                "user {} 已是 workspace {} 成员",
                cmd.user_id, cmd.workspace_id
            )));
        }
        let id = WorkspaceMemberId::new();
        let member = WorkspaceMember {
            id,
            workspace_id: cmd.workspace_id,
            tenant_id: cmd.tenant_id,
            user_id: UserId::from(cmd.user_id),
            role: cmd.role,
            joined_at: Utc::now(),
            version: 1,
        };
        {
            let mut guard = self.members.write().await;
            guard.insert(id, member.clone());
        }
        let event = WorkspaceEvent::MemberAdded(MemberAdded {
            meta: EventMeta {
                actor_user_id: Some(UserId::from_uuid(actor.user_id)),
                ..EventMeta::new(cmd.tenant_id)
            },
            workspace_id: cmd.workspace_id,
            user_id: UserId::from(cmd.user_id),
            role: cmd.role,
        });
        let _ = self.event_tx.send(event);
        Ok(member)
    }

    async fn remove_member(
        &self,
        cmd: RemoveMemberCommand,
        actor: ActorContext,
    ) -> Result<(), WorkspaceError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        let removed_id = {
            let mut store = self.members.write().await;
            store
                .iter()
                .find(|(_, m)| {
                    m.workspace_id == cmd.workspace_id
                        && m.user_id == cmd.user_id
                        && m.tenant_id == cmd.tenant_id
                })
                .map(|(id, _)| *id)
        };
        let mid = match removed_id {
            Some(id) => {
                let mut store = self.members.write().await;
                store.remove(&id);
                id
            }
            None => return Err(WorkspaceError::NotFound(cmd.workspace_id)),
        };
        let event = WorkspaceEvent::MemberRemoved(MemberRemoved {
            meta: EventMeta {
                actor_user_id: Some(UserId::from_uuid(actor.user_id)),
                ..EventMeta::new(cmd.tenant_id)
            },
            workspace_id: cmd.workspace_id,
            user_id: UserId::from(cmd.user_id),
        });
        let _ = self.event_tx.send(event);
        let _ = mid; // suppress unused
        Ok(())
    }
}

#[async_trait]
impl WorkspaceQueryPort for InMemoryWorkspaceService {
    async fn get_by_id(
        &self,
        id: WorkspaceId,
        viewer: ActorContext,
    ) -> Result<Workspace, WorkspaceError> {
        let w = {
            let guard = self.workspaces.read().await;
            guard.get(&id).cloned()
        };
        let w = w.ok_or(WorkspaceError::NotFound(id))?;
        if w.tenant_id != TenantId::from(viewer.tenant_id) {
            return Err(WorkspaceError::PermissionDenied);
        }
        Ok(w)
    }

    async fn get_by_key(
        &self,
        tenant_id: TenantId,
        workspace_key: &str,
        viewer: ActorContext,
    ) -> Result<Workspace, WorkspaceError> {
        Self::check_tenant(&viewer, TenantId::from(tenant_id))?;
        let guard = self.workspaces.read().await;
        guard
            .values()
            .find(|w| w.tenant_id == tenant_id && w.workspace_key == workspace_key)
            .cloned()
            .ok_or(WorkspaceError::NotFound(WorkspaceId::default()))
    }

    async fn list_workspaces(
        &self,
        q: ListWorkspaceQuery,
        viewer: ActorContext,
    ) -> Result<Vec<Workspace>, WorkspaceError> {
        Self::check_tenant(&viewer, q.tenant_id)?;
        let mut all: Vec<Workspace> = {
            let guard = self.workspaces.read().await;
            guard
                .values()
                .filter(|w| w.tenant_id == q.tenant_id)
                .cloned()
                .collect()
        };
        all.sort_by(|a, b| a.workspace_key.cmp(&b.workspace_key));
        let offset = q.offset as usize;
        let limit = q.limit as usize;
        Ok(all.into_iter().skip(offset).take(limit).collect())
    }

    async fn list_members(
        &self,
        workspace_id: WorkspaceId,
        viewer: ActorContext,
    ) -> Result<Vec<WorkspaceMember>, WorkspaceError> {
        Self::check_tenant(&viewer, TenantId::from(viewer.tenant_id))?;
        let guard = self.members.read().await;
        Ok(guard
            .values()
            .filter(|m| m.workspace_id == workspace_id && m.tenant_id == TenantId::from(viewer.tenant_id))
            .cloned()
            .collect())
    }
}

// =====================================================================
// 单元测试
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    fn make_actor(tenant_id: TenantId) -> ActorContext {
        ActorContext::new(Uuid::new_v4(), tenant_id.0).with_role(roles::WORKSPACE_ADMIN)
    }

    #[test]
    fn field_count_audit() {
        assert_eq!(Workspace::FIELD_COUNT, 8);
        assert_eq!(WorkspaceMember::FIELD_COUNT, 7);
    }

    #[tokio::test]
    async fn create_workspace_success() {
        let svc = InMemoryWorkspaceService::new_for_test();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_actor(tenant_id);
        let cmd = CreateWorkspaceCommand {
            tenant_id,
            workspace_key: "acme".to_string(),
            name: "Acme Workspace".to_string(),
            description: Some("main".to_string()),
            owner_user_id: UserId.new(),
        };
        let ws = svc.create_workspace(cmd, actor.clone()).await.unwrap();
        assert_eq!(ws.version, 1);
        assert_eq!(svc.count().await, 1);
        // owner 自动为 Admin 成员
        let members = svc.list_members(ws.id, actor).await.unwrap();
        assert_eq!(members.len(), 1);
        assert!(members[0].is_admin());
        assert_eq!(svc.member_count().await, 1);
    }

    #[tokio::test]
    async fn invariant_01_workspace_key_conflict() {
        let svc = InMemoryWorkspaceService::new_for_test();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_actor(tenant_id);
        let cmd1 = CreateWorkspaceCommand {
            tenant_id,
            workspace_key: "dup".to_string(),
            name: "W1".to_string(),
            description: None,
            owner_user_id: UserId.new(),
        };
        svc.create_workspace(cmd1, actor.clone()).await.unwrap();
        let cmd2 = CreateWorkspaceCommand {
            tenant_id,
            workspace_key: "dup".to_string(),
            name: "W2".to_string(),
            description: None,
            owner_user_id: UserId.new(),
        };
        let res = svc.create_workspace(cmd2, actor).await;
        assert!(matches!(res, Err(WorkspaceError::InvalidState(_))));
    }

    #[tokio::test]
    async fn invariant_03_empty_key_rejected() {
        let svc = InMemoryWorkspaceService::new_for_test();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_actor(tenant_id);
        let cmd = CreateWorkspaceCommand {
            tenant_id,
            workspace_key: "".to_string(),
            name: "Empty".to_string(),
            description: None,
            owner_user_id: UserId.new(),
        };
        let res = svc.create_workspace(cmd, actor).await;
        assert!(matches!(res, Err(WorkspaceError::InvalidState(_))));
    }

    #[tokio::test]
    async fn cross_tenant_access_denied() {
        let svc = InMemoryWorkspaceService::new_for_test();
        let tenant_a = uuid::Uuid::new_v4();
        let actor_a = make_actor(tenant_a);
        let ws = svc
            .create_workspace(
                CreateWorkspaceCommand {
                    tenant_id: tenant_a,
                    workspace_key: "a".to_string(),
                    name: "A".to_string(),
                    description: None,
                    owner_user_id: UserId.new(),
                },
                actor_a,
            )
            .await
            .unwrap();
        let tenant_b = uuid::Uuid::new_v4();
        let actor_b = make_actor(tenant_b);
        let res = svc.get_by_id(ws.id, actor_b).await;
        assert!(matches!(res, Err(WorkspaceError::PermissionDenied)));
    }

    #[tokio::test]
    async fn add_and_remove_member() {
        let svc = InMemoryWorkspaceService::new_for_test();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_actor(tenant_id);
        let ws = svc
            .create_workspace(
                CreateWorkspaceCommand {
                    tenant_id,
                    workspace_key: "ws".to_string(),
                    name: "WS".to_string(),
                    description: None,
                    owner_user_id: UserId.new(),
                },
                actor.clone(),
            )
            .await
            .unwrap();
        let new_user = uuid::Uuid::new_v4();
        let m = svc
            .add_member(
                AddMemberCommand {
                    workspace_id: ws.id,
                    tenant_id,
                    user_id: new_user,
                    role: WorkspaceRole::Member,
                },
                actor.clone(),
            )
            .await
            .unwrap();
        assert!(!m.is_admin());
        // 重复加 → Conflict
        let res = svc
            .add_member(
                AddMemberCommand {
                    workspace_id: ws.id,
                    tenant_id,
                    user_id: new_user,
                    role: WorkspaceRole::Member,
                },
                actor.clone(),
            )
            .await;
        assert!(matches!(res, Err(WorkspaceError::Conflict(_))));
        // 移除
        svc.remove_member(
            RemoveMemberCommand {
                workspace_id: ws.id,
                tenant_id,
                user_id: new_user,
            },
            actor,
        )
        .await
        .unwrap();
        let members = svc
            .list_members(ws.id, make_actor(tenant_id))
            .await
            .unwrap();
        // owner 还在
        assert_eq!(members.len(), 1);
    }

    #[tokio::test]
    async fn event_bus_receives_created() {
        let (svc, mut rx) = InMemoryWorkspaceService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_actor(tenant_id);
        let cmd = CreateWorkspaceCommand {
            tenant_id,
            workspace_key: "evt".to_string(),
            name: "E".to_string(),
            description: None,
            owner_user_id: UserId.new(),
        };
        svc.create_workspace(cmd, actor).await.unwrap();
        let evt = rx.try_recv().expect("应收到 Created 事件");
        assert!(matches!(evt, WorkspaceEvent::Created(_)));
        assert_eq!(evt.subject(), "star.events.workspace.workspace.created.v1");
    }

    #[tokio::test]
    async fn update_workspace_version_conflict() {
        let svc = InMemoryWorkspaceService::new_for_test();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_actor(tenant_id);
        let ws = svc
            .create_workspace(
                CreateWorkspaceCommand {
                    tenant_id,
                    workspace_key: "v".to_string(),
                    name: "V".to_string(),
                    description: None,
                    owner_user_id: UserId.new(),
                },
                actor.clone(),
            )
            .await
            .unwrap();
        let res = svc
            .update_workspace(
                UpdateWorkspaceCommand {
                    workspace_id: ws.id,
                    tenant_id,
                    expected_version: 99,
                    name: Some("New".to_string()),
                    description: None,
                },
                actor,
            )
            .await;
        assert!(matches!(res, Err(WorkspaceError::Conflict(_))));
    }
}
