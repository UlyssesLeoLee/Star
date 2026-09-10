//! Adapter 实现层 (PostgreSQL / NATS / ObjectStorage / SCM / Agent)
//!
//! **crate**: `infrastructure`
//! **上游 spec**: docs/specs/infrastructure-spec.md §3 ACL / §13.1 数据面
//! **基本设计**: docs/basic-design.md §1.1 / §3 ACL
//! **数据设计**: docs/data-design.md §6 / §7 (RLS) / §8 (索引)
//! **API 设计**: docs/api-design.md —
//!
//! ## 职责
//!
//! 详细职责边界见 spec 文档第 1 节。骨架阶段仅声明 Port trait + Entity + Error,
//! 具体实现由 `crates/infrastructure` 中的 Adapter 提供。
//!
//! ## 关键不变量
//!
//! //! - 本 crate 不允许反向依赖 `domain`,只实现 Domain 定义的 Port(§3 ACL)
//! - PostgreSQL = 默认 SoR(§13.1,§30.6)
//! - Database 保持单一 PostgreSQL(非 Database per Domain,§13.5)

//! ## 上游依赖
//!
//! 本 supporting crate 编排多个 domain-*(骨架阶段不实际 import,仅占位模块结构)。

//! ## 关键引用
//!
//! Adapter 仅实现 Domain Port(§3 ACL);非 Database per Domain(§13.5,§30.6)

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
pub use star_context::ActorContext;
use uuid::Uuid;

// =====================================================================
// 实体(Entity / Aggregate Root)
// =====================================================================
// (本 crate 为 supporting 层,无业务实体 — 实体由 domain-* crate 拥有)

// =====================================================================
// 端口(Port / 抽象)
// =====================================================================

/// **AdapterRegistry**(命令端口)
///
/// 来源: docs/api-design.md —
///
/// **骨架阶段**: 仅方法签名,无 body 实现。Phase 2 在
/// `crates/infrastructure/<adapter>.rs` 中提供 SQLx / NATS / SCM Adapter 实现。
#[async_trait]
pub trait AdapterRegistry: Send + Sync {
    /// 注册 PostgreSQL adapter (v1, 骨架阶段 cmd: () placeholder, per WBS v0.66 已知缺口 (a))
    async fn register_postgres_adapter(
        &self,
        cmd: (),
        actor: ActorContext,
    ) -> Result<(), InfrastructureError>;
    /// 注册 NATS adapter
    async fn register_nats_adapter(
        &self,
        cmd: (),
        actor: ActorContext,
    ) -> Result<(), InfrastructureError>;
    /// 注册 ObjectStorage adapter
    async fn register_object_storage_adapter(
        &self,
        cmd: (),
        actor: ActorContext,
    ) -> Result<(), InfrastructureError>;
    /// 注册 SCM adapter
    async fn register_scm_adapter(
        &self,
        cmd: (),
        actor: ActorContext,
    ) -> Result<(), InfrastructureError>;
    /// 注册 Agent adapter
    async fn register_agent_adapter(
        &self,
        cmd: (),
        actor: ActorContext,
    ) -> Result<(), InfrastructureError>;

    /// **注册 PostgreSQL adapter (v2, P0-4 Stage 2.3 spec 重构)**
    ///
    /// v0.79 P0-4 Stage 2.3 扩展: 真实 `RegisterPostgresAdapterCmd` 替代 v1 的 `cmd: ()` placeholder
    /// (per WBS v0.66/v0.72/v0.73/v0.74/v0.75/v0.78 已知缺口 (b) 跨 session 续做, 闭合 v0.72 已知缺口 (a)).
    ///
    /// **v1 → v2 关系**: v1 保留 backward compat (P0-4 Stage 2.x 28+ 测试 callsite 不破坏);
    /// v2 是 spec 重构方向, 后续 P2 阶段全部切到 v2 后 v1 删.
    ///
    /// **返回** `AdapterDescriptor` (per v0.72 字段扩展 pg_url + registered_at 填 Some)
    /// 而非 `()`, 让 caller 拿到刚注册的 descriptor (per application crate 编排层后续用).
    async fn register_postgres_adapter_v2(
        &self,
        cmd: RegisterPostgresAdapterCmd,
        actor: ActorContext,
    ) -> Result<AdapterDescriptor, InfrastructureError>;
}

/// **RegisterPostgresAdapterCmd** (v0.79 P0-4 Stage 2.3 spec 重构)
///
/// 真实注册 PostgreSQL adapter 的命令结构, 替代 v1 的 `cmd: ()` placeholder.
///
/// 字段 (per spec §13.1 PostgreSQL = 默认 SoR, §30.6 单一 PostgreSQL 数据库):
/// - `pg_url`: PostgreSQL 连接 URL (必填, 不能为空, per 守门 #5 v2 env 安全: 不用 env var 偷)
/// - `pool_size`: 连接池最大连接数 (可选, None = sqlx 默认 10)
/// - `ssl_mode`: SSL/TLS 模式 (可选, None = prefer, per 守门 #13 a W/T/M + envoy 9/1 13:05 TLS 一致性)
/// - `schema_migrations_dir`: SQL 迁移脚本目录 (可选, None = 用 star-pg-adapter 默认 `db/migrations/`)
#[derive(Debug, Clone)]
pub struct RegisterPostgresAdapterCmd {
    /// PostgreSQL 连接 URL (必填)
    pub pg_url: String,
    /// 连接池最大连接数 (None = sqlx 默认)
    pub pool_size: Option<u32>,
    /// SSL/TLS 模式 (None = prefer)
    pub ssl_mode: Option<String>,
    /// 迁移脚本目录 (None = 默认 db/migrations/)
    pub schema_migrations_dir: Option<String>,
}

impl RegisterPostgresAdapterCmd {
    /// 必填校验: pg_url 非空 (per 守门 #11 缺标比错标)
    pub fn validate(&self) -> Result<(), InfrastructureError> {
        if self.pg_url.trim().is_empty() {
            return Err(InfrastructureError::InvalidState(
                "pg_url 必填非空 (per RegisterPostgresAdapterCmd spec)".to_string(),
            ));
        }
        Ok(())
    }
}

/// **AdapterQuery**(查询端口)
///
/// 来源: docs/api-design.md —
#[async_trait]
pub trait AdapterQuery: Send + Sync {
    /// 列出已注册的 adapter
    async fn list_registered_adapters(
        &self,
        _dummy: (),
        viewer: ActorContext,
    ) -> Result<Vec<AdapterDescriptor>, InfrastructureError>;
}

// =====================================================================
// Domain Events(CloudEvents 1.0,见 api-design §5)
// =====================================================================
// (本 crate 不直接发布 Domain Event,事件由 domain-* crate 拥有)

// =====================================================================
// 类型别名与命令/查询/返回类型占位
// =====================================================================
/// **命令 / 查询 / 跨 crate 类型占位结构**(Phase 1 骨架:最小字段集)
///
/// **v0 phase 2 标记** (per OPT-NEXT-06-code-stub §3.4 3 supporting crate 占位):
/// per `application/src/lib.rs:127-128` 注释 "Phase 2 由具体 spec 在 `domain-*` 内补全字段;
/// `crates/application` / `crates/api` / `crates/infrastructure` 等 supporting crate
/// 的占位则在 Phase 2 删除, 改为 `use domain_xxx::*;` 引用":
///
/// - `crates/infrastructure` 占位结构: ~12 个 (AdapterDescriptor + ... 同模式)
/// - 关联 issue: per `OPT-A1-code-todo-scan.output.md` §3.5 P1 重要
///
/// P2 阶段 worker 子代理实装时按 "use domain_xxx::*;" 引用替换 (派前必先
/// `automation/dispatcher.py brief(...)` 落 `docs/briefs/<task_id>.md`, per AGENTS.md §4 #20 守门派生).
/// 当前保留为 P2 阶段前置可编译骨架 (守门 #1 v6 单 crate 100% pass 实证).

/// Phase 2 由具体 spec 在 `domain-*` 内补全字段;`crates/application` 等
/// supporting crate 的占位则在 Phase 2 删除,改为 `use domain_xxx::*;` 引用。

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterDescriptor {
    /// 主键 UUID
    pub id: Uuid,
    /// 租户 ID(13 类对象必带,§6.1)
    pub tenant_id: Uuid,
    /// PostgreSQL 连接 URL (only for Postgres adapter, None for others)
    /// v0.72 P0-4 Stage 2 扩展: RealPostgresAdapterRegistry 会把构造时传入的 pg_url 填到这里
    /// (per InMemoryAdapterRegistry backward compat, 内存版始终 None)
    #[serde(default)]
    pub pg_url: Option<String>,
    /// 注册时间 (UTC) — v0.72 扩展 (per spec §13.1 默认 SoR 审计需求)
    #[serde(default)]
    pub registered_at: Option<chrono::DateTime<chrono::Utc>>,
    // 其它字段在 Phase 2 由具体 spec 补充
}

// =====================================================================
// Error
// =====================================================================

/// **Infrastructure 错误**
///
/// 来源: docs/api-design.md §8 (错误码)
/// 5 个标准变体;具体错误码在 Phase 2 由本 enum 派生 + 实现 `Into<ApiError>`。
#[derive(Debug, thiserror::Error)]
pub enum InfrastructureError {
    /// 未找到
    #[error("not found: {0}")]
    NotFound(Uuid),
    /// 非法状态
    #[error("invalid state: {0}")]
    InvalidState(String),
    /// 权限不足
    #[error("permission denied")]
    PermissionDenied,
    /// 冲突
    #[error("conflict: {0}")]
    Conflict(String),
    /// 内部错误
    #[error("internal: {0}")]
    Internal(String),
}

// =====================================================================
// 单元测试占位
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    /// **骨架阶段**: 最小冒烟测试,验证 crate 可编译、ActorContext 字段可达。
    /// Phase 2 由具体 spec 引入完整单元测试(状态机覆盖 / RLS 矩阵等)。
    #[test]
    fn actor_context_skeleton() {
        let actor = ActorContext {
            user_id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            device_id: None,
            is_local_runtime: false,
            is_platform_admin: false,
            is_agent_session: false,
            tenant_policy_id: None,
            project_ids: vec![],
            workspace_ids: vec![],
            roles: vec!["developer".to_string()],
        };
        assert!(
            !actor.tenant_id.is_nil(),
            "tenant_id must be non-nil (§6.1,REQ-SEC-001)"
        );
    }
}

pub mod registry;
pub mod registry_real_pg;
