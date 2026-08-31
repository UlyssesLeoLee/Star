//! domain-relation crate
//!
//! 详细 spec: docs/specs/domain-relation-spec.md §10.5 实体关系图
//! 上游基本设计: docs/basic-design.md §4.10
//! 数据设计: docs/data-design.md §4.10 (`relation` / `relation_group` schema)
//! API 设计: docs/api-design.md §3.10
//!
//! ## 职责
//!
//! 实体关系图(Relation Graph)域,§10.5:
//! - 任意两个实体之间关系的声明(WorkItem ↔ Project / Repository / Symbol / Decision / WorkItem)
//! - 关系分组(RelationGroup)作为业务视图
//! - 支持 1 hop / N hop 关系图遍历(BFS 简化)
//!
//! ## 关键不变量(INV-RL-01~05)
//!
//! - INV-RL-01:Relation 必带 tenant_id,跨 tenant 拒绝
//! - INV-RL-02:自关系禁止(from_type == to_type && from_id == to_id)
//! - INV-RL-03:跨类型关系允许(WorkItem → Symbol 等)
//! - INV-RL-04:跨 tenant 一律拒绝
//! - INV-RL-05:删除 from / to 实体时级联标记 deleted(不物理删)
//!
//! Lead 责任: relation Lead

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;
pub use star_context::ActorContext;

// =====================================================================
// UUID 强类型 ID 宏(参考 domain-tenant / domain-permission 模式)
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
// ID 类型
// =====================================================================

define_uuid_id!(RelationId);
define_uuid_id!(RelationGroupId);
define_uuid_id!(TenantId);
define_uuid_id!(UserId);
define_uuid_id!(ProjectId);
define_uuid_id!(WorkItemId);
define_uuid_id!(RepositoryId);
define_uuid_id!(SymbolId);
define_uuid_id!(DecisionId);

// =====================================================================
// 值对象:ResourceType / RelationType
// =====================================================================

/// **ResourceType** — 关系端点允许的实体类型(§10.5)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    /// WorkItem(任务)
    WorkItem,
    /// Project(项目)
    Project,
    /// Repository(代码仓库)
    Repository,
    /// Symbol(代码符号)
    Symbol,
    /// Decision(架构/产品决策记录)
    Decision,
}

impl ResourceType {
    /// 大写字符串(数据设计 §4.10 序列化约定)
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::WorkItem => "WORK_ITEM",
            Self::Project => "PROJECT",
            Self::Repository => "REPOSITORY",
            Self::Symbol => "SYMBOL",
            Self::Decision => "DECISION",
        }
    }

    /// 从字符串解析(大小写不敏感,便于外部输入)
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_uppercase().as_str() {
            "WORK_ITEM" | "WORKITEM" | "WORK-ITEM" => Some(Self::WorkItem),
            "PROJECT" => Some(Self::Project),
            "REPOSITORY" | "REPO" => Some(Self::Repository),
            "SYMBOL" => Some(Self::Symbol),
            "DECISION" => Some(Self::Decision),
            _ => None,
        }
    }
}

impl std::fmt::Display for ResourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// **RelationType** — 关系种类(§10.5)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationType {
    /// 阻塞(强前置)
    Blocks,
    /// 重复(同义/合并候选)
    Duplicates,
    /// 关闭(实现后关闭目标)
    Closes,
    /// 实现(对决策/任务的实现)
    Implements,
    /// 引用(代码/文档的软引用)
    References,
    /// 依赖(运行时/构建依赖)
    Depends,
    /// 兄弟(同父节点下的关联)
    Sibling,
}

impl RelationType {
    /// 大写字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Blocks => "BLOCKS",
            Self::Duplicates => "DUPLICATES",
            Self::Closes => "CLOSES",
            Self::Implements => "IMPLEMENTS",
            Self::References => "REFERENCES",
            Self::Depends => "DEPENDS",
            Self::Sibling => "SIBLING",
        }
    }

    /// 从字符串解析
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_uppercase().as_str() {
            "BLOCKS" => Some(Self::Blocks),
            "DUPLICATES" => Some(Self::Duplicates),
            "CLOSES" => Some(Self::Closes),
            "IMPLEMENTS" => Some(Self::Implements),
            "REFERENCES" => Some(Self::References),
            "DEPENDS" => Some(Self::Depends),
            "SIBLING" => Some(Self::Sibling),
            _ => None,
        }
    }

    /// 是否为强方向关系(影响 cycle 检测优先级)
    pub fn is_strong(&self) -> bool {
        matches!(self, Self::Blocks | Self::Closes | Self::Depends)
    }
}

// =====================================================================
// 实体:Relation 聚合根 / RelationGroup 实体
// =====================================================================

/// **Relation** — 实体关系(§10.5,聚合根)
///
/// 任意两实体之间的关系。必带 tenant_id(INV-RL-01);
/// from 和 to 不允许指向同一资源(INV-RL-02);
/// 跨类型关系允许(WorkItem → Symbol,INV-RL-03);
/// 跨 tenant 一律拒绝(INV-RL-04);
/// 端点实体删除时级联标记(INV-RL-05)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    pub id: RelationId,
    /// 必带,INV-RL-01
    pub tenant_id: TenantId,
    /// 起点实体类型
    pub from_type: ResourceType,
    /// 起点实体 UUID
    pub from_id: Uuid,
    /// 关系种类
    pub relation_type: RelationType,
    /// 终点实体类型(INV-RL-03:允许与 from_type 不同)
    pub to_type: ResourceType,
    /// 终点实体 UUID
    pub to_id: Uuid,
    /// 备注(可选)
    pub note: Option<String>,
    /// 创建者
    pub created_by: UserId,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 软删除标记(INV-RL-05:级联标记)
    pub deleted: bool,
}

impl Relation {
    /// 端点对 (from_type, from_id)
    pub fn from_endpoint(&self) -> (ResourceType, Uuid) {
        (self.from_type, self.from_id)
    }
    /// 端点对 (to_type, to_id)
    pub fn to_endpoint(&self) -> (ResourceType, Uuid) {
        (self.to_type, self.to_id)
    }
    /// 端点是否包含指定 (type, id)
    pub fn touches(&self, kind: ResourceType, id: Uuid) -> bool {
        (self.from_type == kind && self.from_id == id) || (self.to_type == kind && self.to_id == id)
    }
}

/// **RelationGroup** — 关系分组(§10.5 业务视图)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationGroup {
    pub id: RelationGroupId,
    pub tenant_id: TenantId,
    pub name: String,
    pub description: String,
    pub relation_ids: Vec<RelationId>,
    pub created_by: UserId,
    pub created_at: DateTime<Utc>,
}

impl RelationGroup {
    /// 新建空分组
    pub fn new(tenant_id: TenantId, name: String, description: String, created_by: UserId) -> Self {
        Self {
            id: RelationGroupId::new(),
            tenant_id,
            name,
            description,
            relation_ids: vec![],
            created_by,
            created_at: Utc::now(),
        }
    }
}

// =====================================================================
// 错误
// =====================================================================

/// **RelationError** — 关系域统一错误
#[derive(Debug, Error)]
pub enum RelationError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("permission denied")]
    PermissionDenied,
    #[error("cross-tenant access denied: actor tenant {0} vs resource tenant {1}")]
    CrossTenantDenied(TenantId, TenantId),
    #[error("self-relation not allowed (INV-RL-02): from == to ({0}:{1})")]
    SelfRelation(ResourceType, Uuid),
    #[error("cycle detected: {0}")]
    CycleDetected(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("internal: {0}")]
    Internal(String),
}

impl RelationError {
    /// 错误码
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "RELATION_NOT_FOUND",
            Self::PermissionDenied => "RELATION_PERMISSION_DENIED",
            Self::CrossTenantDenied(_, _) => "RELATION_CROSS_TENANT_DENIED",
            Self::SelfRelation(_, _) => "RELATION_SELF_RELATION",
            Self::CycleDetected(_) => "RELATION_CYCLE_DETECTED",
            Self::Conflict(_) => "RELATION_CONFLICT",
            Self::InvalidInput(_) => "RELATION_INVALID_INPUT",
            Self::Internal(_) => "RELATION_INTERNAL",
        }
    }
}

// =====================================================================
// 命令 / 查询 DTO
// =====================================================================

/// 创建关系命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRelationCommand {
    pub tenant_id: TenantId,
    pub from_type: ResourceType,
    pub from_id: Uuid,
    pub relation_type: RelationType,
    pub to_type: ResourceType,
    pub to_id: Uuid,
    pub note: Option<String>,
}

/// 删除关系命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteRelationCommand {
    pub tenant_id: TenantId,
    pub relation_id: RelationId,
}

/// 创建关系分组命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRelationGroupCommand {
    pub tenant_id: TenantId,
    pub name: String,
    pub description: String,
}

/// 添加关系到分组命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddToGroupCommand {
    pub tenant_id: TenantId,
    pub group_id: RelationGroupId,
    pub relation_id: RelationId,
}

/// 按端点查询(查询 from 或 to)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListByEndpointQuery {
    pub tenant_id: TenantId,
    pub resource_type: ResourceType,
    pub resource_id: Uuid,
    /// Some(true)=仅 from, Some(false)=仅 to, None=双向
    pub from_only: Option<bool>,
}

/// 按关系类型查询
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListByTypeQuery {
    pub tenant_id: TenantId,
    pub relation_type: RelationType,
}

/// 关系图查询
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetGraphQuery {
    pub tenant_id: TenantId,
    pub resource_type: ResourceType,
    pub resource_id: Uuid,
    /// BFS 深度(1 = 直接相邻,2 = 二跳 …),0 视作 1
    pub depth: u32,
    /// 可选按 relation_type 过滤
    pub filter_type: Option<RelationType>,
}

// =====================================================================
// 端口(Port Traits)
// =====================================================================

/// **RelationCommandPort** — 写操作(§3.10)
#[async_trait]
pub trait RelationCommandPort: Send + Sync {
    async fn create_relation(
        &self,
        cmd: CreateRelationCommand,
        actor: &ActorContext,
    ) -> Result<Relation, RelationError>;

    async fn delete_relation(
        &self,
        cmd: DeleteRelationCommand,
        actor: &ActorContext,
    ) -> Result<(), RelationError>;

    async fn create_group(
        &self,
        cmd: CreateRelationGroupCommand,
        actor: &ActorContext,
    ) -> Result<RelationGroup, RelationError>;

    async fn add_to_group(
        &self,
        cmd: AddToGroupCommand,
        actor: &ActorContext,
    ) -> Result<RelationGroup, RelationError>;

    async fn remove_from_group(
        &self,
        cmd: AddToGroupCommand,
        actor: &ActorContext,
    ) -> Result<RelationGroup, RelationError>;
}

/// **RelationQueryPort** — 读操作(§3.10)
#[async_trait]
pub trait RelationQueryPort: Send + Sync {
    async fn get_relation(
        &self,
        id: RelationId,
        actor: &ActorContext,
    ) -> Result<Relation, RelationError>;

    async fn list_from(
        &self,
        q: ListByEndpointQuery,
        actor: &ActorContext,
    ) -> Result<Vec<Relation>, RelationError>;

    async fn list_to(
        &self,
        q: ListByEndpointQuery,
        actor: &ActorContext,
    ) -> Result<Vec<Relation>, RelationError>;

    async fn list_by_type(
        &self,
        q: ListByTypeQuery,
        actor: &ActorContext,
    ) -> Result<Vec<Relation>, RelationError>;

    async fn get_group(
        &self,
        group_id: RelationGroupId,
        actor: &ActorContext,
    ) -> Result<RelationGroup, RelationError>;

    async fn get_graph(
        &self,
        q: GetGraphQuery,
        actor: &ActorContext,
    ) -> Result<RelationGraph, RelationError>;
}

/// 关系图查询结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationGraph {
    pub root: GraphNode,
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub depth: u32,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub resource_type: ResourceType,
    pub resource_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub relation_id: RelationId,
    pub from: GraphNode,
    pub to: GraphNode,
    pub relation_type: RelationType,
}

/// **RelationRepository** — 持久化抽象
#[async_trait]
pub trait RelationRepository: Send + Sync {
    async fn insert_relation(&self, r: Relation) -> Result<(), RelationError>;
    async fn update_relation(&self, r: Relation) -> Result<(), RelationError>;
    async fn get_relation(&self, id: RelationId) -> Result<Relation, RelationError>;
    async fn list_relations(&self, tenant_id: TenantId) -> Result<Vec<Relation>, RelationError>;

    async fn insert_group(&self, g: RelationGroup) -> Result<(), RelationError>;
    async fn update_group(&self, g: RelationGroup) -> Result<(), RelationError>;
    async fn get_group(&self, id: RelationGroupId) -> Result<RelationGroup, RelationError>;
}

// =====================================================================
// InMemoryRelationRepository
// =====================================================================

pub struct InMemoryRelationRepository {
    relations: RwLock<HashMap<RelationId, Relation>>,
    groups: RwLock<HashMap<RelationGroupId, RelationGroup>>,
}

impl InMemoryRelationRepository {
    pub fn new() -> Self {
        Self {
            relations: RwLock::new(HashMap::new()),
            groups: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryRelationRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RelationRepository for InMemoryRelationRepository {
    async fn insert_relation(&self, r: Relation) -> Result<(), RelationError> {
        let mut s = self.relations.write().expect("lock");
        if s.contains_key(&r.id) {
            return Err(RelationError::Conflict(format!("Relation {} 已存在", r.id)));
        }
        s.insert(r.id, r);
        Ok(())
    }
    async fn update_relation(&self, r: Relation) -> Result<(), RelationError> {
        let mut s = self.relations.write().expect("lock");
        s.insert(r.id, r);
        Ok(())
    }
    async fn get_relation(&self, id: RelationId) -> Result<Relation, RelationError> {
        let s = self.relations.read().expect("lock");
        s.get(&id)
            .cloned()
            .ok_or_else(|| RelationError::NotFound(format!("relation:{}", id)))
    }
    async fn list_relations(&self, tenant_id: TenantId) -> Result<Vec<Relation>, RelationError> {
        let s = self.relations.read().expect("lock");
        Ok(s.values()
            .filter(|r| r.tenant_id == tenant_id)
            .cloned()
            .collect())
    }
    async fn insert_group(&self, g: RelationGroup) -> Result<(), RelationError> {
        let mut s = self.groups.write().expect("lock");
        if s.contains_key(&g.id) {
            return Err(RelationError::Conflict(format!(
                "RelationGroup {} 已存在",
                g.id
            )));
        }
        s.insert(g.id, g);
        Ok(())
    }
    async fn update_group(&self, g: RelationGroup) -> Result<(), RelationError> {
        let mut s = self.groups.write().expect("lock");
        s.insert(g.id, g);
        Ok(())
    }
    async fn get_group(&self, id: RelationGroupId) -> Result<RelationGroup, RelationError> {
        let s = self.groups.read().expect("lock");
        s.get(&id)
            .cloned()
            .ok_or_else(|| RelationError::NotFound(format!("group:{}", id)))
    }
}

// =====================================================================
// InMemoryRelationService(实现)
// =====================================================================

pub struct InMemoryRelationService {
    repo: Arc<dyn RelationRepository>,
    relations: Arc<RwLock<HashMap<RelationId, Relation>>>,
    groups: Arc<RwLock<HashMap<RelationGroupId, RelationGroup>>>,
    /// 按 (tenant_id, from_type, from_id) 索引,加速 from 查询
    from_index: Arc<RwLock<HashMap<(TenantId, ResourceType, Uuid), Vec<RelationId>>>>,
    /// 按 (tenant_id, to_type, to_id) 索引,加速 to 查询
    to_index: Arc<RwLock<HashMap<(TenantId, ResourceType, Uuid), Vec<RelationId>>>>,
    /// 重复检测 (tenant, from, type, to) -> RelationId
    dedup_index: Arc<
        RwLock<
            HashMap<
                (
                    TenantId,
                    ResourceType,
                    Uuid,
                    RelationType,
                    ResourceType,
                    Uuid,
                ),
                RelationId,
            >,
        >,
    >,
}

impl InMemoryRelationService {
    pub fn new() -> Self {
        Self {
            repo: Arc::new(InMemoryRelationRepository::new()),
            relations: Arc::new(RwLock::new(HashMap::new())),
            groups: Arc::new(RwLock::new(HashMap::new())),
            from_index: Arc::new(RwLock::new(HashMap::new())),
            to_index: Arc::new(RwLock::new(HashMap::new())),
            dedup_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    pub fn with_repo(repo: Arc<dyn RelationRepository>) -> Self {
        Self {
            repo,
            relations: Arc::new(RwLock::new(HashMap::new())),
            groups: Arc::new(RwLock::new(HashMap::new())),
            from_index: Arc::new(RwLock::new(HashMap::new())),
            to_index: Arc::new(RwLock::new(HashMap::new())),
            dedup_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 索引新创建的关系
    fn index_insert(&self, r: &Relation) {
        let from_key = (r.tenant_id, r.from_type, r.from_id);
        let to_key = (r.tenant_id, r.to_type, r.to_id);
        let dedup_key = (
            r.tenant_id,
            r.from_type,
            r.from_id,
            r.relation_type,
            r.to_type,
            r.to_id,
        );
        self.from_index
            .write()
            .expect("lock")
            .entry(from_key)
            .or_insert_with(Vec::new)
            .push(r.id);
        self.to_index
            .write()
            .expect("lock")
            .entry(to_key)
            .or_insert_with(Vec::new)
            .push(r.id);
        self.dedup_index
            .write()
            .expect("lock")
            .insert(dedup_key, r.id);
    }

    /// 取消索引(用于级联标记 / 删除)
    fn index_remove(&self, r: &Relation) {
        let from_key = (r.tenant_id, r.from_type, r.from_id);
        let to_key = (r.tenant_id, r.to_type, r.to_id);
        let dedup_key = (
            r.tenant_id,
            r.from_type,
            r.from_id,
            r.relation_type,
            r.to_type,
            r.to_id,
        );
        if let Some(v) = self.from_index.write().expect("lock").get_mut(&from_key) {
            v.retain(|id| *id != r.id);
        }
        if let Some(v) = self.to_index.write().expect("lock").get_mut(&to_key) {
            v.retain(|id| *id != r.id);
        }
        self.dedup_index.write().expect("lock").remove(&dedup_key);
    }
}

impl Default for InMemoryRelationService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RelationCommandPort for InMemoryRelationService {
    async fn create_relation(
        &self,
        cmd: CreateRelationCommand,
        actor: &ActorContext,
    ) -> Result<Relation, RelationError> {
        // INV-RL-04: 跨 tenant 拒绝
        if TenantId::from(actor.tenant_id) != cmd.tenant_id {
            return Err(RelationError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                cmd.tenant_id,
            ));
        }
        // INV-RL-02: 自关系禁止
        if cmd.from_type == cmd.to_type && cmd.from_id == cmd.to_id {
            return Err(RelationError::SelfRelation(cmd.from_type, cmd.from_id));
        }
        // 重复检测
        let dedup_key = (
            cmd.tenant_id,
            cmd.from_type,
            cmd.from_id,
            cmd.relation_type,
            cmd.to_type,
            cmd.to_id,
        );
        if self
            .dedup_index
            .read()
            .expect("lock")
            .contains_key(&dedup_key)
        {
            return Err(RelationError::Conflict(format!(
                "duplicate relation {:?} -> {:?} ({})",
                (cmd.from_type, cmd.from_id),
                (cmd.to_type, cmd.to_id),
                cmd.relation_type.as_str()
            )));
        }
        let rel = Relation {
            id: RelationId::new(),
            tenant_id: cmd.tenant_id,
            from_type: cmd.from_type,
            from_id: cmd.from_id,
            relation_type: cmd.relation_type,
            to_type: cmd.to_type,
            to_id: cmd.to_id,
            note: cmd.note,
            created_by: UserId::from(actor.user_id),
            created_at: Utc::now(),
            deleted: false,
        };
        // 持久化(可选,失败回滚)
        self.repo.insert_relation(rel.clone()).await.map_err(|e| {
            // 回滚内存
            self.dedup_index.write().expect("lock").remove(&dedup_key);
            e
        })?;
        // 内存 + 索引
        self.relations
            .write()
            .expect("lock")
            .insert(rel.id, rel.clone());
        self.index_insert(&rel);
        Ok(rel)
    }

    async fn delete_relation(
        &self,
        cmd: DeleteRelationCommand,
        actor: &ActorContext,
    ) -> Result<(), RelationError> {
        if TenantId::from(actor.tenant_id) != cmd.tenant_id {
            return Err(RelationError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                cmd.tenant_id,
            ));
        }
        let mut r = self
            .relations
            .write()
            .expect("lock")
            .get_mut(&cmd.relation_id)
            .cloned()
            .ok_or_else(|| RelationError::NotFound(format!("relation:{}", cmd.relation_id)))?;
        if r.tenant_id != cmd.tenant_id {
            return Err(RelationError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                r.tenant_id,
            ));
        }
        // INV-RL-05: 软删除
        r.deleted = true;
        self.repo.update_relation(r.clone()).await?;
        self.relations
            .write()
            .expect("lock")
            .insert(r.id, r.clone());
        self.index_remove(&r);
        Ok(())
    }

    async fn create_group(
        &self,
        cmd: CreateRelationGroupCommand,
        actor: &ActorContext,
    ) -> Result<RelationGroup, RelationError> {
        if TenantId::from(actor.tenant_id) != cmd.tenant_id {
            return Err(RelationError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                cmd.tenant_id,
            ));
        }
        if cmd.name.trim().is_empty() {
            return Err(RelationError::InvalidInput(
                "group name must not be empty".to_string(),
            ));
        }
        let g = RelationGroup::new(cmd.tenant_id, cmd.name, cmd.description, UserId::from(actor.user_id));
        self.repo.insert_group(g.clone()).await?;
        self.groups.write().expect("lock").insert(g.id, g.clone());
        Ok(g)
    }

    async fn add_to_group(
        &self,
        cmd: AddToGroupCommand,
        actor: &ActorContext,
    ) -> Result<RelationGroup, RelationError> {
        if TenantId::from(actor.tenant_id) != cmd.tenant_id {
            return Err(RelationError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                cmd.tenant_id,
            ));
        }
        // 关系存在性 + 跨租户
        let r = self
            .relations
            .read()
            .expect("lock")
            .get(&cmd.relation_id)
            .cloned()
            .ok_or_else(|| RelationError::NotFound(format!("relation:{}", cmd.relation_id)))?;
        if r.tenant_id != cmd.tenant_id {
            return Err(RelationError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                r.tenant_id,
            ));
        }
        let mut g = self
            .groups
            .read()
            .expect("lock")
            .get(&cmd.group_id)
            .cloned()
            .ok_or_else(|| RelationError::NotFound(format!("group:{}", cmd.group_id)))?;
        if g.tenant_id != cmd.tenant_id {
            return Err(RelationError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                g.tenant_id,
            ));
        }
        if !g.relation_ids.contains(&cmd.relation_id) {
            g.relation_ids.push(cmd.relation_id);
        }
        self.repo.update_group(g.clone()).await?;
        self.groups.write().expect("lock").insert(g.id, g.clone());
        Ok(g)
    }

    async fn remove_from_group(
        &self,
        cmd: AddToGroupCommand,
        actor: &ActorContext,
    ) -> Result<RelationGroup, RelationError> {
        if TenantId::from(actor.tenant_id) != cmd.tenant_id {
            return Err(RelationError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                cmd.tenant_id,
            ));
        }
        let mut g = self
            .groups
            .read()
            .expect("lock")
            .get(&cmd.group_id)
            .cloned()
            .ok_or_else(|| RelationError::NotFound(format!("group:{}", cmd.group_id)))?;
        if g.tenant_id != cmd.tenant_id {
            return Err(RelationError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                g.tenant_id,
            ));
        }
        let before = g.relation_ids.len();
        g.relation_ids.retain(|id| *id != cmd.relation_id);
        if g.relation_ids.len() == before {
            return Err(RelationError::NotFound(format!(
                "relation {} in group",
                cmd.relation_id
            )));
        }
        self.repo.update_group(g.clone()).await?;
        self.groups.write().expect("lock").insert(g.id, g.clone());
        Ok(g)
    }
}

#[async_trait]
impl RelationQueryPort for InMemoryRelationService {
    async fn get_relation(
        &self,
        id: RelationId,
        actor: &ActorContext,
    ) -> Result<Relation, RelationError> {
        let r = self
            .relations
            .read()
            .expect("lock")
            .get(&id)
            .cloned()
            .ok_or_else(|| RelationError::NotFound(format!("relation:{}", id)))?;
        if r.tenant_id != TenantId::from(actor.tenant_id) {
            return Err(RelationError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                r.tenant_id,
            ));
        }
        Ok(r)
    }

    async fn list_from(
        &self,
        q: ListByEndpointQuery,
        actor: &ActorContext,
    ) -> Result<Vec<Relation>, RelationError> {
        if TenantId::from(actor.tenant_id) != q.tenant_id {
            return Err(RelationError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                q.tenant_id,
            ));
        }
        if matches!(q.from_only, Some(false)) {
            return Err(RelationError::InvalidInput(
                "list_from called with from_only=false; use list_to".to_string(),
            ));
        }
        let key = (q.tenant_id, q.resource_type, q.resource_id);
        let ids = self
            .from_index
            .read()
            .expect("lock")
            .get(&key)
            .cloned()
            .unwrap_or_default();
        let relations = self.relations.read().expect("lock");
        Ok(ids
            .iter()
            .filter_map(|id| relations.get(id).cloned())
            .filter(|r| !r.deleted)
            .collect())
    }

    async fn list_to(
        &self,
        q: ListByEndpointQuery,
        actor: &ActorContext,
    ) -> Result<Vec<Relation>, RelationError> {
        if TenantId::from(actor.tenant_id) != q.tenant_id {
            return Err(RelationError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                q.tenant_id,
            ));
        }
        if matches!(q.from_only, Some(true)) {
            return Err(RelationError::InvalidInput(
                "list_to called with from_only=true; use list_from".to_string(),
            ));
        }
        let key = (q.tenant_id, q.resource_type, q.resource_id);
        let ids = self
            .to_index
            .read()
            .expect("lock")
            .get(&key)
            .cloned()
            .unwrap_or_default();
        let relations = self.relations.read().expect("lock");
        Ok(ids
            .iter()
            .filter_map(|id| relations.get(id).cloned())
            .filter(|r| !r.deleted)
            .collect())
    }

    async fn list_by_type(
        &self,
        q: ListByTypeQuery,
        actor: &ActorContext,
    ) -> Result<Vec<Relation>, RelationError> {
        if TenantId::from(actor.tenant_id) != q.tenant_id {
            return Err(RelationError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                q.tenant_id,
            ));
        }
        let relations = self.relations.read().expect("lock");
        Ok(relations
            .values()
            .filter(|r| {
                r.tenant_id == q.tenant_id && r.relation_type == q.relation_type && !r.deleted
            })
            .cloned()
            .collect())
    }

    async fn get_group(
        &self,
        group_id: RelationGroupId,
        actor: &ActorContext,
    ) -> Result<RelationGroup, RelationError> {
        let g = self
            .groups
            .read()
            .expect("lock")
            .get(&group_id)
            .cloned()
            .ok_or_else(|| RelationError::NotFound(format!("group:{}", group_id)))?;
        if g.tenant_id != TenantId::from(actor.tenant_id) {
            return Err(RelationError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                g.tenant_id,
            ));
        }
        Ok(g)
    }

    async fn get_graph(
        &self,
        q: GetGraphQuery,
        actor: &ActorContext,
    ) -> Result<RelationGraph, RelationError> {
        if TenantId::from(actor.tenant_id) != q.tenant_id {
            return Err(RelationError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                q.tenant_id,
            ));
        }
        let depth = q.depth.max(1);
        let relations = self.relations.read().expect("lock");

        // BFS — 同时记录 visited (type, id) 避免循环(INV-RL-02 自关系被前置拒绝,跨实体循环仍可能出现)
        let mut visited: HashSet<(ResourceType, Uuid)> = HashSet::new();
        let mut nodes: Vec<GraphNode> = Vec::new();
        let mut edges: Vec<GraphEdge> = Vec::new();
        let mut edge_seen: HashSet<RelationId> = HashSet::new();

        let mut queue: VecDeque<(ResourceType, Uuid, u32)> = VecDeque::new();
        let root = GraphNode {
            resource_type: q.resource_type,
            resource_id: q.resource_id,
        };
        visited.insert((q.resource_type, q.resource_id));
        nodes.push(root.clone());
        queue.push_back((q.resource_type, q.resource_id, 0));

        while let Some((cur_type, cur_id, cur_depth)) = queue.pop_front() {
            if cur_depth >= depth {
                continue;
            }
            // 找所有以 cur 为 from 或 to 的关系
            let from_ids = self
                .from_index
                .read()
                .expect("lock")
                .get(&(q.tenant_id, cur_type, cur_id))
                .cloned()
                .unwrap_or_default();
            let to_ids = self
                .to_index
                .read()
                .expect("lock")
                .get(&(q.tenant_id, cur_type, cur_id))
                .cloned()
                .unwrap_or_default();
            let mut all_ids: Vec<RelationId> = from_ids.into_iter().chain(to_ids).collect();
            // 去重(同一关系可能 from/to 都命中)
            all_ids.sort();
            all_ids.dedup();
            for rid in all_ids {
                let Some(r) = relations.get(&rid).cloned() else {
                    continue;
                };
                if r.deleted {
                    continue;
                }
                if let Some(ft) = q.filter_type {
                    if r.relation_type != ft {
                        continue;
                    }
                }
                // 边按 relation_id 去重(BFS 从两端都到达时同一关系只记一次)
                if !edge_seen.insert(rid) {
                    continue;
                }
                let from_node = GraphNode {
                    resource_type: r.from_type,
                    resource_id: r.from_id,
                };
                let to_node = GraphNode {
                    resource_type: r.to_type,
                    resource_id: r.to_id,
                };
                edges.push(GraphEdge {
                    relation_id: r.id,
                    from: from_node.clone(),
                    to: to_node.clone(),
                    relation_type: r.relation_type,
                });
                // 把"对端"入队下一层
                let other = if r.from_type == cur_type && r.from_id == cur_id {
                    Some((r.to_type, r.to_id))
                } else if r.to_type == cur_type && r.to_id == cur_id {
                    Some((r.from_type, r.from_id))
                } else {
                    None
                };
                if let Some((ot, oid)) = other {
                    if !visited.contains(&(ot, oid)) {
                        visited.insert((ot, oid));
                        nodes.push(GraphNode {
                            resource_type: ot,
                            resource_id: oid,
                        });
                        queue.push_back((ot, oid, cur_depth + 1));
                    }
                }
            }
        }

        Ok(RelationGraph {
            root,
            nodes,
            edges,
            depth,
            generated_at: Utc::now(),
        })
    }
}

// =====================================================================
// 辅助:端点比较
// =====================================================================

#[inline]
fn endpoint_eq(a: (ResourceType, Uuid), b: (ResourceType, Uuid)) -> bool {
    a.0 == b.0 && a.1 == b.1
}

// =====================================================================
// Tests(>= 12 用例)
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    fn tenant_a() -> TenantId {
        uuid::Uuid::new_v4()
    }
    fn tenant_b() -> TenantId {
        uuid::Uuid::new_v4()
    }
    fn user_a() -> UserId {
        uuid::Uuid::new_v4()
    }
    fn actor(tenant_id: TenantId) -> ActorContext {
        ActorContext::new(user_a(), tenant_id)
    }

    // 1. 基本创建
    #[tokio::test]
    async fn create_relation_basic() {
        let svc = InMemoryRelationService::new();
        let tid = tenant_a();
        let from = Uuid::new_v4();
        let to = Uuid::new_v4();
        let cmd = CreateRelationCommand {
            tenant_id: tid,
            from_type: ResourceType::WorkItem,
            from_id: from,
            relation_type: RelationType::Blocks,
            to_type: ResourceType::WorkItem,
            to_id: to,
            note: Some("blocker".to_string()),
        };
        let r = svc.create_relation(cmd, &actor(tid)).await.unwrap();
        assert_eq!(r.tenant_id, tid);
        assert_eq!(r.from_type, ResourceType::WorkItem);
        assert_eq!(r.from_id, from);
        assert_eq!(r.to_type, ResourceType::WorkItem);
        assert_eq!(r.to_id, to);
        assert_eq!(r.relation_type, RelationType::Blocks);
        assert_eq!(r.note.as_deref(), Some("blocker"));
        assert!(!r.deleted);
    }

    // 2. 自关系拒绝
    #[tokio::test]
    async fn create_self_relation_rejected() {
        let svc = InMemoryRelationService::new();
        let tid = tenant_a();
        let same = Uuid::new_v4();
        let cmd = CreateRelationCommand {
            tenant_id: tid,
            from_type: ResourceType::WorkItem,
            from_id: same,
            relation_type: RelationType::Blocks,
            to_type: ResourceType::WorkItem,
            to_id: same,
            note: None,
        };
        let err = svc.create_relation(cmd, &actor(tid)).await.unwrap_err();
        assert!(matches!(err, RelationError::SelfRelation(_, _)));
    }

    // 3. 跨类型允许(WorkItem -> Symbol,INV-RL-03)
    #[tokio::test]
    async fn cross_type_relation_allowed() {
        let svc = InMemoryRelationService::new();
        let tid = tenant_a();
        let cmd = CreateRelationCommand {
            tenant_id: tid,
            from_type: ResourceType::WorkItem,
            from_id: Uuid::new_v4(),
            relation_type: RelationType::References,
            to_type: ResourceType::Symbol,
            to_id: Uuid::new_v4(),
            note: None,
        };
        let r = svc.create_relation(cmd, &actor(tid)).await.unwrap();
        assert_eq!(r.from_type, ResourceType::WorkItem);
        assert_eq!(r.to_type, ResourceType::Symbol);
        assert_eq!(r.relation_type, RelationType::References);
    }

    // 4. 重复关系拒绝
    #[tokio::test]
    async fn create_duplicate_rejected() {
        let svc = InMemoryRelationService::new();
        let tid = tenant_a();
        let from = Uuid::new_v4();
        let to = Uuid::new_v4();
        let mk = || CreateRelationCommand {
            tenant_id: tid,
            from_type: ResourceType::WorkItem,
            from_id: from,
            relation_type: RelationType::Blocks,
            to_type: ResourceType::Project,
            to_id: to,
            note: None,
        };
        svc.create_relation(mk(), &actor(tid)).await.unwrap();
        let err = svc.create_relation(mk(), &actor(tid)).await.unwrap_err();
        assert!(matches!(err, RelationError::Conflict(_)));
    }

    // 5. list_from
    #[tokio::test]
    async fn list_from_resource() {
        let svc = InMemoryRelationService::new();
        let tid = tenant_a();
        let from = Uuid::new_v4();
        let to_a = Uuid::new_v4();
        let to_b = Uuid::new_v4();
        for to in [to_a, to_b] {
            svc.create_relation(
                CreateRelationCommand {
                    tenant_id: tid,
                    from_type: ResourceType::WorkItem,
                    from_id: from,
                    relation_type: RelationType::References,
                    to_type: ResourceType::Symbol,
                    to_id: to,
                    note: None,
                },
                &actor(tid),
            )
            .await
            .unwrap();
        }
        // 反向关系不应出现在 list_from
        svc.create_relation(
            CreateRelationCommand {
                tenant_id: tid,
                from_type: ResourceType::WorkItem,
                from_id: to_a,
                relation_type: RelationType::Blocks,
                to_type: ResourceType::WorkItem,
                to_id: from,
                note: None,
            },
            &actor(tid),
        )
        .await
        .unwrap();

        let q = ListByEndpointQuery {
            tenant_id: tid,
            resource_type: ResourceType::WorkItem,
            resource_id: from,
            from_only: None,
        };
        let out = svc.list_from(q, &actor(tid)).await.unwrap();
        assert_eq!(out.len(), 2);
        for r in &out {
            assert_eq!(r.from_id, from);
        }
    }

    // 6. list_to
    #[tokio::test]
    async fn list_to_resource() {
        let svc = InMemoryRelationService::new();
        let tid = tenant_a();
        let target = Uuid::new_v4();
        for _ in 0..3 {
            svc.create_relation(
                CreateRelationCommand {
                    tenant_id: tid,
                    from_type: ResourceType::Decision,
                    from_id: Uuid::new_v4(),
                    relation_type: RelationType::Implements,
                    to_type: ResourceType::Project,
                    to_id: target,
                    note: None,
                },
                &actor(tid),
            )
            .await
            .unwrap();
        }
        let q = ListByEndpointQuery {
            tenant_id: tid,
            resource_type: ResourceType::Project,
            resource_id: target,
            from_only: None,
        };
        let out = svc.list_to(q, &actor(tid)).await.unwrap();
        assert_eq!(out.len(), 3);
        for r in &out {
            assert_eq!(r.to_id, target);
        }
    }

    // 7. list_by_relation_type
    #[tokio::test]
    async fn list_by_relation_type() {
        let svc = InMemoryRelationService::new();
        let tid = tenant_a();
        // 3 Blocks, 1 References
        for _ in 0..3 {
            svc.create_relation(
                CreateRelationCommand {
                    tenant_id: tid,
                    from_type: ResourceType::WorkItem,
                    from_id: Uuid::new_v4(),
                    relation_type: RelationType::Blocks,
                    to_type: ResourceType::WorkItem,
                    to_id: Uuid::new_v4(),
                    note: None,
                },
                &actor(tid),
            )
            .await
            .unwrap();
        }
        svc.create_relation(
            CreateRelationCommand {
                tenant_id: tid,
                from_type: ResourceType::WorkItem,
                from_id: Uuid::new_v4(),
                relation_type: RelationType::References,
                to_type: ResourceType::Symbol,
                to_id: Uuid::new_v4(),
                note: None,
            },
            &actor(tid),
        )
        .await
        .unwrap();

        let blocks = svc
            .list_by_type(
                ListByTypeQuery {
                    tenant_id: tid,
                    relation_type: RelationType::Blocks,
                },
                &actor(tid),
            )
            .await
            .unwrap();
        assert_eq!(blocks.len(), 3);
        let refs = svc
            .list_by_type(
                ListByTypeQuery {
                    tenant_id: tid,
                    relation_type: RelationType::References,
                },
                &actor(tid),
            )
            .await
            .unwrap();
        assert_eq!(refs.len(), 1);
    }

    // 8. delete_relation 软删
    #[tokio::test]
    async fn delete_relation() {
        let svc = InMemoryRelationService::new();
        let tid = tenant_a();
        let r = svc
            .create_relation(
                CreateRelationCommand {
                    tenant_id: tid,
                    from_type: ResourceType::WorkItem,
                    from_id: Uuid::new_v4(),
                    relation_type: RelationType::Duplicates,
                    to_type: ResourceType::WorkItem,
                    to_id: Uuid::new_v4(),
                    note: None,
                },
                &actor(tid),
            )
            .await
            .unwrap();
        svc.delete_relation(
            DeleteRelationCommand {
                tenant_id: tid,
                relation_id: r.id,
            },
            &actor(tid),
        )
        .await
        .unwrap();
        // 查询 list_from 不应再出现
        let q = ListByEndpointQuery {
            tenant_id: tid,
            resource_type: ResourceType::WorkItem,
            resource_id: r.from_id,
            from_only: None,
        };
        let out = svc.list_from(q, &actor(tid)).await.unwrap();
        assert!(out.is_empty());
    }

    // 9. 跨 tenant 拒绝(INV-RL-04)
    #[tokio::test]
    async fn cross_tenant_denied() {
        let svc = InMemoryRelationService::new();
        let ta = tenant_a();
        let tb = tenant_b();
        // actor 在 tenant_a,cmd 指向 tenant_b
        let cmd = CreateRelationCommand {
            tenant_id: tb,
            from_type: ResourceType::WorkItem,
            from_id: Uuid::new_v4(),
            relation_type: RelationType::Blocks,
            to_type: ResourceType::WorkItem,
            to_id: Uuid::new_v4(),
            note: None,
        };
        let err = svc.create_relation(cmd, &actor(ta)).await.unwrap_err();
        assert!(matches!(err, RelationError::CrossTenantDenied(_, _)));
    }

    // 10. 分组创建
    #[tokio::test]
    async fn relation_group_create() {
        let svc = InMemoryRelationService::new();
        let tid = tenant_a();
        let g = svc
            .create_group(
                CreateRelationGroupCommand {
                    tenant_id: tid,
                    name: "release-blockers".to_string(),
                    description: "release window blockers".to_string(),
                },
                &actor(tid),
            )
            .await
            .unwrap();
        assert_eq!(g.tenant_id, tid);
        assert_eq!(g.name, "release-blockers");
        assert!(g.relation_ids.is_empty());
    }

    // 11. add_to_group
    #[tokio::test]
    async fn add_to_group() {
        let svc = InMemoryRelationService::new();
        let tid = tenant_a();
        let r = svc
            .create_relation(
                CreateRelationCommand {
                    tenant_id: tid,
                    from_type: ResourceType::WorkItem,
                    from_id: Uuid::new_v4(),
                    relation_type: RelationType::Blocks,
                    to_type: ResourceType::WorkItem,
                    to_id: Uuid::new_v4(),
                    note: None,
                },
                &actor(tid),
            )
            .await
            .unwrap();
        let g = svc
            .create_group(
                CreateRelationGroupCommand {
                    tenant_id: tid,
                    name: "g1".to_string(),
                    description: "".to_string(),
                },
                &actor(tid),
            )
            .await
            .unwrap();
        let g2 = svc
            .add_to_group(
                AddToGroupCommand {
                    tenant_id: tid,
                    group_id: g.id,
                    relation_id: r.id,
                },
                &actor(tid),
            )
            .await
            .unwrap();
        assert!(g2.relation_ids.contains(&r.id));
        // 重复添加幂等
        let g3 = svc
            .add_to_group(
                AddToGroupCommand {
                    tenant_id: tid,
                    group_id: g.id,
                    relation_id: r.id,
                },
                &actor(tid),
            )
            .await
            .unwrap();
        assert_eq!(g3.relation_ids.len(), 1);
    }

    // 12. get_graph 1 hop
    #[tokio::test]
    async fn get_graph_1_hop() {
        let svc = InMemoryRelationService::new();
        let tid = tenant_a();
        let root = Uuid::new_v4();
        let n1 = Uuid::new_v4();
        let n2 = Uuid::new_v4();
        // root -> n1 (Blocks)
        svc.create_relation(
            CreateRelationCommand {
                tenant_id: tid,
                from_type: ResourceType::WorkItem,
                from_id: root,
                relation_type: RelationType::Blocks,
                to_type: ResourceType::WorkItem,
                to_id: n1,
                note: None,
            },
            &actor(tid),
        )
        .await
        .unwrap();
        // n2 -> root (Blocks) — root 仍是 1 hop
        svc.create_relation(
            CreateRelationCommand {
                tenant_id: tid,
                from_type: ResourceType::WorkItem,
                from_id: n2,
                relation_type: RelationType::Blocks,
                to_type: ResourceType::WorkItem,
                to_id: root,
                note: None,
            },
            &actor(tid),
        )
        .await
        .unwrap();
        // 无关关系不应被收录
        svc.create_relation(
            CreateRelationCommand {
                tenant_id: tid,
                from_type: ResourceType::WorkItem,
                from_id: Uuid::new_v4(),
                relation_type: RelationType::Blocks,
                to_type: ResourceType::WorkItem,
                to_id: Uuid::new_v4(),
                note: None,
            },
            &actor(tid),
        )
        .await
        .unwrap();

        let q = GetGraphQuery {
            tenant_id: tid,
            resource_type: ResourceType::WorkItem,
            resource_id: root,
            depth: 1,
            filter_type: None,
        };
        let g = svc.get_graph(q, &actor(tid)).await.unwrap();
        // 1 hop 节点 = root + n1 + n2 = 3
        assert_eq!(g.nodes.len(), 3);
        // 边 = 2
        assert_eq!(g.edges.len(), 2);
        // 根节点是 root
        assert_eq!(g.root.resource_id, root);
    }

    // 13 (额外) get_graph 2 hop
    #[tokio::test]
    async fn get_graph_2_hop() {
        let svc = InMemoryRelationService::new();
        let tid = tenant_a();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();
        // a -> b -> c
        svc.create_relation(
            CreateRelationCommand {
                tenant_id: tid,
                from_type: ResourceType::WorkItem,
                from_id: a,
                relation_type: RelationType::Blocks,
                to_type: ResourceType::WorkItem,
                to_id: b,
                note: None,
            },
            &actor(tid),
        )
        .await
        .unwrap();
        svc.create_relation(
            CreateRelationCommand {
                tenant_id: tid,
                from_type: ResourceType::WorkItem,
                from_id: b,
                relation_type: RelationType::Blocks,
                to_type: ResourceType::WorkItem,
                to_id: c,
                note: None,
            },
            &actor(tid),
        )
        .await
        .unwrap();
        let q = GetGraphQuery {
            tenant_id: tid,
            resource_type: ResourceType::WorkItem,
            resource_id: a,
            depth: 2,
            filter_type: None,
        };
        let g = svc.get_graph(q, &actor(tid)).await.unwrap();
        // a, b, c — 3 节点
        assert_eq!(g.nodes.len(), 3);
        assert_eq!(g.edges.len(), 2);
    }

    // 14 (额外) get_graph filter_type
    #[tokio::test]
    async fn get_graph_filter_type() {
        let svc = InMemoryRelationService::new();
        let tid = tenant_a();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();
        svc.create_relation(
            CreateRelationCommand {
                tenant_id: tid,
                from_type: ResourceType::WorkItem,
                from_id: a,
                relation_type: RelationType::Blocks,
                to_type: ResourceType::WorkItem,
                to_id: b,
                note: None,
            },
            &actor(tid),
        )
        .await
        .unwrap();
        svc.create_relation(
            CreateRelationCommand {
                tenant_id: tid,
                from_type: ResourceType::WorkItem,
                from_id: a,
                relation_type: RelationType::References,
                to_type: ResourceType::WorkItem,
                to_id: c,
                note: None,
            },
            &actor(tid),
        )
        .await
        .unwrap();
        let q = GetGraphQuery {
            tenant_id: tid,
            resource_type: ResourceType::WorkItem,
            resource_id: a,
            depth: 1,
            filter_type: Some(RelationType::Blocks),
        };
        let g = svc.get_graph(q, &actor(tid)).await.unwrap();
        // 只 1 边 Blocks
        assert_eq!(g.edges.len(), 1);
        assert_eq!(g.edges[0].relation_type, RelationType::Blocks);
    }

    // 15 (额外) endpoint_eq 工具
    #[test]
    fn endpoint_eq_helper() {
        let u = Uuid::new_v4();
        assert!(endpoint_eq(
            (ResourceType::WorkItem, u),
            (ResourceType::WorkItem, u)
        ));
        assert!(!endpoint_eq(
            (ResourceType::WorkItem, u),
            (ResourceType::Symbol, u)
        ));
    }

    // 16 (额外) ResourceType / RelationType 解析
    #[test]
    fn type_parse_roundtrip() {
        assert_eq!(
            ResourceType::parse("work_item"),
            Some(ResourceType::WorkItem)
        );
        assert_eq!(ResourceType::parse("REPO"), Some(ResourceType::Repository));
        assert_eq!(ResourceType::parse("nope"), None);
        assert_eq!(RelationType::parse("blocks"), Some(RelationType::Blocks));
        assert_eq!(RelationType::parse("SIBLING"), Some(RelationType::Sibling));
        assert_eq!(RelationType::parse("nope"), None);
    }
}
