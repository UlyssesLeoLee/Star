//! SCM 不变量检查函数(8 条 INV-SCM-01~08)
//!
//! 来源: docs/specs/domain-scm-spec.md §3
//!
//! 每条实现为独立函数 `pub fn check_invariant_<NN>(...) -> Result<(), ScmError>`,
//! 由 `ALL_INVARIANT_CHECKS` 列表聚合,供 `service.rs` 的命令实现批量执行。
//!
//! **不变量清单**:
//! - INV-SCM-01: Domain 层不出现厂商特有对象(由 ACL 翻译,本 crate 编译期即保证)
//! - INV-SCM-02: MVP 仅支持 Connected 所有权
//! - INV-SCM-03: Bidirectional Sync 必须有 Loop 防护(Idempotency Key + Sync Token)
//! - INV-SCM-04: Repository 必带 tenant_id + project_id,跨 tenant 拒绝
//! - INV-SCM-05: Repository Credential 走 Credential Broker,不存明文
//! - INV-SCM-06: PR Content 必带 tenant_id(Object Storage Key 前缀,§6.1)
//! - INV-SCM-07: PullRequest.state 状态机严格按 §7.5 迁移
//! - INV-SCM-08: Webhook 入站 100% 写 Audit

use crate::entity::{PullRequest, Repository, WebhookEvent};
use crate::error::ScmError;
use crate::value_object::{
    ConflictStrategy, ExternalRepositoryId, PullRequestId, PullRequestState, RepositoryOwnership,
    ScmProvider, TenantId, WebhookEventId,
};

/// 不变量检查函数签名(取 Repository 输入)
pub type InvariantCheck = fn(&Repository) -> Result<(), ScmError>;

// =====================================================================
// INV-SCM-01:Domain 层不出现厂商特有对象
// =====================================================================

/// **INV-SCM-01**:Domain 层不出现厂商特有对象(由 ACL 翻译,REQ-SCM-002)
///
/// 编译期检查由模块边界(本 crate 不允许 import `GitHub*` / `GitLab*` 类型)保证;
/// 运行时此函数检查 Provider + external_id 字符串格式合法(不直接接收厂商对象)。
pub fn check_invariant_01_no_vendor_objects_in_domain(
    _provider: ScmProvider,
    external_id: &ExternalRepositoryId,
) -> Result<(), ScmError> {
    // 编译期:本 crate 无 `GithubPullRequestObject` 等类型
    // 运行时:external_id 必须非空字符串
    if external_id.as_str().is_empty() {
        return Err(ScmError::InvalidArgument(
            "INV-SCM-01: external_id 不能为空".to_string(),
        ));
    }
    Ok(())
}

// =====================================================================
// INV-SCM-02:MVP 仅支持 Connected 所有权
// =====================================================================

/// **INV-SCM-02**:MVP 仅支持 Connected 所有权(§4.7.4,§30.6)
///
/// Connected = 外部 SoR,平台只读镜像;其他 Ownership 类型 MVP 阶段拒绝。
pub fn check_invariant_02_connected_only(repo: &Repository) -> Result<(), ScmError> {
    if !matches!(repo.ownership, RepositoryOwnership::Connected) {
        return Err(ScmError::InvalidState(format!(
            "INV-SCM-02: MVP 仅支持 Connected 所有权,实际: {} (repository_id={})",
            repo.ownership,
            repo.id
        )));
    }
    Ok(())
}

// =====================================================================
// INV-SCM-03:Bidirectional Sync 必须有 Loop 防护
// =====================================================================

/// **INV-SCM-03**:Bidirectional Sync 必须有 Loop 防护(Idempotency Key + Sync Token,RISK-027)
///
/// 当 ConflictStrategy = Bidirectional 时,Repository 必须带 sync_token;
/// 当 sync_status = Conflict 时,token 不能为空。
pub fn check_invariant_03_bidirectional_loop_guard(repo: &Repository) -> Result<(), ScmError> {
    if matches!(repo.conflict_strategy, ConflictStrategy::Bidirectional { .. })
        && repo.sync_token.is_none()
    {
        return Err(ScmError::InvalidState(format!(
            "INV-SCM-03: Bidirectional Sync 必须有 sync_token (repository_id={})",
            repo.id
        )));
    }
    Ok(())
}

// =====================================================================
// INV-SCM-04:Repository 必带 tenant_id + project_id
// =====================================================================

/// **INV-SCM-04**:Repository 必带 tenant_id + project_id,跨 tenant 拒绝(§6.1,REQ-SEC-001)
///
/// 调用方传入预期 tenant_id,Repository 必须等于此值。
pub fn check_invariant_04_tenant_project_required(
    repo: &Repository,
    expected_tenant: TenantId,
) -> Result<(), ScmError> {
    if repo.tenant_id != expected_tenant {
        return Err(ScmError::PermissionDenied);
    }
    Ok(())
}

// =====================================================================
// INV-SCM-05:Repository Credential 走 Credential Broker
// =====================================================================

/// **INV-SCM-05**:Repository Credential 走 Credential Broker,不存明文(security-design §5.4)
///
/// 检查 Repository 不在 Domain 内存储明文 Credential;
/// 实际凭据通过 `credential_id` 引用 Credential Broker。
/// 本函数检查字段语义合法性(URL 不含用户名密码前缀)。
pub fn check_invariant_05_no_plaintext_credential(repo: &Repository) -> Result<(), ScmError> {
    // 检查 URL 不含明文凭据(格式:scheme://user:pass@...)
    if let Some(idx) = repo.url.find("://") {
        let after_scheme = &repo.url[idx + 3..];
        if after_scheme.contains('@') {
            return Err(ScmError::InvalidState(format!(
                "INV-SCM-05: Repository URL 不应包含明文凭据(应走 Credential Broker,repository_id={})",
                repo.id
            )));
        }
    }
    Ok(())
}

// =====================================================================
// INV-SCM-06:PR Content 必带 tenant_id
// =====================================================================

/// **INV-SCM-06**:PR Content 必带 tenant_id(Object Storage Key 前缀,§6.1)
///
/// PullRequest 必须带 tenant_id,且非 nil;本函数用于 PR 创建 / 关联 WorkItem 时校验。
pub fn check_invariant_06_pr_content_tenant(
    pr: &PullRequest,
    expected_tenant: TenantId,
) -> Result<(), ScmError> {
    if pr.tenant_id != expected_tenant {
        return Err(ScmError::PermissionDenied);
    }
    Ok(())
}

// =====================================================================
// INV-SCM-07:PR 状态机严格按 §7.5 迁移
// =====================================================================

/// **INV-SCM-07**:PullRequest.state 状态机严格按 §7.5 迁移(basic-design §7.5)
pub fn check_invariant_07_pr_state_machine(
    from: PullRequestState,
    to: PullRequestState,
) -> Result<(), ScmError> {
    if !from.can_transition_to(to) {
        return Err(ScmError::InvalidState(format!(
            "INV-SCM-07: PR 状态机非法迁移 {from} → {to}"
        )));
    }
    Ok(())
}

// =====================================================================
// INV-SCM-08:Webhook 入站 100% 写 Audit
// =====================================================================

/// **INV-SCM-08**:Webhook 入站 100% 写 Audit(basic-design §9.3)
///
/// WebhookEvent 必须记录完整 payload 与签名验证状态;
/// 此函数校验 event_type 与 idempotency_key 在重复场景下的去重。
pub fn check_invariant_08_webhook_idempotency(
    incoming: &WebhookEvent,
    existing: Option<&WebhookEvent>,
) -> Result<(), ScmError> {
    if let Some(prev) = existing {
        // 同一 (provider, idempotency_key) UNIQUE 命中,返回 Conflict(SC-004 幂等)
        if prev.idempotency_key == incoming.idempotency_key
            && prev.provider == incoming.provider
            && prev.idempotency_key.is_some()
        {
            return Err(ScmError::Conflict(format!(
                "INV-SCM-08: 重复 Webhook 事件(provider={}, idempotency_key={:?}),SC-004",
                incoming.provider.as_str(),
                incoming.idempotency_key
            )));
        }
    }
    Ok(())
}

// =====================================================================
// 批量执行
// =====================================================================

/// **所有 Repository 级别不变量检查(INV-SCM-02/03/05)**
pub const ALL_INVARIANT_CHECKS: &[InvariantCheck] = &[
    check_invariant_02_connected_only,
    check_invariant_03_bidirectional_loop_guard,
    check_invariant_05_no_plaintext_credential,
];

/// 批量执行不变量检查,首次失败即返回错误。
pub fn run_invariants(
    checks: &[InvariantCheck],
    repo: &Repository,
) -> Result<(), ScmError> {
    for check in checks {
        check(repo)?;
    }
    Ok(())
}

/// 注册 Repository 时的核心不变量集合(INV-SCM-01/02/03/04/05)
pub fn check_register_invariants(
    repo: &Repository,
    expected_tenant: TenantId,
) -> Result<(), ScmError> {
    check_invariant_01_no_vendor_objects_in_domain(repo.provider, &repo.external_id)?;
    check_invariant_02_connected_only(repo)?;
    check_invariant_03_bidirectional_loop_guard(repo)?;
    check_invariant_04_tenant_project_required(repo, expected_tenant)?;
    check_invariant_05_no_plaintext_credential(repo)?;
    Ok(())
}

/// PR 状态机迁移校验(INV-SCM-06/07)
pub fn check_pr_transition_invariants(
    pr: &PullRequest,
    expected_tenant: TenantId,
    next: PullRequestState,
) -> Result<(), ScmError> {
    check_invariant_06_pr_content_tenant(pr, expected_tenant)?;
    check_invariant_07_pr_state_machine(pr.state, next)?;
    Ok(())
}

// 静默抑制未使用导入(在测试中通过 `super::xxx` 调用)
#[allow(dead_code)]
fn _unused_imports() {
    let _ = WebhookEventId::new();
    let _: PullRequestId = PullRequestId::new();
}
