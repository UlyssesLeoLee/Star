//! # domain-tool
//!
//! **Star L2 Tool Registry business layer (per ADR-0048 D44 + SRS-001 G-4)**
//!
//! **Gate (per ADR-0048 + AGENTS.md)**:
//! - Gate #1 v19: `cargo check --workspace --all-targets -j 4` 0 err
//! - Gate #4.2: concept->physical crate mapping satisfied
//! - Gate #10: author=Ulysses
//! - Gate #13: v0.0.1 stub does not touch DB schema, later v0.1+ W/T/M to be decided
//!
//! **Lead responsibility**: pending real 5-domain Lead signoff
//! (per AGENTS.md section 0 disclaimer gate #3)

#![allow(missing_docs)] // v0.0.1 stub, Phase 2 spec to complete docs later

use std::collections::HashMap;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use thiserror::Error;
use uuid::Uuid;

// =====================================================================
// Sub-module re-exports (per BD-PI-BORROW-001 §2.4 module layout)
// =====================================================================

/// **Tool supporting DTOs** (`ToolArgs` / `ToolOutput` / `ToolExecutionMode`).
pub mod traits;

pub use traits::{ToolArgs, ToolExecutionMode, ToolOutput};

// =====================================================================
// Tool trait (PI-4 重写, per SRS-PI-BORROW-001 §4 FR-19~23 + BD §3.3)
// =====================================================================

/// **Tool** -- L2 Tool business-layer trait (per ADR-0048 D44 + PI-4 REDESIGN)
///
/// ## 历史
///
/// - v0.0.1: 3 方法 (`init` / `shutdown` / `health_check`) — 仅供 ToolRegistry 生命周期管理.
/// - v0.0.2 (ULYS-205 本 turn): 重写为 5 方法 (`schema` / `prepare_arguments` / `execute` /
///   `execution_mode` / `replay`), 旧 3 方法保留 + 标 `#[deprecated]`, 1 版本后删 (per NFR-6).
///
/// ## 新 5 方法契约 (per FR-19~23)
///
/// - `schema()` — JSON Schema (用 `serde_json::Value`), LLM 用于 tool call 参数生成 (per FR-20).
///   Sprint 2 stub 返回静态 object; Sprint 3 切 `schemars::schema_for!` derive
///   (per BD §2.1, schemars dep 待 Sprint 3 评估添加).
/// - `prepare_arguments()` — 校验 LLM 生成的参数, 类型不匹配 → 返
///   [`ToolRegistryError::InvalidOperation`] (per FR-21), 不调 `execute`.
/// - `execute()` — 实际执行已校验参数, 返 [`ToolOutput`] (per FR-23 execute 路径).
/// - `execution_mode()` — 返 [`ToolExecutionMode`] (Sync / Async / Stream / Background),
///   agent loop 据此调度 (per FR-22, BD-G2).
/// - `replay()` — 返历史执行结果 (`Ok(None)` 表示无记录), 用于复现 / 审计 / 调试
///   (per FR-23, "replay 走 audit_log 重建" per BD §4).
///
/// ## 旧 3 方法 (1 版本后删, per NFR-6 兼容)
///
/// `init()` / `shutdown()` / `health_check()` — 仅保留以兼容 v0.0.1 caller. 推荐迁移:
/// Tool 注册表生命周期由 [`ToolRegistry`] 统一管理, Tool impl 不再自管 init/shutdown.
#[async_trait]
pub trait Tool: Send + Sync {
    // ---- 新 5 方法 (PI-4, per FR-19~23) ----

    /// **FR-20**: 返回 JSON Schema 描述, LLM 用于生成 tool call 参数.
    /// Sprint 2 stub 返回静态 `serde_json::Value`. Sprint 3 切 `schemars::schema_for!`.
    fn schema(&self) -> Value;

    /// **FR-21**: 校验 LLM 生成的参数. 类型不匹配 / 必填字段缺失 → 返
    /// [`ToolRegistryError::InvalidOperation`] 含原因. 校验通过 → 构造 [`ToolArgs`] 返回.
    fn prepare_arguments(&self, args: &Value) -> Result<ToolArgs, ToolRegistryError>;

    /// **FR-23 (execute 路径)**: 实际执行已校验参数, 返 [`ToolOutput`].
    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolRegistryError>;

    /// **FR-22**: 调度提示. 返 [`ToolExecutionMode`] 让 agent loop 决定如何 dispatch
    /// (per BD-G2, agent loop 调度逻辑在 Sprint 3 接入).
    fn execution_mode(&self) -> ToolExecutionMode;

    /// **FR-23 (replay 路径)**: 返历史执行结果. `Ok(None)` = 无记录 (audit_log 不可用
    /// 或 id 不存在). Sprint 2 stub 永远返 `Ok(None)`. Sprint 3 切 audit_log 反查
    /// (per BD §4 `ToolOutput` Transaction + audit_log event_type='tool_result').
    fn replay(&self, execution_id: Uuid) -> Result<Option<ToolOutput>, ToolRegistryError>;

    // ---- 旧 3 方法 (#[deprecated], 1 版本后删, per NFR-6) ----

    /// Initialize (e.g. load persistent state, start background workers).
    ///
    /// **Deprecated since 0.0.2**: 推荐迁移到 [`ToolRegistry`] 生命周期管理, Tool impl
    /// 不再自管. Will be removed in 0.1.0.
    #[deprecated(
        since = "0.0.2",
        note = "use ToolRegistry lifecycle instead (per SRS-PI-BORROW-001 NFR-6); will be removed in 0.1.0"
    )]
    async fn init(&self) -> Result<(), ToolRegistryError>;

    /// Shutdown (e.g. flush pending state, release resources).
    ///
    /// **Deprecated since 0.0.2**: 推荐迁移到 [`ToolRegistry`] 生命周期管理.
    #[deprecated(
        since = "0.0.2",
        note = "use ToolRegistry lifecycle instead (per SRS-PI-BORROW-001 NFR-6); will be removed in 0.1.0"
    )]
    async fn shutdown(&self) -> Result<(), ToolRegistryError>;

    /// Health check (return current backend state).
    ///
    /// **Deprecated since 0.0.2**: 推荐迁移到 [`ToolRegistry::health_check`].
    #[deprecated(
        since = "0.0.2",
        note = "use ToolRegistry::health_check instead (per SRS-PI-BORROW-001 NFR-6); will be removed in 0.1.0"
    )]
    async fn health_check(&self) -> Result<ToolRegistryHealth, ToolRegistryError>;
}

// =====================================================================
// Error type
// =====================================================================

/// **ToolRegistry error type** (per domain-* unified pattern)
#[derive(Debug, Error)]
pub enum ToolRegistryError {
    /// Resource not found
    #[error("{0} not found")]
    NotFound(String),

    /// Backend not initialized
    #[error("backend not initialized")]
    NotInitialized,

    /// Resource exhausted
    #[error("{resource} exhausted: {message}")]
    Exhausted { resource: String, message: String },

    /// Invalid operation
    #[error("invalid operation: {0}")]
    InvalidOperation(String),

    /// Backend error
    #[error("backend error: {0}")]
    Backend(String),
}

// =====================================================================
// Health state
// =====================================================================

/// **ToolRegistry health state**
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolRegistryHealth {
    /// Current registered resource count
    pub count: u64,
    /// Backend identifier (e.g. "in-memory", "redis", "postgres")
    pub backend: String,
    /// Backend healthy flag
    pub healthy: bool,
}

// =====================================================================
// ToolRegistry stub (v0.0.1)
// =====================================================================

/// **ToolRegistry** -- L2 Tool Registry stub (per ADR-0048 D44 v0.0.1)
///
/// v0.0.1 stub: in-memory backend. Later v0.1+ will add persistence.
#[derive(Debug, Default, Clone)]
pub struct ToolRegistry {
    /// in-memory storage (key: Uuid, value: metadata)
    storage: HashMap<Uuid, String>,
    /// initialized flag
    initialized: bool,
}

impl ToolRegistry {
    /// Create empty backend (uninitialized)
    pub fn new() -> Self {
        Self {
            storage: HashMap::new(),
            initialized: false,
        }
    }

    /// Mark backend as initialized (v0.0.1 stub helper for test)
    pub fn mark_initialized(&mut self) {
        self.initialized = true;
    }

    /// Current resource count
    pub fn count(&self) -> u64 {
        self.storage.len() as u64
    }

    /// Register a Tool (per ADR-0048 stub)
    pub fn register(&mut self, key: Uuid, value: String) -> Result<(), ToolRegistryError> {
        if !self.initialized {
            return Err(ToolRegistryError::NotInitialized);
        }
        self.storage.insert(key, value);
        Ok(())
    }

    /// Lookup by tool name (per ADR-0048 stub)
    pub fn lookup(&self, key: &Uuid) -> Result<Option<&String>, ToolRegistryError> {
        if !self.initialized {
            return Err(ToolRegistryError::NotInitialized);
        }
        Ok(self.storage.get(key))
    }
}

#[async_trait]
#[allow(deprecated)] // impl the deprecated methods for v0.0.1 backward-compat
impl Tool for ToolRegistry {
    // ---- 新 5 方法 (PI-4) ----

    /// **FR-20 stub**: ToolRegistry is a registry, not an executable tool itself.
    /// It exposes its own registration count + backend identifier as the
    /// schema payload. Real tools (Sprint 3 BashTool/ReadTool/WriteTool) will
    /// return their tool-specific JSON Schema here.
    fn schema(&self) -> Value {
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "ToolRegistry",
            "description": "L2 Tool Registry (not an executable tool itself; registry-only stub)",
            "type": "object",
            "properties": {
                "name": { "type": "string", "description": "Tool name to look up" },
            },
            "required": ["name"],
        })
    }

    /// **FR-21 stub**: validate the `name` field is a string. Real JSON Schema
    /// validation lands in Sprint 3 (per BD §2.1, schemars + jsonschema deps).
    fn prepare_arguments(&self, args: &Value) -> Result<ToolArgs, ToolRegistryError> {
        let name = args.get("name").and_then(Value::as_str).ok_or_else(|| {
            ToolRegistryError::InvalidOperation(
                "missing or non-string 'name' field in arguments".to_string(),
            )
        })?;
        if name.is_empty() {
            return Err(ToolRegistryError::InvalidOperation(
                "'name' field must be non-empty".to_string(),
            ));
        }
        // tool_call_id is supplied by the agent loop on dispatch; the stub
        // uses a fresh Uuid (real impl will receive it as a separate param).
        Ok(ToolArgs::new(Uuid::new_v4(), args.clone()))
    }

    /// **FR-23 execute stub**: ToolRegistry is a registry, not executable;
    /// returns `Backend` error explaining the limitation. Real tools (Sprint 3)
    /// will perform actual work here.
    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolRegistryError> {
        // v0.0.2 stub: ToolRegistry cannot execute tools — it only stores them.
        // Real tools implement execute() to do the actual work.
        let execution_id = Uuid::new_v4();
        Err(ToolRegistryError::Backend(format!(
            "ToolRegistry cannot execute (Sprint 2 stub); got args {:?} with execution_id {}",
            args.arguments, execution_id
        )))
    }

    /// **FR-22 stub**: ToolRegistry reports Synchronous (it has no async/stream
    /// surface). Real tools will return their actual mode here.
    fn execution_mode(&self) -> ToolExecutionMode {
        ToolExecutionMode::Synchronous
    }

    /// **FR-23 replay stub**: Sprint 2 has no audit_log; replay always returns
    /// `Ok(None)`. Sprint 3 will query audit_log (event_type='tool_result')
    /// keyed by `execution_id`.
    fn replay(&self, _execution_id: Uuid) -> Result<Option<ToolOutput>, ToolRegistryError> {
        Ok(None)
    }

    // ---- 旧 3 方法 (v0.0.1 兼容, per NFR-6) ----

    async fn init(&self) -> Result<(), ToolRegistryError> {
        // v0.0.1 stub: no-op for in-memory backend.
        // Note: `initialized` flag is set via `mark_initialized()` for v0.0.1 stub testing;
        // later v0.1+ real backend will use interior mutability (e.g. Arc<Mutex<bool>>).
        Ok(())
    }

    async fn shutdown(&self) -> Result<(), ToolRegistryError> {
        // v0.0.1 stub: no-op for in-memory backend
        Ok(())
    }

    async fn health_check(&self) -> Result<ToolRegistryHealth, ToolRegistryError> {
        Ok(ToolRegistryHealth {
            count: self.count(),
            backend: "in-memory".to_string(),
            healthy: self.initialized,
        })
    }
}

// =====================================================================
// Unit Tests (per Gate #1 v19 >= 1 test)
// =====================================================================

#[cfg(test)]
#[allow(deprecated)] // Tests deliberately exercise v0.0.1 deprecated methods for backward-compat
mod tests {
    use super::*;

    /// **Test 1 (unit)**: ToolRegistry new is empty
    #[test]
    fn toolregistry_new_is_empty() {
        let backend = ToolRegistry::new();
        assert_eq!(backend.count(), 0);
        assert!(!backend.initialized);
    }

    /// **Test 2 (unit)**: register + lookup roundtrip
    #[test]
    fn toolregistry_register_lookup_roundtrip() {
        let mut backend = ToolRegistry::new();
        let key = Uuid::new_v4();
        let value = "test-value".to_string();

        // Before init: should return NotInitialized
        let err = backend.register(key, value.clone()).unwrap_err();
        assert!(matches!(err, ToolRegistryError::NotInitialized));

        // After init: insert + lookup
        backend.mark_initialized();
        backend.register(key, value.clone()).unwrap();
        let result = backend.lookup(&key).unwrap();
        assert_eq!(result, Some(&value));
        assert_eq!(backend.count(), 1);
    }

    /// **Test 3 (IT)**: trait async init + health_check
    #[tokio::test]
    async fn toolregistry_init_and_health_check() {
        let mut backend = ToolRegistry::new();
        backend.mark_initialized();
        backend.init().await.unwrap();

        let health = backend.health_check().await.unwrap();
        assert_eq!(health.backend, "in-memory");
        assert_eq!(health.count, 0);
        assert!(health.healthy);
    }

    /// **Test 4 (IT)**: trait async shutdown no-op
    #[tokio::test]
    async fn toolregistry_shutdown_no_error() {
        let backend = ToolRegistry::new();
        backend.shutdown().await.unwrap();
    }

    // =====================================================================
    // PI-4 tests — Tool trait 5 方法 (FR-19~23, per BD §7 AC-5)
    // =====================================================================

    /// **Test 5 (unit, PI-4 FR-20)**: schema() returns a JSON Schema object
    /// with the expected top-level keys (title + type).
    #[test]
    fn tool_schema_returns_json_schema() {
        let backend = ToolRegistry::new();
        let schema = <ToolRegistry as Tool>::schema(&backend);
        assert_eq!(schema["type"], "object");
        assert_eq!(schema["title"], "ToolRegistry");
        // Must contain a `$schema` declaration (JSON Schema 2020-12)
        assert!(schema.get("$schema").is_some(), "schema missing $schema");
    }

    /// **Test 6 (unit, PI-4 FR-21 happy)**: prepare_arguments accepts a
    /// valid `{ "name": "x" }` payload and returns ToolArgs.
    #[test]
    fn tool_prepare_arguments_happy_path() {
        let backend = ToolRegistry::new();
        let args = json!({ "name": "bash" });
        let out = <ToolRegistry as Tool>::prepare_arguments(&backend, &args).unwrap();
        assert_eq!(out.arguments, args);
        // tool_call_id should be a fresh v4 UUID
        assert_eq!(out.tool_call_id.get_version_num(), 4);
    }

    /// **Test 7 (unit, PI-4 FR-21 missing field)**: prepare_arguments
    /// rejects a payload missing the `name` field (FR-21 InvalidOperation).
    #[test]
    fn tool_prepare_arguments_missing_name_rejected() {
        let backend = ToolRegistry::new();
        let args = json!({ "path": "/tmp" });
        let err = <ToolRegistry as Tool>::prepare_arguments(&backend, &args).unwrap_err();
        assert!(
            matches!(err, ToolRegistryError::InvalidOperation(ref s) if s.contains("name")),
            "expected InvalidOperation mentioning 'name', got {err:?}"
        );
    }

    /// **Test 8 (unit, PI-4 FR-21 empty name)**: prepare_arguments rejects
    /// an empty `name` string.
    #[test]
    fn tool_prepare_arguments_empty_name_rejected() {
        let backend = ToolRegistry::new();
        let args = json!({ "name": "" });
        let err = <ToolRegistry as Tool>::prepare_arguments(&backend, &args).unwrap_err();
        assert!(
            matches!(err, ToolRegistryError::InvalidOperation(ref s) if s.contains("non-empty")),
            "expected InvalidOperation mentioning 'non-empty', got {err:?}"
        );
    }

    /// **Test 9 (unit, PI-4 FR-21 non-string name)**: prepare_arguments rejects
    /// a payload whose `name` field is not a string (e.g. number).
    #[test]
    fn tool_prepare_arguments_non_string_name_rejected() {
        let backend = ToolRegistry::new();
        let args = json!({ "name": 42 });
        let err = <ToolRegistry as Tool>::prepare_arguments(&backend, &args).unwrap_err();
        assert!(matches!(err, ToolRegistryError::InvalidOperation(_)));
    }

    /// **Test 10 (IT, PI-4 FR-23 execute stub)**: ToolRegistry::execute
    /// returns Backend error (it is a registry, not an executable tool).
    /// Sprint 3 BashTool/ReadTool/WriteTool will exercise the success path.
    #[tokio::test]
    async fn tool_execute_stub_returns_backend_error() {
        let backend = ToolRegistry::new();
        let args = ToolArgs::new(Uuid::new_v4(), json!({ "name": "bash" }));
        let err = <ToolRegistry as Tool>::execute(&backend, args)
            .await
            .unwrap_err();
        assert!(
            matches!(err, ToolRegistryError::Backend(ref s) if s.contains("cannot execute")),
            "expected Backend error mentioning 'cannot execute', got {err:?}"
        );
    }

    /// **Test 11 (unit, PI-4 FR-22)**: execution_mode() returns Synchronous
    /// for the registry stub (no async/stream surface).
    #[test]
    fn tool_execution_mode_is_synchronous_for_registry() {
        let backend = ToolRegistry::new();
        assert_eq!(
            <ToolRegistry as Tool>::execution_mode(&backend),
            ToolExecutionMode::Synchronous
        );
    }

    /// **Test 12 (unit, PI-4 FR-23 replay stub)**: replay() returns Ok(None)
    /// (no audit_log in Sprint 2; per BD §8 BD-G3).
    #[test]
    fn tool_replay_stub_returns_none() {
        let backend = ToolRegistry::new();
        let result = <ToolRegistry as Tool>::replay(&backend, Uuid::new_v4()).unwrap();
        assert!(result.is_none());
    }

    /// **Test 13 (unit)**: ToolExecutionMode::as_str() returns snake_case strings.
    #[test]
    fn tool_execution_mode_as_str() {
        assert_eq!(ToolExecutionMode::Synchronous.as_str(), "synchronous");
        assert_eq!(ToolExecutionMode::Async.as_str(), "async");
        assert_eq!(ToolExecutionMode::Stream.as_str(), "stream");
        assert_eq!(ToolExecutionMode::Background.as_str(), "background");
    }

    /// **Test 14 (unit)**: ToolArgs::new + round-trip equality.
    #[test]
    fn tool_args_new_roundtrip() {
        let id = Uuid::new_v4();
        let v = json!({ "path": "/tmp/foo.txt" });
        let args = ToolArgs::new(id, v.clone());
        assert_eq!(args.tool_call_id, id);
        assert_eq!(args.arguments, v);
    }

    /// **Test 15 (unit)**: ToolOutput::new + round-trip equality.
    #[test]
    fn tool_output_new_roundtrip() {
        let id = Uuid::new_v4();
        let r = json!({ "stdout": "hello", "exit_code": 0 });
        let out = ToolOutput::new(id, "bash", r.clone());
        assert_eq!(out.execution_id, id);
        assert_eq!(out.tool_name, "bash");
        assert_eq!(out.result, r);
    }
}
