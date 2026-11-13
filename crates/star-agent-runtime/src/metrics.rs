/*
@cypher schema=1 source_sha256=8af308ac79c136f867af31effe3fb64aa36433800f4472754b7921b1643b8ead
MERGE (self:File {path:"crates/star-agent-runtime/src/metrics.rs"})
@endcypher
*/
//! 可观测性指标 (per SRS §61 REQ-OBS-001 + 02-BD §6.4)。
//!
//! 指标清单全量落字段 (per SRS §61); Trace (per SRS §62) 由 TraceId 贯穿
//! Event / Task / Agent (类型层已带, 见 event.rs / task.rs)。

/// 分位数 (P50/P95/P99, per SRS §64 benchmark 记录)。空样本 → None。
pub fn percentile(sorted_samples: &[f64], q: f64) -> Option<f64> {
    if sorted_samples.is_empty() {
        return None;
    }
    let mut s = sorted_samples.to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let idx = ((q * (s.len() - 1) as f64).round() as usize).min(s.len() - 1);
    Some(s[idx])
}

/// Runtime 指标快照 (per SRS §61 REQ-OBS-001 字段)。
#[derive(Debug, Default, Clone)]
pub struct RuntimeMetrics {
    /// 逻辑 agent 数。
    pub logical_agent_count: usize,
    /// resident agent 数。
    pub resident_agent_count: usize,
    /// HOT agent 数。
    pub hot_agent_count: usize,
    /// WARM agent 数。
    pub warm_agent_count: usize,
    /// COLD agent 数。
    pub cold_agent_count: usize,
    /// 当前模式 ("Lightweight" / "ECS")。
    pub runtime_mode: &'static str,
    /// 模式切换次数。
    pub ecs_switch_count: u64,
    /// 进程 RSS (bytes, 骨架不采集 → 0)。
    pub rss_bytes: u64,
    /// 堆内存 (bytes, 骨架不采集 → 0)。
    pub heap_bytes: u64,
    /// Shared Runtime 内存 (bytes)。
    pub shared_runtime_bytes: u64,
    /// ECS World 内存 (bytes, 骨架估算: resident × 组件大小)。
    pub ecs_world_bytes: u64,
    /// Context cache 内存 (bytes)。
    pub context_cache_bytes: u64,
    /// RAG cache 内存 (bytes)。
    pub rag_cache_bytes: u64,
    /// WARM agent 平均内存 (bytes, 目标 < 100KB per SRS §62)。
    pub avg_warm_agent_bytes: u64,
    /// HOT agent 平均内存 (bytes)。
    pub avg_hot_agent_bytes: u64,
    /// Ready Queue 深度。
    pub ready_queue_depth: usize,
    /// 调度延迟 P50 (ms)。
    pub schedule_latency_p50_ms: Option<f64>,
    /// 调度延迟 P95 (ms)。
    pub schedule_latency_p95_ms: Option<f64>,
    /// 调度延迟 P99 (ms)。
    pub schedule_latency_p99_ms: Option<f64>,
    /// HOT 槽位使用率 0.0-1.0。
    pub hot_slot_usage: f64,
    /// 平均等待时间 (ms)。
    pub agent_wait_time_ms: Option<f64>,
    /// 活跃 LLM 请求。
    pub active_llm_requests: usize,
    /// LLM 队列深度。
    pub llm_queue_depth: usize,
    /// LLM 延迟 P95 (ms)。
    pub llm_latency_p95_ms: Option<f64>,
    /// 输入 tokens 累计。
    pub input_tokens: u64,
    /// 输出 tokens 累计。
    pub output_tokens: u64,
    /// 错误数。
    pub errors: u64,
    /// 活跃 tool 调用。
    pub active_tool_calls: usize,
    /// Tool 队列深度。
    pub tool_queue_depth: usize,
    /// Tool 延迟 P95 (ms)。
    pub tool_latency_p95_ms: Option<f64>,
    /// Tool 错误数。
    pub tool_errors: u64,
}

/// Observer 接口 (per 02-BD §3.4 MetricsSystem)。
pub trait Observer {
    /// 记录调度延迟 (ms)。
    fn record_schedule_latency(&mut self, ms: f64);
    /// 记录 LLM 延迟 (ms)。
    fn record_llm_latency(&mut self, ms: f64);
    /// 记录 Tool 延迟 (ms)。
    fn record_tool_latency(&mut self, ms: f64);
    /// 累计 token。
    fn add_tokens(&mut self, input_tokens: u64, output_tokens: u64);
    /// 错误计数 +1。
    fn inc_error(&mut self);
    /// Tool 错误计数 +1。
    fn inc_tool_error(&mut self);
    /// 派发计数 +1 (per 03-DD §2.2 dispatch step 5)。
    fn inc_dispatched(&mut self);
    /// 用采样补全 RuntimeMetrics (per SRS §61)。
    fn snapshot(&self, metrics: &mut RuntimeMetrics);
}

/// 内存实现 (单线程; 多线程用 Mutex 包装或走 P3-B observability crate)。
#[derive(Debug, Default, Clone)]
pub struct InMemoryObserver {
    /// 派发计数。
    pub dispatched: u64,
    errors: u64,
    tool_errors: u64,
    input_tokens: u64,
    output_tokens: u64,
    schedule_latency_samples: Vec<f64>,
    llm_latency_samples: Vec<f64>,
    tool_latency_samples: Vec<f64>,
}

impl InMemoryObserver {
    /// 空 observer。
    pub fn new() -> Self {
        Self::default()
    }
}

impl Observer for InMemoryObserver {
    fn record_schedule_latency(&mut self, ms: f64) {
        self.schedule_latency_samples.push(ms);
    }

    fn record_llm_latency(&mut self, ms: f64) {
        self.llm_latency_samples.push(ms);
    }

    fn record_tool_latency(&mut self, ms: f64) {
        self.tool_latency_samples.push(ms);
    }

    fn add_tokens(&mut self, input_tokens: u64, output_tokens: u64) {
        self.input_tokens += input_tokens;
        self.output_tokens += output_tokens;
    }

    fn inc_error(&mut self) {
        self.errors += 1;
    }

    fn inc_tool_error(&mut self) {
        self.tool_errors += 1;
    }

    fn inc_dispatched(&mut self) {
        self.dispatched += 1;
    }

    fn snapshot(&self, metrics: &mut RuntimeMetrics) {
        metrics.errors = self.errors;
        metrics.tool_errors = self.tool_errors;
        metrics.input_tokens = self.input_tokens;
        metrics.output_tokens = self.output_tokens;
        metrics.schedule_latency_p50_ms = percentile(&self.schedule_latency_samples, 0.50);
        metrics.schedule_latency_p95_ms = percentile(&self.schedule_latency_samples, 0.95);
        metrics.schedule_latency_p99_ms = percentile(&self.schedule_latency_samples, 0.99);
        metrics.llm_latency_p95_ms = percentile(&self.llm_latency_samples, 0.95);
        metrics.tool_latency_p95_ms = percentile(&self.tool_latency_samples, 0.95);
    }
}
