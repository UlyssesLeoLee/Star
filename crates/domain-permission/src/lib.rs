//! domain-permission crate
//!
//! 详细 spec: docs/specs/domain-permission-spec.md §4.8 Permission 权限层
//! 上游基本设计: docs/basic-design.md §4.8
//! 数据设计: docs/data-design.md §4.8 (`permission_scheme` / `permission_rule` / `role_binding` schema)
//! API 设计: docs/api-design.md §3.8
//!
//! ## 职责
//!
//! 跨 tenant 安全边界的第二道闸门(§16,REQ-SEC-002):
//! - PermissionScheme 聚合根(tenant 内,带 rules)
//! - PermissionRule 值对象(subject × resource × action × effect)
//! - RoleBinding 实体(User → Project × Role 授权)
//! - check() 决策:遍历 rules,先 Deny 后 Allow,默认 Deny(白名单拒绝)
//!
//! ## 关键不变量 (INV-PM-01~05)
//!
//! - INV-PM-01:PermissionScheme 必带 tenant_id,跨 tenant 拒绝
//! - INV-PM-02:Deny 优先于 Allow
//! - INV-PM-03:RoleBinding 唯一 (user, project) — 不允许重复绑定
//! - INV-PM-04:Admin action 仅 tenant_admin / project_admin 可
//! - INV-PM-05:无匹配规则时,默认 Deny (拒绝默认)
//!
//! Lead 责任: permission Lead

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;
pub use star_context::ActorContext;

// =====================================================================
// ID 类型
// =====================================================================

define_uuid_id!(PermissionSchemeId);
define_uuid_id!(PermissionRuleId);
define_uuid_id!(RoleBindingId);
define_uuid_id!(TenantId);
define_uuid_id!(UserId);
define_uuid_id!(ProjectId);
define_uuid_id!(AgentId);
define_uuid_id!(AgentSessionId);
define_uuid_id!(RepositoryId);
define_uuid_id!(WorkItemId);

// =====================================================================
// UUID 强类型 ID 宏(参考 domain-worktree / domain-tenant 模式)
// =====================================================================

#[macro_export]
macro_rules! define_uuid_id {
    ($name:ident) => {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(pub Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }
            pub fn as_uuid(&self) -> Uuid {
                self.0
            }
        }

        impl From<Uuid> for $name {
            fn from(u: Uuid) -> Self {
                Self(u)
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

// =====================================================================
// 值对象:SubjectType / Role / ResourceType / Action / Effect
// =====================================================================

/// **Subject 主体类型** — 权限规则的左操作数
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SubjectType {
    /// 具体 User
    User,
    /// 角色(所有持有该 Role 的 User)
    Role,
    /// Agent 自身(本地 Runtime)
    Agent,
}

impl SubjectType {
    /// 大写字符串序列化(数据设计 §4.8)
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::User => "USER",
            Self::Role => "ROLE",
            Self::Agent => "AGENT",
        }
    }
}

/// **Role** — tenant 内角色(简化,5 种)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Role {
    /// 租户管理员
    TenantAdmin,
    /// 项目管理员
    ProjectAdmin,
    /// 开发者
    Developer,
    /// 只读访客
    Viewer,
    /// Agent 自身
    Agent,
}

impl Role {
    /// 大写字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::TenantAdmin => "TENANT_ADMIN",
            Self::ProjectAdmin => "PROJECT_ADMIN",
            Self::Developer => "DEVELOPER",
            Self::Viewer => "VIEWER",
            Self::Agent => "AGENT",
        }
    }
    /// 是否 admin 类(INV-PM-04 admin action 校验)
    pub fn is_admin(&self) -> bool {
        matches!(self, Self::TenantAdmin | Self::ProjectAdmin)
    }
    /// 从字符串解析(忽略大小写)
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "tenant_admin" | "tenantadmin" | "tenant-admin" => Some(Self::TenantAdmin),
            "project_admin" | "projectadmin" | "project-admin" => Some(Self::ProjectAdmin),
            "developer" | "dev" => Some(Self::Developer),
            "viewer" | "guest" => Some(Self::Viewer),
            "agent" => Some(Self::Agent),
            _ => None,
        }
    }
}

/// **Resource 类型** — 权限规则的右操作数
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    /// Project
    Project,
    /// WorkItem
    WorkItem,
    /// Worktree
    Worktree,
    /// Comment
    Comment,
    /// AgentSession
    AgentSession,
    /// Repository
    Repository,
}

impl ResourceType {
    /// 大写字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Project => "PROJECT",
            Self::WorkItem => "WORK_ITEM",
            Self::Worktree => "WORKTREE",
            Self::Comment => "COMMENT",
            Self::AgentSession => "AGENT_SESSION",
            Self::Repository => "REPOSITORY",
        }
    }
}

/// **Action** — 权限操作
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    /// 读
    Read,
    /// 写(增 / 改)
    Write,
    /// 删
    Delete,
    /// 管理(INV-PM-04 仅 admin)
    Admin,
    /// 审批
    Approve,
    /// 执行(Runtime / Agent)
    Execute,
}

impl Action {
    /// 大写字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Read => "READ",
            Self::Write => "WRITE",
            Self::Delete => "DELETE",
            Self::Admin => "ADMIN",
            Self::Approve => "APPROVE",
            Self::Execute => "EXECUTE",
        }
    }
}

/// **Effect** — Allow / Deny
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Effect {
    /// 允许
    Allow,
    /// 拒绝
    Deny,
}

impl Effect {
    /// 大写字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Allow => "ALLOW",
            Self::Deny => "DENY",
        }
    }
}

// =====================================================================
// 实体:PermissionScheme 聚合根 / PermissionRule 值对象 / RoleBinding 实体
// =====================================================================

/// **PermissionRule** — Scheme 内单条规则(值对象,§4.8)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRule {
    pub id: PermissionRuleId,
    /// 主体类型
    pub subject_type: SubjectType,
    /// 具体 subject 的 UUID(User / Agent 时必带;Role 时为 None 表示"该 Role 的所有人")
    pub subject_id: Option<Uuid>,
    /// 当 subject_type == Role 时,指定具体角色
    pub role: Option<Role>,
    /// 资源类型
    pub resource_type: ResourceType,
    /// 资源 UUID(None = 通配该类型所有资源)
    pub resource_id: Option<Uuid>,
    /// 允许/拒绝的动作集
    pub actions: Vec<Action>,
    /// 效果
    pub effect: Effect,
}

impl PermissionRule {
    /// 规则是否匹配 (subject, resource_type, resource_id, action)
    /// `actor_roles` 用于把 Role 主体展开为 User 的实际角色列表
    pub fn matches(
        &self,
        subject_kind: SubjectType,
        subject_uuid: Option<Uuid>,
        actor_roles: &[Role],
        resource_kind: ResourceType,
        resource_uuid: Option<Uuid>,
        action: Action,
    ) -> bool {
        // 资源类型必须一致
        if self.resource_type != resource_kind {
            return false;
        }
        // 资源 id:None 通配;有值需精确匹配
        if let Some(rid) = self.resource_id {
            if Some(rid) != resource_uuid {
                return false;
            }
        }
        // action 必须在 actions 中
        if !self.actions.contains(&action) {
            return false;
        }
        // subject 匹配
        match (self.subject_type, subject_kind) {
            (SubjectType::User, SubjectType::User) => {
                // subject_id 必带且相等
                self.subject_id == subject_uuid && subject_uuid.is_some()
            }
            (SubjectType::Agent, SubjectType::Agent) => {
                self.subject_id == subject_uuid && subject_uuid.is_some()
            }
            (SubjectType::Role, _) => {
                // 要求 self.role 命中 actor_roles
                if let Some(r) = self.role {
                    actor_roles.contains(&r)
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

/// **PermissionScheme** — 聚合根(§4.8)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionScheme {
    pub id: PermissionSchemeId,
    /// 必带,INV-PM-01
    pub tenant_id: TenantId,
    pub name: String,
    pub rules: Vec<PermissionRule>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    /// 乐观锁
    pub version: u32,
}

impl PermissionScheme {
    /// 新建一个空 Scheme(INV-PM-01: tenant_id 必带)
    pub fn new(tenant_id: TenantId, name: String) -> Self {
        let now = Utc::now();
        Self {
            id: PermissionSchemeId::new(),
            tenant_id,
            name,
            rules: vec![],
            created_at: now,
            updated_at: now,
            version: 1,
        }
    }

    /// 追加 / 替换 rule(upsert 语义,以 id 区分)
    pub fn upsert_rule(&mut self, rule: PermissionRule) {
        if let Some(existing) = self.rules.iter_mut().find(|r| r.id == rule.id) {
            *existing = rule;
        } else {
            self.rules.push(rule);
        }
        self.bump();
    }

    /// 按 id 移除 rule
    pub fn remove_rule(&mut self, rule_id: PermissionRuleId) -> bool {
        let before = self.rules.len();
        self.rules.retain(|r| r.id != rule_id);
        let removed = self.rules.len() != before;
        if removed {
            self.bump();
        }
        removed
    }

    fn bump(&mut self) {
        self.version = self.version.saturating_add(1);
        self.updated_at = Utc::now();
    }
}

/// **RoleBinding** — User × Project × Role 绑定(实体,§4.8,INV-PM-03 唯一)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleBinding {
    pub id: RoleBindingId,
    pub user_id: UserId,
    pub project_id: ProjectId,
    pub tenant_id: TenantId,
    pub role: Role,
    pub granted_by: UserId,
    pub granted_at: DateTime<Utc>,
}

impl RoleBinding {
    /// 唯一性 key (user, project) — INV-PM-03
    pub fn uniqueness_key(user_id: UserId, project_id: ProjectId) -> (UserId, ProjectId) {
        (user_id, project_id)
    }
}

// =====================================================================
// 错误
// =====================================================================

/// **PermissionError** — 权限域统一错误
#[derive(Debug, Error)]
pub enum PermissionError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("permission denied")]
    PermissionDenied,
    #[error("cross-tenant access denied: actor tenant {0} vs resource tenant {1}")]
    CrossTenantDenied(TenantId, TenantId),
    #[error("invalid rule: {0}")]
    InvalidRule(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("internal: {0}")]
    Internal(String),
}

impl PermissionError {
    /// 错误码(对接 API 错误码)
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "PERMISSION_NOT_FOUND",
            Self::PermissionDenied => "PERMISSION_PERMISSION_DENIED",
            Self::CrossTenantDenied(_, _) => "PERMISSION_CROSS_TENANT_DENIED",
            Self::InvalidRule(_) => "PERMISSION_INVALID_RULE",
            Self::Conflict(_) => "PERMISSION_CONFLICT",
            Self::Internal(_) => "PERMISSION_INTERNAL",
        }
    }
}

// =====================================================================
// 命令 / 查询 DTO
// =====================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSchemeCommand {
    pub tenant_id: TenantId,
    pub name: String,
    /// 创建者(用于 grant_by 之类的派生)
    pub actor_user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrantRoleCommand {
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub project_id: ProjectId,
    pub role: Role,
    pub granted_by: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevokeRoleCommand {
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub project_id: ProjectId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertRuleCommand {
    pub tenant_id: TenantId,
    pub scheme_id: PermissionSchemeId,
    pub rule: PermissionRule,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckQuery {
    pub tenant_id: TenantId,
    /// Scheme id(可选;若 None 则不基于 scheme,纯 RoleBinding + Admin 默认)
    pub scheme_id: Option<PermissionSchemeId>,
    pub subject_user_id: UserId,
    pub project_id: ProjectId,
    pub resource_type: ResourceType,
    pub resource_id: Option<Uuid>,
    pub action: Action,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRolesQuery {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSchemeQuery {
    pub tenant_id: TenantId,
    pub scheme_id: PermissionSchemeId,
}

// =====================================================================
// 端口(Port Traits)
// =====================================================================

/// **PermissionCommandPort** — 写操作(§3.8)
#[async_trait]
pub trait PermissionCommandPort: Send + Sync {
    async fn create_scheme(
        &self,
        cmd: CreateSchemeCommand,
        actor: &ActorContext,
    ) -> Result<PermissionScheme, PermissionError>;

    async fn grant_role(
        &self,
        cmd: GrantRoleCommand,
        actor: &ActorContext,
    ) -> Result<RoleBinding, PermissionError>;

    async fn revoke_role(
        &self,
        cmd: RevokeRoleCommand,
        actor: &ActorContext,
    ) -> Result<(), PermissionError>;

    async fn upsert_rule(
        &self,
        cmd: UpsertRuleCommand,
        actor: &ActorContext,
    ) -> Result<PermissionScheme, PermissionError>;
}

/// **PermissionQueryPort** — 读操作(§3.8)
#[async_trait]
pub trait PermissionQueryPort: Send + Sync {
    async fn check(&self, q: CheckQuery, actor: &ActorContext) -> Result<bool, PermissionError>;

    async fn list_roles(
        &self,
        q: ListRolesQuery,
        actor: &ActorContext,
    ) -> Result<Vec<RoleBinding>, PermissionError>;

    async fn get_scheme(
        &self,
        q: GetSchemeQuery,
        actor: &ActorContext,
    ) -> Result<PermissionScheme, PermissionError>;
}

/// **PermissionRepository** — 持久化抽象
#[async_trait]
pub trait PermissionRepository: Send + Sync {
    async fn insert_scheme(&self, s: PermissionScheme) -> Result<(), PermissionError>;
    async fn get_scheme(&self, id: PermissionSchemeId)
        -> Result<PermissionScheme, PermissionError>;
    async fn update_scheme(&self, s: PermissionScheme) -> Result<(), PermissionError>;

    async fn insert_binding(&self, b: RoleBinding) -> Result<(), PermissionError>;
    async fn remove_binding(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
        project_id: ProjectId,
    ) -> Result<(), PermissionError>;
    async fn list_bindings(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
    ) -> Result<Vec<RoleBinding>, PermissionError>;
}

// =====================================================================
// 核心决策:check()  — INV-PM-02 Deny 优先,INV-PM-05 默认 Deny
// =====================================================================

/// 解析 actor 在该 project 持有的所有角色
fn resolve_actor_roles(
    bindings: &[RoleBinding],
    user_id: UserId,
    project_id: ProjectId,
    is_local_runtime: bool,
) -> Vec<Role> {
    let mut roles: HashSet<Role> = bindings
        .iter()
        .filter(|b| b.user_id == user_id && b.project_id == project_id)
        .map(|b| b.role)
        .collect();
    // Agent 自身 subject 时,加上 Role::Agent
    if is_local_runtime {
        roles.insert(Role::Agent);
    }
    let mut out: Vec<Role> = roles.into_iter().collect();
    out.sort_by_key(|r| r.as_str());
    out
}

/// **核心 check** — 遍历 rules,先看是否有 Deny 命中再看 Allow
/// INV-PM-02: Deny 优先
/// INV-PM-04: Admin action 仅 admin Role 可
/// INV-PM-05: 无规则命中 → 默认 Deny
pub fn decide(
    scheme: Option<&PermissionScheme>,
    bindings: &[RoleBinding],
    actor_user_id: UserId,
    project_id: ProjectId,
    resource_type: ResourceType,
    resource_id: Option<Uuid>,
    action: Action,
    is_local_runtime: bool,
) -> Result<bool, PermissionError> {
    let actor_roles = resolve_actor_roles(bindings, actor_user_id, project_id, is_local_runtime);

    // INV-PM-04: Admin 动作需要 admin 类角色
    if action == Action::Admin && !actor_roles.iter().any(|r| r.is_admin()) {
        return Ok(false);
    }

    // 无 scheme 时:仅依赖 RoleBinding 默认允许(Developer/Viewer/Admin)
    // 简单策略:有 admin role → 允许;developer → 允许 Read/Write;viewer → 仅 Read
    let Some(scheme) = scheme else {
        return Ok(default_decide_without_scheme(&actor_roles, action));
    };

    // INV-PM-01: scheme 必带 tenant_id,跨 tenant 拒绝
    if scheme.tenant_id.as_uuid().is_nil() {
        return Err(PermissionError::InvalidRule(
            "INV-PM-01: scheme 缺少 tenant_id".to_string(),
        ));
    }

    // INV-PM-05: 没有任何规则 → 默认 Deny
    if scheme.rules.is_empty() {
        return Ok(false);
    }

    // 1) 扫一遍 Deny
    for rule in &scheme.rules {
        if rule.effect != Effect::Deny {
            continue;
        }
        if rule.matches(
            SubjectType::Role,
            None,
            &actor_roles,
            resource_type,
            resource_id,
            action,
        ) {
            return Ok(false);
        }
        if rule.matches(
            SubjectType::User,
            Some(actor_user_id.as_uuid()),
            &actor_roles,
            resource_type,
            resource_id,
            action,
        ) {
            return Ok(false);
        }
    }

    // 2) 扫一遍 Allow
    for rule in &scheme.rules {
        if rule.effect != Effect::Allow {
            continue;
        }
        if rule.matches(
            SubjectType::Role,
            None,
            &actor_roles,
            resource_type,
            resource_id,
            action,
        ) {
            return Ok(true);
        }
        if rule.matches(
            SubjectType::User,
            Some(actor_user_id.as_uuid()),
            &actor_roles,
            resource_type,
            resource_id,
            action,
        ) {
            return Ok(true);
        }
    }

    // 3) 都没命中 → 默认 Deny
    Ok(false)
}

/// 无 scheme 时的默认策略(向后兼容,Phase 1 兜底)
fn default_decide_without_scheme(roles: &[Role], action: Action) -> bool {
    if roles.iter().any(|r| r.is_admin()) {
        return true;
    }
    if roles.contains(&Role::Developer) {
        return matches!(action, Action::Read | Action::Write | Action::Execute);
    }
    if roles.contains(&Role::Viewer) {
        return matches!(action, Action::Read);
    }
    false
}

// =====================================================================
// InMemoryPermissionService
// =====================================================================

pub struct InMemoryPermissionService {
    repo: Arc<dyn PermissionRepository>,
    schemes: Arc<RwLock<HashMap<PermissionSchemeId, PermissionScheme>>>,
    bindings: Arc<RwLock<HashMap<(UserId, ProjectId), RoleBinding>>>,
}

impl InMemoryPermissionService {
    pub fn new() -> Self {
        Self {
            repo: Arc::new(InMemoryPermissionRepository::new()),
            schemes: Arc::new(RwLock::new(HashMap::new())),
            bindings: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_repo(repo: Arc<dyn PermissionRepository>) -> Self {
        Self {
            repo,
            schemes: Arc::new(RwLock::new(HashMap::new())),
            bindings: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn ensure_tenant(actor: &ActorContext, expected: TenantId) -> Result<(), PermissionError> {
        if TenantId::from(actor.tenant_id) != expected {
            return Err(PermissionError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                expected,
            ));
        }
        Ok(())
    }

    /// 测试便捷构造(用本地 service 内存,不经过 repo)
    pub fn new_for_test() -> Arc<Self> {
        Arc::new(Self::new())
    }
}

impl Default for InMemoryPermissionService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PermissionCommandPort for InMemoryPermissionService {
    async fn create_scheme(
        &self,
        cmd: CreateSchemeCommand,
        actor: &ActorContext,
    ) -> Result<PermissionScheme, PermissionError> {
        Self::ensure_tenant(actor, cmd.tenant_id)?;
        // 仅 tenant_admin / project_admin 可创建 scheme
        if !actor.has_role("tenant_admin") && !actor.has_role("project_admin") {
            return Err(PermissionError::PermissionDenied);
        }
        if cmd.name.trim().is_empty() {
            return Err(PermissionError::InvalidRule(
                "scheme name 不能为空".to_string(),
            ));
        }
        let scheme = PermissionScheme::new(cmd.tenant_id, cmd.name);
        self.repo.insert_scheme(scheme.clone()).await?;
        self.schemes
            .write()
            .expect("schemes lock")
            .insert(scheme.id, scheme.clone());
        Ok(scheme)
    }

    async fn grant_role(
        &self,
        cmd: GrantRoleCommand,
        actor: &ActorContext,
    ) -> Result<RoleBinding, PermissionError> {
        Self::ensure_tenant(actor, cmd.tenant_id)?;
        // 仅 tenant_admin / project_admin 可授权
        if !actor.has_role("tenant_admin") && !actor.has_role("project_admin") {
            return Err(PermissionError::PermissionDenied);
        }
        // INV-PM-03: (user, project) 唯一
        let key = RoleBinding::uniqueness_key(cmd.user_id, cmd.project_id);
        if self
            .bindings
            .read()
            .expect("bindings lock")
            .contains_key(&key)
        {
            return Err(PermissionError::Conflict(format!(
                "RoleBinding 已存在 (user={}, project={})",
                cmd.user_id, cmd.project_id
            )));
        }
        let binding = RoleBinding {
            id: RoleBindingId::new(),
            user_id: cmd.user_id,
            project_id: cmd.project_id,
            tenant_id: cmd.tenant_id,
            role: cmd.role,
            granted_by: cmd.granted_by,
            granted_at: Utc::now(),
        };
        self.repo.insert_binding(binding.clone()).await?;
        self.bindings
            .write()
            .expect("bindings lock")
            .insert(key, binding.clone());
        Ok(binding)
    }

    async fn revoke_role(
        &self,
        cmd: RevokeRoleCommand,
        actor: &ActorContext,
    ) -> Result<(), PermissionError> {
        Self::ensure_tenant(actor, cmd.tenant_id)?;
        if !actor.has_role("tenant_admin") && !actor.has_role("project_admin") {
            return Err(PermissionError::PermissionDenied);
        }
        let key = RoleBinding::uniqueness_key(cmd.user_id, cmd.project_id);
        let removed = self.bindings.write().expect("bindings lock").remove(&key);
        if removed.is_none() {
            return Err(PermissionError::NotFound(format!(
                "RoleBinding (user={}, project={})",
                cmd.user_id, cmd.project_id
            )));
        }
        self.repo
            .remove_binding(cmd.tenant_id, cmd.user_id, cmd.project_id)
            .await?;
        Ok(())
    }

    async fn upsert_rule(
        &self,
        cmd: UpsertRuleCommand,
        actor: &ActorContext,
    ) -> Result<PermissionScheme, PermissionError> {
        Self::ensure_tenant(actor, cmd.tenant_id)?;
        if !actor.has_role("tenant_admin") && !actor.has_role("project_admin") {
            return Err(PermissionError::PermissionDenied);
        }
        // 1) 校验规则合法性(无需持锁)
        match cmd.rule.subject_type {
            SubjectType::Role => {
                if cmd.rule.role.is_none() {
                    return Err(PermissionError::InvalidRule(
                        "Role 主体必须指定 role".to_string(),
                    ));
                }
            }
            SubjectType::User | SubjectType::Agent => {
                if cmd.rule.subject_id.is_none() {
                    return Err(PermissionError::InvalidRule(
                        "User/Agent 主体必须指定 subject_id".to_string(),
                    ));
                }
            }
        }
        if cmd.rule.actions.is_empty() {
            return Err(PermissionError::InvalidRule("actions 不能为空".to_string()));
        }
        // 2) 取出现有 scheme,在本地 upsert rule,然后写回并释放 guard
        let mut updated = {
            let store = self.schemes.read().expect("schemes lock");
            store
                .get(&cmd.scheme_id)
                .cloned()
                .ok_or_else(|| PermissionError::NotFound(format!("scheme {}", cmd.scheme_id)))?
        };
        if updated.tenant_id != cmd.tenant_id {
            return Err(PermissionError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                cmd.tenant_id,
            ));
        }
        updated.upsert_rule(cmd.rule);
        // 3) 写回(短持锁)
        self.schemes
            .write()
            .expect("schemes lock")
            .insert(updated.id, updated.clone());
        // 4) await repo
        self.repo.update_scheme(updated.clone()).await?;
        Ok(updated)
    }
}

#[async_trait]
impl PermissionQueryPort for InMemoryPermissionService {
    async fn check(&self, q: CheckQuery, actor: &ActorContext) -> Result<bool, PermissionError> {
        Self::ensure_tenant(actor, q.tenant_id)?;

        // 跨租户 sanity:actor 的 project 必须在自己 tenant(此处仅比对 tenant_id)
        let scheme = match q.scheme_id {
            Some(sid) => {
                let s = self
                    .schemes
                    .read()
                    .expect("schemes lock")
                    .get(&sid)
                    .cloned();
                match s {
                    Some(s) => {
                        if s.tenant_id != q.tenant_id {
                            return Err(PermissionError::CrossTenantDenied(
                                q.tenant_id,
                                s.tenant_id,
                            ));
                        }
                        Some(s)
                    }
                    None => return Err(PermissionError::NotFound(format!("scheme {}", sid))),
                }
            }
            None => None,
        };

        // 取出该项目所有 bindings(做 in-memory 决策)
        let bindings: Vec<RoleBinding> = self
            .bindings
            .read()
            .expect("bindings lock")
            .values()
            .filter(|b| b.tenant_id == q.tenant_id && b.project_id == q.project_id)
            .cloned()
            .collect();

        decide(
            scheme.as_ref(),
            &bindings,
            q.subject_user_id,
            q.project_id,
            q.resource_type,
            q.resource_id,
            q.action,
            actor.is_local_runtime,
        )
    }

    async fn list_roles(
        &self,
        q: ListRolesQuery,
        actor: &ActorContext,
    ) -> Result<Vec<RoleBinding>, PermissionError> {
        Self::ensure_tenant(actor, q.tenant_id)?;
        let store = self.bindings.read().expect("bindings lock");
        Ok(store
            .values()
            .filter(|b| b.tenant_id == q.tenant_id && b.project_id == q.project_id)
            .cloned()
            .collect())
    }

    async fn get_scheme(
        &self,
        q: GetSchemeQuery,
        actor: &ActorContext,
    ) -> Result<PermissionScheme, PermissionError> {
        Self::ensure_tenant(actor, q.tenant_id)?;
        let s = self
            .schemes
            .read()
            .expect("schemes lock")
            .get(&q.scheme_id)
            .cloned()
            .ok_or_else(|| PermissionError::NotFound(format!("scheme {}", q.scheme_id)))?;
        if s.tenant_id != q.tenant_id {
            return Err(PermissionError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                q.tenant_id,
            ));
        }
        Ok(s)
    }
}

// =====================================================================
// InMemoryPermissionRepository
// =====================================================================

pub struct InMemoryPermissionRepository {
    schemes: RwLock<HashMap<PermissionSchemeId, PermissionScheme>>,
    bindings: RwLock<HashMap<(UserId, ProjectId), RoleBinding>>,
}

impl InMemoryPermissionRepository {
    pub fn new() -> Self {
        Self {
            schemes: RwLock::new(HashMap::new()),
            bindings: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryPermissionRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PermissionRepository for InMemoryPermissionRepository {
    async fn insert_scheme(&self, s: PermissionScheme) -> Result<(), PermissionError> {
        let mut store = self.schemes.write().expect("lock");
        if store.contains_key(&s.id) {
            return Err(PermissionError::Conflict(format!("scheme {} 已存在", s.id)));
        }
        store.insert(s.id, s);
        Ok(())
    }
    async fn get_scheme(
        &self,
        id: PermissionSchemeId,
    ) -> Result<PermissionScheme, PermissionError> {
        self.schemes
            .read()
            .expect("lock")
            .get(&id)
            .cloned()
            .ok_or_else(|| PermissionError::NotFound(format!("scheme {}", id)))
    }
    async fn update_scheme(&self, s: PermissionScheme) -> Result<(), PermissionError> {
        self.schemes.write().expect("lock").insert(s.id, s);
        Ok(())
    }
    async fn insert_binding(&self, b: RoleBinding) -> Result<(), PermissionError> {
        let key = (b.user_id, b.project_id);
        let mut store = self.bindings.write().expect("lock");
        if store.contains_key(&key) {
            return Err(PermissionError::Conflict(format!(
                "binding ({}, {}) exists",
                b.user_id, b.project_id
            )));
        }
        store.insert(key, b);
        Ok(())
    }
    async fn remove_binding(
        &self,
        _tenant_id: TenantId,
        user_id: UserId,
        project_id: ProjectId,
    ) -> Result<(), PermissionError> {
        self.bindings
            .write()
            .expect("lock")
            .remove(&(user_id, project_id));
        Ok(())
    }
    async fn list_bindings(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
    ) -> Result<Vec<RoleBinding>, PermissionError> {
        Ok(self
            .bindings
            .read()
            .expect("lock")
            .values()
            .filter(|b| b.tenant_id == tenant_id && b.project_id == project_id)
            .cloned()
            .collect())
    }
}

// =====================================================================
// 测试(≥12)
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    fn admin_ctx(tenant_id: TenantId) -> ActorContext {
        ActorContext::new(Uuid::new_v4(), tenant_id.0).with_role("tenant_admin")
    }

    fn dev_ctx(tenant_id: TenantId) -> ActorContext {
        ActorContext::new(Uuid::new_v4(), tenant_id.0).with_role("developer")
    }

    fn viewer_ctx(tenant_id: TenantId) -> ActorContext {
        ActorContext::new(Uuid::new_v4(), tenant_id.0).with_role("viewer")
    }

    // ----- 基础 Allow / Deny -----

    #[tokio::test]
    async fn basic_allow() {
        let tenant = uuid::Uuid::new_v4();
        let project = ProjectId::new();
        let user = uuid::Uuid::new_v4();
        let admin = admin_ctx(tenant);

        let svc = InMemoryPermissionService::new();
        // 1) 创建 scheme
        let scheme = svc
            .create_scheme(
                CreateSchemeCommand {
                    tenant_id: tenant,
                    name: "default".into(),
                    actor_user_id: UserId::from(admin.user_id),
                },
                &admin,
            )
            .await
            .unwrap();
        // 2) 加 Allow rule:developer role 可读 Project
        let rule = PermissionRule {
            id: PermissionRuleId::new(),
            subject_type: SubjectType::Role,
            subject_id: None,
            role: Some(Role::Developer),
            resource_type: ResourceType::Project,
            resource_id: None,
            actions: vec![Action::Read],
            effect: Effect::Allow,
        };
        svc.upsert_rule(
            UpsertRuleCommand {
                tenant_id: tenant,
                scheme_id: scheme.id,
                rule,
            },
            &admin,
        )
        .await
        .unwrap();
        // 3) grant developer role 给 user
        svc.grant_role(
            GrantRoleCommand {
                tenant_id: tenant,
                user_id: user,
                project_id: project,
                role: Role::Developer,
                granted_by: UserId::from(admin.user_id),
            },
            &admin,
        )
        .await
        .unwrap();
        // 4) check
        let ok = svc
            .check(
                CheckQuery {
                    tenant_id: tenant,
                    scheme_id: Some(scheme.id),
                    subject_user_id: user,
                    project_id: project,
                    resource_type: ResourceType::Project,
                    resource_id: None,
                    action: Action::Read,
                },
                &ActorContext::new(user.0, tenant.0)
                    .with_role("developer")
                    .with_project(project),
            )
            .await
            .unwrap();
        assert!(ok);
    }

    #[tokio::test]
    async fn basic_deny_no_matching_rule() {
        let tenant = uuid::Uuid::new_v4();
        let project = ProjectId::new();
        let user = uuid::Uuid::new_v4();
        let admin = admin_ctx(tenant);

        let svc = InMemoryPermissionService::new();
        let scheme = svc
            .create_scheme(
                CreateSchemeCommand {
                    tenant_id: tenant,
                    name: "s".into(),
                    actor_user_id: UserId::from(admin.user_id),
                },
                &admin,
            )
            .await
            .unwrap();
        // scheme 没有任何 rule → 默认 Deny(INV-PM-05)
        let ok = svc
            .check(
                CheckQuery {
                    tenant_id: tenant,
                    scheme_id: Some(scheme.id),
                    subject_user_id: user,
                    project_id: project,
                    resource_type: ResourceType::Project,
                    resource_id: None,
                    action: Action::Read,
                },
                &ActorContext::new(user.0, tenant.0),
            )
            .await
            .unwrap();
        assert!(!ok, "无规则必须默认 Deny");
    }

    // ----- INV-PM-02 Deny 优先 -----

    #[tokio::test]
    async fn deny_takes_precedence_over_allow() {
        let tenant = uuid::Uuid::new_v4();
        let project = ProjectId::new();
        let user = uuid::Uuid::new_v4();
        let admin = admin_ctx(tenant);

        let svc = InMemoryPermissionService::new();
        let scheme = svc
            .create_scheme(
                CreateSchemeCommand {
                    tenant_id: tenant,
                    name: "s".into(),
                    actor_user_id: UserId::from(admin.user_id),
                },
                &admin,
            )
            .await
            .unwrap();
        // 两条 role=developer 的规则:Allow Read + Deny Read
        for effect in [Effect::Allow, Effect::Deny] {
            svc.upsert_rule(
                UpsertRuleCommand {
                    tenant_id: tenant,
                    scheme_id: scheme.id,
                    rule: PermissionRule {
                        id: PermissionRuleId::new(),
                        subject_type: SubjectType::Role,
                        subject_id: None,
                        role: Some(Role::Developer),
                        resource_type: ResourceType::Project,
                        resource_id: None,
                        actions: vec![Action::Read],
                        effect,
                    },
                },
                &admin,
            )
            .await
            .unwrap();
        }
        svc.grant_role(
            GrantRoleCommand {
                tenant_id: tenant,
                user_id: user,
                project_id: project,
                role: Role::Developer,
                granted_by: UserId::from(admin.user_id),
            },
            &admin,
        )
        .await
        .unwrap();
        let result = svc
            .check(
                CheckQuery {
                    tenant_id: tenant,
                    scheme_id: Some(scheme.id),
                    subject_user_id: user,
                    project_id: project,
                    resource_type: ResourceType::Project,
                    resource_id: None,
                    action: Action::Read,
                },
                &ActorContext::new(user.0, tenant.0)
                    .with_role("developer")
                    .with_project(project),
            )
            .await
            .unwrap();
        assert!(!result, "Deny 必须胜过 Allow(INV-PM-02)");
    }

    // ----- INV-PM-01 cross-tenant denied -----

    #[tokio::test]
    async fn cross_tenant_check_denied() {
        let tenant_a = uuid::Uuid::new_v4();
        let tenant_b = uuid::Uuid::new_v4();
        let project = ProjectId::new();
        let user = uuid::Uuid::new_v4();
        let admin_a = admin_ctx(tenant_a);

        let svc = InMemoryPermissionService::new();
        let scheme = svc
            .create_scheme(
                CreateSchemeCommand {
                    tenant_id: tenant_a,
                    name: "s".into(),
                    actor_user_id: UserId::from(admin_a.user_id),
                },
                &admin_a,
            )
            .await
            .unwrap();
        // 用 tenant_b 的 actor 去 check tenant_a 的 scheme → CrossTenantDenied
        let actor_b = ActorContext::new(user.0, tenant_b.0);
        let res = svc
            .check(
                CheckQuery {
                    tenant_id: tenant_b,
                    scheme_id: Some(scheme.id),
                    subject_user_id: user,
                    project_id: project,
                    resource_type: ResourceType::Project,
                    resource_id: None,
                    action: Action::Read,
                },
                &actor_b,
            )
            .await;
        assert!(matches!(res, Err(PermissionError::CrossTenantDenied(_, _))));
    }

    // ----- role grant + revoke -----

    #[tokio::test]
    async fn grant_and_revoke_role() {
        let tenant = uuid::Uuid::new_v4();
        let project = ProjectId::new();
        let user = uuid::Uuid::new_v4();
        let admin = admin_ctx(tenant);

        let svc = InMemoryPermissionService::new();
        let binding = svc
            .grant_role(
                GrantRoleCommand {
                    tenant_id: tenant,
                    user_id: user,
                    project_id: project,
                    role: Role::Developer,
                    granted_by: UserId::from(admin.user_id),
                },
                &admin,
            )
            .await
            .unwrap();
        assert_eq!(binding.role, Role::Developer);
        assert_eq!(binding.user_id, user);

        // 重复 grant → Conflict (INV-PM-03)
        let dup = svc
            .grant_role(
                GrantRoleCommand {
                    tenant_id: tenant,
                    user_id: user,
                    project_id: project,
                    role: Role::Viewer,
                    granted_by: UserId::from(admin.user_id),
                },
                &admin,
            )
            .await;
        assert!(matches!(dup, Err(PermissionError::Conflict(_))));

        // revoke
        svc.revoke_role(
            RevokeRoleCommand {
                tenant_id: tenant,
                user_id: user,
                project_id: project,
            },
            &admin,
        )
        .await
        .unwrap();
        // 再次 revoke → NotFound
        let again = svc
            .revoke_role(
                RevokeRoleCommand {
                    tenant_id: tenant,
                    user_id: user,
                    project_id: project,
                },
                &admin,
            )
            .await;
        assert!(matches!(again, Err(PermissionError::NotFound(_))));
    }

    // ----- role binding 唯一性(INV-PM-03)-----

    #[tokio::test]
    async fn role_binding_unique() {
        let tenant = uuid::Uuid::new_v4();
        let project = ProjectId::new();
        let user = uuid::Uuid::new_v4();
        let admin = admin_ctx(tenant);
        let svc = InMemoryPermissionService::new();
        svc.grant_role(
            GrantRoleCommand {
                tenant_id: tenant,
                user_id: user,
                project_id: project,
                role: Role::Developer,
                granted_by: UserId::from(admin.user_id),
            },
            &admin,
        )
        .await
        .unwrap();
        // 尝试给 (user, project) 再绑不同 role
        let res = svc
            .grant_role(
                GrantRoleCommand {
                    tenant_id: tenant,
                    user_id: user,
                    project_id: project,
                    role: Role::Viewer,
                    granted_by: UserId::from(admin.user_id),
                },
                &admin,
            )
            .await;
        assert!(matches!(res, Err(PermissionError::Conflict(_))));
    }

    // ----- INV-PM-04 Admin 角色拒绝 developer -----

    #[tokio::test]
    async fn admin_action_requires_admin_role() {
        let tenant = uuid::Uuid::new_v4();
        let project = ProjectId::new();
        let user = uuid::Uuid::new_v4();
        let admin = admin_ctx(tenant);

        let svc = InMemoryPermissionService::new();
        let scheme = svc
            .create_scheme(
                CreateSchemeCommand {
                    tenant_id: tenant,
                    name: "s".into(),
                    actor_user_id: UserId::from(admin.user_id),
                },
                &admin,
            )
            .await
            .unwrap();
        // 给 developer role 显式 Allow Admin(实际是绕过 INV-PM-04,但我们要校验:
        // 即使有 Allow,INV-PM-04 也会在 decide() 中拒绝 developer 走 admin 动作)
        svc.upsert_rule(
            UpsertRuleCommand {
                tenant_id: tenant,
                scheme_id: scheme.id,
                rule: PermissionRule {
                    id: PermissionRuleId::new(),
                    subject_type: SubjectType::Role,
                    subject_id: None,
                    role: Some(Role::Developer),
                    resource_type: ResourceType::Project,
                    resource_id: None,
                    actions: vec![Action::Admin],
                    effect: Effect::Allow,
                },
            },
            &admin,
        )
        .await
        .unwrap();
        svc.grant_role(
            GrantRoleCommand {
                tenant_id: tenant,
                user_id: user,
                project_id: project,
                role: Role::Developer,
                granted_by: UserId::from(admin.user_id),
            },
            &admin,
        )
        .await
        .unwrap();
        let res = svc
            .check(
                CheckQuery {
                    tenant_id: tenant,
                    scheme_id: Some(scheme.id),
                    subject_user_id: user,
                    project_id: project,
                    resource_type: ResourceType::Project,
                    resource_id: None,
                    action: Action::Admin,
                },
                &ActorContext::new(user.0, tenant.0)
                    .with_role("developer")
                    .with_project(project),
            )
            .await
            .unwrap();
        assert!(!res, "developer 不能执行 Admin 动作(INV-PM-04)");
    }

    // ----- 通配符 resource_id -----

    #[tokio::test]
    async fn wildcard_resource_id() {
        let tenant = uuid::Uuid::new_v4();
        let project = ProjectId::new();
        let user = uuid::Uuid::new_v4();
        let admin = admin_ctx(tenant);
        let svc = InMemoryPermissionService::new();
        let scheme = svc
            .create_scheme(
                CreateSchemeCommand {
                    tenant_id: tenant,
                    name: "s".into(),
                    actor_user_id: UserId::from(admin.user_id),
                },
                &admin,
            )
            .await
            .unwrap();
        // 通配 resource_id(None)
        svc.upsert_rule(
            UpsertRuleCommand {
                tenant_id: tenant,
                scheme_id: scheme.id,
                rule: PermissionRule {
                    id: PermissionRuleId::new(),
                    subject_type: SubjectType::Role,
                    subject_id: None,
                    role: Some(Role::Viewer),
                    resource_type: ResourceType::WorkItem,
                    resource_id: None, // 通配
                    actions: vec![Action::Read],
                    effect: Effect::Allow,
                },
            },
            &admin,
        )
        .await
        .unwrap();
        svc.grant_role(
            GrantRoleCommand {
                tenant_id: tenant,
                user_id: user,
                project_id: project,
                role: Role::Viewer,
                granted_by: UserId::from(admin.user_id),
            },
            &admin,
        )
        .await
        .unwrap();
        // 任意 resource_id 都应命中
        for _ in 0..3 {
            let res = svc
                .check(
                    CheckQuery {
                        tenant_id: tenant,
                        scheme_id: Some(scheme.id),
                        subject_user_id: user,
                        project_id: project,
                        resource_type: ResourceType::WorkItem,
                        resource_id: Some(Uuid::new_v4()),
                        action: Action::Read,
                    },
                    &ActorContext::new(user.0, tenant.0)
                        .with_role("viewer")
                        .with_project(project),
                )
                .await
                .unwrap();
            assert!(res, "通配 resource_id 必须命中");
        }
    }

    // ----- 完整 workflow: grant role → create rule → check 成功 -----

    #[tokio::test]
    async fn end_to_end_workflow() {
        let tenant = uuid::Uuid::new_v4();
        let project = ProjectId::new();
        let user = uuid::Uuid::new_v4();
        let admin = admin_ctx(tenant);
        let svc = InMemoryPermissionService::new();

        // 1) grant role
        svc.grant_role(
            GrantRoleCommand {
                tenant_id: tenant,
                user_id: user,
                project_id: project,
                role: Role::ProjectAdmin,
                granted_by: UserId::from(admin.user_id),
            },
            &admin,
        )
        .await
        .unwrap();
        // 2) create scheme + allow rule
        let scheme = svc
            .create_scheme(
                CreateSchemeCommand {
                    tenant_id: tenant,
                    name: "p-admin".into(),
                    actor_user_id: UserId::from(admin.user_id),
                },
                &admin,
            )
            .await
            .unwrap();
        svc.upsert_rule(
            UpsertRuleCommand {
                tenant_id: tenant,
                scheme_id: scheme.id,
                rule: PermissionRule {
                    id: PermissionRuleId::new(),
                    subject_type: SubjectType::Role,
                    subject_id: None,
                    role: Some(Role::ProjectAdmin),
                    resource_type: ResourceType::Project,
                    resource_id: None,
                    actions: vec![Action::Read, Action::Write, Action::Admin],
                    effect: Effect::Allow,
                },
            },
            &admin,
        )
        .await
        .unwrap();
        // 3) check 各动作
        let actor_user = ActorContext::new(user.0, tenant.0)
            .with_role("project_admin")
            .with_project(project);
        for action in [Action::Read, Action::Write, Action::Admin] {
            let ok = svc
                .check(
                    CheckQuery {
                        tenant_id: tenant,
                        scheme_id: Some(scheme.id),
                        subject_user_id: user,
                        project_id: project,
                        resource_type: ResourceType::Project,
                        resource_id: None,
                        action,
                    },
                    &actor_user,
                )
                .await
                .unwrap();
            assert!(ok, "project_admin 应被允许 {:?}", action);
        }
    }

    // ----- scheme 创建 + 读取 -----

    #[tokio::test]
    async fn create_and_get_scheme() {
        let tenant = uuid::Uuid::new_v4();
        let admin = admin_ctx(tenant);
        let svc = InMemoryPermissionService::new();
        let scheme = svc
            .create_scheme(
                CreateSchemeCommand {
                    tenant_id: tenant,
                    name: "my-scheme".into(),
                    actor_user_id: UserId::from(admin.user_id),
                },
                &admin,
            )
            .await
            .unwrap();
        assert_eq!(scheme.name, "my-scheme");
        assert!(scheme.rules.is_empty());
        let fetched = svc
            .get_scheme(
                GetSchemeQuery {
                    tenant_id: tenant,
                    scheme_id: scheme.id,
                },
                &admin,
            )
            .await
            .unwrap();
        assert_eq!(fetched.id, scheme.id);
    }

    // ----- list roles -----

    #[tokio::test]
    async fn list_roles_for_project() {
        let tenant = uuid::Uuid::new_v4();
        let project = ProjectId::new();
        let admin = admin_ctx(tenant);
        let svc = InMemoryPermissionService::new();
        for _ in 0..3 {
            svc.grant_role(
                GrantRoleCommand {
                    tenant_id: tenant,
                    user_id: UserId.new(),
                    project_id: project,
                    role: Role::Developer,
                    granted_by: UserId::from(admin.user_id),
                },
                &admin,
            )
            .await
            .unwrap();
        }
        // 另一个 project,不应被列出
        svc.grant_role(
            GrantRoleCommand {
                tenant_id: tenant,
                user_id: UserId.new(),
                project_id: ProjectId::new(),
                role: Role::Viewer,
                granted_by: UserId::from(admin.user_id),
            },
            &admin,
        )
        .await
        .unwrap();
        let list = svc
            .list_roles(
                ListRolesQuery {
                    tenant_id: tenant,
                    project_id: project,
                },
                &admin,
            )
            .await
            .unwrap();
        assert_eq!(list.len(), 3);
        assert!(list.iter().all(|b| b.project_id == project));
    }

    // ----- cross-tenant role check 拒绝 -----

    #[tokio::test]
    async fn cross_tenant_role_check_denied() {
        let tenant_a = uuid::Uuid::new_v4();
        let tenant_b = uuid::Uuid::new_v4();
        let project = ProjectId::new();
        let user = uuid::Uuid::new_v4();
        let admin_a = admin_ctx(tenant_a);
        let admin_b = admin_ctx(tenant_b);
        let svc = InMemoryPermissionService::new();
        // 在 tenant_a 授权
        svc.grant_role(
            GrantRoleCommand {
                tenant_id: tenant_a,
                user_id: user,
                project_id: project,
                role: Role::Developer,
                granted_by: UserId::from(admin_a.user_id),
            },
            &admin_a,
        )
        .await
        .unwrap();
        // 用 tenant_b 的 actor 去 list → CrossTenantDenied
        let res = svc
            .list_roles(
                ListRolesQuery {
                    tenant_id: tenant_a,
                    project_id: project,
                },
                &admin_b,
            )
            .await;
        assert!(matches!(res, Err(PermissionError::CrossTenantDenied(_, _))));
    }

    // ----- bonus: create_scheme 拒绝 developer(INV-PM-04 写入侧)-----

    #[tokio::test]
    async fn non_admin_cannot_create_scheme() {
        let tenant = uuid::Uuid::new_v4();
        let dev = dev_ctx(tenant);
        let svc = InMemoryPermissionService::new();
        let res = svc
            .create_scheme(
                CreateSchemeCommand {
                    tenant_id: tenant,
                    name: "x".into(),
                    actor_user_id: UserId::from(dev.user_id),
                },
                &dev,
            )
            .await;
        assert!(matches!(res, Err(PermissionError::PermissionDenied)));
    }

    // ----- bonus: invalid rule rejection -----

    #[tokio::test]
    async fn invalid_rule_rejected() {
        let tenant = uuid::Uuid::new_v4();
        let admin = admin_ctx(tenant);
        let svc = InMemoryPermissionService::new();
        let scheme = svc
            .create_scheme(
                CreateSchemeCommand {
                    tenant_id: tenant,
                    name: "s".into(),
                    actor_user_id: UserId::from(admin.user_id),
                },
                &admin,
            )
            .await
            .unwrap();
        // Role 主体但没 role 字段
        let res = svc
            .upsert_rule(
                UpsertRuleCommand {
                    tenant_id: tenant,
                    scheme_id: scheme.id,
                    rule: PermissionRule {
                        id: PermissionRuleId::new(),
                        subject_type: SubjectType::Role,
                        subject_id: None,
                        role: None, // 缺
                        resource_type: ResourceType::Project,
                        resource_id: None,
                        actions: vec![Action::Read],
                        effect: Effect::Allow,
                    },
                },
                &admin,
            )
            .await;
        assert!(matches!(res, Err(PermissionError::InvalidRule(_))));

        // User 主体但没 subject_id
        let res = svc
            .upsert_rule(
                UpsertRuleCommand {
                    tenant_id: tenant,
                    scheme_id: scheme.id,
                    rule: PermissionRule {
                        id: PermissionRuleId::new(),
                        subject_type: SubjectType::User,
                        subject_id: None, // 缺
                        role: None,
                        resource_type: ResourceType::Project,
                        resource_id: None,
                        actions: vec![Action::Read],
                        effect: Effect::Allow,
                    },
                },
                &admin,
            )
            .await;
        assert!(matches!(res, Err(PermissionError::InvalidRule(_))));

        // 空 actions
        let res = svc
            .upsert_rule(
                UpsertRuleCommand {
                    tenant_id: tenant,
                    scheme_id: scheme.id,
                    rule: PermissionRule {
                        id: PermissionRuleId::new(),
                        subject_type: SubjectType::Role,
                        subject_id: None,
                        role: Some(Role::Developer),
                        resource_type: ResourceType::Project,
                        resource_id: None,
                        actions: vec![],
                        effect: Effect::Allow,
                    },
                },
                &admin,
            )
            .await;
        assert!(matches!(res, Err(PermissionError::InvalidRule(_))));
    }

    // ----- bonus: actor parsing (Role::from_str_opt)-----

    #[test]
    fn role_from_str_parsing() {
        assert_eq!(Role::from_str_opt("tenant_admin"), Some(Role::TenantAdmin));
        assert_eq!(
            Role::from_str_opt("project_admin"),
            Some(Role::ProjectAdmin)
        );
        assert_eq!(Role::from_str_opt("developer"), Some(Role::Developer));
        assert_eq!(Role::from_str_opt("viewer"), Some(Role::Viewer));
        assert_eq!(Role::from_str_opt("agent"), Some(Role::Agent));
        assert_eq!(Role::from_str_opt("unknown"), None);
        // admin 判定
        assert!(Role::TenantAdmin.is_admin());
        assert!(Role::ProjectAdmin.is_admin());
        assert!(!Role::Developer.is_admin());
        assert!(!Role::Viewer.is_admin());
    }
}
