# Brief: OPT-NEXT-05 — Phase H 3 套新架构实装 + DDD Review 终审 (per OPT-WBS-33..40, 推下 session)

**Agent**: worker
**Phase**: OPT-P4-NEXT-SESSION
**Created**: 2026-09-07 12:30 JST
**Status**: 🟡 末段, 等 Phase E.3 真人 + Phase G ECS 选型
**Author**: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签

---

## 1. 任务目标

完成 Phase H 8 子项 (per `STAR-P4-UNIMPL-WBS-001.md` §9):

- **H.1**: LangGraph PostgreSQL checkpointer (per `scripts/automation/lg_checkpoint.py`) — 1M
- **H.2**: 跨仓 RPC (per `scripts/automation/lg_cross_repo.py`) — 0.5M
- **H.3**: 16 tool sub-agent 経由 call (per `scripts/automation/tool_subagent_bridge.py`) — 1.5M
- **H.4**: State schema v1 migration (per LangGraph 04 文档) — 0.5M
- **H.5**: Tree-sitter Rust crate 引入 (per `scripts/automation/treesitter_init.py`) — 1.5M
- **H.6**: 任务卡 ↔ worktree 1:1 绑定 + react-flow (per `scripts/automation/task_graph_view.py`) — 1M
- **H.7**: symbol resolver 跨文件引用追踪 (per `scripts/automation/symbol_resolver.py`) — 0.5M
- **H.8**: DDD Review 21 份 docs 终审 + 签字栏追溯 (5 角色真人到位) — 1M

**总估 token**: 7.5M
**触发**: Phase E.3 真人 + Phase G ECS 选型 + AGENTS §7 #2 16 tool 真实接入完成

## 2. 依赖

| # | 依赖 | 状态 |
|---|---|---|
| 1 | Phase E.3 5 角色真人到位 (E.5 触发) | 🔴 阻塞 |
| 2 | Phase G ECS 选型 (G.2 拍板) | 🔴 阻塞 |
| 3 | AGENTS §7 #2 16 tool 真实接入 (9/5 已 16/16 REAL ✅) | 🟢 完成 (per 9/5 07:56 JST) |
| 4 | PostgreSQL checkpointer Tier 3 (per ADR-0047) | 🟡 设计 done, 装装 E-1..E-5 待 T3 |
| 5 | 跨仓 RPC (Physis / RGS) 实证 | 🟡 mock 备选 (per 8/26 docs 拓扑) |
| 6 | Tree-sitter Rust crate 引入 (H.5) | 🟢 已 partial 落地 (per `H.5 实证` 7/7 test) |
| 7 | react-flow 任务图视图 (H.6) | 🟡 设计 done, 实装 pending |

## 3. 实施路径

### H.1 — LangGraph PG checkpointer

- ADR-0047 5 张表 schema (per 守门 #13 W/T/M 严格)
- 5 阶段装装 E-1..E-5 (per 9/5 G-DEP-08 拍板)
- Tier 1 In-Memory / Tier 2 SQLite / Tier 3 PostgreSQL

### H.2 — 跨仓 RPC

- Physis: 物理引擎 RPC (Physis 仓代码引用)
- RGS: 5 域业务域 (per守门 #5 完全独立, 跨仓 RPC via CLI/gRPC)
- 5 域 Lead 拍板 RPC 契约 (Q-003 衍生)

### H.3 — 16 tool sub-agent 経由 call

- tool 注册: `crates/star-mcp/src/tools/`
- sub-agent 経由: 跟 LangGraph subgraph 对接
- 16 tool 已 16/16 REAL (per 9/5 07:56 JST)

### H.4 — State schema v1 migration

- LangGraph 04 文档 v0.1 → v0.2 (per 9/4 H.4 拍板)
- migration script: `scripts/automation/state_schema_v1_migration.py`

### H.5 — Tree-sitter Rust crate 引入

- `crates/star-treesitter/` (per H.5 实证 7/7 test)
- Tree-sitter Rust grammar 引入
- AST 解析 + symbol table 构建

### H.6 — 任务卡 ↔ worktree 1:1 绑定

- react-flow 视图: 任务图 + worktree 节点
- 1:1 绑定: 每个 work_item 一个 worktree (per `domain-worktree` 实证)

### H.7 — symbol resolver 跨文件引用追踪

- H.5 Tree-sitter 基础上
- 跨 crate 引用追踪
- use / mod / trait / impl 全覆盖

### H.8 — DDD Review 21 份 docs 终审

- 13 docs (per 守门 #3 报告族)
- 6 P3 报告 (per `STAR-P3-WBS-001.md` v0.6)
- 2 INC-SESSION (per H1 + H2 拍板)
- 5 角色真人签字 (per E.3 触发)

## 4. Worktree

```bash
git worktree add -b feat/opt-phase-h D:/Star/.worktrees/wt-opt-phase-h main
cd D:/Star/.worktrees/wt-opt-phase-h
```

## 5. 守门硬约束

- 守门 #1 v19: cargo check 0 err
- 守门 #3: 真人到位签字
- 守门 #10: author=Ulysses
- 守门 #13: W/T/M 派生
- 守门 #12: 禁回溯叙事

## 6. 提交

8 commit (per 8 子项) 或 3 commit (H.1-H.3 / H.4-H.5 / H.6-H.8)

## 7. 失败处理

- 真人不到位 → 报告阻塞 (per E.3 依赖)
- 仍失败: 报告具体错误, 不 commit

## 8. 状态

🟡 **推下 session 末段** (T3-T5 触发, 估 ~10/17 JST 之后)

## 9. 引用

- 触发源: `docs/reports/STAR-P4-OPT-WBS-001.md` §4.2 #5
- 基线: `docs/reports/STAR-P4-UNIMPL-WBS-001.md` §9
- ADR-0046/0047: `docs/architecture/2026-08-26-upgrade/adr/`
- LangGraph: `docs/architecture/2026-09-03-langgraph/`
- Agent Runtime: `docs/architecture/2026-09-03-agent-runtime/`
