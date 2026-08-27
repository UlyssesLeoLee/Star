#![warn(missing_docs)]

//! MCP tool stub: get_symbol
//!
//! per `docs/architecture/2026-08-26-upgrade/spec/mcp/01-mcp-spec.md` §2
//!
//! ## Phase D
//!
//! - 输入:`{name: "<symbol>", file?: "..."}`
//! - 输出:`agent-api/v1#Symbol` mock

use serde_json::{Value, json};

use crate::error::McpError;
use crate::tools::{mock_response, optional_string, require_string};

/// `get_symbol` tool stub
pub(crate) async fn invoke(args: Value) -> Result<Value, McpError> {
    let name = require_string(&args, "name").map_err(McpError::BadRequest)?;
    let file = optional_string(&args, "file")
        .unwrap_or_else(|| "crates/star-cli/src/commands/agent.rs".to_string());
    let body = json!({
        "symbol": {
            "kind": "function",
            "name": name,
            "file": file,
            "line": 1,
            "signature": format!("fn {name}(...) -> _"),
        }
    });
    Ok(mock_response("get_symbol", body))
}
