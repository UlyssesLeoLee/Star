# 51. Performance Requirements

> **状态**：🟡 草案 v0.1
> **依赖**：[arch/06 §3](../../arch/06-threat-model-nfr.md)

## 1. 性能指标

| 性能 | 目标 |
|---|---|
| `star` CLI 启动 | < 100ms |
| `star agent capabilities --json` | < 50ms |
| `star task current --json` | < 200ms (P95) |
| `star context get` (minimal) | < 500ms (P95) |
| `star code search` | < 1s (P95) |
| `star submit` 端到端 | < 5s (typical) |
| MCP tool list | < 1s (with cache) |
| REST API P95 | < 500ms |
| Git Provider 操作 | 不慢于 GitHub/GitLab 1.5x |

## 2. 性能基线测试

```bash
cargo bench --workspace
```

## 3. 性能预算

- 启动时间预算: 50ms Rust binary + 30ms clap parse + 20ms 子命令路由
- JSON 序列化: < 10ms
- 数据库查询: < 50ms (P95)

## 4. 测量

CI 跑性能测试，failure 立即报警。

## 5. 实施位置

- `crates/*/benches/` — Cargo bench
- `tests/perf/` — 端到端性能测试

## 6. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。
