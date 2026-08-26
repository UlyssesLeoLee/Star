#![warn(missing_docs)]

//! MCP tool stub: $t
//!
//! per docs/architecture/2026-08-26-upgrade/spec/mcp/01-mcp-spec.md 搂2
//!
//! Phase D 楠ㄦ灦:鍑芥暟浣?unimplemented!(),瀹屾暣瀹炵幇寰?Phase D.1銆?
use serde_json::Value;

use crate::error::McpError;

/// $docTitle tool stub
///
/// ## Phase D
///
/// - 鍑芥暟浣?unimplemented!() + // TODO Phase D.1
/// - 鎺ュ彈浠绘剰 serde_json::Value 浣滀负 args
/// - 鐪熷疄瀹炵幇寰?Phase D.1 琛ラ綈(per spec 搂2 杈撳叆/杈撳嚭 schema)
pub(crate) async fn invoke(_args: Value) -> Result<Value, McpError> {
    // TODO Phase D.1
    unimplemented!("MCP tool $t not implemented yet (Phase D.1)")
}
