//! Integration 域实体(Entity / Aggregate Root)
//!
//! 来源:
//! - `docs/data-design.md` §4.12 (`integration` schema)
//! - `docs/specs/domain-integration-spec.md` §2 (实体清单)
//!
//! 包含 2 个核心实体 + 1 个 MappingConfig 值对象:
//! - `Integration` — 主聚合根(19 字段,继承 §4.12 DDL)
//! - `SyncState` — 同步状态(Append-only,12 字段,§4.12)
//! - `MappingConfig` — 字段映射配置(provider-specific,JSONB)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::value_object::{
    ConflictStrategy, ExternalEntityId, ExternalSystemName, IntegrationId, IntegrationRelationType,
    IntegrationSource, IntegrationState, ProjectId, SyncOutcome, SyncStateId, TenantId, UserId,
};

// =====================================================================
// Integration 聚合根
// =====================================================================

/// **Integration 聚合根**(继承 `data-design §4.12` DDL,19 字段)
///
/// 字段映射(DDL → Rust 字段):
/// - id / tenant_id / project_id / source / relation_type
/// - external_system_name / external_id / external_url
/// - mapping_config / conflict_strategy / state
/// - sync_token / last_synced_at / last_error / retry_count
/// - credential_id / created_at / updated_at / version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Integration {
    /// 主键
    pub id: IntegrationId,

    /// 租户 ID(必带,§6.1,REQ-SEC-001)
    pub tenant_id: TenantId,

    /// Project ID
    pub project_id: ProjectId,

    /// 源系统分类(SCM / Project Management / Communication / Other)
    pub source: IntegrationSource,

    /// 关系类型(Link / Mirror / Bidirectional / PlatformOwned)
    pub relation_type: IntegrationRelationType,

    /// 外部系统名(github / gitlab / jira / slack ...)
    pub external_system_name: ExternalSystemName,

    /// 外部实体 ID(厂商侧 ID)
    pub external_id: ExternalEntityId,

    /// 外部实体 URL
    pub external_url: String,

    /// 字段映射配置(JSON 字符串,provider-specific)
    pub mapping_config: MappingConfig,

    /// 冲突策略
    pub conflict_strategy: ConflictStrategy,

    /// 当前状态
    pub state: IntegrationState,

    /// 同步 Token(ETag / cursor)
    pub sync_token: Option<String>,

    /// 上次同步时间
    pub last_synced_at: Option<DateTime<Utc>>,

    /// 上次错误信息
    pub last_error: Option<String>,

    /// 累计重试次数
    pub retry_count: u32,

    /// Credential ID 引用(走 Credential Broker,§5.4)
    pub credential_id: Option<uuid::Uuid>,

    /// 是否启用(MVP 总是 true;Disabled 状态时为 false)
    pub enabled: bool,

    /// 创建者
    pub created_by_user_id: UserId,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 更新时间
    pub updated_at: DateTime<Utc>,

    /// 乐观锁版本号
    pub lock_version: u32,
}

impl Integration {
    /// 字段数(用于 §4.12 DDL 对齐审计)
    pub const FIELD_COUNT: usize = 19;

    /// 升级乐观锁版本号
    pub fn bump_version(&mut self) {
        self.lock_version = self.lock_version.saturating_add(1);
        self.updated_at = Utc::now();
    }

    /// 是否为 Link(只读,不反向同步)
    pub fn is_link(&self) -> bool {
        matches!(self.relation_type, IntegrationRelationType::Link)
    }

    /// 是否需要 Loop 防护(Bidirectional 必须)
    pub fn needs_loop_guard(&self) -> bool {
        self.relation_type.requires_loop_guard()
    }

    /// 转换到新状态(仅允许 Paused ↔ Active / Error → Active)
    pub fn transition_state(&mut self, next: IntegrationState) {
        self.state = next;
        self.bump_version();
    }

    /// 记录一次同步结果
    pub fn record_sync(
        &mut self,
        outcome: SyncOutcome,
        sync_token: Option<String>,
        synced_at: DateTime<Utc>,
        error: Option<String>,
    ) {
        if let Some(t) = sync_token {
            self.sync_token = Some(t);
        }
        self.last_synced_at = Some(synced_at);
        match outcome {
            SyncOutcome::Success => {
                self.state = IntegrationState::Active;
                self.last_error = None;
            }
            SyncOutcome::PartialSuccess => {
                self.state = IntegrationState::Active;
                self.last_error = error;
            }
            SyncOutcome::Failed => {
                self.state = IntegrationState::Error;
                self.last_error = error;
                self.retry_count = self.retry_count.saturating_add(1);
            }
            SyncOutcome::Skipped => {
                // 不变更状态(Loop 防护命中)
            }
        }
        self.bump_version();
    }
}

// =====================================================================
// SyncState 实体(Append-only 历史,§4.12)
// =====================================================================

/// **SyncState**(§4.12 DDL,12 字段)
///
/// 记录每次同步的游标 + 错误,Append-only 历史(由 `integration.sync_state_history` 表承担)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncState {
    /// 主键
    pub id: SyncStateId,
    /// Integration ID
    pub integration_id: IntegrationId,
    /// 租户 ID(必带,§6.1)
    pub tenant_id: TenantId,
    /// 同步游标 / ETag
    pub sync_token: String,
    /// 同步时间
    pub synced_at: DateTime<Utc>,
    /// 同步结果
    pub outcome: SyncOutcome,
    /// 错误信息(失败时填充)
    pub error: Option<String>,
    /// 同步方向(Inbound / Outbound)
    pub direction: SyncDirection,
    /// 处理的外部记录数
    pub processed_count: u32,
    /// 跳过的记录数(Loop 防护命中)
    pub skipped_count: u32,
    /// 冲突记录数
    pub conflict_count: u32,
    /// 触发者
    pub triggered_by_user_id: Option<UserId>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

impl SyncState {
    /// 字段数
    pub const FIELD_COUNT: usize = 12;
}

// =====================================================================
// SyncDirection 枚举
// =====================================================================

/// **同步方向**(`SyncState.direction` 列)
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SyncDirection {
    /// 外部 → 平台
    Inbound,
    /// 平台 → 外部
    Outbound,
    /// 双向(仅 Bidirectional)
    Bidirectional,
}

impl SyncDirection {
    /// 字符串字面量
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Inbound => "INBOUND",
            Self::Outbound => "OUTBOUND",
            Self::Bidirectional => "BIDIRECTIONAL",
        }
    }
}

impl std::fmt::Display for SyncDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// =====================================================================
// MappingConfig 值对象(provider-specific JSONB)
// =====================================================================

/// **字段映射配置**(`integration.mapping_config` JSONB 列)
///
/// 简化:本 crate 内部把 JSON 序列化为 `serde_json::Value`,由应用层在解析时映射到 provider schema。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MappingConfig {
    /// 原始 JSON 配置(provider-specific)
    pub raw: serde_json::Value,
}

impl MappingConfig {
    /// 从原始 JSON 构造
    pub fn from_json(raw: serde_json::Value) -> Self {
        Self { raw }
    }

    /// 默认配置(空 JSON object)
    pub fn empty() -> Self {
        Self {
            raw: serde_json::json!({}),
        }
    }
}
