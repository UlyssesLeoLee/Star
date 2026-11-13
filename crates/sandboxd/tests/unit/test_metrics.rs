//! tests/unit/test_metrics.rs — Prometheus 指标单测 (per DD §5.1: 4 TC)
//!
//! 8 指标注册 / label 组合 / 累加 / 文本渲染。

use sandboxd::metrics::SandboxdMetrics;

fn metrics() -> SandboxdMetrics {
    SandboxdMetrics::new().unwrap()
}

#[test]
fn all_metrics_registered_and_render() {
    let m = metrics();
    // 每个 label-vector 指标至少观察 1 次才会出现在文本输出 (prometheus gather 行为)
    m.sessions_total.with_label_values(&["created"]).inc();
    m.sessions_active.inc();
    m.session_duration_seconds.observe(0.1);
    m.resource_limit_hits_total
        .with_label_values(&["memory"])
        .inc();
    m.network_violations_total
        .with_label_values(&["blocked"])
        .inc();
    m.fs_violations_total.with_label_values(&["blocked"]).inc();
    m.capability_violations_total
        .with_label_values(&["dropped"])
        .inc();
    let _timer = m
        .rpc_duration_seconds
        .with_label_values(&["create_session"])
        .start_timer();

    let body = m.render().unwrap();
    for name in [
        "sandboxd_sessions_total",
        "sandboxd_sessions_active",
        "sandboxd_session_duration_seconds",
        "sandboxd_resource_limit_hits_total",
        "sandboxd_network_violations_total",
        "sandboxd_fs_violations_total",
        "sandboxd_capability_violations_total",
        "sandboxd_rpc_duration_seconds",
    ] {
        assert!(body.contains(name), "missing metric {name}\n{body}");
    }
    // TYPE 行 (per Prometheus 文本格式)
    assert!(body.contains("# TYPE sandboxd_sessions_total counter"));
}

#[test]
fn label_combinations_inc() {
    let m = metrics();
    m.sessions_total.with_label_values(&["created"]).inc();
    m.sessions_total.with_label_values(&["created"]).inc();
    m.sessions_total.with_label_values(&["rejected"]).inc();
    m.sessions_active.inc();
    m.sessions_active.inc();
    m.resource_limit_hits_total
        .with_label_values(&["memory"])
        .inc();
    m.network_violations_total
        .with_label_values(&["blocked"])
        .inc();
    m.fs_violations_total.with_label_values(&["blocked"]).inc();
    m.capability_violations_total
        .with_label_values(&["dropped"])
        .inc();
    m.session_duration_seconds.observe(1.5);
    let _timer = m
        .rpc_duration_seconds
        .with_label_values(&["create_session"])
        .start_timer();

    let body = m.render().unwrap();
    // counter 累加
    assert!(
        body.contains("sandboxd_sessions_total{status=\"created\"} 2"),
        "{body}"
    );
    assert!(body.contains("sandboxd_sessions_total{status=\"rejected\"} 1"));
    // gauge 累加
    assert!(body.contains("sandboxd_sessions_active 2"));
    // histogram 计数
    assert!(body.contains("sandboxd_session_duration_seconds_count 1"));
}

#[test]
fn rpc_duration_timer_observes() {
    let m = metrics();
    {
        let timer = m
            .rpc_duration_seconds
            .with_label_values(&["run_command"])
            .start_timer();
        drop(timer);
    }
    let body = m.render().unwrap();
    assert!(body.contains("sandboxd_rpc_duration_seconds_count{rpc=\"run_command\"} 1"));
}

#[test]
fn render_is_valid_prometheus_text() {
    // 每行非注释均为 `name{labels} value` 或 `name value` 形式 (粗略校验)
    let m = metrics();
    m.sessions_total.with_label_values(&["created"]).inc();
    let body = m.render().unwrap();
    for line in body.lines() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        assert!(parts.len() >= 2, "malformed line: {line}");
        assert!(
            parts[0].starts_with("sandboxd_"),
            "unexpected metric line: {line}"
        );
    }
}
