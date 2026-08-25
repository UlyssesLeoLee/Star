//! Collaboration 不变量检查函数(INV-CB-01~08)
//!
//! 来源: docs/specs/domain-collaboration-spec.md §3
//!
//! 每条实现为独立函数 `pub fn check_invariant_<NN>_<name>(...) -> Result<(), CollaborationError>`,
//! 由 `ALL_INVARIANT_CHECKS` 列表聚合,供 `service.rs` 的命令实现批量执行。
//!
//! **不变量清单**:
//! - INV-CB-01: 必带 tenant_id,跨 tenant 拒绝(basic-design §6.1, REQ-SEC-001)
//! - INV-CB-02: 单 Connection ≤ 100 Subscription(api-design §4.2)
//! - INV-CB-03: Presence 60s 心跳过期(spec §2)
//! - INV-CB-04: Realtime Event 必带 tenant_id(AuthorizationChecker 校验)
//! - INV-CB-05: Subscription filter resource_types 非空(spec §4.4)
//! - INV-CB-06: Project 范围必匹配 Session 的 project_id(防止跨 Project 注入)
//! - INV-CB-07: Session owner 才能 close / delete(权限隔离)
//! - INV-CB-08: Cursor 选区范围 position_x / position_y / selection_start / selection_end 合法

use crate::entity::{CollaborationSession, PresenceCursor, PresenceParticipant, RealtimeChannel};
use crate::error::CollaborationError;
use crate::value_object::{ProjectId, TenantId, UserId};

/// 不变量检查函数签名(取 Session 输入)
pub type SessionCheck = fn(&CollaborationSession) -> Result<(), CollaborationError>;
/// 不变量检查函数签名(取 Participant 输入)
pub type ParticipantCheck = fn(&PresenceParticipant) -> Result<(), CollaborationError>;
/// 不变量检查函数签名(取 Channel 输入)
pub type ChannelCheck = fn(&RealtimeChannel) -> Result<(), CollaborationError>;
/// 不变量检查函数签名(取 Cursor 输入)
pub type CursorCheck = fn(&PresenceCursor) -> Result<(), CollaborationError>;

// =====================================================================
// INV-CB-01:必带 tenant_id,跨 tenant 拒绝
// =====================================================================

/// **INV-CB-01**:必带 tenant_id,跨 tenant 拒绝(basic-design §6.1, REQ-SEC-001)
///
/// `actor.tenant_id` 必须与 `expected.tenant_id` 一致,否则 PermissionDenied。
pub fn check_invariant_01_tenant_id_present(
    actor_tenant_id: TenantId,
    expected_tenant_id: TenantId,
) -> Result<(), CollaborationError> {
    if actor_tenant_id != expected_tenant_id {
        return Err(CollaborationError::PermissionDenied);
    }
    Ok(())
}

// =====================================================================
// INV-CB-02:单 Connection ≤ 100 Subscription
// =====================================================================

/// **INV-CB-02**:单 Connection ≤ 100 Subscription(api-design §4.2, spec §8 CB-002)
///
/// 当 channel_count ≥ 100 时,新增 Subscribe 拒绝。
pub fn check_invariant_02_channel_quota(
    connection_id_label: &str,
    channel_count: usize,
) -> Result<(), CollaborationError> {
    const MAX_CHANNELS_PER_CONNECTION: usize = 100;
    if channel_count >= MAX_CHANNELS_PER_CONNECTION {
        return Err(CollaborationError::RateLimited(format!(
            "INV-CB-02: Connection '{connection_id_label}' 已达 {channel_count} 个 Channel 上限,新订阅拒绝 (CB-002)"
        )));
    }
    Ok(())
}

// =====================================================================
// INV-CB-03:Presence 60s 心跳过期
// =====================================================================

/// **INV-CB-03**:Presence 60s 心跳过期(spec §2, basic-design §23.4)
///
/// 距 `last_active_at` 超过 60s 的 Participant 视为 OFFLINE。
pub fn check_invariant_03_heartbeat_not_expired(
    p: &PresenceParticipant,
    now: chrono::DateTime<chrono::Utc>,
) -> Result<(), CollaborationError> {
    const HEARTBEAT_TIMEOUT_SECS: i64 = 60;
    if p.is_stale(now, HEARTBEAT_TIMEOUT_SECS) {
        return Err(CollaborationError::Timeout(format!(
            "INV-CB-03: Participant {} 心跳已过期 (last_active_at={}) (CB-004)",
            p.id, p.last_active_at
        )));
    }
    Ok(())
}

// =====================================================================
// INV-CB-04:Realtime Event 必带 tenant_id,跨 tenant 拒绝
// =====================================================================

/// **INV-CB-04**:Realtime Event 必带 tenant_id 且 AuthorizationChecker 校验一致
/// (api-design §4.4, security-design §3.5)
///
/// `event_tenant_id` 必须与 `subscriber_tenant_id` 一致,否则视为跨租户推送尝试。
pub fn check_invariant_04_event_tenant_match(
    subscriber_tenant_id: TenantId,
    event_tenant_id: TenantId,
) -> Result<(), CollaborationError> {
    if subscriber_tenant_id != event_tenant_id {
        return Err(CollaborationError::PermissionDenied);
    }
    Ok(())
}

// =====================================================================
// INV-CB-05:Subscription filter resource_types 非空
// =====================================================================

/// **INV-CB-05**:Subscription filter.resource_types 非空(spec §4.4)
///
/// 空 filter 表示通配订阅,可能导致 Cross-Resource 信息泄露,故拒绝。
pub fn check_invariant_05_channel_filter_not_empty(
    c: &RealtimeChannel,
) -> Result<(), CollaborationError> {
    if c.filter_resource_types.is_empty() {
        return Err(CollaborationError::InvalidState(
            "INV-CB-05: RealtimeChannel.filter_resource_types 不能为空,必须至少指定 1 个 ResourceType"
                .to_string(),
        ));
    }
    Ok(())
}

// =====================================================================
// INV-CB-06:Project 范围必匹配 Session 的 project_id
// =====================================================================

/// **INV-CB-06**:Project 范围必匹配 Session 的 project_id(防止跨 Project 注入)
///
/// `actor_project_id` 必须 ∈ Session.filter_project_ids 或 filter_project_ids 为空(全 Project 范围)。
pub fn check_invariant_06_project_scope_match(
    actor_project_id: ProjectId,
    session_filter_project_ids: &[ProjectId],
) -> Result<(), CollaborationError> {
    if session_filter_project_ids.is_empty() {
        return Ok(());
    }
    if session_filter_project_ids.contains(&actor_project_id) {
        Ok(())
    } else {
        Err(CollaborationError::PermissionDenied)
    }
}

// =====================================================================
// INV-CB-07:Session owner 才能 close / delete
// =====================================================================

/// **INV-CB-07**:Session owner 才能 close / delete(权限隔离)
///
/// `actor_user_id` 必须 == Session.owner_user_id,或 actor 为 tenant_admin。
pub fn check_invariant_07_owner_or_admin(
    actor_user_id: UserId,
    actor_is_tenant_admin: bool,
    session_owner_user_id: UserId,
) -> Result<(), CollaborationError> {
    if actor_is_tenant_admin || actor_user_id == session_owner_user_id {
        Ok(())
    } else {
        Err(CollaborationError::PermissionDenied)
    }
}

// =====================================================================
// INV-CB-08:Cursor 选区范围合法
// =====================================================================

/// **INV-CB-08**:Cursor 选区范围合法(Range / Block 时 selection_start / selection_end 必须有序)
///
/// selection_start ≤ selection_end,且 Range 时两端均非 None。
pub fn check_invariant_08_cursor_selection_valid(
    c: &PresenceCursor,
) -> Result<(), CollaborationError> {
    use crate::value_object::SelectionShape;
    match c.selection_shape {
        SelectionShape::Point => {
            if c.selection_start.is_some() || c.selection_end.is_some() {
                return Err(CollaborationError::InvalidState(
                    "INV-CB-08: Point 选区不应有 selection_start / selection_end".to_string(),
                ));
            }
        }
        SelectionShape::Range | SelectionShape::Block => {
            let s = c.selection_start.ok_or_else(|| {
                CollaborationError::InvalidState(
                    "INV-CB-08: Range/Block 选区缺少 selection_start".to_string(),
                )
            })?;
            let e = c.selection_end.ok_or_else(|| {
                CollaborationError::InvalidState(
                    "INV-CB-08: Range/Block 选区缺少 selection_end".to_string(),
                )
            })?;
            if s > e {
                return Err(CollaborationError::InvalidState(format!(
                    "INV-CB-08: 选区范围非法 start={s} > end={e}"
                )));
            }
        }
    }
    Ok(())
}

// =====================================================================
// 批量执行
// =====================================================================

/// **所有不变量检查(创建时执行)**
///
/// 实际由 service.rs 在不同命令路径上按需调用单条;
/// 本常量保留为空数组,与 `domain-workflow` 模式保持一致。
pub const ALL_INVARIANT_CHECKS: &[SessionCheck] = &[];

/// 批量执行 Session 不变量检查,首次失败即返回错误。
pub fn run_invariants(
    checks: &[SessionCheck],
    s: &CollaborationSession,
) -> Result<(), CollaborationError> {
    for check in checks {
        check(s)?;
    }
    Ok(())
}

/// 创建 Session 时的核心不变量集合(INV-CB-01 + 基本字段合法性)
pub fn check_create_invariants(
    s: &CollaborationSession,
) -> Result<(), CollaborationError> {
    // INV-CB-01:tenant_id 非 nil
    if s.tenant_id.as_uuid().is_nil() {
        return Err(CollaborationError::InvalidState(
            "INV-CB-01: tenant_id 不能为 nil UUID".to_string(),
        ));
    }
    // Session.name 非空
    if s.name.trim().is_empty() {
        return Err(CollaborationError::InvalidState(
            "CollaborationSession.name 不能为空".to_string(),
        ));
    }
    Ok(())
}
