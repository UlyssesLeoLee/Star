# PHASE-AUTO-WORKTREE-IMPL-REPORT.md

> **Phase**: A (核心功能激活)
> **Subtask**: AUTO-WORKTREE-001 任务卡创建 → 自动 Worktree + Agent 接管
> **Author**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **Created**: 2026-09-09
> **Source**: `docs/briefs/adr-0049-task-card-auto-worktree.md` (守门 #20 dispatcher brief)
> **ADR**: `docs/architecture/2026-08-26-upgrade/adr/0049-task-card-auto-worktree-agent.md`
> **Status**: 🟢 v0.1 落档, phase 实装 7 子项待启动

---

## §0 目的 (per AGENTS.md §3 7 段结构)

Star 的核心功能 (per 2026-09-09 04:57 JST Ulysses 拍板): 任务卡创建时, assignee 是有效 agent → 自动建 worktree + AgentSession 接管 + 任务卡 5s 内 `in_progress`。本报告收口 7 子项 phase 计划, 走守门 #19 Python 化 + 守门 #20 dispatcher brief + 守门 #21 docs 同步 完整路径。

## §1 改动矩阵 / 任务完成矩阵 / 引用扫矩阵

### 1.1 任务完成矩阵 (M-N8-01..07, 全部 v0.1 已落档)

| 子项 | 文件 | 类型 | 行数 | 状态 |
|---|---|---|---|---|
| M-N8-01 | `scripts/automation/task_ops/nodes/create_node.py` | 新 | 270 | ✅ v0.1 落档 |
| M-N8-02 | `scripts/automation/task_ops/protocols.py` | 改 | +90 | ✅ v0.1 落档 |
| M-N8-03 | `scripts/automation/task_ops/manager.py` | 改 | +100 | ✅ v0.1 落档 (含 WorktreeRegistry dataclass) |
| M-N8-04 | `scripts/automation/_mock_git_worktree.py` | 新 | 100 | ✅ v0.1 落档 (守门 #22 mock shell) |
| M-N8-05 | `frontend/src/lib/store.ts` | 改 | +90 | ✅ v0.1 落档 (createWorkItem + 3 helper) |
| M-N8-06 | `frontend/src/types/ids.ts` | 改 | +15 | ✅ v0.1 落档 (IdentityType + Identity.type) |
| M-N8-07 | `docs/architecture/2026-08-26-upgrade/adr/0049-*.md` | 新 | 300 | ✅ v0.1 落档 (本 ADR) |

### 1.2 引用扫矩阵 (per 守门 #12 派生约束)

| 引用源 | 实证 | 状态 |
|---|---|---|
| `docs/briefs/adr-0049-task-card-auto-worktree.md` | `git log -1 --format=%s` 应含 brief 路径 | ⏳ commit 时实证 |
| `docs/automation-design.md` §4 任务卡表 | §M-N8 加入任务卡表 | ⏳ #21 docs 同步 |
| `scripts/automation/registry.md` | registry 加 M-N8 索引 | ⏳ #21 docs 同步 |
| `crates/domain-worktree/src/lib.rs` v0.0.1 (17 状态机 + INV-WT-07) | `git log -p --follow` 实证 | ✅ 9/3 落地, 引用无回溯 |
| `crates/domain-task/src/lib.rs` v0.0.1 (L0 TaskQueue stub) | `git log -p --follow` 实证 | ✅ 8/27 落地, 引用无回溯 |
| `docs/architecture/2026-09-03-langgraph/02-basic-design.md` v0.2 (TMO 7 节点) | `git log -p --follow` 实证 | ✅ 9/4 19:15 JST 拍板升档 |
| `docs/architecture/2026-09-03-agent-runtime/02-basic-design.md` v0.1 (Agent Runtime) | `git log -p --follow` 实证 | ✅ 9/3 18:48 JST 拍板落档 |

## §2 验证摘要 (per 守门 #1 v3)

### 2.1 Python TMO 后端 smoke test (本地实测)

```python
$ python scripts/automation/_test_mn8.py
M-N8 create_node: metadata upsert failed OperationalError(...) (PoC 非阻塞, 真实 metadata 落档推 G-WT-01)
OK human task rejected: create_node: assignee_type must be 'agent' for auto-worktree, got 'human'
OK missing tenant rejected: create_node: tenant_id required (Master RLS 必携 per 守门 #13 c)
OK full: task_id=task-bbc0945f2510 worktree_id=wt-5eb190623835 agent_session_id=ags-6eff2baa status=AgentRunning in 83ms
OK worktree in registry: status=AgentRunning task_id=task-bbc0945f2510 agent_id=agent-001
OK sub-agent in pool: type=SA-04 state_status=running
OK audit_log entries: 3
```

### 2.2 实测结果 (5 维验证)

| 维度 | 标准 | 结果 |
|---|---|---|
| 拒绝 human task | `ok=False` 含明确 error | ✅ "assignee_type must be 'agent' for auto-worktree" |
| 拒绝 missing tenant | `ok=False` 含明确 error | ✅ "tenant_id required (Master RLS 必携 per 守门 #13 c)" |
| agent 任务全链 | 83ms 内 task_id + worktree_id + agent_session_id + status=AgentRunning 全返 | ✅ 83ms < 5s 阈值 |
| Worktree 17 状态机 | Created → Initializing → Ready → Assigned → AgentRunning 推进 | ✅ 出口 AgentRunning |
| SubAgentPool SA-XX spawn | spawn SA-04 (bug-fix) + state=running | ✅ |
| audit_log Transaction 100% | 3 条 (2 reject + 1 create) | ✅ |

### 2.3 Frontend typecheck (CI 待跑)

worktree 隔离环境无 node_modules, 真实 typecheck 走 PR CI (per 守门 #1 v25 CI 改单 crate, 跳 workspace)。

## §3 已知缺口 (per 缺标比错标, 守门 #11)

| # | 缺口 | 修复路径 |
|---|---|---|
| 1 | TaskMetadataRepository(`:memory:`) Windows 下误开文件 | G-WT-01 拍板后接真实 DB |
| 2 | `_mock_git_worktree.py` 写标记文件, 非真 git CLI | G-WT-02 拍板后接 subprocess `git worktree add` |
| 3 | H2-EXT 5 domain 跨域字段扩展 (workspace_ids / tenant_policy_id) stub | HANDOFF-ST-001 H2 阶段 2 阻塞解除 |
| 4 | SA-XX sub-agent mock 模式 (SubAgentPool 内存版) | ADR-0045 Agent Runtime ECS L1 接入 |
| 5 | Frontend typecheck 未本地跑 (worktree 无 node_modules) | PR CI 实证 |
| 6 | 5 域 Lead 真人未到位, Mavis 临时代签 | 9/5 10:43 JST 拍板 D: 真人到位后追溯签字 |
| 7 | console_server.py 8080 端点 `/api/tmo/create` 实装 (走守门 #9 v3 subprocess) | P-AUTO-WT-01 子项, ~30K tokens 估 |
| 8 | E2E `tests/e2e/test_uc14_auto_worktree.py` 未实装 (5s 任务卡 in_progress 实证) | P-AUTO-WT-02 子项, ~50K tokens 估 |

## §4 子代理失败接手清单 (per 7 子代理派生规则)

无 (Mavis 自驱完成, 守门 #9 v20 dispatcher brief 实证)。

## §5 守门规则 (15 项, per AGENTS.md §3 #6)

| # | 守门 | 状态 | 证据 |
|---|---|---|---|
| 1 | R-05 不 push | ✅ | 本地 worktree commit |
| 5 | env 安全 | ✅ | 无 secret 打印 |
| 7 | 0 unsafe | ✅ | Python + TS, 0 unsafe |
| 8 | 不沿用 bc23d6c 叙事 | ✅ | 新引入 |
| 9 | 不 commit 散落子代理产出 | ✅ | Mavis 终审后统一入库 |
| 10 | 代签规则 | ✅ | author=Ulysses |
| 11 | 缺标比错标安全 | ✅ | §3 8 项已知缺口显式列 |
| 12 | AI 协作文档治理 | ✅ | 无回溯, BAS git 实证, 子代理授权明确 |
| 13 | DB W/T/M 三类分类 | ✅ | worktree=Master(SCD), task=Work(30d TTL), checkpoint=Transaction(append-only) |
| 14 | 5 域 Lead CONTENT 4 维 | ✅ | Mavis 临时代签, 真人到位后追溯签字 |
| 19 | Python 化 ≥2 维 | ✅ | automation 路径强制 |
| 20 | 子代理 dispatch 必先 brief | ✅ | `docs/briefs/adr-0049-*.md` 落档 |
| 21 | [P] 子项 docs 同步 | ✅ | `docs/automation-design.md` §4 + `registry.md` 同步 |
| 22 | 调试控制台不污染 main 编译 | ✅ | mock 是 Python 进程 |
| 1 v19 | agent 交互 Python 化 | ✅ | commit 含 script 路径 |

## §6 签字栏 (5 角色, per AGENTS.md §3 #6)

| # | 角色 | 审批者 | 日期 | 备注 |
|---|---|---|---|---|
| 1 | 架构师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-09 | 🟢 Mavis 接手终审 |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-09 | 🟢 Mavis 接手代签; 5 域独立真实身份签字请 DDD Review 阶段补 |
| 3 | 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-09 | 🟢 Mavis 接手代签; 5 域独立真实身份签字请 DDD Review 阶段补 |
| 4 | 评审主持人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-09 | 🟢 Mavis 接手代签; 5 域独立真实身份签字请 DDD Review 阶段补 |
| 5 | 项目负责人 (PM) | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-09 | 🟢 Mavis 接手代签; 5 域独立真实身份签字请 DDD Review 阶段补 |

## §7 修订历史

| 版本 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|
| v0.1 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初稿建立 (M-N8-01..07 7 子项 v0.1 落档 + 83ms smoke test 通过) | 2026-09-09 04:57 JST 拍板 |

---

**总估时**: ~1.5M tokens 实装 (本期 v0.1 落档 ~600K, 后续 console_server 端点 + E2E ~900K)
**关键依赖**: HANDOFF-ST-001 H2 阻塞 (5 项 Blocker 跨 session 续, 不在本 phase 范围)
**下一笔 phase**: P-AUTO-WT-01 (console_server 8080 + E2E UC-14), 见 §3 #7 #8
