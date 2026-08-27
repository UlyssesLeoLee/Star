#![warn(missing_docs)]

//! MCP tool: create_merge_request
//!
//! per `docs/architecture/2026-08-26-upgrade/spec/mcp/01-mcp-spec.md` §2
//!
//! ## Phase D
//!
//! - 输入:`{title, description, base, head}`
//! - 输出:`agent-api/v1#MR` mock

use serde_json::{Value, json};

use crate::error::McpError;
use crate::tools::{mock_response, require_string};

/// `create_merge_request` tool
pub(crate) async fn invoke(args: Value) -> Result<Value, McpError> {
    let title = require_string(&args, "title").map_err(McpError::validation)?;
    let base = require_string(&args, "base").map_err(McpError::validation)?;
    let head = require_string(&args, "head").map_err(McpError::validation)?;
    let body = json!({
        "mr": {
            "id": "MR-mock-001",
            "title": title,
            "status": "OPEN",
            "source_branch": head,
            "target_branch": base,
            "url": "https://example.invalid/mr/MR-mock-001".to_string(),
        }
    });
    Ok(mock_response("create_merge_request", body))
}
