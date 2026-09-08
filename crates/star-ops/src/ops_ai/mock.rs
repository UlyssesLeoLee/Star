// SPDX-License-Identifier: MIT OR Apache-2.0
//! Mock AI 通道 (per OPS-BASIC-DESIGN §5.3 + 守门 #23 + 守门 #24 v2)
//!
//! MVP: subprocess 调 `scripts/automation/ai_log_mock.py`, 解析 JSON 输出
//! 永远启用 (兜底, per ADR-0026 §2.2 L1)

use async_trait::async_trait;
use serde::Deserialize;
use tracing::{info, warn};
use uuid::Uuid;

use crate::error::OpsError;
use crate::ops_ai::AiChannel;
use crate::ops_domain::log::{Anomaly, LogAnalysis, LogEntry, LogLevel, Suggestion};

/// Mock 通道 (守门 #23: 不开 OpenAI/Anthropic 第三方 API)
pub struct MockChannel;

#[async_trait]
impl AiChannel for MockChannel {
    fn name(&self) -> &'static str {
        "mock"
    }

    fn is_enabled(&self) -> bool {
        true
    }

    async fn analyze_log(&self, log: &LogEntry) -> Result<LogAnalysis, OpsError> {
        info!(log_id = %log.id, "MockChannel: calling ai_log_mock.py");

        // MVP: 直接用 stub_for 返 mock 输出 (subprocess 接入 [M] 子项)
        // 跳过 subprocess 调用, 避免 MVP 阶段跨环境依赖
        // 实装路径: tokio::process::Command::new("python").arg("...").output()
        warn!("MockChannel: subprocess 路径 [M] 子项实装, 当前返 stub (守门 #23 mock 等价)");

        Ok(mock_analyze(log))
    }
}

/// 纯内存 mock (不调 subprocess), MVP 阶段简化
/// 守门 #23: confidence < 0.5, 永远标 "needs_review"
fn mock_analyze(log: &LogEntry) -> LogAnalysis {
    // 简单规则: ERROR → 1 个 anomaly + 1 个 suggestion
    let (anomalies, suggestions) = if matches!(log.level, LogLevel::Error) {
        (
            vec![Anomaly {
                kind: "spike".to_string(),
                timestamp: log.timestamp,
                level: log.level,
                message_excerpt: log.message.chars().take(100).collect::<String>(),
            }],
            vec![Suggestion {
                id: format!("s-{}", Uuid::new_v4()),
                text: "检查 helm release / pod health check 配置 (mock 建议)".to_string(),
                confidence: 0.42,
            }],
        )
    } else {
        (vec![], vec![])
    };

    LogAnalysis {
        log_id: log.id,
        summary: format!(
            "{} 条 log 来自 {}, 当前 mock 简化版 (subprocess 路径 [M] 子项实装)",
            anomalies.len() + 1,
            log.source
        ),
        anomalies,
        suggestions,
        // 守门 #23: mock confidence 永远 < 0.5
        confidence: 0.42,
        generated_by: "mock".to_string(),
    }
}

/// Subprocess 路径 (实装阶段启用, MVP 留接口)
#[derive(Debug, Deserialize)]
pub struct MockSubprocessOutput {
    pub summary: String,
    pub anomalies: Vec<Anomaly>,
    pub suggestions: Vec<Suggestion>,
}

/// 调 ai_log_mock.py subprocess (守门 #24 v2, MVP 不启用)
#[allow(dead_code)]
pub async fn call_subprocess_stub(log_message: &str) -> Result<MockSubprocessOutput, OpsError> {
    let output = tokio::process::Command::new("python")
        .arg("scripts/automation/ai_log_mock.py")
        .arg(log_message)
        .output()
        .await
        .map_err(|e| OpsError::Internal(format!("subprocess 调起失败: {}", e)))?;

    if !output.status.success() {
        return Err(OpsError::Internal(format!(
            "ai_log_mock.py exit {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    serde_json::from_slice(&output.stdout)
        .map_err(|e| OpsError::Internal(format!("解析 mock 输出失败: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops_domain::log::LogLevel;
    use chrono::Utc;

    #[test]
    fn mock_channel_always_enabled() {
        assert!(MockChannel.is_enabled());
        assert_eq!(MockChannel.name(), "mock");
    }

    #[test]
    fn mock_analyze_error_log_produces_anomaly() {
        let log = LogEntry {
            id: Uuid::nil(),
            source: "test".to_string(),
            level: LogLevel::Error,
            message: "test error".to_string(),
            timestamp: Utc::now(),
            trace_id: None,
        };
        let analysis = mock_analyze(&log);
        assert_eq!(analysis.anomalies.len(), 1);
        assert_eq!(analysis.suggestions.len(), 1);
        // 守门 #23: mock confidence < 0.5
        assert!(analysis.confidence < 0.5);
        assert_eq!(analysis.generated_by, "mock");
    }

    #[test]
    fn mock_analyze_info_log_produces_no_anomaly() {
        let log = LogEntry {
            id: Uuid::nil(),
            source: "test".to_string(),
            level: LogLevel::Info,
            message: "test info".to_string(),
            timestamp: Utc::now(),
            trace_id: None,
        };
        let analysis = mock_analyze(&log);
        assert_eq!(analysis.anomalies.len(), 0);
    }
}
