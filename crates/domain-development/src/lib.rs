//! Development Execution + Development Context 领域
//!
//! **crate**: `domain-development`
//! **上游 spec**: docs/specs/domain-development-spec.md
//! **基本设计**: docs/basic-design.md §2.1 / §4.9.5 / §4.10.4
//! **数据设计**: docs/data-design.md §4.19 (`development` schema)
//! **API 设计**: docs/api-design.md §3.20 (Development endpoints)
//!
//! ## 职责
//!
//! DevelopmentExecution + ChangeSet + SymbolIndex + RepositoryContext + DevelopmentContext
//! 5 个聚合根,负责 WorkItem 在真实代码环境中的一次或多次执行的结构化记录。
//! - DevelopmentExecution 聚合根(WorkItem → 多个 Worktree / AgentSession / ChangeSet / Validation / Feedback / Commit / PR 的汇总)
//! - ChangeSet 聚合根(不只存 Git Diff,需承载 Files / Symbols / Diff / Risk Signals 等结构化信息,§21.1)
//! - SymbolIndex / RepositoryContext / DevelopmentContext 投影(由 worker 异步刷新)
//! - Diff 全文(Object Storage 引用,§21.1)
//!
//! ## 关键不变量
//!
//! - ChangeSet ≠ Git Diff(INV-DX-01,§21.1)
//! - 1 ChangeSet 关联 1 Commit(INV-DX-02,§4.8.4)
//! - Diff 全文不存 PostgreSQL(INV-DX-03,§4.8.3)
//! - 8 种 Risk Signal 类型(INV-DX-04,§4.8.5)
//! - Object Storage Key 必带 tenant_id 前缀(INV-DX-05,§6.1)
//! - SymbolIndex 跨 Repository 不合并(INV-DX-06,§6.6)
//! - AISelfClaim 必走 Validation Chain(INV-DX-07,VAL-001)
//! - Symbol-aware Context 第一阶段 File-level + Basic Symbol Detection(INV-DX-08,§4.8.6)
//! - DevelopmentExecution.worktree_ids 1..N(INV-DX-09)
//! - 已 commit 的 ChangeSet 不可修改(INV-DX-10)
//!
//! ## 上游依赖
//!
//! 本 crate 仅依赖自身外部依赖,无跨 domain-* crate 依赖(spec §12 边界清单)。
//!
//! ## 关键引用
//!
//! Development Context(§20)合并入本 crate;SymbolIndex / RepositoryContext / DevelopmentContext
//! 由 worker 异步刷新。

#![allow(missing_docs)]
#![warn(rust_2018_idioms)]

// =====================================================================
// 子模块装载
// =====================================================================

pub mod context;
pub mod entity;
pub mod error;
pub mod event;
pub mod invariants;
pub mod macros;
pub mod port;
pub mod service;
pub mod value_object;

// =====================================================================
// 便捷 re-export
// =====================================================================

pub use context::ActorContext;
pub use entity::{
    ChangeSet, DevelopmentContext, DevelopmentExecution, FileChange, IndexedSymbol,
    RepositoryContext, RiskSignal, SymbolChange, SymbolIndex,
};
pub use error::DevelopmentError;
pub use event::{
    ChangeSetObserved, DevelopmentEvent, EventMeta, ExecutionClosed, ExecutionCreated,
    RiskSignalDetected, SymbolIndexRefreshed,
};
pub use invariants::{
    check_append_change_set_invariants, check_invariant_01_structured_change_set,
    check_invariant_02_one_commit_per_change_set, check_invariant_03_diff_reference_only,
    check_invariant_04_eight_risk_signal_kinds, check_invariant_05_tenant_prefix_in_storage_key,
    check_invariant_06_symbol_index_repository_boundary,
    check_invariant_07_ai_self_claim_validation, check_invariant_08_file_level_symbols,
    check_invariant_09_at_least_one_worktree, check_invariant_10_change_set_not_committed,
    ALL_INVARIANT_CHECKS,
};
pub use port::{
    AppendChangeSetCommand, AttachRiskSignalCommand, BuildSymbolIndexCommand,
    CloseExecutionCommand, CreateExecutionCommand, DevelopmentCommandPort, DevelopmentQueryPort,
    DevelopmentRepository, DiffDownloadURL, FileChangeDraft, ListChangeSetQuery,
    ListExecutionQuery, ListSymbolQuery, RiskSignalDraft, SearchSymbolQuery, SymbolSeed,
};
pub use service::InMemoryDevelopmentService;
pub use value_object::{
    roles, AgentSessionId, ChangeSetId, CommitId, ExecutionId, ExecutionState, FileChangeId,
    FileChangeStatus, FilePath, LineRange, ProjectId, RepositoryContextId, RepositoryId,
    RiskSeverity, RiskSignalId, RiskSignalKind, RiskSource, SymbolId, SymbolIndexId,
    SymbolKind, TenantId, UserId, WorkItemId, WorktreeId,
};

// =====================================================================
// 单元测试
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value_object::{
        ExecutionState, FilePath, ProjectId, RiskSeverity, RiskSignalKind, RiskSource,
        TenantId, UserId, WorkItemId, WorktreeId,
    };

    // -------- 测试夹具 --------

    fn make_test_actor(tenant_id: TenantId) -> ActorContext {
        ActorContext::new(UserId::new(), tenant_id).with_role(roles::DEVELOPER)
    }

    fn make_diff_ref(tenant_id: TenantId) -> String {
        format!("development.diff/{tenant_id}/00000000-0000-0000-0000-000000000000.diff")
    }

    // -------- 1. ActorContext + 强类型 ID smoke test --------

    #[test]
    fn actor_context_typed_ids() {
        let tenant_id = TenantId::new();
        let actor = make_test_actor(tenant_id);
        assert!(!actor.tenant_id.as_uuid().is_nil());
        assert!(actor.is_developer());
        assert!(!actor.is_tenant_admin());
    }

    // -------- 2. 实体字段数审计 --------

    #[test]
    fn entity_field_count_audit() {
        assert_eq!(DevelopmentExecution::FIELD_COUNT, 17);
        assert_eq!(ChangeSet::FIELD_COUNT, 19);
        assert_eq!(FileChange::FIELD_COUNT, 7);
        assert_eq!(SymbolChange::FIELD_COUNT, 7);
        assert_eq!(RiskSignal::FIELD_COUNT, 8);
        assert_eq!(SymbolIndex::FIELD_COUNT, 6);
        assert_eq!(IndexedSymbol::FIELD_COUNT, 6);
        assert_eq!(RepositoryContext::FIELD_COUNT, 10);
        assert_eq!(DevelopmentContext::FIELD_COUNT, 10);
    }

    // -------- 3. FilePath 值对象校验 --------

    #[test]
    fn file_path_validation() {
        // 合法的相对路径
        assert!(FilePath::new("src/main.rs").is_ok());
        assert!(FilePath::new("a/b/c.txt").is_ok());
        // 空字符串拒绝
        assert!(FilePath::new("").is_err());
        // 以 / 开头拒绝
        assert!(FilePath::new("/abs/path.rs").is_err());
        // 包含 .. 拒绝
        assert!(FilePath::new("../etc/passwd").is_err());
        // extension 提取
        let p = FilePath::new("src/main.rs").unwrap();
        assert_eq!(p.extension(), Some("rs"));
    }

    // -------- 4. 8 种 RiskSignalKind 锁定(INV-DX-04) --------

    #[test]
    fn eight_risk_signal_kinds_locked() {
        let all = RiskSignalKind::ALL;
        assert_eq!(all.len(), 8);
        for k in all {
            assert!(k.is_known());
        }
    }

    // -------- 5. ExecutionState 状态机合法迁移 --------

    #[test]
    fn execution_state_transitions() {
        use value_object::is_valid_state_transition;
        // 合法
        assert!(is_valid_state_transition(
            ExecutionState::Pending,
            ExecutionState::Running
        ));
        assert!(is_valid_state_transition(
            ExecutionState::Running,
            ExecutionState::Succeeded
        ));
        assert!(is_valid_state_transition(
            ExecutionState::Running,
            ExecutionState::Failed
        ));
        // 终态不可迁出
        assert!(!is_valid_state_transition(
            ExecutionState::Succeeded,
            ExecutionState::Running
        ));
        assert!(!is_valid_state_transition(
            ExecutionState::Failed,
            ExecutionState::Running
        ));
        // 同态禁止
        assert!(!is_valid_state_transition(
            ExecutionState::Running,
            ExecutionState::Running
        ));
    }

    // -------- 6. create_execution 成功路径(INV-DX-09) --------

    #[tokio::test]
    async fn create_execution_success() {
        let svc = InMemoryDevelopmentService::new_for_test();
        let tenant_id = TenantId::new();
        let actor = make_test_actor(tenant_id);
        let cmd = CreateExecutionCommand {
            tenant_id,
            project_id: ProjectId::new(),
            work_item_id: WorkItemId::new(),
            repository_id: RepositoryId::new(),
            worktree_ids: vec![WorktreeId::new()],
        };
        let id = svc
            .create_execution(cmd, actor)
            .await
            .expect("创建成功");
        assert_eq!(svc.execution_count().await, 1);
        // 验证 get_execution
        let viewer = make_test_actor(tenant_id);
        let e = svc.get_execution(id, viewer).await.expect("get");
        assert!(!e.is_terminal());
        assert_eq!(e.execution_state, ExecutionState::Running);
    }

    // -------- 7. INV-DX-09:worktree_ids 空拒绝 --------

    #[tokio::test]
    async fn invariant_09_empty_worktree_ids() {
        let svc = InMemoryDevelopmentService::new_for_test();
        let tenant_id = TenantId::new();
        let actor = make_test_actor(tenant_id);
        let cmd = CreateExecutionCommand {
            tenant_id,
            project_id: ProjectId::new(),
            work_item_id: WorkItemId::new(),
            repository_id: RepositoryId::new(),
            worktree_ids: vec![], // 空
        };
        let res = svc.create_execution(cmd, actor).await;
        assert!(matches!(res, Err(DevelopmentError::InvalidState(_))));
    }

    // -------- 8. append_change_set 成功路径 + INV-DX-01~05 全检 --------

    #[tokio::test]
    async fn append_change_set_success() {
        let svc = InMemoryDevelopmentService::new_for_test();
        let tenant_id = TenantId::new();
        let actor = make_test_actor(tenant_id);
        let exec_id = svc
            .create_execution(
                CreateExecutionCommand {
                    tenant_id,
                    project_id: ProjectId::new(),
                    work_item_id: WorkItemId::new(),
                    repository_id: RepositoryId::new(),
                    worktree_ids: vec![WorktreeId::new()],
                },
                actor.clone(),
            )
            .await
            .unwrap();

        let cmd = AppendChangeSetCommand {
            tenant_id,
            project_id: ProjectId::new(),
            execution_id: exec_id,
            worktree_id: WorktreeId::new(),
            agent_session_id: None,
            commit_id: CommitId::new(),
            diff_reference: make_diff_ref(tenant_id),
            files: vec![FileChangeDraft {
                path: FilePath::new("src/lib.rs").unwrap(),
                old_path: None,
                status: FileChangeStatus::Modified,
                lines_added: 10,
                lines_deleted: 3,
                before_content_hash: Some("h1".to_string()),
                after_content_hash: Some("h2".to_string()),
            }],
            risk_signals: vec![RiskSignalDraft {
                kind: RiskSignalKind::LargeChange,
                severity: RiskSeverity::Medium,
                source: RiskSource::StaticAnalysis,
                evidence: "lines_added=10".to_string(),
                suggested_action: "split pr".to_string(),
            }],
            dependency_changes: vec![],
            schema_changes: vec![],
            config_changes: vec![],
            test_changes: vec![],
        };
        let cs_id = svc
            .append_change_set(cmd, actor)
            .await
            .expect("append 成功");
        assert_eq!(svc.change_set_count().await, 1);
        let viewer = make_test_actor(tenant_id);
        let cs = svc.get_change_set(cs_id, viewer).await.expect("get cs");
        assert_eq!(cs.files.len(), 1);
        assert_eq!(cs.risk_signals.len(), 1);
        assert!(!cs.is_committed);
    }

    // -------- 9. INV-DX-05:Object Storage Key 缺 tenant_id 前缀拒绝 --------

    #[tokio::test]
    async fn invariant_05_storage_key_missing_tenant_prefix() {
        let svc = InMemoryDevelopmentService::new_for_test();
        let tenant_id = TenantId::new();
        let actor = make_test_actor(tenant_id);
        let exec_id = svc
            .create_execution(
                CreateExecutionCommand {
                    tenant_id,
                    project_id: ProjectId::new(),
                    work_item_id: WorkItemId::new(),
                    repository_id: RepositoryId::new(),
                    worktree_ids: vec![WorktreeId::new()],
                },
                actor.clone(),
            )
            .await
            .unwrap();
        let cmd = AppendChangeSetCommand {
            tenant_id,
            project_id: ProjectId::new(),
            execution_id: exec_id,
            worktree_id: WorktreeId::new(),
            agent_session_id: None,
            commit_id: CommitId::new(),
            diff_reference: "wrong-prefix/xxx.diff".to_string(), // 缺 tenant_id 前缀
            files: vec![FileChangeDraft {
                path: FilePath::new("src/lib.rs").unwrap(),
                old_path: None,
                status: FileChangeStatus::Modified,
                lines_added: 1,
                lines_deleted: 0,
                before_content_hash: None,
                after_content_hash: None,
            }],
            risk_signals: vec![RiskSignalDraft {
                kind: RiskSignalKind::LargeChange,
                severity: RiskSeverity::Low,
                source: RiskSource::Heuristic,
                evidence: "test".to_string(),
                suggested_action: "n/a".to_string(),
            }],
            dependency_changes: vec![],
            schema_changes: vec![],
            config_changes: vec![],
            test_changes: vec![],
        };
        let res = svc.append_change_set(cmd, actor).await;
        assert!(matches!(res, Err(DevelopmentError::InvalidObjectStorageKey(_))));
    }

    // -------- 10. INV-DX-07:AISelfClaim 缺 validation_passed_id 拒绝(D-005) --------

    #[tokio::test]
    async fn invariant_07_ai_self_claim_requires_validation() {
        let svc = InMemoryDevelopmentService::new_for_test();
        let tenant_id = TenantId::new();
        let actor = make_test_actor(tenant_id);
        // attach_risk_signal 路径
        let exec_id = svc
            .create_execution(
                CreateExecutionCommand {
                    tenant_id,
                    project_id: ProjectId::new(),
                    work_item_id: WorkItemId::new(),
                    repository_id: RepositoryId::new(),
                    worktree_ids: vec![WorktreeId::new()],
                },
                actor.clone(),
            )
            .await
            .unwrap();
        let cs_id = svc
            .append_change_set(
                AppendChangeSetCommand {
                    tenant_id,
                    project_id: ProjectId::new(),
                    execution_id: exec_id,
                    worktree_id: WorktreeId::new(),
                    agent_session_id: None,
                    commit_id: CommitId::new(),
                    diff_reference: make_diff_ref(tenant_id),
                    files: vec![FileChangeDraft {
                        path: FilePath::new("a.rs").unwrap(),
                        old_path: None,
                        status: FileChangeStatus::Modified,
                        lines_added: 1,
                        lines_deleted: 0,
                        before_content_hash: None,
                        after_content_hash: None,
                    }],
                    risk_signals: vec![RiskSignalDraft {
                        kind: RiskSignalKind::LargeChange,
                        severity: RiskSeverity::Info,
                        source: RiskSource::Heuristic,
                        evidence: "x".to_string(),
                        suggested_action: "y".to_string(),
                    }],
                    dependency_changes: vec![],
                    schema_changes: vec![],
                    config_changes: vec![],
                    test_changes: vec![],
                },
                actor.clone(),
            )
            .await
            .unwrap();
        let res = svc
            .attach_risk_signal(
                AttachRiskSignalCommand {
                    tenant_id,
                    change_set_id: cs_id,
                    kind: RiskSignalKind::AISelfClaim,
                    severity: RiskSeverity::High,
                    source: RiskSource::AIClassifier,
                    evidence: "AI self-claimed all tests pass".to_string(),
                    suggested_action: "Run validation".to_string(),
                    validation_passed_id: None, // 缺 VAL-001
                },
                actor,
            )
            .await;
        assert!(matches!(res, Err(DevelopmentError::ValidationRequired)));
    }

    // -------- 11. INV-DX-10:已 commit 的 ChangeSet 不可再 attach risk signal --------

    #[tokio::test]
    async fn invariant_10_committed_change_set_locked() {
        let svc = InMemoryDevelopmentService::new_for_test();
        let tenant_id = TenantId::new();
        let actor = make_test_actor(tenant_id);
        let exec_id = svc
            .create_execution(
                CreateExecutionCommand {
                    tenant_id,
                    project_id: ProjectId::new(),
                    work_item_id: WorkItemId::new(),
                    repository_id: RepositoryId::new(),
                    worktree_ids: vec![WorktreeId::new()],
                },
                actor.clone(),
            )
            .await
            .unwrap();
        let cs_id = svc
            .append_change_set(
                AppendChangeSetCommand {
                    tenant_id,
                    project_id: ProjectId::new(),
                    execution_id: exec_id,
                    worktree_id: WorktreeId::new(),
                    agent_session_id: None,
                    commit_id: CommitId::new(),
                    diff_reference: make_diff_ref(tenant_id),
                    files: vec![FileChangeDraft {
                        path: FilePath::new("a.rs").unwrap(),
                        old_path: None,
                        status: FileChangeStatus::Modified,
                        lines_added: 1,
                        lines_deleted: 0,
                        before_content_hash: None,
                        after_content_hash: None,
                    }],
                    risk_signals: vec![RiskSignalDraft {
                        kind: RiskSignalKind::LargeChange,
                        severity: RiskSeverity::Info,
                        source: RiskSource::Heuristic,
                        evidence: "x".to_string(),
                        suggested_action: "y".to_string(),
                    }],
                    dependency_changes: vec![],
                    schema_changes: vec![],
                    config_changes: vec![],
                    test_changes: vec![],
                },
                actor.clone(),
            )
            .await
            .unwrap();
        // 标记为 committed
        svc.mark_change_set_committed(cs_id).await.unwrap();
        // 再次 attach 应被拒
        let res = svc
            .attach_risk_signal(
                AttachRiskSignalCommand {
                    tenant_id,
                    change_set_id: cs_id,
                    kind: RiskSignalKind::ConflictRisk,
                    severity: RiskSeverity::Medium,
                    source: RiskSource::Heuristic,
                    evidence: "x".to_string(),
                    suggested_action: "y".to_string(),
                    validation_passed_id: None,
                },
                actor,
            )
            .await;
        assert!(matches!(res, Err(DevelopmentError::Conflict(_))));
    }

    // -------- 12. close_execution 成功路径 + 状态机终态 --------

    #[tokio::test]
    async fn close_execution_success() {
        let svc = InMemoryDevelopmentService::new_for_test();
        let tenant_id = TenantId::new();
        let actor = make_test_actor(tenant_id);
        let exec_id = svc
            .create_execution(
                CreateExecutionCommand {
                    tenant_id,
                    project_id: ProjectId::new(),
                    work_item_id: WorkItemId::new(),
                    repository_id: RepositoryId::new(),
                    worktree_ids: vec![WorktreeId::new()],
                },
                actor.clone(),
            )
            .await
            .unwrap();
        let e = svc
            .close_execution(
                CloseExecutionCommand {
                    tenant_id,
                    execution_id: exec_id,
                    terminal_state: ExecutionState::Succeeded,
                    ended_at: None,
                },
                actor,
            )
            .await
            .expect("close 成功");
        assert!(e.is_terminal());
        assert_eq!(e.execution_state, ExecutionState::Succeeded);
        assert!(e.ended_at.is_some());
    }

    // -------- 13. 跨租户访问被拒 --------

    #[tokio::test]
    async fn cross_tenant_access_denied() {
        let svc = InMemoryDevelopmentService::new_for_test();
        let tenant_a = TenantId::new();
        let actor_a = make_test_actor(tenant_a);
        let exec_id = svc
            .create_execution(
                CreateExecutionCommand {
                    tenant_id: tenant_a,
                    project_id: ProjectId::new(),
                    work_item_id: WorkItemId::new(),
                    repository_id: RepositoryId::new(),
                    worktree_ids: vec![WorktreeId::new()],
                },
                actor_a,
            )
            .await
            .unwrap();
        // Tenant B 访问
        let tenant_b = TenantId::new();
        let actor_b = make_test_actor(tenant_b);
        let res = svc.get_execution(exec_id, actor_b).await;
        assert!(matches!(res, Err(DevelopmentError::PermissionDenied)));
    }

    // -------- 14. 事件总线烟囱测试 --------

    #[tokio::test]
    async fn event_bus_receives_created_and_observed() {
        let (svc, mut rx) = InMemoryDevelopmentService::new();
        let tenant_id = TenantId::new();
        let actor = make_test_actor(tenant_id);
        let exec_id = svc
            .create_execution(
                CreateExecutionCommand {
                    tenant_id,
                    project_id: ProjectId::new(),
                    work_item_id: WorkItemId::new(),
                    repository_id: RepositoryId::new(),
                    worktree_ids: vec![WorktreeId::new()],
                },
                actor.clone(),
            )
            .await
            .unwrap();
        let _ = svc
            .append_change_set(
                AppendChangeSetCommand {
                    tenant_id,
                    project_id: ProjectId::new(),
                    execution_id: exec_id,
                    worktree_id: WorktreeId::new(),
                    agent_session_id: None,
                    commit_id: CommitId::new(),
                    diff_reference: make_diff_ref(tenant_id),
                    files: vec![FileChangeDraft {
                        path: FilePath::new("a.rs").unwrap(),
                        old_path: None,
                        status: FileChangeStatus::Modified,
                        lines_added: 1,
                        lines_deleted: 0,
                        before_content_hash: None,
                        after_content_hash: None,
                    }],
                    risk_signals: vec![RiskSignalDraft {
                        kind: RiskSignalKind::LargeChange,
                        severity: RiskSeverity::High, // High 触发 RiskSignalDetected
                        source: RiskSource::StaticAnalysis,
                        evidence: "x".to_string(),
                        suggested_action: "y".to_string(),
                    }],
                    dependency_changes: vec![],
                    schema_changes: vec![],
                    config_changes: vec![],
                    test_changes: vec![],
                },
                actor,
            )
            .await
            .unwrap();
        // 检查至少收到 ExecutionCreated + ChangeSetObserved + RiskSignalDetected
        let mut got_created = false;
        let mut got_observed = false;
        let mut got_risk = false;
        for _ in 0..20 {
            if let Ok(evt) = rx.try_recv() {
                match evt {
                    DevelopmentEvent::ExecutionCreated(_) => got_created = true,
                    DevelopmentEvent::ChangeSetObserved(_) => got_observed = true,
                    DevelopmentEvent::RiskSignalDetected(_) => got_risk = true,
                    _ => {}
                }
            }
            if got_created && got_observed && got_risk {
                break;
            }
        }
        assert!(got_created, "应收到 ExecutionCreated 事件");
        assert!(got_observed, "应收到 ChangeSetObserved 事件");
        assert!(got_risk, "应收到 RiskSignalDetected 事件(severity=High)");
    }

    // -------- 15. build_symbol_index + 跨租户查询拒绝(D-006) --------

    #[tokio::test]
    async fn build_symbol_index_and_cross_tenant_query() {
        let svc = InMemoryDevelopmentService::new_for_test();
        let tenant_id = TenantId::new();
        let actor = make_test_actor(tenant_id);
        let repo_id = RepositoryId::new();
        let idx = svc
            .build_symbol_index(
                BuildSymbolIndexCommand {
                    tenant_id,
                    repository_id: repo_id,
                    symbol_seeds: vec![SymbolSeed {
                        symbol_ref: "crate::foo::bar".to_string(),
                        kind: SymbolKind::Function,
                        signature: Some("fn bar()".to_string()),
                        file_path: FilePath::new("src/foo.rs").unwrap(),
                        line_range: LineRange::new(10, 20),
                    }],
                },
                actor.clone(),
            )
            .await
            .expect("build 成功");
        assert_eq!(idx.symbols.len(), 1);
        assert_eq!(idx.version, 1);

        // 同租户查询
        let viewer = make_test_actor(tenant_id);
        let q = ListSymbolQuery {
            tenant_id,
            repository_id: repo_id,
            name_prefix: Some("crate::foo".to_string()),
            limit: 10,
            offset: 0,
        };
        let syms = svc.list_symbols(q, viewer.clone()).await.expect("list");
        assert_eq!(syms.len(), 1);

        // 跨租户查询:D-006
        let tenant_b = TenantId::new();
        let actor_b = make_test_actor(tenant_b);
        let res = svc.get_symbol_index(repo_id, actor_b).await;
        assert!(matches!(
            res,
            Err(DevelopmentError::CrossTenantRepositoryAccess { .. })
        ));
    }

    // -------- 16. search_symbol 模糊匹配 --------

    #[tokio::test]
    async fn search_symbol_keyword() {
        let svc = InMemoryDevelopmentService::new_for_test();
        let tenant_id = TenantId::new();
        let actor = make_test_actor(tenant_id);
        let repo_id = RepositoryId::new();
        let _ = svc
            .build_symbol_index(
                BuildSymbolIndexCommand {
                    tenant_id,
                    repository_id: repo_id,
                    symbol_seeds: vec![
                        SymbolSeed {
                            symbol_ref: "crate::foo::bar".to_string(),
                            kind: SymbolKind::Function,
                            signature: None,
                            file_path: FilePath::new("src/foo.rs").unwrap(),
                            line_range: LineRange::new(1, 5),
                        },
                        SymbolSeed {
                            symbol_ref: "crate::foo::baz".to_string(),
                            kind: SymbolKind::Function,
                            signature: None,
                            file_path: FilePath::new("src/foo.rs").unwrap(),
                            line_range: LineRange::new(6, 10),
                        },
                        SymbolSeed {
                            symbol_ref: "crate::other::qux".to_string(),
                            kind: SymbolKind::Function,
                            signature: None,
                            file_path: FilePath::new("src/other.rs").unwrap(),
                            line_range: LineRange::new(1, 5),
                        },
                    ],
                },
                actor,
            )
            .await
            .unwrap();
        let viewer = make_test_actor(tenant_id);
        let res = svc
            .search_symbol(
                SearchSymbolQuery {
                    tenant_id,
                    repository_id: repo_id,
                    keyword: "foo".to_string(),
                    limit: 10,
                },
                viewer,
            )
            .await
            .expect("search");
        assert_eq!(res.len(), 2);
    }

    // -------- 17. bump_version + close 终态正确性 --------

    #[test]
    fn execution_bump_version_and_close() {
        let mut e = DevelopmentExecution {
            id: ExecutionId::new(),
            tenant_id: TenantId::new(),
            project_id: ProjectId::new(),
            work_item_id: WorkItemId::new(),
            repository_id: RepositoryId::new(),
            worktree_ids: vec![WorktreeId::new()],
            agent_session_ids: vec![],
            change_set_ids: vec![],
            validation_result_ids: vec![],
            feedback_ids: vec![],
            commit_ids: vec![],
            pull_request_ids: vec![],
            started_at: chrono::Utc::now(),
            ended_at: None,
            execution_state: ExecutionState::Running,
            lock_version: 1,
            created_by_user_id: UserId::new(),
        };
        e.bump_version();
        assert_eq!(e.lock_version, 2);
        e.close(ExecutionState::Succeeded, chrono::Utc::now());
        assert_eq!(e.execution_state, ExecutionState::Succeeded);
        assert!(e.ended_at.is_some());
        assert!(e.is_terminal());
    }
}
