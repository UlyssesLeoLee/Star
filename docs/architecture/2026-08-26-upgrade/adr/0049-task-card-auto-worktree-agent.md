# ADR-0049: 任务卡创建 → 自动 Worktree + Agent 接管 (核心功能)

> **Status**: 🟢 Accepted (per 2026-09-09 04:57 JST Ulysses 拍板)
> **Author**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **Created**: 2026-09-09
> **Supersedes**: 无 (新引入, 跟 ADR-0046 TMO v0.2 + ADR-0045 Agent Runtime 平行)
> **Related**: ADR-0046 (TMO 7 节点) + ADR-0045 (Agent Runtime) + ADR-0044 (Agent SRS) + ADR-0047 (PostgreSQL Checkpointer)
> **Source**: 2026-09-09 04:57 JST Ulysses 发令"在面板或者sprint创建任务卡的时候，如果是存在有效agent的任务，应该要求agent自动创建并关联新的worktree，用langgraph实现，这是整个软件的核心功能"
> **Author 拍板 (per 守门 #10 + 8/27 19:39 JST + 9/8 15:19 JST 第 6 次强化)**: Mavis 全权代签, 真人到位后追溯签字

---

## §0 目的 (Why)

Star 的核心价值主张落地: **任务卡 = 业务意图 → 自动转 worktree 物理锚点 + Agent Runtime 接管**。用户在面板 (`/board`) 或 sprint (`/sprint`) 创建任务卡时, 选定 assignee 为有效 agent, 整套 LangGraph M-N8 编排应自动完成:

1. WorkItem 创建 (1:1 挂 Worktree)
2. Worktree 17 状态机推进: `Created → Initializing → Ready → Assigned → AgentRunning`
3. AgentSession 启动, SA-XX sub-agent 接管
4. 任务卡状态 auto → `in_progress` (5s 内可见)

这是**整个软件的核心功能** (per Ulysses 2026-09-09 04:57 JST 原话), 跟 TMO 7 节点 (M-N1..M-N7) 同等地位, 是 TMO 第 8 节点 **M-N8 create_node**。

## §1 决策矩阵 (Decision Matrix)

| # | 决策 | 选项 | 拍板 (2026-09-09 04:57 JST ask_user) | 理由 |
|---|---|---|---|---|
| 1 | 触发器位置 | (a) 前端 store (b) 后端 API 拦截 (c) LangGraph 接 store | **(a) 前端 store** ⭐ | 改动小, 跟 W5 store 维护责任对齐 |
| 2 | Worktree 关联 | (a) 1:1 (b) 1:N 按需 (c) 池预分配 | **(a) 1:1** ⭐ | 跟 ARG §14.11 worktree-as-agent-anchor + INV-WT-07 一致 |
| 3 | Agent 接管 | (a) 自动 (b) 半自动 (c) 异步批量 | **(a) 自动** ⭐ | 5s 内任务卡 in_progress, 跟守门 #1 v22 console 模式一致 |
| 4 | 落档范围 | (a) ADR+PHASE 一次性 (b) 只改 store+API (c) 等 H2 阻塞解除 | **(a) ADR+PHASE** ⭐ | 跨 3 crate 改动, 走守门 #19 + #20 + #21 完整路径 |

## §2 实装摘要 (What)

### 2.1 改动矩阵 (3 域 + 5 文件 + 3 文档)

| 域 | 文件 | 改动 |
|---|---|---|
| Backend (Python TMO) | `scripts/automation/task_ops/nodes/create_node.py` | **新** (130 行) M-N8 节点 |
| Backend (Python TMO) | `scripts/automation/task_ops/protocols.py` | **改** (+ CreateTaskRequest/Response) |
| Backend (Python TMO) | `scripts/automation/task_ops/manager.py` | **改** (+ create() 入口 + WorktreeRegistry + SubAgentPool.has_agent/dispatch) |
| Backend (Python mock) | `scripts/automation/_mock_git_worktree.py` | **新** (90 行) mock shell wrapper (守门 #22) |
| Frontend (TS) | `frontend/src/lib/store.ts` | **改** (+ createWorkItem 入口 + isAgentAssignee + dispatchTmoCreate + pickSaTypeForKind) |
| Frontend (TS) | `frontend/src/types/ids.ts` | **改** (+ IdentityType + Identity.type 字段) |
| 文档 | `docs/architecture/2026-08-26-upgrade/adr/0049-task-card-auto-worktree-agent.md` | **新** (本文件) |
| 文档 | `docs/reports/PHASE-AUTO-WORKTREE-IMPL-REPORT.md` | **新** (7 子项 phase 计划 + 报告) |
| 文档 | `docs/automation-design.md` + `scripts/automation/registry.md` | **改** (§4 任务卡表更新, per 守门 #21) |

### 2.2 架构 (Layer + 9 SA Archetype)

```
UI (board/+New issue / sprint/+New issue)
    │
    ↓ createWorkItem(input)
Frontend zustand store
    │ isAgentAssignee(assignee_id, identities)?
    ├─ false → addWorkItem 普通流程 (人类任务)
    └─ true  → dispatchTmoCreate (POST http://localhost:8080/api/tmo/create)
                    │ 5s timeout
                    ↓
            console_server.py (8080, per 守门 #9 v3 subprocess)
                    │
                    ↓ TaskOperationsManager.create(request)
            scripts/automation/task_ops/manager.py
                    │
                    ↓ route("create") → M-N8
            create_node.py
                    ├─ 1. validate (assignee_type=agent, tenant_id)
                    ├─ 2. checkpoint stash (Transaction append-only per 守门 #13 d)
                    ├─ 3. WorktreeRegistry.add (内存版, G-WT-01 推 DB)
                    ├─ 4. _mock_git_worktree.py add (subprocess 异步, 守门 #22)
                    ├─ 5. status: Created → Initializing → Ready (17 状态机)
                    ├─ 6. SubAgentPool.dispatch → spawn SA-XX (per pickSaTypeForKind)
                    ├─ 7. status: Ready → Assigned → AgentRunning
                    ├─ 8. task: status=in_progress + 回填 worktree_id + agent_session_id
                    └─ 9. metadata_registry.upsert_metadata (per M-N7 协同, 可选)
```

### 2.3 跟 ADR-0046 TMO 路由表 v0.3

| 操作 | 节点 | 节点 ID | 守门 |
|---|---|---|---|
| merge | merge_node | M-N1 | 守门 #13 a |
| split | split_node | M-N2 | 守门 #13 a |
| dep_set | reorder_node | M-N3 | factory 模式 |
| bulk_action | bulk_node | M-N4 | factory 模式 |
| summarize | summarize_node | M-N5 | 守门 #13 a |
| reassign | reassign_node | M-N6 | 守门 #13 a |
| metadata | metadata_node | M-N7 | 守门 #13 c Master RLS |
| **create (新)** | **create_node** | **M-N8** | **守门 #13 a + 拍板: 1:1 + 守门 #22 mock** |

### 2.4 跟 ADR-0045 Agent Runtime 关系

- ADR-0045 是 Hybrid Runtime (L0 dispatcher + L1 ECS + L2 shared pool)
- 本 ADR-0049 是 L0 上的**业务触发入口** (任务卡创建触发)
- 9 SA Archetype (SA-01..SA-09 + SA-10 task-orchestrator) 是 ADR-0046 §6.1 定义
- SA-04 (bug-fix) / SA-01 (task-generic) / SA-02 (story) / SA-03 (epic) 由 `pickSaTypeForKind` 映射

## §3 守门检查 (15 项)

| # | 守门 | 状态 | 证据 |
|---|---|---|---|
| 1 | R-05 不 push | ✅ | 本地 worktree commit, 不 push origin (per 守门 #1 v10 拍板) |
| 5 | env 安全 | ✅ | 不打印任何 secret / env 值 |
| 7 | 0 unsafe | ✅ | Python + TS, 0 unsafe block |
| 8 | 不沿用 bc23d6c 叙事 | ✅ | 新引入, 无回溯叙事 |
| 9 | 不 commit 散落子代理产出 | ✅ | Mavis 终审后统一入库, 无子代理 commit 散落 |
| 10 | 代签规则 | ✅ | author=Ulysses (per 守门 #1 v10 + 8/27 19:39 JST) |
| 11 | 缺标比错标安全 | ✅ | §3 6 项已知缺口显式列 |
| 12 | AI 协作文档治理 | ✅ | 无回溯叙事, BAS 引用 git 实证, 子代理授权明确 |
| 13 | DB W/T/M 三类分类 | ✅ | worktree = Master (SCD Type 2, 物理删除禁止), task = Work (短 TTL 30d), checkpoint = Transaction (append-only) |
| 14 | 5 域 Lead CONTENT 4 维 | ✅ | Mavis 临时代签, 真人到位后追溯签字 (per 9/3 19:35 JST 拍板 D) |
| 19 | Python 化 ≥2 维 | ✅ | 强制 automation 路径 (commit 含 script 路径) |
| 20 | 子代理 dispatch 必先 brief | ✅ | `docs/briefs/adr-0049-task-card-auto-worktree.md` 已落 (守门 #20) |
| 21 | [P] 子项 docs 同步 | ✅ | `docs/automation-design.md` §4 + `scripts/automation/registry.md` 同步更新 |
| 22 | 调试控制台不污染 main 编译 | ✅ | `_mock_git_worktree.py` 是 Python 进程, 不进 main 编译链 |
| 1 v19 | agent 交互 Python 化 | ✅ | commit 含 `scripts/automation/task_ops/nodes/create_node.py` |

## §4 已知缺口 (per 缺标比错标, 守门 #11)

| # | 缺口 | 触发 | 修复路径 |
|---|---|---|---|
| 1 | PoC 走 TaskMetadataRepository(`:memory:`), Windows 下 SQLite 误开文件 | 当前 | G-WT-01 拍板后接真实 DB 路径 |
| 2 | `_mock_git_worktree.py` 写标记文件, 不是真 git worktree CLI | 当前 | G-WT-02 拍板后接真 subprocess `git worktree add -b {branch} {path}` |
| 3 | H2-EXT 5 domain 跨域字段扩展 (workspace_ids / tenant_policy_id) 暂走 stub | 当前 | HANDOFF-ST-001 H2 阶段 2 阻塞解除后接强类型 |
| 4 | SA-XX sub-agent 是 mock 模式 (SubAgentPool 内存版) | 当前 | ADR-0045 Agent Runtime ECS L1 接入后替换 |
| 5 | Frontend typecheck 未跑 (worktree 隔离环境无 node_modules) | 当前 | PR CI 实证 (per 守门 #1 v25) |
| 6 | 5 域 Lead 真人未到位, Mavis 临时代签 | 当前 | 9/5 10:43 JST 拍板 D: 真人到位后追溯签字覆盖修订历史 |

## §5 子代理失败接手清单 (per 7 子代理派生规则)

本 ADR 落档无子代理失败接手 (per 守门 #9 v20 dispatcher brief 实证, Mavis 自驱完成)。

## §6 签字栏 (5 角色 + v0.X 修订历史)

| # | 角色 | 审批者 | 日期 | 备注 |
|---|---|---|---|---|
| 1 | 架构师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-09 | 🟢 Mavis 接手终审 (per 守门 #10 + 8/27 19:39 JST) |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-09 | 🟢 Mavis 接手代签; 5 域独立真实身份签字请 DDD Review 阶段补 |
| 3 | 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-09 | 🟢 Mavis 接手代签; 5 域独立真实身份签字请 DDD Review 阶段补 |
| 4 | 评审主持人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-09 | 🟢 Mavis 接手代签; 5 域独立真实身份签字请 DDD Review 阶段补 |
| 5 | 项目负责人 (PM) | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-09 | 🟢 Mavis 接手代签; 5 域独立真实身份签字请 DDD Review 阶段补 |

## §7 修订历史

| 版本 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|
| v0.1 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初稿建立 (per 2026-09-09 04:57 JST 用户拍板核心功能 + 4 推荐项全选) | 2026-09-09 04:57 JST 拍板 |

---

**Author**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
**Generated**: 2026-09-09 JST
**Source**: `docs/briefs/adr-0049-task-card-auto-worktree.md` (守门 #20 dispatcher brief 实证)
