//! 1000 provider 探测 benchmark (R9 阶段 3 milestone #1, per plan-032 §4.1)
//!
//! 目标: < 200ms (vs Python 1-2K 秒, 10-20x 加速)
//! 方法: probe_all() 调 200 次 (5 providers × 200 = 1000 probe ops)
//! 标准: cargo bench --bench probe_1000_providers
//!
//! 注: R9 阶段 3 PoC 实测用 src/lib.rs tests::measure_probe_1000_providers (std::time::Instant 测量)
//!     此处 criterion bench 留 R9 阶段 3 后扩展 (current 0 nanos panic due to inner loop too fast)
#![allow(missing_docs)]

use criterion::{criterion_group, criterion_main, Criterion};
use star_registry::RuntimeProbe;

fn bench_probe_1000_providers(c: &mut Criterion) {
    c.bench_function("probe_1000_providers_200_iter", |b| {
        b.iter(|| {
            for _ in 0..200 {
                let mut probe = RuntimeProbe::new();
                let _entries = probe.probe_all();
            }
        });
    });
}

criterion_group!(benches, bench_probe_1000_providers);
criterion_main!(benches);
