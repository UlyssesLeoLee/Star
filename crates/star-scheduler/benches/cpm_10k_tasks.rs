//! CPM 10K task benchmark (R9 阶段 3 milestone #2, per plan-032 §4.1)
//!
//! 目标: < 100ms (vs MS Project 30s+, 300x 加速)
//! 方法: 创建 10K linear task chain (T0 → T1 → ... → T9999), 跑 critical_path()
//! 标准: cargo bench --bench cpm_10k_tasks
#![allow(missing_docs)] // criterion macro 生成的 item 无 doc (per 守门 #7 missing_docs = deny)

use criterion::{criterion_group, criterion_main, Criterion};
use star_scheduler::{Schedule, Scheduler, Task};

fn bench_cpm_10k_tasks(c: &mut Criterion) {
    c.bench_function("cpm_10k_linear_chain", |b| {
        b.iter(|| {
            let mut sched = Schedule::new("bench");
            // 创建 10K linear chain
            let mut prev: Option<star_scheduler::TaskId> = None;
            for i in 0..10_000 {
                let mut task = Task::new(format!("T{i}"), 1);
                if let Some(p) = prev {
                    task.add_predecessor(p);
                }
                let id = task.id;
                sched.add_task(task);
                prev = Some(id);
            }
            let scheduler = Scheduler::new();
            // 跑 critical_path (拓扑序 + ES/EF/LS/LF + critical path tasks)
            let _ = scheduler.critical_path(&sched);
            let _ = scheduler.critical_path_tasks(&sched);
        });
    });
}

criterion_group!(benches, bench_cpm_10k_tasks);
criterion_main!(benches);
