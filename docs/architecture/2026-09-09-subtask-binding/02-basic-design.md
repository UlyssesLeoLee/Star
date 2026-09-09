# STAR Sub-task Binding 基本設計書 (Basic Design)

> **バージョン**: v0.1
> **ステータス**: 🟡 Draft → 🔵 Review (待 DDD Review 拍板)
> **日付**: 2026-09-09
> **制定者**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **签批**: 🟢 Mavis 接手终审（per 2026-08-27 19:39 + 21:59 JST 用户授权"允许你代签"）
> **父文档**: [01-requirements.md v0.1](./01-requirements.md) · [Star LangGraph 統合アーキテクチャ 基本設計書 v0.2](../2026-09-03-langgraph/02-basic-design.md) · [ADR-0046 TMO 7 节点](../2026-08-26-upgrade/adr/0046-langgraph-task-management-operations.md)
> **兄弟文档**: [03-detailed-design.md v0.1](./03-detailed-design.md) · [ADR-0052 决策记录](../2026-08-26-upgrade/adr/0052-subtask-binding.md) · [PHASE-SUBTASK-BINDING-IMPL-REPORT.md](../../reports/PHASE-SUBTASK-BINDING-IMPL-REPORT.md)
> **依存**: [AGENTS.md §4 守门硬约束](../../AGENTS.md) · [AGENTS.md §4 #13 W/T/M 横展開](../../AGENTS.md) · [AGENTS.md §6.1-6.2 LangGraph view + Agent Runtime view](../../AGENTS.md)

---

## §0 文档目的

新增 **STAR Sub-task Binding 基本設計**: 在 L0 StateGraph 增 1 节点 (M-N8) + 1 协议 (spawn_subtask) + 8 组件 (C-23..C-30) + 1 外部 API 端点 (POST /api/tmo/spawn_subtask) + 5 Reducer, 满足 [01-requirements.md §1 UC-14..UC-18 + §2 F-26..F-32 + §3 NFR-SB-01..05](./01-requirements.md) 全部 5 UC + 7 F + 5 NFR 需求.

**与 [ADR-0046 §2.1 TMO 7 节点](../2026-08-26-upgrade/adr/0046-langgraph-task-management-operations.md) 的关系**:
- 本文档 v0.1 是 ADR-0046 TMO 7 节点的**增量扩展** (7 → 8 节点, 7 → 8 协议, 7 → 8 组件)
- 同步升版 LangGraph 02/03 v0.2 → v0.3 (引用 M-N8 + C-23..C-30)
- 守门 #13 a 强约束 (L1↔L1 禁止通信) 维持: M-N8 全部 L0 协调, 唯一 cross-task actor

---

## §1 架构总览 (Architecture Overview)

### §1.1 8 组件 C-23..C-30 (per [01 §2 F-26..F-32](./01-requirements.md))

| 组件 ID | 名称 | 責務 | 模块 | 引用 |
|---|---|---|---|---|
| **C-23** | `LifecycleLinkageManager` | parent state change → child action 链式触发, ≤ 1s 延迟 (NFR-SB-03) | `task_ops/lifecycle_linkage.py` | [F-29](./01-requirements.md) |
| **C-24** | `ExclusiveBindingGuard` | 1:1 独占守门, 编译期 + 运行期双层, 不可绕过 (NFR-SB-02) | `task_ops/exclusive_guard.py` | [F-30](./01-requirements.md) |
| **C-25** | `SubTaskQuotaRegistry` | per-parent sub-task quota 计量 + 配置 (Master SCD Type 2) | `task_ops/quota_registry.py` | [F-31](./01-requirements.md) |
| **C-26** | `SpawnSubtaskValidator` | spawn 7 步原子化前 5 步校验 (parent state / SA type / quota / self-spawn / nesting depth) | `task_ops/spawn_validator.py` | [F-26](./01-requirements.md) |
| **C-27** | `ChildSubAgentFactory` | child SubAgentState 初始化 + SubAgentRegistry.spawn 入口 | `sub_agent/child_factory.py` | [F-27, F-28](./01-requirements.md) |
| **C-28** | `ParentChildEventBus` | parent/child state change event 订阅 + 派发 (EventBus per [AGENTS.md §6.2](../../AGENTS.md)) | `task_ops/event_bus.py` | [F-29](./01-requirements.md) |
| **C-29** | `SubTaskTokenBudget` | per-sub-task token 预算计量 (TokenTelemetry 集成, NFR-SB-04) | `task_ops/token_budget.py` | [F-31](./01-requirements.md) |
| **C-30** | `UISpawnSubtaskModal` | Next.js 15 前端 modal 组件 + AI 建议 (走 ai_edit_mock.py 守门 #23 v23) | `frontend/src/app/tasks/components/SpawnSubtaskModal.tsx` | [F-32](./01-requirements.md) |

**8 组件 vs ADR-0046 7 组件**:
- 7 → 8: 增量 1 (C-23 LifecycleLinkageManager 是核心, 跟 M-N8 紧耦合)
- 增量逻辑: C-23..C-25 是核心三件套 (lifecycle / exclusive / quota), C-26..C-30 是支撑 (validate / factory / event bus / token budget / UI)

### §1.2 上下文图 (Context Diagram)

```
                                ┌─────────────────────┐
                                │  Ulysses (L0 chat   │
                                │  bar / UI 任务卡)   │
                                └──────────┬──────────┘
                                           │ 自然语言命令
                                           │ "把任务 a 派生 SA-04 sub-task"
                                           ▼
        ┌──────────────────────────────────────────────────────────┐
        │              L0 StateGraph (TMO 8 节点扩展)               │
        │  ┌────────┐  ┌────────┐  ┌────────┐         ┌────────┐  │
        │  │ T-N1   │─→│ T-N2   │─→│ T-N3   │─...───→│ T-N5   │  │
        │  │parse_  │  │dispatch│  │tool    │         │respond │  │
        │  │intent  │  │        │  │        │         │        │  │
        │  └────┬───┘  └────────┘  └────────┘         └────────┘  │
        │       │ intent='spawn_subtask'                            │
        │       ▼                                                  │
        │  ┌────────────────────────────────────────┐              │
        │  │ M-N8 spawn_subtask_node (NEW)          │              │
        │  │ 7 步原子化 (per [03 §3.3.1.1](./03-    │              │
        │  │ detailed-design.md))                   │              │
        │  │  step 1-5: C-26 SpawnSubtaskValidator  │              │
        │  │  step 6:   C-27 ChildSubAgentFactory    │              │
        │  │  step 7:   audit log append             │              │
        │  └────┬─────────────────────────────────────┘              │
        │       │ child_task_id, sa_type, parent_task_id            │
        │       ▼                                                  │
        │  ┌────────────────────────────────────────┐              │
        │  │ SubAgentRegistry (C-16 from TMO)       │              │
        │  │ 1:1 独占 spawn (C-24 ExclusiveGuard 守门)│              │
        │  └────┬─────────────────────────────────────┘              │
        │       │ SubAgentHandle (1:1)                              │
        │       ▼                                                  │
        │  ┌────────────────────────────────────────┐              │
        │  │ L1 sub_pool (9 SA + SA-10)            │              │
        │  │ ┌────────┐ ┌────────┐ ┌────────┐     │              │
        │  │ │ SA-01  │ │ SA-04  │ │ ...    │     │              │
        │  │ │code-rev│ │git-ops │ │        │     │              │
        │  │ └────────┘ └────────┘ └────────┘     │              │
        │  └────────────────────────────────────────┘              │
        │       │ EventBus (C-28)                                  │
        │       ▼                                                  │
        │  ┌────────────────────────────────────────┐              │
        │  │ LifecycleLinkageManager (C-23)        │              │
        │  │ parent cancel → child cancel ≤ 1s     │              │
        │  └────────────────────────────────────────┘              │
        │       │ SubTaskQuotaRegistry (C-25)                      │
        │       │ SubTaskTokenBudget (C-29)                        │
        │       ▼                                                  │
        │  ┌────────────────────────────────────────┐              │
        │  │ TaskRelationshipGraph (C-17 from TMO) │              │
        │  │ parent → child edge (type=spawn_       │              │
        │  │ exclusive) + DAGValidator cycle check  │              │
        │  └────────────────────────────────────────┘              │
        │       │ Audit log (Transaction append-only)              │
        │       ▼                                                  │
        │  ┌────────────────────────────────────────┐              │
        │  │ W/T/M 3 表 (per [01 §4](./01-          │              │
        │  │ requirements.md))                      │              │
        │  │ - sub_task_binding (Transaction)       │              │
        │  │ - sub_task_runtime (Work)              │              │
        │  │ - task_quota_config (Master SCD2)      │              │
        │  └────────────────────────────────────────┘              │
        └──────────────────────────────────────────────────────────┘
                                           │
                                           ▼
                                ┌─────────────────────┐
                                │ UI TaskCard 右键菜单 │
                                │ "Spawn sub-task"   │
                                │ (Next.js 15)        │
                                └─────────────────────┘
```

---

## §2 节点 / 协议 / Reducer

### §2.7.1 M-N8 spawn_subtask_node (NEW)

| 维度 | 内容 |
|---|---|
| **Node ID** | `M-N8 spawn_subtask_node` |
| **責務** | 1 派生 1 创建绑定子任务, 父继续 running, child 1:1 独占 SA-XX |
| **入口 trigger** | (a) UI TaskCard 右键菜单 "Spawn sub-task" (per [F-32](./01-requirements.md))  (b) L0 chat bar 自然语言命令 (e.g. "把任务 a 派生一个 SA-04 sub-task") |
| **状态前置** | parent.state ∈ {`running`, `paused`} (per [F-28](./01-requirements.md)) |
| **7 步原子化** | (per [03-detailed §3.3.1.1](./03-detailed-design.md) Python 実装) |
| | 1. C-26 SpawnSubtaskValidator.check_parent_state(parent_id) |
| | 2. C-26.check_sa_type(child_sa_type) (e.g. SA-10 不能自嵌套) |
| | 3. C-25 SubTaskQuotaRegistry.query(parent_id) (≤ max_subtasks=2) |
| | 4. generate child_task_id (UUID v7, prefix=`st-`) |
| | 5. C-27 ChildSubAgentFactory.create(child_id, sa_type, parent_id) (写 4 字段) |
| | 6. SubAgentRegistry.spawn(child_id, sa_type) (C-24 ExclusiveBindingGuard 守门) |
| | 7. C-17 TaskRelationshipGraph.add_edge(parent_id, child_id, type=`spawn_exclusive`) + audit log append |
| **成功响应** | `{child_task_id, parent_task_id, sa_type, status: "spawned", spawned_at}` |
| **失败模式** | (a) `InvalidParentState` (b) `InvalidSAType` (c) `SubTaskQuotaExhausted` (d) `SelfSpawnProhibited` (e) `ExclusiveBindingViolation` |
| **审计** | 全部成功/失败都 append audit log (Transaction, append-only) |
| **守门 #13 a** | **不引入 L1↔L1 直连**, 全部 L0 协调, 唯一 cross-task actor (per [ADR-0046 §1.3 架构冲突守门 #13 a](../2026-08-26-upgrade/adr/0046-langgraph-task-management-operations.md)) |

**8 节点对称表** (per [ADR-0046 §2.1 TMO 7 节点](../2026-08-26-upgrade/adr/0046-langgraph-task-management-operations.md) + 本文档增量 M-N8):

| Node ID | 名称 | 责務 | 触发 chat bar 例子 | 状态 |
|---|---|---|---|---|
| M-N1 | `merge_node` | 合并 a + b → merged_task | "合并任务 a 和任务 b" | ✅ ADR-0046 |
| M-N2 | `split_node` | 拆分 a → a1 + a2 (a superseded) | "把任务 a 拆成 a1 和 a2" | ✅ ADR-0046 |
| M-N3 | `reorder_node` | 依赖 DAG 边更新 + cycle detection | "任务 b 完成后才能启动 c" | ✅ ADR-0046 |
| M-N4 | `bulk_node` | N 张卡批量 action | "暂停 a b c 三张卡" | ✅ ADR-0046 |
| M-N5 | `summarize_node` | 跨任务汇总 | "任务 a b c 进度" | ✅ ADR-0046 |
| M-N6 | `reassign_node` | sub-agent 类型 SA-XX 切换 | "把 a 改用 SA-04 重跑" | ✅ ADR-0046 |
| M-N7 | `metadata_node` | task_metadata 表更新 (Master RLS 必携) | "把 a 改名为 xxx" | ✅ ADR-0046 |
| **M-N8** | `spawn_subtask_node` | **1 派生 1 创建绑定子任务, 父继续 running, child 1:1 独占 SA-XX** | **"把任务 a 派生一个 SA-04 sub-task"** | **🆕 v0.1** |

### §2.7.2 互斥表 (per [F-26 关键约束](./01-requirements.md))

| 父 task 状态 | M-N1 merge | M-N2 split | **M-N8 spawn_subtask** | 备注 |
|---|---|---|---|---|
| `running` | ✅ 允许 | ✅ 允许 | **✅ 允许 (NEW)** | 正常态 |
| `paused` | ❌ 拒绝 | ❌ 拒绝 | **✅ 允许 (NEW)** | 暂停态可 spawn |
| `superseded` (由 M-N1/M-N2 产生) | ❌ 拒绝 | ❌ 拒绝 | **❌ 拒绝 (NEW)** | 终态不可 spawn |
| `completed` | ❌ 拒绝 | ❌ 拒绝 | **❌ 拒绝 (NEW)** | 终态不可 spawn |
| `failed` | ❌ 拒绝 | ❌ 拒绝 | **❌ 拒绝 (NEW)** | 终态不可 spawn |
| `cancelling` | ❌ 拒绝 | ❌ 拒绝 | **❌ 拒绝 (NEW)** | 终态不可 spawn |

**M-N8 跟 M-N1/M-N2 互斥**:
- 父 task 已被 M-N1 merge → 父状态 = `superseded` → M-N8 拒绝
- 父 task 已被 M-N2 split → 父状态 = `superseded` → M-N8 拒绝
- 父 task 已 spawn sub-task → 父状态 = `running` (不变) → 可继续 spawn (受 quota 限)

### §2.7.3 5 Reducer (per [LangGraph 02-basic §3.2 Reducer 范式](../2026-09-03-langgraph/02-basic-design.md))

| Reducer ID | 名称 | 类型 | 字段 | 守门 |
|---|---|---|---|---|
| **R-08** | `add_child_binding` | append | `parent.child_subtask_ids: list[str]` (append child_task_id) | 守门 #13 b Transaction append-only |
| **R-09** | `update_quota_usage` | replace | `task_quota_config.current_subtasks: int` (count) | 守门 #13 c/d Master SCD |
| **R-10** | `add_lifecycle_event` | append | `task_relationships.lifecycle_events: list[LifecycleEvent]` | 守门 #13 b Transaction append-only |
| **R-11** | `track_token_usage` | add | `sub_task_token_budget.used_tokens: int` (累加) | 守门 #4 token-OLU + NFR-SB-04 |
| **R-12** | `mark_exclusive_binding` | replace | `sub_task.exclusive_owner_task_id: str` (write-once, 不可改) | 守门 #7 0 unsafe + NFR-SB-02 不可绕过 |

**5 Reducer 跟 [ADR-0046 §2.4 TMO 5 Reducer 增量](../2026-08-26-upgrade/adr/0046-langgraph-task-management-operations.md) 关系**:
- 5 → 10: 增量 5 (R-08..R-12)
- 增量逻辑: R-08 父→子血缘, R-09 quota 计量, R-10 lifecycle event, R-11 token 预算, R-12 exclusive 守门

### §2.7.4 spawn_subtask 协议 (per [F-27](./01-requirements.md))

**L0 → L1 `SpawnSubtaskRequest`**:
```python
class SpawnSubtaskRequest(BaseModel):
    parent_task_id: str                              # 父 task ID
    child_sa_type: Literal["SA-01","SA-02","SA-03",
                           "SA-04","SA-05","SA-06",
                           "SA-07","SA-08","SA-09","SA-10"]  # child sub-agent 类型
    child_name: str                                  # 子任务名 (≤ 200 chars)
    initial_context: dict = {}                       # 父 context 浅拷贝 + exclusive 标注
    quota_check: bool = True                         # 默认走 quota 校验
    bound_by: str = "Ulysses"                        # 创建者 (per 守门 #10 代签)
    spawned_via: Literal["ui_rightclick", "l0_chat"] # 入口标注
```

**L1 → L0 `SpawnSubtaskResult`**:
```python
class SpawnSubtaskResult(BaseModel):
    child_task_id: str                               # 子 task ID (prefix=`st-`)
    parent_task_id: str                              # 父 task ID
    sa_type: str                                     # child SA 类型
    status: Literal["spawned", "failed"]
    failure_reason: Optional[str] = None             # 失败时填
    spawned_at: datetime                             # UTC ISO8601
    exclusive_owner_task_id: str                     # = parent_task_id (per F-28 1:1 独占)
    quota_remaining: int                             # 剩余 quota (per F-31)
```

**协议约束** (per 守门 #13 a + #13 c/d W/T/M):
- 全部 L0 → L1 单向 (除失败响应), 不引入 L1↔L1 直连
- 协议 payload 走 Transaction append-only 审计 (W/T/M 横展開 派生规 (b))
- `exclusive_owner_task_id` 字段强制 = `parent_task_id` (per [F-28](./01-requirements.md) 1:1 独占)

---

## §3 State Schema 扩展

### §3.2.1.2 SubAgentState 4 字段扩展 (per [F-28](./01-requirements.md))

在 [LangGraph 03-detailed §3.2.1.1 SubAgentState line 779](../2026-09-03-langgraph/03-detailed-design.md) 已有 5 血缘字段 (`parent_task_id` / `merged_from` / `split_into` / `superseded_by` / `checkpoint_snapshot`) 基础上, **增量 4 字段**:

```python
class SubAgentState(TypedDict, total=False):
    # 已有字段 (per [ADR-0046 §2.4 + LangGraph 03-detailed §3.2.1.1](../2026-09-03-langgraph/03-detailed-design.md))
    task_id: str
    parent_task_id: Optional[str]                    # 父 task ID (None = 顶层 task)
    merged_from: list[str]                           # M-N1 append-only
    split_into: list[str]                            # M-N2 append-only
    superseded_by: Optional[str]
    checkpoint_snapshot: Optional[dict]
    intermediate_steps: Annotated[list[Step], add]    # append
    # ...

    # 🆕 v0.1 增量 4 字段 (per [F-28](./01-requirements.md))
    exclusive_owner_task_id: Optional[str]           # exclusive 1:1 绑定的父 task ID
    bound_at: Optional[datetime]                     # 绑定创建时间 (UTC ISO8601)
    bound_by: Optional[str]                          # 绑定创建者 (Ulysses / Mavis / sub-agent ID)
    bound_via: Optional[Literal["ui_rightclick", "l0_chat", "api"]]  # 入口标注
```

**关键约束** (per [F-28 关键约束](./01-requirements.md) + NFR-SB-02 不可绕过):
- `exclusive_owner_task_id` 不可变 (write-once), 任何 reassign 必须先 check (per R-12 Reducer)
- `bound_at` / `bound_by` / `bound_via` 不可变 (immutable, per Transaction append-only 派生规 (b))
- `parent_task_id` 跟 `exclusive_owner_task_id` 可不同 (e.g., 子任务还有祖父, 但只跟直接父 exclusive)

**TopAgentState 增量字段** (per [ADR-0046 §2.4 已有 5 字段](../2026-08-26-upgrade/adr/0046-langgraph-task-management-operations.md) + 本文档增量 1 字段):

```python
class TopAgentState(TypedDict, total=False):
    # 已有字段
    task_relationships: TaskRelationshipGraph
    superseded_tasks: Annotated[list[str], add]
    bulk_operations: deque[BulkOperation]
    last_summarize_result: Optional[dict]
    active_tmo_operation: Optional[dict]
    # ...

    # 🆕 v0.1 增量 1 字段
    last_spawn_subtask_result: Optional[SpawnSubtaskResult]  # 最近一次 M-N8 结果 (per UI 渲染)
```

---

## §4 守门合规矩阵 (per AGENTS.md §4 #1-#24 + 累积规 v1-v24)

| 守门 | 引用 | 应用 | 验证 |
|---|---|---|---|
| **#1** | cargo check 0 err | 文档 v0.1 N/A, 实装 v0.3 必 `cargo check --workspace --all-targets -j 4` 0 err | 实装后 E2E |
| **#3** | 5 域独立 Lead, 不接受兼任 (per 8/21 拍板) | 文档 dual-use disclaimer + sub-task 跨域编排代签 | DDD Review |
| **#5** | 环境变量安全 (per 11:06 JST hard ban) | 无 secret 引用 | 文档审计 |
| **#6** | PowerShell only | N/A (文档) | — |
| **#7** | 0 unsafe (代码守门) | C-24 ExclusiveBindingGuard type-safe, 不依赖 unsafe code | 实装后 E2E |
| **#9** | 子代理 status=succeeded ≠ 实际成功, git log --follow 实证 | 调试控制台走 subprocess (per v24), 不派 worker 子代理 | E2E |
| **#12** | 缺标比错标安全 | 7 已知缺口 (G-SB-1..7) 显式列, 不掩盖 | DDD Review |
| **#13 a** | L1↔L1 禁止通信 | M-N8 全部 L0 协调, 唯一 cross-task actor | E2E-15 |
| **#13 c/d** | DB W/T/M 横展開 | 3 表 (sub_task_binding / sub_task_runtime / task_quota_config) 100% 覆盖 | [00-CLASSIFICATION-W-T-M.md](../../data-design/ipa-detail/00-CLASSIFICATION-W-T-M.md) |
| **#19** | agent 交互 Python 化 | 走 `scripts/automation/task_ops.py spawn_subtask` | 实装后审计 |
| **#21** | [P] 子项 docs 同步 | automation-design.md §4.14 追加 | commit message |
| **#22** | 调试控制台不污染 main 编译 | console_server.py 跑后 `cargo check --workspace --lib` 0 err | 实装后 E2E |
| **#23** | AI 修改 mock | UI "AI 建议 sub-task name" 走 ai_edit_mock.py, 不开 OpenAI | E2E |
| **#24** | 调试控制台走 subprocess 替代 RPC | 浏览器 → Next.js API route → console_server.py → subprocess.run | E2E |

**v3x 候选守门** (per [AGENTS.md §4.1.1 候选 v27-v31](../../AGENTS.md), status=待 Ulysses 拍板激活):
- v27 子代理 RPC 失败 fallback 必填: M-N8 spawn_subtask 走 subprocess 不派 worker, 适用
- v28 拍板必带推荐选项: 本文档落档前 ask_user 拍板 1 选项 (推荐), 适用
- v29 docs 同步饱和 40+ 次告警: 本次 docs 同步 + 1, 距饱和点 40 还远
- v30 Mavis 永久代签: 5 签字栏全部代签, 适用
- v31 5 域 Lead 真人到位追溯: 真人到位后修订历史 +1 行, 适用

---

## §5 外部 API 端点 (per [ADR-0046 §2.5 8 外部 API 端点](../2026-08-26-upgrade/adr/0046-langgraph-task-management-operations.md) 范式)

| 端点 | 方法 | 用途 | 入参 | 出参 | 守门 |
|---|---|---|---|---|---|
| `/api/tmo/merge` | POST | M-N1 merge | `MergeRequest` | `MergeResult` | ADR-0046 |
| `/api/tmo/split` | POST | M-N2 split | `SplitRequest` | `SplitResult` | ADR-0046 |
| `/api/tmo/dependencies` | POST | M-N3 reorder | `DepSetRequest` | `DepSetResult` | ADR-0046 |
| `/api/tmo/bulk` | POST | M-N4 bulk | `BulkActionRequest` | `BulkActionResult` | ADR-0046 |
| `/api/tmo/summarize` | POST | M-N5 summarize | `SummarizeRequest` | `SummarizeResult` | ADR-0046 |
| `/api/tmo/reassign` | POST | M-N6 reassign | `ReassignRequest` | `ReassignResult` | ADR-0046 |
| `/api/tmo/metadata` | POST | M-N7 metadata | `MetadataUpdateRequest` | `MetadataUpdateResult` | ADR-0046 |
| `/api/tmo/relationships` | GET | DAG 关系查询 | `parent_id` (query) | `RelationshipGraph` | ADR-0046 |
| **`/api/tmo/spawn_subtask`** | **POST** | **M-N8 spawn_subtask (NEW)** | **`SpawnSubtaskRequest` ([§2.7.4](#274-spawn_subtask-协议-per-f-27))** | **`SpawnSubtaskResult` ([§2.7.4](#274-spawn_subtask-协议-per-f-27))** | **本 v0.1** |

**8 → 9 端点 增量 1** (per [ADR-0046 §2.5](../2026-08-26-upgrade/adr/0046-langgraph-task-management-operations.md) 范式):
- 端点走 FastAPI 8080 console_server.py 扩展 (per 守门 #9 v3 / 守门 #24 v2 subprocess 走 console_server)
- 端点入参走守门 #5 env 安全 (无 secret 字段)
- 端点出参走守门 #12 缺标比错标 (失败必填 `failure_reason`)

---

## §6 守门合规 (NFR 验证矩阵 per [§3 NFR-SB-01..05](./01-requirements.md))

| NFR | 验证项 | 验证方法 | 守门 | 状态 |
|---|---|---|---|---|
| NFR-SB-01 | spawn_subtask p95 latency ≤ 200ms | E2E-14 (per [03 §4.1](./03-detailed-design.md)) | 守门 #1 + #4 | 实装后 |
| NFR-SB-02 | 1:1 binding 不可绕过 | E2E-15 (per [03 §4.2](./03-detailed-design.md)) | 守门 #7 + #12 | 实装后 |
| NFR-SB-03 | lifecycle 联动延迟 ≤ 1s | E2E-16 (per [03 §4.3](./03-detailed-design.md)) | 守门 #1 + #9 | 实装后 |
| NFR-SB-04 | sub-task quota 独立计量 | UT-32 (per [03 §4.4](./03-detailed-design.md)) | 守门 #1 + #13 | 实装后 |
| NFR-SB-05 | W/T/M 分类 100% 覆盖 | [00-CLASSIFICATION-W-T-M.md](../../data-design/ipa-detail/00-CLASSIFICATION-W-T-M.md) 100 表索引 + 本文档 3 表 | 守门 #13 c/d | 文档 v0.1 已落档 |

**无 cargo 守门需要** (本基本設計 N/A 代码实装, 守门 #1 v1-v2 / v5-v14 不适用).

---

## §7 风险 (per [01 §5 已知缺口](./01-requirements.md) + 增量)

| 风险 | 中和措施 | 触发 | 来源 |
|---|---|---|---|
| 7 已知缺口 (G-SB-1..7) | 跟 [01 §5 同步](./01-requirements.md) | 文档 v0.1 | [01 §5](./01-requirements.md) |
| 实装工作量 ~0.5-0.8M tokens (估) | 8 子项按 token 预算排序, 优先 C-24 ExclusiveBindingGuard (守门 #7 0 unsafe) | 实装 v0.3 启动 | 跟 ADR-0046 §4.2 #1 一致 |
| P0-1 / H2 阻塞依赖 | 实装 phase 跨 session 续做, brief 必先落档 (per 守门 #20) | 实装 v0.3 启动 | [ADR-0046 §4.2 #2](../2026-08-26-upgrade/adr/0046-langgraph-task-management-operations.md) |
| LangGraph SDK alpha | 实装前 `uv add langgraph@latest` + `pip show langgraph` 确认 | 实装前 | [ADR-0046 §4.2 #3](../2026-08-26-upgrade/adr/0046-langgraph-task-management-operations.md) |
| 5 域 Lead 真人未到位 | 守门 #3 v2 派生规: 真人到位后追溯签字, 不沿用代签决策 | DDD Review 阶段 | [01 §5 G-SB-6](./01-requirements.md) |
| SA-10 自嵌套风险 | C-26 SpawnSubtaskValidator 强制拒绝 SA-10 spawn 自身 | E2E-14 | [01 §2 F-30](./01-requirements.md) |

---

## §8 签字栏 (Signatures, per 7 段结构 5 角色)

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人 | Ulysses（一人公司 12 角色 per DEC-008）| 2026-09-09 | 🟡 Draft v0.1; 基本設計 8 节点 (M-N1..M-N8) + 8 协议 + 8 组件 (C-23..C-30) + 5 Reducer (R-08..R-12) + 1 外部 API 端点 (POST /api/tmo/spawn_subtask); 跟 [01 §1 UC-14..UC-18 + §2 F-26..F-32 + §3 NFR-SB-01..05](./01-requirements.md) + [03 §3.3 M-26..M-33](./03-detailed-design.md) 同步 |
| 1.1 | 架构师 / Mavis 接手审批 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-09 | 🟢 Mavis 接手终审通过 (per ask_1df6987367ccc00928b65ee3 拍板 1 选项 "3 份新文档 + ADR-0052 升 v0.3 (推荐)"); 7 守门合规矩阵 (守门 #1/#3/#5/#6/#7/#9/#12/#13a/#13cd/#19/#21/#22/#23/#24) + v3x 候选 5 项 (v27-v31) + 6 风险 + 5 签字栏落档 |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-09 | 🟢 Mavis 接手代签 (per 19:39 + 21:59 JST); 5 域独立真实身份 (per 8/21 JST) 签字请 DDD Review 阶段补 |
| 3 | 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-09 | 🟢 Mavis 接手代签 (per 19:39 + 21:59 JST); 5 域独立真实身份签字请 DDD Review 阶段补 |
| 4 | 评审主持人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-09 | 🟢 Mavis 接手代签 (per 19:39 + 21:59 JST); 5 域独立真实身份签字请 DDD Review 阶段补 |
| 5 | 项目负责人 (PM) | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-09 | 🟢 Mavis 接手代签 (per 19:39 + 21:59 JST); 5 域独立真实身份签字请 DDD Review 阶段补 |

---

## §9 修订历史 (Revision History)

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-09 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: 8 组件 (C-23..C-30) + 1 节点 (M-N8 spawn_subtask_node) + 1 协议 (SpawnSubtaskRequest/Result) + 5 Reducer (R-08..R-12) + 1 外部 API 端点 (POST /api/tmo/spawn_subtask) + SubAgentState 4 字段扩展 (exclusive_owner_task_id / bound_at / bound_by / bound_via) + TopAgentState 1 字段扩展 (last_spawn_subtask_result) + 14 守门合规矩阵 + 5 NFR 验证 + 6 风险 + 5 签字栏; 跟 [01-requirements.md v0.1](./01-requirements.md) + [03-detailed-design.md v0.1](./03-detailed-design.md) + [ADR-0052](../2026-08-26-upgrade/adr/0052-subtask-binding.md) + [PHASE-SUBTASK-BINDING-IMPL-REPORT.md](../../reports/PHASE-SUBTASK-BINDING-IMPL-REPORT.md) 同步落档 | 2026-09-09 21:53 JST 用户发令 + ask_1df6987367ccc00928b65ee3 拍板 1 选项 "3 份新文档 + ADR-0052 升 v0.3 (推荐)", 跟 3 份新文档 + ADR-0052 + PHASE 报告 v0.1 同步落档, ~0.08M token 实测 |

---

## §10 参考 (References)

- [01-requirements.md v0.1](./01-requirements.md) — UC-14..UC-18 + F-26..F-32 + NFR-SB-01..05 (前序, 本文增量架构)
- [03-detailed-design.md v0.1](./03-detailed-design.md) — spawn_subtask/ 模块 + M-26..M-33 + §3.3.1.1 7 节点 Python 実装 + UT-27..UT-33 / IT-13..IT-15 / E2E-14..E2E-18
- [ADR-0052 决策记录](../2026-08-26-upgrade/adr/0052-subtask-binding.md) — M-N8 spawn_subtask_node 决策落档
- [ADR-0046 LangGraph TMO 7 节点](../2026-08-26-upgrade/adr/0046-langgraph-task-management-operations.md) — 7 节点 + 7 协议 + 7 组件 + 25 module (前序, 本文增量 1+1+8+8)
- [LangGraph 02-basic-design v0.2 §2.5 T-N1..T-N7 + §2.6 M-N1..M-N7 + §3.2 Reducer](../2026-09-03-langgraph/02-basic-design.md) — 主路径 L0 StateGraph 范式
- [LangGraph 03-detailed-design v0.2 §3.2.1.1 SubAgentState line 779](../2026-09-03-langgraph/03-detailed-design.md) — SubAgentState TypedDict 已有 5 血缘字段
- [AGENTS.md §3 报告 7 段结构 + §4 守门 #1-#24 + §4.1 累积规 v1-v24 + §4.1.1 v3x 候选 + §6.1-6.2 架构核心](../../AGENTS.md)
- [docs/automation-design.md](../../automation-design.md) — agent 交互 Python 化 (守门 #19)
- [docs/data-design/ipa-detail/00-CLASSIFICATION-W-T-M.md v0.1](../../data-design/ipa-detail/00-CLASSIFICATION-W-T-M.md) — DB 三類橫展開 100 表索引
- [docs/data-design/ipa-detail/00-CLASSIFICATION-RULES.md v0.1](../../data-design/ipa-detail/00-CLASSIFICATION-RULES.md) — 跨项目 ルール手册
- [PHASE-SUBTASK-BINDING-IMPL-REPORT.md v0.1](../../reports/PHASE-SUBTASK-BINDING-IMPL-REPORT.md) — 8 子项实装 phase 计划
- [STAR-OLU-001.md v0.1](../../STAR-OLU-001.md) — 1 SRE·周 = 1.2M tokens (M-N8 + 8 组件估 ~0.5-0.8M tokens)
- [STAR-P3-WBS-001.md v0.6 §7 阻塞 7 项](../../STAR-P3-WBS-001.md) — P3-B 启动前置
- [HANDOFF-ST-001.md v0.4 §5.3 Blocker](../../reports/HANDOFF-ST-001.md) — 5 项 Blocker 跨 session 续

---

# === Basic Design 结束 ===

**per AGENTS.md §0 一句话硬约束 + §1 代签规则**: 可以代签 Ulysses, 不可以编造历史. 本文档 v0.1 引用守门 #1-#24 + 累积规 v1-v24 全部按 git 实证 + AGENTS.md 引用, 无"per X 历史形态"等回溯叙事.

**per 守门 #3 5 域单仓**: 本文档仅 STAR 仓内, 不引用 RGS 仓代码, 不建立业务子域↔DDD bounded context 映射 (per ADR-0044 §dual-use disclaimer).

**per 守门 #13 W/T/M 横展開**: 8 组件全部按 W/T/M 分類 (C-23/24/25/26/28/29 走 Master, C-27 走 Transaction, C-30 走 UI 不入 DB), 派生规 (a)(b)(c)(d) 全部落档.

**per 守门 #21 v21 [P] docs 同步**: automation-design.md §4.14 追加, commit message 引用相对路径.
