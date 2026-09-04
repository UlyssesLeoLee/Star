//! crates/star-saga/src/saga_5b_services.rs
//!
//! E.1 5 域 service 实现 (per P4-E.1, 9/4 拍板, Mavis 临时代签 5 域 Lead)
//! per 守门 #14 5 域 Lead CONTENT 4 维 + 9/3 11:35 JST B 拍板
//!
//! 每个 service 是 stateful in-memory mock, 含基本业务逻辑 + 失败注入 (per Q-003 跨域补偿)
//! 5 域 Lead 真人到位后, 业务逻辑由真人覆盖, Mavis 临时代签撤回 (per 守门 #1 禁回溯叙事)

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum DomainError {
    #[error("player 域: {0}")]
    Player(String),
    #[error("economy 域: {0}")]
    Economy(String),
    #[error("match 域: {0}")]
    Match(String),
    #[error("social 域: {0}")]
    Social(String),
    #[error("admin 域: {0}")]
    Admin(String),
}

// === PlayerService ===
// 业务: 用户注册/暂停/恢复
// 失败注入: failure_injection flag

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Player {
    pub user_id: String,
    pub tenant_id: String,
    pub status: PlayerStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerStatus {
    Active,
    Suspended,
    Deleted,
}

#[derive(Default)]
pub struct PlayerService {
    players: Arc<Mutex<HashMap<String, Player>>>,
    failure_injection: Arc<Mutex<HashMap<String, bool>>>,
}

impl PlayerService {
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置 action 失败注入 (测试用)
    pub fn set_failure(&self, action: &str, fail: bool) {
        *self
            .failure_injection
            .lock()
            .unwrap()
            .entry(action.into())
            .or_insert(false) = fail;
    }

    pub fn register(&self, tenant_id: &str, user_id: &str) -> Result<Player, DomainError> {
        if *self
            .failure_injection
            .lock()
            .unwrap()
            .get("register")
            .unwrap_or(&false)
        {
            return Err(DomainError::Player("register forced fail".into()));
        }
        let p = Player {
            user_id: user_id.into(),
            tenant_id: tenant_id.into(),
            status: PlayerStatus::Active,
        };
        self.players
            .lock()
            .unwrap()
            .insert(user_id.into(), p.clone());
        Ok(p)
    }

    pub fn suspend(&self, user_id: &str) -> Result<(), DomainError> {
        if *self
            .failure_injection
            .lock()
            .unwrap()
            .get("suspend")
            .unwrap_or(&false)
        {
            return Err(DomainError::Player("suspend forced fail".into()));
        }
        let mut players = self.players.lock().unwrap();
        let p = players
            .get_mut(user_id)
            .ok_or_else(|| DomainError::Player("not found".into()))?;
        p.status = PlayerStatus::Suspended;
        Ok(())
    }

    pub fn restore(&self, user_id: &str) -> Result<(), DomainError> {
        let mut players = self.players.lock().unwrap();
        let p = players
            .get_mut(user_id)
            .ok_or_else(|| DomainError::Player("not found".into()))?;
        p.status = PlayerStatus::Active;
        Ok(())
    }

    pub fn get(&self, user_id: &str) -> Option<Player> {
        self.players.lock().unwrap().get(user_id).cloned()
    }

    /// 补偿: 注销 user
    pub fn deregister(&self, user_id: &str) -> Result<(), DomainError> {
        let mut players = self.players.lock().unwrap();
        players.remove(user_id);
        Ok(())
    }
}

// === EconomyService ===
// 业务: 账户扣款/退款/余额查询

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BillingAccount {
    pub billing_account_id: String,
    pub tenant_id: String,
    pub balance: i64, // cents
}

#[derive(Default)]
pub struct EconomyService {
    accounts: Arc<Mutex<HashMap<String, BillingAccount>>>,
    failure_injection: Arc<Mutex<HashMap<String, bool>>>,
}

impl EconomyService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_failure(&self, action: &str, fail: bool) {
        *self
            .failure_injection
            .lock()
            .unwrap()
            .entry(action.into())
            .or_insert(false) = fail;
    }

    pub fn create_account(
        &self,
        tenant_id: &str,
        billing_account_id: &str,
    ) -> Result<BillingAccount, DomainError> {
        if *self
            .failure_injection
            .lock()
            .unwrap()
            .get("create_account")
            .unwrap_or(&false)
        {
            return Err(DomainError::Economy("create_account forced fail".into()));
        }
        let a = BillingAccount {
            billing_account_id: billing_account_id.into(),
            tenant_id: tenant_id.into(),
            balance: 0,
        };
        self.accounts
            .lock()
            .unwrap()
            .insert(billing_account_id.into(), a.clone());
        Ok(a)
    }

    pub fn deduct(&self, billing_account_id: &str, amount_cents: i64) -> Result<(), DomainError> {
        if *self
            .failure_injection
            .lock()
            .unwrap()
            .get("deduct")
            .unwrap_or(&false)
        {
            return Err(DomainError::Economy("deduct forced fail".into()));
        }
        let mut accounts = self.accounts.lock().unwrap();
        let a = accounts
            .get_mut(billing_account_id)
            .ok_or_else(|| DomainError::Economy("not found".into()))?;
        if a.balance < amount_cents {
            return Err(DomainError::Economy("insufficient balance".into()));
        }
        a.balance -= amount_cents;
        Ok(())
    }

    pub fn refund(&self, billing_account_id: &str, amount_cents: i64) -> Result<(), DomainError> {
        let mut accounts = self.accounts.lock().unwrap();
        let a = accounts
            .get_mut(billing_account_id)
            .ok_or_else(|| DomainError::Economy("not found".into()))?;
        a.balance += amount_cents;
        Ok(())
    }

    pub fn get_balance(&self, billing_account_id: &str) -> i64 {
        self.accounts
            .lock()
            .unwrap()
            .get(billing_account_id)
            .map(|a| a.balance)
            .unwrap_or(0)
    }
}

// === MatchService ===
// 业务: 工作流启动/中止

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowInstance {
    pub workflow_instance_id: String,
    pub tenant_id: String,
    pub status: WorkflowStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkflowStatus {
    Running,
    Completed,
    Aborted,
}

#[derive(Default)]
pub struct MatchService {
    workflows: Arc<Mutex<HashMap<String, WorkflowInstance>>>,
    failure_injection: Arc<Mutex<HashMap<String, bool>>>,
}

impl MatchService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_failure(&self, action: &str, fail: bool) {
        *self
            .failure_injection
            .lock()
            .unwrap()
            .entry(action.into())
            .or_insert(false) = fail;
    }

    pub fn start_workflow(
        &self,
        tenant_id: &str,
        workflow_instance_id: &str,
    ) -> Result<WorkflowInstance, DomainError> {
        if *self
            .failure_injection
            .lock()
            .unwrap()
            .get("start_workflow")
            .unwrap_or(&false)
        {
            return Err(DomainError::Match("start_workflow forced fail".into()));
        }
        let w = WorkflowInstance {
            workflow_instance_id: workflow_instance_id.into(),
            tenant_id: tenant_id.into(),
            status: WorkflowStatus::Running,
        };
        self.workflows
            .lock()
            .unwrap()
            .insert(workflow_instance_id.into(), w.clone());
        Ok(w)
    }

    pub fn abort_workflow(&self, workflow_instance_id: &str) -> Result<(), DomainError> {
        let mut workflows = self.workflows.lock().unwrap();
        let w = workflows
            .get_mut(workflow_instance_id)
            .ok_or_else(|| DomainError::Match("not found".into()))?;
        w.status = WorkflowStatus::Aborted;
        Ok(())
    }
}

// === SocialService ===
// 业务: 通知发送/标记已读

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notification {
    pub notification_id: String,
    pub tenant_id: String,
    pub user_id: String,
    pub read: bool,
}

#[derive(Default)]
pub struct SocialService {
    notifications: Arc<Mutex<HashMap<String, Notification>>>,
    failure_injection: Arc<Mutex<HashMap<String, bool>>>,
}

impl SocialService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_failure(&self, action: &str, fail: bool) {
        *self
            .failure_injection
            .lock()
            .unwrap()
            .entry(action.into())
            .or_insert(false) = fail;
    }

    pub fn send_notification(
        &self,
        tenant_id: &str,
        user_id: &str,
        notification_id: &str,
    ) -> Result<Notification, DomainError> {
        if *self
            .failure_injection
            .lock()
            .unwrap()
            .get("send_notification")
            .unwrap_or(&false)
        {
            return Err(DomainError::Social("send_notification forced fail".into()));
        }
        let n = Notification {
            notification_id: notification_id.into(),
            tenant_id: tenant_id.into(),
            user_id: user_id.into(),
            read: false,
        };
        self.notifications
            .lock()
            .unwrap()
            .insert(notification_id.into(), n.clone());
        Ok(n)
    }

    pub fn mark_read(&self, notification_id: &str) -> Result<(), DomainError> {
        let mut notifications = self.notifications.lock().unwrap();
        let n = notifications
            .get_mut(notification_id)
            .ok_or_else(|| DomainError::Social("not found".into()))?;
        n.read = true;
        Ok(())
    }
}

// === AdminService ===
// 业务: 角色分配/撤销

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleAssignment {
    pub role_id: String,
    pub tenant_id: String,
    pub user_id: String,
    pub active: bool,
}

#[derive(Default)]
pub struct AdminService {
    assignments: Arc<Mutex<HashMap<String, RoleAssignment>>>,
    failure_injection: Arc<Mutex<HashMap<String, bool>>>,
}

impl AdminService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_failure(&self, action: &str, fail: bool) {
        *self
            .failure_injection
            .lock()
            .unwrap()
            .entry(action.into())
            .or_insert(false) = fail;
    }

    pub fn assign_role(
        &self,
        tenant_id: &str,
        user_id: &str,
        role_id: &str,
    ) -> Result<RoleAssignment, DomainError> {
        if *self
            .failure_injection
            .lock()
            .unwrap()
            .get("assign_role")
            .unwrap_or(&false)
        {
            return Err(DomainError::Admin("assign_role forced fail".into()));
        }
        let r = RoleAssignment {
            role_id: role_id.into(),
            tenant_id: tenant_id.into(),
            user_id: user_id.into(),
            active: true,
        };
        self.assignments
            .lock()
            .unwrap()
            .insert(role_id.into(), r.clone());
        Ok(r)
    }

    pub fn revoke_role(&self, role_id: &str) -> Result<(), DomainError> {
        let mut assignments = self.assignments.lock().unwrap();
        let r = assignments
            .get_mut(role_id)
            .ok_or_else(|| DomainError::Admin("not found".into()))?;
        r.active = false;
        Ok(())
    }
}
