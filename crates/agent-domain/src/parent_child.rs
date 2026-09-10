//! A2.3 parent_child connector (per DD-CANVAS-AGENT-001 §3.1 + §4.14.1 + 守门 #13 a RLS + 守门 #19 v19 累积规).
//!
//! `ParentChildConnector` 表示 supervisor → worker 父子关系 (1:1, per BD-CANVAS-AGENT-001 §3 A2.3).
//! 主要业务方法: 创建/查询/删除, 0 外部 I/O (纯 in-memory).

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// `agent-domain` parent_child 错误 (per DD-AGENT §4.7 + 守门 #13 a RLS 13 类).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ParentChildError {
    /// parent_id == child_id (self-loop 禁止)
    SelfLoop,
    /// tenant_id 未设置 (守门 #13 a RLS 13 类)
    TenantIdRequired,
}

/// A2.3 父子关系 connector (per BD-CANVAS-AGENT-001 §3 A2.3 1:1 supervisor → worker).
///
/// 字段: parent_id (supervisor) + child_id (worker) + tenant_id (守门 #13 a) + created_at.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParentChildConnector {
    /// connector ID
    pub id: Uuid,
    /// parent agent_id (supervisor)
    pub parent_id: Uuid,
    /// child agent_id (worker)
    pub child_id: Uuid,
    /// tenant_id (守门 #13 a 100% RLS 13 类 必携)
    pub tenant_id: Uuid,
    /// 创建时间 (UTC)
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl ParentChildConnector {
    /// A2.3 业务方法: 创建 父子 关系.
    ///
    /// 守门 #13 a: tenant_id 必填 (非 nil Uuid).
    /// 守门: parent_id != child_id (SelfLoop 禁止).
    pub fn new(
        id: Uuid,
        parent_id: Uuid,
        child_id: Uuid,
        tenant_id: Uuid,
    ) -> Result<Self, ParentChildError> {
        if tenant_id.is_nil() {
            return Err(ParentChildError::TenantIdRequired);
        }
        if parent_id == child_id {
            return Err(ParentChildError::SelfLoop);
        }
        Ok(Self {
            id,
            parent_id,
            child_id,
            tenant_id,
            created_at: chrono::Utc::now(),
        })
    }

    /// A2.3 业务方法: 验证是否有效 (parent != child + tenant != nil).
    pub fn is_valid(&self) -> bool {
        self.parent_id != self.child_id && !self.tenant_id.is_nil()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parent_child_new_ok() {
        let id = Uuid::new_v4();
        let parent = Uuid::new_v4();
        let child = Uuid::new_v4();
        let tenant = Uuid::new_v4();
        let c = ParentChildConnector::new(id, parent, child, tenant).expect("valid");
        assert_eq!(c.id, id);
        assert_eq!(c.parent_id, parent);
        assert_eq!(c.child_id, child);
        assert!(c.is_valid());
    }

    #[test]
    fn test_parent_child_new_self_loop() {
        let id = Uuid::new_v4();
        let agent = Uuid::new_v4();
        let tenant = Uuid::new_v4();
        let result = ParentChildConnector::new(id, agent, agent, tenant);
        assert_eq!(result.err(), Some(ParentChildError::SelfLoop));
    }

    #[test]
    fn test_parent_child_new_tenant_required() {
        let id = Uuid::new_v4();
        let parent = Uuid::new_v4();
        let child = Uuid::new_v4();
        let result = ParentChildConnector::new(id, parent, child, Uuid::nil());
        assert_eq!(result.err(), Some(ParentChildError::TenantIdRequired));
    }

    #[test]
    fn test_parent_child_is_valid_self_loop() {
        let id = Uuid::new_v4();
        let agent = Uuid::new_v4();
        let tenant = Uuid::new_v4();
        // 即使 new() 拒绝 self-loop, 直接构造后 is_valid 应返回 false
        let c = ParentChildConnector {
            id,
            parent_id: agent,
            child_id: agent,
            tenant_id: tenant,
            created_at: chrono::Utc::now(),
        };
        assert!(!c.is_valid());
    }
}
