// crates/star-ops/benches/cluster_bench.rs
//
// F-01 cluster update 端到端 PT 雏形 (per 守門 #7 v3 + 守門 #1 v3)
//
// 范围: criterion bench, 测 cluster 4 endpoint 端到端延迟 (含 helm_canary_mock.sh subprocess)
// 守門: P95 < 200ms (per OPS-BASIC-DESIGN §3.3 NFR-PT-01)
//
// 守門 #7 v3: clippy 0 err 是 advisory, PT bench 不强制 100% pass (per 守門 #1 v3)
// 但 PT 路径要打通, 验证 mock subprocess 真的快 (< 200ms P95)

// bench 作为独立 crate, 关掉 missing_docs 顶层 deny
#![allow(missing_docs)]

use criterion::{criterion_group, criterion_main, Criterion};
use star_ops::ops_api::{router, AppState};
use star_ops::ops_domain::cluster::{CanaryRequest, RollbackRequest};
use std::hint::black_box;

fn bench_cluster_list(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime must init");
    let app = router(AppState::new());

    c.bench_function("cluster_list_subprocess", |b| {
        b.iter(|| {
            let _ = rt.block_on(async {
                use axum::body::Body;
                use axum::http::Request;
                use tower::ServiceExt;
                let _resp = app
                    .clone()
                    .oneshot(
                        Request::builder()
                            .method("GET")
                            .uri("/api/ops/cluster/releases")
                            .body(Body::empty())
                            .unwrap(),
                    )
                    .await
                    .expect("oneshot must succeed");
            });
        });
    });
}

fn bench_cluster_canary(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime must init");

    c.bench_function("cluster_canary_subprocess", |b| {
        b.iter(|| {
            let _ = rt.block_on(async {
                let _ack =
                    star_ops::ops_domain::cluster::HelmRelease::trigger_canary(&CanaryRequest {
                        release_name: black_box("star-mcp".to_string()),
                        canary_weight: black_box(10),
                        target_revision: black_box(Some(4)),
                    })
                    .await
                    .expect("trigger_canary must succeed");
            });
        });
    });
}

fn bench_cluster_status(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime must init");

    c.bench_function("cluster_status_subprocess", |b| {
        b.iter(|| {
            let _ = rt.block_on(async {
                let _status =
                    star_ops::ops_domain::cluster::HelmRelease::status(black_box("star-mcp"))
                        .await
                        .expect("status must succeed");
            });
        });
    });
}

fn bench_cluster_rollback(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime must init");

    c.bench_function("cluster_rollback_subprocess", |b| {
        b.iter(|| {
            let _ = rt.block_on(async {
                let _ack = star_ops::ops_domain::cluster::HelmRelease::rollback(&RollbackRequest {
                    release_name: black_box("star-mcp".to_string()),
                    target_revision: black_box(2),
                })
                .await
                .expect("rollback must succeed");
            });
        });
    });
}

criterion_group!(
    benches,
    bench_cluster_list,
    bench_cluster_canary,
    bench_cluster_status,
    bench_cluster_rollback,
);
criterion_main!(benches);
