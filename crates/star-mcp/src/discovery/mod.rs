//! # star-mcp AI 工具自动发现 (Tool Auto-Discovery)
//!
//! per `ADR-0049 AI 工具自动扫描 + 链接 拍板` (2026-09-08)
//! per `PHASE-LANGGRAPH-TMO-IMPL-REPORT.md` 类比
//!
//! 4 源扫描: `.mcp.json` / Claude / OpenAI / Cursor/Windsurf
//! 凭证: env_var passthrough (per 守门 #5)
//! 触发: 启动时自动 + 设置界面手动 (per 拍板 D-04)
//!
//! 守门合规:
//! - 守门 #5: 不打印 env 字段值, 仅引用
//! - 守门 #7: 0 unsafe
//! - 守门 #13 a: scanner 在 L0 启动层, 经 registry 注册后 L1 可调
//! - 守门 #10: author=Ulysses

pub(crate) mod auth_broker;
pub(crate) mod registry;
pub(crate) mod scanner;

pub(crate) use auth_broker::AuthBroker;
pub(crate) use registry::{ToolEntry, ToolRegistry};
pub(crate) use scanner::{ToolDiscoveryScanner, ToolSource, ToolSourceConfig};
