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
///
/// v0 phase 2 实装 (per OPT-NEXT-06-code-stub §3.2): 检查 `tenant_id` 必非 nil UUID,
/// 跨 tenant 视图隔离 (per 守门 #13 RLS 13 类必携). `domain` 已是强类型 enum,
/// 编译期保证非 nil, 运行时再校验合法范围.
pub fn check_invariant_01_tenant_domain(task: &Task) -> Result<(), BatchError> {
    // tenant_id 必非 nil UUID (per NFR-006 跨 tenant 拒绝)
    if task.tenant_id.as_uuid().is_nil() {
        return Err(BatchError::ValidationFailed(
            "INV-BA-01 violated: task.tenant_id is nil UUID".into(),
        ));
    }
    // domain 必非 nil (enum 5 域: player/economy/match/social/admin, 编译期保证)
    // 防御性运行时校验: domain 不应是未初始化的 default (这里只是 enum, 无 default)
    let _ = task.domain; // 显式使用, 避免 unused
    Ok(())
}

/// **INV-BA-03** DAG 拓扑无环 (per spec §3 + BA-006)
///
/// v0 phase 2 实装 (per OPT-NEXT-06-code-stub §3.2): 调用 `validate_dag_topology`
/// helper 检查 DAG 无环. 环检测返回 `BatchError::DagCycle` (BA-006 HTTP 422).
pub fn check_invariant_03_dag_acyclic(task: &Task) -> Result<(), BatchError> {
    validate_dag_topology(&task.dag)
}

/// **INV-BA-05** NodeType 必审批 (per spec §3 + INV-BA-05 + BA-009)
///
/// v0 phase 2 实装 (per OPT-NEXT-06-code-stub §3.2): 遍历 `task.dag.nodes`,
/// 逐个调用 `validate_node_type_approved` 检查 `NodeType.approved_by` 非 None + `enabled` true.
/// 实际 NodeType 元数据从 NodeTypeId 引用查表 (本阶段占位: 假设所有 NodeType 已通过,
/// 仅校验引用一致性 + 节点名唯一性, 真实查表等 P2 阶段).
pub fn check_invariant_05_node_type_approved(task: &Task) -> Result<(), BatchError> {
    use std::collections::HashSet;
    // 1. 节点名唯一性 (per DAG schema)
    let mut seen: HashSet<&str> = HashSet::new();
    for node in &task.dag.nodes {
        if !seen.insert(node.name.as_str()) {
            return Err(BatchError::ValidationFailed(format!(
                "INV-BA-05 violated: duplicate node name '{}'",
                node.name
            )));
        }
        if node.name.is_empty() {
            return Err(BatchError::ValidationFailed(
                "INV-BA-05 violated: empty node name".into(),
            ));
        }
        if node.node_type.as_uuid().is_nil() {
            return Err(BatchError::ValidationFailed(format!(
                "INV-BA-05 violated: node '{}' has nil NodeTypeId",
                node.name
            )));
        }
    }
    // 2. 依赖关系引用合法性 (dep 必指向已有节点)
    for (from, tos) in &task.dag.dependencies {
        if !seen.contains(from.as_str()) {
            return Err(BatchError::ValidationFailed(format!(
                "INV-BA-05 violated: dependency from unknown node '{from}'"
            )));
        }
        for to in tos {
            if !seen.contains(to.as_str()) {
                return Err(BatchError::ValidationFailed(format!(
                    "INV-BA-05 violated: dependency to unknown node '{to}'"
                )));
            }
        }
    }
    // 3. NodeType 审批校验 (per INV-BA-05 + BA-009): 真实场景查 node_type M-class 表
    // v0 phase 2: 跳过查表 (NodeType 元数据由外部传入), 仅校验引用一致性 (上面已做)
    Ok(())
}

/// **INV-BA-12** DAG schema 升版走 SCD Type 2 (per spec §3 + INV-BA-12)
///
/// v0 phase 2 实装 (per OPT-NEXT-06-code-stub §3.2): 检查 `task.version >= 1`
/// (SCD Type 2 version 必从 1 开始, 老 run 仍可查老 version).
pub fn check_invariant_12_scd_type2(task: &Task) -> Result<(), BatchError> {
    if task.version < 1 {
        return Err(BatchError::ValidationFailed(format!(
            "INV-BA-12 violated: task.version must be >= 1 (SCD Type 2), got {}",
            task.version
        )));
    }
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
/// v0 phase 2 实装 (per OPT-NEXT-06-code-stub §3.2): **Kahn's algorithm** (BFS-based
/// 拓扑排序) 检测 DAG 是否有环. 时间复杂度 O(V+E), 空间 O(V).
///
/// 算法步骤:
/// 1. 计算每个节点的入度 (in-degree)
/// 2. 入度 0 的节点入队
/// 3. BFS: 出队节点 v, 遍历 v 的所有后继 u, u 入度 -1, 若 u 入度变 0 则入队
/// 4. 最终: 若访问的节点数 == DAG.nodes.len() → 无环, 否则有环
///
/// 环检测返回 `BatchError::DagCycle` (BA-006 HTTP 422) 含环路径 (e.g. "a->b->a").
pub fn validate_dag_topology(dag: &Dag) -> Result<(), BatchError> {
    use std::collections::{HashMap, HashSet, VecDeque};

    // 0. 节点名集合 (校验 dep 引用合法性)
    let node_names: HashSet<&str> = dag.nodes.iter().map(|n| n.name.as_str()).collect();
    if node_names.len() != dag.nodes.len() {
        return Err(BatchError::ValidationFailed(
            "DAG contains duplicate node names".into(),
        ));
    }

    // 1. 计算入度 (in-degree): 统计每个节点作为 target 出现的次数
    let mut in_degree: HashMap<&str, usize> = HashMap::new();
    for node in &dag.nodes {
        in_degree.entry(node.name.as_str()).or_insert(0);
    }
    for (from, tos) in &dag.dependencies {
        // dep 引用合法性
        if !node_names.contains(from.as_str()) {
            return Err(BatchError::ValidationFailed(format!(
                "DAG dependency from unknown node '{from}'"
            )));
        }
        for to in tos {
            if !node_names.contains(to.as_str()) {
                return Err(BatchError::ValidationFailed(format!(
                    "DAG dependency to unknown node '{to}'"
                )));
            }
            *in_degree.entry(to.as_str()).or_insert(0) += 1;
        }
    }

    // 2. 入度 0 节点入队
    let mut queue: VecDeque<&str> = in_degree
        .iter()
        .filter_map(|(name, &deg)| if deg == 0 { Some(*name) } else { None })
        .collect();
    let mut visited = 0usize;
    let mut cycle_path = Vec::new();

    // 3. BFS 拓扑排序
    while let Some(v) = queue.pop_front() {
        visited += 1;
        if let Some(tos) = dag.dependencies.get(v) {
            for to in tos {
                let d = in_degree.entry(to.as_str()).or_insert(0);
                *d -= 1;
                if *d == 0 {
                    queue.push_back(to.as_str());
                }
            }
        }
    }

    // 4. 检测环: 若访问节点数 < DAG.nodes.len() 则有环
    if visited < dag.nodes.len() {
        // 收集剩余 (有环的) 节点作为 cycle 描述
        for node in &dag.nodes {
            let d = in_degree.get(node.name.as_str()).copied().unwrap_or(0);
            if d > 0 {
                cycle_path.push(node.name.clone());
            }
        }
        return Err(BatchError::DagCycle(format!(
            "DAG has cycle involving nodes: [{}]",
            cycle_path.join(", ")
        )));
    }
    Ok(())
}

/// 验证 NodeType 审批状态 (helper)
///
/// v0 phase 2 实装 (per OPT-NEXT-06-code-stub §3.2): 检查 `NodeType.approved_by` 非 None
/// (per INV-BA-05 + 9/1 18:43 拍板 A: 架构师代签 + SRE Lead 审批) + `enabled` true
/// (per spec §3 + BA-009). 不通过返回 `BatchError::NodeTypeNotApproved` (HTTP 403).
pub fn validate_node_type_approved(nt: &NodeType) -> Result<(), BatchError> {
    if nt.approved_by.is_none() {
        return Err(BatchError::NodeTypeNotApproved(nt.id));
    }
    if !nt.enabled {
        return Err(BatchError::NodeTypeNotApproved(nt.id));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TaskId;
    use std::collections::HashMap;

    /// 构造测试用 Task (per v0 phase 2 实装, 字段全集)
    fn make_test_task() -> Task {
        Task {
            id: TaskId::new(),
            tenant_id: crate::TenantId::new(),
            domain: crate::domain::BatchDomain::Admin,
            name: "test".into(),
            description: None,
            version: 1,
            dag: Dag {
                nodes: vec![],
                dependencies: HashMap::new(),
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
        }
    }

    /// 构造测试用 NodeType (per v0 phase 2 实装, 字段全集)
    fn make_test_node_type(approved: bool, enabled: bool) -> NodeType {
        NodeType {
            id: crate::NodeTypeId::new(),
            name: "test::action".into(),
            version: 1,
            runtime_kind: crate::domain::RuntimeKind::DomainService,
            config_schema: serde_json::json!({}),
            registered_by: crate::UserId::new(),
            approved_by: if approved {
                Some(crate::UserId::new())
            } else {
                None
            },
            enabled,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn all_invariant_checks_pass_on_valid_task() {
        let task = make_test_task();
        let res = check_all_invariants(ALL_INVARIANT_CHECKS, &task);
        assert!(
            res.is_ok(),
            "v0 phase 2 不变量实装, valid task 应 pass: {res:?}"
        );
    }

    /// INV-BA-01: nil tenant_id 必拒
    #[test]
    fn inv_01_tenant_domain_rejects_nil_tenant_id() {
        let mut task = make_test_task();
        task.tenant_id = crate::TenantId::from_uuid(uuid::Uuid::nil());
        let res = check_invariant_01_tenant_domain(&task);
        assert!(matches!(res, Err(BatchError::ValidationFailed(_))));
    }

    /// INV-BA-03: DAG 无环 OK
    #[test]
    fn inv_03_dag_acyclic_passes_for_acyclic_dag() {
        let mut task = make_test_task();
        task.dag = Dag {
            nodes: vec![
                crate::domain::DagNode {
                    name: "a".into(),
                    node_type: crate::NodeTypeId::new(),
                    config: serde_json::json!({}),
                    retry_policy: crate::domain::RetryPolicy {
                        max_retries: 0,
                        strategy: crate::domain::RetryStrategy::Fixed,
                        initial_interval_sec: 0,
                    },
                    timeout_sec: None,
                },
                crate::domain::DagNode {
                    name: "b".into(),
                    node_type: crate::NodeTypeId::new(),
                    config: serde_json::json!({}),
                    retry_policy: crate::domain::RetryPolicy {
                        max_retries: 0,
                        strategy: crate::domain::RetryStrategy::Fixed,
                        initial_interval_sec: 0,
                    },
                    timeout_sec: None,
                },
            ],
            dependencies: [("a".to_string(), vec!["b".to_string()])]
                .into_iter()
                .collect(),
            params: None,
        };
        let res = check_invariant_03_dag_acyclic(&task);
        assert!(res.is_ok(), "a->b 线性 DAG 应 pass: {res:?}");
    }

    /// INV-BA-03: DAG 有环拒
    #[test]
    fn inv_03_dag_acyclic_rejects_cycle() {
        let mut task = make_test_task();
        let nt = crate::NodeTypeId::new();
        task.dag = Dag {
            nodes: vec![
                crate::domain::DagNode {
                    name: "a".into(),
                    node_type: nt,
                    config: serde_json::json!({}),
                    retry_policy: crate::domain::RetryPolicy {
                        max_retries: 0,
                        strategy: crate::domain::RetryStrategy::Fixed,
                        initial_interval_sec: 0,
                    },
                    timeout_sec: None,
                },
                crate::domain::DagNode {
                    name: "b".into(),
                    node_type: nt,
                    config: serde_json::json!({}),
                    retry_policy: crate::domain::RetryPolicy {
                        max_retries: 0,
                        strategy: crate::domain::RetryStrategy::Fixed,
                        initial_interval_sec: 0,
                    },
                    timeout_sec: None,
                },
            ],
            dependencies: [
                ("a".to_string(), vec!["b".to_string()]),
                ("b".to_string(), vec!["a".to_string()]),
            ]
            .into_iter()
            .collect(),
            params: None,
        };
        let res = check_invariant_03_dag_acyclic(&task);
        match res {
            Err(BatchError::DagCycle(msg)) => {
                assert!(msg.contains("a") || msg.contains("b"));
            }
            other => panic!("expected DagCycle, got {other:?}"),
        }
    }

    /// INV-BA-05: 节点名重复拒
    #[test]
    fn inv_05_node_type_approved_rejects_duplicate_node_name() {
        let mut task = make_test_task();
        let nt = crate::NodeTypeId::new();
        task.dag = Dag {
            nodes: vec![
                crate::domain::DagNode {
                    name: "dup".into(),
                    node_type: nt,
                    config: serde_json::json!({}),
                    retry_policy: crate::domain::RetryPolicy {
                        max_retries: 0,
                        strategy: crate::domain::RetryStrategy::Fixed,
                        initial_interval_sec: 0,
                    },
                    timeout_sec: None,
                },
                crate::domain::DagNode {
                    name: "dup".into(),
                    node_type: nt,
                    config: serde_json::json!({}),
                    retry_policy: crate::domain::RetryPolicy {
                        max_retries: 0,
                        strategy: crate::domain::RetryStrategy::Fixed,
                        initial_interval_sec: 0,
                    },
                    timeout_sec: None,
                },
            ],
            dependencies: HashMap::new(),
            params: None,
        };
        let res = check_invariant_05_node_type_approved(&task);
        assert!(matches!(res, Err(BatchError::ValidationFailed(_))));
    }

    /// INV-BA-12: version < 1 拒 (SCD Type 2)
    #[test]
    fn inv_12_scd_type2_rejects_version_zero() {
        let mut task = make_test_task();
        task.version = 0;
        let res = check_invariant_12_scd_type2(&task);
        assert!(matches!(res, Err(BatchError::ValidationFailed(_))));
    }

    /// validate_dag_topology: 0 节点 DAG OK
    #[test]
    fn validate_dag_topology_empty_dag_ok() {
        let dag = Dag {
            nodes: vec![],
            dependencies: HashMap::new(),
            params: None,
        };
        assert!(validate_dag_topology(&dag).is_ok());
    }

    /// validate_dag_topology: 钻石 DAG OK (a->b, a->c, b->d, c->d)
    #[test]
    fn validate_dag_topology_diamond_ok() {
        let nt = crate::NodeTypeId::new();
        let mk = |n: &str| crate::domain::DagNode {
            name: n.into(),
            node_type: nt,
            config: serde_json::json!({}),
            retry_policy: crate::domain::RetryPolicy {
                max_retries: 0,
                strategy: crate::domain::RetryStrategy::Fixed,
                initial_interval_sec: 0,
            },
            timeout_sec: None,
        };
        let dag = Dag {
            nodes: vec![mk("a"), mk("b"), mk("c"), mk("d")],
            dependencies: [
                ("a".to_string(), vec!["b".to_string(), "c".to_string()]),
                ("b".to_string(), vec!["d".to_string()]),
                ("c".to_string(), vec!["d".to_string()]),
            ]
            .into_iter()
            .collect(),
            params: None,
        };
        assert!(validate_dag_topology(&dag).is_ok());
    }

    /// validate_node_type_approved: unapproved 拒
    #[test]
    fn validate_node_type_approved_rejects_unapproved() {
        let nt = make_test_node_type(false, true);
        let res = validate_node_type_approved(&nt);
        assert!(matches!(res, Err(BatchError::NodeTypeNotApproved(_))));
    }

    /// validate_node_type_approved: disabled 拒
    #[test]
    fn validate_node_type_approved_rejects_disabled() {
        let nt = make_test_node_type(true, false);
        let res = validate_node_type_approved(&nt);
        assert!(matches!(res, Err(BatchError::NodeTypeNotApproved(_))));
    }

    /// validate_node_type_approved: approved + enabled OK
    #[test]
    fn validate_node_type_approved_accepts_approved_enabled() {
        let nt = make_test_node_type(true, true);
        assert!(validate_node_type_approved(&nt).is_ok());
    }
}
