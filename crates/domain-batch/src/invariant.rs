//! 12 关键不变量 (INV-BA-01~12, per BATCH-REQ-001 §3.3 + ADR-0040 §D39 + spec §3)
//!
//! v0 phase 1: 不变量 check 函数 stub, 返回 Ok(()) 默认.
//! v0 phase 2: 实装每条不变量, 配单测覆盖.
//! 完整列表:
//!
//! - **INV-BA-01** 必带 tenant_id + domain (5 域视图), 跨 tenant/跨域拒绝 (per NFR-006)
//! - **INV-BA-02** 节点执行是异步的 (Worker 拉取, 不阻塞业务事务)
//! - **INV-BA-03** DAG 拓扑无环, 创建时拓扑校验, 环检测 422 BA-006
//! - **INV-BA-04** 节点执行幂等, 失败重试用同一 `idempotency_key` (NodeId + RunId + RetryIdx)
//! - **INV-BA-05** 节点类型注册需架构师代签 SRE Lead 审批 (per 9/1 18:43 拍板 A)
//! - **INV-BA-06** `batch_event` append-only, 不可修改/删除 (审计用, 冷热分层)
//! - **INV-BA-07** 节点超时自动 kill + 标 `failed`, 超时时间 per-node 可配
//! - **INV-BA-08** 节点 shell 执行走 non-root user + 白名单命令 + 资源限制 (per ADR-0025)
//! - **INV-BA-09** batch 引擎 crash 后, running 节点可 resume (per ADR-0030 Lease 30s heartbeat)
//! - **INV-BA-10** 5 域 (player/economy/match/social/admin) 视图隔离 (per 8/21 JST 拒绝兼任)
//! - **INV-BA-11** 节点 sql 写操作走 per-tenant db role + 写操作审计
//! - **INV-BA-12** DAG 定义 schema 升版走 SCD Type 2, 老 run 用老 version

use crate::domain::{Dag, NodeType, Task};
use crate::error::BatchError;

/// 不变量检查函数签名
pub type InvariantCheck = fn(&Task) -> Result<(), BatchError>;

/// **INV-BA-01** tenant_id + domain 必非 nil (per spec §3)
pub fn check_invariant_01_tenant_domain(_task: &Task) -> Result<(), BatchError> {
    // v0 phase 1: stub, v0 phase 2 实装
    Ok(())
}

/// **INV-BA-03** DAG 拓扑无环 (per spec §3 + BA-006)
pub fn check_invariant_03_dag_acyclic(task: &Task) -> Result<(), BatchError> {
    // v0 phase 1: stub, v0 phase 2 实装 (用 DagOrchestrator::validate_topology)
    let _ = task;
    Ok(())
}

/// **INV-BA-05** NodeType 必审批 (per spec §3 + INV-BA-05 + BA-009)
pub fn check_invariant_05_node_type_approved(_task: &Task) -> Result<(), BatchError> {
    // v0 phase 1: stub, v0 phase 2 实装
    Ok(())
}

/// **INV-BA-12** DAG schema 升版走 SCD Type 2 (per spec §3 + INV-BA-12)
pub fn check_invariant_12_scd_type2(_task: &Task) -> Result<(), BatchError> {
    // v0 phase 1: stub, v0 phase 2 实装
    Ok(())
}

/// 全部 12 不变量 (v0 phase 1: 4 stub 实装, 8 stub 占位)
pub const ALL_INVARIANT_CHECKS: &[InvariantCheck] = &[
    check_invariant_01_tenant_domain,
    check_invariant_03_dag_acyclic,
    check_invariant_05_node_type_approved,
    check_invariant_12_scd_type2,
];

/// 运行全部不变量检查
pub fn check_all_invariants(checks: &[InvariantCheck], task: &Task) -> Result<(), BatchError> {
    for c in checks {
        c(task)?;
    }
    Ok(())
}

/// 验证 DAG 拓扑 (helper, 走 `DagOrchestrator::validate_topology` 接口)
///
/// v0 phase 1: stub.
/// v0 phase 2: 实装 DFS/Kahn 算法, 检测环返回 `BatchError::DagCycle` (BA-006).
pub fn validate_dag_topology(_dag: &Dag) -> Result<(), BatchError> {
    // v0 phase 1: stub pass, v0 phase 2 实装
    Ok(())
}

/// 验证 NodeType 审批状态 (helper)
///
/// v0 phase 1: stub.
/// v0 phase 2: 检查 `NodeType.approved_by` 非 None + `enabled` true.
pub fn validate_node_type_approved(_nt: &NodeType) -> Result<(), BatchError> {
    // v0 phase 1: stub pass, v0 phase 2 实装
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TaskId;

    #[test]
    fn all_invariant_checks_pass_on_empty_task() {
        let task = Task {
            id: TaskId::new(),
            tenant_id: crate::TenantId::new(),
            domain: crate::domain::BatchDomain::Admin,
            name: "test".into(),
            description: None,
            version: 1,
            dag: Dag {
                nodes: vec![],
                dependencies: std::collections::HashMap::new(),
                params: None,
            },
            cron: None,
            timezone: "UTC".into(),
            enabled: true,
            catchup_policy: crate::domain::CatchupPolicy::Skip,
            trigger_type: crate::domain::TriggerType::Manual,
            event_filter: None,
            alert_rule_ids: vec![],
            sla_id: None,
            created_by: crate::UserId::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            last_run_at: None,
            status: crate::domain::TaskStatus::Draft,
        };
        let res = check_all_invariants(ALL_INVARIANT_CHECKS, &task);
        assert!(res.is_ok(), "v0 phase 1 不变量 stub 全部应 pass: {res:?}");
    }

    #[test]
    fn validate_dag_topology_stub_pass() {
        let dag = Dag {
            nodes: vec![],
            dependencies: std::collections::HashMap::new(),
            params: None,
        };
        assert!(validate_dag_topology(&dag).is_ok());
    }

    #[test]
    fn validate_node_type_approved_stub_pass() {
        let nt = NodeType {
            id: crate::NodeTypeId::new(),
            name: "test::action".into(),
            version: 1,
            runtime_kind: crate::domain::RuntimeKind::DomainService,
            config_schema: serde_json::json!({}),
            registered_by: crate::UserId::new(),
            approved_by: None, // v0 phase 2 实装时改为必填
            enabled: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        assert!(validate_node_type_approved(&nt).is_ok());
    }
}
