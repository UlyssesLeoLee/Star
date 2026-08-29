//! Star API Agent Monitor + Audit (B.9 实装, per 2026-08-30 07:40 JST wt-b9-api-audit)
//!
//! 职责:
//!   1. **ApiCallEvent**: 1 次 API 调用的完整审计事件 (per 9 AI Audit 关键字段, per domain-audit spec INV-AU-02)
//!   2. **ApiMonitor**: 内存聚合 (按 provider 计数 + p50/p95 latency + 错误率)
//!   3. **record_call()**: 在 B.1/B.6 OpenClawClient.generate / HermesClient.generate 调完后调用, 1 行接入
//!   4. **AuditSink trait**: 抽象审计出口, 内存 sink (默认) + 文件 sink (Phase 2) + domain-audit sink (跨 crate 整合)
//!
//! 已知缺口 (per 缺标比错标 — 8/26 JST 偏好):
//!   1. 当前 AuditSink 只有 InMemorySink, 缺 FileSink / domain-audit sink (Phase 2 接)
//!   2. p50/p95 简化版 (采样 1000 calls, 满了重置), 缺 sliding window
//!   3. 不接 KMS, audit 明文 (per E.4 KMS 集成凭证到位后)
//!   4. 不接 OpenTelemetry / Prometheus (Phase 2)
//!
//! 不做 (per 守门):
//!   - 不动 domain-audit crate (per Phase D 整合策略)
//!   - 不写 UI (per frontend)
//!   - 不接 OpenClaw/Hermes 真实调用, 只 1 行 API 接入

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// API 调用结果
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiCallStatus {
    Success,
    /// 4xx (永久错误, 不重试)
    ClientError,
    /// 5xx (服务器错误, 可重试)
    ServerError,
    /// 网络 / timeout
    NetworkError,
    /// B.8 触发降级 (后续走 CLI)
    FallbackTriggered,
}

/// 1 次 API 调用的完整审计事件 (per domain-audit INV-AU-02 9 字段简化)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiCallEvent {
    /// 唯一事件 ID
    pub event_id: Uuid,
    /// API provider (openclaw / hermes / claude / codex)
    pub provider: String,
    /// 调用的 endpoint (per B.1 OpenClawClient.config.base_url)
    pub endpoint: String,
    /// 调用的开始时间
    pub started_at: chrono::DateTime<chrono::Utc>,
    /// 调用耗时 (ms)
    pub duration_ms: u64,
    /// 调用状态
    pub status: ApiCallStatus,
    /// 错误消息 (if status != Success)
    pub error_message: Option<String>,
    /// token 数 (per B.7 quota Usage)
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    /// 是否走了 mock 模式 (per B.1/B.6 mock_mode)
    pub is_mock: bool,
    /// 触发降级时填, 降级目标 CLI kind
    pub fallback_target: Option<String>,
}

impl ApiCallEvent {
    pub fn new(provider: impl Into<String>, endpoint: impl Into<String>) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            provider: provider.into(),
            endpoint: endpoint.into(),
            started_at: chrono::Utc::now(),
            duration_ms: 0,
            status: ApiCallStatus::Success,
            error_message: None,
            prompt_tokens: 0,
            completion_tokens: 0,
            is_mock: false,
            fallback_target: None,
        }
    }

    /// 标记成功 + token
    pub fn succeeded(mut self, prompt_tokens: u32, completion_tokens: u32) -> Self {
        self.status = ApiCallStatus::Success;
        self.prompt_tokens = prompt_tokens;
        self.completion_tokens = completion_tokens;
        self.duration_ms = (chrono::Utc::now() - self.started_at).num_milliseconds() as u64;
        self
    }

    /// 标记失败
    pub fn failed(mut self, status: ApiCallStatus, error: impl Into<String>) -> Self {
        self.status = status;
        self.error_message = Some(error.into());
        self.duration_ms = (chrono::Utc::now() - self.started_at).num_milliseconds() as u64;
        self
    }

    /// 标记降级
    pub fn fallback(mut self, target: impl Into<String>) -> Self {
        self.fallback_target = Some(target.into());
        self.status = ApiCallStatus::FallbackTriggered;
        self
    }
}

/// Provider 聚合统计
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProviderStats {
    pub total_calls: u64,
    pub success_calls: u64,
    pub error_calls: u64,
    pub fallback_calls: u64,
    pub total_prompt_tokens: u64,
    pub total_completion_tokens: u64,
    /// 最近 1000 calls 的 latency 列表 (ms), 简化 p50/p95
    pub recent_latencies: Vec<u64>,
}

impl ProviderStats {
    pub fn success_rate(&self) -> f64 {
        if self.total_calls == 0 {
            0.0
        } else {
            self.success_calls as f64 / self.total_calls as f64
        }
    }

    pub fn p50_latency_ms(&self) -> u64 {
        if self.recent_latencies.is_empty() {
            return 0;
        }
        let mut sorted = self.recent_latencies.clone();
        sorted.sort_unstable();
        sorted[sorted.len() / 2]
    }

    pub fn p95_latency_ms(&self) -> u64 {
        if self.recent_latencies.is_empty() {
            return 0;
        }
        let mut sorted = self.recent_latencies.clone();
        sorted.sort_unstable();
        sorted[(sorted.len() as f64 * 0.95) as usize]
    }

    pub fn record(&mut self, event: &ApiCallEvent) {
        self.total_calls += 1;
        match event.status {
            ApiCallStatus::Success => {
                self.success_calls += 1;
                self.total_prompt_tokens += event.prompt_tokens as u64;
                self.total_completion_tokens += event.completion_tokens as u64;
            }
            ApiCallStatus::FallbackTriggered => self.fallback_calls += 1,
            _ => self.error_calls += 1,
        }
        // 简化: 只保留最近 1000 latencies, 满了丢最早的
        self.recent_latencies.push(event.duration_ms);
        if self.recent_latencies.len() > 1000 {
            self.recent_latencies.remove(0);
        }
    }
}

/// Audit Sink trait (抽象审计出口)
pub trait AuditSink: Send + Sync {
    fn write(&self, event: &ApiCallEvent);
}

/// 内存 sink (默认, 跨 session 不持久)
pub struct InMemorySink {
    events: Arc<RwLock<Vec<ApiCallEvent>>>,
}

impl InMemorySink {
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn events(&self) -> Vec<ApiCallEvent> {
        self.events.read().unwrap().clone()
    }

    pub fn len(&self) -> usize {
        self.events.read().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.read().unwrap().is_empty()
    }
}

impl Default for InMemorySink {
    fn default() -> Self {
        Self::new()
    }
}

impl AuditSink for InMemorySink {
    fn write(&self, event: &ApiCallEvent) {
        self.events.write().unwrap().push(event.clone());
    }
}

/// ApiMonitor: 跨 provider 监控 + 聚合
pub struct ApiMonitor {
    sinks: Vec<Arc<dyn AuditSink>>,
    stats: Arc<RwLock<HashMap<String, ProviderStats>>>,
}

impl ApiMonitor {
    pub fn new() -> Self {
        Self {
            sinks: Vec::new(),
            stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_sink(sink: Arc<dyn AuditSink>) -> Self {
        let mut m = Self::new();
        m.sinks.push(sink);
        m
    }

    /// 记录 1 次 API 调用 (在 B.1/B.6 OpenClawClient.generate 调完后调用)
    pub fn record(&self, event: ApiCallEvent) {
        // 1) 更新聚合
        let mut stats = self.stats.write().unwrap();
        let provider_stats = stats.entry(event.provider.clone()).or_default();
        provider_stats.record(&event);
        drop(stats);

        // 2) 写入所有 sink
        for sink in &self.sinks {
            sink.write(&event);
        }
    }

    /// 获取 provider 统计
    pub fn stats(&self, provider: &str) -> Option<ProviderStats> {
        self.stats.read().unwrap().get(provider).cloned()
    }

    /// 列出所有 provider
    pub fn providers(&self) -> Vec<String> {
        self.stats.read().unwrap().keys().cloned().collect()
    }
}

impl Default for ApiMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_new_defaults_success() {
        let e = ApiCallEvent::new("openclaw", "https://api.openclaw.dev/v1");
        assert_eq!(e.provider, "openclaw");
        assert_eq!(e.status, ApiCallStatus::Success);
        assert_eq!(e.duration_ms, 0);
    }

    #[test]
    fn event_succeeded_records_tokens() {
        let e = ApiCallEvent::new("openclaw", "x")
            .succeeded(100, 50);
        assert_eq!(e.prompt_tokens, 100);
        assert_eq!(e.completion_tokens, 50);
        assert_eq!(e.status, ApiCallStatus::Success);
    }

    #[test]
    fn event_failed_records_error() {
        let e = ApiCallEvent::new("openclaw", "x")
            .failed(ApiCallStatus::ServerError, "503 service unavailable");
        assert_eq!(e.status, ApiCallStatus::ServerError);
        assert!(e.error_message.is_some());
    }

    #[test]
    fn event_fallback_records_target() {
        let e = ApiCallEvent::new("openclaw", "x")
            .fallback("claude");
        assert_eq!(e.fallback_target, Some("claude".into()));
        assert_eq!(e.status, ApiCallStatus::FallbackTriggered);
    }

    #[test]
    fn provider_stats_success_rate() {
        let mut stats = ProviderStats::default();
        for _ in 0..7 {
            let e = ApiCallEvent::new("openclaw", "x").succeeded(10, 5);
            stats.record(&e);
        }
        for _ in 0..3 {
            let e = ApiCallEvent::new("openclaw", "x").failed(ApiCallStatus::ServerError, "503");
            stats.record(&e);
        }
        assert_eq!(stats.total_calls, 10);
        assert_eq!(stats.success_calls, 7);
        assert_eq!(stats.error_calls, 3);
        assert!((stats.success_rate() - 0.7).abs() < 0.001);
    }

    #[test]
    fn provider_stats_p50_p95() {
        let mut stats = ProviderStats::default();
        for i in 1..=100 {
            let mut e = ApiCallEvent::new("openclaw", "x");
            e.duration_ms = i * 10; // 10ms, 20ms, ..., 1000ms
            stats.record(&e);
        }
        // p50 (median) ≈ 500ms (50th of [10, 20, ..., 1000])
        assert!((500..=510).contains(&stats.p50_latency_ms()));
        // p95 ≈ 950ms (95th)
        assert!(stats.p95_latency_ms() >= 940);
    }

    #[test]
    fn in_memory_sink_writes_and_reads() {
        let sink = InMemorySink::new();
        assert_eq!(sink.len(), 0);
        let e = ApiCallEvent::new("openclaw", "x").succeeded(10, 5);
        sink.write(&e);
        assert_eq!(sink.len(), 1);
        let events = sink.events();
        assert_eq!(events[0].event_id, e.event_id);
    }

    #[test]
    fn monitor_record_updates_stats_and_sink() {
        let sink = Arc::new(InMemorySink::new());
        let monitor = ApiMonitor::with_sink(sink.clone());
        monitor.record(ApiCallEvent::new("openclaw", "x").succeeded(10, 5));
        monitor.record(ApiCallEvent::new("openclaw", "x").succeeded(20, 10));
        monitor.record(ApiCallEvent::new("hermes", "y").failed(ApiCallStatus::NetworkError, "timeout"));
        assert_eq!(sink.len(), 3);
        let openclaw_stats = monitor.stats("openclaw").unwrap();
        assert_eq!(openclaw_stats.total_calls, 2);
        assert_eq!(openclaw_stats.success_calls, 2);
        let hermes_stats = monitor.stats("hermes").unwrap();
        assert_eq!(hermes_stats.total_calls, 1);
        assert_eq!(hermes_stats.error_calls, 1);
        assert_eq!(monitor.providers().len(), 2);
    }

    #[test]
    fn monitor_event_json_round_trip() {
        let e = ApiCallEvent::new("openclaw", "https://api.openclaw.dev/v1")
            .succeeded(100, 50);
        let json = serde_json::to_string(&e).unwrap();
        let parsed: ApiCallEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.event_id, e.event_id);
        assert_eq!(parsed.prompt_tokens, 100);
    }
}
