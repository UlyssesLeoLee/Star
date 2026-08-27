#![warn(missing_docs)]

//! MCP tool stub: get_workspace
//!
//! per `docs/architecture/2026-08-26-upgrade/spec/mcp/01-mcp-spec.md` §2
//!
//! ## Phase D
//!
//! - 输入:`{workspace_id?: "<id>"}`
//! - 输出:`agent-api/v1#Workspace` mock(per `spec/ide-api/01-schema.md` §2.1)

use serde_json::{Value, json};

use crate::error::McpError;
use crate::tools::{mock_response, optional_string};

/// `get_workspace` tool stub
pub(crate) async fn invoke(args: Value) -> Result<Value, McpError> {
    let ws_id = optional_string(&args, "workspace_id").unwrap_or_else(|| "ws-default".to_string());
    let body = json!({
        "workspace": {
            "id": ws_id,
            "name": "main-workspace",
            "repository": {
                "id": "repo-1",
                "provider": "gitgit",
                "url": "https://github.com/UlyssesLeoLee/Star"
            },
            "worktree_id": "wt-STAR-1024",
            "open_files": [],
            "active_symbol": null,
            "diagnostics": [],
            "ide_client": "vscode",
            "ide_version": "1.95.0",
        }
    });
    Ok(mock_response("get_workspace", body))
}
