//! A8.1-A8.3 cluster sort/filter (per DD-CANVAS-AGENT-001 §3.1 + §4.14.1 + 守门 #13 a RLS + 守门 #19 v19 累积规).
//!
//! `ClusterSortFilter` 表示 agent 聚类/排序/过滤 (per BD-CANVAS-AGENT-001 §3 A8.1 role 排序 + A8.2 kind 排序 + A8.3 status/token/started_at 过滤).
//! 主要业务方法: sort_by_role + sort_by_kind + filter_by_status, 0 外部 I/O.

use crate::models::agent::{Agent, AgentKind, AgentRole, AgentState};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// `agent-domain` cluster 错误.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ClusterError {
    /// tenant_id 未设置 (守门 #13 a RLS 13 类)
    TenantIdRequired,
}

/// A8.1-A8.3 agent cluster sort/filter (per BD-CANVAS-AGENT-001 §3 A8).
///
/// 字段: tenant_id (守门 #13 a) + agents (要 sort/filter 的 agent 列表).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClusterSortFilter {
    /// tenant_id (守门 #13 a 100% RLS 13 类 必携)
    pub tenant_id: Uuid,
    /// agents 列表
    pub agents: Vec<Agent>,
}

impl ClusterSortFilter {
    /// A8.1 业务方法: 创建 cluster (空 agents 列表, 后续 add).
    pub fn new(tenant_id: Uuid) -> Result<Self, ClusterError> {
        if tenant_id.is_nil() {
            return Err(ClusterError::TenantIdRequired);
        }
        Ok(Self {
            tenant_id,
            agents: Vec::new(),
        })
    }

    /// A8.1 业务方法: 排序 by role (Supervisor > Reviewer > Worker, per A8.1 dropdown 3 类).
    pub fn sort_by_role(&self) -> Vec<Agent> {
        let mut sorted = self.agents.clone();
        sorted.sort_by_key(|a| match a.role {
            AgentRole::Supervisor => 0,
            AgentRole::Reviewer => 1,
            AgentRole::Worker => 2,
        });
        sorted
    }

    /// A8.2 业务方法: 排序 by kind (Sa01..Sa09 > Custom, per A8.2 [kind ASC, started_at ASC, id ASC] 稳定排序).
    pub fn sort_by_kind(&self) -> Vec<Agent> {
        let mut sorted = self.agents.clone();
        sorted.sort_by(|a, b| {
            let a_key = kind_sort_key(a.kind);
            let b_key = kind_sort_key(b.kind);
            a_key.cmp(&b_key).then_with(|| {
                a.started_at
                    .cmp(&b.started_at)
                    .then_with(|| a.id.cmp(&b.id))
            })
        });
        sorted
    }

    /// A8.3 业务方法: 过滤 by status (per A8.3 dropdown 多选).
    pub fn filter_by_status(&self, status: AgentState) -> Vec<Agent> {
        self.agents
            .iter()
            .filter(|a| a.status == status)
            .cloned()
            .collect()
    }

    /// A8.1 业务方法: agent 总数.
    pub fn agent_count(&self) -> usize {
        self.agents.len()
    }
}

/// A8.2 kind 排序 key: Sa01=0, Sa02=1, ..., Sa09=8, Custom=9.
fn kind_sort_key(kind: AgentKind) -> u8 {
    match kind {
        AgentKind::Sa01 => 0,
        AgentKind::Sa02 => 1,
        AgentKind::Sa03 => 2,
        AgentKind::Sa04 => 3,
        AgentKind::Sa05 => 4,
        AgentKind::Sa06 => 5,
        AgentKind::Sa07 => 6,
        AgentKind::Sa08 => 7,
        AgentKind::Sa09 => 8,
        AgentKind::Custom => 9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::agent::{Agent, AgentKind, AgentRole, AgentState, Domain};
    use chrono::Utc;

    fn make_agent(role: AgentRole, kind: AgentKind, status: AgentState, name: &str) -> Agent {
        Agent {
            id: Uuid::new_v4(),
            name: name.to_string(),
            avatar_url: None,
            role,
            kind,
            domain: Some(Domain::Player),
            status,
            token_usage: 0,
            token_budget: 1_200_000,
            parent_session_id: None,
            pipeline_agent_ids: Vec::new(),
            started_at: Utc::now(),
            tenant_id: Uuid::new_v4(),
            version: 1,
        }
    }

    #[test]
    fn test_cluster_new_ok() {
        let tenant = Uuid::new_v4();
        let c = ClusterSortFilter::new(tenant).expect("valid");
        assert_eq!(c.tenant_id, tenant);
        assert_eq!(c.agent_count(), 0);
    }

    #[test]
    fn test_cluster_new_tenant_required() {
        let result = ClusterSortFilter::new(Uuid::nil());
        assert_eq!(result.err(), Some(ClusterError::TenantIdRequired));
    }

    #[test]
    fn test_cluster_sort_by_role_supervisor_first() {
        let tenant = Uuid::new_v4();
        let mut c = ClusterSortFilter::new(tenant).unwrap();
        c.agents.push(make_agent(
            AgentRole::Worker,
            AgentKind::Custom,
            AgentState::Running,
            "w1",
        ));
        c.agents.push(make_agent(
            AgentRole::Supervisor,
            AgentKind::Sa01,
            AgentState::Running,
            "s1",
        ));
        c.agents.push(make_agent(
            AgentRole::Reviewer,
            AgentKind::Sa02,
            AgentState::Running,
            "r1",
        ));
        let sorted = c.sort_by_role();
        assert_eq!(sorted[0].role, AgentRole::Supervisor);
        assert_eq!(sorted[1].role, AgentRole::Reviewer);
        assert_eq!(sorted[2].role, AgentRole::Worker);
    }

    #[test]
    fn test_cluster_sort_by_kind_sa01_first() {
        let tenant = Uuid::new_v4();
        let mut c = ClusterSortFilter::new(tenant).unwrap();
        c.agents.push(make_agent(
            AgentRole::Worker,
            AgentKind::Custom,
            AgentState::Running,
            "a",
        ));
        c.agents.push(make_agent(
            AgentRole::Worker,
            AgentKind::Sa05,
            AgentState::Running,
            "b",
        ));
        c.agents.push(make_agent(
            AgentRole::Worker,
            AgentKind::Sa01,
            AgentState::Running,
            "c",
        ));
        let sorted = c.sort_by_kind();
        assert_eq!(sorted[0].kind, AgentKind::Sa01);
        assert_eq!(sorted[1].kind, AgentKind::Sa05);
        assert_eq!(sorted[2].kind, AgentKind::Custom);
    }

    #[test]
    fn test_cluster_filter_by_status_running() {
        let tenant = Uuid::new_v4();
        let mut c = ClusterSortFilter::new(tenant).unwrap();
        c.agents.push(make_agent(
            AgentRole::Worker,
            AgentKind::Custom,
            AgentState::Running,
            "a",
        ));
        c.agents.push(make_agent(
            AgentRole::Worker,
            AgentKind::Custom,
            AgentState::Failed,
            "b",
        ));
        c.agents.push(make_agent(
            AgentRole::Worker,
            AgentKind::Custom,
            AgentState::Running,
            "c",
        ));
        let running = c.filter_by_status(AgentState::Running);
        assert_eq!(running.len(), 2);
    }

    #[test]
    fn test_cluster_agent_count() {
        let tenant = Uuid::new_v4();
        let mut c = ClusterSortFilter::new(tenant).unwrap();
        c.agents.push(make_agent(
            AgentRole::Worker,
            AgentKind::Custom,
            AgentState::Running,
            "a",
        ));
        c.agents.push(make_agent(
            AgentRole::Worker,
            AgentKind::Custom,
            AgentState::Running,
            "b",
        ));
        assert_eq!(c.agent_count(), 2);
    }
}
