//! `star-mcp` JSON-RPC 2.0 transport + MCP 标准方法
//!
//! per `docs/architecture/2026-08-26-upgrade/spec/mcp/01-mcp-spec.md` §1 + 2026-07-28 关键变更
//!
//! ## Phase D.3 + D.5+ 实装
//!
//! - JSON-RPC 2.0 协议 (id / method / params / result / error)
//! - 7 个 MCP 标准方法: `initialize` / `tools/list` / `tools/call` /
//!   `resources/list` / `resources/read` / `prompts/list` / `prompts/get`
//! - 5 个错误码: -32700 / -32600 / -32601 / -32602 / -32603
//! - 16 tool inputSchema (复用 mod.rs 现有 16 tool invoke)
//! - capabilities 含 `tools` + `resources` + `prompts` (per 2025-06-27 spec)
//! - **不**依赖 rmcp (per 任务 brief 极简骨架约束)
//!
//! ## 守门规则
//!
//! - 0 unsafe
//! - Phase D.5+ 在 Cargo.toml 新增 axum + tokio-stream 依赖 (D.3 的 0 新外部依赖已不适用)
//! - RUSTFLAGS=-D warnings 必 pass

#![warn(missing_docs)]

use std::io::{self, BufRead, Write};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::error::McpError;
use crate::prompts;
use crate::resources;
use crate::tools;

/// JSON-RPC 2.0 request
#[derive(Debug, Deserialize)]
pub(crate) struct JsonRpcRequest {
    /// 必须 = "2.0"
    #[allow(dead_code)]
    pub jsonrpc: String,
    /// 请求 ID (null = notification, 我们不实现 notification)
    pub id: Value,
    /// 方法名
    pub method: String,
    /// 参数
    #[serde(default)]
    pub params: Value,
}

/// JSON-RPC 2.0 success response
#[derive(Debug, Serialize)]
pub(crate) struct JsonRpcSuccess {
    pub jsonrpc: &'static str,
    pub id: Value,
    pub result: Value,
}

/// JSON-RPC 2.0 error response
#[derive(Debug, Serialize)]
pub(crate) struct JsonRpcError {
    pub jsonrpc: &'static str,
    pub id: Value,
    pub error: JsonRpcErrorBody,
}

#[derive(Debug, Serialize)]
pub(crate) struct JsonRpcErrorBody {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// MCP 标准错误码 (per spec/mcp/01 + JSON-RPC 2.0)
pub(crate) mod error_code {
    pub(crate) const PARSE_ERROR: i32 = -32700;
    pub(crate) const INVALID_REQUEST: i32 = -32600;
    pub(crate) const METHOD_NOT_FOUND: i32 = -32601;
    pub(crate) const INVALID_PARAMS: i32 = -32602;
    pub(crate) const INTERNAL_ERROR: i32 = -32603;
}

/// 16 tool 信息 (name + description + inputSchema)
/// per spec/mcp/01 §2 (per 子代理 A P1-E 修复后)
pub(crate) fn tools_list() -> Value {
    json!({
        "tools": [
            {
                "name": "get_issue",
                "description": "Retrieve an issue by id (mock, returns Issue schema)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "issue_id": { "type": "string", "description": "Issue ID (e.g. STAR-1024)" }
                    },
                    "required": ["issue_id"]
                }
            },
            {
                "name": "search_issues",
                "description": "Search issues by query (mock)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": { "type": "string" },
                        "limit": { "type": "integer", "default": 20 }
                    },
                    "required": ["query"]
                }
            },
            {
                "name": "get_current_task",
                "description": "Retrieve current task (mock, returns CurrentTask schema)",
                "inputSchema": { "type": "object", "properties": {} }
            },
            {
                "name": "get_workspace",
                "description": "Retrieve workspace (mock, returns WorkspaceSummary schema)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "workspace_id": { "type": "string" }
                    }
                }
            },
            {
                "name": "get_worktree",
                "description": "Retrieve worktree (mock, returns Worktree schema)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "worktree_id": { "type": "string" }
                    }
                }
            },
            {
                "name": "create_worktree",
                "description": "Create worktree (mock)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "issue_id": { "type": "string" },
                        "branch_name": { "type": "string" }
                    },
                    "required": ["issue_id"]
                }
            },
            {
                "name": "search_code",
                "description": "Search code (mock, returns CodeSearchResult)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": { "type": "string" },
                        "limit": { "type": "integer", "default": 20 }
                    },
                    "required": ["query"]
                }
            },
            {
                "name": "get_symbol",
                "description": "Lookup symbol (mock, returns SymbolResult)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "name": { "type": "string" }
                    },
                    "required": ["name"]
                }
            },
            {
                "name": "find_references",
                "description": "Find symbol references (mock)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "name": { "type": "string" }
                    },
                    "required": ["name"]
                }
            },
            {
                "name": "get_code_context",
                "description": "Get code context (mock)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "file": { "type": "string" },
                        "range": { "type": "object" }
                    },
                    "required": ["file"]
                }
            },
            {
                "name": "get_context",
                "description": "Get context (mock, returns Context schema)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "issue_id": { "type": "string" }
                    },
                    "required": ["issue_id"]
                }
            },
            {
                "name": "create_merge_request",
                "description": "Create merge request (mock, returns MR schema)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "title": { "type": "string" },
                        "base": { "type": "string" },
                        "head": { "type": "string" }
                    },
                    "required": ["title", "base", "head"]
                }
            },
            {
                "name": "request_review",
                "description": "Request review (mock, returns Review schema)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "mr_id": { "type": "string" }
                    },
                    "required": ["mr_id"]
                }
            },
            {
                "name": "run_validation",
                "description": "Run validation (mock, returns ValidationResult)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "worktree_id": { "type": "string" }
                    }
                }
            },
            {
                "name": "get_pipeline_status",
                "description": "Get pipeline status (mock, returns PipelineStatus)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "pipeline_run_id": { "type": "string" }
                    },
                    "required": ["pipeline_run_id"]
                }
            },
            {
                "name": "submit",
                "description": "Universal Submit (per spec/flows/05, 12-step, dry-run default)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "worktree_id": { "type": "string" },
                        "force": { "type": "boolean", "default": false }
                    }
                }
            }
        ]
    })
}

/// handle 1 个 JSON-RPC 2.0 request
pub(crate) async fn handle(req: JsonRpcRequest) -> Result<JsonRpcSuccess, JsonRpcError> {
    match req.method.as_str() {
        "initialize" => handle_initialize(&req),
        "tools/list" => handle_tools_list(&req),
        "tools/call" => handle_tools_call(&req).await,
        "resources/list" => resources::handle_resources_list(&req),
        "resources/read" => resources::handle_resources_read(&req).await,
        "prompts/list" => prompts::handle_prompts_list(&req),
        "prompts/get" => prompts::handle_prompts_get(&req).await,
        method => Err(error(method_not_found(method), req.id.clone())),
    }
}

fn handle_initialize(req: &JsonRpcRequest) -> Result<JsonRpcSuccess, JsonRpcError> {
    // capabilities: tools + resources + prompts (per 2025-06-27 MCP spec)
    // Phase D.5+ 把 resources + prompts 加进 capabilities (per 任务 brief)
    let result = json!({
        "protocolVersion": "2025-06-27",
        "capabilities": {
            "tools": {},
            "resources": {},
            "prompts": {}
        },
        "serverInfo": {
            "name": "star-mcp",
            "version": "0.1.0"
        }
    });
    Ok(JsonRpcSuccess { jsonrpc: "2.0", id: req.id.clone(), result })
}

fn handle_tools_list(req: &JsonRpcRequest) -> Result<JsonRpcSuccess, JsonRpcError> {
    Ok(JsonRpcSuccess { jsonrpc: "2.0", id: req.id.clone(), result: tools_list() })
}

async fn handle_tools_call(req: &JsonRpcRequest) -> Result<JsonRpcSuccess, JsonRpcError> {
    // params = { name: "<tool>", arguments: {...} }
    let name = req
        .params
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| error(invalid_params("missing 'name' in params"), req.id.clone()))?
        .to_string();
    let arguments = req
        .params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));

    // 路由到 16 tool
    // per spec/mcp/01 §3.2: error.data = 完整 agent-api/v1#Error 6 字段(per F-06 修复)
    let tool_result = dispatch(&name, arguments)
        .await
        .map_err(|e| {
            let data = serde_json::to_value(&e).ok();
            error(
                JsonRpcErrorBody {
                    code: error_code::INTERNAL_ERROR,
                    message: e.to_string(),
                    data,
                },
                req.id.clone(),
            )
        })?;

    // tools/call 响应: { content: [{type: "text", text: "<JSON 字符串>"}], isError: false }
    let result = json!({
        "content": [
            {
                "type": "text",
                "text": serde_json::to_string_pretty(&tool_result)
                        .unwrap_or_else(|_| "{}".to_string())
            }
        ],
        "isError": false
    });
    Ok(JsonRpcSuccess { jsonrpc: "2.0", id: req.id.clone(), result })
}

/// 16 tool dispatch (复用 mod.rs 现有 invoke fn)
async fn dispatch(tool: &str, args: Value) -> Result<Value, McpError> {
    match tool {
        "get_issue" => tools::get_issue::invoke(args).await,
        "search_issues" => tools::search_issues::invoke(args).await,
        "get_current_task" => tools::get_current_task::invoke(args).await,
        "get_workspace" => tools::get_workspace::invoke(args).await,
        "get_worktree" => tools::get_worktree::invoke(args).await,
        "create_worktree" => tools::create_worktree::invoke(args).await,
        "search_code" => tools::search_code::invoke(args).await,
        "get_symbol" => tools::get_symbol::invoke(args).await,
        "find_references" => tools::find_references::invoke(args).await,
        "get_code_context" => tools::get_code_context::invoke(args).await,
        "get_context" => tools::get_context::invoke(args).await,
        "create_merge_request" => tools::create_merge_request::invoke(args).await,
        "request_review" => tools::request_review::invoke(args).await,
        "run_validation" => tools::run_validation::invoke(args).await,
        "get_pipeline_status" => tools::get_pipeline_status::invoke(args).await,
        "submit" => tools::submit::invoke(args).await,
        unknown => Err(McpError::unknown_tool(unknown)),
    }
}

// 错误构造 helpers
fn method_not_found(method: &str) -> JsonRpcErrorBody {
    JsonRpcErrorBody { code: error_code::METHOD_NOT_FOUND, message: format!("method not found: {method}"), data: None }
}

fn invalid_params(msg: &str) -> JsonRpcErrorBody {
    JsonRpcErrorBody { code: error_code::INVALID_PARAMS, message: msg.to_string(), data: None }
}

fn error(body: JsonRpcErrorBody, id: Value) -> JsonRpcError {
    JsonRpcError { jsonrpc: "2.0", id, error: body }
}

/// 从 stdin 读 1 行 + 解析 + handle + 写回 stdout
/// 完整 JSON-RPC 2.0 端到端 (multi-turn 通过循环调用 run_session)
pub(crate) async fn run_session<R: BufRead, W: Write>(mut reader: R, mut writer: W) -> io::Result<()> {
    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line)?;
        if n == 0 {
            // EOF: 退出
            return Ok(());
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue; // 跳过空行
        }

        // 解析 request
        let req: JsonRpcRequest = match serde_json::from_str(trimmed) {
            Ok(r) => r,
            Err(e) => {
                let err = JsonRpcError {
                    jsonrpc: "2.0",
                    id: Value::Null,
                    error: JsonRpcErrorBody {
                        code: error_code::PARSE_ERROR,
                        message: format!("parse error: {e}"),
                        data: None,
                    },
                };
                writeln!(writer, "{}", serde_json::to_string(&err).unwrap_or_default())?;
                writer.flush()?;
                continue;
            }
        };

        // 校验 jsonrpc field
        if req.jsonrpc != "2.0" {
            let err = JsonRpcError {
                jsonrpc: "2.0",
                id: req.id.clone(),
                error: JsonRpcErrorBody {
                    code: error_code::INVALID_REQUEST,
                    message: format!("unsupported jsonrpc version: {}", req.jsonrpc),
                    data: None,
                },
            };
            writeln!(writer, "{}", serde_json::to_string(&err).unwrap_or_default())?;
            writer.flush()?;
            continue;
        }

        // handle
        let result = handle(req).await;
        let response = match result {
            Ok(success) => serde_json::to_string(&success).unwrap_or_default(),
            Err(err) => serde_json::to_string(&err).unwrap_or_default(),
        };
        writeln!(writer, "{response}")?;
        writer.flush()?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_initialize() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(1),
            method: "initialize".to_string(),
            params: json!({}),
        };
        let res = handle(req).await.unwrap();
        assert_eq!(res.jsonrpc, "2.0");
        let result = res.result.as_object().unwrap();
        assert_eq!(result.get("protocolVersion").unwrap().as_str().unwrap(), "2025-06-27");
        assert!(result.get("capabilities").is_some());
        assert!(result.get("serverInfo").is_some());

        // Phase D.5+: capabilities 必含 tools + resources + prompts
        let capabilities = result.get("capabilities").unwrap().as_object().unwrap();
        assert!(capabilities.contains_key("tools"), "tools capability missing");
        assert!(capabilities.contains_key("resources"), "resources capability missing");
        assert!(capabilities.contains_key("prompts"), "prompts capability missing");
    }

    #[tokio::test]
    async fn test_tools_list() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(2),
            method: "tools/list".to_string(),
            params: json!({}),
        };
        let res = handle(req).await.unwrap();
        let tools = res.result.get("tools").unwrap().as_array().unwrap();
        assert_eq!(tools.len(), 16);
        let names: Vec<&str> = tools.iter().map(|t| t.get("name").unwrap().as_str().unwrap()).collect();
        assert!(names.contains(&"get_issue"));
        assert!(names.contains(&"submit"));
    }

    #[tokio::test]
    async fn test_tools_call_get_issue() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(3),
            method: "tools/call".to_string(),
            params: json!({
                "name": "get_issue",
                "arguments": { "issue_id": "STAR-1024" }
            }),
        };
        let res = handle(req).await.unwrap();
        let content = res.result.get("content").unwrap().as_array().unwrap();
        assert_eq!(content.len(), 1);
        let text = content[0].get("text").unwrap().as_str().unwrap();
        let parsed: Value = serde_json::from_str(text).unwrap();
        assert_eq!(parsed.get("schema_version").unwrap().as_str().unwrap(), "agent-api/v1");
        let issue = parsed.get("issue").unwrap();
        assert_eq!(issue.get("id").unwrap().as_str().unwrap(), "STAR-1024");
    }

    #[tokio::test]
    async fn test_method_not_found() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(99),
            method: "unknown/method".to_string(),
            params: json!({}),
        };
        let res = handle(req).await;
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert_eq!(err.error.code, error_code::METHOD_NOT_FOUND);
    }

    #[tokio::test]
    async fn test_session_e2e_initialize_then_tools_list() {
        use std::io::Cursor;
        let input = b"{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{}}\n{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n";
        let mut output = Vec::new();
        run_session(Cursor::new(&input[..]), &mut output).await.unwrap();
        let s = String::from_utf8(output).unwrap();
        // 2 个 JSON-RPC response
        assert!(s.contains("\"protocolVersion\""));
        assert!(s.contains("\"tools\""));
        assert!(s.matches("\"jsonrpc\":\"2.0\"").count() >= 2);
    }
}
