//! CRDT 100 节点并发编辑 benchmark (R9 阶段 3 milestone #3, per plan-032 §4.1)
//!
//! 目标: < 50ms 收敛时间 (vs Miro 500ms+, 10x 加速)
//! 方法: 创建 100 节点 canvas, 模拟 100 并发 edit (阶段 1 MockBackend apply_ops, 阶段 2 yrs CRDT 实际)
//! 标准: cargo bench --bench concurrent_edit_100_nodes
#![allow(missing_docs, unused_imports)] // criterion macro 生成的 item 无 doc + 预留 Uuid 给将来扩展

use criterion::{criterion_group, criterion_main, Criterion};
use star_canvas::{
    Canvas, CanvasBackend, CanvasOp, MockBackend, Node, NodeKind, Position3D, Size3D,
};
use uuid::Uuid;

fn bench_concurrent_edit_100_nodes(c: &mut Criterion) {
    c.bench_function("concurrent_edit_100_nodes_apply", |b| {
        b.iter(|| {
            let mut canvas = Canvas::new("bench");
            // 准备 100 节点
            let mut ops: Vec<CanvasOp> = Vec::with_capacity(100);
            for i in 0..100 {
                let node = Node::new(
                    NodeKind::Rect,
                    Position3D::new_2d(i as f64, i as f64),
                    Size3D::new_2d(100.0, 100.0),
                    format!("node-{i}"),
                    None,
                    "Mavis",
                );
                ops.push(CanvasOp::UpsertNode(node));
            }
            // 模拟 100 并发 edit (按序 apply 模拟)
            let mut backend = MockBackend;
            backend.apply_ops(&mut canvas, ops).unwrap();
        });
    });
}

criterion_group!(benches, bench_concurrent_edit_100_nodes);
criterion_main!(benches);
