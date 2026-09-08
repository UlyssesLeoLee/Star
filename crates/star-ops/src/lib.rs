// SPDX-License-Identifier: MIT OR Apache-2.0
//! `crates/star-ops` — STAR Ops Console (MVP-骨架)
//!
//! per `docs/requirements/SRS-STAR-OPS-001.md` v0.1
//! per `docs/basic-design/OPS-BASIC-DESIGN-001.md` v0.1
//!
//! ## 范围 (MVP-骨架 — 仅 routing 形状 + 8 stub, 不接业务逻辑)
//!
//! - **4 tab** (集群更新 / log AI / 运维数据 / 文档) — frontend `/ops` 路由占位
//! - **8 REST stub 端点** — 全部 `501 NOT_IMPLEMENTED` + 真实契约可见
//! - **3 子域** (cluster / log / metrics) — 各 1 数据结构 + stub 方法
//! - **AI 通道 (Hybrid)** — mock (subprocess 调 `scripts/automation/ai_log_mock.py`)
//!   + OpenAI stub + Anthropic stub + Fallback Ladder 4 级 (per ADR-0026 §2.2)
//!
//! ## 部署 (MVP)
//!
//! - axum 0.8 binary, 端口 8090 (per star-mcp 8080/8081 顺延)
//! - 单实例, K8s deployment YAML 在 OPS-BASIC-DESIGN §6.1
//! - 健康检查: `GET /healthz` + `GET /readyz`
//!
//! ## 守门规则 (per AGENTS.md §4.1 累积规)
//!
//! - 0 unsafe (守门 #7)
//! - `cargo check --workspace --all-targets -j 4` 0 err (守门 #1 v19)
//! - `cargo test -p star-ops --lib -j 4` 100% pass (守门 #1 v25 单 crate)
//! - `cargo fmt --all -- --check` 0 err
//! - `cargo clippy --workspace --all-targets -- -D warnings` 0 err (advisory 守门 #7 v3)
//!
//! ## 不做什么 (per SRS-001 §3.2)
//!
//! - **不**实装 4 类功能端到端 (F-01..F-04) — 等 [M] 子项拍板
//! - **不**实装 K8s/Helm client (R-05 守门)
//! - **不**实装真实 OpenAI/Anthropic 调用 — 仅 stub (守门 #23)
//! - **不**实装持久化 — MVP 内存 + JSON stub (守门 #13 W/T/M 表结构先定)
//! - **不**实装跨域编排 — 守门 #3 5 域独立 Lead 硬约束

#![allow(missing_docs)] // 骨架阶段, 业务端点 [M] 子项实装时补

pub mod error;
pub mod ops_ai;
pub mod ops_api;
pub mod ops_domain;

pub use error::{ErrorSourceKind, OpsError, OpsErrorBody, OpsErrorResponse};
