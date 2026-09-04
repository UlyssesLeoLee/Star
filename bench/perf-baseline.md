# STAR Performance Baseline (Phase H v0.1)

> **状态**：Draft v0.1 (待实测)
> **日期**：2026-08-28
> **修订人**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **触发**：per ADR-0036 §8 D15 / 2026-08-27 21:59 JST 用户授权
> **修订 v0.1.1**：2026-09-05 Rust 1.98.0 → 1.98.1 (per https://blog.rust-lang.org/2026/09/03/Rust-1.98.1/ , 修 vtable UB #161441, Mavis 接手代签 Ulysses)

## §1 测试环境
- CPU: TODO (实测填)
- RAM: TODO
- OS: Windows 11
- Rust: 1.98.1 (仓根 rust-toolchain.toml 锁)
- workspace: 28 crates + 3 new Phase G + 3 new Phase H

## §2 性能基线指标（per ADR-0036 §2 D15）

| 指标 | 目标 | 实测 (Phase H 启动) | 状态 |
|------|------|---------------------|------|
| cargo build 全 workspace | < 60s | TODO | 待测 |
| cargo test 全 workspace | < 30s | TODO | 待测 |
| cargo clippy -D warnings | 0 errors | TODO | 待测 |
| 22 domain handler read | < 1ms p99 | TODO | 待测 |
| Saga 执行 5 步 | < 100ms p99 | TODO | 待测 |
| Cache hit rate (Phase H+) | > 80% | TODO | Phase G+ |

## §3 与 ADR-0036 §2 D15 对齐
- P50 < 50ms
- P95 < 200ms
- P99 < 500ms
- Error rate < 0.1%

## §4 已知缺口
1. 实测基线缺失（Phase H 启动时跑）
2. 跨节点性能未测（Phase G+）
3. Cache 命中率未测（Phase G+）
4. 5 域 Saga 协调性能未测（Phase H+）

## §5 修订历史
| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|------|------|--------|----------|------|
| v0.1 | 2026-08-28 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版：6 指标 + 3 缺口 + 4 状态 | ADR-0036 §8 D15 Phase H |
