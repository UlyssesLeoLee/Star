// crates/star-ops/benches/log_upload_bench.rs
//
// F-02 端到端 PT 雏形 (per 守门 #7 v3 + 守门 #1 v3)
//
// 范围: criterion bench, 测 log_upload 端到端延迟 (含 mock subprocess)
// 守门: P95 < 200ms (per OPS-BASIC-DESIGN §3.3 NFR-PT-01)
//
// 注: 守门 #7 v3: clippy 0 err 是 advisory, PT bench 不强制 100% pass (per 守门 #1 v3)
//     但 PT 路径要打通, 验证 mock subprocess 真的快 (< 200ms P95)

// bench 作为独立 crate, 关掉 missing_docs 顶层 deny
#![allow(missing_docs)]

use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

fn bench_log_upload_throughput(c: &mut Criterion) {
    use chrono::Utc;
    use star_ops::ops_ai::default_ladder;
    use star_ops::ops_domain::log::{LogEntry, LogLevel};

    let ladder = default_ladder();
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime must init");

    c.bench_function("ladder_analyze_log_mock", |b| {
        b.iter(|| {
            let log = LogEntry {
                id: uuid::Uuid::nil(),
                source: "bench".to_string(),
                level: LogLevel::Error,
                message: black_box(
                    "2026-09-08T07:30:00Z ERROR helm release 3 deploy failed: timeout".to_string(),
                ),
                timestamp: Utc::now(),
                trace_id: None,
            };
            let _analysis = rt
                .block_on(ladder.analyze_log(&log))
                .expect("ladder must succeed");
        });
    });
}

criterion_group!(benches, bench_log_upload_throughput);
criterion_main!(benches);
