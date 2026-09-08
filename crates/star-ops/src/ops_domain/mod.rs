// SPDX-License-Identifier: MIT OR Apache-2.0
//! `star-ops` 4 子域聚合 (per SRS-001 §4 F-01..F-04)
//!
//! MVP-骨架阶段: 各子域 1 数据结构 + 1 stub 方法, 不接业务逻辑
//! F-01..F-04 端到端实装: 各子域调真实路径 (F-04 walkdir 扫 docs/ 4 子目录)

pub mod cluster;
pub mod docs;
pub mod log;
pub mod metrics;

pub use cluster::{
    CanaryRequest, HelmActionAck, HelmMockOutput, HelmRelease, ReleaseStatus, RollbackRequest,
};
pub use docs::{DocCategory, DocRef, DocScanner};
pub use log::{LogAnalysis, LogEntry, LogLevel};
pub use metrics::{MetricsAggregator, OpsMetric, TrendDirection};
