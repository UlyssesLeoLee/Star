#![warn(missing_docs)]

//! MCP tool stub: get_issue
//!
//! per `docs/architecture/2026-08-26-upgrade/spec/mcp/01-mcp-spec.md` §2
//!
//! ## Phase D
//!
//! - 输入:`{issue_id: "<id>"}`
//! - 输出:`agent-api/v1#Issue` mock

use serde_json::{Value, json};

use crate::error::McpError;
use crate::tools::{mock_response, require_string};

/// `get_issue` tool stub
pub(crate) async fn invoke(args: Value) -> Result<Value, McpError> {
    let issue_id = require_string(&args, "issue_id").map_err(McpError::BadRequest)?;
    let body = json!({
        "issue": {
            "id": issue_id,
            "title": format!("Mock issue {issue_id}"),
            "status": "OPEN",
            "priority": "MEDIUM",
            "labels": ["mock"],
            "assignee": null,
            "created_at": "2026-08-27T00:00:00Z",
            "updated_at": "2026-08-27T00:00:00Z",
        }
    });
    Ok(mock_response("get_issue", body))
}
