// SPDX-License-Identifier: MIT OR Apache-2.0
//! Mock AI 通道 (per OPS-BASIC-DESIGN §5.3 + 守门 #23 + 守门 #24 v2)
//!
//! F-02 端到端: subprocess 调 `scripts/automation/ai_log_mock.py`, 解析 JSON 输出
//! 永远启用 (兜底, per ADR-0026 §2.2 L1)
//!
//! 守门 #19 v19: agent 跟外部交互走 subprocess (per scripts/automation/ai_log_mock.py)
//! 守门 #24 v2: 调试控制台走 subprocess 替代 RPC, 可重放可观测

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
        info!(log_id = %log.id, source = %log.source, "MockChannel: subprocess 调 ai_log_mock.py (F-02 端到端)");

        // F-02 端到端: 真实 subprocess 调 ai_log_mock.py, 解析 stdout JSON
        // 守门 #19 v19: 外部功能点强制走 scripts/automation/<purpose>.py
        match call_subprocess_stub(&log.message).await {
            Ok(output) => Ok(LogAnalysis {
                log_id: log.id,
                summary: output.summary,
                anomalies: output.anomalies,
                suggestions: output.suggestions,
                confidence: 0.42, // 守门 #23: mock confidence 永远 < 0.5
                generated_by: "mock".to_string(),
            }),
            Err(e) => {
                // subprocess 失败时降级到 in-process mock (保证兜底可用 per ADR-0026 §2.2 L1)
                warn!(err = %e, "MockChannel: subprocess 调用失败, 降级到 in-process mock");
                Ok(mock_analyze(log))
            }
        }
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

/// Subprocess 输出 (F-02 端到端实装, 跟 ai_log_mock.py 顶层 JSON 形状对齐)
#[derive(Debug, Deserialize)]
pub struct MockSubprocessOutput {
    pub summary: String,
    pub anomalies: Vec<Anomaly>,
    pub suggestions: Vec<Suggestion>,
}

/// 调 ai_log_mock.py subprocess (守门 #19 v19 + 守门 #24 v2, F-02 端到端启用)
pub async fn call_subprocess_stub(log_message: &str) -> Result<MockSubprocessOutput, OpsError> {
    // 守门 #19 v19: 走 scripts/automation/ai_log_mock.py 统一入口
    // 守门 #24 v2: subprocess 替代 RPC, 解析 stdout JSON
    let script_path = resolve_mock_script_path();
    let mut cmd = tokio::process::Command::new("python");
    cmd.arg(&script_path).arg("--log").arg(log_message);
    // Windows 中文系统默认 codepage 936 (GBK), 强制 PYTHONIOENCODING=utf-8
    // 让 python stdout 输出 UTF-8 bytes, 避免 serde_json 解析失败
    cmd.env("PYTHONIOENCODING", "utf-8");
    let output = cmd
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

    // F-02 端到端: ai_log_mock.py v0.1 顶层包装含 confidence/generated_by/elapsed_ms/needs_review,
    // MockSubprocessOutput 简化为内部子集, 这里只解析 anomalies/suggestions/summary
    #[derive(Deserialize)]
    struct FullOutput {
        summary: String,
        anomalies: Vec<Anomaly>,
        suggestions: Vec<Suggestion>,
    }
    let parsed: FullOutput = serde_json::from_slice(&output.stdout)
        .map_err(|e| OpsError::Internal(format!("解析 mock 输出失败: {}", e)))?;

    Ok(MockSubprocessOutput {
        summary: parsed.summary,
        anomalies: parsed.anomalies,
        suggestions: parsed.suggestions,
    })
}

/// 解析 ai_log_mock.py 路径 (per 守门 #19 v19)
/// 优先级: STAR_OPS_MOCK_SCRIPT env > 当前 cwd > CARGO_MANIFEST_DIR 向上找 scripts/automation
fn resolve_mock_script_path() -> String {
    if let Ok(p) = std::env::var("STAR_OPS_MOCK_SCRIPT") {
        return p;
    }
    // 尝试当前 cwd
    if let Ok(cwd) = std::env::current_dir() {
        let candidate = cwd
            .join("scripts")
            .join("automation")
            .join("ai_log_mock.py");
        if candidate.exists() {
            return candidate.to_string_lossy().into_owned();
        }
        // worktree root 嵌套
        for ancestor in cwd.ancestors() {
            let cand = ancestor
                .join("scripts")
                .join("automation")
                .join("ai_log_mock.py");
            if cand.exists() {
                return cand.to_string_lossy().into_owned();
            }
        }
    }
    // fallback: 相对路径 (假设从 worktree root 跑)
    "scripts/automation/ai_log_mock.py".to_string()
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

    /// F-02 端到端: call_subprocess_stub 真实调 ai_log_mock.py
    /// 守门 #9 + 守门 #24 v2: 实证 subprocess 路径可用
    /// tokio::test 因为 call_subprocess_stub 是 async
    #[tokio::test]
    async fn call_subprocess_stub_real_invocation() {
        // 守门 #23: confidence 永远 < 0.5 (mock 派生规)
        let out = call_subprocess_stub("2026-09-08 ERROR test failure")
            .await
            .expect("subprocess must succeed");
        assert!(!out.summary.is_empty(), "summary 非空");
        // 至少 1 条 suggestion (mock 总是给)
        assert!(!out.suggestions.is_empty(), "suggestions 至少 1 条");
        // 守门 #23: mock 永远标 needs_review
        for s in &out.suggestions {
            assert!(
                s.confidence < 0.5,
                "mock confidence 永远 < 0.5, got {}",
                s.confidence
            );
        }
    }

    // ============ UT-IT-51 §2.3 Phase 5 ops_ai 派生缺口 (per brief §2.1) ============

    /// 派生 #19: mock 通道 100% 成功 (跟 baseline mock_channel_always_enabled 互补)
    /// 守门 #23: L1 mock 兜底, 永远 enabled + 永远 Ok
    #[tokio::test]
    async fn mock_channel_always_succeeds() {
        let ch = MockChannel;
        let log = LogEntry {
            id: uuid::Uuid::new_v4(),
            source: "test".to_string(),
            level: LogLevel::Error,
            message: "2026-09-08 ERROR test failure".to_string(),
            timestamp: chrono::Utc::now(),
            trace_id: None,
        };
        // 走 mock_analyze 路径 (subprocess 可能失败, 走兜底)
        // L1 mock 兜底必返 Ok
        let result = ch.analyze_log(&log).await;
        assert!(result.is_ok(), "L1 mock 兜底必 100% Ok");
        let analysis = result.expect("Ok");
        assert_eq!(analysis.generated_by, "mock", "L1 必返 mock 通道");
        assert!(analysis.confidence < 0.5, "mock confidence 必 < 0.5");
    }
}
