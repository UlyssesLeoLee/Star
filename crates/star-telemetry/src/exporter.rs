// MetricsExporter (per G.9 brief §3.9):
// - PrometheusExporter: 文本格式 (实装)
// - OpenTelemetryExporter: OTLP stub (Phase G+ 实装)
use async_trait::async_trait;
use std::sync::Arc;

use super::meter::TokenMeter;
use super::TelemetryError;

/// 指标导出 trait
#[async_trait]
pub trait MetricsExporter: Send + Sync {
    /// 导出为字符串
    async fn export(&self) -> Result<String, TelemetryError>;
    /// 导出器名称
    fn name(&self) -> &'static str;
}

/// Prometheus 导出器 (per G.9)
pub struct PrometheusExporter {
    meter: Arc<TokenMeter>,
}

impl PrometheusExporter {
    pub fn new(meter: Arc<TokenMeter>) -> Self {
        Self { meter }
    }
}

#[async_trait]
impl MetricsExporter for PrometheusExporter {
    async fn export(&self) -> Result<String, TelemetryError> {
        let g = self.meter.global_usage().await;
        let mut out = String::new();
        // HELP / TYPE 行 (per Prometheus 文本格式)
        out.push_str("# HELP star_token_input_total Total input tokens consumed\n");
        out.push_str("# TYPE star_token_input_total counter\n");
        out.push_str(&format!("star_token_input_total {}\n", g.input_tokens));
        out.push_str("# HELP star_token_output_total Total output tokens produced\n");
        out.push_str("# TYPE star_token_output_total counter\n");
        out.push_str(&format!("star_token_output_total {}\n", g.output_tokens));
        out.push_str("# HELP star_token_calls_total Total LLM call count\n");
        out.push_str("# TYPE star_token_calls_total counter\n");
        out.push_str(&format!("star_token_calls_total {}\n", g.call_count));
        out.push_str("# HELP star_token_cost_usd_total Total cost in USD\n");
        out.push_str("# TYPE star_token_cost_usd_total counter\n");
        out.push_str(&format!("star_token_cost_usd_total {:.6}\n", g.cost_usd));
        Ok(out)
    }
    fn name(&self) -> &'static str {
        "prometheus"
    }
}

/// OpenTelemetry 导出器 (per G.9, Phase G+ stub)
pub struct OpenTelemetryExporter {
    meter: Arc<TokenMeter>,
    /// 端点 (e.g. "http://localhost:4318/v1/metrics")
    pub endpoint: String,
}

impl OpenTelemetryExporter {
    pub fn new(meter: Arc<TokenMeter>, endpoint: String) -> Self {
        Self { meter, endpoint }
    }
}

#[async_trait]
impl MetricsExporter for OpenTelemetryExporter {
    async fn export(&self) -> Result<String, TelemetryError> {
        // OTel OTLP HTTP JSON 格式 (Phase G+ stub)
        // 当前只 export 序列化数据, 不实际发请求
        let g = self.meter.global_usage().await;
        let body = serde_json::json!({
            "endpoint": self.endpoint,
            "format": "otlp-http",
            "metrics": [
                {
                    "name": "star.token.input",
                    "value": g.input_tokens,
                },
                {
                    "name": "star.token.output",
                    "value": g.output_tokens,
                },
                {
                    "name": "star.token.calls",
                    "value": g.call_count,
                },
                {
                    "name": "star.token.cost_usd",
                    "value": g.cost_usd,
                },
            ],
        });
        serde_json::to_string_pretty(&body).map_err(|e| TelemetryError::Export(e.to_string()))
    }
    fn name(&self) -> &'static str {
        "opentelemetry"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CallRecord;
    use uuid::Uuid;

    #[tokio::test]
    async fn prometheus_export() {
        let meter = Arc::new(TokenMeter::new());
        let agent = Uuid::new_v4();
        meter
            .record(CallRecord {
                call_id: Uuid::new_v4(),
                tenant_id: Uuid::new_v4(),
                agent_id: agent,
                model: "gpt-4o".into(),
                input_tokens: 1000,
                output_tokens: 500,
                timestamp_ms: 1_700_000_000_000,
                cost_usd: Some(0.05),
            })
            .await;

        let exp = PrometheusExporter::new(meter);
        let s = exp.export().await.unwrap();
        assert!(s.contains("star_token_input_total 1000"));
        assert!(s.contains("star_token_output_total 500"));
        assert!(s.contains("star_token_calls_total 1"));
        assert!(s.contains("# TYPE star_token_input_total counter"));
    }

    #[tokio::test]
    async fn opentelemetry_export_stub() {
        let meter = Arc::new(TokenMeter::new());
        let exp = OpenTelemetryExporter::new(meter, "http://localhost:4318/v1/metrics".into());
        let s = exp.export().await.unwrap();
        assert!(s.contains("\"format\": \"otlp-http\""));
        assert!(s.contains("\"name\": \"star.token.input\""));
    }
}
