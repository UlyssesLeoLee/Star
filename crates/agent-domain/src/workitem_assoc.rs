//! A5.1-A5.2 workitem_assoc (per DD-CANVAS-AGENT-001 §3.1 + §4.14.1 + 守门 #13 a RLS + 守门 #19 v19 累积规).
//!
//! `WorkItemDragIn` 表示 1 agent : N work-item 关联 (per BD-CANVAS-AGENT-001 §3 A5.1 drag work-item → agent_node 范围 + A5.2 work-item 状态映射).
//! 主要业务方法: 拖入/移除/查询, 0 外部 I/O.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// `agent-domain` workitem_assoc 错误.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkItemAssocError {
    /// work_item_id 已存在 (重复拖入 禁止)
    DuplicateWorkItem,
    /// work_item_id 不存在 (remove 时 找不到)
    NotFound,
    /// tenant_id 未设置 (守门 #13 a RLS 13 类)
    TenantIdRequired,
}

/// A5.1-A5.2 1 agent : N work-item 关联 (per BD-CANVAS-AGENT-001 §3 A5.1 drag work-item 范围).
///
/// 字段: agent_id (中心) + work_item_ids (N 个 work-item 拖入) + tenant_id (守门 #13 a) + last_sync_at (A5.2 ≤ 200ms P95).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkItemDragIn {
    /// 中心 agent_id (A5.1 1 个)
    pub agent_id: Uuid,
    /// 周围 work_item_ids (A5.1 N 个, drag 范围)
    pub work_item_ids: Vec<Uuid>,
    /// 上次同步时间 (UTC, A5.2 ≤ 200ms P95)
    pub last_sync_at: chrono::DateTime<chrono::Utc>,
    /// tenant_id (守门 #13 a 100% RLS 13 类 必携)
    pub tenant_id: Uuid,
}

impl WorkItemDragIn {
    /// A5.1 业务方法: 创建 workitem drag-in (空 N, 后续 drop).
    pub fn new(agent_id: Uuid, tenant_id: Uuid) -> Result<Self, WorkItemAssocError> {
        if tenant_id.is_nil() {
            return Err(WorkItemAssocError::TenantIdRequired);
        }
        Ok(Self {
            agent_id,
            work_item_ids: Vec::new(),
            last_sync_at: chrono::Utc::now(),
            tenant_id,
        })
    }

    /// A5.1 业务方法: 拖入 work_item_id (drop).
    pub fn drop_workitem(&mut self, work_item_id: Uuid) -> Result<(), WorkItemAssocError> {
        if self.work_item_ids.contains(&work_item_id) {
            return Err(WorkItemAssocError::DuplicateWorkItem);
        }
        self.work_item_ids.push(work_item_id);
        self.last_sync_at = chrono::Utc::now();
        Ok(())
    }

    /// A5.2 业务方法: 移除 work_item_id.
    pub fn remove_workitem(&mut self, work_item_id: Uuid) -> Result<(), WorkItemAssocError> {
        let pos = self.work_item_ids.iter().position(|w| *w == work_item_id);
        match pos {
            Some(i) => {
                self.work_item_ids.remove(i);
                self.last_sync_at = chrono::Utc::now();
                Ok(())
            }
            None => Err(WorkItemAssocError::NotFound),
        }
    }

    /// A5.1 业务方法: work-item 数量.
    pub fn workitem_count(&self) -> usize {
        self.work_item_ids.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workitem_dragin_new_ok() {
        let agent = Uuid::new_v4();
        let tenant = Uuid::new_v4();
        let d = WorkItemDragIn::new(agent, tenant).expect("valid");
        assert_eq!(d.agent_id, agent);
        assert_eq!(d.tenant_id, tenant);
        assert_eq!(d.workitem_count(), 0);
    }

    #[test]
    fn test_workitem_dragin_new_tenant_required() {
        let agent = Uuid::new_v4();
        let result = WorkItemDragIn::new(agent, Uuid::nil());
        assert_eq!(result.err(), Some(WorkItemAssocError::TenantIdRequired));
    }

    #[test]
    fn test_workitem_dragin_drop_ok() {
        let tenant = Uuid::new_v4();
        let mut d = WorkItemDragIn::new(Uuid::new_v4(), tenant).unwrap();
        let w1 = Uuid::new_v4();
        let w2 = Uuid::new_v4();
        d.drop_workitem(w1).unwrap();
        d.drop_workitem(w2).unwrap();
        assert_eq!(d.workitem_count(), 2);
    }

    #[test]
    fn test_workitem_dragin_drop_duplicate() {
        let tenant = Uuid::new_v4();
        let mut d = WorkItemDragIn::new(Uuid::new_v4(), tenant).unwrap();
        let w = Uuid::new_v4();
        d.drop_workitem(w).unwrap();
        let result = d.drop_workitem(w);
        assert_eq!(result.err(), Some(WorkItemAssocError::DuplicateWorkItem));
    }

    #[test]
    fn test_workitem_dragin_remove_ok() {
        let tenant = Uuid::new_v4();
        let mut d = WorkItemDragIn::new(Uuid::new_v4(), tenant).unwrap();
        let w = Uuid::new_v4();
        d.drop_workitem(w).unwrap();
        d.remove_workitem(w).unwrap();
        assert_eq!(d.workitem_count(), 0);
    }

    #[test]
    fn test_workitem_dragin_remove_not_found() {
        let tenant = Uuid::new_v4();
        let mut d = WorkItemDragIn::new(Uuid::new_v4(), tenant).unwrap();
        let result = d.remove_workitem(Uuid::new_v4());
        assert_eq!(result.err(), Some(WorkItemAssocError::NotFound));
    }
}
