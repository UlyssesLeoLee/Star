// SPDX-License-Identifier: MIT OR Apache-2.0
//! F-02 log 子域 (per SRS-001 §4 F-02)
//!
//! MVP-骨架: 1 数据结构 `LogEntry` + 1 `LogAnalysis` + 2 stub method
//! 实装阶段: 接入 log 采集 agent (Fluent Bit/Loki) + 真实 LLM 通道

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Log 级别 (标准 5 级)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// Log 原始数据 (per OPS-BASIC-DESIGN §4.1 表 4 `ops_log_entry` W 类)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: Uuid,
    pub source: String,
    pub level: LogLevel,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    /// 跨调用链追踪 ID
    pub trace_id: Option<String>,
}

impl LogEntry {
    /// MVP stub: 1 条假数据 (per OPS-BASIC-DESIGN §3.2)
    pub fn stub() -> Self {
        Self {
            id: Uuid::nil(),
            source: "k8s-pod/star-mcp-7d8b".to_string(),
            level: LogLevel::Error,
            message: "2026-09-08T07:30:00Z ERROR helm release 3 deploy failed: timeout".to_string(),
            timestamp: "2026-09-08T07:30:00Z".parse().expect("hardcoded UTC"),
            trace_id: Some("trace-stub-001".to_string()),
        }
    }
}

/// 异常类型 (per OPS-BASIC-DESIGN §3.2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anomaly {
    /// 异常类型: spike / drop / pattern / unknown
    #[serde(rename = "type")]
    pub kind: String,
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub message_excerpt: String,
}

/// AI 建议 (per OPS-BASIC-DESIGN §3.2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    pub id: String,
    pub text: String,
    /// 0.0-1.0, 守门 #23: < 0.5 必标 "需人工 review"
    pub confidence: f32,
}

/// Log AI 分析结果 (per OPS-BASIC-DESIGN §3.2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogAnalysis {
    pub log_id: Uuid,
    pub summary: String,
    pub anomalies: Vec<Anomaly>,
    pub suggestions: Vec<Suggestion>,
    /// 0.0-1.0, 守门 #23 派生规: 永远 < 0.5 (mock)
    pub confidence: f32,
    /// AI 通道名 (mock / openai / anthropic / openai_stub / anthropic_stub)
    pub generated_by: String,
}

impl LogAnalysis {
    /// MVP stub: 1 条 AI 分析结果 (mock 输出, per OPS-BASIC-DESIGN §3.2)
    pub fn stub_for(log_id: Uuid) -> Self {
        Self {
            log_id,
            summary: "12 条 log 中 3 条 ERROR, 集中在 07:30:00, 跟 helm release 3 部署时间吻合"
                .to_string(),
            anomalies: vec![Anomaly {
                kind: "spike".to_string(),
                timestamp: "2026-09-08T07:30:00Z".parse().expect("hardcoded UTC"),
                level: LogLevel::Error,
                message_excerpt: "helm release 3 deploy failed: timeout".to_string(),
            }],
            suggestions: vec![Suggestion {
                id: "s1".to_string(),
                text: "检查 helm release 3 的 health check 配置".to_string(),
                confidence: 0.42,
            }],
            confidence: 0.42,
            generated_by: "mock".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_entry_stub_has_error_level() {
        let entry = LogEntry::stub();
        assert_eq!(entry.level, LogLevel::Error);
        assert!(entry.trace_id.is_some());
    }

    #[test]
    fn log_analysis_stub_confidence_below_threshold() {
        // 守门 #23: mock confidence 永远 < 0.5
        let analysis = LogAnalysis::stub_for(Uuid::nil());
        assert!(analysis.confidence < 0.5);
        assert_eq!(analysis.generated_by, "mock");
    }
}
