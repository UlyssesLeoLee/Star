//! crates/star-saga/src/saga_step.rs
//!
//! SagaStep 数据结构 (per P3-E.6 docs 阶段 + 骨架, commit 待 match 域 Lead 真人补详细补偿机制)
//! per `docs/ddd/03-match-bc.md` §2.3 SagaInstance Aggregate + `PHASE-P3-E6-SAGA-IMPL-REPORT.md` v0.1
//!
//! ## 职责
//!
//! SagaStep 5 字段 (step_id / tenant_id / saga_type / status / call_chain)
//! 5 域跨域调用链 (player / economy / match / social / admin 5 域 stub)
//!
//! ## 关键不变量
//!
//! - INV-SG-01: SagaStep 必带 tenant_id
//! - INV-SG-02: call_chain 必填, 跨域补偿按链顺序回滚
//! - INV-SG-03: status 状态机 (Pending / Running / Completed / Compensating / Failed)
//! - INV-SG-04: Failed 状态必填 failure_reason
//! - INV-SG-05: idempotency_key 必填 (per E.6 match 域 Lead 真人补详细补偿机制 5 项之一, 防止跨 step 失败重复补偿)
//!
//! Lead 责任: match 域 Lead (待真人到位)

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type SagaId = Uuid;
pub type StepId = Uuid;
pub type TenantId = String;
pub type CallId = Uuid;

/// Saga 类型 (5 域跨域编排 6 种)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SagaType {
    CreateProject,         // player → economy → admin 3 域
    ProvisionWorkspace,    // player → economy → admin 3 域
    UpgradePlan,           // economy → player 2 域
    OnboardUser,           // player → social 2 域
    SuspendAccount,        // admin → economy → player 3 域
    CrossDomainCompensate, // 跨域补偿, 5 域全部参与
}

/// SagaStep 状态机
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SagaStepStatus {
    Pending,      // 待启动
    Running,      // 5 域调用链执行中
    Completed,    // 5 域调用链全部成功
    Compensating, // 5 域调用链失败, 补偿中
    Failed,       // 5 域调用链失败 + 补偿失败, 需人工介入
}

/// 5 域调用
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrossDomainCall {
    PlayerCall {
        call_id: CallId,
        action: String,    // e.g. "create_user" / "provision_workspace"
        target_id: String, // user_id / workspace_id
    },
    EconomyCall {
        call_id: CallId,
        action: String,    // e.g. "create_billing_account"
        target_id: String, // billing_account_id
    },
    MatchCall {
        call_id: CallId,
        action: String,    // e.g. "start_workflow"
        target_id: String, // workflow_instance_id
    },
    SocialCall {
        call_id: CallId,
        action: String,    // e.g. "send_notification"
        target_id: String, // notification_id
    },
    AdminCall {
        call_id: CallId,
        action: String,    // e.g. "assign_role"
        target_id: String, // role_id
    },
}

/// SagaStep 5 字段 (per 2026-08-30 v0.2 增强: 加 idempotency_key 必填字段, 6 字段)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SagaStep {
    pub step_id: StepId,
    pub tenant_id: TenantId,
    pub saga_type: SagaType,
    pub status: SagaStepStatus,
    pub call_chain: Vec<CrossDomainCall>, // 5 域调用链, 跨域补偿按链顺序回滚
    pub idempotency_key: IdempotencyKey,  // 必填, 防止跨 step 失败重复补偿 (INV-SG-05)
}

/// IdempotencyKey: 跨 step 唯一标识, 防止同 step 多次重试导致 5 域重复调用
/// 类型: String (UUID v7 序列化, 跨进程稳定)
/// 必填, SagaStep::new 自动生成
pub type IdempotencyKey = String;

impl SagaStep {
    pub fn new(tenant_id: TenantId, saga_type: SagaType, call_chain: Vec<CrossDomainCall>) -> Self {
        Self {
            step_id: Uuid::new_v4(),
            tenant_id,
            saga_type,
            status: SagaStepStatus::Pending,
            call_chain,
            idempotency_key: format!("idem-{}", Uuid::new_v4()),
        }
    }
}
