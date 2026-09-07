// SPDX-License-Identifier: MIT OR Apache-2.0
//! `star-telemetry` — Token Telemetry (Phase G.9, per AGENTS §6.1)
//!
//! **目的**: Token 计量 + Prometheus 导出
//!
//! **架构 (per G.9 brief)**:
//! - `TokenMeter`: 计量 token 用量 (per agent / per call)
//! - `PrometheusExporter`: 导出 Prometheus 文本格式
//! - `OpenTelemetryExporter`: OTel stub (Phase G+ 实装)
//!
//! **守门 #4 派生 (per token-OLU 基线)**: 1 SRE · 周 ≈ 1M tokens, 计 token 不计人天
//!
//! **Mavis 临时代签 5 域 Lead 决策 (per 9/3 11:35 JST 反转)**: Telemetry schema 决策 Mavis 临时代签, 真人到位后追溯签字

#![allow(missing_docs)] // G.9 PoC 启动, Phase 2 spec 完成后补 doc

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::RwLock;
use uuid::Uuid;

mod exporter;
mod meter;

pub use exporter::{MetricsExporter, OpenTelemetryExporter, PrometheusExporter};
pub use meter::{TokenMeter, TokenUsage, TokenUsageByAgent};

/// Telemetry 错误
#[derive(Debug, Error)]
pub enum TelemetryError {
    #[error("export: {0}")]
    Export(String),
    #[error("meter: {0}")]
    Meter(String),
}

/// 调用记录 (per agent / per call, per G.9)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallRecord {
    /// 调用 ID
    pub call_id: Uuid,
    /// 租户 ID
    pub tenant_id: Uuid,
    /// Agent ID
    pub agent_id: Uuid,
    /// 模型 (e.g. "gpt-4o" / "claude-3.5-sonnet")
    pub model: String,
    /// input token 数
    pub input_tokens: u64,
    /// output token 数
    pub output_tokens: u64,
    /// 时间戳 (ms since epoch)
    pub timestamp_ms: u64,
    /// 成本 (USD, optional, per-token pricing)
    pub cost_usd: Option<f64>,
}

impl CallRecord {
    /// 总 token
    pub fn total_tokens(&self) -> u64 {
        self.input_tokens + self.output_tokens
    }
}
