//! A2.1 1:N handoff connector (per DD-CANVAS-AGENT-001 §3.1 + §4.14.1 + 守门 #13 a RLS + 守门 #19 v19 累积规).
//!
//! `HandoffConnector` 表示 1 个 supervisor 跟 N 个 worker 之间的 1:N handoff 关系
//! (per ARG.A2.1 1:N handoff connector, 派生自 DD-AGENT-RELATIONSHIP-001 §3.2.2).
//! 主要业务方法: 创建/校验/查询 target agent_ids, 0 外部 I/O (纯 in-memory).

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// `agent-domain` handoff 错误 (per DD-AGENT §4.7 + 守门 #13 a 100% RLS 13 类 tenant_id 必填).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HandoffError {
    /// target_agent_ids 空 (1:N 必须 ≥ 1 target)
    EmptyTargets,
    /// target_agent_ids 包含 from_agent_id (self-handoff 禁止)
    SelfHandoff,
    /// tenant_id 未设置 (守门 #13 a RLS 13 类)
    TenantIdRequired,
}

/// A2.1 handoff connector (per DD-CANVAS-AGENT-001 §3.1 + §4.14.1 + BD-CANVAS-AGENT-001 §3 A2.1).
///
/// 字段: from_agent_id (supervisor) + target_agent_ids (N workers) + tenant_id (守门 #13 a) + created_at.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HandoffConnector {
    /// handoff connector ID (v4 UUID)
    pub id: Uuid,
    /// source agent_id (supervisor, A2.1 1:N 中的 1)
    pub from_agent_id: Uuid,
    /// target agent_ids (workers, A2.1 1:N 中的 N)
    pub target_agent_ids: Vec<Uuid>,
    /// tenant_id (守门 #13 a 100% RLS 13 类 必携)
    pub tenant_id: Uuid,
    /// 创建时间 (UTC)
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl HandoffConnector {
    /// A2.1 业务方法: 创建 handoff connector (1:N handoff).
    ///
    /// 守门 #13 a: tenant_id 必填 (非 nil Uuid).
    /// 守门: target_agent_ids 必须非空 (EmptyTargets) + 不能含 from_agent_id (SelfHandoff).
    pub fn new(
        id: Uuid,
        from_agent_id: Uuid,
        target_agent_ids: Vec<Uuid>,
        tenant_id: Uuid,
    ) -> Result<Self, HandoffError> {
        if tenant_id.is_nil() {
            return Err(HandoffError::TenantIdRequired);
        }
        if target_agent_ids.is_empty() {
            return Err(HandoffError::EmptyTargets);
        }
        if target_agent_ids.contains(&from_agent_id) {
            return Err(HandoffError::SelfHandoff);
        }
        Ok(Self {
            id,
            from_agent_id,
            target_agent_ids,
            tenant_id,
            created_at: chrono::Utc::now(),
        })
    }

    /// A2.1 业务方法: target agent_id 数量 (per BD §3 A2.1 1:N 中的 N).
    pub fn target_count(&self) -> usize {
        self.target_agent_ids.len()
    }

    /// A2.1 业务方法: target agent_id 是否包含指定 agent_id.
    pub fn targets(&self, agent_id: Uuid) -> bool {
        self.target_agent_ids.contains(&agent_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handoff_connector_new_ok() {
        let id = Uuid::new_v4();
        let from = Uuid::new_v4();
        let targets = vec![Uuid::new_v4(), Uuid::new_v4()];
        let tenant = Uuid::new_v4();
        let h = HandoffConnector::new(id, from, targets.clone(), tenant).expect("valid handoff");
        assert_eq!(h.id, id);
        assert_eq!(h.from_agent_id, from);
        assert_eq!(h.target_agent_ids, targets);
        assert_eq!(h.tenant_id, tenant);
    }

    #[test]
    fn test_handoff_connector_new_empty_targets() {
        let id = Uuid::new_v4();
        let from = Uuid::new_v4();
        let tenant = Uuid::new_v4();
        let result = HandoffConnector::new(id, from, vec![], tenant);
        assert_eq!(result.err(), Some(HandoffError::EmptyTargets));
    }

    #[test]
    fn test_handoff_connector_new_self_handoff() {
        let id = Uuid::new_v4();
        let agent = Uuid::new_v4();
        let tenant = Uuid::new_v4();
        let result = HandoffConnector::new(id, agent, vec![agent], tenant);
        assert_eq!(result.err(), Some(HandoffError::SelfHandoff));
    }

    #[test]
    fn test_handoff_connector_new_tenant_required() {
        let id = Uuid::new_v4();
        let from = Uuid::new_v4();
        let targets = vec![Uuid::new_v4()];
        let result = HandoffConnector::new(id, from, targets, Uuid::nil());
        assert_eq!(result.err(), Some(HandoffError::TenantIdRequired));
    }

    #[test]
    fn test_handoff_connector_target_count() {
        let from = Uuid::new_v4();
        let targets = vec![Uuid::new_v4(); 3];
        let tenant = Uuid::new_v4();
        let h = HandoffConnector::new(Uuid::new_v4(), from, targets, tenant).unwrap();
        assert_eq!(h.target_count(), 3);
    }

    #[test]
    fn test_handoff_connector_targets_contains() {
        let from = Uuid::new_v4();
        let t1 = Uuid::new_v4();
        let t2 = Uuid::new_v4();
        let tenant = Uuid::new_v4();
        let h = HandoffConnector::new(Uuid::new_v4(), from, vec![t1, t2], tenant).unwrap();
        assert!(h.targets(t1));
        assert!(h.targets(t2));
        assert!(!h.targets(Uuid::new_v4()));
    }
}
