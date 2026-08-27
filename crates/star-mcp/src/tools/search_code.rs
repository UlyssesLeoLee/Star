#![warn(missing_docs)]

//! MCP tool stub: search_code
//!
//! per `docs/architecture/2026-08-26-upgrade/spec/mcp/01-mcp-spec.md` §2
//!
//! ## Phase D
//!
//! - 输入:`{query: "...", limit?: N, paths?: [...]}`
//! - 输出:`agent-api/v1#CodeSearchResult` mock

use serde_json::{Value, json};

use crate::error::McpError;
use crate::tools::{mock_response, optional_string, require_string};

/// `search_code` tool stub
pub(crate) async fn invoke(args: Value) -> Result<Value, McpError> {
    let query = require_string(&args, "query").map_err(McpError::BadRequest)?;
    let limit = args
        .get("limit")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(10);
    let _ = optional_string(&args, "paths"); // Phase D 忽略 paths
    let body = json!({
        "query": query,
        "total": 1,
        "results": [
            {
                "file": "crates/star-cli/src/commands/agent.rs",
                "line": 42,
                "snippet": format!("// match: {query}"),
            }
        ],
        "limit": limit,
    });
    Ok(mock_response("search_code", body))
}
