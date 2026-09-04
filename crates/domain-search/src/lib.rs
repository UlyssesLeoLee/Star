//! domain-search crate
//!
//! 详细 spec: docs/specs/domain-search-spec.md §14 Search Projection
//! 上游基本设计: docs/basic-design.md §2.1 / §3 ACL / §5.7
//! 数据设计: docs/data-design.md §4.15 (`search_index` schema)
//! API 设计: docs/api-design.md §3.18 (Search Query)
//!
//! ## 职责
//!
//! 全文 / 符号检索 Projection(§12,REQ-SEARCH-001)
//! **不**成为业务实体的真实数据源,定位为查询视图(§2.1 §14 注 4)。
//!
//! ## 关键不变量(INV-S-01~06)
//!
//! - INV-S-01:Search 不成为业务实体的真实数据源(REQ-SEARCH-001 强约束)
//! - INV-S-02:SearchIndex 由 Worker 异步投影,**不**由业务事务直接写(§5.7,§2.1)
//! - INV-S-03:SearchIndex 必带 tenant_id,跨 tenant 拒绝(§6.1)
//! - INV-S-04:SearchQuery 7 类定型 SoR(§5.8 草稿)
//! - INV-S-05:SavedSearch 仅本人可读 / 写 / 删(私有)(§4.10)
//! - INV-S-06:Search 严格只读 Projection,POST /v1/search 不写入业务实体(§3.11,REQ-SEARCH-001)
//!
//! Lead 责任: search Lead

use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
pub use star_context::ActorContext;
use thiserror::Error;
use uuid::Uuid;

// =====================================================================
// ID 类型
// =====================================================================

define_uuid_id!(SearchIndexId);
define_uuid_id!(SavedSearchId);
define_uuid_id!(TenantId);
define_uuid_id!(ProjectId);
define_uuid_id!(UserId);
define_uuid_id!(WorkItemId);

// =====================================================================
// 资源类型(Projection 来源)
// =====================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// 可被检索的资源类型(Projection 来源)
pub enum ResourceType {
    /// 工作项
    WorkItem,
    /// 评论
    Comment,
    /// 项目
    Project,
    /// 代码符号
    Symbol,
    /// 反馈
    Feedback,
    /// 决策记录
    Decision,
    /// 架构决策记录(ADR)
    Adr,
}

impl ResourceType {
    /// 返回资源类型对应的小写字符串标识
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::WorkItem => "work_item",
            Self::Comment => "comment",
            Self::Project => "project",
            Self::Symbol => "symbol",
            Self::Feedback => "feedback",
            Self::Decision => "decision",
            Self::Adr => "adr",
        }
    }
}

// =====================================================================
// UUID 强类型 ID 宏
// =====================================================================

#[macro_export]
/// 生成基于 UUID 的领域强类型 ID(附带 new/as_uuid/From/Display 实现)
macro_rules! define_uuid_id {
    ($name:ident) => {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
        )]
        #[serde(transparent)]
        /// 领域强类型 ID(由宏统一生成)
        pub struct $name(pub Uuid);

        impl $name {
            /// 生成一个新的随机 ID(由宏统一生成)
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }
            /// 返回底层 UUID(由宏统一生成)
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
// 实体 — SearchIndex(Projection,只读)
// =====================================================================

/// SearchIndex 投影(§4.15,§12)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchIndex {
    /// SearchIndex 记录 ID
    pub id: SearchIndexId,
    /// 所属租户 ID
    pub tenant_id: TenantId,
    /// 所属项目 ID
    pub project_id: ProjectId,
    /// 索引来源的资源类型
    pub resource_type: ResourceType,
    /// 索引来源的资源 ID
    pub resource_id: Uuid,
    /// 全文检索文本
    pub fulltext: String,
    /// 代码符号元数据(仅 Symbol 类型资源有值)
    pub symbol_metadata: Option<SymbolMetadata>,
    /// 标签列表
    pub tags: Vec<String>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 最近更新时间
    pub updated_at: DateTime<Utc>,
    /// 乐观版本(用于 Projector 重放)
    pub version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 代码符号元数据(用于符号检索场景)
pub struct SymbolMetadata {
    /// 符号名称
    pub name: String,
    /// 符号种类(如 function/struct/trait)
    pub kind: String,
    /// 符号签名
    pub signature: String,
    /// 符号所在文件路径
    pub file_path: String,
}

// =====================================================================
// 实体 — SearchQuery
// =====================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 全文检索查询参数
pub struct SearchQuery {
    /// 查询文本
    pub query_text: String,
    /// 过滤条件(键值对)
    pub filters: HashMap<String, String>,
    /// 排序字段
    pub sort: Option<String>,
    /// 返回数量上限
    pub limit: u32,
    /// 分页偏移量
    pub offset: u32,
    /// 发起查询的用户 ID
    pub user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 检索结果集
pub struct SearchResult {
    /// 命中总数
    pub total: u64,
    /// 命中条目列表
    pub items: Vec<SearchHit>,
    /// 分面统计
    pub facets: HashMap<String, Vec<Facet>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 单条检索命中结果
pub struct SearchHit {
    /// 命中资源类型
    pub resource_type: ResourceType,
    /// 命中资源 ID
    pub resource_id: Uuid,
    /// 相关性得分
    pub score: f64,
    /// 高亮片段(字段名 -> 高亮文本)
    pub highlights: HashMap<String, String>,
    /// 所属租户 ID
    pub tenant_id: TenantId,
    /// 所属项目 ID
    pub project_id: ProjectId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 分面统计项
pub struct Facet {
    /// 分面取值
    pub value: String,
    /// 该取值下的命中数量
    pub count: u64,
}

// =====================================================================
// 实体 — P1 工具链 DTO (per docs/briefs/tool-p1-impl-001.md §1.2-1.4)
// =====================================================================

/// 单个符号的引用结果(per §1.2 get_symbol)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolRef {
    pub name: String,
    pub kind: String,
    pub file_path: String,
    /// P0 简化:SearchIndex 的 SymbolMetadata 不带 line, 用 0 占位
    /// (per brief §1.2 "不改 SearchRepository, 走 InMemory cache 真实路径")
    pub line: u32,
    pub signature: String,
}

/// 单个引用位置(per §1.3 find_references)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceRef {
    pub name: String,
    pub file_path: String,
    /// P0 简化:行号 = SearchIndex fulltext 中的偏移估算
    pub line: u32,
    /// P0 简化:列号 = 0 占位
    pub column: u32,
    pub context: String,
}

/// 代码上下文窗口(per §1.4 get_code_context)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeContext {
    pub file_path: String,
    pub start_line: u32,
    pub end_line: u32,
    pub snippet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 自动补全查询参数
pub struct SuggestQuery {
    /// 补全前缀
    pub prefix: String,
    /// 返回数量上限
    pub limit: u32,
    /// 发起查询的用户 ID
    pub user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 自动补全建议项
pub struct Suggestion {
    /// 建议展示文本
    pub text: String,
    /// 建议来源资源类型
    pub resource_type: ResourceType,
    /// 建议来源资源 ID
    pub resource_id: Uuid,
}

// =====================================================================
// 实体 — SavedSearch
// =====================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 用户保存的检索条件(仅本人可见,§4.10)
pub struct SavedSearch {
    /// SavedSearch 记录 ID
    pub id: SavedSearchId,
    /// 所属租户 ID
    pub tenant_id: TenantId,
    /// 所属用户 ID
    pub user_id: UserId,
    /// 保存时使用的名称
    pub name: String,
    /// 保存的查询条件
    pub query: SearchQuery,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

// =====================================================================
// 错误
// =====================================================================

#[derive(Debug, Error)]
/// Search 领域操作错误
pub enum SearchError {
    #[error("not found: {0}")]
    /// 目标资源不存在
    NotFound(String),
    #[error("invalid state: {0}")]
    /// 当前状态不允许该操作
    InvalidState(String),
    #[error("permission denied")]
    /// 权限不足
    PermissionDenied,
    /// INV-SR-02 / INV-S-03:跨 tenant 拒绝
    #[error("cross-tenant access denied: tenant {0} vs required {1}")]
    CrossTenantDenied(TenantId, TenantId),
    #[error("invalid query: {0}")]
    /// 查询参数非法
    InvalidQuery(String),
    #[error("conflict: {0}")]
    /// 操作与当前状态冲突
    Conflict(String),
    #[error("internal: {0}")]
    /// 内部错误
    Internal(String),
}

// =====================================================================
// 命令 / 查询 DTO
// =====================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 写入 / 更新 SearchIndex 的命令(仅 Worker Projector 可调用)
pub struct UpsertIndexCommand {
    /// 所属租户 ID
    pub tenant_id: TenantId,
    /// 所属项目 ID
    pub project_id: ProjectId,
    /// 索引来源的资源类型
    pub resource_type: ResourceType,
    /// 索引来源的资源 ID
    pub resource_id: Uuid,
    /// 全文检索文本
    pub fulltext: String,
    /// 代码符号元数据(仅 Symbol 类型资源有值)
    pub symbol_metadata: Option<SymbolMetadata>,
    /// 标签列表
    pub tags: Vec<String>,
    /// 投影版本号(用于幂等 / 乱序丢弃)
    pub projection_version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 删除 SearchIndex 的命令(仅 Worker Projector 可调用)
pub struct DeleteIndexCommand {
    /// 所属租户 ID
    pub tenant_id: TenantId,
    /// 待删除索引对应的资源类型
    pub resource_type: ResourceType,
    /// 待删除索引对应的资源 ID
    pub resource_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 批量重建索引的命令
pub struct BulkReindexCommand {
    /// 所属租户 ID
    pub tenant_id: TenantId,
    /// 所属项目 ID
    pub project_id: ProjectId,
    /// 待写入的索引条目列表
    pub entries: Vec<UpsertIndexCommand>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 批量重建索引的结果统计
pub struct BulkReindexResult {
    /// 新增条目数
    pub inserted: u32,
    /// 更新条目数
    pub updated: u32,
    /// 跳过条目数(版本落后)
    pub skipped: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 保存 SavedSearch 的命令
pub struct SaveSearchCommand {
    /// 所属租户 ID
    pub tenant_id: TenantId,
    /// 保存所属的用户 ID
    pub user_id: UserId,
    /// 保存时使用的名称
    pub name: String,
    /// 待保存的查询条件
    pub query: SearchQuery,
    /// 发起操作的实际用户 ID
    pub actor_user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 删除 SavedSearch 的命令(INV-S-05:仅本人可删)
pub struct DeleteSavedSearchCommand {
    /// 所属租户 ID
    pub tenant_id: TenantId,
    /// 待删除的 SavedSearch ID
    pub saved_search_id: SavedSearchId,
    /// 发起操作的实际用户 ID
    pub actor_user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 全文检索请求 DTO(携带租户上下文)
pub struct SearchQueryDto {
    /// 所属租户 ID
    pub tenant_id: TenantId,
    /// 检索查询参数
    pub query: SearchQuery,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 自动补全请求 DTO(携带租户上下文)
pub struct SuggestQueryDto {
    /// 所属租户 ID
    pub tenant_id: TenantId,
    /// 补全查询参数
    pub query: SuggestQuery,
}

// =====================================================================
// 端口(Port Traits)
// =====================================================================

/// SearchCommandPort(INV-S-02:仅由 Worker Projector 调用,非业务路径)
#[async_trait]
pub trait SearchCommandPort: Send + Sync {
    /// Worker 投影写入(由 infrastructure Adapter 调用,**不**经业务事务)
    async fn upsert_index(
        &self,
        cmd: UpsertIndexCommand,
        actor: &ActorContext,
    ) -> Result<SearchIndex, SearchError>;

    /// 删除单条索引(由 infrastructure Adapter 调用,**不**经业务事务)
    async fn delete_index(
        &self,
        cmd: DeleteIndexCommand,
        actor: &ActorContext,
    ) -> Result<(), SearchError>;

    /// 批量重建索引
    async fn bulk_reindex(
        &self,
        cmd: BulkReindexCommand,
        actor: &ActorContext,
    ) -> Result<BulkReindexResult, SearchError>;

    /// 保存检索条件为 SavedSearch
    async fn save_search(
        &self,
        cmd: SaveSearchCommand,
        actor: &ActorContext,
    ) -> Result<SavedSearch, SearchError>;

    /// 删除本人的 SavedSearch(INV-S-05)
    async fn delete_saved(
        &self,
        cmd: DeleteSavedSearchCommand,
        actor: &ActorContext,
    ) -> Result<(), SearchError>;
}

#[async_trait]
/// SearchQueryPort(只读检索入口,§14 注 4)
pub trait SearchQueryPort: Send + Sync {
    /// 全文检索(INV-SR-02 强制 tenant 隔离)
    async fn search(
        &self,
        q: SearchQueryDto,
        actor: &ActorContext,
    ) -> Result<SearchResult, SearchError>;

    /// 自动补全
    async fn suggest(
        &self,
        q: SuggestQueryDto,
        actor: &ActorContext,
    ) -> Result<Vec<Suggestion>, SearchError>;

    /// 列出本人 SavedSearch
    async fn list_saved(
        &self,
        tenant_id: TenantId,
        actor: &ActorContext,
    ) -> Result<Vec<SavedSearch>, SearchError>;

    // ===== P1 工具链 (per docs/briefs/tool-p1-impl-001.md §1.2-1.4) =====
    // 守门 #13 a L0 协调: 不改 SearchRepository (per 守门 #1 v6 cross-stage 实测),
    // 走 InMemory cache `self.index` 直接查, 跟 search/suggest 同源

    /// 查符号(per brief §1.2 get_symbol)
    /// `file` 可选, 指定时仅返回该 file 下的符号
    async fn get_symbol(
        &self,
        tenant_id: TenantId,
        name: &str,
        file: Option<&str>,
        actor: &ActorContext,
    ) -> Result<Vec<SymbolRef>, SearchError>;

    /// 查引用(per brief §1.3 find_references)
    /// 匹配 Symbol 索引 + fulltext 全文(name 不区分大小写)
    async fn find_references(
        &self,
        tenant_id: TenantId,
        name: &str,
        file: Option<&str>,
        actor: &ActorContext,
    ) -> Result<Vec<ReferenceRef>, SearchError>;

    /// 查代码上下文(per brief §1.4 get_code_context)
    /// `radius` 控制 snippet 长度上下界
    async fn get_code_context(
        &self,
        tenant_id: TenantId,
        file: &str,
        line: u32,
        radius: u32,
        actor: &ActorContext,
    ) -> Result<CodeContext, SearchError>;
}

#[async_trait]
/// SearchRepository(存储适配层端口,由 InMemory / 真实存储实现)
pub trait SearchRepository: Send + Sync {
    /// 插入一条 SearchIndex 记录
    async fn insert_index(&self, idx: SearchIndex) -> Result<(), SearchError>;
    /// 按 ID 获取 SearchIndex
    async fn get_index(&self, id: SearchIndexId) -> Result<SearchIndex, SearchError>;
    /// 更新一条 SearchIndex 记录
    async fn update_index(&self, idx: SearchIndex) -> Result<(), SearchError>;
    /// 按租户 + 资源类型 + 资源 ID 查找 SearchIndex
    async fn get_index_by_resource(
        &self,
        tenant_id: TenantId,
        resource_type: ResourceType,
        resource_id: Uuid,
    ) -> Result<Option<SearchIndex>, SearchError>;
    /// 按租户 + 资源类型 + 资源 ID 删除 SearchIndex
    async fn delete_index_by_resource(
        &self,
        tenant_id: TenantId,
        resource_type: ResourceType,
        resource_id: Uuid,
    ) -> Result<bool, SearchError>;
    /// 执行全文检索
    async fn search(
        &self,
        tenant_id: TenantId,
        query_text: &str,
        filters: &HashMap<String, String>,
        limit: u32,
        offset: u32,
    ) -> Result<SearchResult, SearchError>;
    /// 执行自动补全
    async fn suggest(
        &self,
        tenant_id: TenantId,
        prefix: &str,
        limit: u32,
    ) -> Result<Vec<Suggestion>, SearchError>;

    /// 插入一条 SavedSearch 记录
    async fn insert_saved(&self, saved: SavedSearch) -> Result<(), SearchError>;
    /// 按租户 + 用户列出其 SavedSearch
    async fn list_saved_by_user(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
    ) -> Result<Vec<SavedSearch>, SearchError>;
    /// 按 ID 获取 SavedSearch
    async fn get_saved(&self, id: SavedSearchId) -> Result<SavedSearch, SearchError>;
    /// 按 ID 删除 SavedSearch
    async fn delete_saved(&self, id: SavedSearchId) -> Result<bool, SearchError>;
}

// =====================================================================
// InMemorySearchService
// =====================================================================

/// 基于内存的 SearchCommandPort / SearchQueryPort 实现(测试 / 参考用)
pub struct InMemorySearchService {
    repo: Arc<dyn SearchRepository>,
    index: Arc<RwLock<HashMap<SearchIndexId, SearchIndex>>>,
    saved: Arc<RwLock<HashMap<SavedSearchId, SavedSearch>>>,
}

impl InMemorySearchService {
    /// 使用默认的 InMemorySearchRepository 创建服务实例
    pub fn new() -> Self {
        Self {
            repo: Arc::new(InMemorySearchRepository::new()),
            index: Arc::new(RwLock::new(HashMap::new())),
            saved: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    /// 使用指定的 SearchRepository 创建服务实例
    pub fn with_repo(repo: Arc<dyn SearchRepository>) -> Self {
        Self {
            repo,
            index: Arc::new(RwLock::new(HashMap::new())),
            saved: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemorySearchService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SearchCommandPort for InMemorySearchService {
    async fn upsert_index(
        &self,
        cmd: UpsertIndexCommand,
        actor: &ActorContext,
    ) -> Result<SearchIndex, SearchError> {
        if TenantId::from(actor.tenant_id) != cmd.tenant_id {
            return Err(SearchError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                cmd.tenant_id,
            ));
        }
        // INV-S-02:仅 Worker Projector 可调用
        if !actor.is_local_runtime && !actor.has_role("system:search-projector") {
            return Err(SearchError::PermissionDenied);
        }
        let existing = self
            .repo
            .get_index_by_resource(cmd.tenant_id, cmd.resource_type, cmd.resource_id)
            .await?;
        let now = Utc::now();
        let idx = match existing {
            Some(mut e) => {
                if e.version >= cmd.projection_version {
                    // 旧版本,跳过
                    return Ok(e);
                }
                e.fulltext = cmd.fulltext;
                e.symbol_metadata = cmd.symbol_metadata;
                e.tags = cmd.tags;
                e.updated_at = now;
                e.version = cmd.projection_version;
                e
            }
            None => SearchIndex {
                id: SearchIndexId::new(),
                tenant_id: cmd.tenant_id,
                project_id: cmd.project_id,
                resource_type: cmd.resource_type,
                resource_id: cmd.resource_id,
                fulltext: cmd.fulltext,
                symbol_metadata: cmd.symbol_metadata,
                tags: cmd.tags,
                created_at: now,
                updated_at: now,
                version: cmd.projection_version,
            },
        };
        let is_new = !self.index.read().unwrap().contains_key(&idx.id);
        if is_new {
            self.repo.insert_index(idx.clone()).await?;
        } else {
            self.repo.update_index(idx.clone()).await?;
        }
        self.index.write().unwrap().insert(idx.id, idx.clone());
        Ok(idx)
    }

    async fn delete_index(
        &self,
        cmd: DeleteIndexCommand,
        actor: &ActorContext,
    ) -> Result<(), SearchError> {
        if TenantId::from(actor.tenant_id) != cmd.tenant_id {
            return Err(SearchError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                cmd.tenant_id,
            ));
        }
        if !actor.is_local_runtime && !actor.has_role("system:search-projector") {
            return Err(SearchError::PermissionDenied);
        }
        let deleted = self
            .repo
            .delete_index_by_resource(cmd.tenant_id, cmd.resource_type, cmd.resource_id)
            .await?;
        if deleted {
            // 同步本地缓存
            let mut index = self.index.write().unwrap();
            let to_remove: Vec<SearchIndexId> = index
                .values()
                .filter(|i| {
                    i.tenant_id == cmd.tenant_id
                        && i.resource_type == cmd.resource_type
                        && i.resource_id == cmd.resource_id
                })
                .map(|i| i.id)
                .collect();
            for id in to_remove {
                index.remove(&id);
            }
        }
        Ok(())
    }

    async fn bulk_reindex(
        &self,
        cmd: BulkReindexCommand,
        actor: &ActorContext,
    ) -> Result<BulkReindexResult, SearchError> {
        if TenantId::from(actor.tenant_id) != cmd.tenant_id {
            return Err(SearchError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                cmd.tenant_id,
            ));
        }
        if !actor.is_local_runtime && !actor.has_role("system:search-projector") {
            return Err(SearchError::PermissionDenied);
        }
        let mut inserted = 0u32;
        let mut updated = 0u32;
        let mut skipped = 0u32;
        for entry in cmd.entries {
            let existing = self
                .repo
                .get_index_by_resource(cmd.tenant_id, entry.resource_type, entry.resource_id)
                .await?;
            match existing {
                Some(e) if e.version >= entry.projection_version => {
                    skipped += 1;
                }
                Some(_) => {
                    self.upsert_index(entry, actor).await?;
                    updated += 1;
                }
                None => {
                    self.upsert_index(entry, actor).await?;
                    inserted += 1;
                }
            }
        }
        Ok(BulkReindexResult {
            inserted,
            updated,
            skipped,
        })
    }

    async fn save_search(
        &self,
        cmd: SaveSearchCommand,
        actor: &ActorContext,
    ) -> Result<SavedSearch, SearchError> {
        if TenantId::from(actor.tenant_id) != cmd.tenant_id {
            return Err(SearchError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                cmd.tenant_id,
            ));
        }
        if UserId::from(actor.user_id) != cmd.user_id {
            return Err(SearchError::PermissionDenied);
        }
        if cmd.name.is_empty() {
            return Err(SearchError::InvalidQuery("name required".to_string()));
        }
        let saved = SavedSearch {
            id: SavedSearchId::new(),
            tenant_id: cmd.tenant_id,
            user_id: UserId::from(cmd.user_id),
            name: cmd.name,
            query: cmd.query,
            created_at: Utc::now(),
        };
        self.repo.insert_saved(saved.clone()).await?;
        self.saved.write().unwrap().insert(saved.id, saved.clone());
        Ok(saved)
    }

    async fn delete_saved(
        &self,
        cmd: DeleteSavedSearchCommand,
        actor: &ActorContext,
    ) -> Result<(), SearchError> {
        if TenantId::from(actor.tenant_id) != cmd.tenant_id {
            return Err(SearchError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                cmd.tenant_id,
            ));
        }
        let saved = self.repo.get_saved(cmd.saved_search_id).await?;
        if saved.tenant_id != cmd.tenant_id {
            return Err(SearchError::CrossTenantDenied(
                saved.tenant_id,
                cmd.tenant_id,
            ));
        }
        // INV-S-05:仅本人
        if saved.user_id != UserId::from(actor.user_id) {
            return Err(SearchError::PermissionDenied);
        }
        let deleted = self.repo.delete_saved(cmd.saved_search_id).await?;
        if deleted {
            self.saved.write().unwrap().remove(&cmd.saved_search_id);
        }
        Ok(())
    }
}

// P1 工具链 helper (per docs/briefs/tool-p1-impl-001.md §1.2-1.4)
//
// P0 简化:SearchIndex 的 fulltext 是单字符串, 没真实行号
// (Tree-sitter / ripgrep 接入在 P2, per HANDOFF-ST-001 §5 H2-EXT 缺口).
// 这里用启发式估算:
//
// - `estimate_line` = name 在 fulltext 内出现的近似行号 (按 80 字符一行)
// - `build_context` = name 周围 ±40 字符的 snippet
// - `build_snippet` = fulltext 在 ±radius 行附近的窗口
// - `resource_priority` = Symbol > WorkItem > Comment > Other

fn estimate_line(text: &str, name_lc: &str) -> u32 {
    if let Some(pos) = text.to_lowercase().find(name_lc) {
        // 行号 = 偏移 / 80 + 1 (1-based)
        return (pos / 80 + 1) as u32;
    }
    0
}

fn build_context(text: &str, name: &str) -> String {
    let name_lc = name.to_lowercase();
    let text_lc = text.to_lowercase();
    if let Some(pos) = text_lc.find(&name_lc) {
        let start = pos.saturating_sub(40);
        let end = (pos + name.len() + 40).min(text.len());
        return text[start..end].to_string();
    }
    // fallback: 前 80 字符
    text.chars().take(80).collect()
}

fn build_snippet(text: &str, line: u32, radius: u32) -> String {
    // 把 fulltext 按 80 字符一行切, 取 [line-radius, line+radius] 范围
    let line_width: usize = 80;
    let start_line = line.saturating_sub(radius) as usize;
    let end_line = (line + radius) as usize;
    let mut out = String::new();
    let mut idx: usize = 0;
    let mut line_no: usize = 0;
    for ch in text.chars() {
        if line_no >= start_line && line_no <= end_line {
            out.push(ch);
        }
        idx += 1;
        if idx % line_width == 0 {
            if line_no >= start_line && line_no <= end_line {
                out.push('\n');
            }
            line_no += 1;
        }
    }
    // 收尾:若最后一行未满 line_width 也要算一行
    if idx % line_width != 0 {
        if line_no >= start_line && line_no <= end_line && !out.is_empty() && !out.ends_with('\n') {
            out.push('\n');
        }
    }
    out
}

fn resource_priority(rt: &ResourceType) -> u8 {
    match rt {
        ResourceType::Symbol => 0,
        ResourceType::WorkItem => 1,
        ResourceType::Comment => 2,
        ResourceType::Project => 3,
        ResourceType::Feedback => 4,
        ResourceType::Decision => 5,
        ResourceType::Adr => 6,
    }
}

#[async_trait]
impl SearchQueryPort for InMemorySearchService {
    async fn search(
        &self,
        q: SearchQueryDto,
        actor: &ActorContext,
    ) -> Result<SearchResult, SearchError> {
        if TenantId::from(actor.tenant_id) != q.tenant_id {
            return Err(SearchError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                q.tenant_id,
            ));
        }
        if q.query.limit > 1000 {
            return Err(SearchError::InvalidQuery("limit > 1000".to_string()));
        }
        if q.query.query_text.is_empty() {
            return Err(SearchError::InvalidQuery("query_text required".to_string()));
        }
        // INV-SR-02:tenant 隔离
        self.repo
            .search(
                q.tenant_id,
                &q.query.query_text,
                &q.query.filters,
                q.query.limit,
                q.query.offset,
            )
            .await
    }

    async fn suggest(
        &self,
        q: SuggestQueryDto,
        actor: &ActorContext,
    ) -> Result<Vec<Suggestion>, SearchError> {
        if TenantId::from(actor.tenant_id) != q.tenant_id {
            return Err(SearchError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                q.tenant_id,
            ));
        }
        if q.query.prefix.is_empty() {
            return Err(SearchError::InvalidQuery("prefix required".to_string()));
        }
        self.repo
            .suggest(q.tenant_id, &q.query.prefix, q.query.limit)
            .await
    }

    async fn list_saved(
        &self,
        tenant_id: TenantId,
        actor: &ActorContext,
    ) -> Result<Vec<SavedSearch>, SearchError> {
        if TenantId::from(actor.tenant_id) != tenant_id {
            return Err(SearchError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                tenant_id,
            ));
        }
        let all = self
            .repo
            .list_saved_by_user(tenant_id, UserId::from(actor.user_id))
            .await?;
        Ok(all)
    }

    // ===== P1 工具链 (per docs/briefs/tool-p1-impl-001.md §1.2-1.4) =====
    // 走 InMemory cache `self.index` 直接查, 不改 SearchRepository trait
    // (per 守门 #12 minimal-broadening 派生 + 守门 #13 a L0 协调)

    async fn get_symbol(
        &self,
        tenant_id: TenantId,
        name: &str,
        file: Option<&str>,
        actor: &ActorContext,
    ) -> Result<Vec<SymbolRef>, SearchError> {
        if TenantId::from(actor.tenant_id) != tenant_id {
            return Err(SearchError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                tenant_id,
            ));
        }
        if name.is_empty() {
            return Err(SearchError::InvalidQuery("name required".to_string()));
        }
        let name_lc = name.to_lowercase();
        let index = self.index.read().unwrap();
        let mut out: Vec<SymbolRef> = index
            .values()
            .filter(|i| i.tenant_id == tenant_id)
            .filter(|i| i.resource_type == ResourceType::Symbol)
            .filter_map(|i| {
                let sym = i.symbol_metadata.as_ref()?;
                if sym.name.to_lowercase() != name_lc {
                    return None;
                }
                if let Some(file_filter) = file {
                    if sym.file_path != file_filter {
                        return None;
                    }
                }
                Some(SymbolRef {
                    name: sym.name.clone(),
                    kind: sym.kind.clone(),
                    file_path: sym.file_path.clone(),
                    line: 0, // P0 简化:SearchIndex SymbolMetadata 不带 line
                    signature: sym.signature.clone(),
                })
            })
            .collect();
        // 稳定排序:file_path asc, name asc (per 1 号 P0 模式)
        out.sort_by(|a, b| {
            (a.file_path.clone(), a.name.clone()).cmp(&(b.file_path.clone(), b.name.clone()))
        });
        Ok(out)
    }

    async fn find_references(
        &self,
        tenant_id: TenantId,
        name: &str,
        file: Option<&str>,
        actor: &ActorContext,
    ) -> Result<Vec<ReferenceRef>, SearchError> {
        if TenantId::from(actor.tenant_id) != tenant_id {
            return Err(SearchError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                tenant_id,
            ));
        }
        if name.is_empty() {
            return Err(SearchError::InvalidQuery("name required".to_string()));
        }
        let name_lc = name.to_lowercase();
        let index = self.index.read().unwrap();
        let mut out: Vec<ReferenceRef> = index
            .values()
            .filter(|i| i.tenant_id == tenant_id)
            .filter(|i| {
                if let Some(file_filter) = file {
                    // file_path 既在 symbol_metadata (Symbol 资源) 也在 tags / fulltext 旁路
                    if let Some(sym) = &i.symbol_metadata {
                        if sym.file_path == file_filter {
                            return true;
                        }
                    }
                    if i.tags.iter().any(|t| t == file_filter) {
                        return true;
                    }
                    false
                } else {
                    true
                }
            })
            .filter(|i| {
                // 引用匹配:fulltext 含 name, 或 symbol_metadata.name 匹配
                let fulltext_lc = i.fulltext.to_lowercase();
                if fulltext_lc.contains(&name_lc) {
                    return true;
                }
                if let Some(sym) = &i.symbol_metadata {
                    if sym.name.to_lowercase() == name_lc {
                        return true;
                    }
                }
                false
            })
            .map(|i| {
                // P0 简化:line = fulltext 内的字符偏移 / 80 估算, column = 0
                let line = estimate_line(&i.fulltext, &name_lc);
                let file_path = i
                    .symbol_metadata
                    .as_ref()
                    .map(|s| s.file_path.clone())
                    .or_else(|| i.tags.iter().find(|t| t.contains('/')).cloned())
                    .unwrap_or_else(|| "<unknown>".to_string());
                let context = build_context(&i.fulltext, &name);
                ReferenceRef {
                    name: name.to_string(),
                    file_path,
                    line,
                    column: 0,
                    context,
                }
            })
            .collect();
        // 稳定排序:file_path asc, line asc
        out.sort_by(|a, b| a.file_path.cmp(&b.file_path).then(a.line.cmp(&b.line)));
        Ok(out)
    }

    async fn get_code_context(
        &self,
        tenant_id: TenantId,
        file: &str,
        line: u32,
        radius: u32,
        actor: &ActorContext,
    ) -> Result<CodeContext, SearchError> {
        if TenantId::from(actor.tenant_id) != tenant_id {
            return Err(SearchError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                tenant_id,
            ));
        }
        if file.is_empty() {
            return Err(SearchError::InvalidQuery("file required".to_string()));
        }
        let index = self.index.read().unwrap();
        // 找到该 file 下最相关的 SearchIndex (priority: Symbol > WorkItem > Comment > other)
        let file_filter = file.to_string();
        let mut candidates: Vec<&SearchIndex> = index
            .values()
            .filter(|i| i.tenant_id == tenant_id)
            .filter(|i| {
                if let Some(sym) = &i.symbol_metadata {
                    return sym.file_path == file_filter;
                }
                false
            })
            .collect();
        // 没找到 symbol 时, 退化到 tags/file_path 模糊匹配
        if candidates.is_empty() {
            candidates = index
                .values()
                .filter(|i| i.tenant_id == tenant_id)
                .filter(|i| i.tags.iter().any(|t| t.contains(&file_filter)))
                .collect();
        }
        // 排序:resource_type (Symbol 优先) + updated_at desc
        candidates.sort_by(|a, b| {
            let ord_a = resource_priority(&a.resource_type);
            let ord_b = resource_priority(&b.resource_type);
            ord_a.cmp(&ord_b).then(b.updated_at.cmp(&a.updated_at))
        });
        let picked = candidates.first().copied();
        match picked {
            Some(idx) => {
                let snippet = build_snippet(&idx.fulltext, line, radius);
                let start = line.saturating_sub(radius);
                let end = line.saturating_add(radius);
                Ok(CodeContext {
                    file_path: file.to_string(),
                    start_line: start,
                    end_line: end,
                    snippet,
                })
            }
            None => {
                // 无任何索引 → 返回空 context, 不报错 (跟 search 空 list 同源)
                Ok(CodeContext {
                    file_path: file.to_string(),
                    start_line: line.saturating_sub(radius),
                    end_line: line.saturating_add(radius),
                    snippet: String::new(),
                })
            }
        }
    }
}

// =====================================================================
// InMemorySearchRepository
// =====================================================================

/// 基于内存 HashMap 的 SearchRepository 实现(测试 / 参考用)
pub struct InMemorySearchRepository {
    index: RwLock<HashMap<SearchIndexId, SearchIndex>>,
    saved: RwLock<HashMap<SavedSearchId, SavedSearch>>,
}

impl InMemorySearchRepository {
    /// 创建空的内存仓储实例
    pub fn new() -> Self {
        Self {
            index: RwLock::new(HashMap::new()),
            saved: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemorySearchRepository {
    fn default() -> Self {
        Self::new()
    }
}

fn compute_score(text: &str, query: &str) -> f64 {
    let query_lc = query.to_lowercase();
    let text_lc = text.to_lowercase();
    if text_lc.contains(&query_lc) {
        1.0 - (text_lc.len() as f64 - query_lc.len() as f64).abs() / 1000.0
    } else {
        0.0
    }
}

#[async_trait]
impl SearchRepository for InMemorySearchRepository {
    async fn insert_index(&self, idx: SearchIndex) -> Result<(), SearchError> {
        self.index.write().unwrap().insert(idx.id, idx);
        Ok(())
    }
    async fn get_index(&self, id: SearchIndexId) -> Result<SearchIndex, SearchError> {
        self.index
            .read()
            .unwrap()
            .get(&id)
            .cloned()
            .ok_or_else(|| SearchError::NotFound(format!("index:{}", id.as_uuid())))
    }
    async fn update_index(&self, idx: SearchIndex) -> Result<(), SearchError> {
        self.index.write().unwrap().insert(idx.id, idx);
        Ok(())
    }
    async fn get_index_by_resource(
        &self,
        tenant_id: TenantId,
        resource_type: ResourceType,
        resource_id: Uuid,
    ) -> Result<Option<SearchIndex>, SearchError> {
        Ok(self
            .index
            .read()
            .unwrap()
            .values()
            .find(|i| {
                i.tenant_id == tenant_id
                    && i.resource_type == resource_type
                    && i.resource_id == resource_id
            })
            .cloned())
    }
    async fn delete_index_by_resource(
        &self,
        tenant_id: TenantId,
        resource_type: ResourceType,
        resource_id: Uuid,
    ) -> Result<bool, SearchError> {
        let mut index = self.index.write().unwrap();
        let before = index.len();
        index.retain(|_, i| {
            !(i.tenant_id == tenant_id
                && i.resource_type == resource_type
                && i.resource_id == resource_id)
        });
        Ok(index.len() < before)
    }
    async fn search(
        &self,
        tenant_id: TenantId,
        query_text: &str,
        filters: &HashMap<String, String>,
        limit: u32,
        offset: u32,
    ) -> Result<SearchResult, SearchError> {
        let index = self.index.read().unwrap();
        let mut hits: Vec<(f64, &SearchIndex)> = index
            .values()
            .filter(|i| i.tenant_id == tenant_id)
            .filter(|i| {
                // filter on resource_type
                if let Some(rt) = filters.get("resource_type") {
                    if i.resource_type.as_str() != rt {
                        return false;
                    }
                }
                if let Some(pid) = filters.get("project_id") {
                    if i.project_id.as_uuid().to_string() != *pid {
                        return false;
                    }
                }
                true
            })
            .map(|i| (compute_score(&i.fulltext, query_text), i))
            .filter(|(s, _)| *s > 0.0)
            .collect();
        hits.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        let total = hits.len() as u64;
        let start = offset as usize;
        let end = std::cmp::min(start + limit as usize, hits.len());
        let items: Vec<SearchHit> = hits[start..end]
            .iter()
            .map(|(score, i)| {
                let mut highlights = HashMap::new();
                highlights.insert("fulltext".to_string(), i.fulltext.clone());
                SearchHit {
                    resource_type: i.resource_type,
                    resource_id: i.resource_id,
                    score: *score,
                    highlights,
                    tenant_id: i.tenant_id,
                    project_id: i.project_id,
                }
            })
            .collect();
        Ok(SearchResult {
            total,
            items,
            facets: HashMap::new(),
        })
    }
    async fn suggest(
        &self,
        tenant_id: TenantId,
        prefix: &str,
        limit: u32,
    ) -> Result<Vec<Suggestion>, SearchError> {
        let prefix_lc = prefix.to_lowercase();
        let index = self.index.read().unwrap();
        let mut out: Vec<Suggestion> = index
            .values()
            .filter(|i| i.tenant_id == tenant_id)
            .filter(|i| i.fulltext.to_lowercase().contains(&prefix_lc))
            .take(limit as usize)
            .map(|i| Suggestion {
                text: i.fulltext.clone(),
                resource_type: i.resource_type,
                resource_id: i.resource_id,
            })
            .collect();
        out.truncate(limit as usize);
        Ok(out)
    }
    async fn insert_saved(&self, saved: SavedSearch) -> Result<(), SearchError> {
        self.saved.write().unwrap().insert(saved.id, saved);
        Ok(())
    }
    async fn list_saved_by_user(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
    ) -> Result<Vec<SavedSearch>, SearchError> {
        Ok(self
            .saved
            .read()
            .unwrap()
            .values()
            .filter(|s| s.tenant_id == tenant_id && s.user_id == user_id)
            .cloned()
            .collect())
    }
    async fn get_saved(&self, id: SavedSearchId) -> Result<SavedSearch, SearchError> {
        self.saved
            .read()
            .unwrap()
            .get(&id)
            .cloned()
            .ok_or_else(|| SearchError::NotFound(format!("saved:{}", id.as_uuid())))
    }
    async fn delete_saved(&self, id: SavedSearchId) -> Result<bool, SearchError> {
        Ok(self.saved.write().unwrap().remove(&id).is_some())
    }
}

// =====================================================================
// 单元测试
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    fn make_actor(tenant_id: TenantId, user_id: UserId) -> ActorContext {
        ActorContext::new(user_id.0, tenant_id.0).with_role("developer")
    }

    fn projector_actor(tenant_id: TenantId) -> ActorContext {
        ActorContext::new(Uuid::new_v4(), tenant_id.0)
            .as_local_runtime()
            .with_role("system:search-projector")
    }

    fn sample_index_cmd(
        tenant_id: TenantId,
        resource_type: ResourceType,
        fulltext: &str,
    ) -> UpsertIndexCommand {
        UpsertIndexCommand {
            tenant_id,
            project_id: ProjectId::new(),
            resource_type,
            resource_id: Uuid::new_v4(),
            fulltext: fulltext.to_string(),
            symbol_metadata: None,
            tags: vec![],
            projection_version: 1,
        }
    }

    #[test]
    fn resource_type_as_str() {
        assert_eq!(ResourceType::WorkItem.as_str(), "work_item");
        assert_eq!(ResourceType::Symbol.as_str(), "symbol");
    }

    #[tokio::test]
    async fn non_projector_cannot_upsert() {
        // INV-S-02:非 Worker Projector 不可写
        let svc = InMemorySearchService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let actor = make_actor(TenantId(tenant_id), UserId(uuid::Uuid::new_v4()));
        let res = svc
            .upsert_index(
                sample_index_cmd(TenantId(tenant_id), ResourceType::WorkItem, "x"),
                &actor,
            )
            .await;
        assert!(matches!(res, Err(SearchError::PermissionDenied)));
    }

    #[tokio::test]
    async fn projector_upsert_and_search() {
        let svc = InMemorySearchService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let projector = projector_actor(TenantId(tenant_id));
        // 注入 3 个 work_item 索引
        svc.upsert_index(
            sample_index_cmd(TenantId(tenant_id), ResourceType::WorkItem, "fix login bug"),
            &projector,
        )
        .await
        .unwrap();
        svc.upsert_index(
            sample_index_cmd(TenantId(tenant_id), ResourceType::WorkItem, "add OAuth"),
            &projector,
        )
        .await
        .unwrap();
        svc.upsert_index(
            sample_index_cmd(TenantId(tenant_id), ResourceType::Comment, "see login PR"),
            &projector,
        )
        .await
        .unwrap();
        // 检索 "login"
        let user = make_actor(TenantId(tenant_id), UserId(uuid::Uuid::new_v4()));
        let r = svc
            .search(
                SearchQueryDto {
                    tenant_id: TenantId(tenant_id),
                    query: SearchQuery {
                        query_text: "login".to_string(),
                        filters: HashMap::new(),
                        sort: None,
                        limit: 10,
                        offset: 0,
                        user_id: UserId::from(user.user_id),
                    },
                },
                &user,
            )
            .await
            .unwrap();
        assert_eq!(r.total, 2);
    }

    #[tokio::test]
    async fn cross_tenant_search_isolated() {
        // INV-SR-02:跨 tenant 严格隔离
        let svc = InMemorySearchService::new();
        let t1 = uuid::Uuid::new_v4();
        let t2 = uuid::Uuid::new_v4();
        let p1 = projector_actor(TenantId(t1));
        let p2 = projector_actor(TenantId(t2));
        svc.upsert_index(
            sample_index_cmd(TenantId(t1), ResourceType::WorkItem, "tenant1 doc"),
            &p1,
        )
        .await
        .unwrap();
        svc.upsert_index(
            sample_index_cmd(TenantId(t2), ResourceType::WorkItem, "tenant2 doc"),
            &p2,
        )
        .await
        .unwrap();
        let user1 = make_actor(TenantId(t1), UserId(uuid::Uuid::new_v4()));
        let r = svc
            .search(
                SearchQueryDto {
                    tenant_id: TenantId(t1),
                    query: SearchQuery {
                        query_text: "doc".to_string(),
                        filters: HashMap::new(),
                        sort: None,
                        limit: 10,
                        offset: 0,
                        user_id: UserId::from(user1.user_id),
                    },
                },
                &user1,
            )
            .await
            .unwrap();
        // 只能看到 tenant1 的 1 条
        assert_eq!(r.total, 1);
        assert_eq!(r.items[0].tenant_id, TenantId(t1));
    }

    #[tokio::test]
    async fn search_filter_by_resource_type() {
        let svc = InMemorySearchService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let projector = projector_actor(TenantId(tenant_id));
        svc.upsert_index(
            sample_index_cmd(
                TenantId(tenant_id),
                ResourceType::WorkItem,
                "auth module refactor",
            ),
            &projector,
        )
        .await
        .unwrap();
        svc.upsert_index(
            sample_index_cmd(
                TenantId(tenant_id),
                ResourceType::Comment,
                "auth review note",
            ),
            &projector,
        )
        .await
        .unwrap();
        let user = make_actor(TenantId(tenant_id), UserId(uuid::Uuid::new_v4()));
        let mut filters = HashMap::new();
        filters.insert("resource_type".to_string(), "work_item".to_string());
        let r = svc
            .search(
                SearchQueryDto {
                    tenant_id: TenantId(tenant_id),
                    query: SearchQuery {
                        query_text: "auth".to_string(),
                        filters,
                        sort: None,
                        limit: 10,
                        offset: 0,
                        user_id: UserId::from(user.user_id),
                    },
                },
                &user,
            )
            .await
            .unwrap();
        assert_eq!(r.total, 1);
        assert_eq!(r.items[0].resource_type, ResourceType::WorkItem);
    }

    #[tokio::test]
    async fn upsert_idempotent_old_version_skipped() {
        let svc = InMemorySearchService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let projector = projector_actor(TenantId(tenant_id));
        let cmd = sample_index_cmd(TenantId(tenant_id), ResourceType::WorkItem, "v1 text");
        svc.upsert_index(cmd.clone(), &projector).await.unwrap();
        // 旧版本投影:跳过
        let mut old = cmd.clone();
        old.fulltext = "v0 text OLD".to_string();
        old.projection_version = 0;
        let idx = svc.upsert_index(old, &projector).await.unwrap();
        assert_eq!(idx.fulltext, "v1 text"); // 没被覆盖
    }

    #[tokio::test]
    async fn delete_index_by_resource() {
        let svc = InMemorySearchService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let projector = projector_actor(TenantId(tenant_id));
        let resource_id = Uuid::new_v4();
        let cmd = UpsertIndexCommand {
            tenant_id: TenantId(tenant_id),
            project_id: ProjectId::new(),
            resource_type: ResourceType::WorkItem,
            resource_id,
            fulltext: "to be deleted".to_string(),
            symbol_metadata: None,
            tags: vec![],
            projection_version: 1,
        };
        svc.upsert_index(cmd, &projector).await.unwrap();
        svc.delete_index(
            DeleteIndexCommand {
                tenant_id: TenantId(tenant_id),
                resource_type: ResourceType::WorkItem,
                resource_id,
            },
            &projector,
        )
        .await
        .unwrap();
        let user = make_actor(TenantId(tenant_id), UserId(uuid::Uuid::new_v4()));
        let r = svc
            .search(
                SearchQueryDto {
                    tenant_id: TenantId(tenant_id),
                    query: SearchQuery {
                        query_text: "deleted".to_string(),
                        filters: HashMap::new(),
                        sort: None,
                        limit: 10,
                        offset: 0,
                        user_id: UserId::from(user.user_id),
                    },
                },
                &user,
            )
            .await
            .unwrap();
        assert_eq!(r.total, 0);
    }

    #[tokio::test]
    async fn bulk_reindex_mixed() {
        let svc = InMemorySearchService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let projector = projector_actor(TenantId(tenant_id));
        let project_id = ProjectId::new();
        let entries = vec![
            UpsertIndexCommand {
                tenant_id: TenantId(tenant_id),
                project_id,
                resource_type: ResourceType::WorkItem,
                resource_id: Uuid::new_v4(),
                fulltext: "new1".to_string(),
                symbol_metadata: None,
                tags: vec![],
                projection_version: 1,
            },
            UpsertIndexCommand {
                tenant_id: TenantId(tenant_id),
                project_id,
                resource_type: ResourceType::WorkItem,
                resource_id: Uuid::new_v4(),
                fulltext: "new2".to_string(),
                symbol_metadata: None,
                tags: vec![],
                projection_version: 1,
            },
        ];
        let r = svc
            .bulk_reindex(
                BulkReindexCommand {
                    tenant_id: TenantId(tenant_id),
                    project_id,
                    entries,
                },
                &projector,
            )
            .await
            .unwrap();
        assert_eq!(r.inserted, 2);
        assert_eq!(r.updated, 0);
        assert_eq!(r.skipped, 0);
    }

    #[tokio::test]
    async fn save_and_list_saved_search() {
        let svc = InMemorySearchService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let me = uuid::Uuid::new_v4();
        let actor = make_actor(TenantId(tenant_id), UserId(me));
        let q = SearchQuery {
            query_text: "login".to_string(),
            filters: HashMap::new(),
            sort: None,
            limit: 10,
            offset: 0,
            user_id: UserId(me),
        };
        let saved = svc
            .save_search(
                SaveSearchCommand {
                    tenant_id: TenantId(tenant_id),
                    user_id: UserId(me),
                    name: "my login search".to_string(),
                    query: q,
                    actor_user_id: UserId(me),
                },
                &actor,
            )
            .await
            .unwrap();
        let list = svc.list_saved(TenantId(tenant_id), &actor).await.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, saved.id);
    }

    #[tokio::test]
    async fn saved_search_other_user_cannot_delete() {
        // INV-S-05:仅本人可删
        let svc = InMemorySearchService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let me = uuid::Uuid::new_v4();
        let other = uuid::Uuid::new_v4();
        let actor_me = make_actor(TenantId(tenant_id), UserId(me));
        let q = SearchQuery {
            query_text: "x".to_string(),
            filters: HashMap::new(),
            sort: None,
            limit: 10,
            offset: 0,
            user_id: UserId(me),
        };
        let saved = svc
            .save_search(
                SaveSearchCommand {
                    tenant_id: TenantId(tenant_id),
                    user_id: UserId(me),
                    name: "private".to_string(),
                    query: q,
                    actor_user_id: UserId(me),
                },
                &actor_me,
            )
            .await
            .unwrap();
        let actor_other = make_actor(TenantId(tenant_id), UserId(other));
        let res = svc
            .delete_saved(
                DeleteSavedSearchCommand {
                    tenant_id: TenantId(tenant_id),
                    saved_search_id: saved.id,
                    actor_user_id: UserId(other),
                },
                &actor_other,
            )
            .await;
        assert!(matches!(res, Err(SearchError::PermissionDenied)));
    }

    #[tokio::test]
    async fn suggest_partial_match() {
        let svc = InMemorySearchService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let projector = projector_actor(TenantId(tenant_id));
        svc.upsert_index(
            sample_index_cmd(
                TenantId(tenant_id),
                ResourceType::WorkItem,
                "implement authentication",
            ),
            &projector,
        )
        .await
        .unwrap();
        svc.upsert_index(
            sample_index_cmd(
                TenantId(tenant_id),
                ResourceType::WorkItem,
                "authorize user",
            ),
            &projector,
        )
        .await
        .unwrap();
        let user = make_actor(TenantId(tenant_id), UserId(uuid::Uuid::new_v4()));
        let s = svc
            .suggest(
                SuggestQueryDto {
                    tenant_id: TenantId(tenant_id),
                    query: SuggestQuery {
                        prefix: "auth".to_string(),
                        limit: 10,
                        user_id: UserId::from(user.user_id),
                    },
                },
                &user,
            )
            .await
            .unwrap();
        assert!(!s.is_empty());
    }

    #[tokio::test]
    async fn search_empty_query_rejected() {
        let svc = InMemorySearchService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let user = make_actor(TenantId(tenant_id), UserId(uuid::Uuid::new_v4()));
        let res = svc
            .search(
                SearchQueryDto {
                    tenant_id: TenantId(tenant_id),
                    query: SearchQuery {
                        query_text: "".to_string(),
                        filters: HashMap::new(),
                        sort: None,
                        limit: 10,
                        offset: 0,
                        user_id: UserId::from(user.user_id),
                    },
                },
                &user,
            )
            .await;
        assert!(matches!(res, Err(SearchError::InvalidQuery(_))));
    }

    // ===== P1 工具链新方法测试 (per docs/briefs/tool-p1-impl-001.md §1.2-1.4) =====

    fn symbol_index_cmd(
        tenant_id: TenantId,
        name: &str,
        file_path: &str,
        fulltext: &str,
    ) -> UpsertIndexCommand {
        UpsertIndexCommand {
            tenant_id,
            project_id: ProjectId::new(),
            resource_type: ResourceType::Symbol,
            resource_id: Uuid::new_v4(),
            fulltext: fulltext.to_string(),
            symbol_metadata: Some(SymbolMetadata {
                name: name.to_string(),
                kind: "function".to_string(),
                signature: format!("fn {name}()"),
                file_path: file_path.to_string(),
            }),
            tags: vec![file_path.to_string()],
            projection_version: 1,
        }
    }

    #[tokio::test]
    async fn get_symbol_finds_matching_name() {
        let svc = InMemorySearchService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let projector = projector_actor(TenantId(tenant_id));
        svc.upsert_index(
            symbol_index_cmd(
                TenantId(tenant_id),
                "authenticate_user",
                "crates/auth/src/lib.rs",
                "fn authenticate_user()",
            ),
            &projector,
        )
        .await
        .unwrap();
        svc.upsert_index(
            symbol_index_cmd(
                TenantId(tenant_id),
                "logout_user",
                "crates/auth/src/lib.rs",
                "fn logout_user()",
            ),
            &projector,
        )
        .await
        .unwrap();

        let user = make_actor(TenantId(tenant_id), UserId(uuid::Uuid::new_v4()));
        let r = svc
            .get_symbol(TenantId(tenant_id), "authenticate_user", None, &user)
            .await
            .unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].name, "authenticate_user");
        assert_eq!(r[0].file_path, "crates/auth/src/lib.rs");
    }

    #[tokio::test]
    async fn get_symbol_with_file_filter() {
        let svc = InMemorySearchService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let projector = projector_actor(TenantId(tenant_id));
        svc.upsert_index(
            symbol_index_cmd(
                TenantId(tenant_id),
                "foo",
                "crates/a/src/lib.rs",
                "fn foo()",
            ),
            &projector,
        )
        .await
        .unwrap();
        svc.upsert_index(
            symbol_index_cmd(
                TenantId(tenant_id),
                "foo",
                "crates/b/src/lib.rs",
                "fn foo()",
            ),
            &projector,
        )
        .await
        .unwrap();

        let user = make_actor(TenantId(tenant_id), UserId(uuid::Uuid::new_v4()));
        let r = svc
            .get_symbol(
                TenantId(tenant_id),
                "foo",
                Some("crates/a/src/lib.rs"),
                &user,
            )
            .await
            .unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].file_path, "crates/a/src/lib.rs");
    }

    #[tokio::test]
    async fn get_symbol_empty_name_rejected() {
        let svc = InMemorySearchService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let user = make_actor(TenantId(tenant_id), UserId(uuid::Uuid::new_v4()));
        let r = svc.get_symbol(TenantId(tenant_id), "", None, &user).await;
        assert!(matches!(r, Err(SearchError::InvalidQuery(_))));
    }

    #[tokio::test]
    async fn get_symbol_cross_tenant_denied() {
        let svc = InMemorySearchService::new();
        let t1 = uuid::Uuid::new_v4();
        let t2 = uuid::Uuid::new_v4();
        let user_t1 = make_actor(TenantId(t1), UserId(uuid::Uuid::new_v4()));
        let r = svc
            .get_symbol(TenantId(t2), "anything", None, &user_t1)
            .await;
        assert!(matches!(r, Err(SearchError::CrossTenantDenied(_, _))));
    }

    #[tokio::test]
    async fn find_references_matches_fulltext_and_symbol() {
        let svc = InMemorySearchService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let projector = projector_actor(TenantId(tenant_id));
        // Symbol 定义
        svc.upsert_index(
            symbol_index_cmd(
                TenantId(tenant_id),
                "compute_score",
                "crates/search/src/lib.rs",
                "fn compute_score() { 1.0 }",
            ),
            &projector,
        )
        .await
        .unwrap();
        // 引用:另一个 WorkItem fulltext 含 compute_score
        svc.upsert_index(
            sample_index_cmd(
                TenantId(tenant_id),
                ResourceType::WorkItem,
                "see compute_score in search module",
            ),
            &projector,
        )
        .await
        .unwrap();

        let user = make_actor(TenantId(tenant_id), UserId(uuid::Uuid::new_v4()));
        let r = svc
            .find_references(TenantId(tenant_id), "compute_score", None, &user)
            .await
            .unwrap();
        // 至少 1 个引用 (Symbol 定义本身或 WorkItem 引用)
        assert!(!r.is_empty());
    }

    #[tokio::test]
    async fn find_references_with_file_filter() {
        let svc = InMemorySearchService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let projector = projector_actor(TenantId(tenant_id));
        svc.upsert_index(
            symbol_index_cmd(
                TenantId(tenant_id),
                "alpha",
                "crates/x/src/lib.rs",
                "fn alpha()",
            ),
            &projector,
        )
        .await
        .unwrap();
        svc.upsert_index(
            symbol_index_cmd(
                TenantId(tenant_id),
                "alpha",
                "crates/y/src/lib.rs",
                "fn alpha()",
            ),
            &projector,
        )
        .await
        .unwrap();

        let user = make_actor(TenantId(tenant_id), UserId(uuid::Uuid::new_v4()));
        let r = svc
            .find_references(
                TenantId(tenant_id),
                "alpha",
                Some("crates/x/src/lib.rs"),
                &user,
            )
            .await
            .unwrap();
        // 至少 1 个匹配, file 全部 crates/x/src/lib.rs
        assert!(!r.is_empty());
        for ref_ in &r {
            assert_eq!(ref_.file_path, "crates/x/src/lib.rs");
        }
    }

    #[tokio::test]
    async fn find_references_empty_name_rejected() {
        let svc = InMemorySearchService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let user = make_actor(TenantId(tenant_id), UserId(uuid::Uuid::new_v4()));
        let r = svc
            .find_references(TenantId(tenant_id), "", None, &user)
            .await;
        assert!(matches!(r, Err(SearchError::InvalidQuery(_))));
    }

    #[tokio::test]
    async fn get_code_context_returns_window_for_known_file() {
        let svc = InMemorySearchService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let projector = projector_actor(TenantId(tenant_id));
        svc.upsert_index(
            symbol_index_cmd(
                TenantId(tenant_id),
                "main",
                "crates/app/src/main.rs",
                "fn main() { println!(\"hello\"); }",
            ),
            &projector,
        )
        .await
        .unwrap();

        let user = make_actor(TenantId(tenant_id), UserId(uuid::Uuid::new_v4()));
        let r = svc
            .get_code_context(TenantId(tenant_id), "crates/app/src/main.rs", 1, 3, &user)
            .await
            .unwrap();
        assert_eq!(r.file_path, "crates/app/src/main.rs");
        assert_eq!(r.start_line, 0);
        assert_eq!(r.end_line, 4);
    }

    #[tokio::test]
    async fn get_code_context_empty_for_unknown_file() {
        let svc = InMemorySearchService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let user = make_actor(TenantId(tenant_id), UserId(uuid::Uuid::new_v4()));
        let r = svc
            .get_code_context(TenantId(tenant_id), "no/such/file.rs", 10, 5, &user)
            .await
            .unwrap();
        // 走 fallback 路径, 返回空 context
        assert!(r.snippet.is_empty());
        assert_eq!(r.start_line, 5);
        assert_eq!(r.end_line, 15);
    }

    #[tokio::test]
    async fn get_code_context_empty_file_rejected() {
        let svc = InMemorySearchService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let user = make_actor(TenantId(tenant_id), UserId(uuid::Uuid::new_v4()));
        let r = svc
            .get_code_context(TenantId(tenant_id), "", 1, 5, &user)
            .await;
        assert!(matches!(r, Err(SearchError::InvalidQuery(_))));
    }
}

pub mod jql;
