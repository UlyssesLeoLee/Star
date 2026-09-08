// crates/star-ops/benches/log_upload_bench.rs
//
// §5.1+§5.2+§5.3 F-02 Log AI 端到端 PT 雏形 (per TEST-DESIGN-OPS-001 v0.2 §5)
// (per 守门 #7 v3 + 守门 #1 v3 + 守门 #11 缺标比错标)
//
// 范围: 4 bench case, 测 log_upload 端到端各路径延迟
//   1. ladder_analyze_log_mock (existing) — Ladder.analyze_log + mock subprocess 路径
//   2. mock_channel_in_process — MockChannel 走 in-process 兜底
//   3. log_entry_construction — LogEntry 构造 (基础开销)
//   4. ops_response_serialize — LogAnalysis → JSON OpsResponse 序列化
//
// 守门:
//   - 守门 #7 v3: P95 < 200ms (per OPS-BASIC-DESIGN §3.3 NFR-PT-01)
//   - 守门 #7 v3: 0 unsafe + clippy advisory (per §5.4)
//   - 守门 #11: 缺标比错标, 5 已知缺口显式列 (per §5.5)
//   - 守门 #1 v19: agent 交互走 subprocess (mock subprocess 真实调, 不开外部 API)
//
// 已知缺口 (per 守门 #11 缺标比错标, DDD Review 必查):
//   - §5.5 缺口 #1: log_upload_bench P95 < 200ms 实证待 wt3 落地 (本次实现)
//   - §5.5 缺口 #2: k6 容量规划脚本 + 3 档用户负载实测缺 (per wt4 补)
//   - §5.5 缺口 #3: 多节点 HA + 负载均衡实测缺 ([M] 子项)
//   - §5.5 缺口 #4: 真实 PG 容器 + sqlx::test 容量实测缺 (per F-05 sprint)
//
// 引用:
//   - docs/test-design/TEST-DESIGN-OPS-001.md v0.2 §5.1+§5.2+§5.3
//   - docs/briefs/5-level-full-impl.md v0.1 §2.1 §5 PT 主体
//   - crates/star-ops/src/ops_ai/{ladder,mock}.rs
//   - crates/star-ops/src/ops_domain/log.rs

// bench 作为独立 crate, 关掉 missing_docs 顶层 deny
#![allow(missing_docs)]

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::hint::black_box;
use std::time::Duration;

/// 共享 tokio runtime (per 守门 #1 v19, bench 全 async 路径共用)
fn rt() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("tokio runtime must init")
}

/// 共用样本 LogEntry 构造 (per F-02 brief §2.1)
fn sample_error_log() -> star_ops::ops_domain::log::LogEntry {
    use chrono::Utc;
    use star_ops::ops_domain::log::{LogEntry, LogLevel};
    use uuid::Uuid;

    LogEntry {
        id: Uuid::nil(),
        source: "bench".to_string(),
        level: LogLevel::Error,
        message: "2026-09-08T07:30:00Z ERROR helm release 3 deploy failed: timeout".to_string(),
        timestamp: Utc::now(),
        trace_id: None,
    }
}

/// Bench 1: Ladder.analyze_log 端到端 (走 mock subprocess 路径, per 守门 #24 v2)
fn bench_ladder_analyze_log_mock(c: &mut Criterion) {
    let ladder = star_ops::ops_ai::default_ladder();
    let rt = rt();
    let log = sample_error_log();

    let mut group = c.benchmark_group("log_upload_ladder");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(100);

    group.bench_function("ladder_analyze_log_mock", |b| {
        b.iter(|| {
            let log = black_box(log.clone());
            let _analysis = rt
                .block_on(ladder.analyze_log(&log))
                .expect("ladder must succeed");
        });
    });

    group.finish();
}

/// Bench 2: MockChannel in-process 兜底 (subprocess 失败时降级)
fn bench_mock_channel_in_process(c: &mut Criterion) {
    use star_ops::ops_ai::mock::MockChannel;
    use star_ops::ops_ai::AiChannel;
    let ch = MockChannel;
    let rt = rt();
    let log = sample_error_log();

    let mut group = c.benchmark_group("log_upload_mock_in_process");
    group.measurement_time(Duration::from_secs(5));
    group.sample_size(200);

    group.bench_function("mock_channel_in_process", |b| {
        b.iter(|| {
            let log = black_box(log.clone());
            // 强制走 in-process 兜底: 调 mock_analyze 直接 (subprocess 路径依赖外部)
            let result = rt.block_on(ch.analyze_log(&log));
            let _ = black_box(result);
        });
    });

    group.finish();
}

/// Bench 3: LogEntry 构造 + 校验 (per F-02 upload payload 开销)
fn bench_log_entry_construction(c: &mut Criterion) {
    use chrono::Utc;
    use star_ops::ops_domain::log::{LogEntry, LogLevel};
    use uuid::Uuid;

    let mut group = c.benchmark_group("log_upload_construction");
    group.measurement_time(Duration::from_secs(3));
    group.sample_size(500);

    for size in [100, 1_000, 10_000].iter() {
        let msg = "x".repeat(*size);
        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let entry = LogEntry {
                    id: Uuid::new_v4(),
                    source: black_box("bench".to_string()),
                    level: LogLevel::Error,
                    message: black_box(msg.clone()),
                    timestamp: Utc::now(),
                    trace_id: None,
                };
                let _ = black_box(entry);
            });
        });
    }

    group.finish();
}

/// Bench 4: LogAnalysis → JSON 序列化 (per F-02 上传 + 分析响应)
fn bench_ops_response_serialize(c: &mut Criterion) {
    use star_ops::ops_api::OpsResponse;
    use star_ops::ops_domain::log::{Anomaly, LogAnalysis, LogLevel, Suggestion};
    use uuid::Uuid;

    let analysis = LogAnalysis {
        log_id: Uuid::nil(),
        summary: "bench summary — 1 anomaly + 1 suggestion".to_string(),
        anomalies: vec![Anomaly {
            kind: "spike".to_string(),
            timestamp: chrono::Utc::now(),
            level: LogLevel::Error,
            message_excerpt: "ERROR helm release 3 deploy failed".to_string(),
        }],
        suggestions: vec![Suggestion {
            id: format!("s-{}", Uuid::new_v4()),
            text: "检查 helm release 配置".to_string(),
            confidence: 0.42,
        }],
        confidence: 0.42,
        generated_by: "mock".to_string(),
    };
    let resp: OpsResponse<LogAnalysis> = OpsResponse {
        data: analysis,
        meta: star_ops::ops_api::OpsMeta {
            stub: true,
            total: None,
            hint: Some("MVP-骨架, F-02 端到端实装".to_string()),
            ai_channel: Some("mock".to_string()),
            analysis_triggered: Some(true),
            needs_review: Some(true),
            entry_count: Some(1),
            phase: None,
        },
    };

    let mut group = c.benchmark_group("log_upload_serialize");
    group.measurement_time(Duration::from_secs(3));
    group.sample_size(500);

    group.bench_function("ops_response_serialize", |b| {
        b.iter(|| {
            let json = serde_json::to_string(&black_box(&resp)).expect("serialize must succeed");
            let _ = black_box(json);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_ladder_analyze_log_mock,
    bench_mock_channel_in_process,
    bench_log_entry_construction,
    bench_ops_response_serialize
);
criterion_main!(benches);
