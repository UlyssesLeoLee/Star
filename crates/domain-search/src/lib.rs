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

#![warn(missing_docs)]

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
pub enum ResourceType {
    WorkItem,
    Comment,
    Project,
    Symbol,
    Feedback,
    Decision,
    Adr,
}

impl ResourceType {
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
// 实体 — SearchIndex(Projection,只读)
// =====================================================================

/// SearchIndex 投影(§4.15,§12)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchIndex {
    pub id: SearchIndexId,
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub resource_type: ResourceType,
    pub resource_id: Uuid,
    pub fulltext: String,
    pub symbol_metadata: Option<SymbolMetadata>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// 乐观版本(用于 Projector 重放)
    pub version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolMetadata {
    pub name: String,
    pub kind: String,
    pub signature: String,
    pub file_path: String,
}

// =====================================================================
// 实体 — SearchQuery
// =====================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub query_text: String,
    pub filters: HashMap<String, String>,
    pub sort: Option<String>,
    pub limit: u32,
    pub offset: u32,
    pub user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub total: u64,
    pub items: Vec<SearchHit>,
    pub facets: HashMap<String, Vec<Facet>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    pub resource_type: ResourceType,
    pub resource_id: Uuid,
    pub score: f64,
    pub highlights: HashMap<String, String>,
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Facet {
    pub value: String,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestQuery {
    pub prefix: String,
    pub limit: u32,
    pub user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    pub text: String,
    pub resource_type: ResourceType,
    pub resource_id: Uuid,
}

// =====================================================================
// 实体 — SavedSearch
// =====================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedSearch {
    pub id: SavedSearchId,
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub name: String,
    pub query: SearchQuery,
    pub created_at: DateTime<Utc>,
}

// =====================================================================
// 错误
// =====================================================================

#[derive(Debug, Error)]
pub enum SearchError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("invalid state: {0}")]
    InvalidState(String),
    #[error("permission denied")]
    PermissionDenied,
    /// INV-SR-02 / INV-S-03:跨 tenant 拒绝
    #[error("cross-tenant access denied: tenant {0} vs required {1}")]
    CrossTenantDenied(TenantId, TenantId),
    #[error("invalid query: {0}")]
    InvalidQuery(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("internal: {0}")]
    Internal(String),
}

// =====================================================================
// 命令 / 查询 DTO
// =====================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertIndexCommand {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub resource_type: ResourceType,
    pub resource_id: Uuid,
    pub fulltext: String,
    pub symbol_metadata: Option<SymbolMetadata>,
    pub tags: Vec<String>,
    pub projection_version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteIndexCommand {
    pub tenant_id: TenantId,
    pub resource_type: ResourceType,
    pub resource_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkReindexCommand {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub entries: Vec<UpsertIndexCommand>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkReindexResult {
    pub inserted: u32,
    pub updated: u32,
    pub skipped: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveSearchCommand {
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub name: String,
    pub query: SearchQuery,
    pub actor_user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteSavedSearchCommand {
    pub tenant_id: TenantId,
    pub saved_search_id: SavedSearchId,
    pub actor_user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQueryDto {
    pub tenant_id: TenantId,
    pub query: SearchQuery,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestQueryDto {
    pub tenant_id: TenantId,
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

    async fn delete_index(
        &self,
        cmd: DeleteIndexCommand,
        actor: &ActorContext,
    ) -> Result<(), SearchError>;

    async fn bulk_reindex(
        &self,
        cmd: BulkReindexCommand,
        actor: &ActorContext,
    ) -> Result<BulkReindexResult, SearchError>;

    async fn save_search(
        &self,
        cmd: SaveSearchCommand,
        actor: &ActorContext,
    ) -> Result<SavedSearch, SearchError>;

    async fn delete_saved(
        &self,
        cmd: DeleteSavedSearchCommand,
        actor: &ActorContext,
    ) -> Result<(), SearchError>;
}

#[async_trait]
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
}

#[async_trait]
pub trait SearchRepository: Send + Sync {
    async fn insert_index(&self, idx: SearchIndex) -> Result<(), SearchError>;
    async fn get_index(&self, id: SearchIndexId) -> Result<SearchIndex, SearchError>;
    async fn update_index(&self, idx: SearchIndex) -> Result<(), SearchError>;
    async fn get_index_by_resource(
        &self,
        tenant_id: TenantId,
        resource_type: ResourceType,
        resource_id: Uuid,
    ) -> Result<Option<SearchIndex>, SearchError>;
    async fn delete_index_by_resource(
        &self,
        tenant_id: TenantId,
        resource_type: ResourceType,
        resource_id: Uuid,
    ) -> Result<bool, SearchError>;
    async fn search(
        &self,
        tenant_id: TenantId,
        query_text: &str,
        filters: &HashMap<String, String>,
        limit: u32,
        offset: u32,
    ) -> Result<SearchResult, SearchError>;
    async fn suggest(
        &self,
        tenant_id: TenantId,
        prefix: &str,
        limit: u32,
    ) -> Result<Vec<Suggestion>, SearchError>;

    async fn insert_saved(&self, saved: SavedSearch) -> Result<(), SearchError>;
    async fn list_saved_by_user(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
    ) -> Result<Vec<SavedSearch>, SearchError>;
    async fn get_saved(&self, id: SavedSearchId) -> Result<SavedSearch, SearchError>;
    async fn delete_saved(&self, id: SavedSearchId) -> Result<bool, SearchError>;
}

// =====================================================================
// InMemorySearchService
// =====================================================================

pub struct InMemorySearchService {
    repo: Arc<dyn SearchRepository>,
    index: Arc<RwLock<HashMap<SearchIndexId, SearchIndex>>>,
    saved: Arc<RwLock<HashMap<SavedSearchId, SavedSearch>>>,
}

impl InMemorySearchService {
    pub fn new() -> Self {
        Self {
            repo: Arc::new(InMemorySearchRepository::new()),
            index: Arc::new(RwLock::new(HashMap::new())),
            saved: Arc::new(RwLock::new(HashMap::new())),
        }
    }
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
}

// =====================================================================
// InMemorySearchRepository
// =====================================================================

pub struct InMemorySearchRepository {
    index: RwLock<HashMap<SearchIndexId, SearchIndex>>,
    saved: RwLock<HashMap<SavedSearchId, SavedSearch>>,
}

impl InMemorySearchRepository {
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
                sample_index_cmd(tenant_id, ResourceType::WorkItem, "x"),
                &actor,
            )
            .await;
        assert!(matches!(res, Err(SearchError::PermissionDenied)));
    }

    #[tokio::test]
    async fn projector_upsert_and_search() {
        let svc = InMemorySearchService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let projector = projector_actor(tenant_id);
        // 注入 3 个 work_item 索引
        svc.upsert_index(
            sample_index_cmd(tenant_id, ResourceType::WorkItem, "fix login bug"),
            &projector,
        )
        .await
        .unwrap();
        svc.upsert_index(
            sample_index_cmd(tenant_id, ResourceType::WorkItem, "add OAuth"),
            &projector,
        )
        .await
        .unwrap();
        svc.upsert_index(
            sample_index_cmd(tenant_id, ResourceType::Comment, "see login PR"),
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
        let p1 = projector_actor(t1);
        let p2 = projector_actor(t2);
        svc.upsert_index(
            sample_index_cmd(t1, ResourceType::WorkItem, "tenant1 doc"),
            &p1,
        )
        .await
        .unwrap();
        svc.upsert_index(
            sample_index_cmd(t2, ResourceType::WorkItem, "tenant2 doc"),
            &p2,
        )
        .await
        .unwrap();
        let user1 = make_actor(TenantId(t1), uuid::Uuid::new_v4());
        let r = svc
            .search(
                SearchQueryDto {
                    tenant_id: t1,
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
        assert_eq!(r.items[0].tenant_id, t1);
    }

    #[tokio::test]
    async fn search_filter_by_resource_type() {
        let svc = InMemorySearchService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let projector = projector_actor(tenant_id);
        svc.upsert_index(
            sample_index_cmd(tenant_id, ResourceType::WorkItem, "auth module refactor"),
            &projector,
        )
        .await
        .unwrap();
        svc.upsert_index(
            sample_index_cmd(tenant_id, ResourceType::Comment, "auth review note"),
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
        let projector = projector_actor(tenant_id);
        let cmd = sample_index_cmd(tenant_id, ResourceType::WorkItem, "v1 text");
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
        let projector = projector_actor(tenant_id);
        let resource_id = Uuid::new_v4();
        let cmd = UpsertIndexCommand {
            tenant_id,
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
        let projector = projector_actor(tenant_id);
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
        let actor = make_actor(TenantId(tenant_id), me);
        let q = SearchQuery {
            query_text: "login".to_string(),
            filters: HashMap::new(),
            sort: None,
            limit: 10,
            offset: 0,
            user_id: me,
        };
        let saved = svc
            .save_search(
                SaveSearchCommand {
                    tenant_id: TenantId(tenant_id),
                    user_id: me,
                    name: "my login search".to_string(),
                    query: q,
                    actor_user_id: me,
                },
                &actor,
            )
            .await
            .unwrap();
        let list = svc.list_saved(tenant_id, &actor).await.unwrap();
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
        let actor_me = make_actor(TenantId(tenant_id), me);
        let q = SearchQuery {
            query_text: "x".to_string(),
            filters: HashMap::new(),
            sort: None,
            limit: 10,
            offset: 0,
            user_id: me,
        };
        let saved = svc
            .save_search(
                SaveSearchCommand {
                    tenant_id: TenantId(tenant_id),
                    user_id: me,
                    name: "private".to_string(),
                    query: q,
                    actor_user_id: me,
                },
                &actor_me,
            )
            .await
            .unwrap();
        let actor_other = make_actor(TenantId(tenant_id), other);
        let res = svc
            .delete_saved(
                DeleteSavedSearchCommand {
                    tenant_id: TenantId(tenant_id),
                    saved_search_id: saved.id,
                    actor_user_id: other,
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
        let projector = projector_actor(tenant_id);
        svc.upsert_index(
            sample_index_cmd(
                tenant_id: TenantId(tenant_id),
                ResourceType::WorkItem,
                "implement authentication",
            ),
            &projector,
        )
        .await
        .unwrap();
        svc.upsert_index(
            sample_index_cmd(tenant_id, ResourceType::WorkItem, "authorize user"),
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
}

pub mod jql;
