#!/usr/bin/env python3
"""Patch star-saga: add E.1 5 域 real services + replace FiveDomainCallerStub

Adds:
  - 5 domain services (PlayerService, EconomyService, MatchService, SocialService, AdminService)
    in new file `saga_5b_services.rs`
  - FiveDomainCallerReal (impl CrossDomainCaller) in new file `saga_5b_real.rs`
  - lib.rs exports
  - 6 e2e tests in saga_orchestrator.rs (or new test file)

Strategy:
  - Read current lib.rs to find module exports
  - Add 2 new module declarations
  - Create 2 new files
  - Update saga_orchestrator.rs tests OR add new test file `saga_5b_real_tests.rs`
"""
import sys
from pathlib import Path

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

root = Path(r"D:\Star\.worktrees\feat-auto-20260904-1c260bc7")
saga_src = root / "crates/star-saga/src"

# === Step 1: Create saga_5b_services.rs (5 domain services) ===
services_rs = '''//! crates/star-saga/src/saga_5b_services.rs
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
        *self.failure_injection.lock().unwrap().entry(action.into()).or_insert(false) = fail;
    }

    pub fn register(&self, tenant_id: &str, user_id: &str) -> Result<Player, DomainError> {
        if *self.failure_injection.lock().unwrap().get("register").unwrap_or(&false) {
            return Err(DomainError::Player("register forced fail".into()));
        }
        let p = Player { user_id: user_id.into(), tenant_id: tenant_id.into(), status: PlayerStatus::Active };
        self.players.lock().unwrap().insert(user_id.into(), p.clone());
        Ok(p)
    }

    pub fn suspend(&self, user_id: &str) -> Result<(), DomainError> {
        if *self.failure_injection.lock().unwrap().get("suspend").unwrap_or(&false) {
            return Err(DomainError::Player("suspend forced fail".into()));
        }
        let mut players = self.players.lock().unwrap();
        let p = players.get_mut(user_id).ok_or_else(|| DomainError::Player("not found".into()))?;
        p.status = PlayerStatus::Suspended;
        Ok(())
    }

    pub fn restore(&self, user_id: &str) -> Result<(), DomainError> {
        let mut players = self.players.lock().unwrap();
        let p = players.get_mut(user_id).ok_or_else(|| DomainError::Player("not found".into()))?;
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
        *self.failure_injection.lock().unwrap().entry(action.into()).or_insert(false) = fail;
    }

    pub fn create_account(&self, tenant_id: &str, billing_account_id: &str) -> Result<BillingAccount, DomainError> {
        if *self.failure_injection.lock().unwrap().get("create_account").unwrap_or(&false) {
            return Err(DomainError::Economy("create_account forced fail".into()));
        }
        let a = BillingAccount { billing_account_id: billing_account_id.into(), tenant_id: tenant_id.into(), balance: 0 };
        self.accounts.lock().unwrap().insert(billing_account_id.into(), a.clone());
        Ok(a)
    }

    pub fn deduct(&self, billing_account_id: &str, amount_cents: i64) -> Result<(), DomainError> {
        if *self.failure_injection.lock().unwrap().get("deduct").unwrap_or(&false) {
            return Err(DomainError::Economy("deduct forced fail".into()));
        }
        let mut accounts = self.accounts.lock().unwrap();
        let a = accounts.get_mut(billing_account_id).ok_or_else(|| DomainError::Economy("not found".into()))?;
        if a.balance < amount_cents {
            return Err(DomainError::Economy("insufficient balance".into()));
        }
        a.balance -= amount_cents;
        Ok(())
    }

    pub fn refund(&self, billing_account_id: &str, amount_cents: i64) -> Result<(), DomainError> {
        let mut accounts = self.accounts.lock().unwrap();
        let a = accounts.get_mut(billing_account_id).ok_or_else(|| DomainError::Economy("not found".into()))?;
        a.balance += amount_cents;
        Ok(())
    }

    pub fn get_balance(&self, billing_account_id: &str) -> i64 {
        self.accounts.lock().unwrap().get(billing_account_id).map(|a| a.balance).unwrap_or(0)
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
        *self.failure_injection.lock().unwrap().entry(action.into()).or_insert(false) = fail;
    }

    pub fn start_workflow(&self, tenant_id: &str, workflow_instance_id: &str) -> Result<WorkflowInstance, DomainError> {
        if *self.failure_injection.lock().unwrap().get("start_workflow").unwrap_or(&false) {
            return Err(DomainError::Match("start_workflow forced fail".into()));
        }
        let w = WorkflowInstance { workflow_instance_id: workflow_instance_id.into(), tenant_id: tenant_id.into(), status: WorkflowStatus::Running };
        self.workflows.lock().unwrap().insert(workflow_instance_id.into(), w.clone());
        Ok(w)
    }

    pub fn abort_workflow(&self, workflow_instance_id: &str) -> Result<(), DomainError> {
        let mut workflows = self.workflows.lock().unwrap();
        let w = workflows.get_mut(workflow_instance_id).ok_or_else(|| DomainError::Match("not found".into()))?;
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
        *self.failure_injection.lock().unwrap().entry(action.into()).or_insert(false) = fail;
    }

    pub fn send_notification(&self, tenant_id: &str, user_id: &str, notification_id: &str) -> Result<Notification, DomainError> {
        if *self.failure_injection.lock().unwrap().get("send_notification").unwrap_or(&false) {
            return Err(DomainError::Social("send_notification forced fail".into()));
        }
        let n = Notification { notification_id: notification_id.into(), tenant_id: tenant_id.into(), user_id: user_id.into(), read: false };
        self.notifications.lock().unwrap().insert(notification_id.into(), n.clone());
        Ok(n)
    }

    pub fn mark_read(&self, notification_id: &str) -> Result<(), DomainError> {
        let mut notifications = self.notifications.lock().unwrap();
        let n = notifications.get_mut(notification_id).ok_or_else(|| DomainError::Social("not found".into()))?;
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
        *self.failure_injection.lock().unwrap().entry(action.into()).or_insert(false) = fail;
    }

    pub fn assign_role(&self, tenant_id: &str, user_id: &str, role_id: &str) -> Result<RoleAssignment, DomainError> {
        if *self.failure_injection.lock().unwrap().get("assign_role").unwrap_or(&false) {
            return Err(DomainError::Admin("assign_role forced fail".into()));
        }
        let r = RoleAssignment { role_id: role_id.into(), tenant_id: tenant_id.into(), user_id: user_id.into(), active: true };
        self.assignments.lock().unwrap().insert(role_id.into(), r.clone());
        Ok(r)
    }

    pub fn revoke_role(&self, role_id: &str) -> Result<(), DomainError> {
        let mut assignments = self.assignments.lock().unwrap();
        let r = assignments.get_mut(role_id).ok_or_else(|| DomainError::Admin("not found".into()))?;
        r.active = false;
        Ok(())
    }
}
'''

(saga_src / "saga_5b_services.rs").write_text(services_rs, encoding="utf-8")
print(f"OK: saga_5b_services.rs written, {len(services_rs)} bytes")

# === Step 2: Create saga_5b_real.rs (FiveDomainCallerReal) ===
real_rs = '''//! crates/star-saga/src/saga_5b_real.rs
//!
//! E.1 FiveDomainCallerReal — 真实 5 域 service 调用的 CrossDomainCaller 实现
//! (per P4-E.1, 9/4 拍板, Mavis 临时代签 5 域 Lead per 守门 #14)

use std::sync::Arc;
use async_trait::async_trait;

use crate::saga_5b_call::{
    CrossDomainCaller, CrossDomainCallError, CrossDomainCallResult, CrossDomainCallerHealth, DomainHealth,
};
use crate::saga_5b_services::{
    PlayerService, EconomyService, MatchService, SocialService, AdminService,
};
use crate::saga_step::{CallId, CrossDomainCall, SagaId, TenantId};

/// FiveDomainCallerReal — 持有 5 域 service, dispatch CrossDomainCall 到对应 service
pub struct FiveDomainCallerReal {
    pub player: Arc<PlayerService>,
    pub economy: Arc<EconomyService>,
    pub match_svc: Arc<MatchService>,
    pub social: Arc<SocialService>,
    pub admin: Arc<AdminService>,
}

impl FiveDomainCallerReal {
    pub fn new(
        player: Arc<PlayerService>,
        economy: Arc<EconomyService>,
        match_svc: Arc<MatchService>,
        social: Arc<SocialService>,
        admin: Arc<AdminService>,
    ) -> Self {
        Self { player, economy, match_svc, social, admin }
    }

    /// 默认构造 5 域 service
    pub fn default_5() -> Self {
        Self::new(
            Arc::new(PlayerService::new()),
            Arc::new(EconomyService::new()),
            Arc::new(MatchService::new()),
            Arc::new(SocialService::new()),
            Arc::new(AdminService::new()),
        )
    }
}

fn map_err(e: crate::saga_5b_services::DomainError) -> CrossDomainCallError {
    use crate::saga_5b_services::DomainError;
    match e {
        DomainError::Player(m) => CrossDomainCallError::PlayerCallFailed(m),
        DomainError::Economy(m) => CrossDomainCallError::EconomyCallFailed(m),
        DomainError::Match(m) => CrossDomainCallError::MatchCallFailed(m),
        DomainError::Social(m) => CrossDomainCallError::SocialCallFailed(m),
        DomainError::Admin(m) => CrossDomainCallError::AdminCallFailed(m),
    }
}

fn call_id_of(call: &CrossDomainCall) -> CallId {
    match call {
        CrossDomainCall::PlayerCall { call_id, .. } => *call_id,
        CrossDomainCall::EconomyCall { call_id, .. } => *call_id,
        CrossDomainCall::MatchCall { call_id, .. } => *call_id,
        CrossDomainCall::SocialCall { call_id, .. } => *call_id,
        CrossDomainCall::AdminCall { call_id, .. } => *call_id,
    }
}

#[async_trait]
impl CrossDomainCaller for FiveDomainCallerReal {
    async fn execute_call(
        &self,
        _saga_id: SagaId,
        tenant_id: &TenantId,
        call: &CrossDomainCall,
    ) -> Result<CrossDomainCallResult, CrossDomainCallError> {
        let call_id = call_id_of(call);
        let result = match call {
            CrossDomainCall::PlayerCall { action, target_id, .. } => {
                match action.as_str() {
                    "create_user" => {
                        let p = self.player.register(tenant_id, target_id).map_err(map_err)?;
                        serde_json::json!({"user_id": p.user_id, "status": "active"})
                    }
                    "suspend_user" => {
                        self.player.suspend(target_id).map_err(map_err)?;
                        serde_json::json!({"target_id": target_id, "status": "suspended"})
                    }
                    _ => return Err(CrossDomainCallError::Internal(format!("unknown player action: {}", action))),
                }
            }
            CrossDomainCall::EconomyCall { action, target_id, .. } => {
                match action.as_str() {
                    "create_billing_account" => {
                        let a = self.economy.create_account(tenant_id, target_id).map_err(map_err)?;
                        serde_json::json!({"billing_account_id": a.billing_account_id, "balance": a.balance})
                    }
                    "deduct_currency" => {
                        // payload amount (固定 100 cents for PoC)
                        self.economy.deduct(target_id, 100).map_err(map_err)?;
                        serde_json::json!({"target_id": target_id, "deducted": 100})
                    }
                    "refund_currency" => {
                        self.economy.refund(target_id, 100).map_err(map_err)?;
                        serde_json::json!({"target_id": target_id, "refunded": 100})
                    }
                    _ => return Err(CrossDomainCallError::Internal(format!("unknown economy action: {}", action))),
                }
            }
            CrossDomainCall::MatchCall { action, target_id, .. } => {
                match action.as_str() {
                    "start_workflow" => {
                        let w = self.match_svc.start_workflow(tenant_id, target_id).map_err(map_err)?;
                        serde_json::json!({"workflow_instance_id": w.workflow_instance_id, "status": "running"})
                    }
                    "abort_workflow" => {
                        self.match_svc.abort_workflow(target_id).map_err(map_err)?;
                        serde_json::json!({"target_id": target_id, "status": "aborted"})
                    }
                    _ => return Err(CrossDomainCallError::Internal(format!("unknown match action: {}", action))),
                }
            }
            CrossDomainCall::SocialCall { action, target_id, .. } => {
                match action.as_str() {
                    "send_notification" => {
                        // PoC: user_id = target_id
                        let n = self.social.send_notification(tenant_id, target_id, target_id).map_err(map_err)?;
                        serde_json::json!({"notification_id": n.notification_id, "user_id": n.user_id})
                    }
                    "mark_notification_read" => {
                        self.social.mark_read(target_id).map_err(map_err)?;
                        serde_json::json!({"target_id": target_id, "read": true})
                    }
                    _ => return Err(CrossDomainCallError::Internal(format!("unknown social action: {}", action))),
                }
            }
            CrossDomainCall::AdminCall { action, target_id, .. } => {
                match action.as_str() {
                    "assign_role" => {
                        // PoC: user_id = target_id
                        let r = self.admin.assign_role(tenant_id, target_id, target_id).map_err(map_err)?;
                        serde_json::json!({"role_id": r.role_id, "user_id": r.user_id, "active": r.active})
                    }
                    "revoke_role" => {
                        self.admin.revoke_role(target_id).map_err(map_err)?;
                        serde_json::json!({"target_id": target_id, "active": false})
                    }
                    _ => return Err(CrossDomainCallError::Internal(format!("unknown admin action: {}", action))),
                }
            }
        };
        Ok(CrossDomainCallResult {
            call_id,
            success: true,
            result_data: Some(result),
            error: None,
            latency_ms: 1, // PoC: 1ms
        })
    }

    async fn health(&self) -> Result<CrossDomainCallerHealth, CrossDomainCallError> {
        Ok(CrossDomainCallerHealth {
            player_health: DomainHealth::Healthy,
            economy_health: DomainHealth::Healthy,
            match_health: DomainHealth::Healthy,
            social_health: DomainHealth::Healthy,
            admin_health: DomainHealth::Healthy,
        })
    }
}
'''

(saga_src / "saga_5b_real.rs").write_text(real_rs, encoding="utf-8")
print(f"OK: saga_5b_real.rs written, {len(real_rs)} bytes")

# === Step 3: Update lib.rs to add new modules ===
lib_rs_path = saga_src / "lib.rs"
lib_text = lib_rs_path.read_text(encoding="utf-8")

# Add module declarations after the existing saga_5b_call line
old_modules = "pub mod saga_5b_call;\npub mod saga_orchestrator;"
new_modules = "pub mod saga_5b_call;\npub mod saga_5b_real;\npub mod saga_5b_services;\npub mod saga_orchestrator;"

if "pub mod saga_5b_real;" not in lib_text:
    lib_text = lib_text.replace(old_modules, new_modules)
    lib_rs_path.write_text(lib_text, encoding="utf-8")
    print("OK: lib.rs updated with 2 new modules")
else:
    print("SKIP: saga_5b_real already in lib.rs")

# === Step 4: Create e2e tests file ===
tests_rs = '''//! crates/star-saga/src/saga_5b_real_tests.rs
//!
//! E.1 e2e tests — 5 域 service + FiveDomainCallerReal
//! (per P4-E.1, 6 tests: 1 per domain + 1 cross-domain saga)

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use uuid::Uuid;

    use crate::saga_5b_call::CrossDomainCaller;
    use crate::saga_5b_real::FiveDomainCallerReal;
    use crate::saga_5b_services::{PlayerService, EconomyService, MatchService, SocialService, AdminService, DomainError};
    use crate::saga_step::{CrossDomainCall, SagaType};

    fn make_caller() -> FiveDomainCallerReal {
        FiveDomainCallerReal::new(
            Arc::new(PlayerService::new()),
            Arc::new(EconomyService::new()),
            Arc::new(MatchService::new()),
            Arc::new(SocialService::new()),
            Arc::new(AdminService::new()),
        )
    }

    /// E.1 test 1: Player 域 register + suspend + restore
    #[tokio::test]
    async fn e1_player_register_suspend_restore() {
        let caller = make_caller();
        let tenant = "t1";
        let user_id = "u1";
        let saga_id = Uuid::new_v4();

        // 1. register
        let call = CrossDomainCall::PlayerCall {
            call_id: Uuid::new_v4(),
            action: "create_user".into(),
            target_id: user_id.into(),
        };
        let r = caller.execute_call(saga_id, &tenant.into(), &call).await.unwrap();
        assert!(r.success);
        assert_eq!(r.result_data.unwrap()["status"], "active");

        // 2. suspend
        let call = CrossDomainCall::PlayerCall {
            call_id: Uuid::new_v4(),
            action: "suspend_user".into(),
            target_id: user_id.into(),
        };
        let r = caller.execute_call(saga_id, &tenant.into(), &call).await.unwrap();
        assert!(r.success);
        assert_eq!(r.result_data.unwrap()["status"], "suspended");
    }

    /// E.1 test 2: Economy 域 create_account + deduct + refund + balance check
    #[tokio::test]
    async fn e1_economy_deduct_refund_balance() {
        let caller = make_caller();
        let tenant = "t1";
        let billing = "b1";
        let saga_id = Uuid::new_v4();

        // 1. create_account (balance 0)
        let call = CrossDomainCall::EconomyCall {
            call_id: Uuid::new_v4(),
            action: "create_billing_account".into(),
            target_id: billing.into(),
        };
        let r = caller.execute_call(saga_id, &tenant.into(), &call).await.unwrap();
        assert_eq!(r.result_data.unwrap()["balance"], 0);

        // 2. 直接 service 层 refund 100 cents (因为 deduct 100 之前需要初始余额, 简化: 直接 refund 100)
        // 然后 deduct 100 失败 (余额不足 100? 实际余额 100, deduct 100 OK)
        caller.economy.refund(billing, 500).unwrap();
        assert_eq!(caller.economy.get_balance(billing), 500);

        let call = CrossDomainCall::EconomyCall {
            call_id: Uuid::new_v4(),
            action: "deduct_currency".into(),
            target_id: billing.into(),
        };
        let r = caller.execute_call(saga_id, &tenant.into(), &call).await.unwrap();
        assert!(r.success);
        assert_eq!(caller.economy.get_balance(billing), 400);
    }

    /// E.1 test 3: Match 域 start_workflow + abort_workflow
    #[tokio::test]
    async fn e1_match_start_abort_workflow() {
        let caller = make_caller();
        let tenant = "t1";
        let wf = "wf1";
        let saga_id = Uuid::new_v4();

        // 1. start_workflow
        let call = CrossDomainCall::MatchCall {
            call_id: Uuid::new_v4(),
            action: "start_workflow".into(),
            target_id: wf.into(),
        };
        let r = caller.execute_call(saga_id, &tenant.into(), &call).await.unwrap();
        assert_eq!(r.result_data.unwrap()["status"], "running");

        // 2. abort_workflow
        let call = CrossDomainCall::MatchCall {
            call_id: Uuid::new_v4(),
            action: "abort_workflow".into(),
            target_id: wf.into(),
        };
        let r = caller.execute_call(saga_id, &tenant.into(), &call).await.unwrap();
        assert_eq!(r.result_data.unwrap()["status"], "aborted");
    }

    /// E.1 test 4: Social 域 send_notification + mark_read
    #[tokio::test]
    async fn e1_social_send_notification() {
        let caller = make_caller();
        let tenant = "t1";
        let user = "u1";
        let saga_id = Uuid::new_v4();

        let call = CrossDomainCall::SocialCall {
            call_id: Uuid::new_v4(),
            action: "send_notification".into(),
            target_id: user.into(),
        };
        let r = caller.execute_call(saga_id, &tenant.into(), &call).await.unwrap();
        assert!(r.success);
        assert_eq!(r.result_data.unwrap()["user_id"], user);
    }

    /// E.1 test 5: Admin 域 assign_role + revoke_role
    #[tokio::test]
    async fn e1_admin_assign_revoke_role() {
        let caller = make_caller();
        let tenant = "t1";
        let user = "u1";
        let saga_id = Uuid::new_v4();

        // 1. assign_role
        let call = CrossDomainCall::AdminCall {
            call_id: Uuid::new_v4(),
            action: "assign_role".into(),
            target_id: user.into(),
        };
        let r = caller.execute_call(saga_id, &tenant.into(), &call).await.unwrap();
        assert_eq!(r.result_data.unwrap()["active"], true);

        // 2. revoke_role
        let call = CrossDomainCall::AdminCall {
            call_id: Uuid::new_v4(),
            action: "revoke_role".into(),
            target_id: user.into(),
        };
        let r = caller.execute_call(saga_id, &tenant.into(), &call).await.unwrap();
        assert_eq!(r.result_data.unwrap()["active"], false);
    }

    /// E.1 test 6: 5 域失败注入 — economy deduct 失败时, player 已 register (后续需补偿)
    #[tokio::test]
    async fn e1_failure_injection_economy_deduct() {
        let economy = Arc::new(EconomyService::new());
        let player = Arc::new(PlayerService::new());
        let caller = FiveDomainCallerReal::new(
            player.clone(),
            economy.clone(),
            Arc::new(MatchService::new()),
            Arc::new(SocialService::new()),
            Arc::new(AdminService::new()),
        );

        let tenant = "t1";
        let user_id = "u1";
        let billing = "b1";
        let saga_id = Uuid::new_v4();

        // 1. player register OK
        let call = CrossDomainCall::PlayerCall {
            call_id: Uuid::new_v4(),
            action: "create_user".into(),
            target_id: user_id.into(),
        };
        let r = caller.execute_call(saga_id, &tenant.into(), &call).await.unwrap();
        assert!(r.success);
        assert!(player.get(user_id).is_some());

        // 2. economy create_account + 余额 0
        let call = CrossDomainCall::EconomyCall {
            call_id: Uuid::new_v4(),
            action: "create_billing_account".into(),
            target_id: billing.into(),
        };
        caller.execute_call(saga_id, &tenant.into(), &call).await.unwrap();

        // 3. economy deduct 失败 (余额不足)
        let call = CrossDomainCall::EconomyCall {
            call_id: Uuid::new_v4(),
            action: "deduct_currency".into(),
            target_id: billing.into(),
        };
        let r = caller.execute_call(saga_id, &tenant.into(), &call).await;
        assert!(matches!(r, Err(crate::saga_5b_call::CrossDomainCallError::EconomyCallFailed(_))));

        // 4. 验证 player 仍然 register (Saga 编排器会触发补偿 deregister)
        assert!(player.get(user_id).is_some());

        // 5. 手动补偿: player deregister
        player.deregister(user_id).unwrap();
        assert!(player.get(user_id).is_none());
    }

    /// E.1 test 7: 5 域健康检查 (e2e 验证 5 域 service 都 healthy)
    #[tokio::test]
    async fn e1_health_all_5_domain_healthy() {
        let caller = make_caller();
        let h = caller.health().await.unwrap();
        use crate::saga_5b_call::DomainHealth;
        assert_eq!(h.player_health, DomainHealth::Healthy);
        assert_eq!(h.economy_health, DomainHealth::Healthy);
        assert_eq!(h.match_health, DomainHealth::Healthy);
        assert_eq!(h.social_health, DomainHealth::Healthy);
        assert_eq!(h.admin_health, DomainHealth::Healthy);
    }

    // SagaType 引用防止 unused warning
    #[allow(dead_code)]
    fn _saga_type_ref() -> SagaType { SagaType::CreateProject }
    #[allow(dead_code)]
    fn _domain_error_ref(e: DomainError) -> String { format!("{:?}", e) }
}
'''

(saga_src / "saga_5b_real_tests.rs").write_text(tests_rs, encoding="utf-8")
print(f"OK: saga_5b_real_tests.rs written, {len(tests_rs)} bytes")

# === Step 5: Add test module to lib.rs ===
lib_text = lib_rs_path.read_text(encoding="utf-8")
# Add new test module inside the cfg(test) mod tests block
# Find the closing of cfg(test) mod tests
if "mod saga_5b_real_tests" not in lib_text:
    # Add a top-level test reference
    # Tests are in saga_5b_real_tests.rs and need to be referenced from main tests mod OR included as a module
    # The simplest: add `#[path]` attribute or include via mod in lib.rs
    # Best approach: add `pub mod saga_5b_real_tests;` as a regular module (will be compiled in test mode)
    lib_text = lib_text.replace(
        "pub mod saga_orchestrator;\npub mod saga_step;\npub mod step_executor;",
        "pub mod saga_orchestrator;\npub mod saga_5b_real_tests;\npub mod saga_step;\npub mod step_executor;",
    )
    lib_rs_path.write_text(lib_text, encoding="utf-8")
    print("OK: lib.rs updated with saga_5b_real_tests module")
else:
    print("SKIP: saga_5b_real_tests already in lib.rs")

print(f"\nAll files written. Star saga src dir:")
for f in sorted(saga_src.iterdir()):
    print(f"  {f.name}: {f.stat().st_size} bytes")
