//! A10.1-A10.2 settings_integration (per DD-CANVAS-AGENT-001 §3.1 + §4.14.1 + 守门 #13 a RLS + 守门 #19 v19 累积规).
//!
//! `AgentSettingsTab` 表示 agent settings 集成 (per BD-CANVAS-AGENT-001 §3 A10.1 detail panel Settings tab + A10.2 实时设置编辑).
//! 主要业务方法: 加载 settings + 实时更新, 0 外部 I/O.

use crate::models::agent::{Agent, AgentKind, AgentRole};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// `agent-domain` settings_integration 错误.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SettingsError {
    /// tenant_id 未设置 (守门 #13 a RLS 13 类)
    TenantIdRequired,
    /// token_budget = 0 不允许 (per A10.2 实时设置 防止设 0 关预算)
    ZeroBudget,
}

/// A10.1-A10.2 agent settings (per BD-CANVAS-AGENT-001 §3 A10).
///
/// 字段: agent_id + role + kind + token_budget + tenant_id (守门 #13 a) + updated_at.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentSettings {
    /// agent_id
    pub agent_id: Uuid,
    /// role
    pub role: AgentRole,
    /// kind
    pub kind: AgentKind,
    /// token_budget (per A7.2 默认 1.2M / SRE·周)
    pub token_budget: u64,
    /// tenant_id (守门 #13 a 100% RLS 13 类 必携)
    pub tenant_id: Uuid,
    /// 上次更新时间 (UTC)
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl AgentSettings {
    /// A10.1 业务方法: 加载 settings from Agent (per BD §3 A10.1 settings tab 集成).
    pub fn load_from_agent(agent: &Agent) -> Result<Self, SettingsError> {
        if agent.tenant_id.is_nil() {
            return Err(SettingsError::TenantIdRequired);
        }
        Ok(Self {
            agent_id: agent.id,
            role: agent.role,
            kind: agent.kind,
            token_budget: agent.token_budget,
            tenant_id: agent.tenant_id,
            updated_at: Utc::now(),
        })
    }

    /// A10.2 业务方法: 实时更新 settings (per BD §3 A10.2 实时编辑 role/kind/token_budget).
    ///
    /// 守门: token_budget != 0 (防止设 0 关预算).
    pub fn update(
        &mut self,
        role: AgentRole,
        kind: AgentKind,
        token_budget: u64,
    ) -> Result<(), SettingsError> {
        if token_budget == 0 {
            return Err(SettingsError::ZeroBudget);
        }
        self.role = role;
        self.kind = kind;
        self.token_budget = token_budget;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// A10.1 业务方法: 转换为 Agent (apply settings back to agent).
    pub fn apply_to_agent(&self, agent: &mut Agent) {
        agent.role = self.role;
        agent.kind = self.kind;
        agent.token_budget = self.token_budget;
        // 不动 status (per A10.2 实时编辑 0 改 status 字段)
    }
}

/// A10.1 settings tab 集成 (per BD-CANVAS-AGENT-001 §3 A10).
///
/// 字段: settings (per agent_id 索引) + tenant_id (守门 #13 a).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentSettingsTab {
    /// tenant_id (守门 #13 a 100% RLS 13 类 必携)
    pub tenant_id: Uuid,
    /// settings by agent_id
    pub settings: std::collections::HashMap<Uuid, AgentSettings>,
}

impl AgentSettingsTab {
    /// A10.1 业务方法: 创建 settings tab.
    pub fn new(tenant_id: Uuid) -> Result<Self, SettingsError> {
        if tenant_id.is_nil() {
            return Err(SettingsError::TenantIdRequired);
        }
        Ok(Self {
            tenant_id,
            settings: std::collections::HashMap::new(),
        })
    }

    /// A10.1 业务方法: 加载 settings 列表 from agents.
    pub fn load_agents(&mut self, agents: &[Agent]) -> Result<usize, SettingsError> {
        let mut count = 0;
        for agent in agents {
            let s = AgentSettings::load_from_agent(agent)?;
            self.settings.insert(agent.id, s);
            count += 1;
        }
        Ok(count)
    }

    /// A10.2 业务方法: 实时更新指定 agent settings.
    pub fn update_agent(
        &mut self,
        agent_id: Uuid,
        role: AgentRole,
        kind: AgentKind,
        token_budget: u64,
    ) -> Result<(), SettingsError> {
        let s = self
            .settings
            .get_mut(&agent_id)
            .ok_or(SettingsError::ZeroBudget)?;
        s.update(role, kind, token_budget)
    }

    /// A10.1 业务方法: 已加载 settings 数量.
    pub fn settings_count(&self) -> usize {
        self.settings.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::agent::{Agent, AgentState, Domain};

    fn make_agent() -> Agent {
        Agent {
            id: Uuid::new_v4(),
            name: "test".to_string(),
            avatar_url: None,
            role: AgentRole::Worker,
            kind: AgentKind::Custom,
            domain: Some(Domain::Player),
            status: AgentState::Running,
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
    fn test_settings_load_from_agent_ok() {
        let agent = make_agent();
        let s = AgentSettings::load_from_agent(&agent).expect("valid");
        assert_eq!(s.agent_id, agent.id);
        assert_eq!(s.role, AgentRole::Worker);
        assert_eq!(s.token_budget, 1_200_000);
    }

    #[test]
    fn test_settings_load_from_agent_tenant_required() {
        let mut agent = make_agent();
        agent.tenant_id = Uuid::nil();
        let result = AgentSettings::load_from_agent(&agent);
        assert_eq!(result.err(), Some(SettingsError::TenantIdRequired));
    }

    #[test]
    fn test_settings_update_ok() {
        let agent = make_agent();
        let mut s = AgentSettings::load_from_agent(&agent).unwrap();
        s.update(AgentRole::Supervisor, AgentKind::Sa01, 2_000_000)
            .expect("valid");
        assert_eq!(s.role, AgentRole::Supervisor);
        assert_eq!(s.kind, AgentKind::Sa01);
        assert_eq!(s.token_budget, 2_000_000);
    }

    #[test]
    fn test_settings_update_zero_budget() {
        let agent = make_agent();
        let mut s = AgentSettings::load_from_agent(&agent).unwrap();
        let result = s.update(AgentRole::Worker, AgentKind::Custom, 0);
        assert_eq!(result.err(), Some(SettingsError::ZeroBudget));
    }

    #[test]
    fn test_settings_tab_load_agents() {
        let tenant = Uuid::new_v4();
        let mut tab = AgentSettingsTab::new(tenant).unwrap();
        let agents = vec![make_agent(), make_agent(), make_agent()];
        let count = tab.load_agents(&agents).expect("valid");
        assert_eq!(count, 3);
        assert_eq!(tab.settings_count(), 3);
    }

    #[test]
    fn test_settings_tab_update_agent() {
        let tenant = Uuid::new_v4();
        let mut tab = AgentSettingsTab::new(tenant).unwrap();
        let agent = make_agent();
        let agent_id = agent.id;
        tab.load_agents(&[agent]).unwrap();
        tab.update_agent(agent_id, AgentRole::Supervisor, AgentKind::Sa04, 1_500_000)
            .expect("valid");
        let s = tab.settings.get(&agent_id).unwrap();
        assert_eq!(s.role, AgentRole::Supervisor);
        assert_eq!(s.kind, AgentKind::Sa04);
    }
}
