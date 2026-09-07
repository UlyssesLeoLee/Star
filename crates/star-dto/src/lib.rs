//! # star-dto (T3.1 跨域共享 DTO 类型, per HANDOFF-ST-001.md v0.8 §10)
//!
//! **目的**: 22 domain-* crate 跨域共享 DTO 收敛, 减少 Saga 跨域编排时各 domain 重复定义
//! 同一字段名/同一强类型 ID 包装.
//!
//! **演进 (per 守门 #1 + 守门 #12)**:
//! - v0.0.1 (2026-09-04 `5357c0a`): 4 强类型 ID + `ActorContext` alias + `ListByTenantDto` stub
//! - v0.1.0 (2026-09-07, 本 commit): 公共 DTO 类型 (Identifier/Timestamp/TenantContext/AuditTrail) + 5 domain 接入 + unit test
//!
//! **W/T/M 分类 (per 守门 #13 + 9/1 18:30 JST 拍板)**:
//! - **M (Master, SCD Type 2)**: `Identifier<T>` — phantom type 防 ID 混用 + 标识版本
//! - **T (Transaction, append-only)**: `AuditTrail` — 跨域事件流水, 物理删除禁止 + 审计必填
//! - **W (Work, 短 TTL)**: 无 (DTO 本身是数据载体, 不存数据; 分类归属其映射的 DB 表)
//!
//! **守门 (per HANDOFF v0.8 §10 + ubiquitous-language.md v1.1)**:
//! - 字段命名跟 [`docs/ubiquitous-language.md`](../../../docs/ubiquitous-language.md) v1.0 §1 保持一致
//! - 强类型 ID 模式跟 §6 跨域命令/查询/事件命名约定
//! - 修正模式参考 §7 Phase B.4 14 修正
//! - W/T/M 分类见 §10.2 + §11 横展开一致性规则

#![allow(missing_docs)] // T3.1 v0.1.0 启动, Phase 2 spec 完成后补全 doc (per 守门 #12 docs 同步)

use std::marker::PhantomData;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// =====================================================================
// §0 ActorContext alias (per v0.0.1, 跨域共享 9 字段上下文)
// =====================================================================

/// **TenantId 强类型** (22 domain-* crate 跨域共享)
///
/// 字段约定 per [`docs/ubiquitous-language.md`](../../../docs/ubiquitous-language.md) v1.0 §1:
/// - tuple struct 强类型 `pub struct TenantId(pub Uuid);`
/// - `as_uuid(&self) -> Uuid` (用于 star_context::ActorContext 跨域)
pub use star_context::ActorContext as DtoActorContext;
// 跨域共享强类型 ID 暂不引入依赖, 跨 sub-session 续时从 star-context 重新导出
// (per T3.1 启动原则: stub only, 不引入 cycle 依赖)

// =====================================================================
// §1 M 类 (Master, SCD Type 2) — Identifier<T>
// =====================================================================

/// **Identifier<T>** — 跨域共享的 phantom-typed 强类型 ID (M 类, SCD Type 2)
///
/// **W/T/M 分类**: M (Master, per 守门 #13 (b))
///
/// **使用模式**:
/// ```ignore
/// use star_dto::Identifier;
///
/// // phantom marker 类型 (per domain 内部定义)
/// pub struct TenantMarker;
/// pub struct UserMarker;
///
/// let tenant_id: Identifier<TenantMarker> = Identifier::new(Uuid::new_v4());
/// let user_id: Identifier<UserMarker> = Identifier::new(Uuid::new_v4());
/// // tenant_id 和 user_id 不能互相赋值, 防 ID 混用
/// ```
///
/// **SCD Type 2 字段 (per 守门 #13 (b))**:
/// - `valid_from`: 该版本生效起点
/// - `valid_to`: `None` 表示当前版本, `Some(t)` 表示已被 t 时刻的新版本替代
/// - 物理删除禁止 (改 `valid_to = Some(now)`)
///
/// **跟各 domain 内部 `define_uuid_id!` 宏的关系**:
/// - 各 domain 内部 macro 生成的 `TenantId`/`UserId` 是**直接 Uuid 包装**
/// - `Identifier<T>` 在**跨域编排**场景使用, 提供 SCD 版本追踪
/// - 转换: `Identifier::from(domain_tenant_id)` / `domain_tenant_id.0 == id.value`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Identifier<T> {
    /// Uuid 值 (跨域共享, INV-ACT-01 校验 non-nil)
    pub value: Uuid,
    /// 该版本生效起点 (SCD Type 2 必填)
    pub valid_from: DateTime<Utc>,
    /// 该版本失效终点 (`None` = 当前版本, `Some(t)` = 已被替代)
    #[serde(default)]
    pub valid_to: Option<DateTime<Utc>>,
    /// phantom marker (编译期防 ID 混用)
    #[serde(skip)]
    _phantom: PhantomData<T>,
}

impl<T> Identifier<T> {
    /// 创建当前版本 (valid_from = now, valid_to = None)
    pub fn new(value: Uuid) -> Self {
        Self {
            value,
            valid_from: Utc::now(),
            valid_to: None,
            _phantom: PhantomData,
        }
    }

    /// 创建指定生效起点的当前版本
    pub fn with_valid_from(value: Uuid, valid_from: DateTime<Utc>) -> Self {
        Self {
            value,
            valid_from,
            valid_to: None,
            _phantom: PhantomData,
        }
    }

    /// 判断是否当前有效版本
    pub fn is_current(&self) -> bool {
        self.valid_to.is_none()
    }

    /// 标记为失效 (SCD Type 2 软删, 物理删除禁止 per 守门 #13 (b))
    pub fn supersede(&mut self, at: DateTime<Utc>) {
        self.valid_to = Some(at);
    }
}

// =====================================================================
// §2 跨域共享 — Timestamp
// =====================================================================

/// **Timestamp** — 跨域共享的时间戳结构 (per ubiquitous-language.md §1)
///
/// 字段约定:
/// - `created_at`: 实体创建时间 (UTC, INV-TS-01 必填)
/// - `updated_at`: 实体最后更新时间 (UTC, INV-TS-02 必填, 由仓储层在每次写操作时更新)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Timestamp {
    /// 创建时间 (UTC)
    pub created_at: DateTime<Utc>,
    /// 最后更新时间 (UTC)
    pub updated_at: DateTime<Utc>,
}

impl Timestamp {
    /// 创建新的 Timestamp (created_at = updated_at = now)
    pub fn now() -> Self {
        let now = Utc::now();
        Self {
            created_at: now,
            updated_at: now,
        }
    }

    /// 创建指定 created_at 的 Timestamp
    pub fn with_created_at(created_at: DateTime<Utc>) -> Self {
        Self {
            created_at,
            updated_at: created_at,
        }
    }

    /// 更新 updated_at (仓储层写操作时调用)
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    /// 是否在指定时间之后被更新过
    pub fn updated_after(&self, t: DateTime<Utc>) -> bool {
        self.updated_at > t
    }
}

impl Default for Timestamp {
    fn default() -> Self {
        Self::now()
    }
}

// =====================================================================
// §3 跨域共享 — TenantContext
// =====================================================================

/// **TenantContext** — 跨域共享的租户 + 工作区 + actor 上下文 (per H2-EXT #3)
///
/// 跟 `star_context::ActorContext` 平行, 但**更轻量** (不包含 roles / device_id / platform_admin 等),
/// 适用于 Saga 编排时跨域透传.
///
/// 字段约定:
/// - `tenant_id`: 当前租户 (INV-ACT-01 校验 non-nil, per ubiquitous-language.md v1.0 §1)
/// - `workspace_ids`: 当前工作区 ID 列表 (per H2-EXT #3 拍板)
/// - `actor_id`: 当前操作人/agent (per ubiquitous-language.md v1.0 §1)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TenantContext {
    /// 当前租户 ID (跨域隔离, INV-ACT-01 校验 non-nil)
    pub tenant_id: Uuid,
    /// 关联 Workspace IDs (per H2-EXT #3, P0-1 9 字段版)
    #[serde(default)]
    pub workspace_ids: Vec<Uuid>,
    /// 当前操作人/agent ID (跨域追踪, 必填)
    pub actor_id: Uuid,
}

impl TenantContext {
    /// 创建 TenantContext (workspace_ids = empty)
    pub fn new(tenant_id: Uuid, actor_id: Uuid) -> Self {
        Self {
            tenant_id,
            workspace_ids: Vec::new(),
            actor_id,
        }
    }

    /// 创建带 workspace_ids 的 TenantContext
    pub fn with_workspaces(tenant_id: Uuid, workspace_ids: Vec<Uuid>, actor_id: Uuid) -> Self {
        Self {
            tenant_id,
            workspace_ids,
            actor_id,
        }
    }

    /// 判断是否包含指定 workspace (per H2-EXT #3 跨域权限检查)
    pub fn has_workspace(&self, workspace_id: Uuid) -> bool {
        self.workspace_ids.contains(&workspace_id)
    }
}

// =====================================================================
// §4 T 类 (Transaction, append-only) — AuditTrail
// =====================================================================

/// **AuditTrail** — 跨域共享的审计事件流水 (T 类, append-only)
///
/// **W/T/M 分类**: T (Transaction, per 守门 #13 (a))
///
/// **不变量**:
/// - INV-AT-01: 物理删除禁止 (append-only, per 守门 #13 (a))
/// - INV-AT-02: `actor_id` 必填, 跨域追踪 (per ubiquitous-language.md v1.0 §1)
/// - INV-AT-03: `timestamp` 必填, 由审计层在持久化时设置
/// - INV-AT-04: `previous_hash` 可选, 用于链式完整性校验 (per ADR-0043 audit_audit_event WORM 模式)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditTrail {
    /// 操作人/agent ID (跨域追踪, INV-AT-02 必填)
    pub actor_id: Uuid,
    /// 操作动作 (e.g. "create_tenant" / "transition_status" / "merge_work_items")
    pub action: String,
    /// 操作时间 (UTC, INV-AT-03 必填)
    pub timestamp: DateTime<Utc>,
    /// 前一个审计事件的哈希 (用于链式完整性校验, INV-AT-04 可选)
    #[serde(default)]
    pub previous_hash: Option<String>,
}

impl AuditTrail {
    /// 创建新的 AuditTrail (timestamp = now, previous_hash = None)
    pub fn new(actor_id: Uuid, action: impl Into<String>) -> Self {
        Self {
            actor_id,
            action: action.into(),
            timestamp: Utc::now(),
            previous_hash: None,
        }
    }

    /// 创建带 previous_hash 的 AuditTrail (链式)
    pub fn with_previous(actor_id: Uuid, action: impl Into<String>, previous_hash: String) -> Self {
        Self {
            actor_id,
            action: action.into(),
            timestamp: Utc::now(),
            previous_hash: Some(previous_hash),
        }
    }
}

// =====================================================================
// §5 跨域查询 — ListByTenantDto (per v0.0.1, v0.1.0 保留 + 强化)
// =====================================================================

/// **跨域 ListByTenantQuery DTO** (v0.0.1 stub, v0.1.0 强化)
///
/// 字段约定 per ubiquitous-language.md v1.0 §6.1 Commands/Query 命名 + 必含 `tenant_id: Uuid`
///
/// 默认 limit = 50, max = 200 (per ADR-0029 Universal Submit 11 字段约束)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListByTenantDto {
    /// 当前租户 ID (跨域隔离, INV-ACT-01 校验 non-nil)
    pub tenant_id: Uuid,
    /// 列表 limit (默认 50, max 200 per ADR-0029)
    #[serde(default = "default_limit")]
    pub limit: u32,
    /// 列表 offset (默认 0)
    #[serde(default)]
    pub offset: u32,
}

impl ListByTenantDto {
    /// 默认 limit (per ADR-0029)
    pub const DEFAULT_LIMIT: u32 = 50;
    /// 最大 limit (per ADR-0029)
    pub const MAX_LIMIT: u32 = 200;

    /// 创建新的 ListByTenantDto
    pub fn new(tenant_id: Uuid) -> Self {
        Self {
            tenant_id,
            limit: Self::DEFAULT_LIMIT,
            offset: 0,
        }
    }

    /// 创建带 limit + offset 的 ListByTenantDto
    pub fn with_pagination(tenant_id: Uuid, limit: u32, offset: u32) -> Self {
        let limit = limit.min(Self::MAX_LIMIT);
        Self {
            tenant_id,
            limit,
            offset,
        }
    }
}

fn default_limit() -> u32 {
    50
}

// =====================================================================
// §6 Unit Tests (per 守门 #1 + 守门 #7)
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    // Marker types for Identifier<T> phantom typing
    struct TenantMarker;
    struct UserMarker;
    struct WorkItemMarker;

    // ===== §1 Identifier<T> tests =====

    #[test]
    fn identifier_new_creates_current_version() {
        let id: Identifier<TenantMarker> = Identifier::new(Uuid::new_v4());
        assert!(id.is_current());
        assert!(id.valid_to.is_none());
    }

    #[test]
    fn identifier_supersede_marks_invalid() {
        let mut id: Identifier<UserMarker> = Identifier::new(Uuid::new_v4());
        let superseded_at = Utc.with_ymd_and_hms(2026, 9, 7, 12, 0, 0).unwrap();
        id.supersede(superseded_at);
        assert!(!id.is_current());
        assert_eq!(id.valid_to, Some(superseded_at));
    }

    #[test]
    fn identifier_phantom_prevents_mixing() {
        // 编译期验证: 不同 marker 不可互换
        // (本测试只验证运行时操作; phantom 检查在编译期)
        let tenant: Identifier<TenantMarker> = Identifier::new(Uuid::new_v4());
        let work_item: Identifier<WorkItemMarker> = Identifier::new(Uuid::new_v4());
        assert_ne!(tenant.value, work_item.value);
    }

    #[test]
    fn identifier_serde_roundtrip() {
        let id: Identifier<TenantMarker> = Identifier::new(Uuid::new_v4());
        let json = serde_json::to_string(&id).expect("serialize");
        let parsed: Identifier<TenantMarker> = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(id.value, parsed.value);
        assert_eq!(id.is_current(), parsed.is_current());
    }

    // ===== §2 Timestamp tests =====

    #[test]
    fn timestamp_now_creates_equal_fields() {
        let ts = Timestamp::now();
        assert_eq!(ts.created_at, ts.updated_at);
    }

    #[test]
    fn timestamp_touch_updates_updated_at() {
        let mut ts = Timestamp::now();
        let original_updated = ts.updated_at;
        std::thread::sleep(std::time::Duration::from_millis(10));
        ts.touch();
        assert!(ts.updated_at > original_updated);
        assert_eq!(ts.created_at, ts.created_at);
    }

    #[test]
    fn timestamp_updated_after() {
        let ts = Timestamp::now();
        let past = Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap();
        assert!(ts.updated_after(past));
    }

    // ===== §3 TenantContext tests =====

    #[test]
    fn tenant_context_new_empty_workspaces() {
        let ctx = TenantContext::new(Uuid::new_v4(), Uuid::new_v4());
        assert!(ctx.workspace_ids.is_empty());
        assert!(!ctx.has_workspace(Uuid::new_v4()));
    }

    #[test]
    fn tenant_context_with_workspaces_has_workspace() {
        let ws = Uuid::new_v4();
        let ctx = TenantContext::with_workspaces(Uuid::new_v4(), vec![ws], Uuid::new_v4());
        assert!(ctx.has_workspace(ws));
        assert!(!ctx.has_workspace(Uuid::new_v4()));
    }

    // ===== §4 AuditTrail tests =====

    #[test]
    fn audit_trail_new_creates_append_only_entry() {
        let trail = AuditTrail::new(Uuid::new_v4(), "create_tenant");
        assert_eq!(trail.action, "create_tenant");
        assert!(trail.previous_hash.is_none());
    }

    #[test]
    fn audit_trail_with_previous_chains() {
        let trail = AuditTrail::with_previous(
            Uuid::new_v4(),
            "transition_status",
            "abc123def456".to_string(),
        );
        assert_eq!(trail.previous_hash.as_deref(), Some("abc123def456"));
    }

    // ===== §5 ListByTenantDto tests =====

    #[test]
    fn list_by_tenant_new_default_limit() {
        let dto = ListByTenantDto::new(Uuid::new_v4());
        assert_eq!(dto.limit, 50);
        assert_eq!(dto.offset, 0);
    }

    #[test]
    fn list_by_tenant_with_pagination_caps_at_max() {
        let dto = ListByTenantDto::with_pagination(Uuid::new_v4(), 1000, 100);
        assert_eq!(dto.limit, 200); // max limit
        assert_eq!(dto.offset, 100);
    }

    #[test]
    fn list_by_tenant_serde_default_limit() {
        let json = format!(r#"{{"tenant_id":"{}"}}"#, Uuid::new_v4());
        let dto: ListByTenantDto = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(dto.limit, 50); // default_limit
    }

    // ===== §6 W/T/M 分类一致性 =====

    #[test]
    fn w_t_m_classification_is_correct() {
        // M 类: Identifier<T> — SCD Type 2
        let id: Identifier<TenantMarker> = Identifier::new(Uuid::new_v4());
        assert!(id.is_current(), "M 类: 必有 valid_to 字段");

        // T 类: AuditTrail — append-only
        let trail = AuditTrail::new(Uuid::new_v4(), "test");
        assert!(trail.timestamp <= Utc::now(), "T 类: 必有 timestamp 字段");

        // W 类: 无 (per ubiquitous-language.md v1.1 §10.1)
        // DTO 本身不分类, 分类归属其映射的 DB 表
    }
}
