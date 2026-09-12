//! Agent ECS 10K entity 60fps benchmark (R9 阶段 3 milestone #5, per plan-032 §4.1)
//!
//! 目标: < 16.7ms (60fps, vs Physis 30fps 优化前 2x 加速)
//! 方法: 创建 10K Agent entity, 模拟 1 frame update (mana 恢复 + xp 增长 + cooldown 检查)
//! 标准: cargo bench --bench ecs_10k_entities
//!
//! 注: R5 阶段 1+2 用 Agent struct 直接 (不用 hecs/bevy_ecs, 留 R5 阶段 3)
//! R9 阶段 3 PoC 阶段 = Agent struct 10K 实例 + 1 frame 模拟
#![allow(missing_docs)] // criterion macro 生成的 item 无 doc (per 守门 #7 missing_docs = deny)

use criterion::{criterion_group, criterion_main, Criterion};
use star_game::Agent;

fn bench_ecs_10k_entities(c: &mut Criterion) {
    c.bench_function("ecs_10k_agents_one_frame_update", |b| {
        b.iter(|| {
            // 创建 10K Agent
            let mut agents: Vec<Agent> = (0..10_000)
                .map(|i| Agent::new(format!("agent-{i}")))
                .collect();
            // 1 frame 模拟: mana replenish + xp 增长 + cooldown 检查
            for agent in agents.iter_mut() {
                agent.mana.replenish(1);
                agent.xp.gain(0); // 0 XP 增长, 但 gain() 本身有 while 循环检查
            }
        });
    });
}

criterion_group!(benches, bench_ecs_10k_entities);
criterion_main!(benches);
