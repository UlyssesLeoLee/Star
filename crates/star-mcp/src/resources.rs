//! `star-mcp` Resources 能力(per MCP 2025-06-27 spec §3, Phase E 实装)
//!
//! per `docs/architecture/2026-08-26-upgrade/spec/mcp/01-mcp-spec.md` §4 (Phase 2 候选, Phase E 提升到 Level 3+)
//!
//! ## Phase E 实装
//!
//! - **4 个核心 resource** (per 任务 brief):
//!   - `workspace://current` — 当前 workspace 摘要(per `agent-api/v1#WorkspaceSummary`)
//!   - `worktree://{id}`     — worktree 详情(per `agent-api/v1#Worktree`)
//!   - `agent://{id}/state`  — agent session 状态(per ADR-0030 Lease + Heartbeat)
//!   - `decision://{id}`     — 决策记录(per `flows/02` Decision schema)
//! - **`ResourcesHandler` struct** + URI 解析 + 错误处理走 `error.rs` 6-field 错误模型
//! - 数据源: **mock-but-functional** —— 返回带 `_mock: true` 标记的 JSON, 真实数据源接入留 Phase F
//!
//! ## 守门规则
//!
//! - 0 unsafe
//! - URI 解析失败 → `McpError::user_input()` (per F-06 source_kind = user_input)
//! - 资源不存在 → `McpError::new(RESOURCE_NOT_FOUND, ...)`
//! - 缺标比错标安全: mock 数据不编造, 加 `_mock: true` + `_todo: "Phase F 接入真实数据源"` 标记
//! - 旧 D.5+ `star://tools/*` 资源移除(per Phase E 任务 brief 明确换 URI scheme)
//!
//! ## MCP 2025-06-27 协议契合
//!
//! - `resources/list` 返回 `{ resources: [{ uri, name, description, mimeType }] }` 数组
//! - `resources/read` 返回 `{ contents: [{ uri, mimeType, text }] }` 数组(per spec §3.2)
//! - URI scheme 必为 `<scheme>://<path>` 格式(per spec §3.1)

#![warn(missing_docs)]

use async_trait::async_trait;
use serde::Serialize;
use serde_json::{Value, json};

use crate::error::{ErrorSourceKind, McpError, error_code};
use crate::transport::{JsonRpcError, JsonRpcErrorBody, JsonRpcRequest, JsonRpcSuccess};

/// Resources handler (Phase E: 4 核心 resource + Phase H: 22 domain handler 框架)
///
/// Phase H (per `docs/architecture/2026-08-26-upgrade/spec/agents/02-data-sources-spec.md` §2.2 URI 模式
/// + `spec/mcp/02-resources-prompts-spec.md` §2 + `spec/cache/01-cache-contract-spec.md` §4):
/// - 内置 22 domain handler 槽位 (`Vec<Box<dyn DynResource>>`), 每 domain 对应一种 `crate::handlers::*Handler`
/// - 全部 mock-but-functional (per Phase E mock 守门), 真实数据源接入留 Phase H+
///
/// 当前是 unit struct, 未来可注入 `Arc<WorktreeService>` / `Arc<AgentService>` 做真实数据接入。
#[allow(unreachable_pub)] // pub(crate) module
pub struct ResourcesHandler {
    /// Phase H: 22 domain handler 列表 (per spec/agents/02 §2 + spec/mcp/02 §2)
    pub(crate) domains: Vec<Box<dyn DynResource>>,
    // Phase F 占位: 真实数据源依赖将在此处注入
    // e.g. _worktree_service: Arc<dyn WorktreeReadPort>,
    //      _agent_service: Arc<dyn AgentStateReadPort>,
    //      _decision_service: Arc<dyn DecisionReadPort>,
}

#[allow(unreachable_pub)] // ResourcesHandler is pub(crate); methods inherit that scope
impl Default for ResourcesHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(unreachable_pub)] // ResourcesHandler is pub(crate); methods inherit that scope
impl ResourcesHandler {
    /// 构造新 handler (Phase E: 空 domain 列表)
    pub fn new() -> Self {
        Self { domains: Vec::new() }
    }

    /// 构造带 22 domain handler 列表的 handler (Phase H 工厂)
    ///
    /// per `docs/architecture/2026-08-26-upgrade/spec/agents/02-data-sources-spec.md` §2
    /// + `spec/mcp/02-resources-prompts-spec.md` §2:22 domain URI 模式注册
    /// + `spec/cache/01-cache-contract-spec.md` §4 TTL 策略。
    ///
    /// 22 handler 全部 mock-but-functional (per AGENTS.md 缺标比错标安全守门), 真实数据源接入
    /// 由 Phase H+ 排期, 当前标 `TODO: Phase H+ 接 crates/domain-*` 真实数据。
    pub fn with_domains(domains: Vec<Box<dyn DynResource>>) -> Self {
        Self { domains }
    }

    /// 列出所有可读 resource
    ///
    /// per `spec/mcp/01` §3: 数组元素 = `{ uri, name, description, mimeType }`
    /// Phase H: 4 核心 (Phase E) + 22 domain (Phase H) = 26 entries
    #[allow(dead_code)] // 公开 API, 供 Phase F handler 注入后用
    pub fn list(&self) -> Vec<Value> {
        let mut out = vec![
            self.resource_descriptor("workspace://current", "current workspace summary"),
            self.resource_descriptor("worktree://wt-STAR-1024", "worktree detail (example)"),
            self.resource_descriptor("agent://agent-1/state", "agent session state (example)"),
            self.resource_descriptor("decision://dec-001", "decision record (example)"),
        ];
        // Phase H: 22 domain URI 注入
        for d in &self.domains {
            out.push(self.resource_descriptor(
                d.uri_pattern(),
                d.description(),
            ));
        }
        out
    }

    fn resource_descriptor(&self, uri: &str, description: &str) -> Value {
        let name = uri.split("://").next().unwrap_or(uri);
        json!({
            "uri": uri,
            "name": name,
            "description": description,
            "mimeType": "application/json"
        })
    }

    /// 读取指定 URI 的资源内容
    ///
    /// 返回 JSON 字符串(text 字段), 失败时返回 6-field `McpError`。
    #[allow(dead_code)] // 公开 API
    pub async fn read(&self, uri: &str) -> Result<Value, McpError> {
        // URI scheme 解析
        let (scheme, path) = parse_uri(uri)?;

        // 优先级 (per spec/agents/02 §2.2 L80-86 + spec/mcp/02 §1.6 L69-79):
        //   1) 4 核心 mock 仅在 URI 匹配 Phase E example id 时生效:
        //      - workspace://current
        //      - worktree://wt-STAR-1024 (mock example)
        //      - agent://agent-1/state (mock example)
        //      - decision://dec-001 (mock example)
        //   2) 24 domain handler — Phase H 真实数据接入 (UUID 格式 id)
        // 注: B.2.6 起 24 domain 注册, worktree/agent 真实 handler 接收
        //   UUID 格式 id; mock example 走 4 核心, 真实 UUID 走 domain handler
        match (scheme, path) {
            ("workspace", "current") => return self.read_workspace_current(uri).await,
            ("worktree", "wt-STAR-1024") => return self.read_worktree(uri, "wt-STAR-1024").await,
            ("agent", "agent-1/state") => {
                return self.read_agent_state(uri, "agent-1", "state").await;
            }
            ("decision", "dec-001") => return self.read_decision(uri, "dec-001").await,
            _ => {}
        }

        // Phase H: 24 domain handler fallback (UUID 格式 ID)
        for d in &self.domains {
            let pattern_scheme = d.uri_pattern().split("://").next().unwrap_or("");
            if pattern_scheme == scheme {
                return match d.read_json(path).await {
                    Ok(Some(data)) => Ok(contents(uri, &data)),
                    Ok(None) => Err(McpError::new(
                        error_code::RESOURCE_NOT_FOUND,
                        format!("resource not found: {uri}"),
                        "mcp",
                        ErrorSourceKind::UserInput,
                        false,
                        None,
                    )),
                    Err(e) => Err(McpError::new(
                        error_code::INTERNAL,
                        format!("domain handler read failed: {e}"),
                        "mcp",
                        ErrorSourceKind::External,
                        true,
                        None,
                    )),
                };
            }
        }

        Err(McpError::new(
            error_code::RESOURCE_URI_INVALID,
            format!("unsupported resource scheme: '{scheme}://'"),
            "mcp",
            ErrorSourceKind::UserInput,
            false,
            Some("supported schemes: workspace://, worktree://, agent://, decision:// + 22 domain (Phase H)".to_string()),
        ))
    }

    // ===== 4 个核心 resource mock-but-functional 实装 =====

    /// `workspace://current` — 当前 workspace 摘要
    ///
    /// per `agent-api/v1#WorkspaceSummary` (spec/agent-api/01 §3.15)
    /// TODO(Phase F): 注入 `WorkspaceReadPort` 真实数据源
    async fn read_workspace_current(&self, uri: &str) -> Result<Value, McpError> {
        // mock-but-functional: 返回符合 §3.15 schema 的 JSON, 加 _mock 标记
        let body = json!({
            "_mock": true,
            "_todo": "Phase F: inject WorkspaceReadPort for live data",
            "schema_version": "agent-api/v1",
            "workspace": {
                "id": "ws-current",
                "name": "current workspace (mock)",
                "repository": {
                    "id": "repo-1",
                    "provider": "git",
                    "url": "https://example.invalid/repo.git"
                },
                "worktree_id": "wt-STAR-1024",
                "agent_session_id": "agent-1",
                "created_at": "2026-08-27T00:00:00Z",
                "updated_at": "2026-08-27T19:00:00Z"
            }
        });
        Ok(contents(uri, &body))
    }

    /// `worktree://{id}` — worktree 详情
    ///
    /// per `agent-api/v1#Worktree` (spec/agent-api/01 §3.2)
    /// TODO(Phase F): 注入 `WorktreeReadPort` 真实数据源
    async fn read_worktree(&self, uri: &str, id: &str) -> Result<Value, McpError> {
        if id.is_empty() {
            return Err(McpError::new(
                error_code::USER_INPUT,
                "worktree URI must include a non-empty id (e.g. worktree://wt-STAR-1024)",
                "mcp",
                ErrorSourceKind::UserInput,
                false,
                None,
            ));
        }
        let body = json!({
            "_mock": true,
            "_todo": "Phase F: inject WorktreeReadPort for live data",
            "schema_version": "agent-api/v1",
            "worktree": {
                "id": id,
                "branch": "feat/example",
                "base": "main",
                "head_sha": "0000000",
                "status": "open",
                "issue_id": null,
                "agent_session_id": "agent-1",
                "created_at": "2026-08-27T00:00:00Z"
            }
        });
        Ok(contents(uri, &body))
    }

    /// `agent://{id}/state` — agent session 状态
    ///
    /// per ADR-0030 Lease + Heartbeat, 11 字段 agent session 视角
    /// TODO(Phase F): 注入 `AgentStateReadPort` 真实数据源
    async fn read_agent_state(&self, uri: &str, id: &str, suffix: &str) -> Result<Value, McpError> {
        if id.is_empty() {
            return Err(McpError::user_input(
                "agent URI must include a non-empty id (e.g. agent://agent-1/state)",
                None,
            ));
        }
        if suffix != "state" {
            return Err(McpError::new(
                error_code::RESOURCE_URI_INVALID,
                format!("unsupported agent resource suffix: '/{suffix}' (expected '/state')"),
                "mcp",
                ErrorSourceKind::UserInput,
                false,
                Some(format!("try '{uri}state' instead")),
            ));
        }
        let body = json!({
            "_mock": true,
            "_todo": "Phase F: inject AgentStateReadPort for live lease/heartbeat data",
            "schema_version": "agent-api/v1",
            "agent": {
                "id": id,
                "state": "Running",
                "last_heartbeat_at": "2026-08-27T19:00:00Z",
                "lease_expires_at": "2026-08-27T19:05:00Z",
                "current_state": "Step 3 of 12 (Universal Submit)",
                "current_step": 3,
                "retry_count": 0,
                "artifacts": [],
                "checkpoint": "ckpt-001",
                "recovery_hint": "use lease resume (per ADR-0030)"
            }
        });
        Ok(contents(uri, &body))
    }

    /// `decision://{id}` — 决策记录
    ///
    /// per `spec/flows/02` Decision schema
    /// TODO(Phase F): 注入 `DecisionReadPort` 真实数据源
    async fn read_decision(&self, uri: &str, id: &str) -> Result<Value, McpError> {
        if id.is_empty() {
            return Err(McpError::user_input(
                "decision URI must include a non-empty id (e.g. decision://dec-001)",
                None,
            ));
        }
        let body = json!({
            "_mock": true,
            "_todo": "Phase F: inject DecisionReadPort for live decision log",
            "schema_version": "agent-api/v1",
            "decision": {
                "id": id,
                "title": "decision record (mock)",
                "status": "recorded",
                "actor": "agent-1",
                "context_refs": ["issue:STAR-1024", "worktree:wt-STAR-1024"],
                "alternatives_considered": [],
                "chosen": "mock choice (no real data yet)",
                "rationale": "Phase E stub — Phase F will populate from real decision log",
                "created_at": "2026-08-27T00:00:00Z"
            }
        });
        Ok(contents(uri, &body))
    }
}

// ======================================================================
// Phase H: 22 domain Resource trait + KeyBuilder + ResourceError
// (per `docs/architecture/2026-08-26-upgrade/spec/agents/02-data-sources-spec.md` §2
//   + `spec/mcp/02-resources-prompts-spec.md` §2
//   + `spec/cache/01-cache-contract-spec.md` §3 §4)
// ======================================================================

/// Cache key builder (per `spec/cache/01-cache-contract-spec.md` §3 L119-126)
///
/// 22 domain handler 用 `KeyBuilder::for_resource(scheme, id)` 生成统一 cache key
/// (e.g. `agent:agent-1`), Phase G+ 接入 Redis 后端后实际生效。
#[allow(unreachable_pub)]
pub(crate) struct KeyBuilder;

impl KeyBuilder {
    /// Build cache key from resource URI per `spec/cache/01` §3
    ///
    /// 格式: `<scheme>:<id>` (e.g. `agent:agent-1` / `worktree:wt-STAR-1024`)
    /// Phase H mock handler 在 read 时调用此函数生成 cache key (Phase G+ 真正写入 Redis)。
    #[allow(dead_code)] // Phase G+ Redis 接入后由 cache 层调用
    pub(crate) fn for_resource(scheme: &str, id: &str) -> String {
        format!("{scheme}:{id}")
    }
}

/// Resource read error (per `spec/mcp/03-error-model-spec.md` §3 6-field error model)
///
/// Phase H mock handler 的 read 错误, 映射到 McpError 走 6 字段模型
/// (per `spec/mcp/03` §1 定义)。
#[allow(unreachable_pub)]
#[allow(dead_code)] // 部分 variant 留给 Phase H+ 真实数据接入
#[derive(Debug, thiserror::Error)]
pub(crate) enum ResourceError {
    /// 资源不存在 (`RESOURCE_NOT_FOUND`)
    #[error("not found: {0}")]
    NotFound(String),
    /// URI 格式非法 (`RESOURCE_URI_INVALID`)
    #[error("invalid URI: {0}")]
    InvalidUri(String),
    /// 权限拒绝 (`POLICY_DENIED`, per spec/agents/02 §4 写权限矩阵)
    #[error("permission denied: {0}")]
    PermissionDenied(String),
    /// 序列化失败 (内部)
    #[error("serialize error: {0}")]
    Serialize(String),
    /// 上游错误 (内部)
    #[error("internal error: {0}")]
    Internal(String),
}

/// Resource trait (per `spec/mcp/02` §2 + `spec/agents/02` §2.2)
///
/// 22 domain handler 各自实现此 trait 暴露 URI-patterned resource。
///
/// 关联类型 `Data` 用具体结构体 (e.g. `AgentData`) 提供类型安全,
/// 同时通过 blanket impl 自动获得 `DynResource` 以注册到 `ResourcesHandler::domains`。
#[allow(unreachable_pub)]
#[async_trait]
pub(crate) trait Resource: Send + Sync {
    /// 资源数据类型 (e.g. `AgentData` / `WorktreeData`)
    ///
    /// 必须 `Serialize` 以便 `DynResource::read_json` 转换为 `serde_json::Value`
    type Data: Serialize + Send + Sync;

    /// URI 模式 (e.g. `agent://{id}` / `worktree://{id}`)
    fn uri_pattern(&self) -> &str;

    /// Cache TTL in seconds (per `spec/cache/01` §4 TTL 策略表)
    #[allow(dead_code)] // 真实 cache 层 Phase G+ 接入时调用
    fn cache_ttl_sec(&self) -> u32;

    /// 读取资源(返回 typed Data)
    ///
    /// 返回 `Ok(None)` 表示资源不存在 (per `spec/mcp/02` §3 404 语义)。
    async fn read(&self, id: &str) -> Result<Option<Self::Data>, ResourceError>;
}

/// Type-erased Resource (per Phase H 设计: 22 handler 装入 `Vec<Box<dyn DynResource>>`)
///
/// 把 `Resource<Data = X>` 适配为统一 JSON 输出, 允许 `ResourcesHandler` 持有异构 handler 列表。
#[allow(unreachable_pub)]
#[async_trait]
pub(crate) trait DynResource: Send + Sync {
    /// URI 模式 (e.g. `agent://{id}`)
    fn uri_pattern(&self) -> &str;

    /// 人类可读描述 (e.g. `agent session state`)
    fn description(&self) -> &str;

    /// Cache TTL in seconds (per `spec/cache/01` §4)
    #[allow(dead_code)] // 真实 cache 层 Phase G+ 接入时调用
    fn cache_ttl_sec(&self) -> u32;

    /// 读取资源, 序列化为 `serde_json::Value` (统一 JSON 输出)
    async fn read_json(&self, id: &str) -> Result<Option<Value>, ResourceError>;
}

/// Blanket impl: 任何 `Resource` 自动 `DynResource`
///
/// 委托 `Resource::uri_pattern` / `cache_ttl_sec` / `read` 序列化为 `Value`。
#[async_trait]
impl<T> DynResource for T
where
    T: Resource,
    T::Data: Serialize + Send + Sync,
{
    fn uri_pattern(&self) -> &str {
        Resource::uri_pattern(self)
    }
    fn description(&self) -> &str {
        // 默认从 uri_pattern 提取 scheme: `agent://{id}` → `agent domain resource`
        let scheme = self.uri_pattern().split("://").next().unwrap_or("");
        // 我们借用 self.uri_pattern 一次: 在 description 闭包内复制
        // 简单处理: 用 Box::leak 不可行 (会泄漏), 改为返回静态字符串
        // 这里返回 scheme 名字 + " domain resource"
        match scheme {
            "agent" => "agent domain resource",
            "worktree" => "worktree domain resource",
            "feedback" => "feedback domain resource",
            "audit" => "audit domain resource",
            "automation" => "automation domain resource",
            "context" => "context domain resource",
            "decision" => "decision domain resource",
            "identity" => "identity domain resource",
            "integration" => "integration domain resource",
            "notification" => "notification domain resource",
            "permission" => "permission domain resource",
            "scm" => "scm domain resource",
            "search" => "search domain resource",
            "tenant" => "tenant domain resource",
            "validation" => "validation domain resource",
            "work_item" => "work_item domain resource",
            "board" => "board domain resource",
            "collaboration" => "collaboration domain resource",
            "comment" => "comment domain resource",
            "development" => "development domain resource",
            "planning" => "planning domain resource",
            "project" => "project domain resource",
            "relation" => "relation domain resource",
            _ => "domain resource",
        }
    }
    fn cache_ttl_sec(&self) -> u32 {
        Resource::cache_ttl_sec(self)
    }
    async fn read_json(&self, id: &str) -> Result<Option<Value>, ResourceError> {
        match Resource::read(self, id).await? {
            Some(data) => serde_json::to_value(&data)
                .map(Some)
                .map_err(|e| ResourceError::Serialize(e.to_string())),
            None => Ok(None),
        }
    }
}

/// 把 resource body 包装成 MCP `resources/read` 标准 `contents[]` 格式
fn contents(uri: &str, body: &Value) -> Value {
    let text = serde_json::to_string_pretty(body).unwrap_or_else(|_| "{}".to_string());
    json!({
        "contents": [
            {
                "uri": uri,
                "mimeType": "application/json",
                "text": text
            }
        ]
    })
}

/// URI 解析: `<scheme>://<path>`
///
/// 拆分 scheme 与 path, 校验基本格式
fn parse_uri(uri: &str) -> Result<(&str, &str), McpError> {
    let (scheme, rest) = uri.split_once("://").ok_or_else(|| {
        McpError::new(
            error_code::RESOURCE_URI_INVALID,
            format!("uri must be 'scheme://path' format, got: {uri}"),
            "mcp",
            ErrorSourceKind::UserInput,
            false,
            Some("supported schemes: workspace://, worktree://, agent://, decision://".to_string()),
        )
    })?;
    if scheme.is_empty() || rest.is_empty() {
        return Err(McpError::new(
            error_code::RESOURCE_URI_INVALID,
            format!("uri must have non-empty scheme and path, got: {uri}"),
            "mcp",
            ErrorSourceKind::UserInput,
            false,
            None,
        ));
    }
    Ok((scheme, rest))
}

// ===== JSON-RPC 2.0 入口(per Phase D.5+ 接口契约, 委托给 handler) =====

/// 处理 `resources/list` 请求 — 返回 4 个 core + 24 domain resource 描述
pub(crate) fn handle_resources_list(req: &JsonRpcRequest) -> Result<JsonRpcSuccess, JsonRpcError> {
    // B.2.6 fix: 必须用 with_domains 注册 24 domain handler, 否则 dispatch 走 4 核心 fallback
    let handler = ResourcesHandler::with_domains(crate::handlers::all_domain_handlers());
    let resources = handler.list();
    let result = json!({ "resources": resources });
    Ok(JsonRpcSuccess { jsonrpc: "2.0", id: req.id.clone(), result })
}

/// 处理 `resources/read` 请求
///
/// 期望 params = { "uri": "<scheme>://<path>" }
pub(crate) async fn handle_resources_read(req: &JsonRpcRequest) -> Result<JsonRpcSuccess, JsonRpcError> {
    let uri = req
        .params
        .get("uri")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            JsonRpcError {
                jsonrpc: "2.0",
                id: req.id.clone(),
                error: JsonRpcErrorBody {
                    code: crate::transport::error_code::INVALID_PARAMS,
                    message: "missing 'uri' in params".to_string(),
                    data: None,
                },
            }
        })?;

    let handler = ResourcesHandler::with_domains(crate::handlers::all_domain_handlers());
    match handler.read(uri).await {
        Ok(result) => Ok(JsonRpcSuccess { jsonrpc: "2.0", id: req.id.clone(), result }),
        Err(e) => Err(JsonRpcError {
            jsonrpc: "2.0",
            id: req.id.clone(),
            error: JsonRpcErrorBody {
                code: crate::transport::error_code::INVALID_PARAMS,
                message: e.to_string(),
                data: serde_json::to_value(&e).ok(),
            },
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn handler() -> ResourcesHandler {
        ResourcesHandler::new()
    }

    // ===== 1. workspace://current =====

    #[tokio::test]
    async fn test_read_workspace_current_returns_summary() {
        let h = handler();
        let v = h.read("workspace://current").await.unwrap();
        let contents = v.get("contents").unwrap().as_array().unwrap();
        assert_eq!(contents.len(), 1);
        let item = &contents[0];
        assert_eq!(item.get("uri").unwrap().as_str().unwrap(), "workspace://current");
        assert_eq!(item.get("mimeType").unwrap().as_str().unwrap(), "application/json");
        let text = item.get("text").unwrap().as_str().unwrap();
        let parsed: Value = serde_json::from_str(text).unwrap();
        assert_eq!(parsed.get("schema_version").unwrap().as_str().unwrap(), "agent-api/v1");
        let workspace = parsed.get("workspace").unwrap();
        assert_eq!(workspace.get("id").unwrap().as_str().unwrap(), "ws-current");
        assert!(parsed.get("_mock").unwrap().as_bool().unwrap(), "must be marked _mock");
    }

    // ===== 2. worktree://{id} =====

    #[tokio::test]
    async fn test_read_worktree_returns_mock() {
        let h = handler();
        let v = h.read("worktree://wt-STAR-1024").await.unwrap();
        let contents = v.get("contents").unwrap().as_array().unwrap();
        let text = contents[0].get("text").unwrap().as_str().unwrap();
        let parsed: Value = serde_json::from_str(text).unwrap();
        let wt = parsed.get("worktree").unwrap();
        assert_eq!(wt.get("id").unwrap().as_str().unwrap(), "wt-STAR-1024");
        assert_eq!(wt.get("status").unwrap().as_str().unwrap(), "open");
    }

    #[tokio::test]
    async fn test_read_worktree_empty_id_rejected() {
        let h = handler();
        let err = h.read("worktree://").await.unwrap_err();
        // parse_uri 早于 read_worktree 拦截 empty path → RESOURCE_URI_INVALID
        assert_eq!(err.code, error_code::RESOURCE_URI_INVALID);
        assert_eq!(err.source_kind, ErrorSourceKind::UserInput);
    }

    // ===== 3. agent://{id}/state =====

    #[tokio::test]
    async fn test_read_agent_state_returns_lease_info() {
        let h = handler();
        let v = h.read("agent://agent-1/state").await.unwrap();
        let text = v.get("contents").unwrap().as_array().unwrap()[0]
            .get("text").unwrap().as_str().unwrap();
        let parsed: Value = serde_json::from_str(text).unwrap();
        let agent = parsed.get("agent").unwrap();
        assert_eq!(agent.get("id").unwrap().as_str().unwrap(), "agent-1");
        assert!(agent.get("state").is_some());
        assert!(agent.get("lease_expires_at").is_some(), "ADR-0030 lease field");
    }

    #[tokio::test]
    async fn test_read_agent_state_wrong_suffix_rejected() {
        let h = handler();
        let err = h.read("agent://agent-1/foo").await.unwrap_err();
        assert_eq!(err.code, error_code::RESOURCE_URI_INVALID);
        assert!(err.hint.is_some());
    }

    // ===== 4. decision://{id} =====

    #[tokio::test]
    async fn test_read_decision_returns_mock() {
        let h = handler();
        let v = h.read("decision://dec-001").await.unwrap();
        let text = v.get("contents").unwrap().as_array().unwrap()[0]
            .get("text").unwrap().as_str().unwrap();
        let parsed: Value = serde_json::from_str(text).unwrap();
        let dec = parsed.get("decision").unwrap();
        assert_eq!(dec.get("id").unwrap().as_str().unwrap(), "dec-001");
        assert!(parsed.get("_mock").unwrap().as_bool().unwrap());
    }

    // ===== 错误路径 / URI 解析 =====

    #[tokio::test]
    async fn test_read_unsupported_scheme_rejected() {
        let h = handler();
        let err = h.read("http://example.com/").await.unwrap_err();
        assert_eq!(err.code, error_code::RESOURCE_URI_INVALID);
        assert!(err.message.contains("http"));
    }

    #[tokio::test]
    async fn test_read_missing_scheme_separator_rejected() {
        let h = handler();
        let err = h.read("not-a-uri").await.unwrap_err();
        assert_eq!(err.code, error_code::RESOURCE_URI_INVALID);
    }

    // ===== resources/list 入口测试 =====

    #[tokio::test]
    async fn test_list_returns_4_resources() {
        let h = handler();
        let resources = h.list();
        assert_eq!(resources.len(), 4, "4 core resources per Phase E");
        // 校验 4 个 URI scheme
        let uris: Vec<&str> = resources.iter()
            .map(|r| r.get("uri").unwrap().as_str().unwrap())
            .collect();
        assert!(uris.contains(&"workspace://current"));
        assert!(uris.iter().any(|u| u.starts_with("worktree://")));
        assert!(uris.iter().any(|u| u.starts_with("agent://")));
        assert!(uris.iter().any(|u| u.starts_with("decision://")));
    }

    #[tokio::test]
    async fn test_list_includes_mimetype() {
        let h = handler();
        let resources = h.list();
        for r in &resources {
            assert_eq!(r.get("mimeType").unwrap().as_str().unwrap(), "application/json");
        }
    }

    // ===== JSON-RPC 入口测试(委托给 handler) =====

    #[tokio::test]
    async fn test_handle_resources_list_jsonrpc_entry() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(1),
            method: "resources/list".to_string(),
            params: json!({}),
        };
        let res = handle_resources_list(&req).unwrap();
        let arr = res.result.get("resources").unwrap().as_array().unwrap();
        assert_eq!(arr.len(), 4);
    }

    #[tokio::test]
    async fn test_handle_resources_read_jsonrpc_entry() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(2),
            method: "resources/read".to_string(),
            params: json!({ "uri": "workspace://current" }),
        };
        let res = handle_resources_read(&req).await.unwrap();
        let contents = res.result.get("contents").unwrap().as_array().unwrap();
        assert_eq!(contents.len(), 1);
    }

    #[tokio::test]
    async fn test_handle_resources_read_missing_uri() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(3),
            method: "resources/read".to_string(),
            params: json!({}),
        };
        let res = handle_resources_read(&req).await;
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert_eq!(err.error.code, crate::transport::error_code::INVALID_PARAMS);
    }

    #[tokio::test]
    async fn test_handle_resources_read_error_has_data_envelope() {
        // per spec/mcp/01 §3.2: error.data 应含 6 字段 agent-api/v1#Error
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(4),
            method: "resources/read".to_string(),
            params: json!({ "uri": "http://invalid" }),
        };
        let res = handle_resources_read(&req).await;
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(err.error.data.is_some(), "error.data must contain 6-field Error");
        let data = err.error.data.unwrap();
        assert!(data.get("code").is_some());
        assert!(data.get("source_kind").is_some());
    }

    // ===== Phase H: 22 domain handler 注册 (per spec/agents/02 §2.2 + spec/mcp/02 §2) =====

    fn handler_with_domains() -> ResourcesHandler {
        ResourcesHandler::with_domains(crate::handlers::all_domain_handlers())
    }

    #[tokio::test]
    async fn test_with_domains_returns_28_resources_in_list() {
        // 4 核心 (Phase E) + 24 domain (Phase H: 22 task brief + scm + workspace) = 28
        // B.2.5 起新增 workspace handler (per spec/integration/01 §2 Tier 2)
        let h = handler_with_domains();
        let resources = h.list();
        assert_eq!(
            resources.len(),
            28,
            "4 Phase E core + 24 Phase H domain (含 B.2.5 workspace) = 28"
        );
    }

    #[tokio::test]
    async fn test_with_domains_list_includes_all_22_domain_schemes() {
        let h = handler_with_domains();
        let resources = h.list();
        let uris: Vec<String> = resources.iter()
            .filter_map(|r| r.get("uri").and_then(Value::as_str).map(String::from))
            .collect();
        // 校验 22 domain scheme (per spec/agents/02 §2.2 URI 模式)
        let schemes = [
            "agent://", "audit://", "automation://", "board://",
            "collaboration://", "comment://", "context://", "decision://",
            "development://", "feedback://", "identity://", "integration://",
            "notification://", "permission://", "planning://", "project://",
            "relation://", "scm://", "search://", "tenant://",
            "validation://", "workitem://", "worktree://",
        ];
        for s in schemes {
            assert!(uris.iter().any(|u| u.starts_with(s)), "missing scheme {s}");
        }
    }

    #[tokio::test]
    async fn test_read_dispatches_to_domain_handler() {
        // 验证 read() 走 domain handler 路径
        // B.2.6 起 agent handler 接真实数据 (UUID), 此测试改用
        // 仍为 mock 的 audit URI (audit_id 字段保留, 验证 dispatch 路径)
        let h = handler_with_domains();
        let v = h.read("audit://audit-42").await.unwrap();
        let text = v.get("contents").unwrap().as_array().unwrap()[0]
            .get("text").unwrap().as_str().unwrap();
        let parsed: Value = serde_json::from_str(text).unwrap();
        assert_eq!(parsed.get("audit_id").and_then(Value::as_str), Some("audit-42"));
    }

    #[tokio::test]
    async fn test_read_dispatches_each_domain_scheme() {
        // 验证 read() 对每个 domain scheme 都能正确分发
        // (Phase B.2 + B.2.5 + B.2.6 真实接入: tenant/identity/permission/
        //  workspace/project/workitem/worktree/agent/feedback 用 UUID, 移
        //  除自 dispatch 测试, 在各自 handler roundtrip test 中端到端覆盖)
        let h = handler_with_domains();
        let cases = [
            ("audit://a-1", "audit_id"),
            ("decision://d-1", "dec_id"),
            ("validation://v-1", "val_id"),
            ("search://q-1", "query_id"),
        ];
        for (uri, id_field) in cases {
            let v = h.read(uri).await.unwrap_or_else(|e| panic!("{uri} failed: {e}"));
            let text = v.get("contents").unwrap().as_array().unwrap()[0]
                .get("text").unwrap().as_str().unwrap();
            let parsed: Value = serde_json::from_str(text).unwrap();
            let id_value = parsed.get(id_field).and_then(Value::as_str)
                .unwrap_or_else(|| panic!("{uri} missing field {id_field}"));
            let expected_id = uri.rsplit('/').next().unwrap();
            assert_eq!(id_value, expected_id, "{uri} field {id_field}");
        }
    }
}
