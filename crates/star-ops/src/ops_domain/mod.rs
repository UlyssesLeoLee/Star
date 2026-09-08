// SPDX-License-Identifier: MIT OR Apache-2.0
//! `star-ops` 3 子域聚合 (per SRS-001 §4 F-01..F-04)
//!
//! MVP-骨架阶段: 各子域 1 数据结构 + 1 stub 方法, 不接业务逻辑
//! 实装阶段: 各子域按 [M] 子项拍板逐个实装 (累计估 ~2.0M token per SRS-001 §10.3)

pub mod cluster;
pub mod log;
pub mod metrics;

pub use cluster::{CanaryRequest, HelmRelease, ReleaseStatus, RollbackRequest};
pub use log::{LogAnalysis, LogEntry, LogLevel};
pub use metrics::{OpsMetric, TrendDirection};
