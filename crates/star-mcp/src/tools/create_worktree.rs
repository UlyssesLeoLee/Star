#![warn(missing_docs)]

//! MCP tool stub: create_worktree
//!
//! per `docs/architecture/2026-08-26-upgrade/spec/mcp/01-mcp-spec.md` §2
//!
//! ## Phase D
//!
//! - 输入:`{issue_id: "<id>", branch_name?: "..."}`
//! - 输出:`agent-api/v1#Worktree` mock

use serde_json::{Value, json};

use crate::error::McpError;
use crate::tools::{mock_response, optional_string, require_string};

/// `create_worktree` tool stub
pub(crate) async fn invoke(args: Value) -> Result<Value, McpError> {
    let issue_id = require_string(&args, "issue_id").map_err(McpError::validation)?;
    let branch =
        optional_string(&args, "branch_name").unwrap_or_else(|| format!("feature/{issue_id}"));
    let wt_id = format!("wt-{issue_id}");
    let body = json!({
        "worktree": {
            "id": wt_id,
            "path": format!("/repos/owner/repo/{wt_id}"),
            "branch": branch,
            "head_commit": "0000000000000000000000000000000000000000",
            "dirty": true,
            "agent_session_id": "agent-mock",
            "ide_session_id": "ide-mock",
            "created_at": "2026-08-27T00:00:00Z",
        }
    });
    Ok(mock_response("create_worktree", body))
}
