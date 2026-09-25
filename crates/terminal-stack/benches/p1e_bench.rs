//! `p1e_bench.rs` — P1-E Integration benchmarks (per ULYS-232).
//!
//! **目的**: 验证 NFR-PERF-002: 1000 pane scrollback 写入 ≥ 60th write throughput.
//! **MVP v0**: bench core data structures (ScrollbackBuffer + WsHub broadcast + persistence).
//! **不在 P1-E 范围**: 真实 PTY I/O (P2 followup).
//!
//! Run: `cargo bench -p terminal-stack --bench p1e_bench`
//!
//! 守门:
//! - #1 v15 cargo test 实证
//! - #7 `unsafe_code = "forbid"` (workspace lint, no-op for benches)

use std::sync::Arc;
use std::time::Duration as RustDuration;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use uuid::Uuid;

use terminal_stack::scrollback_buffer::{ScrollbackBuffer, ScrollbackLine, ScrollbackSource};
use terminal_stack::ws::hub::WsHub;
use terminal_stack::ws::protocol::ServerMessage;

/// Benchmark 1: ScrollbackBuffer write throughput (per AC-5 NFR-PERF-002).
fn bench_scrollback_buffer_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("scrollback_buffer_write");
    for pane_count in [10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*pane_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(pane_count),
            pane_count,
            |b, &count| {
                b.iter(|| {
                    let mut buffer = ScrollbackBuffer::new(5000);
                    for i in 0..count {
                        let text = format!("benchmark line {i} — payload bytes to fill capacity");
                        buffer.append(text, ScrollbackSource::Stdout).unwrap();
                    }
                    buffer.total_bytes()
                });
            },
        );
    }
    group.finish();
}

/// Benchmark 2: WsHub broadcast throughput (per AC-4 multi-client).
fn bench_ws_hub_broadcast(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("ws_hub_broadcast");
    for client_count in [1, 10, 100].iter() {
        group.throughput(Throughput::Elements(*client_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(client_count),
            client_count,
            |b, &count| {
                b.iter(|| {
                    rt.block_on(async {
                        let hub = WsHub::new();
                        let pane_id = Uuid::new_v4();
                        hub.register_pane(pane_id);
                        let mut streams = Vec::new();
                        for _ in 0..count {
                            if let Ok(s) = hub.subscribe(pane_id) {
                                streams.push(s);
                            }
                        }
                        for seq in 0..100 {
                            let msg = ServerMessage::Output {
                                pane_id,
                                data: format!("broadcast line {seq}"),
                                seq,
                            };
                            let _ = hub.broadcast_message(pane_id, msg);
                        }
                        streams.len()
                    })
                });
            },
        );
    }
    group.finish();
}

/// Benchmark 3: Scrollback persistence in-memory store (per AC-3 + P1-A integration).
fn bench_persistence_in_memory_append(c: &mut Criterion) {
    let mut group = c.benchmark_group("persistence_in_memory_append");
    let count = 1000;
    group.throughput(Throughput::Elements(count));
    group.bench_function(BenchmarkId::from_parameter(count), |b| {
        b.iter(|| {
            let persistence = Arc::new(
                terminal_stack::persistence::TerminalStackPersistence::in_memory().unwrap(),
            );
            let pane_id = Uuid::new_v4();
            let now = chrono::Utc::now();
            for i in 0..count {
                let line = ScrollbackLine {
                    id: Uuid::new_v4(),
                    timestamp: now,
                    text: format!("persistence bench line {i}"),
                    source: ScrollbackSource::Stdout,
                    byte_len: 32,
                };
                persistence.append_scrollback_line(pane_id, &line).unwrap();
            }
        });
    });
    group.finish();
}

criterion_group!(
    name = p1e_benches;
    config = Criterion::default().measurement_time(RustDuration::from_secs(5));
    targets =
            bench_scrollback_buffer_write,
            bench_ws_hub_broadcast,
            bench_persistence_in_memory_append
);
criterion_main!(p1e_benches);
