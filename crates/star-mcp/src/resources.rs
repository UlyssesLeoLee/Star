//! `star-mcp` Resources 能力(per 2025-06-27 MCP spec §3)
//!
//! Phase D.5+ 实装:
//! - `resources/list`: 暴露 16 tool 资源(URI = `star://tools/<name>`)
//! - `resources/read`: 读取资源(mock-but-functional, 返回工具描述 JSON)
//!
//! ## 守门规则
//!
//! - 0 unsafe
//! - 复用 `transport::tools_list` 现有 16 tool inputSchema, 不重复维护
//! - 资源 read 走 `tools::invoke` 同款 mock 数据

#![warn(missing_docs)]

use serde_json::{Value, json};

use crate::transport::{JsonRpcError, JsonRpcErrorBody, JsonRpcRequest, JsonRpcSuccess, error_code, tools_list};

/// 资源 URI 前缀(per MCP 2025-06-27 spec §3.1 URI scheme)
pub(crate) const RESOURCE_URI_PREFIX: &str = "star://tools/";

/// 处理 `resources/list` 请求
///
/// 返回 16 tool 资源列表(URI = `star://tools/<tool_name>`, mimeType = `application/json`)
pub(crate) fn handle_resources_list(req: &JsonRpcRequest) -> Result<JsonRpcSuccess, JsonRpcError> {
    // 复用 tools_list 的 16 tool 数据, 但包装为 resources 字段
    let tools_obj = tools_list();
    let tools_arr = tools_obj
        .get("tools")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            // 理论上不会发生: tools_list 总是返回 {"tools": [...]}
            err_internal("tools_list() did not return a 'tools' array", &req.id)
        })?;

    let resources: Vec<Value> = tools_arr
        .iter()
        .map(|t| {
            let name = t.get("name").and_then(Value::as_str).unwrap_or("");
            let description = t.get("description").and_then(Value::as_str).unwrap_or("");
            json!({
                "uri": format!("{RESOURCE_URI_PREFIX}{name}"),
                "name": name,
                "description": description,
                "mimeType": "application/json"
            })
        })
        .collect();

    let result = json!({ "resources": resources });
    Ok(JsonRpcSuccess { jsonrpc: "2.0", id: req.id.clone(), result })
}

/// 处理 `resources/read` 请求
///
/// 期望 params = { "uri": "star://tools/<name>" }
/// 返回 mock JSON (per spec/agent-api/v1 schema) 通过 text content
pub(crate) fn handle_resources_read(req: &JsonRpcRequest) -> Result<JsonRpcSuccess, JsonRpcError> {
    let uri = req
        .params
        .get("uri")
        .and_then(Value::as_str)
        .ok_or_else(|| err_invalid_params("missing 'uri' in params", &req.id))?;

    // 校验 prefix
    let tool_name = uri.strip_prefix(RESOURCE_URI_PREFIX).ok_or_else(|| {
        err_invalid_params(
            format!("uri must start with '{RESOURCE_URI_PREFIX}', got: {uri}").as_str(),
            &req.id,
        )
    })?;

    // 复用 tools_list 校验 tool 是否存在(16 tool 之一)
    let tools_obj = tools_list();
    let tools_arr = tools_obj.get("tools").and_then(Value::as_array).ok_or_else(|| {
        err_internal("tools_list() did not return a 'tools' array", &req.id)
    })?;

    let matched = tools_arr.iter().find(|t| {
        t.get("name").and_then(Value::as_str) == Some(tool_name)
    });

    let tool_value = match matched {
        Some(t) => t.clone(),
        None => {
            return Err(err_invalid_params(
                format!("unknown resource uri: {uri}").as_str(),
                &req.id,
            ));
        }
    };

    // 返回 resources/read 标准格式
    // per MCP 2025-06-27 spec §3.2:
    //   contents: [{ uri, mimeType, text }]
    let text = serde_json::to_string_pretty(&tool_value).unwrap_or_else(|_| "{}".to_string());
    let result = json!({
        "contents": [
            {
                "uri": uri,
                "mimeType": "application/json",
                "text": text
            }
        ]
    });
    Ok(JsonRpcSuccess { jsonrpc: "2.0", id: req.id.clone(), result })
}

// 错误构造 helpers (与 transport.rs 保持一致)
fn err_invalid_params(msg: &str, id: &Value) -> JsonRpcError {
    JsonRpcError {
        jsonrpc: "2.0",
        id: id.clone(),
        error: JsonRpcErrorBody {
            code: error_code::INVALID_PARAMS,
            message: msg.to_string(),
            data: None,
        },
    }
}

fn err_internal(msg: &str, id: &Value) -> JsonRpcError {
    JsonRpcError {
        jsonrpc: "2.0",
        id: id.clone(),
        error: JsonRpcErrorBody {
            code: error_code::INTERNAL_ERROR,
            message: msg.to_string(),
            data: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_resources_list_returns_16() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(1),
            method: "resources/list".to_string(),
            params: json!({}),
        };
        let res = handle_resources_list(&req).unwrap();
        let resources = res.result.get("resources").unwrap().as_array().unwrap();
        assert_eq!(resources.len(), 16, "16 tool resources per P1-F + submit");

        // 验证 URI 前缀 + name
        let first = &resources[0];
        assert_eq!(
            first.get("uri").unwrap().as_str().unwrap(),
            "star://tools/get_issue"
        );
        assert_eq!(first.get("name").unwrap().as_str().unwrap(), "get_issue");
        assert_eq!(first.get("mimeType").unwrap().as_str().unwrap(), "application/json");
    }

    #[tokio::test]
    async fn test_resources_read_known_tool() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(2),
            method: "resources/read".to_string(),
            params: json!({ "uri": "star://tools/submit" }),
        };
        let res = handle_resources_read(&req).unwrap();
        let contents = res.result.get("contents").unwrap().as_array().unwrap();
        assert_eq!(contents.len(), 1);
        let item = &contents[0];
        assert_eq!(item.get("uri").unwrap().as_str().unwrap(), "star://tools/submit");
        let text = item.get("text").unwrap().as_str().unwrap();
        // text 是 tool 描述 JSON, 必含 name + inputSchema
        let parsed: Value = serde_json::from_str(text).unwrap();
        assert_eq!(parsed.get("name").unwrap().as_str().unwrap(), "submit");
        assert!(parsed.get("inputSchema").is_some());
    }

    #[tokio::test]
    async fn test_resources_read_unknown_uri() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(3),
            method: "resources/read".to_string(),
            params: json!({ "uri": "star://tools/nonexistent" }),
        };
        let res = handle_resources_read(&req);
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert_eq!(err.error.code, error_code::INVALID_PARAMS);
    }

    #[tokio::test]
    async fn test_resources_read_wrong_prefix() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(4),
            method: "resources/read".to_string(),
            params: json!({ "uri": "http://example.com/foo" }),
        };
        let res = handle_resources_read(&req);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().error.code, error_code::INVALID_PARAMS);
    }

    #[tokio::test]
    async fn test_resources_read_missing_uri() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(5),
            method: "resources/read".to_string(),
            params: json!({}),
        };
        let res = handle_resources_read(&req);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().error.code, error_code::INVALID_PARAMS);
    }
}
