//! Integration 不变量检查函数(6 条 INV-I-01~06)
//!
//! 来源: `docs/specs/domain-integration-spec.md` §3
//!
//! 每条实现为独立函数 `pub fn check_invariant_<NN>(...) -> Result<(), IntegrationError>`。
//!
//! **不变量清单**:
//! - INV-I-01: 4 类关系分类必带(Link / Mirror / Bidirectional / PlatformOwned)
//! - INV-I-02: Bidirectional Sync 必须有 Loop 防护(Idempotency Key + Sync Token)
//! - INV-I-03: 必带 tenant_id,跨 tenant 拒绝
//! - INV-I-04: 凭据走 Credential Broker,不存明文
//! - INV-I-05: 每条关系定义 Source / Ownership / Version / External ID / Sync Token / Last Synced / Conflict Strategy
//! - INV-I-06: 默认 Link(WorkItem ↔ GitHub Issue),不反向同步

use crate::entity::Integration;
use crate::error::IntegrationError;
use crate::value_object::{
    ConflictStrategy, IntegrationId, IntegrationRelationType, TenantId,
};

/// 不变量检查函数签名(取 Integration 输入)
pub type InvariantCheck = fn(&Integration) -> Result<(), IntegrationError>;

// =====================================================================
// INV-I-01: 4 类关系分类必带
// =====================================================================

/// **INV-I-01**: 4 类关系分类必带(Link / Mirror / Bidirectional / PlatformOwned)
///
/// 本检查仅作语义校验:确认 relation_type 是 4 类之一(类型系统已保证);
/// 此外校验 `external_system_name` 非空、`external_id` 非空、`external_url` 非空。
pub fn check_invariant_01_relation_type_classified(integration: &Integration) -> Result<(), IntegrationError> {
    if integration.external_system_name.as_str().is_empty() {
        return Err(IntegrationError::InvalidArgument(
            "INV-I-01: external_system_name 不能为空".to_string(),
        ));
    }
    if integration.external_id.as_str().is_empty() {
        return Err(IntegrationError::InvalidArgument(
            "INV-I-01: external_id 不能为空".to_string(),
        ));
    }
    if integration.external_url.is_empty() {
        return Err(IntegrationError::InvalidArgument(
            "INV-I-01: external_url 不能为空".to_string(),
        ));
    }
    // relation_type 类型系统已保证 4 类之一,无需运行时校验
    Ok(())
}

// =====================================================================
// INV-I-02: Bidirectional Sync 必须有 Loop 防护
// =====================================================================

/// **INV-I-02**: Bidirectional Sync 必须有 Loop 防护(Idempotency Key + Sync Token,RISK-027)
///
/// 当 `relation_type = Bidirectional` 时:
/// 1. `sync_token` 必须非空(否则 I-004 拒绝)
/// 2. `conflict_strategy` 应为 `Bidirectional` 变体或显式声明 `ManualReview`
pub fn check_invariant_02_bidirectional_loop_guard(integration: &Integration) -> Result<(), IntegrationError> {
    if !integration.needs_loop_guard() {
        return Ok(());
    }
    if integration.sync_token.is_none() {
        return Err(IntegrationError::LoopGuardMissing(format!(
            "INV-I-02: Bidirectional Sync 缺 sync_token(integration_id={}),无法防止回声(RISK-027)",
            integration.id
        )));
    }
    // 推荐:Bidirectional 必须显式声明 Bidirectional 冲突策略
    if !matches!(
        integration.conflict_strategy,
        ConflictStrategy::Bidirectional { .. } | ConflictStrategy::ManualReview
    ) {
        return Err(IntegrationError::LoopGuardMissing(format!(
            "INV-I-02: Bidirectional Sync 推荐使用 ConflictStrategy::Bidirectional 或 ManualReview(integration_id={})",
            integration.id
        )));
    }
    Ok(())
}

// =====================================================================
// INV-I-03: 必带 tenant_id,跨 tenant 拒绝
// =====================================================================

/// **INV-I-03**: 必带 tenant_id,跨 tenant 拒绝(§6.1,REQ-SEC-001)
pub fn check_invariant_03_tenant_required(
    integration: &Integration,
    expected_tenant: TenantId,
) -> Result<(), IntegrationError> {
    if integration.tenant_id != expected_tenant {
        return Err(IntegrationError::PermissionDenied);
    }
    Ok(())
}

// =====================================================================
// INV-I-04: 凭据走 Credential Broker,不存明文
// =====================================================================

/// **INV-I-04**: 凭据走 Credential Broker,不存明文(security-design §5.4)
///
/// 校验 URL 不含 `user:pass@` 形式的明文凭据(由 `credential_id` 引用 Credential Broker)。
pub fn check_invariant_04_no_plaintext_credential(integration: &Integration) -> Result<(), IntegrationError> {
    if let Some(idx) = integration.external_url.find("://") {
        let after_scheme = &integration.external_url[idx + 3..];
        if after_scheme.contains('@') {
            return Err(IntegrationError::InvalidState(format!(
                "INV-I-04: Integration URL 不应包含明文凭据(应走 Credential Broker,integration_id={})",
                integration.id
            )));
        }
    }
    Ok(())
}

// =====================================================================
// INV-I-05: 每条关系定义 Source / Ownership / Version / External ID / Sync Token / Last Synced / Conflict Strategy
// =====================================================================

/// **INV-I-05**: 字段完整性校验。
///
/// 除 Link 外,其他 3 类关系都应有 sync_token;所有关系必须有 conflict_strategy。
pub fn check_invariant_05_required_fields(integration: &Integration) -> Result<(), IntegrationError> {
    if integration.relation_type.requires_sync_token() && integration.sync_token.is_none() {
        return Err(IntegrationError::InvalidState(format!(
            "INV-I-05: 关系类型 {} 必须有 sync_token(integration_id={})",
            integration.relation_type.as_str(),
            integration.id
        )));
    }
    Ok(())
}

// =====================================================================
// INV-I-06: 4 类关系不混用 + Link 不反向同步
// =====================================================================

/// **INV-I-06**: 4 类关系不混用(每条 Integration 仅一种 relation_type,类型系统已保证);
///
/// **Link 不反向同步**:`Link` 关系不应携带 `sync_token` / `last_synced_at`(只读链接)。
pub fn check_invariant_06_link_no_reverse_sync(integration: &Integration) -> Result<(), IntegrationError> {
    if integration.is_link() {
        if integration.sync_token.is_some() {
            return Err(IntegrationError::InvalidState(format!(
                "INV-I-06: Link 关系不应携带 sync_token(只读链接不反向同步,integration_id={})",
                integration.id
            )));
        }
        if integration.last_synced_at.is_some() {
            return Err(IntegrationError::InvalidState(format!(
                "INV-I-06: Link 关系不应有 last_synced_at(只读链接,integration_id={})",
                integration.id
            )));
        }
    }
    Ok(())
}

// =====================================================================
// 批量执行
// =====================================================================

/// **所有 Integration 级别不变量检查(INV-I-01/02/04/05/06)**
pub const ALL_INVARIANT_CHECKS: &[InvariantCheck] = &[
    check_invariant_01_relation_type_classified,
    check_invariant_02_bidirectional_loop_guard,
    check_invariant_04_no_plaintext_credential,
    check_invariant_05_required_fields,
    check_invariant_06_link_no_reverse_sync,
];

/// 批量执行不变量检查,首次失败即返回错误。
pub fn run_invariants(
    checks: &[InvariantCheck],
    integration: &Integration,
) -> Result<(), IntegrationError> {
    for check in checks {
        check(integration)?;
    }
    Ok(())
}

/// 注册 Integration 时的核心不变量集合(INV-I-01/02/03/04/05/06)
pub fn check_register_invariants(
    integration: &Integration,
    expected_tenant: TenantId,
) -> Result<(), IntegrationError> {
    check_invariant_01_relation_type_classified(integration)?;
    check_invariant_02_bidirectional_loop_guard(integration)?;
    check_invariant_03_tenant_required(integration, expected_tenant)?;
    check_invariant_04_no_plaintext_credential(integration)?;
    check_invariant_05_required_fields(integration)?;
    check_invariant_06_link_no_reverse_sync(integration)?;
    Ok(())
}

// 静默抑制未使用导入
#[allow(dead_code)]
fn _unused_imports() {
    let _ = IntegrationId::new();
    let _: IntegrationRelationType = IntegrationRelationType::Link;
}
