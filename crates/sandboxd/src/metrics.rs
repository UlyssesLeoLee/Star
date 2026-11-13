//! `sandboxd` metrics 模块 (per FR-6.1, DD-SANDBOX-002 §2.4)
//!
//! 9 Prometheus 指标 (跟 FR-6.1 1:1):
//!
//! | 指标 | 类型 | 标签 |
//! |---|---|---|
//! | `sandboxd_sessions_total` | counter | status=created/failed/rejected/destroyed |
//! | `sandboxd_sessions_active` | gauge | — |
//! | `sandboxd_session_duration_seconds` | histogram | — |
//! | `sandboxd_resource_limit_hits_total` | counter | limit_type=memory/cpu/processes |
//! | `sandboxd_network_violations_total` | counter | action=blocked/allowed |
//! | `sandboxd_fs_violations_total` | counter | action=blocked/allowed |
//! | `sandboxd_capability_violations_total` | counter | action=dropped/allowed |
//! | `sandboxd_rpc_duration_seconds` | histogram | rpc=create_session/run_command/destroy_session/stream_logs |
//!
//! scrape endpoint: `GET /metrics` (跟 /healthz 同 axum 进程, per FR-6.1)。
//! 复用 `prometheus` crate 0.13 (per BD §0 选型), star-telemetry 的 PrometheusExporter
//! 为 token 专用指标, 不适合挂载 sandboxd 指标 (per D-9 派生标注)。

use prometheus::{
    Encoder, Histogram, HistogramOpts, HistogramVec, IntCounterVec, IntGauge, Opts, Registry,
    TextEncoder,
};

/// sandboxd 指标集合 (per FR-6.1)
pub struct SandboxdMetrics {
    /// Prometheus registry
    pub registry: Registry,
    /// session 计数 {status}
    pub sessions_total: IntCounterVec,
    /// 活跃 session 数
    pub sessions_active: IntGauge,
    /// session 存活时长直方图
    pub session_duration_seconds: Histogram,
    /// resource 维限制命中 {limit_type}
    pub resource_limit_hits_total: IntCounterVec,
    /// network 维违规 {action}
    pub network_violations_total: IntCounterVec,
    /// fs 维违规 {action}
    pub fs_violations_total: IntCounterVec,
    /// capability 维违规 {action}
    pub capability_violations_total: IntCounterVec,
    /// RPC 延迟直方图 {rpc}
    pub rpc_duration_seconds: HistogramVec,
}

impl SandboxdMetrics {
    /// 注册全部 9 指标 (per FR-6.1 指标清单)
    pub fn new() -> Result<Self, prometheus::Error> {
        let registry = Registry::new();

        let sessions_total = IntCounterVec::new(
            Opts::new(
                "sandboxd_sessions_total",
                "Total sandbox sessions by status",
            ),
            &["status"],
        )?;
        let sessions_active = IntGauge::new("sandboxd_sessions_active", "Active sandbox sessions")?;
        let session_duration_seconds = Histogram::with_opts(HistogramOpts::new(
            "sandboxd_session_duration_seconds",
            "Sandbox session lifetime in seconds",
        ))?;
        let resource_limit_hits_total = IntCounterVec::new(
            Opts::new(
                "sandboxd_resource_limit_hits_total",
                "Resource limit hits by limit type",
            ),
            &["limit_type"],
        )?;
        let network_violations_total = IntCounterVec::new(
            Opts::new(
                "sandboxd_network_violations_total",
                "Network policy violations by action",
            ),
            &["action"],
        )?;
        let fs_violations_total = IntCounterVec::new(
            Opts::new(
                "sandboxd_fs_violations_total",
                "FS policy violations by action",
            ),
            &["action"],
        )?;
        let capability_violations_total = IntCounterVec::new(
            Opts::new(
                "sandboxd_capability_violations_total",
                "Capability policy violations by action",
            ),
            &["action"],
        )?;
        let rpc_duration_seconds = HistogramVec::new(
            HistogramOpts::new(
                "sandboxd_rpc_duration_seconds",
                "gRPC RPC latency in seconds",
            ),
            &["rpc"],
        )?;

        registry.register(Box::new(sessions_total.clone()))?;
        registry.register(Box::new(sessions_active.clone()))?;
        registry.register(Box::new(session_duration_seconds.clone()))?;
        registry.register(Box::new(resource_limit_hits_total.clone()))?;
        registry.register(Box::new(network_violations_total.clone()))?;
        registry.register(Box::new(fs_violations_total.clone()))?;
        registry.register(Box::new(capability_violations_total.clone()))?;
        registry.register(Box::new(rpc_duration_seconds.clone()))?;

        Ok(Self {
            registry,
            sessions_total,
            sessions_active,
            session_duration_seconds,
            resource_limit_hits_total,
            network_violations_total,
            fs_violations_total,
            capability_violations_total,
            rpc_duration_seconds,
        })
    }

    /// Prometheus 文本格式渲染 (/metrics 响应体, per FR-6.1 scrape)
    pub fn render(&self) -> Result<String, prometheus::Error> {
        let encoder = TextEncoder::new();
        let mut buf = Vec::new();
        encoder.encode(&self.registry.gather(), &mut buf)?;
        Ok(String::from_utf8_lossy(&buf).into_owned())
    }
}
