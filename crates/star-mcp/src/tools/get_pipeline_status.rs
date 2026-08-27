#![warn(missing_docs)]

//! MCP tool: get_pipeline_status
//!
//! per `docs/architecture/2026-08-26-upgrade/spec/mcp/01-mcp-spec.md` §2
//!
//! ## Phase D
//!
//! - 输入:`{pipeline_run_id}`
//! - 输出:`agent-api/v1#PipelineStatus` mock

use serde_json::{Value, json};

use crate::error::McpError;
use crate::tools::{mock_response, require_string};

/// `get_pipeline_status` tool
pub(crate) async fn invoke(args: Value) -> Result<Value, McpError> {
    let pipeline_run_id = require_string(&args, "pipeline_run_id").map_err(McpError::validation)?;
    let body = json!({
        "pipeline": {
            "id": pipeline_run_id,
            "status": "SUCCESS",
            "url": format!("https://example.invalid/pipelines/{pipeline_run_id}"),
        }
    });
    Ok(mock_response("get_pipeline_status", body))
}
