# 51. Performance Requirements

> **状态**：🟡 草案 v0.2
> **依赖**：[arch/06 §3](../../arch/06-threat-model-nfr.md) (normative) · [arch/06 §2 NFR 表](../../arch/06-threat-model-nfr.md) (NFR-PERF-001 / NFR-PERF-002 / NFR-PERF-003)

## 1. 性能指标（per P2-9 修复 2026-08-27 跟 arch/06 §3 去重 + 明确 normative 源）

### 1.1 Normative 源 = arch/06 §3 性能表

> **本表为 normative 源**（per [arch/06 §3](../../arch/06-threat-model-nfr.md)）。[arch/06 §2 NFR-PERF-001 / NFR-PERF-002 / NFR-PERF-003](../../arch/06-threat-model-nfr.md) 三条作为验收门指标。**本 spec 不重抄 arch/06 §3 的 7 项，避免双源不一致**。

### 1.2 acceptance/14 补充项（arch/06 §3 未列，per P2-9 修复 2026-08-27 显式标"补充"）

| 性能 | 目标 | 说明 |
|---|---|---|
| `star context get` (minimal) | < 500ms (P95) | arch/06 §3 未列；本 spec 补充 — Phase D 实施时跑 [arch/06 §3 benchmark](../../arch/06-threat-model-nfr.md) 时一起测 |
| Git Provider 操作 | 不慢于 GitHub/GitLab 1.5x | arch/06 §3 未列，但 **arch/06 §2 NFR-PERF-003** 显式定义；本 spec 不重抄 NFR 编号，**只**指 [arch/06 §2 NFR-PERF-003](../../arch/06-threat-model-nfr.md) 为 normative |

### 1.3 性能指标全量引用表（消除重复）

| 性能 | 目标 | Normative 源 |
|---|---|---|
| `star` CLI 启动 | < 100ms | [arch/06 §3](../../arch/06-threat-model-nfr.md) |
| `star agent capabilities --json` | < 50ms | [arch/06 §3](../../arch/06-threat-model-nfr.md) |
| `star task current --json` | < 200ms (P95) | [arch/06 §3](../../arch/06-threat-model-nfr.md) |
| `star code search` | < 1s (P95) | [arch/06 §3](../../arch/06-threat-model-nfr.md) |
| `star submit` 端到端 | < 5s (typical) | [arch/06 §3](../../arch/06-threat-model-nfr.md) |
| MCP tool list | < 1s (with cache) | [arch/06 §3](../../arch/06-threat-model-nfr.md) |
| REST API P95 | < 500ms | [arch/06 §3](../../arch/06-threat-model-nfr.md) |
| `star context get` (minimal) | < 500ms (P95) | [acceptance/14 §1.2](../acceptance/14-performance-requirements.md) 补充 (本 spec 独有) |
| Git Provider 操作 | 不慢于 GitHub/GitLab 1.5x | [arch/06 §2 NFR-PERF-003](../../arch/06-threat-model-nfr.md) |
| `star` CLI 命令响应（P95）| < 200ms (本地) / < 2s (REST) | [arch/06 §2 NFR-PERF-001](../../arch/06-threat-model-nfr.md) |
| MCP tool invoke (P95) | < 500ms | [arch/06 §2 NFR-PERF-002](../../arch/06-threat-model-nfr.md) |

> **冲突来源**（per 子代理 C P2-9）：原 acceptance/14 §1 表 9 项与 arch/06 §3 表 7 项**部分重复**（CLI 启动 / agent capabilities / task current / code search / submit / MCP tool list / REST API P95 7 项重叠），且差异项 `star code search` < 1s / `star submit` 端到端 < 5s 跟 arch/06 §3 完全一致但 acceptance/14 写"P95" arch/06 §3 也写"P95" — **重复而无新增信息**。修法：acceptance/14 §1 改 §1.1 明确"arch/06 §3 = normative"，§1.2 仅列 acceptance/14 补充项，§1.3 加全量引用表消除歧义。

## 2. 性能基线测试

```bash
cargo bench --workspace
# arch/06 §3 NFR 跑通性: per cargo bench 输出对照 arch/06 §3 表
```

## 3. 性能预算（arch/06 §3 未列的微架构级预算）

- 启动时间预算: 50ms Rust binary + 30ms clap parse + 20ms 子命令路由
- JSON 序列化: < 10ms
- 数据库查询: < 50ms (P95)

> §3 是微架构级性能分解（arch/06 §3 不列），仅作为 Phase D 实施时定位瓶颈的参考。

## 4. 测量

CI 跑性能测试，failure 立即报警。CI 失败时**第一检查** [arch/06 §3 7 项](../../arch/06-threat-model-nfr.md) 哪条不达标。

## 5. 实施位置

- `crates/*/benches/` — Cargo bench
- `tests/perf/` — 端到端性能测试
- Phase D 实施时优先对 `arch/06 §3` 7 项 + `arch/06 §2 NFR-PERF-001/002/003` 3 项跑 benchmark，再补 acceptance/14 §1.2 的 2 项

## 6. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-26 | Mavis（per DEC-008）| 初版：§1 性能指标 9 项 | Phase C 54 份 spec 草案 |
| v0.2 | 2026-08-27 | Ulysses（一人公司 12 角色 per DEC-008）| P2-9：§1 跟 arch/06 §3 去重 + 明确 normative 源 = arch/06 §3 · §1.1 标"不重抄 arch/06 §3" · §1.2 列 acceptance/14 补充项（context get + Git Provider 引用 arch/06 §2 NFR-PERF-003） · §1.3 加全量引用表消除歧义 | 8 子代理 INTERFACE-REVIEW-C P2-9 + P1-BLOCKERS-SUMMARY v0.2 |

> v0.2 fix: 2026-08-27 per C-14 (P2-9)
