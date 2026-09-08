// crates/star-ops/benches/metrics_bench.rs
//
// F-03 端到端 PT 雏形 (per 守门 #7 v3 + 守门 #1 v3)
//
// 范围: criterion bench, 测 metrics_summary 端到端延迟
// 守门: P95 < 200ms (per OPS-BASIC-DESIGN §3.3 NFR-PT-01)
//
// 注: 守门 #7 v3: clippy 0 err 是 advisory, PT bench 不强制 100% pass (per 守门 #1 v3)
//     但 PT 路径要打通, 验证 star-telemetry 真实调能 < 200ms P95

// bench 作为独立 crate, 关掉 missing_docs 顶层 deny
#![allow(missing_docs)]

use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use std::sync::Arc;
use uuid::Uuid;

fn bench_metrics_summary_throughput(c: &mut Criterion) {
    use star_ops::ops_domain::metrics::MetricsAggregator;

    let agg = Arc::new(MetricsAggregator::new());
    let agent = Uuid::new_v4();
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime must init");

    // 预热: 落 10 次 record_call
    rt.block_on(async {
        for _ in 0..10 {
            agg.record_call(black_box(agent), black_box("gpt-4o"), 100, 50)
                .await;
        }
    });

    c.bench_function("metrics_summary_via_telemetry", |b| {
        b.iter(|| {
            // summary 返 Vec<OpsMetric> (no Result, per wt2 MetricsAggregator::summary)
            let summary = rt.block_on(agg.summary());
            black_box(summary);
        });
    });
}

fn bench_metrics_record_call_throughput(c: &mut Criterion) {
    use star_ops::ops_domain::metrics::MetricsAggregator;

    let agg = Arc::new(MetricsAggregator::new());
    let agent = Uuid::new_v4();
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime must init");

    c.bench_function("metrics_record_call", |b| {
        b.iter(|| {
            rt.block_on(agg.record_call(
                black_box(agent),
                black_box("gpt-4o"),
                black_box(100),
                black_box(50),
            ));
        });
    });
}

criterion_group!(
    benches,
    bench_metrics_summary_throughput,
    bench_metrics_record_call_throughput
);
criterion_main!(benches);
