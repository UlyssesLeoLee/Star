#![warn(missing_docs)]

//! MCP tool stub: search_issues
//!
//! per `docs/architecture/2026-08-26-upgrade/spec/mcp/01-mcp-spec.md` §2
//!
//! ## Phase D
//!
//! - 输入:`{query: "...", filters?: {...}}`
//! - 输出:`agent-api/v1#IssueList` mock(2 条)

use serde_json::{json, Value};

use crate::error::McpError;
use crate::tools::{mock_response, require_string};

/// `search_issues` tool stub
pub(crate) async fn invoke(args: Value) -> Result<Value, McpError> {
    let query = require_string(&args, "query").map_err(McpError::validation)?;
    let body = json!({
        "query": query,
        "total": 2,
        "issues": [
            {
                "id": "ISSUE-1",
                "title": format!("Mock match for '{query}' #1"),
                "status": "OPEN",
                "labels": ["mock"],
            },
            {
                "id": "ISSUE-2",
                "title": format!("Mock match for '{query}' #2"),
                "status": "IN_PROGRESS",
                "labels": ["mock"],
            }
        ],
    });
    Ok(mock_response("search_issues", body))
}
