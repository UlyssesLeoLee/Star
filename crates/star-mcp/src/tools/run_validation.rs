#![warn(missing_docs)]

//! MCP tool: run_validation
//!
//! per `docs/architecture/2026-08-26-upgrade/spec/mcp/01-mcp-spec.md` §2
//!
//! ## Phase D
//!
//! - 输入:`{worktree_id?}`
//! - 输出:`agent-api/v1#ValidationResult` mock

use serde_json::{json, Value};

use crate::error::McpError;
use crate::tools::mock_response;

/// `run_validation` tool
pub(crate) async fn invoke(_args: Value) -> Result<Value, McpError> {
    let body = json!({
        "validation": {
            "passed": 0,
            "failed": 0,
            "skipped": 0,
            "failed_tests": [],
        }
    });
    Ok(mock_response("run_validation", body))
}
