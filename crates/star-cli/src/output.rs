//! `--json` 输出守门(per `docs/.../spec/cli/01-cli-spec.md` §1 / §3)
//!
//! 即使 mock 也走 `agent-api/v1` 稳定 schema,确保后续 Phase D.1 替换
//! mock 实现时 schema 字段不破坏下游 agent 解析。
//!
//! ## 守门原则
//!
//! - 所有 `serde_json::Value` 输出都过 `to_string_pretty` 而不是 `to_string`,
//!   保证 shell 解析 + diff 友好
//! - 顶层始终含 `schema_version: "agent-api/v1"`
//! - 不在 lib 层做 ANSI color(mvp 不需要 `--no-color` 分支)

use serde::Serialize;

use crate::error::StarError;

/// `agent-api/v1` schema 守门标记(per `spec/agent-api/01-schema.md` §1)
pub(crate) const SCHEMA_VERSION: &str = "agent-api/v1";

/// 把任何 `Serialize` 值序列化为 pretty JSON(平铺,不带 envelope)
///
/// 主要给 schema 本身就是根级 `Capabilities` 的命令用
/// (per `spec/acceptance/12-capability-discovery.md` §3 例子,Capabilities 是平铺对象)
pub(crate) fn json_pretty<T: Serialize>(data: T) -> Result<String, StarError> {
    let v = serde_json::to_value(data)?;
    Ok(serde_json::to_string_pretty(&v)?)
}
