#![warn(missing_docs)]

//! MCP tool stub: get_worktree
//!
//! per `docs/architecture/2026-08-26-upgrade/spec/mcp/01-mcp-spec.md` §2
//!
//! ## Phase D
//!
//! - 输入:`{worktree_id?: "<id>"}`
//! - 输出:`agent-api/v1#Worktree` mock(per `spec/agent-api/01-schema.md` §3.2)

use serde_json::{Value, json};

use crate::error::McpError;
use crate::tools::{mock_response, optional_string};

/// `get_worktree` tool stub
pub(crate) async fn invoke(args: Value) -> Result<Value, McpError> {
    let wt_id = optional_string(&args, "worktree_id").unwrap_or_else(|| "wt-STAR-1024".to_string());
    let body = json!({
        "worktree": {
            "id": wt_id,
            "path": format!("/repos/owner/repo/{wt_id}"),
            "branch": format!("feature/{wt_id}"),
            "head_commit": "0000000000000000000000000000000000000000",
            "dirty": false,
            "agent_session_id": "agent-mock",
            "ide_session_id": "ide-mock",
            "created_at": "2026-08-27T00:00:00Z",
        }
    });
    Ok(mock_response("get_worktree", body))
}
