//! `sandboxd` crate (WBS 6.5.2 实装)
//!
//! **Sandbox-as-a-Service 独立 daemon** — gRPC IPC + 4 维隔离 (resource / network / fs /
//! capability) + session 池化 + 跨 client 复用 (per `SRS-SANDBOX-002` v0.1.2 +
//! `SANDBOX-BASIC-DESIGN-002` v0.1.2 + `DD-SANDBOX-002` v0.1.1 + `docs/architecture/SANDBOX-002.md` v0.1.1)。
//!
//! ## 组件划分 (per BD-002 §2.1)
//!
//! | 模块 | 文件 | 职责 |
//! |---|---|---|
//! | `config` | [`config`] | YAML 配置加载 + 校验 (FR-1.1) |
//! | `error` | [`error`] | 6-field 错误模型 (复用 star-mcp 模式) |
//! | `proto` | [`proto`] | tonic-build 生成 gRPC schema (FR-2.1) |
//! | `policy` | [`policy`] | 4 维策略 schema + 校验 + 匹配 (FR-3) |
//! | `backend` | [`backend`] | 跨平台隔离后端 trait + 工厂 + 三平台实装 (FR-5) |
//! | `orchestrator` | [`orchestrator`] | session 池化 + 生命周期 + 调度 (FR-1.2 / FR-4) |
//! | `server` | [`server`] | tonic gRPC server + 4 RPC handler (FR-2) |
//! | `metrics` | [`metrics`] | Prometheus 指标 (FR-6.1) |
//! | `health` | [`health`] | /healthz + /metrics HTTP endpoint (FR-6.3) |
//! | `audit` | [`audit`] | audit log 写入 (JSON Lines + PG, FR-6.2) |
//! | `client` | [`client`] | Rust gRPC client + fail-open 降级 (FR-7.1) |
//! | `logging` | [`logging`] | tracing 初始化 |
//!
//! ## 关键不变量
//!
//! - INV-SBX-01: 0 unsafe (per 守门 #7, workspace `unsafe_code = "forbid"`)
//! - INV-SBX-02: 4 维策略在 CreateSession 时冻结快照 (policy_snapshot), 后续 policy 变更不影响已创建 session
//! - INV-SBX-03: audit log 写失败 → fail-closed (BLOCK session 创建, per FR-7.2)
//! - INV-SBX-04: sandboxd 不可用 → client 走 SANDBOX-001 v0.2 subprocess 降级 (per FR-7.1)
//! - INV-SBX-05: env 内容不进 audit log / 日志 (仅 KEY 名, per 守门 #5)

#![forbid(unsafe_code)]
// 6-field 错误模型 (per DD §2.1) 有意携带诊断字符串, Err 体积换可观测性
#![allow(clippy::result_large_err)]

pub mod audit;
pub mod backend;
pub mod client;
pub mod config;
pub mod error;
pub mod health;
pub mod logging;
pub mod metrics;
pub mod orchestrator;
pub mod policy;
#[allow(missing_docs, unreachable_pub, rust_2018_idioms, clippy::all)]
pub mod proto {
    //! tonic-build 自动生成的 protobuf / gRPC schema (`sandboxd.v1`)。
    //!
    //! 本模块内容由 `build.rs` 从 `proto/sandboxd.proto` 生成 (per BD-002 §3.1)，
    //! 生成代码不参与 missing_docs / unreachable_pub / clippy 门禁。
    tonic::include_proto!("sandboxd.v1");
}
pub mod server;

/// crate 版本 (跟 healthz version 字段一致, per FR-6.3)
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod wire {
    //! `proto/sandboxd.proto` 生成类型的扁平重导出 (gRPC wire schema, per FR-2.1)。
    //!
    //! 注: tonic-build 0.12 生成扁平布局 (无 package 嵌套模块), service 模块以
    //! service 名命名 (`sandbox_service_client` / `sandbox_service_server`)。
    pub use crate::proto::{
        sandbox_service_client, sandbox_service_server, CapabilityPolicy, CommandSpec,
        CreateSessionRequest, CreateSessionResponse, DestroySessionRequest, DestroySessionResponse,
        FsPathRule, FsPolicy, LogEntry, NetworkPolicy, ResourcePolicy, RunCommandRequest,
        RunCommandResponse, SessionSpec, StreamLogsRequest,
    };
}
