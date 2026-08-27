#![warn(missing_docs)]

//! MCP tool: request_review
//!
//! per `docs/architecture/2026-08-26-upgrade/spec/mcp/01-mcp-spec.md` §2
//!
//! ## Phase D
//!
//! - 输入:`{mr_id, reviewers?}`
//! - 输出:`agent-api/v1#Review` mock

use serde_json::{Value, json};

use crate::error::McpError;
use crate::tools::{mock_response, require_string};

/// `request_review` tool
pub(crate) async fn invoke(args: Value) -> Result<Value, McpError> {
    let mr_id = require_string(&args, "mr_id").map_err(McpError::BadRequest)?;
    let body = json!({
        "review": {
            "id": "REV-mock-001",
            "mr_id": mr_id,
            "status": "PENDING",
            "reviewers": args.get("reviewers").cloned().unwrap_or(json!([])),
        }
    });
    Ok(mock_response("request_review", body))
}