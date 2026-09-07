# Brief: OPT-NEXT-07 — G-DEP-01/02/04/05 实装 (per OPT-Q-08..10, 推下 session)

**Agent**: worker
**Phase**: OPT-P4-NEXT-SESSION
**Created**: 2026-09-07 12:30 JST
**Status**: 🟡 等待 P3-F #5 触发
**Author**: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签

---

## 1. 任务目标

实装 G-DEP-01/02/04/05 4 子项 (per `PHASE-LANGGRAPH-TMO-IMPL-REPORT.md` §3.3 + 9/5 报告 §3.3-3.6):

- **G-DEP-01**: P0 工具实装 (TMO-04/06 阻塞) — 3 tool per 9/5 §3.3:
  - `create_merge_request`
  - `create_worktree`
  - `search_issues`
  - 估 0.4-0.6M
- **G-DEP-02**: P1 工具实装 (TMO-05 阻塞) — 4 tool per 9/5 §3.4:
  - `search_code`
  - `get_symbol`
  - `find_references`
  - `get_code_context`
  - 估 0.3-0.5M
- **G-DEP-04**: task_metadata DDL 落地 (TMO-07 内存版 registry 待替换) — `CREATE TABLE task_metadata` + RLS POLICY
  - 已 done (per `G-TMO-04-DDL-IMPL-REPORT.md` 114 行 + `task_metadata_ddl.py` 267 行, commit `5e5b1c2`)
  - 本 brief 跳过
- **G-DEP-05**: LangGraph SDK 0.2.x `interrupt_response` API alpha 确认
  - 已 done (per `G-TMO-05-SDK-FINDINGS.md` + 实装用纯 asyncio + TypedDict)
  - 本 brief 跳过

**总估 token**: 0.7-1.1M (G-DEP-01 + G-DEP-02)

## 2. 依赖 (per 9/5 PHASE-LANGGRAPH-TMO-IMPL-REPORT §3.3)

| # | 依赖 | 状态 |
|---|---|---|
| 1 | TMO-04/06 manager dispatch 实证 (per `manager.dispatch 5/5 ok=True`) | 🟢 完成 (per `feat/tmo-05-06-07` commit `7b1a432`) |
| 2 | TMO-05 manager dispatch 实证 | 🟢 完成 |
| 3 | LangGraph SDK 0.2.x interrupt_response API (G-DEP-05) | 🟢 完成 (per `G-TMO-05-SDK-FINDINGS.md`) |
| 4 | 凭证切真 (per B.5 OpenClaw / B.6 Hermes) | 🟡 mock 备选可维持 (per 9/3 11:35 JST 拍板 A) |
| 5 | P3-F #5 拍板 (TMO 实装触发) | 🟡 等待 |

## 3. 实施路径

### G-DEP-01 P0 工具 (3 tool)

- `create_merge_request`: 跟 `domain-worktree` + `domain-scm` 集成
- `create_worktree`: 跟 `star-mcp/src/tools/create_worktree.rs` 集成
- `search_issues`: 跟 `domain-issue` (待命名) 或 JQL 替代
- 实证 5/5 ok (per TMO manager.dispatch)

### G-DEP-02 P1 工具 (4 tool)

- `search_code`: Tree-sitter AST (per H.5 落地)
- `get_symbol`: symbol table
- `find_references`: H.7 跨文件引用追踪
- `get_code_context`: 上下文窗口 (per G.8 Context Tiering)
- 实证 5/5 ok

## 4. Worktree

```bash
git worktree add -b feat/opt-g-dep D:/Star/.worktrees/wt-opt-g-dep main
cd D:/Star/.worktrees/wt-opt-g-dep
```

## 5. 守门硬约束

- 守门 #1 v19: cargo check 0 err
- 守门 #13 a: TMO 7 节点全 L0 协调 (per 9/4 wt-tmo-01..04 实证 4 类 cycle + O(V+E))
- 守门 #10: author=Ulysses
- 守门 #12: 禁回溯叙事
- 守门 #13: W/T/M 派生 (task_metadata DDL per Master RLS 必携)

## 6. 提交

2 commit (per G-DEP-01 + G-DEP-02)

## 7. 失败处理

- TMO manager.dispatch 5/5 不全 → 报告 + 拍板 (per TMO-05/06 实证)
- 仍失败: 报告具体错误, 不 commit

## 8. 状态

🟢 **可启动** (P3-F #5 触发, 等 Ulysses 拍板)

## 9. 引用

- 触发源: `docs/reports/STAR-P4-OPT-WBS-001.md` §3.2 #10
- 基线: `docs/reports/PHASE-LANGGRAPH-TMO-IMPL-REPORT.md` §3.3
- 9/5 报告: `docs/reports/PHASE-P4-V2-TMO-CI-IMPL-REPORT.md` §3.3-3.6
- TMO manager: `crates/star-tmo/src/manager.rs` (per ADR-0046)
