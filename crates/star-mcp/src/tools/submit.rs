#![warn(missing_docs)]

//! MCP tool: submit
//!
//! per `docs/architecture/2026-08-26-upgrade/spec/mcp/01-mcp-spec.md` §2 (per P1-F)
//! per `docs/architecture/2026-08-26-upgrade/spec/flows/05-universal-submit.md` 12 步
//!
//! ## Phase D
//!
//! - 输入:`{worktree_id?, force?}`
//! - 输出:`agent-api/v1#SubmitResult` mock

use serde_json::{Value, json};

use crate::error::McpError;
use crate::tools::mock_response;

/// `submit` tool
pub(crate) async fn invoke(_args: Value) -> Result<Value, McpError> {
    let body = json!({
        "status": "OK",
        "commit_sha": "deadbeef0000000000000000000000000000000",
        "mr_id": "MR-mock-001",
        "pipeline_run_id": "PIPE-mock-001",
        "validation_passed": true,
        "policy_checked": true,
    });
    Ok(mock_response("submit", body))
}