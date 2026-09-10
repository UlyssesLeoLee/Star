//! A4.1-A4.2 worktree_assoc (per DD-CANVAS-AGENT-001 §3.1 + §4.14.1 + 守门 #13 a RLS + 守门 #19 v19 累积规).
//!
//! `WorktreeRing` 表示 1 agent : N worktree 关联 (per BD-CANVAS-AGENT-001 §3 A4.1 + A4.2 实时色卡同步).
//! 主要业务方法: 添加/移除 worktree/查询, 0 外部 I/O.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// `agent-domain` worktree_assoc 错误.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorktreeAssocError {
    /// worktree_id 已存在 (重复添加 禁止)
    DuplicateWorktree,
    /// worktree_id 不存在 (remove 时 找不到)
    NotFound,
    /// tenant_id 未设置 (守门 #13 a RLS 13 类)
    TenantIdRequired,
}

/// A4.1-A4.2 1 agent : N worktree 关联 ring (per BD-CANVAS-AGENT-001 §3 A4.1 圆环散开 N 个 worktree_node).
///
/// 字段: agent_id (中心) + worktree_ids (N 个 worktree 围绕) + tenant_id (守门 #13 a) + last_sync_at (A4.2 ≤ 200ms P95).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorktreeRing {
    /// 中心 agent_id (A4.1 1 个)
    pub agent_id: Uuid,
    /// 周围 worktree_ids (A4.1 N 个)
    pub worktree_ids: Vec<Uuid>,
    /// 上次同步时间 (UTC, A4.2 ≤ 200ms P95)
    pub last_sync_at: chrono::DateTime<chrono::Utc>,
    /// tenant_id (守门 #13 a 100% RLS 13 类 必携)
    pub tenant_id: Uuid,
}

impl WorktreeRing {
    /// A4.1 业务方法: 创建 worktree ring (空 N, 后续 add).
    pub fn new(agent_id: Uuid, tenant_id: Uuid) -> Result<Self, WorktreeAssocError> {
        if tenant_id.is_nil() {
            return Err(WorktreeAssocError::TenantIdRequired);
        }
        Ok(Self {
            agent_id,
            worktree_ids: Vec::new(),
            last_sync_at: chrono::Utc::now(),
            tenant_id,
        })
    }

    /// A4.1 业务方法: 添加 worktree_id.
    pub fn add_worktree(&mut self, worktree_id: Uuid) -> Result<(), WorktreeAssocError> {
        if self.worktree_ids.contains(&worktree_id) {
            return Err(WorktreeAssocError::DuplicateWorktree);
        }
        self.worktree_ids.push(worktree_id);
        self.last_sync_at = chrono::Utc::now();
        Ok(())
    }

    /// A4.2 业务方法: 移除 worktree_id.
    pub fn remove_worktree(&mut self, worktree_id: Uuid) -> Result<(), WorktreeAssocError> {
        let pos = self.worktree_ids.iter().position(|w| *w == worktree_id);
        match pos {
            Some(i) => {
                self.worktree_ids.remove(i);
                self.last_sync_at = chrono::Utc::now();
                Ok(())
            }
            None => Err(WorktreeAssocError::NotFound),
        }
    }

    /// A4.1 业务方法: worktree 数量.
    pub fn worktree_count(&self) -> usize {
        self.worktree_ids.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worktree_ring_new_ok() {
        let agent = Uuid::new_v4();
        let tenant = Uuid::new_v4();
        let r = WorktreeRing::new(agent, tenant).expect("valid");
        assert_eq!(r.agent_id, agent);
        assert_eq!(r.tenant_id, tenant);
        assert_eq!(r.worktree_count(), 0);
    }

    #[test]
    fn test_worktree_ring_new_tenant_required() {
        let agent = Uuid::new_v4();
        let result = WorktreeRing::new(agent, Uuid::nil());
        assert_eq!(result.err(), Some(WorktreeAssocError::TenantIdRequired));
    }

    #[test]
    fn test_worktree_ring_add_ok() {
        let tenant = Uuid::new_v4();
        let mut r = WorktreeRing::new(Uuid::new_v4(), tenant).unwrap();
        let w1 = Uuid::new_v4();
        let w2 = Uuid::new_v4();
        r.add_worktree(w1).unwrap();
        r.add_worktree(w2).unwrap();
        assert_eq!(r.worktree_count(), 2);
    }

    #[test]
    fn test_worktree_ring_add_duplicate() {
        let tenant = Uuid::new_v4();
        let mut r = WorktreeRing::new(Uuid::new_v4(), tenant).unwrap();
        let w = Uuid::new_v4();
        r.add_worktree(w).unwrap();
        let result = r.add_worktree(w);
        assert_eq!(result.err(), Some(WorktreeAssocError::DuplicateWorktree));
    }

    #[test]
    fn test_worktree_ring_remove_ok() {
        let tenant = Uuid::new_v4();
        let mut r = WorktreeRing::new(Uuid::new_v4(), tenant).unwrap();
        let w = Uuid::new_v4();
        r.add_worktree(w).unwrap();
        r.remove_worktree(w).unwrap();
        assert_eq!(r.worktree_count(), 0);
    }

    #[test]
    fn test_worktree_ring_remove_not_found() {
        let tenant = Uuid::new_v4();
        let mut r = WorktreeRing::new(Uuid::new_v4(), tenant).unwrap();
        let result = r.remove_worktree(Uuid::new_v4());
        assert_eq!(result.err(), Some(WorktreeAssocError::NotFound));
    }
}
