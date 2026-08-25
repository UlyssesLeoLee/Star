//! Integration 端口(Port Traits)与命令/查询 DTO
//!
//! 来源:
//! - `docs/api-design.md` §3.13 (Integration / SyncState)
//! - `docs/specs/domain-integration-spec.md` §4 (接口签名)
//!
//! **端口清单**:
//! - `IntegrationCommandPort`: 7 方法(create / update / configure / pause / resume / trigger_sync / handle_webhook)
//! - `IntegrationQueryPort`: 4 方法(get / list / get_sync_state / get_history)
//! - `IntegrationRepository`: 仓储抽象(infrastructure crate 实现)

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::context::ActorContext;
use crate::entity::{Integration, SyncState};
use crate::error::IntegrationError;
use crate::value_object::{
    ConflictStrategy, ExternalEntityId, ExternalSystemName, IntegrationId,
    IntegrationRelationType, IntegrationSource, IntegrationState, ProjectId, TenantId, UserId,
};

// =====================================================================
// 命令 DTO
// =====================================================================

/// `CreateIntegrationCommand`(创建 Integration)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIntegrationCommand {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// Project ID
    pub project_id: ProjectId,
    /// 源系统分类(SCM / PM / Communication / Other)
    pub source: IntegrationSource,
    /// 关系类型(Link / Mirror / Bidirectional / PlatformOwned)
    pub relation_type: IntegrationRelationType,
    /// 外部系统名
    pub external_system_name: ExternalSystemName,
    /// 外部实体 ID
    pub external_id: ExternalEntityId,
    /// 外部 URL
    pub external_url: String,
    /// 冲突策略(默认 ManualReview)
    pub conflict_strategy: ConflictStrategy,
    /// Credential ID 引用(走 Credential Broker)
    pub credential_id: Option<uuid::Uuid>,
    /// 初始 sync_token(Link 关系留 None,其他关系必填)
    pub initial_sync_token: Option<String>,
}

/// `UpdateIntegrationCommand`(更新 Integration 字段)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateIntegrationCommand {
    /// Integration ID
    pub integration_id: IntegrationId,
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 新冲突策略
    pub conflict_strategy: Option<ConflictStrategy>,
    /// 新 sync_token
    pub sync_token: Option<String>,
    /// 新外部 URL
    pub external_url: Option<String>,
    /// 新 Credential ID
    pub credential_id: Option<uuid::Uuid>,
}

/// `ConfigureIntegrationCommand`(B/configure,如配置 Loop 防护 / 字段映射)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigureIntegrationCommand {
    /// Integration ID
    pub integration_id: IntegrationId,
    /// 租户 ID
    pub tenant_id: TenantId,
    /// Loop 防护 Idempotency Key(必填,Bidirectional)
    pub idempotency_key: Option<String>,
    /// 字段映射(覆盖现有 mapping_config)
    pub mapping_json: Option<serde_json::Value>,
}

/// `PauseIntegrationCommand`(暂停 Integration)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PauseIntegrationCommand {
    pub integration_id: IntegrationId,
    pub tenant_id: TenantId,
}

/// `ResumeIntegrationCommand`(恢复 Integration)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResumeIntegrationCommand {
    pub integration_id: IntegrationId,
    pub tenant_id: TenantId,
}

/// `TriggerSyncCommand`(手动触发同步)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerSyncCommand {
    pub integration_id: IntegrationId,
    pub tenant_id: TenantId,
    /// 强制刷新(忽略 last_synced_at)
    pub force: bool,
}

/// `HandleWebhookCommand`(处理入站 Webhook)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandleWebhookCommand {
    pub integration_id: IntegrationId,
    pub tenant_id: TenantId,
    /// 外部事件 ID(用于幂等)
    pub external_event_id: String,
    /// 原始 payload
    pub payload: String,
    /// 签名
    pub signature: Option<String>,
}

// =====================================================================
// 查询 DTO
// =====================================================================

/// `ListByProjectQuery`(列出 Project 下的 Integration)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListByProjectQuery {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    /// 按 source 过滤(None = 全部)
    pub source_filter: Option<IntegrationSource>,
    /// 按 relation_type 过滤(None = 全部)
    pub relation_type_filter: Option<IntegrationRelationType>,
    /// 按 state 过滤(None = 全部)
    pub state_filter: Option<IntegrationState>,
    /// 仅活跃
    pub active_only: bool,
}

/// `GetHistoryQuery`(列出 SyncState 历史)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetHistoryQuery {
    pub tenant_id: TenantId,
    pub integration_id: IntegrationId,
    /// 限制返回条数
    pub limit: u32,
    /// 起始时间
    pub since: Option<DateTime<Utc>>,
}

// =====================================================================
// 端口:IntegrationCommandPort(7 方法)
// =====================================================================

/// **Integration 命令端口**(写操作 7 方法)
#[async_trait]
pub trait IntegrationCommandPort: Send + Sync {
    /// 创建 Integration(INV-I-01/02/03/04/05/06 校验)
    async fn create_integration(
        &self,
        cmd: CreateIntegrationCommand,
        actor: ActorContext,
    ) -> Result<Integration, IntegrationError>;

    /// 配置 Integration(Loop 防护 / 字段映射)
    async fn configure_integration(
        &self,
        cmd: ConfigureIntegrationCommand,
        actor: ActorContext,
    ) -> Result<Integration, IntegrationError>;

    /// 暂停 Integration
    async fn pause_integration(
        &self,
        cmd: PauseIntegrationCommand,
        actor: ActorContext,
    ) -> Result<Integration, IntegrationError>;

    /// 恢复 Integration
    async fn resume_integration(
        &self,
        cmd: ResumeIntegrationCommand,
        actor: ActorContext,
    ) -> Result<Integration, IntegrationError>;

    /// 触发同步(发事件,Worker 异步执行)
    async fn trigger_sync(
        &self,
        cmd: TriggerSyncCommand,
        actor: ActorContext,
    ) -> Result<SyncState, IntegrationError>;

    /// 处理入站 Webhook(INV-I-02 Loop 防护 + 幂等)
    async fn handle_webhook(
        &self,
        cmd: HandleWebhookCommand,
    ) -> Result<SyncState, IntegrationError>;
}

// =====================================================================
// 端口:IntegrationQueryPort(4 方法)
// =====================================================================

/// **Integration 查询端口**(读操作 4 方法)
#[async_trait]
pub trait IntegrationQueryPort: Send + Sync {
    /// 按 ID 查询 Integration(带租户隔离校验)
    async fn get_integration(
        &self,
        id: IntegrationId,
        viewer: ActorContext,
    ) -> Result<Integration, IntegrationError>;

    /// 列出 Project 下的 Integration
    async fn list_by_project(
        &self,
        q: ListByProjectQuery,
        viewer: ActorContext,
    ) -> Result<Vec<Integration>, IntegrationError>;

    /// 取得当前 SyncState(最新一条)
    async fn get_sync_state(
        &self,
        id: IntegrationId,
        viewer: ActorContext,
    ) -> Result<SyncState, IntegrationError>;

    /// 列出 SyncState 历史(Append-only)
    async fn get_history(
        &self,
        q: GetHistoryQuery,
        viewer: ActorContext,
    ) -> Result<Vec<SyncState>, IntegrationError>;
}

// =====================================================================
// 端口:IntegrationRepository(仓储抽象)
// =====================================================================

/// **Integration 仓储端口**(供 SQLx / 内存 / 测试 Adapter 实现)
#[async_trait]
pub trait IntegrationRepository: Send + Sync {
    /// Integration CRUD
    async fn insert_integration(&self, integration: &Integration) -> Result<(), IntegrationError>;
    async fn find_integration_by_id(
        &self,
        id: IntegrationId,
    ) -> Result<Option<Integration>, IntegrationError>;
    async fn update_integration(&self, integration: &Integration) -> Result<(), IntegrationError>;
    async fn delete_integration(&self, id: IntegrationId) -> Result<(), IntegrationError>;
    async fn list_integrations_by_project(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        source_filter: Option<IntegrationSource>,
        relation_type_filter: Option<IntegrationRelationType>,
        state_filter: Option<IntegrationState>,
        active_only: bool,
    ) -> Result<Vec<Integration>, IntegrationError>;

    /// SyncState CRUD
    async fn insert_sync_state(&self, sync_state: &SyncState) -> Result<(), IntegrationError>;
    async fn find_latest_sync_state(
        &self,
        integration_id: IntegrationId,
    ) -> Result<Option<SyncState>, IntegrationError>;
    async fn list_sync_states(
        &self,
        integration_id: IntegrationId,
        limit: u32,
        since: Option<DateTime<Utc>>,
    ) -> Result<Vec<SyncState>, IntegrationError>;

    /// Webhook 幂等性查询(按 integration_id + external_event_id)
    async fn find_sync_state_by_webhook_event(
        &self,
        integration_id: IntegrationId,
        external_event_id: &str,
    ) -> Result<Option<SyncState>, IntegrationError>;
}

// 静默抑制未使用导入
#[allow(dead_code)]
fn _unused_imports() {
    let _ = UserId::new();
}
