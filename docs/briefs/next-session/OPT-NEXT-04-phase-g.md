# Brief: OPT-NEXT-04 — Phase G G-1~G-9 (per OPT-WBS-24..32, 推下 session)

**Agent**: worker
**Phase**: OPT-P4-NEXT-SESSION
**Created**: 2026-09-07 12:30 JST
**Status**: 🟢 7/9 done (G.1/3/4-mock/5/6/8/9) | 🟡 2/9 推下 session (G.2 ECS 选型 + G.7 Crash Recovery)
**Last updated**: 2026-09-07 18:43 JST (Mavis 接手代签, per守门 #10 + 8/27 19:39 JST 授权)
**Author**: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签

---

## 1. 任务目标

完成 Phase G 9 子项 (per `STAR-P4-UNIMPL-WBS-001.md` §8):

- **G.1**: L0 SQLite 任务队列 (per `scripts/automation/l0_queue_poc.py`) — 1.5M
- **G.2**: L1 bevy_ecs / flecs 选型 (per `scripts/automation/ecs_bench.py`) — 2M
- **G.3**: EventBus + Mailbox (per `scripts/automation/eventbus_proto.py`) — 1M
- **G.4**: Shared LLM/HTTP/MCP Pool (per `scripts/automation/shared_pool.py`) — 2M
- **G.5**: Tenant Quota + 多租户隔离 (per `scripts/automation/tenant_quota.py`) — 1.5M
- **G.6**: Memory Store (per `scripts/automation/memory_store.py`) — 1M
- **G.7**: Crash Recovery + Checkpoint (per `scripts/automation/recovery_proto.py`) — 1M
- **G.8**: Context Tiering (per AGENTS §6.1 SRS-001 G-1) — 1M
- **G.9**: Token 计量 telemetry (per AGENTS §6.1 SRS-001 G-1) — 1M

**总估 token**: 12M
**部分独立**: G.1-G.9 各自独立, 可跟 B (Phase B) 并行

## 2. 依赖 (per SRS-001 G-1~G-9)

| # | 依赖 | 状态 | 实证 commit (per守门 #12) |
|---|---|---|---|
| 1 | L0 SQLite 任务队列 schema (G-1) | 🟢 **done** (star-taskqueue) | `38d417e` (G.1 L0 SQLite WAL + 6 状态机) |
| 2 | L1 ECS 选型 (G-2) | 🔴 阻塞 (待 DDD Review 拍板, **推下 session**) | — |
| 3 | 22 domain-identity 联动 (G-2 实证) | 🟡 partial (per OPT-A1 §3.5 占位, 依赖 G-2 选型) | — |
| 4 | LLM/HTTP/MCP Pool (G-4) | 🟢 mock 备选落地 (per `053e0ec` improve 7 backend error) | `053e0ec` (provider/http_client/in_memory_backend 改进) |
| 5 | Tenant Quota schema (G-5) | 🟢 **done** (star-quota) | `53becc9` (G.5 3 资源 + SCD Type 2 + 13 類 RLS) |
| 6 | Crash Recovery persistence (G-7) | 🔴 阻塞 (G-DEP-08 PostgreSQL Tier 3 待 5 域 Lead T3) | — |
| 7 | EventBus (G-3) | 🟢 **done** (star-eventbus) | `c5a7897` (G.3 EventBus + Mailbox 3 模式) |
| 8 | Memory Store (G-6) | 🟢 **done** (star-memory) | `82e834b` (G.6 Memory Store 短期/长期 W+T) |
| 9 | Context Tiering (G-8) | 🟢 **done** (star-context-tiering) | `391e7cf` (G.8 L0/L1/L2 上下文分层) |
| 10 | Token 计量 telemetry (G-9) | 🟢 **done** (star-telemetry) | `821d702` (G.9 Token 计量 + Prometheus/OTel 导出) |

**完成 commit 总数**: 6 commit (per守门 #20 1 per file group) + 1 merge `c4f991f`
**测试实证**: 47 tests pass (30 unit + 17 IT)
**cargo check**: 0 err 1.19s 实证 (per守门 #1 v19)

## 3. 实施路径

### G.1 — L0 任务队列

- SQLite WAL mode + tokio async
- TaskQueue trait: `enqueue` / `dequeue` / `ack` / `fail`
- 估 1.5M (跟 `domain-task` 命名冲突 per OPT-A3 §4, 待 ADR 拍板)

### G.2 — ECS 选型 (关键)

- benchmark: bevy_ecs vs flecs vs 自实现
- 性能目标: 1M agent on 16-32GB 单机 (per SRS-001)
- 选型报告: `docs/architecture/2026-09-03-agent-runtime/04-ecs-bench-report.md`

### G.3 — EventBus

- in-process pub/sub + 跨进程 via Redis stream
- Mailbox 模式: at-most-once / at-least-once / exactly-once

### G.4 — Shared Pool

- LLM Pool: 跟 mock 备选 (per `29692a7` 凭证) 切真
- HTTP Pool: connection pool
- MCP Pool: 12+ tool real source (per守门 #4 16 tool 16/16 REAL done per 9/5)

### G.5 — Tenant Quota

- 资源配额: compute / storage / LLM token
- 多租户隔离: RLS 13 類 (per守门 #13 d)

### G.6 — Memory Store

- 短期 / 长期 memory 分层 (per SRS-001 §3)

### G.7 — Crash Recovery

- Checkpoint 3-tier: In-Memory / SQLite / PostgreSQL
- Recovery protocol: state machine replay

### G.8 — Context Tiering

- L0/L1/L2 上下文分层 (per AGENTS §6.1)

### G.9 — Telemetry

- Token 计量: per agent / per call
- Metrics export: Prometheus / OpenTelemetry

## 4. Worktree

```bash
git worktree add -b feat/opt-phase-g D:/Star/.worktrees/wt-opt-phase-g main
cd D:/Star/.worktrees/wt-opt-phase-g
```

## 5. 守门硬约束

- 守门 #1 v19: cargo check 0 err
- 守门 #3: ECS 选型需 DDD Review 拍板
- 守门 #10: author=Ulysses
- 守门 #13: W/T/M 派生
- 守门 #5: 禁打印 secret

## 6. 提交

9 commit (per 9 子项) 或 3 commit (G.1-G.3 / G.4-G.6 / G.7-G.9)

## 7. 失败处理

ECS 选型阻塞 → 报告 + 拍板启动 (per `STAR-P4-OPT-WBS-001.md` §4.2 #5)

## 8. 状态 (更新于 2026-09-07 18:43 JST)

🟢 **主体完成** (7/9 done per commit `c4f991f` merge 14:36 JST):
- G.1 L0 SQLite 任务队列 → star-taskqueue (`38d417e`)
- G.3 EventBus + Mailbox → star-eventbus (`c5a7897`)
- G.4 Shared Pool (mock 备选) → provider/http_client/in_memory_backend (`053e0ec`)
- G.5 Tenant Quota → star-quota (`53becc9`)
- G.6 Memory Store → star-memory (`82e834b`)
- G.8 Context Tiering → star-context-tiering (`391e7cf`)
- G.9 Token 计量 telemetry → star-telemetry (`821d702`)

🟡 **推下 session** (2/9 阻塞, 等外部拍板):
- G.2 ECS 选型 — 等 DDD Review 拍板 (bevy_ecs vs flecs vs 自实现)
- G.7 Crash Recovery — 等 5 域 Lead T3 (~ 2026-09-26 JST) + ADR-0047 PostgreSQL Tier 3 启动 (per G-DEP-08 拍板)

## 9. 引用

- 触发源: `docs/reports/STAR-P4-OPT-WBS-001.md` §4.2 #4
- 基线: `docs/reports/STAR-P4-UNIMPL-WBS-001.md` §8
- SRS: `docs/requirements/SRS-STAR-AGENT-RUNTIME-001.md` G-1~G-9
- Architecture: `docs/architecture/2026-09-03-agent-runtime/02-basic-design.md`
