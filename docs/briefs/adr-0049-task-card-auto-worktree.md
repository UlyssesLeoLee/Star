# ADR-0049 Brief: 任务卡自动 worktree + agent 接管

> **Task ID**: `adr-0049-task-card-auto-worktree`
> **Created**: 2026-09-09 04:57 JST
> **Author**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #10 + 8/27 19:39 JST)
> **Source**: Ulysses 2026-09-09 04:57 JST 发令"在面板或者sprint创建任务卡的时候，如果是存在有效agent的任务，应该要求agent自动创建并关联新的worktree，用langgraph实现，这是整个软件的核心功能"
> **联动**: 守门 #1 v19 (Python 化 ≥2 维) + 守门 #9 v20 (子代理 dispatch 必先 brief) + 守门 #12 v21 ([P] 子项 docs 同步)

---

## 1. 目标 (Why)

Star 的核心功能 (per 2026-09-09 04:57 JST 用户拍板):

> **在面板 (board) 或 sprint 创建任务卡时, 如果是存在有效 agent 的任务, 应该要求 agent 自动创建并关联新的 worktree, 用 LangGraph 实现**

这是 Star 的**核心价值主张** — 任务卡 = 业务意图 → 自动转 worktree 物理锚点 + agent Runtime 接管 → 整套 LangGraph 编排不再是 demo, 是真生产链路。

## 2. 已知事实 (Evidence)

- **当前状态**:
  - `domain-worktree` (17 状态机, INV-WT-07 1 WorkItem → 0/1/N Worktree)
  - `domain-task` (L0 TaskQueue stub, v0.0.1, 缺面板/sprint 任务卡层)
  - `domain-agent` (L0/L1/L2 实体已存在)
  - `frontend/src/app/board/page.tsx` Kanban 视图有 `transitionWorkItem` 但无 worktree 触发
  - `frontend/src/app/(app)/sprint/page.tsx` `+ New issue` 表单是 stub (line 21 明确: "per Phase 2+ 真正接入后端 issue create API")
- **已有基础 (TMO v0.2)**:
  - ADR-0046 TMO 7 节点 (M-N1..M-N7) 全部 PoC stub 落地 (per `PHASE-LANGGRAPH-TMO-IMPL-REPORT.md`)
  - `scripts/automation/task_ops/manager.py` + `nodes/*.py` + `protocols.py` + `dag_validator.py` 全部骨架可用
  - 守门 #13 a L1↔L1 禁止 → 所有跨 task 协调必须 L0 (TaskOperationsManager C-16 唯一入口)
  - 守门 #22 调试控制台 port 8080 跟 main 解耦
- **Blocked 路径**:
  - P0-1 17 份 ActorContext 重复 (per 守门 #1 v16 实证, 9/8 已修 246→0 err, 但仍在 H2 阶段 2)
  - H2-EXT 5 domain 跨域字段扩展仍有 290 err + 5 Blocker (per HANDOFF-ST-001 §5.3)
  - domain-task 0.0.1 stub 缺面板/任务卡层 spec (per AGENTS.md §4.2 实装前一致性门)

## 3. 排除路径 (Ruled-out)

- ❌ 前端 store 完全被 LangGraph 接管 — 跟 W5 store 维护责任冲突
- ❌ 1 WorkItem → 0..N Worktree 按需 — 用户已拍 1:1, 跟 ARG §14.11 worktree-as-agent-anchor 一致
- ❌ 异步后台批量 dispatch — 用户拍自动接管, 单卡 5s 内可见到 in_progress
- ❌ 不落 ADR 直接改 — 跨 3 crate 改动不合守门 #21

## 4. 范围 (Scope) + Owner

| # | 子项 | Owner | 落档 |
|---|---|---|---|
| M-N8-01 | 新增 `scripts/automation/task_ops/nodes/create_node.py` (TMO 第 8 节点) | Mavis | `scripts/automation/task_ops/nodes/create_node.py` |
| M-N8-02 | 扩展 `protocols.py` 加 `CreateTaskRequest/Response` | Mavis | `scripts/automation/task_ops/protocols.py` |
| M-N8-03 | 扩展 `manager.py` 加 `create()` 入口 + `OPERATION_TO_NODE["create"] = "M-N8"` | Mavis | `scripts/automation/task_ops/manager.py` |
| M-N8-04 | 前端 `store.createWorkItem` 内嵌检测 assignee 是 agent → 调 LangGraph TMO API (走 console_server.py 8080) | Mavis | `frontend/src/lib/store.ts` + `frontend/src/components/sprint/NewIssueForm.tsx` (新) |
| M-N8-05 | `domain-worktree` 新增 `Worktree` 实体 + 17 状态机 (per `crates/domain-worktree/src/lib.rs` 已有 162 行骨架, 升级 v0.1) | Mavis | `crates/domain-worktree/src/lib.rs` (v0.0.1 → v0.1) |
| M-N8-06 | ADR-0049 + PHASE-REPORT 7 子项 phase 计划 | Mavis | `docs/architecture/2026-08-26-upgrade/adr/0049-*.md` + `docs/reports/PHASE-AUTO-WORKTREE-IMPL-REPORT.md` |
| M-N8-07 | docs/automation-design.md §4 任务卡表 + registry.md 更新 (守门 #21) | Mavis | `docs/automation-design.md` + `scripts/automation/registry.md` |

**Out of scope** (本 ADR 不动):
- P0-1 / H2 阻塞 (per HANDOFF-ST-001, 5 项 Blocker 跨 session 续, 不在本 ADR 范围)
- 5 域 Lead 真人到位 (per 守门 #14 v2, 真人到位后追溯签字, 不影响本 ADR 落地)
- Agent Runtime 深度整合 (per ADR-0045 Hybrid Runtime, 是后续 P3-C 范围)
- 真实 Git worktree CLI 调用 (per 守门 #22 不污染 main 编译, 本期走 memory 模拟 + 1 个 mock shell script)

## 5. 交付物 (Deliverable)

1. **代码** (3 份新文件 + 3 份修改):
   - 新: `scripts/automation/task_ops/nodes/create_node.py` (~150 行, 跟 M-N7 同骨架)
   - 新: `frontend/src/components/sprint/NewIssueForm.tsx` (~120 行, sprint + board 共用)
   - 新: `scripts/automation/_mock_git_worktree.py` (~50 行, mock shell wrapper, 守门 #22)
   - 改: `scripts/automation/task_ops/protocols.py` (+30 行)
   - 改: `scripts/automation/task_ops/manager.py` (+20 行)
   - 改: `frontend/src/lib/store.ts` (+createWorkItem, +isAgent 检测, +LangGraph dispatch)
2. **文档** (3 份):
   - `docs/architecture/2026-08-26-upgrade/adr/0049-task-card-auto-worktree-agent.md` (新, ADR 7 段结构)
   - `docs/reports/PHASE-AUTO-WORKTREE-IMPL-REPORT.md` (新, 7 子项 phase 计划)
   - `docs/automation-design.md` §4 任务卡表 + `scripts/automation/registry.md` (改)
3. **commit** (1 笔, author=Ulysses per 守门 #10):
   - message 含 `per docs/briefs/adr-0049-task-card-auto-worktree.md` (守门 #20)
   - 含 `per scripts/automation/task_ops/nodes/create_node.py` (守门 #19)

## 6. 验收 (Acceptance)

| 维度 | 标准 | 实证方法 |
|---|---|---|
| 功能 | sprint/+ New issue 选 agent assignee → 5s 内 worktree Created → Ready → AgentRunning, 任务卡 status auto in_progress | E2E: store.createWorkItem 后 5s 内 (1) `domain-worktree` 实例化 (2) SA-XX sub-agent 启动 (3) UI 状态切换 |
| 守门 | 守门 #1/4/5/7/9/10/11/12/13/19/20/21/22 全部 0 违反 | 守门表 13 项逐项 ✓ |
| 测试 | UT-27 create_node + 1 IT (worktree 创建 + agent dispatch) + 1 E2E (面板创建 → 5s 接管) | `tests/unit/test_task_ops_nodes.py` + `tests/integration/test_create_node.py` + `tests/e2e/test_uc14_auto_worktree.py` |
| 文档 | 4 份 (ADR + PHASE-REPORT + brief + automation-design) 修订历史齐全 + 守门表 13 项 | 修订历史表 + 守门表 |
| git | commit author = Ulysses, message 含 brief + script 路径 | `git log -1 --format='%an'` + `git log -1 --format='%s'` |
| **缺标比错标** | 6 项已知缺口显式列 (per §3) | PHASE-REPORT §3 |

## 7. 格式

- 简短直陈, 不重复本 brief
- 报告 `PHASE-AUTO-WORKTREE-IMPL-REPORT.md` v0.1 (7 段结构 per AGENTS.md §3)
- 7 子项 phase 估算: ~800K tokens (1 周) — 走守门 #19 Python 化 + 守门 #9 v3 subprocess

---

**Mavis 接手审批通过 (per 2026-09-09 04:57 JST 用户拍板 + 守门 #10 + 8/27 19:39 JST 授权 + 9/8 15:19 JST 第 6 次强化)**
