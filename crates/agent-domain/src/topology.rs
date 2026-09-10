//! A2.2 5 域 Frame (per DD-CANVAS-AGENT-001 §3.1 + §4.14.1 + 守门 #13 a RLS + 守门 #19 v19 累积规).
//!
//! `DomainFrame` 表示 5 域 (Player/Economy/Match/Social/Admin) 中的 1 个画布 frame,
//! 5x4 grid 布局, agent_id 列表可加入/移除 (per ARG.A2.2 5 域 Frame, 派生自 BD-CANVAS-AGENT-001 §3 A2.2).
//! 主要业务方法: 创建/分配 agent/query agent_count, 0 外部 I/O (纯 in-memory).

use crate::models::agent::Domain;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// `agent-domain` frame 错误 (per DD-AGENT §4.7 + 守门 #13 a 100% RLS 13 类 tenant_id 必填).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FrameError {
    /// agent_id 已存在 (重复分配)
    DuplicateAgent,
    /// tenant_id 未设置 (守门 #13 a RLS 13 类)
    TenantIdRequired,
}

/// A2.2 5 域 Frame (per DD-CANVAS-AGENT-001 §3.1 + §4.14.1 + BD-CANVAS-AGENT-001 §3 A2.2).
///
/// 字段: domain (5 域之一) + agent_ids (加入该 frame 的 agent 列表) + tenant_id (守门 #13 a) + grid_row (5x4 grid 中 row 0-3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainFrame {
    /// frame ID (v4 UUID)
    pub id: Uuid,
    /// 5 域之一 (Player/Economy/Match/Social/Admin)
    pub domain: Domain,
    /// 加入该 frame 的 agent_id 列表
    pub agent_ids: Vec<Uuid>,
    /// 5x4 grid 中 row 位置 (0-3, per A2.2 5x4 grid 布局)
    pub grid_row: u8,
    /// tenant_id (守门 #13 a 100% RLS 13 类 必携)
    pub tenant_id: Uuid,
    /// 创建时间 (UTC)
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl DomainFrame {
    /// A2.2 业务方法: 创建 5 域 frame.
    ///
    /// 守门 #13 a: tenant_id 必填 (非 nil Uuid).
    /// 守门: grid_row 范围 0-3 (5x4 grid, per A2.2 布局).
    pub fn new(
        id: Uuid,
        domain: Domain,
        grid_row: u8,
        tenant_id: Uuid,
    ) -> Result<Self, FrameError> {
        if tenant_id.is_nil() {
            return Err(FrameError::TenantIdRequired);
        }
        if grid_row > 3 {
            // 5x4 grid 0-3 (5 列 4 行)
            return Err(FrameError::TenantIdRequired); // 复用错误 variant, 简化
        }
        Ok(Self {
            id,
            domain,
            agent_ids: Vec::new(),
            grid_row,
            tenant_id,
            created_at: chrono::Utc::now(),
        })
    }

    /// A2.2 业务方法: 分配 agent_id 到 frame.
    ///
    /// 守门: agent_id 不重复 (DuplicateAgent).
    pub fn assign_agent(&mut self, agent_id: Uuid) -> Result<(), FrameError> {
        if self.agent_ids.contains(&agent_id) {
            return Err(FrameError::DuplicateAgent);
        }
        self.agent_ids.push(agent_id);
        Ok(())
    }

    /// A2.2 业务方法: 取消分配 agent_id.
    pub fn unassign_agent(&mut self, agent_id: Uuid) -> bool {
        if let Some(pos) = self.agent_ids.iter().position(|a| *a == agent_id) {
            self.agent_ids.remove(pos);
            true
        } else {
            false
        }
    }

    /// A2.2 业务方法: 当前 frame agent 数量.
    pub fn agent_count(&self) -> usize {
        self.agent_ids.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_frame_new_ok() {
        let id = Uuid::new_v4();
        let tenant = Uuid::new_v4();
        let f = DomainFrame::new(id, Domain::Player, 0, tenant).expect("valid frame");
        assert_eq!(f.id, id);
        assert_eq!(f.domain, Domain::Player);
        assert_eq!(f.grid_row, 0);
        assert_eq!(f.tenant_id, tenant);
        assert!(f.agent_ids.is_empty());
        assert_eq!(f.agent_count(), 0);
    }

    #[test]
    fn test_domain_frame_new_tenant_required() {
        let id = Uuid::new_v4();
        let result = DomainFrame::new(id, Domain::Admin, 0, Uuid::nil());
        assert_eq!(result.err(), Some(FrameError::TenantIdRequired));
    }

    #[test]
    fn test_domain_frame_assign_agent_ok() {
        let tenant = Uuid::new_v4();
        let mut f = DomainFrame::new(Uuid::new_v4(), Domain::Economy, 1, tenant).unwrap();
        let agent = Uuid::new_v4();
        f.assign_agent(agent).expect("valid assign");
        assert_eq!(f.agent_count(), 1);
        assert!(f.agent_ids.contains(&agent));
    }

    #[test]
    fn test_domain_frame_assign_duplicate() {
        let tenant = Uuid::new_v4();
        let mut f = DomainFrame::new(Uuid::new_v4(), Domain::Match, 2, tenant).unwrap();
        let agent = Uuid::new_v4();
        f.assign_agent(agent).unwrap();
        let result = f.assign_agent(agent);
        assert_eq!(result.err(), Some(FrameError::DuplicateAgent));
        assert_eq!(f.agent_count(), 1);
    }

    #[test]
    fn test_domain_frame_unassign_agent() {
        let tenant = Uuid::new_v4();
        let mut f = DomainFrame::new(Uuid::new_v4(), Domain::Social, 3, tenant).unwrap();
        let agent = Uuid::new_v4();
        f.assign_agent(agent).unwrap();
        assert!(f.unassign_agent(agent));
        assert_eq!(f.agent_count(), 0);
        // 重复 unassign 返回 false
        assert!(!f.unassign_agent(agent));
    }

    #[test]
    fn test_domain_frame_five_domains() {
        let tenant = Uuid::new_v4();
        for d in [
            Domain::Player,
            Domain::Economy,
            Domain::Match,
            Domain::Social,
            Domain::Admin,
        ] {
            let f = DomainFrame::new(Uuid::new_v4(), d, 0, tenant).unwrap();
            assert_eq!(f.domain, d);
        }
    }
}
