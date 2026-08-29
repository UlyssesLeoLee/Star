#![warn(missing_docs)]

//! MCP tool stub: get_code_context
//!
//! per `docs/architecture/2026-08-26-upgrade/spec/mcp/01-mcp-spec.md` §2
//!
//! ## Phase D
//!
//! - 输入:`{file: "...", range: [start, end]}`
//! - 输出:`agent-api/v1#CodeContext` mock

use serde_json::{json, Value};

use crate::error::McpError;
use crate::tools::mock_response;

/// `get_code_context` tool stub
pub(crate) async fn invoke(args: Value) -> Result<Value, McpError> {
    let file = args
        .get("file")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| McpError::validation("missing 'file'".to_string()))?;
    let range = args
        .get("range")
        .cloned()
        .unwrap_or_else(|| serde_json::json!([0, 0]));
    let body = json!({
        "file": file,
        "range": range,
        "context": format!("// mock context for {file} (range = {range})"),
    });
    Ok(mock_response("get_code_context", body))
}
