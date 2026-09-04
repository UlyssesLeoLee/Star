//! # star-dto (T3.1 跨域共享 DTO 类型, per HANDOFF-ST-001.md v0.8 §10)
//!
//! **目的**: 22 domain-* crate 跨域共享 DTO 收敛, 减少 Saga 跨域编排时各 domain 重复定义
//! 同一字段名/同一强类型 ID 包装.
//!
//! **演进 (per 守门 #1 + 守门 #12)**:
//! - v0.0.1 (本 session 启动): 4 强类型 ID + ActorContext alias 最小骨架, 跨 sub-session 续
//! - v0.1.0 (T3.1 收官): 22 domain-* DTO 全量 re-export + Saga 跨域事件 DTO
//!
//! **守门 (per HANDOFF v0.8 §10)**:
//! - 字段命名跟 [`docs/ubiquitous-language.md`](../../../docs/ubiquitous-language.md) v1.0 §1 保持一致
//! - 强类型 ID 模式跟 §6 跨域命令/查询/事件命名约定
//! - 修正模式参考 §7 Phase B.4 14 修正

#![allow(missing_docs)] // T3.1 启动 stub, Phase 2 spec 完成后补 doc

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// **TenantId 强类型** (22 domain-* crate 跨域共享)
///
/// 字段约定 per [`docs/ubiquitous-language.md`](../../../docs/ubiquitous-language.md) v1.0 §1:
/// - tuple struct 强类型 `pub struct TenantId(pub Uuid);`
/// - `as_uuid(&self) -> Uuid` (用于 star_context::ActorContext 跨域)
pub use star_context::ActorContext as DtoActorContext;
// 跨域共享强类型 ID 暂不引入依赖, 跨 sub-session 续时从 star-context 重新导出
// (per T3.1 启动原则: stub only, 不引入 cycle 依赖)

/// **跨域 ListByTenantQuery DTO** (v0.0.1 stub)
///
/// 字段约定 per ubiquitous-language.md v1.0 §6.1 Commands/Query 命名 + 必含 `tenant_id: TenantId`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListByTenantDto {
    /// 当前租户 ID (跨域隔离, INV-ACT-01 校验 non-nil)
    pub tenant_id: Uuid,
    /// 列表 limit (默认 50, max 200)
    #[serde(default = "default_limit")]
    pub limit: u32,
    /// 列表 offset (默认 0)
    #[serde(default)]
    pub offset: u32,
}

fn default_limit() -> u32 {
    50
}
