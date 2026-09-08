// crates/star-ops/benches/docs_bench.rs
//
// F-04 端到端 PT 雏形 (per 守门 #7 v3 + 守门 #1 v3)
//
// 范围: criterion bench, 测 docs_list 端到端延迟
// 守门: P95 < 200ms (per OPS-BASIC-DESIGN §3.4 NFR-PT-01)
//
// 注: 守门 #7 v3: clippy 0 err 是 advisory, PT bench 不强制 100% pass (per 守门 #1 v3)
//     但 PT 路径要打通, 验证 walkdir 真实扫 docs/ 4 子目录能 < 200ms P95
//
// 跟 F-01 cluster_bench / F-02 log_upload_bench / F-03 metrics_bench 同 pattern

// bench 作为独立 crate, 关掉 missing_docs 顶层 deny
#![allow(missing_docs)]

use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use std::sync::Arc;

fn bench_docs_list_throughput(c: &mut Criterion) {
    use star_ops::ops_domain::docs::DocScanner;

    let scanner = Arc::new(DocScanner::new());

    c.bench_function("docs_list_walkdir_4_subdirs", |b| {
        b.iter(|| {
            let docs = scanner.list();
            black_box(docs);
        });
    });
}

fn bench_docs_list_stub_throughput(c: &mut Criterion) {
    use star_ops::ops_domain::docs::DocRef;

    c.bench_function("docs_list_stub_fallback", |b| {
        b.iter(|| {
            let docs = DocRef::stub();
            black_box(docs);
        });
    });
}

criterion_group!(
    benches,
    bench_docs_list_throughput,
    bench_docs_list_stub_throughput
);
criterion_main!(benches);
