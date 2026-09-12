//! WBS 5 态状态机 100K task 吞吐 benchmark (R9 阶段 3 milestone #4, per plan-032 §4.1)
//!
//! 目标: > 100K task/秒 (vs Jira 10K task/秒, 10x 加速)
//! 方法: 创建 100K WbsTaskRow, transition_batch Pending → InProgress
//! 标准: cargo bench --bench transition_batch_100k
#![allow(missing_docs)] // criterion macro 生成的 item 无 doc (per 守门 #7 missing_docs = deny)

use criterion::{criterion_group, criterion_main, Criterion};
use star_task::{ActorId, ActorType, TaskId, WbsTaskRow};
use std::time::SystemTime;
use uuid::Uuid;

fn bench_transition_batch_100k(c: &mut Criterion) {
    c.bench_function("transition_batch_100k_tasks", |b| {
        b.iter(|| {
            let now = SystemTime::now();
            // 100K task: Pending → InProgress
            for i in 0..100_000 {
                let mut row = WbsTaskRow::new(TaskId(Uuid::new_v4()), now);
                let actor_id = ActorId(Uuid::new_v4());
                let _ = row.transition(
                    star_task::TaskStatus::InProgress,
                    ActorType::System,
                    actor_id,
                    Some(format!("batch-{i}")),
                    now,
                );
            }
        });
    });
}

criterion_group!(benches, bench_transition_batch_100k);
criterion_main!(benches);
