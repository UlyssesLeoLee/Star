// TokenMeter + Prometheus 集成测试 (per G.9)
#![allow(missing_docs)]
use star_telemetry::{CallRecord, MetricsExporter, PrometheusExporter, TokenMeter};
use std::sync::Arc;
use uuid::Uuid;

fn make_call(agent: Uuid, model: &str, input: u64, output: u64) -> CallRecord {
    CallRecord {
        call_id: Uuid::new_v4(),
        tenant_id: Uuid::new_v4(),
        agent_id: agent,
        model: model.into(),
        input_tokens: input,
        output_tokens: output,
        timestamp_ms: 1_700_000_000_000,
        cost_usd: Some(0.01),
    }
}

#[tokio::test]
async fn per_agent_per_call_token_meter() {
    let meter = TokenMeter::new();
    let agent_a = Uuid::new_v4();
    let agent_b = Uuid::new_v4();
    meter.record(make_call(agent_a, "gpt-4o", 100, 50)).await;
    meter.record(make_call(agent_a, "gpt-4o", 200, 100)).await;
    meter
        .record(make_call(agent_b, "claude-3.5", 500, 300))
        .await;

    let g = meter.global_usage().await;
    assert_eq!(g.input_tokens, 800);
    assert_eq!(g.call_count, 3);

    let ua = meter.agent_usage(agent_a).await.unwrap();
    assert_eq!(ua.usage.input_tokens, 300);
    let ub = meter.agent_usage(agent_b).await.unwrap();
    assert_eq!(ub.usage.input_tokens, 500);
}

#[tokio::test]
async fn prometheus_text_format() {
    let meter = Arc::new(TokenMeter::new());
    let agent = Uuid::new_v4();
    meter.record(make_call(agent, "gpt-4o", 1000, 500)).await;
    meter.record(make_call(agent, "gpt-4o", 500, 250)).await;

    let exp = PrometheusExporter::new(meter);
    let s = exp.export().await.unwrap();
    // HELP / TYPE 元数据
    assert!(s.contains("# HELP star_token_input_total"));
    assert!(s.contains("# TYPE star_token_input_total counter"));
    // 数据
    assert!(s.contains("star_token_input_total 1500"));
    assert!(s.contains("star_token_output_total 750"));
    assert!(s.contains("star_token_calls_total 2"));
    // exporter 名称
    assert_eq!(exp.name(), "prometheus");
}
