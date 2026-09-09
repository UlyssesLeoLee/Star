# STAR Sub-task Binding 要件定義書 (Requirements Specification)

> **バージョン**: v0.1
> **ステータス**: 🟡 Draft → 🔵 Review (待 DDD Review 拍板)
> **日付**: 2026-09-09
> **制定者**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **签批**: 🟢 Mavis 接手终审（per 2026-08-27 19:39 + 21:59 JST 用户授权"允许你代签"）
> **父文档**: [STAR × GitGit AI/IDE 零厂商适配架构升级 Plan](../2026-08-26-upgrade-plan.md) (待归档) · [Star LangGraph 統合アーキテクチャ 要件定義書 v0.2](../2026-09-03-langgraph/01-requirements.md) · [ADR-0046 TMO 7 节点](../2026-08-26-upgrade/adr/0046-langgraph-task-management-operations.md)
> **兄弟文档**: [02-basic-design.md v0.1](./02-basic-design.md) · [03-detailed-design.md v0.1](./03-detailed-design.md) · [ADR-0052 决策记录](../2026-08-26-upgrade/adr/0052-subtask-binding.md) · [PHASE-SUBTASK-BINDING-IMPL-REPORT.md](../../reports/PHASE-SUBTASK-BINDING-IMPL-REPORT.md)
> **依存**: [ADR-0030 Agent Lease/Heartbeat/Resume](../2026-08-26-upgrade/adr/0030-agent-lease-heartbeat-resume.md) · [ADR-0032 MCP Transport stdio](../2026-08-26-upgrade/adr/0032-mcp-transport-stdio.md) · [ADR-0033 代签规则反转](../2026-08-26-upgrade/adr/0033-agent-co-signing-policy.md) · [ADR-0046 TMO 7 节点](../2026-08-26-upgrade/adr/0046-langgraph-task-management-operations.md) · [AGENTS.md §4 守门硬约束](../../AGENTS.md) · [AGENTS.md §4 #13 W/T/M 横展開](../../AGENTS.md)
> **適用**: STAR 仓内 (per 守门 #3 5 域单仓), 跨 session 续做; dual-use disclaimer per ADR-0044 §dual-use: 5 域 (player/economy/match/social/admin) ≠ 9 SA Type (SA-01..SA-09) + SA-10, 不建立业务子域↔DDD bounded context 映射

---

## §0 文档目的

新增 **STAR Sub-task Binding 路径**：通过在**父任务卡交互下命令** (UI 右键 / L0 chat bar) 创建**绑定子任务** (bound sub-task)，子任务**1:1 独占**专属子代理 (exclusive sub-agent, L1 SA-XX instance), 形成 **父任务卡 (running) → 绑定子任务 (1:1) → 专属 sub-agent (1:1)** 的硬绑定生命周期链.

**与现有 TMO M-N2 split (per [ADR-0046 §2.1](../2026-08-26-upgrade/adr/0046-langgraph-task-management-operations.md)) 的关键差异**：

| 维度 | M-N2 split (现状) | **M-N8 spawn_subtask (新)** |
|---|---|---|
| 派生数量 | 1 → N (拆解血缘) | **1 → 1** (单 child) |
| 父任务终态 | **superseded** (终止) | **继续 running** (并存) |
| child sub-agent 类型 | 继承父 task_type | **独立指定 SA-01..SA-09 + SA-10** |
| 派生语义 | split_into 血缘 | **exclusive 1:1 绑定** |
| 资源隔离 | 共享 N 並行 ≤ 50 配额 | **独立 sub-task quota** |
| reassign 兼容 | 允许 M-N6 reassign | **禁止 M-N6** (exclusive 不可换) |
| UI 入口 | 仅 L0 chat bar (per [ADR-0046 §1.1](../2026-08-26-upgrade/adr/0046-langgraph-task-management-operations.md)) | **UI 任务卡右键 + L0 chat bar 双入口** |
| lifecycle 联动 | 各自独立 (per [03-detailed §3.2.1.1 M-N2](../2026-09-03-langgraph/03-detailed-design.md)) | **父 cancel/pause → child cancel/pause** (链式) |

**不涉及** (per 2026-09-09 21:53 JST 用户发令 + ADR-0044 §0 STAR 不涉及)：
- 物理引擎 (Physis 独立产品线)
- 3D 渲染 / HUD
- 跨机分布式 (per 守门 #3 5 域单仓, 跨机待 P3-F 评估, 本阶段 ❌ N/A)

---

## §1 Use Cases (UC-14..UC-18)

### UC-14: Spawn sub-task from parent task card (UI right-click)

**Actor**: Ulysses (一人公司 12 角色 per DEC-008, 当前 Mavis 临时代签 per 守门 #3 反转 B 11:35 JST)
**Trigger**: 父任务卡 (running / paused 状态) UI 右键菜单点击 "Spawn sub-task" 选项
**Pre-condition**:
- 父任务状态 ∈ {`running`, `paused`} (不允许从 `superseded` / `completed` / `failed` 派生)
- 父任务 sub-task quota 未耗尽 (per [F-31](#f-31-subtaskquotaregistry-c-25-per-sub-task-token-预算) + NFR-SB-04)
- L0 chat 窗口存在 (UI 容器)

**Main flow**:
1. UI 任务卡右键菜单显示 "Spawn sub-task" (per [F-32](#f-32-ui-taskcard-right-click-spawn-sub-task-入口))
2. 点击 → 弹 modal "Select sub-agent type" (SA-01..SA-09 + SA-10 下拉) + "Sub-task name" (默认 = 父任务名 + " - sub-N")
3. 用户选 SA-XX + 输入 sub-task name → 点 "Confirm"
4. UI → Next.js API route `POST /api/tmo/spawn_subtask` (per 守门 #9 v3 console_server.py 走 subprocess)
5. console_server.py → `TaskOperationsManager.spawn_subtask(parent_task_id, child_sa_type, child_name)`
6. L0 StateGraph 进入 `M-N8 spawn_subtask_node` (per [02-basic §2.7.1](./02-basic-design.md))
7. 7 步原子化执行 (per [03-detailed §3.3.1.1](./03-detailed-design.md)):
   7.1. 校验父任务状态 (F-28 binding guard)
   7.2. 校验 SA-XX 类型合法 (F-30)
   7.3. 校验 quota 未耗尽 (F-31)
   7.4. 生成 child task_id (UUID v7, sub-task namespace prefix `st-`)
   7.5. 创建 child SubAgentState (initial), 写 `parent_task_id` / `exclusive_owner_task_id` / `bound_at` / `bound_by` 4 字段
   7.6. SubAgentRegistry.spawn(child_task_id, sa_type) (in-process subgraph)
   7.7. TaskRelationshipGraph.add_edge(parent → child, edge_type=`spawn_exclusive`) (DAG, 4 字段之一, per [ADR-0046 §2.3 C-17](../2026-08-26-upgrade/adr/0046-langgraph-task-management-operations.md))
8. UI 任务卡列表刷新: 父任务卡下方出现 child 卡 (1:1 mirror), 状态 = `running`
9. console_server.py 返回 `{child_task_id, status: "spawned", sa_type, parent_task_id}` → UI toast "Sub-task spawned"

**Post-condition**:
- child task 状态 = `running`, 1:1 独占 1 个 SA-XX instance
- 父 task 状态 = 继续 `running` (不变, **不** superseded)
- TaskRelationshipGraph 新增 1 条 edge (parent → child, type=`spawn_exclusive`)
- Audit log append (Transaction) — sub-task binding event

**Alt flow A1**: 用户取消 modal → UI 关闭, 不创建 sub-task
**Alt flow A2**: SA-XX 选错 (e.g., SA-10 task-orchestrator 自嵌套) → 校验失败, modal 提示 "SA-10 不能 spawn 自身 sub-task" (per [F-30](#f-30-exclusivebindingguard-c-24-11-独占守门) 守门规)
**Alt flow A3**: quota 耗尽 → modal 提示 "Parent sub-task quota exhausted (max=2)", 用户可申请 quota 提升 (per [F-31](#f-31-subtaskquotaregistry-c-25-per-sub-task-token-预算) + NFR-SB-04)

### UC-15: Spawn sub-task via L0 chat bar command

**Actor**: Ulysses (Mavis 临时代签)
**Trigger**: L0 底端聊天栏输入自然语言命令, e.g. "把任务 a 派生一个 SA-04 sub-task, 名字叫 git-init-st"
**Pre-condition**: 同 UC-14 + 父任务 ID 可在 L0 chat 上下文引用 (current task)

**Main flow**:
1. L0 chat bar 接收 user input → `parse_intent_node` (per [LangGraph 02-basic §2.5](../2026-09-03-langgraph/02-basic-design.md) T-N1)
2. 意图识别: `intent='spawn_subtask'`, 提取 `{parent_task_id, child_sa_type, child_name}`
3. L0 StateGraph 路由到 `M-N8 spawn_subtask_node` (跟 UC-14 共享 7 步原子化, per [F-26](#f-26-spawn_subtask_node-m-n8-l0-stategraph-节点))
4. 同 UC-14 step 7-9
5. L0 chat bar 回复 "✅ Sub-task spawned: `st-0192f8c4-...` (parent=`a`, SA-04 git-ops)"

**Alt flow B1**: 意图模糊 (e.g. "派生一个 sub-task" 但没指定 SA-XX) → L0 chat 追问 "Select SA type: SA-01..SA-09 + SA-10"
**Alt flow B2**: parent_task_id 不存在 → L0 chat 报错 "Task `xxx` not found"

### UC-16: Lifecycle linkage (parent cancel/pause → child)

**Actor**: System (LifecycleLinkageManager 自动触发, per [F-29](#f-29-lifecyclelinkagemanager-c-23-parentchild-联动))
**Trigger**: 父任务状态转换到 `cancelling` / `paused`
**Pre-condition**: child task 存在 + edge type = `spawn_exclusive`

**Main flow**:
1. parent 触发 cancel → L0 → `cancel_parent_node` 改 parent 状态 = `cancelling`
2. LifecycleLinkageManager 订阅 parent state change event (EventBus per [AGENTS.md §6.2](../../AGENTS.md))
3. 查找 TaskRelationshipGraph 中所有 children (type=`spawn_exclusive`)
4. 链式触发 child cancel (L0 → L1 sub_pool.cancel(child_id, reason="parent_cancelled"))
5. child 状态 = `cancelling` → `cancelled`, parent 状态 = `cancelled`
6. Audit log append (Transaction) — 链式 lifecycle event

**Latency**: parent state change → child state change ≤ 1s (per NFR-SB-03)

**Alt flow C1**: parent pause → child pause (不 cancel, 可 resume) — LifecycleLinkageManager 走 `pause_child` 路径
**Alt flow C2**: child 主动 cancel → parent **不变** (child 独立 cancel 是允许的, 不会向上传播)
**Alt flow C3**: child 失败 → parent 进入 `degraded` 状态 (警告但不强制 cancel, per [F-29](#f-29-lifecyclelinkagemanager-c-23-parentchild-联动) 派生规)

### UC-17: Sub-task exclusive 1:1 binding enforcement

**Actor**: System (ExclusiveBindingGuard 编译期 + 运行期双层守门, per [F-30](#f-30-exclusivebindingguard-c-24-11-独占守门))
**Trigger**: 任何尝试对 exclusive sub-task 的 M-N6 reassign 操作 / 二次 spawn sub-task / 外部直接调用 SubAgentRegistry.spawn(child_id)

**Pre-condition**: child task exclusive = true (per `exclusive_owner_task_id` 字段)

**Main flow**:
1. 外部尝试 `ReassignManager.reassign(child_id, new_sa_type)` (per [ADR-0046 §2.3 C-21](../2026-08-26-upgrade/adr/0046-langgraph-task-management-operations.md))
2. ExclusiveBindingGuard.check(child_id) → 抛 `ExclusiveBindingViolation` exception
3. 错误信息: "Sub-task `st-xxx` is exclusive-bound to parent `yyy`, reassign prohibited (per NFR-SB-02)"
4. 守门失败, audit log append (Transaction), UI 提示用户

**Pre-condition check 阶段**:
- 编译期: type system 强制 `SubAgentState.exclusive_owner_task_id: str | None`, 任何 reassign 必须先 check 字段
- 运行期: ExclusiveBindingGuard.check() 在所有 L1 sub_pool operation 入口强制调用 (per [F-30](#f-30-exclusivebindingguard-c-24-11-独占守门) 守门规)

**Latency**: ≤ 50ms (per NFR-SB-02 不可绕过守门)

**Alt flow D1**: 试图 spawn sub-task 到非 `running` / `paused` 父任务 → 抛 `InvalidParentState`
**Alt flow D2**: 试图 spawn sub-task 到自己 (parent_task_id == child_task_id) → 抛 `SelfSpawnProhibited`

### UC-18: Sub-task quota isolation (independent token budget)

**Actor**: System (SubTaskQuotaRegistry + TokenTelemetry, per [F-31](#f-31-subtaskquotaregistry-c-25-per-sub-task-token-预算) + NFR-SB-04)
**Trigger**: child sub-agent 每次 LLM call / MCP tool call / checkpoint 写

**Pre-condition**: child task 存在 + SubTaskQuotaRegistry 配额配置 (默认 2 sub-task / 父任务, per NFR-SB-04 默认值)

**Main flow**:
1. child SA-XX 触发 LLM call → TokenTelemetry.record(child_task_id, tokens, op)
2. SubTaskQuotaRegistry.query(parent_task_id) → 父任务当前 active sub-task count
3. 若 count < quota, 允许; 若 >= quota, 抛 `SubTaskQuotaExhausted` (UC-14 Alt A3 触发点)
4. 每次 sub-task lifecycle event (spawn / cancel / complete) → SubTaskQuotaRegistry.update()
5. Audit log append (Transaction) — quota usage event

**默认 quota**: 2 sub-task / 父任务 (per NFR-SB-04 默认值, Master SCD 可配, 改需 DDD Review 拍板)

**Latency**: ≤ 10ms (in-memory counter, per NFR-SB-04)

**Alt flow E1**: 父任务 cancel → SubTaskQuotaRegistry 批量释放 (所有 children quota 同时释放)
**Alt flow E2**: quota 提升请求 → UI 弹 "Request quota increase" 按钮 → L0 chat 提示 (需 DDD Review 拍板后手工调整)

---

## §2 Functional Requirements (F-26..F-32)

### F-26: spawn_subtask_node (M-N8) L0 StateGraph 节点

新增 L0 StateGraph 第 8 个 TMO 节点 `M-N8 spawn_subtask_node`, 跟 [ADR-0046 §2.1 M-N1..M-N7](../2026-08-26-upgrade/adr/0046-langgraph-task-management-operations.md) 对称 (8 节点, 8 协议, 8 组件):

| Node ID | 名称 | 責務 | 触发 chat bar 例子 |
|---|---|---|---|
| **M-N8** | `spawn_subtask_node` | 1 派生 1 创建绑定子任务, 父继续 running, child 1:1 独占 SA-XX | "把任务 a 派生一个 SA-04 sub-task" |

**关键约束** (per 守门 #13 a L1↔L1 禁止通信):
- M-N8 **只走 L0 协调**, 不引入 L1↔L1 直连 (per [ADR-0046 §3.1 备选 A 拒绝理由](../2026-08-26-upgrade/adr/0046-langgraph-task-management-operations.md))
- M-N8 跟 M-N1 merge_node + M-N2 split_node 互斥 (per [02-basic §2.7.2](./02-basic-design.md) 互斥表):
  - 父 task 状态 `superseded` → 不能 spawn (per [F-28](#f-28-subagenthandlebound_to_parent-11-绑定字段) 守门)
  - 父 task 已 merge → spawn 操作被 M-N1 终态拦截
  - 父 task 已 split → spawn 操作被 M-N2 终态拦截

### F-27: spawn_subtask protocol (L0 → L1 通信)

新增第 8 通信协议 `spawn_subtask_request` (L0 → L1 单向) + `spawn_subtask_result` (L1 → L0 响应):

```python
# L0 → L1 (per [LangGraph 02-basic §2.3](../2026-09-03-langgraph/02-basic-design.md))
class SpawnSubtaskRequest(BaseModel):
    parent_task_id: str
    child_sa_type: Literal["SA-01", "SA-02", ..., "SA-09", "SA-10"]
    child_name: str
    initial_context: dict  # 父 task context 浅拷贝 + 标注 "spawned_exclusive"
    quota_check: bool = True  # 默认走 quota 校验

# L1 → L0
class SpawnSubtaskResult(BaseModel):
    child_task_id: str
    parent_task_id: str
    sa_type: str
    status: Literal["spawned", "failed"]
    failure_reason: Optional[str] = None
    spawned_at: datetime
```

**字段约束** (per [F-28](#f-28-subagenthandlebound_to_parent-11-绑定字段)):
- `child_task_id` 必须 = `st-` prefix (UUID v7) (per [03-detailed §3.3.1.1 step 7.4](./03-detailed-design.md))
- `parent_task_id` 必须 ∈ 现有 task 集合
- `child_sa_type` ∈ 9 SA + SA-10 (per [LangGraph 03-detailed §3.5.1-§3.5.10](../2026-09-03-langgraph/03-detailed-design.md))
- `child_name` 非空, 长度 ≤ 200 chars

### F-28: SubAgentHandle.bound_to_parent 1:1 绑定字段

扩展 SubAgentState (per [LangGraph 03-detailed §3.2.1.1 line 779](../2026-09-03-langgraph/03-detailed-design.md)) 增加 4 字段:

| 字段 | 类型 | 含义 |
|---|---|---|
| `parent_task_id` | `Optional[str]` | 父 task ID (None = 顶层 task, 非 spawn 派生) |
| `exclusive_owner_task_id` | `Optional[str]` | **exclusive 1:1 绑定** 的父 task ID (None = 不独占, 允许 reassign) |
| `bound_at` | `Optional[datetime]` | 绑定创建时间 (UTC, ISO8601) |
| `bound_by` | `Optional[str]` | 绑定创建者 (Ulysses / Mavis / sub-agent ID) |

**关键约束**:
- `parent_task_id` 跟 `exclusive_owner_task_id` 可不同 (e.g., 子任务还有祖父, 但只跟直接父 exclusive)
- exclusive sub-task 的 `exclusive_owner_task_id` **不可变** (per NFR-SB-02 不可绕过)
- exclusive sub-task **不可** reassign (M-N6 拒绝, per [F-30](#f-30-exclusivebindingguard-c-24-11-独占守门))
- exclusive sub-task **不可** merge (M-N1 拒绝, 互斥, per [F-26](#f-26-spawn_subtask_node-m-n8-l0-stategraph-节点) 互斥表)
- exclusive sub-task **不可** split (M-N2 拒绝, 互斥)

**保留字段** (per [ADR-0046 §2.4](../2026-08-26-upgrade/adr/0046-langgraph-task-management-operations.md) 已有):
- `merged_from` (append-only)
- `split_into` (append-only)
- `superseded_by`
- `checkpoint_snapshot`

### F-29: LifecycleLinkageManager (C-23) parent/child 联动

新增第 23 组件 `LifecycleLinkageManager`, 监听父 task state change event, 链式触发 child action:

| Parent 事件 | Child 动作 | 延迟 |
|---|---|---|
| parent cancel → `cancelling` | child cancel → `cancelling` → `cancelled` | ≤ 1s (NFR-SB-03) |
| parent pause → `paused` | child pause → `paused` | ≤ 1s |
| parent resume → `running` | child resume → `running` (if child was paused) | ≤ 1s |
| parent fail → `failed` | child cancel (链式, 不可逆) | ≤ 1s |
| child cancel (独立) | **不变 parent** (child 独立 cancel 允许, 不会向上传播) | — |
| child fail → `failed` | parent 进入 `degraded` (警告, 不强制 cancel) | ≤ 5s |

**实现** (per [03-detailed §3.3.2](./03-detailed-design.md)):
- EventBus subscription pattern (per [AGENTS.md §6.2](../../AGENTS.md) 已有 EventBus 设计)
- TaskRelationshipGraph.traverse(parent_id) 找所有 children
- 串行 or 并行触发 child action (默认串行, per [03-detailed §3.3.2 实现细节](./03-detailed-design.md))

### F-30: ExclusiveBindingGuard (C-24) 1:1 独占守门

新增第 24 组件 `ExclusiveBindingGuard`, 编译期 + 运行期双层守门:

**编译期守门** (per Python type system):
```python
class SubAgentState(TypedDict, total=False):
    exclusive_owner_task_id: Optional[str]  # 不可变字段, 任何 reassign 必须先 check

# 编译期 type hint 强制 read-only
@property
def exclusive_owner_task_id(self) -> Optional[str]:
    return self._exclusive_owner_task_id  # 不可写

# reassign 函数签名强制 check
def reassign(task_id: str, new_sa_type: str) -> bool:
    if ExclusiveBindingGuard.is_exclusive(task_id):
        raise ExclusiveBindingViolation(...)
    # ...
```

**运行期守门** (per [03-detailed §3.3.3](./03-detailed-design.md)):
- ExclusiveBindingGuard.is_exclusive(task_id) 在 L1 sub_pool **所有** operation 入口强制调用
- sub_pool 入口: `spawn` / `cancel` / `pause` / `resume` / `reassign` 5 个全部强制 check
- 守门失败抛 `ExclusiveBindingViolation`, audit log append, UI 提示

**派生约束**:
- SA-10 task-orchestrator **不能** spawn 自身 sub-task (per UC-14 Alt A2 + NFR-SB-02)
- 父任务 + child sub-task 必须是不同 SA-XX 类型 (不允许 self-spawn 同类型, 避免无限递归)
- 父任务 exclusive = true 时, 不能再 spawn sub-task (防止 exclusive 链式污染)

### F-31: SubTaskQuotaRegistry (C-25) per-sub-task token 预算

新增第 25 组件 `SubTaskQuotaRegistry`, 独立 quota 计量 + 配置 (Master SCD Type 2):

**默认 quota** (per NFR-SB-04):
- 每个父任务最多 2 个 active sub-task
- 每个 sub-task 独立 token 预算 (per UC-18 默认 100K tokens / sub-task)
- quota 配置存 `task_quota_config` 表 (Master SCD Type 2, per 守门 #13 c/d W/T/M)

**schema** (per [03-detailed §3.3.4](./03-detailed-design.md)):
```sql
CREATE TABLE task_quota_config (
    config_id UUID PRIMARY KEY,        -- Master PK
    parent_task_id UUID NOT NULL,
    max_subtasks INT NOT NULL DEFAULT 2,
    per_subtask_token_budget BIGINT NOT NULL DEFAULT 100000,
    effective_from TIMESTAMPTZ NOT NULL,
    effective_to TIMESTAMPTZ,          -- SCD Type 2 end
    is_current BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    created_by TEXT NOT NULL,
    -- W/T/M: Master, 物理删除禁止, SCD Type 2, RLS 13 類必携 (per 守门 #13 c/d)
    RLS_TENANT_ISOLATION CHECK (...)   -- per 守门 #13
);
```

**派生约束** (per 守门 #13 c/d):
- Master 表: 物理删除禁止, SCD Type 2, RLS 13 类必携
- 子任务状态 = Work (短 TTL, 完成后清理)
- 子任务血缘 = Transaction (append-only audit)

**TokenTelemetry 集成** (per NFR-SB-04):
- 每次 LLM call / MCP tool call → TokenTelemetry.record(child_task_id, tokens, op)
- 达到预算 → 抛 `SubTaskTokenBudgetExhausted`, child 状态 = `failed`, parent 状态 = `degraded`

### F-32: UI TaskCard right-click "Spawn sub-task" 入口

新增 UI 任务卡右键菜单 (per 守门 #9 v3 + 守门 #24 v2 调试控制台走 subprocess 范式):

**前端** (Next.js 15 App Router, per [frontend/src/app/tasks/page.tsx](../../frontend/src/app/tasks/page.tsx) 已有):
- TaskCard 组件加 `onContextMenu` handler
- 右键菜单 5 项: "Pause" / "Resume" / "Cancel" / "Spawn sub-task" (NEW) / "View dependencies"
- "Spawn sub-task" → 弹 modal (UC-14 step 2)

**后端 API** (FastAPI 8080 console_server.py, per [ADR-0046 §2.5 8 外部 API 端点](../2026-08-26-upgrade/adr/0046-langgraph-task-management-operations.md) 范式):
- `POST /api/tmo/spawn_subtask` 新增第 9 端点
- 走 subprocess.run 调 `scripts/automation/task_ops.py spawn_subtask` (per 守门 #9 v3 + 守门 #19 Python 化)
- 返 `{child_task_id, status, sa_type, parent_task_id}` → UI toast

**LLM 增强** (可选, per NFR-P-01 first token ≤ 200ms):
- modal "Sub-task name" 输入框 + AI 建议按钮 (per 守门 #23 v23 AI mock, 走 ai_edit_mock.py 不开 OpenAI)
- AI 建议: "根据父任务 `a` (SA-04 git-ops) + 当前 context, 推荐 sub-task name = `git-init-st-2026`"

---

## §3 Non-Functional Requirements (NFR-SB-01..05)

| NFR | 内容 | 验证 | 守门引用 |
|---|---|---|---|
| **NFR-SB-01** | spawn_subtask p95 latency ≤ 200ms (跟 NFR-P-01 first token 一致) | E2E-14: spawn 100 次, p95 < 200ms | 守门 #1 + 守门 #4 token-OLU |
| **NFR-SB-02** | 1:1 binding 不可绕过 (ExclusiveBindingGuard 编译期 + 运行期双层守门) | E2E-15: 尝试 reassign exclusive sub-task 必失败, exception 信息准确 | 守门 #7 0 unsafe + 守门 #12 缺标比错标 |
| **NFR-SB-03** | lifecycle 联动延迟 ≤ 1s (parent cancel → child cancel) | E2E-16: parent cancel 后 1s 内 child 状态 = cancelled | 守门 #1 + 守门 #9 子代理 status 实证 |
| **NFR-SB-04** | sub-task quota 独立计量 (TokenTelemetry per-sub-task) | UT-32: TokenTelemetry.record 100 次, 累计 = 100K 时抛 `SubTaskTokenBudgetExhausted` | 守门 #1 + 守门 #13 W/T/M |
| **NFR-SB-05** | W/T/M 分类: sub-task card = Work (短 TTL), 血缘 log = Transaction (append-only), quota config = Master (SCD Type 2) | per `docs/data-design/ipa-detail/00-CLASSIFICATION-W-T-M.md` v0.1 100 表 W/T/M 三類索引实绩对齐 | 守门 #13 a/b/c/d 全段 |

**派生 NFR** (per [AGENTS.md §4 守门 #1-#24 + 累积规 v1-v24](../../AGENTS.md)):
- 守门 #4 token-OLU: spawn_subtask token 预算单独计量, 不跟父任务合并
- 守门 #9 v3: 调试控制台走 subprocess 替代 RPC (per [AGENTS.md §4.1 v24](../../AGENTS.md))
- 守门 #13 a L1↔L1 禁止通信: M-N8 全部 L0 协调
- 守门 #13 c/d W/T/M: 3 表横展開
- 守门 #19 v19 agent 交互 Python 化: 走 `scripts/automation/task_ops.py spawn_subtask` (per [docs/automation-design.md](../../automation-design.md))
- 守门 #21 v21 [P] docs 同步: 落档时更新 `docs/automation-design.md §4.14` (per [AGENTS.md §4.1 v21](../../AGENTS.md))

---

## §4 W/T/M 数据分类 (per 守门 #13 c/d 強制約)

新增 3 表, 全部 W/T/M 三類橫展開, 100% 表覆盖:

| # | 表名 | 分類 | 物理刪除 | 監査 | RLS | SCD | 引用基线 |
|---|---|---|---|---|---|---|---|
| 1 | `sub_task_binding` (子任务绑定关系) | **Transaction** (业务事实, append-only) | ❌ 禁止 | ✅ 必携 | ✅ 13 类 | N/A (append) | [00-CLASSIFICATION-W-T-M.md v0.1](../../data-design/ipa-detail/00-CLASSIFICATION-W-T-M.md) |
| 2 | `sub_task_runtime` (子任务运行时状态) | **Work** (短 TTL, 完成后清理) | ✅ 允许 (TTL 24h) | N/A | N/A | N/A | 同上 |
| 3 | `task_quota_config` (父任务 sub-task 配额配置) | **Master** (参考, 慢变, SCD Type 2) | ❌ 禁止 | ✅ 必携 | ✅ 13 类 | ✅ Type 2 | 同上 |

**派生规** (per 守门 #13):
- (a) Work = 物理删除 / タイマー失効 / 短 TTL 明示 retention (24h)
- (b) Transaction = 物理删除禁止 + 監査必須 + RLS 13 類必携
- (c) Master = 物理删除禁止 + SCD Type 2 + RLS 13 類必携
- (d) Master 100% RLS / Transaction 100% audit / Work 100% retention_period

**RLS 13 类** (per 守门 #13 + [00-CLASSIFICATION-RULES.md v0.1](../../data-design/ipa-detail/00-CLASSIFICATION-RULES.md)):
- tenant_id / workspace_id / user_id / session_id / device_id / agent_id / role_id / permission_id / policy_id / event_id / tag_id / category_id / status_id 13 维隔离
- sub_task_binding: tenant_id + workspace_id + parent_task_id 3 维必携
- sub_task_runtime: tenant_id + device_id 2 维必携
- task_quota_config: tenant_id + parent_task_id 2 维必携

---

## §5 已知缺口 (per 守门 #12 缺标比错标安全)

- **G-SB-1**: UI 任务卡右键菜单仅 Next.js 前端实现, Tauri / VSCode / JetBrains IDE 适配未启动 (per [AGENTS.md §6.2 多 IDE 适配](../../AGENTS.md))
- **G-SB-2**: sub-task quota 默认值 (2 sub-task / 父, 100K tokens / sub-task) 未走 DDD Review 拍板, 待 P3-B 启动时 Lead 校准
- **G-SB-3**: ExclusiveBindingGuard 运行期守门在 L1 sub_pool 5 入口全覆盖未实证, E2E-15 待实装后跑
- **G-SB-4**: LifecycleLinkageManager 链式触发深度未限制 (理论 N 层, 实际 SubTaskQuotaRegistry 限 2 层, 需 DDD Review 确认)
- **G-SB-5**: LangGraph SDK 0.2.x interrupt_response API alpha 风险 (per [ADR-0046 §4.2 #3](../2026-08-26-upgrade/adr/0046-langgraph-task-management-operations.md)), spawn_subtask 走 L0 in-process 不依赖 alpha API, 但 child SA-XX 内部若用 interrupt 需实装前 `uv add langgraph@latest` 确认
- **G-SB-6**: 5 域 Lead 真人未到位 (per 守门 #3 反转 B 11:35 JST), sub-task 跨域编排决策仍 Mavis 临时代签, 真人到位后追溯签字, 不沿用代签决策 (per 守门 #1 禁回溯叙事)
- **G-SB-7**: M-N8 spawn_subtask 实装待 P0-1 / H2 阻塞解除 (per [ADR-0046 §4.2 #2](../2026-08-26-upgrade/adr/0046-langgraph-task-management-operations.md)), 当前文档 v0.1 落档, 实装 phase 跨 session 续做

**DDD Review 必查**: G-SB-1 / G-SB-2 / G-SB-4 / G-SB-6 + §3 NFR-SB-01..05 + §4 W/T/M 3 表横展開.

---

## §6 签字栏 (Signatures, per 7 段结构 5 角色)

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人 | Ulysses（一人公司 12 角色 per DEC-008）| 2026-09-09 | 🟡 Draft v0.1; 3 份 IPA 文档 (需求/基本/详细) 落档 + ADR-0052 决策记录 + LangGraph 02/03 升 v0.3 (TMO 7 → 8 节点); 跟 [01 §UC-14..UC-18 + F-26..F-32 + NFR-SB-01..05](#) + [02 §2.7 M-N8](./02-basic-design.md) + [03 §3.3 M-26..M-33](./03-detailed-design.md) 同步 |
| 1.1 | 架构师 / Mavis 接手审批 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-09 | 🟢 Mavis 接手终审通过 (per 2026-09-09 21:53 JST 用户发令"现在是否适合具有子代理功能, 通过在父任务卡交互下命令, 创建绑定子任务, 子任务有专属子代理" + ask_1df6987367ccc00928b65ee3 拍板 1 选项 "3 份新文档 + ADR-0052 升 v0.3 (推荐)"); 3 备选方案 (TMO M-N2 split 路径覆盖 / 仅 1 份 ADR / 直接进实装) 全部拒绝理由 + 5 风险 (3 实装 / 1 SDK alpha / 1 真人未到位) + 7 已知缺口 (G-SB-1..7) + 5 签字栏落档 |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-09 | 🟢 Mavis 接手代签 (per 19:39 + 21:59 JST); 5 域独立真实身份 (per 8/21 JST) 签字请 DDD Review 阶段补 |
| 3 | 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-09 | 🟢 Mavis 接手代签 (per 19:39 + 21:59 JST); 5 域独立真实身份签字请 DDD Review 阶段补 |
| 4 | 评审主持人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-09 | 🟢 Mavis 接手代签 (per 19:39 + 21:59 JST); 5 域独立真实身份签字请 DDD Review 阶段补 |
| 5 | 项目负责人 (PM) | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-09 | 🟢 Mavis 接手代签 (per 19:39 + 21:59 JST); 5 域独立真实身份签字请 DDD Review 阶段补 |

---

## §7 修订历史 (Revision History)

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-09 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: UC-14..UC-18 (5 use cases) + F-26..F-32 (7 functional requirements) + NFR-SB-01..05 (5 non-functional) + 3 表 W/T/M 横展開 + 7 已知缺口 (G-SB-1..7) + 5 签字栏 (Mavis 接手代签); 跟 [02-basic-design.md v0.1](./02-basic-design.md) + [03-detailed-design.md v0.1](./03-detailed-design.md) + [ADR-0052](../2026-08-26-upgrade/adr/0052-subtask-binding.md) + [PHASE-SUBTASK-BINDING-IMPL-REPORT.md](../../reports/PHASE-SUBTASK-BINDING-IMPL-REPORT.md) 同步落档 | 2026-09-09 21:53 JST 用户发令"现在是否适合具有子代理功能, 通过在父任务卡交互下命令, 创建绑定子任务, 子任务有专属子代理? 如果没有, 制定需求和基本设计详细设计" + ask_1df6987367ccc00928b65ee3 拍板 1 选项 "3 份新文档 + ADR-0052 升 v0.3 (推荐)", 跟 3 份新文档 + ADR-0052 + PHASE 报告 v0.1 同步落档, ~0.05M token 实测 |

---

## §8 参考 (References)

- [STAR LangGraph 統合アーキテクチャ 要件定義書 v0.2](../2026-09-03-langgraph/01-requirements.md) — UC-01..UC-13 + F-01..F-25 + NFR-P-01..05 + S-01..06 (前序, 本文档 v0.1 增量 5 UC + 7 F + 5 NFR)
- [02-basic-design.md v0.1](./02-basic-design.md) — §2.7 M-N8 + 8 组件 C-23..C-30 + 5 Reducer + 9 API 端点
- [03-detailed-design.md v0.1](./03-detailed-design.md) — spawn_subtask/ 模块 + M-26..M-33 + §3.3.1.1 7 节点 Python 実装 + UT-27..UT-33 / IT-13..IT-15 / E2E-14..E2E-18
- [ADR-0046 LangGraph TMO 7 节点](../2026-08-26-upgrade/adr/0046-langgraph-task-management-operations.md) — 7 节点 + 7 协议 + 7 组件 + 25 module (前序, 本文档 v0.1 增量 1 节点 + 1 协议 + 8 组件 + 8 module)
- [ADR-0044 STAR Agent Runtime SRS Baseline](../2026-08-26-upgrade/adr/0044-star-agent-runtime-srs.md) — 113 节 SRS + 12 节已落地 / 8 部分 / 60 待 P3-B-F / 4 N/A
- [ADR-0045 STAR Agent Runtime Design](../2026-08-26-upgrade/adr/0045-star-agent-runtime-design.md) — Agent Runtime view §3.5 + §1 ECS 选型
- [ADR-0030 Agent Lease/Heartbeat/Resume](../2026-08-26-upgrade/adr/0030-agent-lease-heartbeat-resume.md) — 11 字段 + 跨 Agent Handoff
- [ADR-0032 MCP Transport stdio](../2026-08-26-upgrade/adr/0032-mcp-transport-stdio.md) — 16 tools
- [ADR-0033 代签规则反转](../2026-08-26-upgrade/adr/0033-agent-co-signing-policy.md) — Mavis 接手代签授权
- [PHASE-SUBTASK-BINDING-IMPL-REPORT.md v0.1](../../reports/PHASE-SUBTASK-BINDING-IMPL-REPORT.md) — 8 子项实装 phase 计划 (M-N8 + C-23..C-30 + UI 入口)
- [AGENTS.md §3 报告 7 段结构 + §4 守门 #1-#24 + §4.1 累积规 v1-v24 + §6 ADR 索引](../../AGENTS.md)
- [docs/automation-design.md](../../automation-design.md) — agent 交互 Python 化 (守门 #19)
- [docs/data-design/ipa-detail/00-CLASSIFICATION-W-T-M.md v0.1](../../data-design/ipa-detail/00-CLASSIFICATION-W-T-M.md) — DB 三類橫展開 100 表索引
- [docs/data-design/ipa-detail/00-CLASSIFICATION-RULES.md v0.1](../../data-design/ipa-detail/00-CLASSIFICATION-RULES.md) — 跨项目 ルール手册 + 4 段检查清单 + 派生守门 10 条 CW-01~CW-10
- [LangGraph Documentation](https://langchain-ai.github.io/langgraph/) — StateGraph / Checkpoint / Subgraph / Interrupt / Command
- [STAR-OLU-001.md v0.1](../../STAR-OLU-001.md) — 1 SRE·周 = 1.2M tokens 独立基线 (sub-task 实装估 ~0.5-0.8M tokens)
- [STAR-P3-WBS-001.md v0.6 §7 阻塞 7 项](../../STAR-P3-WBS-001.md) — P3-B 启动前置
- [HANDOFF-ST-001.md v0.4 §5.3 Blocker](../../reports/HANDOFF-ST-001.md) — 5 项 Blocker 跨 session 续

---

# === Requirements 结束 ===

**per AGENTS.md §0 一句话硬约束 + §1 代签规则**: 可以代签 Ulysses, 不可以编造历史. 本文档 v0.1 引用守门 #1-#24 + 累积规 v1-v24 全部按 git 实证 + AGENTS.md 引用, 无"per X 历史形态"等回溯叙事.

**per 守门 #3 5 域单仓**: 本文档仅 STAR 仓内, 不引用 RGS 仓代码, 不建立业务子域↔DDD bounded context 映射 (per ADR-0044 §dual-use disclaimer).

**per 守门 #21 v21 [P] docs 同步**: automation-design.md §4.14 追加, commit message 引用相对路径.

**per 守门 #13 W/T/M 横展開**: 3 表 (sub_task_binding / sub_task_runtime / task_quota_config) 100% 覆盖 W/T/M 三類, 派生规 (a)(b)(c)(d) 全部落档.
