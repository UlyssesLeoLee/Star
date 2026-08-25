//! InMemoryDevelopmentService:Phase 2 提供的内存实现
//!
//! 来源: spec/domain-development-spec.md §5(实施策略)
//!
//! **目标**:为 `DevelopmentCommandPort` + `DevelopmentQueryPort` + `DevelopmentRepository`
//! 提供真实可工作的实现,用于本地集成测试与 P0 演示,
//! 不依赖任何数据库 / NATS 外部基础设施。
//!
//! **Phase 3 计划**:`crates/infrastructure` 提供 SQLx / NATS Adapter 取代本实现。

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;

use crate::context::ActorContext;
use crate::entity::{
    ChangeSet, DevelopmentContext, DevelopmentExecution, IndexedSymbol, RepositoryContext,
    RiskSignal, SymbolIndex,
};
use crate::error::DevelopmentError;
use crate::event::{
    ChangeSetObserved, DevelopmentEvent, EventMeta, ExecutionClosed, ExecutionCreated,
    RiskSignalDetected, SymbolIndexRefreshed,
};
use crate::invariants::{
    check_append_change_set_invariants, check_invariant_07_ai_self_claim_validation,
    check_invariant_09_at_least_one_worktree, check_invariant_10_change_set_not_committed,
    check_terminal_state,
};
use crate::port::{
    AppendChangeSetCommand, AttachRiskSignalCommand, BuildSymbolIndexCommand,
    CloseExecutionCommand, CreateExecutionCommand, DevelopmentCommandPort, DevelopmentQueryPort,
    DevelopmentRepository, DiffDownloadURL, ListChangeSetQuery, ListExecutionQuery,
    ListSymbolQuery, SearchSymbolQuery,
};
use crate::value_object::{
    is_valid_state_transition, ChangeSetId, ExecutionId, ExecutionState, FilePath, LineRange,
    RepositoryId, RiskSeverity, RiskSignalId, SymbolId, SymbolKind, TenantId,
};

// =====================================================================
// InMemoryDevelopmentService
// =====================================================================

/// **InMemory Development 命令/查询/仓库服务**(Phase 2 真实实现)
///
/// 内部使用 `Arc<RwLock<HashMap>>` 模拟仓储;事件通过 `mpsc::UnboundedSender` 发送。
pub struct InMemoryDevelopmentService {
    /// Execution 存储
    executions: Arc<RwLock<HashMap<ExecutionId, DevelopmentExecution>>>,
    /// ChangeSet 存储
    change_sets: Arc<RwLock<HashMap<ChangeSetId, ChangeSet>>>,
    /// SymbolIndex 存储(repository_id → index)
    symbol_indexes: Arc<RwLock<HashMap<RepositoryId, SymbolIndex>>>,
    /// RepositoryContext 存储
    repository_contexts: Arc<RwLock<HashMap<RepositoryId, RepositoryContext>>>,
    /// DevelopmentContext 存储(execution_id → context)
    development_contexts: Arc<RwLock<HashMap<ExecutionId, DevelopmentContext>>>,
    /// 事件发送器
    event_tx: mpsc::UnboundedSender<DevelopmentEvent>,
}

impl InMemoryDevelopmentService {
    /// 创建新的内存服务(返回服务和事件接收端)。
    pub fn new() -> (Arc<Self>, mpsc::UnboundedReceiver<DevelopmentEvent>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let svc = Arc::new(Self {
            executions: Arc::new(RwLock::new(HashMap::new())),
            change_sets: Arc::new(RwLock::new(HashMap::new())),
            symbol_indexes: Arc::new(RwLock::new(HashMap::new())),
            repository_contexts: Arc::new(RwLock::new(HashMap::new())),
            development_contexts: Arc::new(RwLock::new(HashMap::new())),
            event_tx: tx,
        });
        (svc, rx)
    }

    /// 仅创建服务(事件接收端丢弃,适合 fire-and-forget 测试)。
    pub fn new_for_test() -> Arc<Self> {
        Self::new().0
    }

    /// 当前 Execution 数量
    pub async fn execution_count(&self) -> usize {
        self.executions.read().expect("executions lock").len()
    }

    /// 当前 ChangeSet 数量
    pub async fn change_set_count(&self) -> usize {
        self.change_sets.read().expect("change_sets lock").len()
    }

    /// 校验 actor 与命令的 tenant_id 一致
    fn check_tenant(actor: &ActorContext, expected: TenantId) -> Result<(), DevelopmentError> {
        if actor.tenant_id != expected {
            return Err(DevelopmentError::PermissionDenied);
        }
        Ok(())
    }
}

impl Default for InMemoryDevelopmentService {
    fn default() -> Self {
        Self::new().0.as_ref().clone()
    }
}

// 手工 Clone(因为内部字段是 Arc,Clone 便宜)
impl Clone for InMemoryDevelopmentService {
    fn clone(&self) -> Self {
        Self {
            executions: self.executions.clone(),
            change_sets: self.change_sets.clone(),
            symbol_indexes: self.symbol_indexes.clone(),
            repository_contexts: self.repository_contexts.clone(),
            development_contexts: self.development_contexts.clone(),
            event_tx: self.event_tx.clone(),
        }
    }
}

// =====================================================================
// DevelopmentCommandPort 实现(5 方法)
// =====================================================================

#[async_trait]
impl DevelopmentCommandPort for InMemoryDevelopmentService {
    async fn create_execution(
        &self,
        cmd: CreateExecutionCommand,
        actor: ActorContext,
    ) -> Result<ExecutionId, DevelopmentError> {
        // 1. 租户校验
        Self::check_tenant(&actor, cmd.tenant_id)?;

        // 2. INV-DX-09:worktree_ids 1..N
        check_invariant_09_at_least_one_worktree(&cmd.worktree_ids)?;

        // 3. 构造 Execution
        let now = chrono::Utc::now();
        let exec = DevelopmentExecution {
            id: ExecutionId::new(),
            tenant_id: cmd.tenant_id,
            project_id: cmd.project_id,
            work_item_id: cmd.work_item_id,
            repository_id: cmd.repository_id,
            worktree_ids: cmd.worktree_ids,
            agent_session_ids: Vec::new(),
            change_set_ids: Vec::new(),
            validation_result_ids: Vec::new(),
            feedback_ids: Vec::new(),
            commit_ids: Vec::new(),
            pull_request_ids: Vec::new(),
            started_at: now,
            ended_at: None,
            execution_state: ExecutionState::Running,
            lock_version: 1,
            created_by_user_id: actor.user_id,
        };
        let id = exec.id;

        // 4. 持久化
        self.executions
            .write()
            .expect("executions lock")
            .insert(id, exec);

        // 5. 事件
        let event = DevelopmentEvent::ExecutionCreated(ExecutionCreated {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id.into_uuid()),
                ..EventMeta::new(cmd.tenant_id)
            },
            execution_id: id,
            work_item_id: cmd.work_item_id.into_uuid(),
            repository_id: cmd.repository_id,
        });
        let _ = self.event_tx.send(event);

        Ok(id)
    }

    async fn append_change_set(
        &self,
        cmd: AppendChangeSetCommand,
        actor: ActorContext,
    ) -> Result<ChangeSetId, DevelopmentError> {
        // 1. 租户校验
        Self::check_tenant(&actor, cmd.tenant_id)?;

        // 2. 验证 Execution 存在 & 同租户
        let exec = {
            let store = self.executions.read().expect("executions lock");
            store
                .get(&cmd.execution_id)
                .cloned()
                .ok_or(DevelopmentError::ExecutionNotFound(cmd.execution_id))?
        };
        if exec.tenant_id != cmd.tenant_id {
            return Err(DevelopmentError::PermissionDenied);
        }
        if exec.is_terminal() {
            return Err(DevelopmentError::Conflict(format!(
                "Execution {} 已进入终态({}),不可追加 ChangeSet",
                exec.id, exec.execution_state
            )));
        }

        // 3. 构造 ChangeSet
        let now = chrono::Utc::now();
        let files: Vec<crate::entity::FileChange> = cmd
            .files
            .iter()
            .map(|f| crate::entity::FileChange {
                id: uuid::Uuid::new_v4(),
                path: f.path.clone(),
                old_path: f.old_path.clone(),
                status: f.status,
                lines_added: f.lines_added,
                lines_deleted: f.lines_deleted,
                before_content_hash: f.before_content_hash.clone(),
                after_content_hash: f.after_content_hash.clone(),
            })
            .collect();

        let added_lines: u32 = files.iter().map(|f| f.lines_added).sum();
        let deleted_lines: u32 = files.iter().map(|f| f.lines_deleted).sum();
        let renamed_files: u32 = files
            .iter()
            .filter(|f| matches!(f.status, crate::value_object::FileChangeStatus::Renamed))
            .count() as u32;
        let generated_files: u32 = files
            .iter()
            .filter(|f| matches!(f.status, crate::value_object::FileChangeStatus::Generated))
            .count() as u32;

        let risk_signals: Vec<RiskSignal> = cmd
            .risk_signals
            .iter()
            .map(|r| {
                // INV-DX-07
                check_invariant_07_ai_self_claim_validation(r.kind, None)?;
                Ok::<_, DevelopmentError>(RiskSignal {
                    id: RiskSignalId::new(),
                    change_set_id: ChangeSetId::nil(), // 稍后覆盖
                    tenant_id: cmd.tenant_id,
                    kind: r.kind,
                    severity: r.severity,
                    source: r.source,
                    evidence: r.evidence.clone(),
                    suggested_action: r.suggested_action.clone(),
                    created_at: now,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        let change_set_id = ChangeSetId::new();
        let mut change_set = ChangeSet {
            id: change_set_id,
            tenant_id: cmd.tenant_id,
            project_id: cmd.project_id,
            worktree_id: cmd.worktree_id,
            agent_session_id: cmd.agent_session_id,
            commit_id: cmd.commit_id,
            files,
            symbols: Vec::new(),
            diff_reference: cmd.diff_reference.clone(),
            added_lines,
            deleted_lines,
            renamed_files,
            generated_files,
            risk_signals,
            dependency_changes: cmd.dependency_changes,
            schema_changes: cmd.schema_changes,
            config_changes: cmd.config_changes,
            test_changes: cmd.test_changes,
            created_at: now,
            lock_version: 1,
            is_committed: false,
        };
        // 回填 change_set_id 到 risk_signals
        for r in &mut change_set.risk_signals {
            r.change_set_id = change_set_id;
        }

        // 4. 批量不变量检查(INV-DX-01/02/03/04/05)
        check_append_change_set_invariants(&change_set)?;

        // 5. 持久化
        self.change_sets
            .write()
            .expect("change_sets lock")
            .insert(change_set_id, change_set.clone());

        // 6. 反向更新 Execution
        {
            let mut store = self.executions.write().expect("executions lock");
            if let Some(e) = store.get_mut(&cmd.execution_id) {
                e.change_set_ids.push(change_set_id);
                if !cmd.commit_id.as_uuid().is_nil() {
                    e.commit_ids.push(cmd.commit_id);
                }
                e.bump_version();
            }
        }

        // 7. 事件:ChangeSetObserved
        let high_count = change_set.high_severity_signal_count();
        let event = DevelopmentEvent::ChangeSetObserved(ChangeSetObserved {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id.into_uuid()),
                ..EventMeta::new(cmd.tenant_id)
            },
            change_set_id,
            worktree_id: cmd.worktree_id,
            agent_session_id: cmd.agent_session_id,
            commit_id: cmd.commit_id,
            risk_signal_count: high_count as u32,
        });
        let _ = self.event_tx.send(event);

        // 8. 事件:RiskSignalDetected(severity >= High)
        for r in &change_set.risk_signals {
            if r.severity >= RiskSeverity::High {
                let evt = DevelopmentEvent::RiskSignalDetected(RiskSignalDetected {
                    meta: EventMeta {
                        actor_user_id: Some(actor.user_id.into_uuid()),
                        ..EventMeta::new(cmd.tenant_id)
                    },
                    change_set_id,
                    kind: r.kind,
                    severity: r.severity,
                    evidence: r.evidence.clone(),
                });
                let _ = self.event_tx.send(evt);
            }
        }

        Ok(change_set_id)
    }

    async fn attach_risk_signal(
        &self,
        cmd: AttachRiskSignalCommand,
        actor: ActorContext,
    ) -> Result<RiskSignal, DevelopmentError> {
        // 1. 租户校验
        Self::check_tenant(&actor, cmd.tenant_id)?;

        // 2. 验证 ChangeSet 存在 & 同租户
        let mut cs = {
            let store = self.change_sets.read().expect("change_sets lock");
            store
                .get(&cmd.change_set_id)
                .cloned()
                .ok_or(DevelopmentError::ChangeSetNotFound(cmd.change_set_id))?
        };
        if cs.tenant_id != cmd.tenant_id {
            return Err(DevelopmentError::PermissionDenied);
        }

        // 3. INV-DX-10
        check_invariant_10_change_set_not_committed(&cs)?;

        // 4. INV-DX-04 类型合法
        if !cmd.kind.is_known() {
            return Err(DevelopmentError::InvalidRiskSignalKind(cmd.kind.to_string()));
        }

        // 5. INV-DX-07:AISelfClaim 必填 validation_passed_id
        check_invariant_07_ai_self_claim_validation(cmd.kind, cmd.validation_passed_id)?;

        // 6. 构造
        let signal = RiskSignal {
            id: RiskSignalId::new(),
            change_set_id: cmd.change_set_id,
            tenant_id: cmd.tenant_id,
            kind: cmd.kind,
            severity: cmd.severity,
            source: cmd.source,
            evidence: cmd.evidence.clone(),
            suggested_action: cmd.suggested_action.clone(),
            created_at: chrono::Utc::now(),
        };

        // 7. 持久化
        cs.risk_signals.push(signal.clone());
        cs.bump_version();
        self.change_sets
            .write()
            .expect("change_sets lock")
            .insert(cmd.change_set_id, cs);

        // 8. 事件(severity >= High)
        if signal.severity >= RiskSeverity::High {
            let evt = DevelopmentEvent::RiskSignalDetected(RiskSignalDetected {
                meta: EventMeta {
                    actor_user_id: Some(actor.user_id.into_uuid()),
                    ..EventMeta::new(cmd.tenant_id)
                },
                change_set_id: cmd.change_set_id,
                kind: signal.kind,
                severity: signal.severity,
                evidence: signal.evidence.clone(),
            });
            let _ = self.event_tx.send(evt);
        }

        Ok(signal)
    }

    async fn close_execution(
        &self,
        cmd: CloseExecutionCommand,
        actor: ActorContext,
    ) -> Result<DevelopmentExecution, DevelopmentError> {
        // 1. 租户校验
        Self::check_tenant(&actor, cmd.tenant_id)?;

        // 2. 终态校验
        check_terminal_state(cmd.terminal_state)?;

        // 3. 取 Execution
        let mut exec = {
            let store = self.executions.read().expect("executions lock");
            store
                .get(&cmd.execution_id)
                .cloned()
                .ok_or(DevelopmentError::ExecutionNotFound(cmd.execution_id))?
        };
        if exec.tenant_id != cmd.tenant_id {
            return Err(DevelopmentError::PermissionDenied);
        }
        if exec.is_terminal() {
            return Err(DevelopmentError::Conflict(format!(
                "Execution {} 已处于终态({})",
                exec.id, exec.execution_state
            )));
        }

        // 4. 状态机合法迁移
        if !is_valid_state_transition(exec.execution_state, cmd.terminal_state) {
            return Err(DevelopmentError::InvalidState(format!(
                "Execution 状态机非法迁移: {} → {}",
                exec.execution_state, cmd.terminal_state
            )));
        }

        // 5. 关闭
        let ended_at = cmd.ended_at.unwrap_or_else(chrono::Utc::now);
        let change_set_count = exec.change_set_ids.len() as u32;
        exec.close(cmd.terminal_state, ended_at);

        // 6. 持久化
        self.executions
            .write()
            .expect("executions lock")
            .insert(cmd.execution_id, exec.clone());

        // 7. 事件
        let event = DevelopmentEvent::ExecutionClosed(ExecutionClosed {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id.into_uuid()),
                ..EventMeta::new(cmd.tenant_id)
            },
            execution_id: cmd.execution_id,
            ended_at,
            change_set_count,
        });
        let _ = self.event_tx.send(event);

        Ok(exec)
    }

    async fn build_symbol_index(
        &self,
        cmd: BuildSymbolIndexCommand,
        actor: ActorContext,
    ) -> Result<SymbolIndex, DevelopmentError> {
        // 1. 租户校验
        Self::check_tenant(&actor, cmd.tenant_id)?;

        // 2. 构造 IndexedSymbol 列表
        let now = chrono::Utc::now();
        let symbols: Vec<IndexedSymbol> = cmd
            .symbol_seeds
            .into_iter()
            .map(|s| IndexedSymbol {
                id: SymbolId::new(),
                symbol_ref: s.symbol_ref,
                kind: s.kind,
                signature: s.signature,
                file_path: s.file_path,
                line_range: s.line_range,
            })
            .collect();

        // 3. INV-DX-08:每个 symbol 必含 file_path
        for s in &symbols {
            crate::invariants::check_invariant_08_file_level_symbols(s)?;
        }

        // 4. 取得或创建 SymbolIndex(INV-DX-06:跨 Repository 不合并)
        let mut store = self.symbol_indexes.write().expect("symbol_indexes lock");
        let symbol_count = symbols.len() as u32;
        let new_index = match store.remove(&cmd.repository_id) {
            Some(mut existing) => {
                existing.symbols = symbols;
                existing.bump_version();
                existing
            }
            None => SymbolIndex {
                id: uuid::Uuid::new_v4(),
                tenant_id: cmd.tenant_id,
                repository_id: cmd.repository_id,
                symbols,
                last_refresh_at: now,
                version: 1,
            },
        };
        let version = new_index.version;
        store.insert(cmd.repository_id, new_index.clone());

        // 5. 事件
        let event = DevelopmentEvent::SymbolIndexRefreshed(SymbolIndexRefreshed {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id.into_uuid()),
                ..EventMeta::new(cmd.tenant_id)
            },
            repository_id: cmd.repository_id,
            version,
            symbol_count,
        });
        let _ = self.event_tx.send(event);

        Ok(new_index)
    }
}

// =====================================================================
// DevelopmentQueryPort 实现(7+ 方法)
// =====================================================================

#[async_trait]
impl DevelopmentQueryPort for InMemoryDevelopmentService {
    async fn get_execution(
        &self,
        id: ExecutionId,
        viewer: ActorContext,
    ) -> Result<DevelopmentExecution, DevelopmentError> {
        let store = self.executions.read().expect("executions lock");
        let e = store
            .get(&id)
            .cloned()
            .ok_or(DevelopmentError::ExecutionNotFound(id))?;
        if e.tenant_id != viewer.tenant_id {
            return Err(DevelopmentError::PermissionDenied);
        }
        Ok(e)
    }

    async fn list_executions(
        &self,
        q: ListExecutionQuery,
        viewer: ActorContext,
    ) -> Result<Vec<DevelopmentExecution>, DevelopmentError> {
        Self::check_tenant(&viewer, q.tenant_id)?;
        let store = self.executions.read().expect("executions lock");
        let mut out: Vec<DevelopmentExecution> = store
            .values()
            .filter(|e| e.tenant_id == q.tenant_id)
            .filter(|e| {
                if let Some(wt) = q.worktree_id {
                    e.worktree_ids.contains(&wt)
                } else {
                    true
                }
            })
            .filter(|e| {
                if let Some(wi) = q.work_item_id {
                    e.work_item_id == wi
                } else {
                    true
                }
            })
            .cloned()
            .collect();
        out.sort_by_key(|a| std::cmp::Reverse(a.started_at));
        let offset = q.offset as usize;
        let limit = q.limit.max(1) as usize;
        let end = (offset + limit).min(out.len());
        let start = offset.min(out.len());
        Ok(out[start..end].to_vec())
    }

    async fn get_change_set(
        &self,
        id: ChangeSetId,
        viewer: ActorContext,
    ) -> Result<ChangeSet, DevelopmentError> {
        let store = self.change_sets.read().expect("change_sets lock");
        let c = store
            .get(&id)
            .cloned()
            .ok_or(DevelopmentError::ChangeSetNotFound(id))?;
        if c.tenant_id != viewer.tenant_id {
            return Err(DevelopmentError::PermissionDenied);
        }
        Ok(c)
    }

    async fn list_change_sets(
        &self,
        q: ListChangeSetQuery,
        viewer: ActorContext,
    ) -> Result<Vec<ChangeSet>, DevelopmentError> {
        Self::check_tenant(&viewer, q.tenant_id)?;
        let store = self.change_sets.read().expect("change_sets lock");
        let mut out: Vec<ChangeSet> = store
            .values()
            .filter(|c| c.tenant_id == q.tenant_id)
            .filter(|c| {
                if let Some(eid) = q.execution_id {
                    // 通过 Execution 反查需要双表 join,这里用反向方式:遍历 executions 拿到 change_set_ids
                    // 简单起见:额外读取 executions
                    let exec_store = self.executions.read().expect("executions lock");
                    exec_store
                        .get(&eid)
                        .map(|e| e.change_set_ids.contains(&c.id))
                        .unwrap_or(false)
                } else {
                    true
                }
            })
            .filter(|c| {
                if let Some(wt) = q.worktree_id {
                    c.worktree_id == wt
                } else {
                    true
                }
            })
            .cloned()
            .collect();
        out.sort_by_key(|a| std::cmp::Reverse(a.created_at));
        let offset = q.offset as usize;
        let limit = q.limit.max(1) as usize;
        let end = (offset + limit).min(out.len());
        let start = offset.min(out.len());
        Ok(out[start..end].to_vec())
    }

    async fn get_diff_url(
        &self,
        id: ChangeSetId,
        viewer: ActorContext,
    ) -> Result<DiffDownloadURL, DevelopmentError> {
        let cs = self.get_change_set(id, viewer.clone()).await?;
        // 模拟短期预签名 URL(Phase 2 内存实现,实际由 infrastructure 颁发)
        let url = format!(
            "https://object-storage.local/{}/{}?expired_in=300s",
            cs.diff_reference, id
        );
        let expires_at = chrono::Utc::now() + chrono::Duration::seconds(300);
        Ok(DiffDownloadURL {
            change_set_id: id,
            url,
            expires_at,
        })
    }

    async fn get_symbol_index(
        &self,
        repository_id: RepositoryId,
        viewer: ActorContext,
    ) -> Result<SymbolIndex, DevelopmentError> {
        let store = self.symbol_indexes.read().expect("symbol_indexes lock");
        match store.get(&repository_id) {
            Some(idx) => {
                // INV-D-006:跨租户拒绝
                if idx.tenant_id != viewer.tenant_id {
                    return Err(DevelopmentError::CrossTenantRepositoryAccess {
                        repository_id: repository_id.into_uuid(),
                        tenant_id: viewer.tenant_id.into_uuid(),
                    });
                }
                Ok(idx.clone())
            }
            None => Ok(SymbolIndex {
                id: uuid::Uuid::new_v4(),
                tenant_id: viewer.tenant_id,
                repository_id,
                symbols: Vec::new(),
                last_refresh_at: chrono::Utc::now(),
                version: 0,
            }),
        }
    }

    async fn list_symbols(
        &self,
        q: ListSymbolQuery,
        viewer: ActorContext,
    ) -> Result<Vec<IndexedSymbol>, DevelopmentError> {
        Self::check_tenant(&viewer, q.tenant_id)?;
        let idx = self
            .get_symbol_index(q.repository_id, viewer.clone())
            .await?;
        let out: Vec<IndexedSymbol> = idx
            .symbols
            .into_iter()
            .filter(|s| {
                if let Some(prefix) = &q.name_prefix {
                    s.symbol_ref.starts_with(prefix)
                } else {
                    true
                }
            })
            .skip(q.offset as usize)
            .take(q.limit.max(1) as usize)
            .collect();
        Ok(out)
    }

    async fn search_symbol(
        &self,
        q: SearchSymbolQuery,
        viewer: ActorContext,
    ) -> Result<Vec<IndexedSymbol>, DevelopmentError> {
        Self::check_tenant(&viewer, q.tenant_id)?;
        let idx = self
            .get_symbol_index(q.repository_id, viewer.clone())
            .await?;
        let needle = q.keyword.to_lowercase();
        let out: Vec<IndexedSymbol> = idx
            .symbols
            .into_iter()
            .filter(|s| s.symbol_ref.to_lowercase().contains(&needle))
            .take(q.limit.max(1) as usize)
            .collect();
        Ok(out)
    }

    async fn get_repository_context(
        &self,
        repository_id: RepositoryId,
        viewer: ActorContext,
    ) -> Result<RepositoryContext, DevelopmentError> {
        let store = self.repository_contexts.read().expect("repository_contexts lock");
        match store.get(&repository_id) {
            Some(rc) => {
                if rc.tenant_id != viewer.tenant_id {
                    return Err(DevelopmentError::CrossTenantRepositoryAccess {
                        repository_id: repository_id.into_uuid(),
                        tenant_id: viewer.tenant_id.into_uuid(),
                    });
                }
                Ok(rc.clone())
            }
            None => Ok(RepositoryContext {
                id: uuid::Uuid::new_v4(),
                tenant_id: viewer.tenant_id,
                repository_id,
                primary_language: None,
                framework: None,
                build_system: None,
                test_framework: None,
                total_files: 0,
                total_lines: 0,
                last_indexed_at: chrono::Utc::now(),
            }),
        }
    }

    async fn get_development_context(
        &self,
        execution_id: ExecutionId,
        viewer: ActorContext,
    ) -> Result<DevelopmentContext, DevelopmentError> {
        Self::check_tenant(&viewer, viewer.tenant_id)?;
        let store = self.development_contexts.read().expect("development_contexts lock");
        match store.get(&execution_id) {
            Some(dc) => {
                if dc.tenant_id != viewer.tenant_id {
                    return Err(DevelopmentError::PermissionDenied);
                }
                Ok(dc.clone())
            }
            None => {
                // 不存在时,根据 Execution 派生
                let exec_store = self.executions.read().expect("executions lock");
                let exec = exec_store
                    .get(&execution_id)
                    .cloned()
                    .ok_or(DevelopmentError::ExecutionNotFound(execution_id))?;
                if exec.tenant_id != viewer.tenant_id {
                    return Err(DevelopmentError::PermissionDenied);
                }
                Ok(DevelopmentContext {
                    id: uuid::Uuid::new_v4(),
                    tenant_id: viewer.tenant_id,
                    project_id: exec.project_id,
                    work_item_id: exec.work_item_id,
                    execution_id,
                    relevant_symbols: Vec::new(),
                    relevant_files: Vec::new(),
                    architecture_constraints: Vec::new(),
                    last_compiled_at: None,
                    version: 0,
                })
            }
        }
    }
}

// =====================================================================
// DevelopmentRepository 实现(基础设施层使用)
// =====================================================================

#[async_trait]
impl DevelopmentRepository for InMemoryDevelopmentService {
    async fn insert_execution(
        &self,
        e: &DevelopmentExecution,
    ) -> Result<(), DevelopmentError> {
        self.executions
            .write()
            .expect("executions lock")
            .insert(e.id, e.clone());
        Ok(())
    }

    async fn find_execution_by_id(
        &self,
        id: ExecutionId,
    ) -> Result<Option<DevelopmentExecution>, DevelopmentError> {
        Ok(self
            .executions
            .read()
            .expect("executions lock")
            .get(&id)
            .cloned())
    }

    async fn list_executions(
        &self,
        tenant_id: TenantId,
        worktree_id: Option<crate::value_object::WorktreeId>,
        work_item_id: Option<crate::value_object::WorkItemId>,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<DevelopmentExecution>, DevelopmentError> {
        let store = self.executions.read().expect("executions lock");
        let mut out: Vec<DevelopmentExecution> = store
            .values()
            .filter(|e| e.tenant_id == tenant_id)
            .filter(|e| match worktree_id {
                Some(wt) => e.worktree_ids.contains(&wt),
                None => true,
            })
            .filter(|e| match work_item_id {
                Some(wi) => e.work_item_id == wi,
                None => true,
            })
            .cloned()
            .collect();
        out.sort_by_key(|a| std::cmp::Reverse(a.started_at));
        let offset = offset as usize;
        let limit = limit.max(1) as usize;
        let end = (offset + limit).min(out.len());
        let start = offset.min(out.len());
        Ok(out[start..end].to_vec())
    }

    async fn update_execution(
        &self,
        e: &DevelopmentExecution,
    ) -> Result<(), DevelopmentError> {
        self.executions
            .write()
            .expect("executions lock")
            .insert(e.id, e.clone());
        Ok(())
    }

    async fn insert_change_set(&self, c: &ChangeSet) -> Result<(), DevelopmentError> {
        self.change_sets
            .write()
            .expect("change_sets lock")
            .insert(c.id, c.clone());
        Ok(())
    }

    async fn find_change_set_by_id(
        &self,
        id: ChangeSetId,
    ) -> Result<Option<ChangeSet>, DevelopmentError> {
        Ok(self
            .change_sets
            .read()
            .expect("change_sets lock")
            .get(&id)
            .cloned())
    }

    async fn list_change_sets(
        &self,
        tenant_id: TenantId,
        execution_id: Option<ExecutionId>,
        worktree_id: Option<crate::value_object::WorktreeId>,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<ChangeSet>, DevelopmentError> {
        let store = self.change_sets.read().expect("change_sets lock");
        let mut out: Vec<ChangeSet> = store
            .values()
            .filter(|c| c.tenant_id == tenant_id)
            .filter(|c| {
                if let Some(wt) = worktree_id {
                    c.worktree_id == wt
                } else {
                    true
                }
            })
            .filter(|c| {
                if let Some(eid) = execution_id {
                    let exec_store = self.executions.read().expect("executions lock");
                    exec_store
                        .get(&eid)
                        .map(|e| e.change_set_ids.contains(&c.id))
                        .unwrap_or(false)
                } else {
                    true
                }
            })
            .cloned()
            .collect();
        out.sort_by_key(|a| std::cmp::Reverse(a.created_at));
        let offset = offset as usize;
        let limit = limit.max(1) as usize;
        let end = (offset + limit).min(out.len());
        let start = offset.min(out.len());
        Ok(out[start..end].to_vec())
    }

    async fn mark_change_set_committed(
        &self,
        id: ChangeSetId,
    ) -> Result<(), DevelopmentError> {
        let mut store = self.change_sets.write().expect("change_sets lock");
        if let Some(cs) = store.get_mut(&id) {
            cs.is_committed = true;
            cs.bump_version();
            Ok(())
        } else {
            Err(DevelopmentError::ChangeSetNotFound(id))
        }
    }

    async fn find_symbol_index_by_repository(
        &self,
        repository_id: RepositoryId,
    ) -> Result<Option<SymbolIndex>, DevelopmentError> {
        Ok(self
            .symbol_indexes
            .read()
            .expect("symbol_indexes lock")
            .get(&repository_id)
            .cloned())
    }

    async fn upsert_symbol_index(
        &self,
        idx: &SymbolIndex,
    ) -> Result<(), DevelopmentError> {
        self.symbol_indexes
            .write()
            .expect("symbol_indexes lock")
            .insert(idx.repository_id, idx.clone());
        Ok(())
    }
}

// 让 change_set_id 可由 nil 创建(供 append 内部使用)
trait ChangeSetIdExt {
    fn nil() -> Self;
}
impl ChangeSetIdExt for ChangeSetId {
    fn nil() -> Self {
        Self::from_uuid(uuid::Uuid::nil())
    }
}

// 引入以避免未使用警告
#[allow(dead_code)]
const _: Option<FilePath> = None;
#[allow(dead_code)]
const _: Option<LineRange> = None;
#[allow(dead_code)]
const _: Option<SymbolKind> = None;
