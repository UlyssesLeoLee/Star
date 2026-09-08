# P-AUTO-WT-01 Brief: console_server `/api/tmo/create` 端点 + E2E UC-14

> **Task ID**: `p-auto-wt-01-console-server`
> **Created**: 2026-09-09 08:13 JST
> **Author**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #10 + 8/27 19:39 JST)
> **Source**: PHASE-AUTO-WORKTREE-IMPL-REPORT.md §3 #7 #8 (8 项已知缺口, 2 P 子项启动)
> **联动**: 守门 #1 v19 (Python 化 ≥2 维) + 守门 #9 v3 (subprocess 替代 RPC) + 守门 #22 (调试控制台不污染 main 编译) + 守门 #13 a (L0 唯一入口) + 守门 #13 d (Transaction 100% audit)

---

## 1. 目标 (Why)

P-AUTO-WT-01 收口 PHASE-AUTO-WORKTREE-IMPL-REPORT.md §3 已知缺口 #7 + #8:
- **#7** console_server.py 8080 `/api/tmo/create` 端点未实装 (前端 store.dispatchTmoCreate 5s timeout 实测 fail)
- **#8** E2E `tests/e2e/test_uc14_auto_worktree.py` 未实装 (5s 任务卡 in_progress 实证)

## 2. 已知事实 (Evidence)

- **当前状态** (per commit `f8275d4`):
  - `scripts/automation/task_ops/nodes/create_node.py` M-N8 已实装, 83ms smoke pass
  - `scripts/automation/task_ops/manager.py` `create()` + `_create_task()` 已实装
  - `scripts/automation/_mock_git_worktree.py` 已实装
  - `scripts/automation/api/routes_tmo.py` 9 端点已实装 (merge/split/dependencies/reorder/graph/bulk/bulk-health/relationships/metadata), 缺 `/create` (M-N8)
  - `scripts/automation/console_server.py` mount 了 `routes_tmo` router
  - `frontend/src/lib/store.ts` createWorkItem + dispatchTmoCreate 已实装
- **smoke test 实证** (per `f8275d4`):
  - `manager.create({...})` 83ms 内 full happy path 通过
  - 缺 HTTP 端点落地, 前端 `dispatchTmoCreate` fetch 5s timeout 会失败
- **联动**:
  - 守门 #22: console_server.py 跑 8080 后端, 不进 main 编译链
  - 守门 #9 v3: 走 console_server.py subprocess, 不用 RPC
  - 守门 #1 v25: CI cargo test 改单 crate 跳过 workspace, 不影响本 P 子项 (Python only)

## 3. 排除路径 (Ruled-out)

- ❌ 跳过 console_server 端点, 走前端直接调 manager.create — 跟守门 #9 v3 实证冲突 (5/5 subagent RPC 不可靠)
- ❌ 不写 E2E, 只做单元 — 缺标 (per 守门 #11 缺标比错标安全), 不能实证 5s 任务卡 in_progress
- ❌ 加新的 Rust HTTP endpoint — 跨 crate 改动 + 违反守门 #19 Python 化

## 4. 范围 (Scope) + Owner

| # | 子项 | Owner | 落档 |
|---|---|---|---|
| P-AUTO-WT-01-a | `routes_tmo.py` 加 `POST /api/tmo/create` 端点 (M-N8) | Mavis | `scripts/automation/api/routes_tmo.py` (+ ~120 行) |
| P-AUTO-WT-01-b | `routes_tmo.py` 头注释更新: 8 → 9 端点 + M-N8 加入 | Mavis | `scripts/automation/api/routes_tmo.py` (头 14 行) |
| P-AUTO-WT-01-c | `routes_tmo.py` `tmo_operations()` 加 `implemented_nodes: M-N8` | Mavis | `scripts/automation/api/routes_tmo.py` (line 158) |
| P-AUTO-WT-01-d | `console_server.py` mount 注释 + start_print 加 M-N8 | Mavis | `scripts/automation/console_server.py` (line 195, 351) |
| P-AUTO-WT-01-e | 起 console_server 8080 后台, curl 端到端走通 (human reject + agent happy path) | Mavis | bash 实测 |
| P-AUTO-WT-01-f | E2E `tests/e2e/test_uc14_auto_worktree.py` (5s 任务卡 in_progress 实证) | Mavis | `tests/e2e/test_uc14_auto_worktree.py` (新, ~80 行) |
| P-AUTO-WT-01-g | PHASE-AUTO-WORKTREE-IMPL-REPORT.md v0.2 (2 子项收官) | Mavis | `docs/reports/PHASE-AUTO-WORKTREE-IMPL-REPORT.md` (+1 节) |
| P-AUTO-WT-01-h | registry.md v0.7 + automation-design.md v1.0 (守门 #21) | Mavis | 2 docs |

**Out of scope** (本 P 子项不动):
- P0-1 / H2 阻塞 (5 项 Blocker 跨 session 续, 不在本 P 范围)
- 真实 Git worktree CLI 集成 (per G-WT-02 推 H2 阻塞解除后)
- Agent Runtime L1 ECS 接入 (per ADR-0045, 后续 P3-C 范围)
- 5 域 Lead 真人到位 (per 守门 #14 v2 拍板 D, 真人到位后追溯签字)

## 5. 交付物 (Deliverable)

1. **代码** (2 份修改 + 1 份新):
   - 改: `scripts/automation/api/routes_tmo.py` (+~120 行, 加 /create 端点)
   - 改: `scripts/automation/console_server.py` (头注释 + start_print)
   - 新: `tests/e2e/test_uc14_auto_worktree.py` (~80 行)
2. **文档** (3 份):
   - `docs/reports/PHASE-AUTO-WORKTREE-IMPL-REPORT.md` v0.2 (P-AUTO-WT-01 + P-AUTO-WT-02 收口)
   - `docs/automation-design.md` v1.0 (§4.17 + 修订历史)
   - `scripts/automation/registry.md` v0.7 (§1 + §5.3 + 修订历史)
3. **commit** (1 笔, author=Ulysses per 守门 #10):
   - message 含 `per docs/briefs/p-auto-wt-01-console-server.md` (守门 #20)
   - 含 `per scripts/automation/api/routes_tmo.py` (守门 #19)

## 6. 验收 (Acceptance)

| 维度 | 标准 | 实证方法 |
|---|---|---|
| HTTP 端点 | POST `http://localhost:8080/api/tmo/create` 200 OK | curl 实证 |
| human task 拒 | HTTP 400 + error 含 "assignee_type must be 'agent'" | curl 测试 |
| agent task 通 | HTTP 200 + result 含 task_id + worktree_id + agent_session_id + worktree_status=AgentRunning + task_status=in_progress | curl 测试 |
| 5s 任务卡 in_progress | curl 响应 time < 5s (per 拍板 5s 阈值) | curl `-w '%{time_total}'` |
| E2E UC-14 | `tests/e2e/test_uc14_auto_worktree.py` 跑通, 5s 内验证 worktree Ready + AgentSession running + 任务卡 in_progress | `python tests/e2e/test_uc14_auto_worktree.py` |
| 守门 | 守门 #1/4/5/7/9/10/11/12/13/19/20/21/22 全部 0 违反 | 守门表逐项 ✓ |
| 文档 | 3 份修订历史齐全 + 守门表 + 已知缺口 | 修订历史表 + 守门表 |
| git | commit author = Ulysses, message 含 brief + script 路径 | `git log -1 --format='%an'` + `git log -1 --format='%s'` |

## 7. 格式

- 简短直陈, 不重复本 brief
- 报告 `PHASE-AUTO-WORKTREE-IMPL-REPORT.md` v0.2 (7 段结构 per AGENTS.md §3)
- 2 子项 phase 估: ~80K tokens (跟 PHASE-REPORT v0.1 §3 #7 #8 估一致)

---

**Mavis 接手审批通过 (per 2026-09-09 08:13 JST Ulysses "推进" 拍板 + 守门 #10 + 8/27 19:39 JST 授权 + 9/8 15:19 JST 第 6 次强化 + 9/8 15:29 JST 第 7 次强化自驱)**
