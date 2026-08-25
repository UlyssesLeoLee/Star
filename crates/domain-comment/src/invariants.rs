//! Comment 不变量(6 条 INV-C-01~06)
//!
//! 来源: docs/specs/domain-comment-spec.md §3

use crate::entity::{Attachment, Comment};
use crate::error::CommentError;
use crate::value_object::{TenantId, UserId};

// =====================================================================
// INV-C-01:Comment 必带 tenant_id
// =====================================================================

/// **INV-C-01**:Comment 必带 tenant_id,跨 tenant 拒绝
pub fn check_invariant_01_tenant_id_present(c: &Comment) -> Result<(), CommentError> {
    if c.tenant_id.as_uuid().is_nil() {
        return Err(CommentError::InvalidState(
            "INV-C-01: Comment 必带 tenant_id".to_string(),
        ));
    }
    if c.project_id.as_uuid().is_nil() {
        return Err(CommentError::InvalidState(
            "INV-C-01: Comment 必带 project_id".to_string(),
        ));
    }
    if c.body.trim().is_empty() {
        return Err(CommentError::InvalidState(
            "INV-C-03: body 不能为空".to_string(),
        ));
    }
    Ok(())
}

// =====================================================================
// INV-C-02:Comment ≠ Feedback(由 UI 区分,本函数仅占位)
// =====================================================================

/// **INV-C-02**:Comment 不替代 Feedback(本函数仅占位,实际由 UI 区分)
pub fn check_invariant_02_not_feedback(_: &Comment) -> Result<(), CommentError> {
    Ok(())
}

// =====================================================================
// INV-C-03:Attachment Object Storage Key 必带 tenant_id 前缀
// =====================================================================

/// **INV-C-03**:Attachment object_key 必带 tenant_id 前缀
pub fn check_invariant_03_object_key_tenant_prefix(
    a: &Attachment,
) -> Result<(), CommentError> {
    let prefix = format!("{}/", a.tenant_id);
    if !a.object_key.starts_with(&prefix) {
        return Err(CommentError::InvalidState(format!(
            "INV-C-03: object_key '{}' 缺少 tenant_id 前缀 '{}'",
            a.object_key, prefix
        )));
    }
    Ok(())
}

// =====================================================================
// INV-C-04:删除是软删除(INV-C-04 由 service 层处理,本函数占位)
// =====================================================================

/// **INV-C-04**:删除是软删除(本函数仅占位,实际在 delete_comment 中置 deleted_at)
pub fn check_invariant_04_soft_delete_placeholder() -> Result<(), CommentError> {
    Ok(())
}

// =====================================================================
// INV-C-05:AI 提的 Comment author_agent_id 必带
// =====================================================================

/// **INV-C-05**:若 author 标志为 AI,则 author_agent_id 必非空
pub fn check_invariant_05_agent_required(
    c: &Comment,
    is_agent_session: bool,
) -> Result<(), CommentError> {
    if is_agent_session && c.author_agent_id.is_none() {
        return Err(CommentError::InvalidState(
            "INV-C-05: AI 提的 Comment author_agent_id 必带".to_string(),
        ));
    }
    Ok(())
}

// =====================================================================
// INV-C-06:@mention 自动触发 Notification(由 service 在 create 时发布 MentionNotified 事件)
// =====================================================================

/// **INV-C-06** 占位
pub fn check_invariant_06_mention_notified_placeholder() -> Result<(), CommentError> {
    Ok(())
}

// =====================================================================
// body 长度限制(默认 10K 字符)
// =====================================================================

/// **body 长度限制**(C-003)
pub fn check_body_length(body: &str, max_chars: usize) -> Result<(), CommentError> {
    if body.chars().count() > max_chars {
        return Err(CommentError::InvalidState(format!(
            "C-003: body 长度超过 {max_chars} 字符"
        )));
    }
    Ok(())
}

/// **附件大小限制**(C-005,默认 50MB)
pub fn check_attachment_size(size: u64, max_bytes: u64) -> Result<(), CommentError> {
    if size > max_bytes {
        return Err(CommentError::InvalidState(format!(
            "C-005: 附件大小 {size} 超过 {max_bytes} bytes"
        )));
    }
    Ok(())
}

// =====================================================================
// 批量执行
// =====================================================================

pub type InvariantCheck = fn(&Comment) -> Result<(), CommentError>;

pub const ALL_INVARIANT_CHECKS: &[InvariantCheck] = &[];

pub fn run_invariants(checks: &[InvariantCheck], c: &Comment) -> Result<(), CommentError> {
    for check in checks {
        check(c)?;
    }
    Ok(())
}

pub fn check_create_invariants(
    c: &Comment,
    is_agent_session: bool,
) -> Result<(), CommentError> {
    check_invariant_01_tenant_id_present(c)?;
    check_invariant_02_not_feedback(c)?;
    check_invariant_05_agent_required(c, is_agent_session)?;
    check_body_length(&c.body, 10_000)?;
    Ok(())
}

// 静默引用
#[allow(dead_code)]
fn _unused_id(u: UserId) -> UserId {
    u
}
#[allow(dead_code)]
fn _unused_tenant(t: TenantId) -> TenantId {
    t
}
