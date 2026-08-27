#![warn(missing_docs)]

//! MCP tool stub: get_context
//!
//! per `docs/architecture/2026-08-26-upgrade/spec/mcp/01-mcp-spec.md` §2
//!
//! ## Phase D
//!
//! - 输入:`{issue_id: "<id>"}`
//! - 输出:`agent-api/v1#Context` mock

use serde_json::{Value, json};

use crate::error::McpError;
use crate::tools::{mock_response, require_string};

/// `get_context` tool stub
pub(crate) async fn invoke(args: Value) -> Result<Value, McpError> {
    let issue_id = require_string(&args, "issue_id").map_err(McpError::BadRequest)?;
    let body = json!({
        "context": {
            "issue_id": issue_id,
            "linked_files": [
                "docs/architecture/2026-08-26-upgrade/spec/flows/05-universal-submit.md",
                "crates/star-cli/src/commands/submit.rs"
            ],
            "linked_specs": [
                "arch/03-star-ai-compat-arch.md",
                "arch/04-star-ide-gateway-arch.md"
            ],
            "linked_mrs": [],
            "summary": format!("Phase D mock context for issue {issue_id}"),
        }
    });
    Ok(mock_response("get_context", body))
}
