# Brief wt-sb-01-subtask-binding-impl: Sub-task Binding v0.4 实装 (M-26..M-33 + UI + API + 3 表 DDL + 5 E2E + 3 IT + 7 UT)

> **Status**: 🟡 Active (per 2026-09-09 22:31 JST 用户发令"开子代理和worktree并行处理" + ask_4b06eee1bba60b2727e8bccb 拍板 4 个 wt 并行 + 逐个 rebase + ff merge, 推荐项)
> **Created**: 2026-09-09
> **Authority**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3 永久代签)
> **Worktree**: `wt-sb-01-subtask-binding-impl` (新)
> **依赖**: Sub-task Binding 文档 v0.1 已 commit `7a46c97` (per AGENTS.md §8 v0.80)
> **跨 session 续**: per 守门 #20 v20 + 守门 #27 v27 (主 session 准备 + 子代理 dispatch + 二次验证 fallback)

---

## 0. 任务目标 (Objective)

在 `wt-sb-01-subtask-binding-impl` worktree 实装 Sub-task Binding v0.4, 涵盖 8 module (M-26..M-33) + UI C-30 UISpawnSubtaskModal + 9 API 端点 + 3 表 DDL + 5 E2E + 3 IT + 7 UT, per [ADR-0052 v1.0](../architecture/2026-08-26-upgrade/adr/0052-subtask-binding.md) §5 实施计划 + [03-detailed-design.md v0.1](../architecture/2026-09-09-subtask-binding/03-detailed-design.md) §1-§5.

## 1. 已知事实 (Known Facts)

- **7 文件已 commit `7a46c97`**: 3 IPA docs + ADR-0052 + PHASE 报告 + AGENTS.md §6/§6.1/§8 + automation-design.md §4.19
- **TMO 7 节点已实装**: per ADR-0046 + PHASE-LANGGRAPH-TMO-IMPL-REPORT v0.3.2 (commit `7b1a432/1d7dc68/ce9b8df` + PR #13 squash `5e5b1c2`)
- **9 SA + SA-10 已落**: per LangGraph 03-detailed §3.5.1-§3.5.10
- **M-N8 spawn_subtask_node 设计**: per 03-detailed §3.3.1.1 7 步 Python 実装
- **8 module M-26..M-33 设计**: spawn_subtask_node / lifecycle_linkage / exclusive_guard / quota_registry / spawn_validator / child_factory / event_bus / token_budget
- **3 表 W/T/M 横展開**: sub_task_binding (Transaction append-only) + sub_task_runtime (Work TTL 24h) + task_quota_config (Master SCD Type 2 + RLS 13 類)
- **守门 #9 v3**: 调试控制台走 subprocess 替代 RPC (子代理 RPC 不可靠实证 10/10 失败)
- **守门 #19 v19**: agent 交互 Python 化 (走 `scripts/automation/task_ops.py spawn_subtask`)
- **守门 #7 0 unsafe**: C-24 ExclusiveBindingGuard 编译期 + 运行期双层守门
- **守门 #13 c/d W/T/M**: 3 表 100% 覆盖, RLS 13 類必携
- **守门 #14 v3**: Mavis 永久代签全部签字栏

## 2. 路径 (Ruled-out Paths)

- ❌ **主分支直接实装** (在 worktree 操作, per 守门 #9 主体规则)
- ❌ **重写 v0.1 文档** (per 守门 #1 禁回溯叙事)
- ❌ **派 worker 子代理** (RPC 不可靠实证, 走 subprocess.run 替代)
- ❌ **cargo test --workspace** (per 守门 #1 v25 panic 实证, 改单 crate)

## 3. 范围 (Exact Scope)

### 3.1 在 worktree `wt-sb-01-subtask-binding-impl` 新建 / 修改

```
crates/  # 新增 crates (or 扩展现有 crate, per 03 §1 模块结构)
├── task_ops/spawn_subtask/  # 8 module
│   ├── spawn_subtask_node.py   # M-26
│   ├── lifecycle_linkage.py    # M-27
│   ├── exclusive_guard.py      # M-28
│   ├── quota_registry.py       # M-29
│   ├── spawn_validator.py      # M-30
│   ├── child_factory.py        # M-31
│   ├── event_bus.py            # M-32
│   └── token_budget.py         # M-33
├── scripts/automation/
│   └── task_ops.py spawn_subtask  # 守门 #19 Python 化
├── frontend/src/app/tasks/
│   ├── components/SpawnSubtaskModal.tsx  # C-30
│   └── api/tmo/spawn_subtask/route.ts    # 9 端点
└── db/migrations/
    ├── 001_sub_task_binding.sql       # Transaction append-only
    ├── 002_sub_task_runtime.sql       # Work TTL 24h
    └── 003_task_quota_config.sql      # Master SCD Type 2 + RLS 13 類

tests/
├── ut/test_ut_27..33.py     # 7 unit test
├── it/test_it_13..15.py     # 3 integration test
└── e2e/test_e2e_14..18.py   # 5 E2E test

docs/reports/PHASE-SUBTASK-BINDING-IMPL-REPORT.md v0.2  # 实装实证
```

### 3.2 SubAgentState 4 字段扩展 (per [02-basic §3.2.1.2](../architecture/2026-09-09-subtask-binding/02-basic-design.md))

```python
class SubAgentState(TypedDict, total=False):
    # 已有 5 血缘字段 (per LangGraph 03 §3.2.1.1)
    parent_task_id: Optional[str]
    merged_from: list[str]
    split_into: list[str]
    superseded_by: Optional[str]
    checkpoint_snapshot: Optional[dict]
    # 🆕 4 binding 字段
    exclusive_owner_task_id: Optional[str]   # write-once (per R-12)
    bound_at: Optional[datetime]
    bound_by: Optional[str]
    bound_via: Optional[Literal["ui_rightclick", "l0_chat", "api"]]
```

### 3.3 TopAgentState 1 字段扩展

```python
class TopAgentState(TypedDict, total=False):
    # ...
    # 🆕 1 字段
    last_spawn_subtask_result: Optional[SpawnSubtaskResult]
```

## 4. 验收 (Acceptance Criteria)

### 4.1 守门合规 (per [02-basic §4 14 守门 + 03-detailed §6](../architecture/2026-09-09-subtask-binding/02-basic-design.md))

| # | 守门 | 验证 |
|---|---|---|
| 1 | `cargo check --workspace --all-targets -j 4` 0 err | 实装完成后 |
| 2 | `cargo fmt --check` 0 diff | 实装完成后 |
| 3 | `cargo clippy --all-targets -- -D warnings` 0 warning | 实装完成后 (per 守门 #7) |
| 4 | `cargo test -p <affected-crate> --lib -j 4` 0 fail (单 crate, 跳 workspace per 守门 #1 v25) | 实装完成后 |
| 5 | E2E-14..18 5/5 pass | per 03 §4.6 |
| 6 | UT-27..33 7/7 pass | per 03 §4.4 |
| 7 | IT-13..15 3/3 pass | per 03 §4.5 |
| 8 | 3 表 DDL 跑通 (PostgreSQL Tier 3 待 5 域 Lead 真人到位) | 实装完成后 |

### 4.2 跨 session 续做 (per 守门 #20 v20 + 守门 #27 v27)

- **本次 session (mvs_0649e004aebc41259a4ec93a093e9e3)**: Phase A 准备 (worktree + brief + 报告)
- **下次 session**: M-26..M-33 8 module 实装 (估 ~1.05M tokens, 1-2 session)
- **后续 session**: UI + API + 3 表 DDL + 5 E2E + 3 IT + 7 UT (估 ~0.5M tokens)
- **merge session**: cargo check 0 err + git log --follow + rebase + ff merge

### 4.3 commit message 引用 brief 路径 (per 守门 #21 v21)

```bash
git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' commit -m "feat(subtask-binding): M-26 spawn_subtask_node 7 步 Python 実装

  per docs/briefs/wt-sb-01-subtask-binding-impl.md (Sub-task Binding v0.4 实装 brief).
  ..."
```

## 5. 守门硬约束 (per AGENTS.md §4 + §4.1 累积规 v1-v24 + §4.1.1 v3x 候选)

- 守门 #9 v3: 调试控制台走 subprocess 替代 RPC
- 守门 #9 v19: agent 交互 Python 化 (走 scripts/automation/task_ops.py)
- 守门 #9 v20: 子代理 dispatch 必先 brief 落档
- 守门 #12: 缺标比错标, 7 已知缺口 G-SB-1..7 显式列
- 守门 #13 a: L1↔L1 禁止通信 → M-N8 全部 L0 协调
- 守门 #13 c/d: DB W/T/M 横展開, RLS 13 類必携
- 守门 #19 v19: agent 交互 Python 化
- 守门 #21 v21: [P] docs 同步必更新 §4 + registry
- 守门 #27 v27 候选: 子代理 RPC 失败 fallback (verify 二次验证 + retry 2 次 + root session 实装)
- 守门 #28 v28 候选: Mavis 拍板时必带推荐项 (本次 ask_4b06eee1 已带 ✅)
- 守门 #1 v25: cargo test 改单 crate, 跳 workspace
- 守门 #1 v22: 调试控制台不污染 main 编译

## 6. 引用 (References)

- [ADR-0052 Star Sub-task Binding 路径 v1.0](../architecture/2026-08-26-upgrade/adr/0052-subtask-binding.md) — 决策记录
- [01-requirements.md v0.1](../architecture/2026-09-09-subtask-binding/01-requirements.md) — UC-14..UC-18 + F-26..F-32 + NFR-SB-01..05
- [02-basic-design.md v0.1](../architecture/2026-09-09-subtask-binding/02-basic-design.md) — 8 组件 C-23..C-30 + M-N8 + 1 协议 + 5 Reducer + 1 API 端点
- [03-detailed-design.md v0.1](../architecture/2026-09-09-subtask-binding/03-detailed-design.md) — 8 module M-26..M-33 + 7 步 Python 実装 + 3 表 DDL + UT/IT/E2E 矩阵
- [PHASE-SUBTASK-BINDING-IMPL-REPORT.md v0.1](../reports/PHASE-SUBTASK-BINDING-IMPL-REPORT.md) — 21 子项任务完成矩阵 + 14 守门合规
- [LangGraph 03-detailed v0.2 §3.5 Subgraphs](../architecture/2026-09-03-langgraph/03-detailed-design.md) — 9 SA + SA-10 范式
- [AGENTS.md §3 报告 7 段结构 + §4 守门 #1-#24 + §4.1 累积规 v1-v24 + §4.1.1 v3x 候选](../../AGENTS.md)
- [docs/automation-design.md](../../automation-design.md) — agent 交互 Python 化 (守门 #19)
- [docs/data-design/ipa-detail/00-CLASSIFICATION-W-T-M.md v0.1](../../data-design/ipa-detail/00-CLASSIFICATION-W-T-M.md) — DB 三類橫展開

---

**per 守门 #14 v3 Mavis 永久代签**: author = Ulysses <ulysses@mavis.local>, 修订人 = Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手, 审批 = 架构师 (Mavis 接手 agent per DEC-008).