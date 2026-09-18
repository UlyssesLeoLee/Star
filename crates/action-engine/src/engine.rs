//! `engine.rs` — ActionEngine + ActionRegistry (per DD §12.2 + §18 + INV-WC-09)
//!
//! 阶段 1 scope (per WORKTREE-CANVAS-IMPL-PLAN-001 §4.8 T8.1):
//! - `ActionEngine::dispatch(request, idempotency_key)` — 入口
//! - `ActionRegistry::register / get` — 18 Action 注册表
//! - dispatch 顺序: idempotency check → RBAC → confirm → context build
//!
//! 真实 Action 实现 (Merge / Delete / SyncMain 等) 在 P2 / ULYS-57.4 BFF 落点实装;
//! 本期 T8 仅完成 Engine 框架 + 18 Action metadata 注册 (assertion: count = 18)。

use std::collections::HashMap;
use std::sync::Arc;

#[allow(unused_imports)]
// ActionClass 仅在 tests mod `ActionType::X => ActionClass::Y` 路径使用; clippy lib profile 误报 (qualified path 不被识别为 import use)
use crate::action::{
    default_metadata, Action, ActionClass, ActionMetadata, ActionRequest, ActionResult, ActionType,
};
use crate::audit_writer::AuditWriter;
use crate::error::ActionError;
use crate::idempotency::{IdempotencyRecord, IdempotencyStore};
use crate::rbac::{require_confirm_or_die, require_permission, Role, User};
#[allow(unused_imports)]
// Uuid 在 `pub async fn dispatch(..., idempotency_key: Uuid, ...)` 签名中使用; clippy lib profile 误报
use uuid::Uuid;

/// Action Registry — 18 Action 注册表 (per INV-WC-09)
///
/// 真实 Action impl 用 trait object 注入; MVP 阶段只用 metadata 注册,
/// 保证 18 个 ActionType 都可达 (`Self::registered_count == 18`)。
pub struct ActionRegistry {
    /// ActionType -> metadata (本期 T8 用 metadata 占位, 不存具体 impl)
    metadata: HashMap<ActionType, ActionMetadata>,
    /// ActionType -> 实际 Action impl (P2 / BFF 注入)
    #[allow(clippy::type_complexity)]
    actions: HashMap<ActionType, Arc<dyn Action>>,
}

impl std::fmt::Debug for ActionRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActionRegistry")
            .field("registered", &self.metadata.len())
            .finish()
    }
}

impl Default for ActionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ActionRegistry {
    /// 构造一个新 Registry
    pub fn new() -> Self {
        Self {
            metadata: HashMap::new(),
            actions: HashMap::new(),
        }
    }

    /// 注册 Action metadata
    pub fn register(&mut self, action_type: ActionType) -> &mut Self {
        self.metadata
            .insert(action_type, default_metadata(action_type));
        self
    }

    /// 注册 Action impl (P2 实装时用)
    pub fn register_impl(&mut self, action: Arc<dyn Action>) -> &mut Self {
        let m = action.metadata();
        let at = m.action_type;
        self.metadata.insert(at, m);
        self.actions.insert(at, action);
        self
    }

    /// 注册全部 18 个 Action metadata (per INV-WC-09 守门)
    pub fn register_all(&mut self) -> &mut Self {
        for at in ActionType::ALL.iter().copied() {
            self.register(at);
        }
        self
    }

    /// 取 Action metadata
    pub fn metadata(&self, action_type: ActionType) -> Option<&ActionMetadata> {
        self.metadata.get(&action_type)
    }

    /// 取 Action impl (P2 用)
    pub fn action(&self, action_type: ActionType) -> Option<Arc<dyn Action>> {
        self.actions.get(&action_type).cloned()
    }

    /// 已注册数量 (守门 UT-1: `action_18_types_register_correctly`)
    pub fn registered_count(&self) -> usize {
        self.metadata.len()
    }

    /// 全部已注册 ActionType (按 enum 顺序)
    pub fn registered_types(&self) -> Vec<ActionType> {
        let mut v: Vec<ActionType> = self.metadata.keys().copied().collect();
        v.sort_by_key(|a| *a as u8);
        v
    }
}

/// ActionEngine — 18 Action 派发入口
///
/// `dispatch` 流程 (per DD §18 + §42 + §44):
/// 1. Idempotency check (`idempotency_store.check(key)`)
/// 2. RBAC check (`require_permission`)
/// 3. Confirm check (`require_confirm_or_die`)
/// 4. Action execute (P2 / BFF 实装, 本期 T8 stub)
/// 5. Audit append
/// 6. Idempotency record
pub struct ActionEngine {
    /// Registry
    registry: ActionRegistry,
    /// Idempotency Store (内存版)
    idempotency: IdempotencyStore,
    /// Audit Writer
    audit: AuditWriter,
}

impl std::fmt::Debug for ActionEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActionEngine")
            .field("registered", &self.registry.registered_count())
            .finish()
    }
}

impl ActionEngine {
    /// 构造一个新 ActionEngine (注册全部 18 Action)
    pub fn new() -> Self {
        let mut registry = ActionRegistry::new();
        registry.register_all();
        Self {
            registry,
            idempotency: IdempotencyStore::new(),
            audit: AuditWriter::new(),
        }
    }

    /// 构造 with custom registry
    pub fn with_registry(registry: ActionRegistry) -> Self {
        Self {
            registry,
            idempotency: IdempotencyStore::new(),
            audit: AuditWriter::new(),
        }
    }

    /// 取 registry
    pub fn registry(&self) -> &ActionRegistry {
        &self.registry
    }

    /// 取 idempotency store
    pub fn idempotency(&self) -> &IdempotencyStore {
        &self.idempotency
    }

    /// 取 audit writer
    pub fn audit(&self) -> &AuditWriter {
        &self.audit
    }

    /// Dispatch: 入口
    ///
    /// - `request.user_role`: 调用方解析 (per RBAC 决策)
    /// - `idempotency_key`: 客户端传入 (per FR-ACTION-006)
    ///
    /// 返回 ActionResult (成功 / 失败). 错误以 `ActionError` 上抛.
    pub async fn dispatch(
        &self,
        request: ActionRequest,
        user_role: Role,
        idempotency_key: Uuid,
    ) -> Result<ActionResult, ActionError> {
        // 1. Idempotency check
        if let Some(replay) = self.idempotency.check(idempotency_key) {
            let result: ActionResult = serde_json::from_value(replay.result).map_err(|e| {
                ActionError::new("ACTION.IDEMPOTENCY_REPLAY", "replay deser failed").with_source(e)
            })?;
            return Ok(result);
        }

        // 2. RBAC check
        let user = User::new(request.user_id, user_role);
        require_permission(&user, request.action_type)?;

        // 3. Confirm check
        require_confirm_or_die(request.action_type, request.confirm)?;

        // 4. Action execute (P2 / BFF 落点实装; 本期 stub)
        let result = self.execute_stub(&request).await?;

        // 5. Audit append
        self.audit
            .append(
                format!("{:?}", request.action_type),
                request.worktree_id,
                request.user_id,
                "user",
                if result.success { "success" } else { "failed" },
                idempotency_key,
                request.params.clone(),
            )
            .map_err(|e| {
                ActionError::new("AUDIT.WRITE_FAILED", "audit write failed").with_source(e)
            })?;

        // 6. Idempotency record
        let value = serde_json::to_value(&result).map_err(|e| {
            ActionError::new("ACTION.SERIALIZE", "result serialize failed").with_source(e)
        })?;
        self.idempotency.record(IdempotencyRecord {
            key: idempotency_key,
            worktree_id: request.worktree_id,
            result: value,
            created_at: chrono::Utc::now(),
        });

        Ok(result)
    }

    /// Action execute stub (本期 T8 MVP)
    ///
    /// 真实实现 (Merge / Delete / SyncMain 等) 在 P2 / ULYS-57.4 BFF 落点,
    /// 本期仅返回 success + 默认 new_state。
    async fn execute_stub(&self, request: &ActionRequest) -> Result<ActionResult, ActionError> {
        // ActionClass 取 metadata, 不强制要求 impl 存在 (P2)
        let _ = self
            .registry
            .metadata(request.action_type)
            .ok_or_else(|| ActionError::not_available(&format!("{:?}", request.action_type)))?;

        let new_state = match request.action_type {
            ActionType::Merge => Some("merged".to_string()),
            ActionType::Delete | ActionType::ForceDelete | ActionType::Cleanup => {
                Some("deleted".to_string())
            }
            ActionType::Archive => Some("archived".to_string()),
            ActionType::MarkSuperseded => Some("superseded".to_string()),
            ActionType::SyncMain | ActionType::Rebase | ActionType::ForceRebase => {
                Some("synced".to_string())
            }
            _ => None,
        };

        Ok(ActionResult {
            success: true,
            worktree_id: request.worktree_id,
            new_state,
            duration_ms: 0,
            warnings: vec![],
            errors: vec![],
        })
    }

    /// 列出 ActionType (per FR-ACTION-007 — list_available)
    pub fn list_types(&self) -> Vec<ActionType> {
        self.registry.registered_types()
    }
}

impl Default for ActionEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_register_all_has_eighteen() {
        // 守门 UT-1 实证: 18 Action metadata 全部注册
        let mut reg = ActionRegistry::new();
        reg.register_all();
        assert_eq!(reg.registered_count(), 18);
        assert_eq!(reg.registered_types().len(), 18);
        for at in ActionType::ALL.iter().copied() {
            assert!(reg.metadata(at).is_some());
        }
    }

    #[test]
    fn registry_metadata_classification_matches() {
        // 守门: 全部 Action metadata 分类与 INV-WC-09 一致
        let mut reg = ActionRegistry::new();
        reg.register_all();
        for at in ActionType::ALL.iter().copied() {
            let m = reg.metadata(at).unwrap();
            let expected = match at {
                ActionType::Open
                | ActionType::OpenInIDE
                | ActionType::Compare
                | ActionType::Focus
                | ActionType::ExplainRisk
                | ActionType::ListEvents => ActionClass::Safe,
                ActionType::SyncMain
                | ActionType::Rebase
                | ActionType::CreatePR
                | ActionType::Archive
                | ActionType::MarkSuperseded => ActionClass::Warning,
                ActionType::Delete
                | ActionType::Cleanup
                | ActionType::ForceMerge
                | ActionType::ForceRebase
                | ActionType::ForceDelete
                | ActionType::Merge
                | ActionType::RemoveDependency => ActionClass::Destructive,
            };
            assert_eq!(m.classification, expected, "{at:?} classification mismatch");
        }
    }

    #[tokio::test]
    async fn dispatch_happy_path_records_audit_and_idempotency() {
        let engine = ActionEngine::new();
        let req = ActionRequest {
            action_type: ActionType::Compare,
            worktree_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            confirm: false,
            params: serde_json::json!({}),
        };
        let idem_key = Uuid::new_v4();
        let result = engine
            .dispatch(req, Role::PM, idem_key)
            .await
            .expect("PM can Compare");
        assert!(result.success);

        // Audit 1 record
        assert_eq!(engine.audit().len(), 1);
        // Idempotency 1 record
        assert_eq!(engine.idempotency().len(), 1);
    }

    #[tokio::test]
    async fn dispatch_idempotent_replay_returns_same_result() {
        let engine = ActionEngine::new();
        let req = ActionRequest {
            action_type: ActionType::Compare,
            worktree_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            confirm: false,
            params: serde_json::json!({}),
        };
        let idem_key = Uuid::new_v4();

        // First dispatch
        let r1 = engine
            .dispatch(req.clone(), Role::PM, idem_key)
            .await
            .unwrap();
        // Second dispatch (同 key)
        let r2 = engine.dispatch(req, Role::PM, idem_key).await.unwrap();
        assert_eq!(r1.worktree_id, r2.worktree_id);
        // Audit 仍只 1 条 (replay 不重复 audit)
        assert_eq!(engine.audit().len(), 1);
    }

    #[tokio::test]
    async fn dispatch_destructive_requires_confirm() {
        let engine = ActionEngine::new();
        let req = ActionRequest {
            action_type: ActionType::Delete,
            worktree_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            confirm: false,
            params: serde_json::json!({}),
        };
        let err = engine
            .dispatch(req, Role::Owner, Uuid::new_v4())
            .await
            .unwrap_err();
        assert_eq!(err.code, "ACTION_CONFIRM_REQUIRED");
    }

    #[tokio::test]
    async fn dispatch_rbac_denies_viewer_destructive() {
        let engine = ActionEngine::new();
        let req = ActionRequest {
            action_type: ActionType::Delete,
            worktree_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            confirm: true,
            params: serde_json::json!({}),
        };
        let err = engine
            .dispatch(req, Role::Viewer, Uuid::new_v4())
            .await
            .unwrap_err();
        assert_eq!(err.code, "ACTION_DENIED_BY_RBAC");
    }
}
