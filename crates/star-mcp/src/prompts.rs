//! `star-mcp` Prompts 能力(per MCP 2025-06-27 spec §4, Phase E 实装)
//!
//! per `docs/architecture/2026-08-26-upgrade/spec/mcp/01-mcp-spec.md` §5 (Phase 2 候选, Phase E 提升到 Level 3+)
//!
//! ## Phase E 实装
//!
//! - **5 个 prompt 模板** (per 任务 brief):
//!   - `submit`     — 引导用户走 Universal Submit 12 步(per `spec/flows/05`)
//!   - `review`     — 引导用户提交 MR review(per `spec/flows/04`)
//!   - `context`    — 引导用户收集 issue / worktree / agent 上下文
//!   - `workflow`   — 引导用户选择领域 workflow(5 域 per DEC-008)
//!   - `debug`      — 引导用户排查 agent / pipeline 失败
//! - **`PromptsHandler` struct** + 模板渲染 + 错误处理走 `error.rs` 6-field 错误模型
//! - 模板风格: AskUser / AskAssistant 占位(per MCP spec §4 "messages[]" 格式)
//!
//! ## 守门规则
//!
//! - 0 unsafe
//! - 模板内容是 mock-but-functional — 加 `_mock: true` 标记, 不编造未实装的 workflow
//! - 缺标比错标安全: prompt 名变更 / 字段调整标 TODO, 不假装"完整实装"
//! - 旧 D.5+ "MVP 不实装" 行为移除(per Phase E 任务 brief 明确实装 5 prompt)
//!
//! ## MCP 2025-06-27 协议契合
//!
//! - `prompts/list` 返回 `{ prompts: [{ name, description, arguments? }] }`
//! - `prompts/get` 返回 `{ description, messages: [{ role, content: { type, text } }] }`
//! - `role` 必为 `"user"` 或 `"assistant"`(per spec §4)
//! - `content.type` 必为 `"text"`(MVP 范围)

#![warn(missing_docs)]

use serde_json::{Value, json};

use crate::error::{ErrorSourceKind, McpError, error_code};
use crate::transport::{JsonRpcError, JsonRpcErrorBody, JsonRpcRequest, JsonRpcSuccess};

/// Prompts handler (Phase E: 5 核心 prompt 模板)
///
/// 当前是 unit struct, 未来可注入 prompt 模板仓库(per Phase F+)。
#[allow(unreachable_pub)] // pub(crate) module
#[derive(Debug, Default, Clone)]
pub struct PromptsHandler {}

#[allow(unreachable_pub)] // PromptsHandler is pub(crate); methods inherit that scope
impl PromptsHandler {
    /// 构造新 handler
    pub fn new() -> Self {
        Self {}
    }

    /// 列出所有可用 prompt 模板
    ///
    /// per `spec/mcp/01` §5: 数组元素 = `{ name, description, arguments? }`
    #[allow(dead_code)] // 公开 API
    pub fn list(&self) -> Vec<Value> {
        vec![
            self.prompt_descriptor(
                "submit",
                "Guide user through Universal Submit 12-step flow (per spec/flows/05)",
                &[
                    ("worktree_id", "string", "target worktree id (e.g. wt-STAR-1024)"),
                    ("force", "boolean?", "force submit even if validation fails (default: false)"),
                ],
            ),
            self.prompt_descriptor(
                "review",
                "Compose an MR review request (per spec/flows/04)",
                &[
                    ("mr_id", "string", "merge request id"),
                    ("reviewers", "string[]?", "optional reviewer handles (comma-separated)"),
                ],
            ),
            self.prompt_descriptor(
                "context",
                "Collect full context for an issue/worktree/agent session",
                &[
                    ("issue_id", "string?", "issue id (e.g. STAR-1024)"),
                    ("worktree_id", "string?", "worktree id (e.g. wt-STAR-1024)"),
                ],
            ),
            self.prompt_descriptor(
                "workflow",
                "Pick a domain workflow (5 domains per DEC-008: player / economy / match / social / admin)",
                &[("domain", "string", "domain name (player | economy | match | social | admin)")],
            ),
            self.prompt_descriptor(
                "debug",
                "Diagnose agent / pipeline / submit failures (per F-06 Error 6 字段)",
                &[
                    ("trace_id", "string?", "trace id from previous error"),
                    ("agent_id", "string?", "agent session id"),
                ],
            ),
        ]
    }

    fn prompt_descriptor(&self, name: &str, description: &str, args: &[(&str, &str, &str)]) -> Value {
        let arguments: Vec<Value> = args
            .iter()
            .map(|(arg_name, arg_type, arg_desc)| {
                json!({
                    "name": arg_name,
                    "type": arg_type,
                    "description": arg_desc,
                    "required": !arg_name.ends_with('?') && !arg_type.ends_with('?')
                })
            })
            .collect();

        json!({
            "name": name,
            "description": description,
            "arguments": arguments
        })
    }

    /// 获取指定 prompt 的渲染结果
    ///
    /// 返回 MCP 标准的 `{ description, messages: [...] }` 结构。
    /// 失败时返回 6-field `McpError`。
    #[allow(dead_code)] // 公开 API
    pub async fn get(&self, name: &str, arguments: &Value) -> Result<Value, McpError> {
        match name {
            "submit" => self.get_submit(arguments),
            "review" => self.get_review(arguments),
            "context" => self.get_context(arguments),
            "workflow" => self.get_workflow(arguments),
            "debug" => self.get_debug(arguments),
            other => Err(McpError::new(
                error_code::PROMPT_NOT_FOUND,
                format!("unknown prompt: '{other}' (available: submit, review, context, workflow, debug)"),
                "mcp",
                ErrorSourceKind::UserInput,
                false,
                Some("call prompts/list to enumerate available prompts".to_string()),
            )),
        }
    }

    // ===== 5 个 prompt 模板 =====

    /// `submit` — Universal Submit 12 步引导
    fn get_submit(&self, args: &Value) -> Result<Value, McpError> {
        let worktree_id = args.get("worktree_id").and_then(Value::as_str);
        let worktree = worktree_id.unwrap_or("<unset — set worktree_id>");
        let force = args.get("force").and_then(Value::as_bool).unwrap_or(false);

        let description = format!(
            "Universal Submit 12-step flow for worktree '{worktree}' (force={force})"
        );

        let user_text = format!(
            "请按以下步骤执行 Universal Submit(per `spec/flows/05` 12 步流程):\n\n\
             1. 校验 worktree_id = '{worktree}' 存在且未锁定\n\
             2. 收集 commit + 改动文件列表\n\
             3. 解析 issue_id 关联(从 branch 名推断)\n\
             4. 跑 `cargo test -p <changed_crate>` (per §5 run_validation)\n\
             5. 跑 `cargo clippy -p <changed_crate> --all-targets RUSTFLAGS=-D warnings`\n\
             6. policy check(per ADR-0021 Zero Vendor Cooperation)\n\
             7. 决定 dry-run vs force(force={force})\n\
             8. 生成 MR title + description 草案\n\
             9. 调用 `submit` MCP tool 触发 (§5 §11)\n\
             10. 等待 CI pipeline 完成\n\
             11. request_review(per §5 §12)\n\
             12. 报告最终 SubmitResult 给用户\n\n\
             _mock: true(Phase E stub; 真实 workflow 由 Phase F agent runtime 驱动)"
        );

        let assistant_text = format!(
            "Phase E stub 提示: 用户已要求走 Universal Submit '{worktree}'。\
             Phase F 接入后将: 自动调 tools/call(submit) + run_validation + \
             create_merge_request + request_review 4 件套, 返回 SubmitResult。\n\n\
             强制覆盖: {force}"
        );

        Ok(prompt_messages(&description, &user_text, &assistant_text))
    }

    /// `review` — MR review 请求引导
    fn get_review(&self, args: &Value) -> Result<Value, McpError> {
        let mr_id = require_string_arg(args, "mr_id")?;
        let reviewers = args
            .get("reviewers")
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(Value::as_str)
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_else(|| "<unset — default: codeowners>".to_string());

        let description = format!("Compose MR review request for '{mr_id}' (reviewers: {reviewers})");

        let user_text = format!(
            "请为 MR '{mr_id}' 生成 review 请求:\n\n\
             1. 校验 MR '{mr_id}' 存在且可 review(状态 = open)\n\
             2. 校验 reviewers = '{reviewers}' 都是合法 handle\n\
             3. 收集 MR 改动统计(diff size / files / commits)\n\
             4. 调用 `request_review` MCP tool 触发\n\
             5. 等待 Reviewer response 最多 24h\n\
             6. 汇总 review 结果给用户\n\n\
             _mock: true(Phase E stub)"
        );

        let assistant_text = format!(
            "Phase E stub: 即将调 `tools/call(request_review, mr_id='{mr_id}', reviewers='{reviewers}')`。"
        );

        Ok(prompt_messages(&description, &user_text, &assistant_text))
    }

    /// `context` — 上下文收集引导
    fn get_context(&self, args: &Value) -> Result<Value, McpError> {
        let issue_id = args.get("issue_id").and_then(Value::as_str);
        let worktree_id = args.get("worktree_id").and_then(Value::as_str);

        // 至少要一个引用, 否则 prompt 范围模糊
        if issue_id.is_none() && worktree_id.is_none() {
            return Err(McpError::new(
                error_code::PROMPT_ARG_MISSING,
                "context prompt requires at least one of: issue_id, worktree_id",
                "mcp",
                ErrorSourceKind::UserInput,
                false,
                Some("provide either issue_id='STAR-N' or worktree_id='wt-N'".to_string()),
            ));
        }

        let description = format!(
            "Collect context for issue={} worktree={}",
            issue_id.unwrap_or("<none>"),
            worktree_id.unwrap_or("<none>")
        );

        let user_text = format!(
            "请收集以下上下文信息:\n\n\
             - issue: {} → 调 `get_issue(issue_id='{}')` + `get_context(issue_id='{}')`\n\
             - worktree: {} → 调 `get_worktree(worktree_id='{}')` + `resources/read worktree://{}`\n\
             - 汇总返回:\n\
               * issue.title / status / description\n\
               * worktree.branch / base / head_sha / agent_session_id\n\
               * context.related_mrs / related_decisions\n\n\
             _mock: true(Phase E stub)",
            issue_id.unwrap_or("N/A"),
            issue_id.unwrap_or("N/A"),
            issue_id.unwrap_or("N/A"),
            worktree_id.unwrap_or("N/A"),
            worktree_id.unwrap_or("N/A"),
            worktree_id.unwrap_or("N/A")
        );

        let assistant_text = "Phase E stub: 即将并发调 4 个 read tool + 1 个 resources/read, 合并返回。".to_string();

        Ok(prompt_messages(&description, &user_text, &assistant_text))
    }

    /// `workflow` — 领域 workflow 选择
    fn get_workflow(&self, args: &Value) -> Result<Value, McpError> {
        let domain = require_string_arg(args, "domain")?;

        // 校验 domain ∈ 5 域 per DEC-008
        let valid = ["player", "economy", "match", "social", "admin"];
        if !valid.contains(&domain.as_str()) {
            return Err(McpError::new(
                error_code::USER_INPUT,
                format!("invalid domain '{domain}' (per DEC-008, 5 domains: player | economy | match | social | admin)"),
                "mcp",
                ErrorSourceKind::UserInput,
                false,
                Some(format!("valid: {}", valid.join(" | "))),
            ));
        }

        let description = format!("Workflow for domain '{domain}' (5-domain per DEC-008)");

        let user_text = format!(
            "请引导用户走 '{domain}' 域 workflow:\n\n\
             1. 列出 '{domain}' 域典型任务类型(per `spec/flows/0X`)\n\
             2. 询问用户具体任务目标\n\
             3. 推荐 3-5 个 tool 组合(per 5 域独立 Lead 矩阵)\n\
             4. 走对应 Lead Review 流程\n\
             5. 落地到 5 域 Lead RACI 表\n\n\
             _mock: true(Phase E stub; 真实 workflow 模板 Phase F 由各域 Lead 注入)"
        );

        let assistant_text = format!(
            "Phase E stub: '{domain}' 域 Lead (per DEC-008 5 域独立 Lead 拒绝兼任) 将负责 workflow 落地。"
        );

        Ok(prompt_messages(&description, &user_text, &assistant_text))
    }

    /// `debug` — Agent / pipeline 失败排查
    fn get_debug(&self, args: &Value) -> Result<Value, McpError> {
        let trace_id = args.get("trace_id").and_then(Value::as_str);
        let agent_id = args.get("agent_id").and_then(Value::as_str);

        // 至少 1 个线索
        if trace_id.is_none() && agent_id.is_none() {
            return Err(McpError::new(
                error_code::PROMPT_ARG_MISSING,
                "debug prompt requires at least one of: trace_id, agent_id",
                "mcp",
                ErrorSourceKind::UserInput,
                false,
                Some("provide trace_id='<from_error_data>' or agent_id='agent-N'".to_string()),
            ));
        }

        let description = format!(
            "Diagnose failure: trace={} agent={}",
            trace_id.unwrap_or("<none>"),
            agent_id.unwrap_or("<none>")
        );

        let user_text = format!(
            "请按以下步骤排查失败:\n\n\
             1. 取 6 字段 Error 数据:\n\
                - code (e.g. WORKTREE_CONFLICT, AGENT_TIMEOUT, POLICY_DENIED)\n\
                - message / source_module / source_kind / retriable / hint\n\
             2. 校验 trace_id: {}\n\
             3. 校验 agent_id: {} → 调 `resources/read agent://{}/state`\n\
             4. 跑 `get_pipeline_status(pipeline_run_id=...)`\n\
             5. 检查 lease_expires_at(per ADR-0030) — 若过期, 走 resume\n\
             6. 给出恢复建议(per retriable 字段决定 retry vs manual fix)\n\n\
             _mock: true(Phase E stub)",
            trace_id.unwrap_or("N/A"),
            agent_id.unwrap_or("N/A"),
            agent_id.unwrap_or("N/A")
        );

        let assistant_text = "Phase E stub: 即将串行调 3 个 read + 1 个 resources/read, 按 6 字段 Error 决策路径。".to_string();

        Ok(prompt_messages(&description, &user_text, &assistant_text))
    }
}

/// 必需字符串参数取值 + 缺失时返回 PROMPT_ARG_MISSING
fn require_string_arg(args: &Value, name: &str) -> Result<String, McpError> {
    args.get(name)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| {
            McpError::new(
                error_code::PROMPT_ARG_MISSING,
                format!("missing required argument '{name}' (string)"),
                "mcp",
                ErrorSourceKind::UserInput,
                false,
                Some(format!("provide {name}=<value> in arguments")),
            )
        })
}

/// 包装为 MCP `prompts/get` 标准响应 `{ description, messages: [...] }`
fn prompt_messages(description: &str, user_text: &str, assistant_text: &str) -> Value {
    json!({
        "description": description,
        "messages": [
            {
                "role": "user",
                "content": {
                    "type": "text",
                    "text": user_text
                }
            },
            {
                "role": "assistant",
                "content": {
                    "type": "text",
                    "text": assistant_text
                }
            }
        ]
    })
}

// ===== JSON-RPC 2.0 入口(per Phase D.5+ 接口契约, 委托给 handler) =====

/// 处理 `prompts/list` 请求 — 返回 5 个 prompt 模板
pub(crate) fn handle_prompts_list(req: &JsonRpcRequest) -> Result<JsonRpcSuccess, JsonRpcError> {
    let handler = PromptsHandler::new();
    let prompts = handler.list();
    let result = json!({ "prompts": prompts });
    Ok(JsonRpcSuccess { jsonrpc: "2.0", id: req.id.clone(), result })
}

/// 处理 `prompts/get` 请求
///
/// 期望 params = { "name": "<prompt_name>", "arguments": {...} }
pub(crate) async fn handle_prompts_get(req: &JsonRpcRequest) -> Result<JsonRpcSuccess, JsonRpcError> {
    let name = req.params.get("name").and_then(Value::as_str).ok_or_else(|| {
        JsonRpcError {
            jsonrpc: "2.0",
            id: req.id.clone(),
            error: JsonRpcErrorBody {
                code: crate::transport::error_code::INVALID_PARAMS,
                message: "missing 'name' in params".to_string(),
                data: None,
            },
        }
    })?;
    let arguments = req
        .params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));

    let handler = PromptsHandler::new();
    match handler.get(name, &arguments).await {
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

    fn handler() -> PromptsHandler {
        PromptsHandler::new()
    }

    // ===== 1. submit =====

    #[tokio::test]
    async fn test_get_submit_with_worktree() {
        let h = handler();
        let v = h
            .get("submit", &json!({ "worktree_id": "wt-STAR-1024", "force": false }))
            .await
            .unwrap();
        let desc = v.get("description").unwrap().as_str().unwrap();
        assert!(desc.contains("wt-STAR-1024"));
        let messages = v.get("messages").unwrap().as_array().unwrap();
        assert_eq!(messages.len(), 2, "user + assistant");
        assert_eq!(messages[0].get("role").unwrap().as_str().unwrap(), "user");
        assert_eq!(messages[1].get("role").unwrap().as_str().unwrap(), "assistant");
        let user_text = messages[0].get("content").unwrap().get("text").unwrap().as_str().unwrap();
        assert!(user_text.contains("Universal Submit"), "must reference spec/flows/05");
        assert!(user_text.contains("_mock: true"), "must be marked mock per Phase E");
    }

    #[tokio::test]
    async fn test_get_submit_without_worktree_uses_placeholder() {
        // submit 允许 worktree_id 缺省(用户后续补充)
        let h = handler();
        let v = h.get("submit", &json!({})).await.unwrap();
        let desc = v.get("description").unwrap().as_str().unwrap();
        assert!(desc.contains("<unset"));
    }

    // ===== 2. review =====

    #[tokio::test]
    async fn test_get_review_requires_mr_id() {
        let h = handler();
        let err = h.get("review", &json!({})).await.unwrap_err();
        assert_eq!(err.code, error_code::PROMPT_ARG_MISSING);
        assert!(err.hint.is_some());
    }

    #[tokio::test]
    async fn test_get_review_with_mr_id() {
        let h = handler();
        let v = h
            .get(
                "review",
                &json!({ "mr_id": "mr-001", "reviewers": ["alice", "bob"] }),
            )
            .await
            .unwrap();
        let desc = v.get("description").unwrap().as_str().unwrap();
        assert!(desc.contains("mr-001"));
        assert!(desc.contains("alice, bob"));
    }

    // ===== 3. context =====

    #[tokio::test]
    async fn test_get_context_requires_at_least_one_ref() {
        let h = handler();
        let err = h.get("context", &json!({})).await.unwrap_err();
        assert_eq!(err.code, error_code::PROMPT_ARG_MISSING);
    }

    #[tokio::test]
    async fn test_get_context_with_issue_only() {
        let h = handler();
        let v = h.get("context", &json!({ "issue_id": "STAR-1024" })).await.unwrap();
        let messages = v.get("messages").unwrap().as_array().unwrap();
        let user_text = messages[0].get("content").unwrap().get("text").unwrap().as_str().unwrap();
        assert!(user_text.contains("STAR-1024"));
        assert!(user_text.contains("get_issue"));
    }

    #[tokio::test]
    async fn test_get_context_with_both_refs() {
        let h = handler();
        let v = h
            .get(
                "context",
                &json!({ "issue_id": "STAR-1024", "worktree_id": "wt-STAR-1024" }),
            )
            .await
            .unwrap();
        let desc = v.get("description").unwrap().as_str().unwrap();
        assert!(desc.contains("STAR-1024"));
        assert!(desc.contains("wt-STAR-1024"));
    }

    // ===== 4. workflow =====

    #[tokio::test]
    async fn test_get_workflow_valid_domain() {
        let h = handler();
        for domain in ["player", "economy", "match", "social", "admin"] {
            let v = h.get("workflow", &json!({ "domain": domain })).await.unwrap();
            let desc = v.get("description").unwrap().as_str().unwrap();
            assert!(desc.contains(domain));
        }
    }

    #[tokio::test]
    async fn test_get_workflow_invalid_domain_rejected() {
        let h = handler();
        let err = h.get("workflow", &json!({ "domain": "unknown" })).await.unwrap_err();
        assert_eq!(err.code, error_code::USER_INPUT);
        assert!(err.message.contains("unknown"));
        assert!(err.hint.is_some(), "must suggest valid domains");
    }

    #[tokio::test]
    async fn test_get_workflow_missing_domain() {
        let h = handler();
        let err = h.get("workflow", &json!({})).await.unwrap_err();
        assert_eq!(err.code, error_code::PROMPT_ARG_MISSING);
    }

    // ===== 5. debug =====

    #[tokio::test]
    async fn test_get_debug_with_trace_id() {
        let h = handler();
        let v = h
            .get("debug", &json!({ "trace_id": "trace-001" }))
            .await
            .unwrap();
        let messages = v.get("messages").unwrap().as_array().unwrap();
        let user_text = messages[0].get("content").unwrap().get("text").unwrap().as_str().unwrap();
        assert!(user_text.contains("trace-001"));
        assert!(user_text.contains("WORKTREE_CONFLICT") || user_text.contains("code"));
    }

    #[tokio::test]
    async fn test_get_debug_with_agent_id() {
        let h = handler();
        let v = h.get("debug", &json!({ "agent_id": "agent-1" })).await.unwrap();
        let user_text = v.get("messages").unwrap().as_array().unwrap()[0]
            .get("content").unwrap().get("text").unwrap().as_str().unwrap();
        assert!(user_text.contains("agent-1"));
        assert!(user_text.contains("agent://agent-1/state"), "must reference resources/read URI");
    }

    #[tokio::test]
    async fn test_get_debug_requires_at_least_one_clue() {
        let h = handler();
        let err = h.get("debug", &json!({})).await.unwrap_err();
        assert_eq!(err.code, error_code::PROMPT_ARG_MISSING);
    }

    // ===== 错误路径 =====

    #[tokio::test]
    async fn test_get_unknown_prompt_rejected() {
        let h = handler();
        let err = h.get("nonexistent", &json!({})).await.unwrap_err();
        assert_eq!(err.code, error_code::PROMPT_NOT_FOUND);
        assert!(err.hint.is_some());
    }

    // ===== list 入口 =====

    #[tokio::test]
    async fn test_list_returns_5_prompts() {
        let h = handler();
        let prompts = h.list();
        assert_eq!(prompts.len(), 5, "5 core prompts per Phase E");
        let names: Vec<&str> = prompts
            .iter()
            .map(|p| p.get("name").unwrap().as_str().unwrap())
            .collect();
        for required in ["submit", "review", "context", "workflow", "debug"] {
            assert!(names.contains(&required), "missing prompt: {required}");
        }
    }

    #[tokio::test]
    async fn test_list_includes_arguments_schema() {
        let h = handler();
        let prompts = h.list();
        for p in &prompts {
            let name = p.get("name").unwrap().as_str().unwrap();
            let args = p.get("arguments").unwrap().as_array().unwrap();
            assert!(!args.is_empty(), "prompt '{name}' must declare at least one argument");
            for arg in args {
                assert!(arg.get("name").is_some());
                assert!(arg.get("type").is_some());
            }
        }
    }

    // ===== JSON-RPC 入口测试 =====

    #[tokio::test]
    async fn test_handle_prompts_list_jsonrpc_entry() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(1),
            method: "prompts/list".to_string(),
            params: json!({}),
        };
        let res = handle_prompts_list(&req).unwrap();
        let arr = res.result.get("prompts").unwrap().as_array().unwrap();
        assert_eq!(arr.len(), 5);
    }

    #[tokio::test]
    async fn test_handle_prompts_get_jsonrpc_entry() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(2),
            method: "prompts/get".to_string(),
            params: json!({
                "name": "submit",
                "arguments": { "worktree_id": "wt-test" }
            }),
        };
        let res = handle_prompts_get(&req).await.unwrap();
        let messages = res.result.get("messages").unwrap().as_array().unwrap();
        assert_eq!(messages.len(), 2);
    }

    #[tokio::test]
    async fn test_handle_prompts_get_missing_name() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(3),
            method: "prompts/get".to_string(),
            params: json!({}),
        };
        let res = handle_prompts_get(&req).await;
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert_eq!(err.error.code, crate::transport::error_code::INVALID_PARAMS);
    }

    #[tokio::test]
    async fn test_handle_prompts_get_error_data_envelope() {
        // per spec/mcp/01 §3.2: error.data 应含 6 字段 agent-api/v1#Error
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(4),
            method: "prompts/get".to_string(),
            params: json!({ "name": "review" }), // 缺 mr_id
        };
        let res = handle_prompts_get(&req).await;
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(err.error.data.is_some());
        let data = err.error.data.unwrap();
        assert_eq!(data["code"], "PROMPT_ARG_MISSING");
        assert_eq!(data["source_kind"], "user_input");
    }
}
