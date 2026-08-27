//! `star-mcp` Prompts 能力(per 2025-06-27 MCP spec §4)
//!
//! Phase D.5+ 实装:
//! - `prompts/list`: 返回 0 个 prompt(MVP 暂不实装 prompt 模板)
//! - `prompts/get`: mock 返回错误(MVP 不可用)
//!
//! ## 守门规则
//!
//! - 0 unsafe
//! - MVP 故意返回 0 prompts, 避免编造未实装的 prompt
//! - 缺标比错标安全(per 8/27 11:09 拍板)

#![warn(missing_docs)]

use serde_json::json;

use crate::transport::{JsonRpcError, JsonRpcErrorBody, JsonRpcRequest, JsonRpcSuccess, error_code};

/// 处理 `prompts/list` 请求
///
/// MVP: 返回 0 个 prompt(per Phase D.5+ 范围, prompt 模板留 Phase D.6+)
pub(crate) fn handle_prompts_list(req: &JsonRpcRequest) -> Result<JsonRpcSuccess, JsonRpcError> {
    // 故意返回空数组, 不编造任何 prompt
    let result = json!({ "prompts": [] });
    Ok(JsonRpcSuccess { jsonrpc: "2.0", id: req.id.clone(), result })
}

/// 处理 `prompts/get` 请求
///
/// MVP: 不可用(per Phase D.5+ 范围, 实际 prompt 内容留 Phase D.6+)
pub(crate) fn handle_prompts_get(req: &JsonRpcRequest) -> Result<JsonRpcSuccess, JsonRpcError> {
    // 期望 params = { "name": "<prompt_name>", "arguments": {...} }
    // MVP 不接受任何 prompt, 一律返回 -32601 method not found
    // 注: 这里故意使用 METHOD_NOT_FOUND 而非 INVALID_PARAMS, 因为 prompts/get 当前未实装
    let _ = req.params.get("name");
    Err(JsonRpcError {
        jsonrpc: "2.0",
        id: req.id.clone(),
        error: JsonRpcErrorBody {
            code: error_code::METHOD_NOT_FOUND,
            message: "prompts/get is not implemented in MVP (per Phase D.5+ scope)".to_string(),
            data: None,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_prompts_list_returns_zero() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(1),
            method: "prompts/list".to_string(),
            params: json!({}),
        };
        let res = handle_prompts_list(&req).unwrap();
        let prompts = res.result.get("prompts").unwrap().as_array().unwrap();
        assert_eq!(prompts.len(), 0, "MVP returns 0 prompts (per Phase D.5+ scope)");
    }

    #[tokio::test]
    async fn test_prompts_get_returns_method_not_found() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(2),
            method: "prompts/get".to_string(),
            params: json!({ "name": "submit_pr" }),
        };
        let res = handle_prompts_get(&req);
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert_eq!(err.error.code, error_code::METHOD_NOT_FOUND);
        assert!(err.error.message.contains("not implemented"));
    }
}
