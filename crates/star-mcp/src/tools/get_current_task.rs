#![warn(missing_docs)]

//! MCP tool stub: get_current_task
//!
//! per `docs/architecture/2026-08-26-upgrade/spec/mcp/01-mcp-spec.md` §2
//!
//! ## Phase D
//!
//! - 输入:`{}`
//! - 输出:`agent-api/v1#Task` mock

use serde_json::{Value, json};

use crate::error::McpError;
use crate::tools::mock_response;

/// `get_current_task` tool stub
pub(crate) async fn invoke(_args: Value) -> Result<Value, McpError> {
    let body = json!({
        "task": {
            "id": "STAR-1024",
            "title": "Phase D 骨架 — STAR CLI / MCP / Context 三 crate 落地",
            "status": "IN_PROGRESS",
            "assigned_to": "agent-mock",
            "context_refs": ["DEC-008", "arch/03-star-ai-compat-arch.md"],
            "labels": ["phase-d", "skeleton", "mvp"],
            "updated_at": "2026-08-27T00:00:00Z",
        }
    });
    Ok(mock_response("get_current_task", body))
}
