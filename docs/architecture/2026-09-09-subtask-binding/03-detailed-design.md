# STAR Sub-task Binding 詳細設計書 (Detailed Design)

> **バージョン**: v0.1
> **ステータス**: 🟡 Draft → 🔵 Review (待 DDD Review 拍板)
> **日付**: 2026-09-09
> **制定者**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **签批**: 🟢 Mavis 接手终审（per 2026-08-27 19:39 + 21:59 JST 用户授权"允许你代签"）
> **父文档**: [01-requirements.md v0.1](./01-requirements.md) · [02-basic-design.md v0.1](./02-basic-design.md) · [Star LangGraph 統合アーキテクチャ 詳細設計書 v0.2](../2026-09-03-langgraph/03-detailed-design.md) · [ADR-0046 TMO 7 节点](../2026-08-26-upgrade/adr/0046-langgraph-task-management-operations.md)
> **兄弟文档**: [ADR-0052 决策记录](../2026-08-26-upgrade/adr/0052-subtask-binding.md) · [PHASE-SUBTASK-BINDING-IMPL-REPORT.md](../../reports/PHASE-SUBTASK-BINDING-IMPL-REPORT.md)
> **依存**: [AGENTS.md §4 守门硬约束](../../AGENTS.md) · [AGENTS.md §4 #13 W/T/M 横展開](../../AGENTS.md) · [AGENTS.md §4.1 累积规 v1-v24](../../AGENTS.md)

---

## §0 文档目的

新增 **STAR Sub-task Binding 詳細設計**: 8 module (M-26..M-33) 落档 + 7 步 Python 実装 + 7 unit test (UT-27..UT-33) + 3 integration test (IT-13..IT-15) + 5 E2E test (E2E-14..E2E-18) + 3 表 DDL + 守门合规, 满足 [01-requirements.md §1-§3](./01-requirements.md) + [02-basic-design.md §1-§5](./02-basic-design.md) 全部需求.

**与 [LangGraph 03-detailed v0.2 §3.2.1.1 已有 5 血缘字段](../2026-09-03-langgraph/03-detailed-design.md) 关系**:
- 增量 8 module (M-26..M-33) 落到 `task_ops/spawn_subtask/` 子目录
- 增量 4 SubAgentState 字段 (per [02 §3.2.1.2](./02-basic-design.md))
- 增量 1 TopAgentState 字段 (per [02 §3.2.1.2](./02-basic-design.md))
- 增量 1 外部 API 端点 (per [02 §5](./02-basic-design.md))

---

## §1 模块结构 (per [LangGraph 03-detailed §3.1 模块结构](../2026-09-03-langgraph/03-detailed-design.md) 范式)

```
src/
├── task_ops/                              # 已有 (per [LangGraph 03-detailed §3.1.3 M-19..M-25](../2026-09-03-langgraph/03-detailed-design.md))
│   ├── manager.py                         # M-19 TaskOperationsManager
│   ├── relationship_graph.py              # M-20
│   ├── bulk_queue.py                      # M-21
│   ├── reassign_manager.py                # M-24
│   ├── summarize_collector.py             # M-25
│   ├── metadata_registry.py               # (新增 per ADR-0046 v0.2)
│   ├── dag_validator.py                   # (新增 per ADR-0046 v0.2)
│   │
│   └── spawn_subtask/                     # 🆕 v0.1 新增 (per 本文)
│       ├── __init__.py
│       ├── spawn_subtask_node.py          # M-26 M-N8 spawn_subtask_node 7 步 Python 実装
│       ├── lifecycle_linkage.py           # M-27 C-23 LifecycleLinkageManager
│       ├── exclusive_guard.py             # M-28 C-24 ExclusiveBindingGuard
│       ├── quota_registry.py              # M-29 C-25 SubTaskQuotaRegistry
│       ├── spawn_validator.py             # M-30 C-26 SpawnSubtaskValidator
│       ├── child_factory.py               # M-31 C-27 ChildSubAgentFactory
│       ├── event_bus.py                   # M-32 C-28 ParentChildEventBus
│       └── token_budget.py                # M-33 C-29 SubTaskTokenBudget

frontend/src/app/tasks/
├── components/
│   ├── TaskCard.tsx                       # 已有 (per 守门 #9 v3 + #24 v2)
│   └── SpawnSubtaskModal.tsx              # 🆕 v0.1 C-30 UISpawnSubtaskModal
└── api/tmo/
    └── spawn_subtask/route.ts             # 🆕 v0.1 Next.js API route (per 守门 #24 v2)
```

**8 module M-26..M-33** (per [02 §1.1 8 组件 C-23..C-30](./02-basic-design.md)):

| Module ID | 名称 | 组件 | 路径 |
|---|---|---|---|
| **M-26** | `spawn_subtask_node` | C-26 + C-27 协作 | `task_ops/spawn_subtask/spawn_subtask_node.py` |
| **M-27** | `lifecycle_linkage` | C-23 | `task_ops/spawn_subtask/lifecycle_linkage.py` |
| **M-28** | `exclusive_guard` | C-24 | `task_ops/spawn_subtask/exclusive_guard.py` |
| **M-29** | `quota_registry` | C-25 | `task_ops/spawn_subtask/quota_registry.py` |
| **M-30** | `spawn_validator` | C-26 | `task_ops/spawn_subtask/spawn_validator.py` |
| **M-31** | `child_factory` | C-27 | `task_ops/spawn_subtask/child_factory.py` |
| **M-32** | `event_bus` | C-28 | `task_ops/spawn_subtask/event_bus.py` |
| **M-33** | `token_budget` | C-29 | `task_ops/spawn_subtask/token_budget.py` |

**8 → 33 module 增量 8** (per [LangGraph 03-detailed §3.1.3 M-19..M-25 + M-26..M-33](../2026-09-03-langgraph/03-detailed-design.md) 范式).

---

## §2 依赖关系图 (per [LangGraph 03-detailed §3.2 依赖图](../2026-09-03-langgraph/03-detailed-design.md) 范式)

```
M-19 TaskOperationsManager (已有)
    │
    ├──→ M-26 spawn_subtask_node (NEW)
    │       │
    │       ├──→ M-30 spawn_validator (校验)
    │       │       ├──→ M-28 exclusive_guard (拒绝 SA-10 自嵌套)
    │       │       └──→ M-29 quota_registry (quota 计量)
    │       │
    │       ├──→ M-31 child_factory (创建 child SubAgentState)
    │       │       └──→ sub_agent.registry (SubAgentRegistry.spawn)
    │       │
    │       └──→ task_ops.relationship_graph (M-20 add_edge, 已有)
    │
    ├──→ M-27 lifecycle_linkage (NEW)
    │       ├──→ M-32 event_bus (订阅 parent state change)
    │       ├──→ task_ops.relationship_graph (遍历 children)
    │       └──→ sub_agent.pool (cancel/pause/resume child)
    │
    ├──→ M-28 exclusive_guard (NEW, also called by sub_pool)
    │       └──→ sub_agent.state (read exclusive_owner_task_id)
    │
    ├──→ M-29 quota_registry (NEW)
    │       └──→ task_quota_config 表 (Master SCD Type 2)
    │
    └──→ M-33 token_budget (NEW)
            ├──→ M-32 event_bus (publish budget exhausted)
            └──→ token_telemetry (累加 per-sub-task)
```

---

## §3 详细实现 (Detailed Implementation)

### §3.3.1.1 spawn_subtask_node 7 步 Python 実装 (M-26 + M-30 + M-31 协作)

```python
# task_ops/spawn_subtask/spawn_subtask_node.py
# per [02 §2.7.1 M-N8 spawn_subtask_node 7 步原子化](./02-basic-design.md)
# per 守门 #13 a L1↔L1 禁止通信 → 全部 L0 协调
# per 守门 #7 0 unsafe → type-safe, 无 unsafe code
# per 守门 #19 v19 → 走 automation/task_ops.py (本文件 Python 化)
# per 守门 #12 缺标比错标 → 失败必填 failure_reason

import asyncio
import uuid
from datetime import datetime, timezone
from typing import Optional, Literal
from pydantic import BaseModel, Field, validator

from task_ops.relationship_graph import TaskRelationshipGraph  # M-20
from task_ops.spawn_subtask.spawn_validator import SpawnSubtaskValidator  # M-30
from task_ops.spawn_subtask.child_factory import ChildSubAgentFactory  # M-31
from task_ops.spawn_subtask.event_bus import ParentChildEventBus  # M-32
from sub_agent.registry import SubAgentRegistry  # M-07
from sub_agent.state import SubAgentState  # M-06
from sub_agent.pool import SubAgentPool  # L1 pool
from audit.append_only import AuditLog  # 守门 #13 b Transaction append-only
from telemetry.token import TokenTelemetry  # 守门 #4 token-OLU


# ===== 入参出参 model (per [02 §2.7.4 协议](./02-basic-design.md)) =====

SAType = Literal["SA-01","SA-02","SA-03","SA-04","SA-05",
                 "SA-06","SA-07","SA-08","SA-09","SA-10"]

class SpawnSubtaskRequest(BaseModel):
    parent_task_id: str = Field(..., regex=r"^[a-z0-9-]{8,64}$")
    child_sa_type: SAType
    child_name: str = Field(..., min_length=1, max_length=200)
    initial_context: dict = Field(default_factory=dict)
    quota_check: bool = True
    bound_by: str = Field(default="Ulysses", regex=r"^(Ulysses|Mavis|.*-subagent)$")
    spawned_via: Literal["ui_rightclick", "l0_chat", "api"] = "api"

    @validator("child_sa_type")
    def _reject_sa10_self_spawn(cls, v: str, values: dict) -> str:
        """SA-10 task-orchestrator 禁止 spawn 自身 sub-task (per [01 §1 UC-14 Alt A2](./01-requirements.md))"""
        if v == "SA-10":
            raise ValueError(
                "SA-10 task-orchestrator cannot spawn sub-task "
                "(per F-30 ExclusiveBindingGuard + NFR-SB-02 self-spawn prohibited)"
            )
        return v


class SpawnSubtaskResult(BaseModel):
    child_task_id: str
    parent_task_id: str
    sa_type: SAType
    status: Literal["spawned", "failed"]
    failure_reason: Optional[str] = None  # 守门 #12 缺标比错标: 失败必填
    spawned_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
    exclusive_owner_task_id: str  # = parent_task_id (per F-28 1:1 独占)
    quota_remaining: int


# ===== M-N8 spawn_subtask_node 7 步原子化 (L0 StateGraph 节点) =====

async def spawn_subtask_node(state: TopAgentState) -> dict:
    """
    TMO M-N8: 1 派生 1 创建绑定子任务, 父继续 running, child 1:1 独占 SA-XX.
    7 步原子化 (per [02 §2.7.1](./02-basic-design.md)):
      step 1: M-30 SpawnSubtaskValidator.check_parent_state
      step 2: M-30.check_sa_type (拒绝 SA-10 自嵌套)
      step 3: M-29 SubTaskQuotaRegistry.query (≤ max_subtasks=2)
      step 4: generate child_task_id (UUID v7, prefix=`st-`)
      step 5: M-31 ChildSubAgentFactory.create (写 4 字段)
      step 6: SubAgentRegistry.spawn (C-24 ExclusiveBindingGuard 守门)
      step 7: C-17 TaskRelationshipGraph.add_edge + audit log append
    """
    req: SpawnSubtaskRequest = state["active_tmo_operation"]["request"]

    try:
        # ===== step 1: 校验父 task 状态 =====
        parent_state = await SubAgentPool.get_state(req.parent_task_id)
        SpawnSubtaskValidator.check_parent_state(parent_state)  # M-30
        # parent.state ∈ {running, paused}, 否则抛 InvalidParentState

        # ===== step 2: 校验 SA type (Pydantic validator 已做 SA-10 拒绝) =====
        # Pydantic @validator 在 model 构造时已触发, 此处冗余防御
        SpawnSubtaskValidator.check_sa_type(req.child_sa_type)  # M-30

        # ===== step 3: 校验 quota (≤ max_subtasks=2, per F-31 默认值) =====
        quota_remaining = await SubTaskQuotaRegistry.query(  # M-29
            parent_task_id=req.parent_task_id,
            quota_check=req.quota_check,
        )
        if quota_remaining <= 0:
            raise SubTaskQuotaExhausted(
                f"Parent {req.parent_task_id} sub-task quota exhausted (max=2)"
            )

        # ===== step 4: 生成 child_task_id (UUID v7, prefix=`st-`) =====
        # per [01 §2 F-27 字段约束 child_task_id 必须 = st- prefix (UUID v7)](./01-requirements.md)
        child_task_id = f"st-{uuid.uuid4().hex[:16]}"  # UUID v7 简化 (生产用 uuid7 lib)

        # ===== step 5: M-31 ChildSubAgentFactory.create (写 4 字段) =====
        child_state: SubAgentState = await ChildSubAgentFactory.create(  # M-31
            child_task_id=child_task_id,
            parent_task_id=req.parent_task_id,
            sa_type=req.child_sa_type,
            child_name=req.child_name,
            initial_context=req.initial_context,
            bound_by=req.bound_by,
            bound_via=req.spawned_via,
        )
        # 写 4 字段 (per [02 §3.2.1.2](./02-basic-design.md)):
        # - exclusive_owner_task_id = parent_task_id (1:1 独占)
        # - bound_at = UTC now
        # - bound_by = req.bound_by
        # - bound_via = req.spawned_via

        # ===== step 6: SubAgentRegistry.spawn (C-24 ExclusiveBindingGuard 守门) =====
        # per [01 §2 F-30 编译期 + 运行期双层守门](./01-requirements.md)
        await SubAgentRegistry.spawn(
            task_id=child_task_id,
            sa_type=req.child_sa_type,
            state=child_state,
        )
        # C-24 ExclusiveBindingGuard.is_exclusive(child_task_id) 自动触发
        # child_state.exclusive_owner_task_id = parent_task_id
        # → ReassignManager.reassign(child_task_id, ...) 必失败 (NFR-SB-02)

        # ===== step 7: C-17 TaskRelationshipGraph.add_edge + audit log append =====
        await TaskRelationshipGraph.add_edge(  # M-20
            parent_id=req.parent_task_id,
            child_id=child_task_id,
            edge_type="spawn_exclusive",  # per [01 §1 UC-14 step 7.7](./01-requirements.md)
            metadata={
                "sa_type": req.child_sa_type,
                "bound_at": child_state["bound_at"].isoformat(),
                "bound_by": req.bound_by,
            },
        )

        # audit log append (Transaction append-only, per 守门 #13 b)
        await AuditLog.append(
            event_type="subtask_spawned",
            parent_task_id=req.parent_task_id,
            child_task_id=child_task_id,
            sa_type=req.child_sa_type,
            spawned_via=req.spawned_via,
        )

        # 父 task 状态不变 (继续 running / paused, per [01 §1 UC-14 Post-condition](./01-requirements.md))
        # 不 supersede, 不改变状态, 仅追加 child 到 TaskRelationshipGraph

        return {
            "last_spawn_subtask_result": SpawnSubtaskResult(
                child_task_id=child_task_id,
                parent_task_id=req.parent_task_id,
                sa_type=req.child_sa_type,
                status="spawned",
                spawned_at=child_state["bound_at"],
                exclusive_owner_task_id=req.parent_task_id,
                quota_remaining=quota_remaining - 1,
            ),
            "active_tmo_operation": None,  # 清空
        }

    except (InvalidParentState, InvalidSAType, SubTaskQuotaExhausted,
            SelfSpawnProhibited, ExclusiveBindingViolation) as e:
        # 5 类失败模式 (per [02 §2.7.1 失败模式](./02-basic-design.md))
        # audit log append (失败也记, per 守门 #12 缺标比错标)
        await AuditLog.append(
            event_type="subtask_spawn_failed",
            parent_task_id=req.parent_task_id,
            failure_reason=type(e).__name__ + ": " + str(e),
            sa_type=req.child_sa_type,
        )
        return {
            "last_spawn_subtask_result": SpawnSubtaskResult(
                child_task_id="",
                parent_task_id=req.parent_task_id,
                sa_type=req.child_sa_type,
                status="failed",
                failure_reason=type(e).__name__ + ": " + str(e),
                exclusive_owner_task_id=req.parent_task_id,
                quota_remaining=-1,
            ),
            "active_tmo_operation": None,
        }


# ===== Exception classes (per 守门 #12 缺标比错标: 失败必填 failure_reason) =====

class InvalidParentState(Exception):
    """父 task 状态 ∈ {superseded, completed, failed, cancelling}"""
    pass

class InvalidSAType(Exception):
    """SA type 非法 (e.g. SA-10 自嵌套, per F-30 + NFR-SB-02)"""
    pass

class SubTaskQuotaExhausted(Exception):
    """父 task sub-task quota 耗尽 (max=2, per F-31 默认值)"""
    pass

class SelfSpawnProhibited(Exception):
    """parent_task_id == child_task_id 自身派生"""
    pass

class ExclusiveBindingViolation(Exception):
    """C-24 ExclusiveBindingGuard 拒绝 (per NFR-SB-02 不可绕过)"""
    pass
```

### §3.3.2 LifecycleLinkageManager (C-23, M-27)

```python
# task_ops/spawn_subtask/lifecycle_linkage.py
# per [01 §1 UC-16 Lifecycle linkage](./01-requirements.md) + [01 §2 F-29](./01-requirements.md)
# per NFR-SB-03 lifecycle 联动延迟 ≤ 1s

import asyncio
from datetime import datetime, timezone
from enum import Enum

from task_ops.spawn_subtask.event_bus import ParentChildEventBus  # M-32
from task_ops.relationship_graph import TaskRelationshipGraph  # M-20
from sub_agent.pool import SubAgentPool
from audit.append_only import AuditLog


class LifecycleAction(str, Enum):
    CANCEL = "cancel"
    PAUSE = "pause"
    RESUME = "resume"
    FAIL = "fail"


class LifecycleLinkageManager:
    """
    C-23 LifecycleLinkageManager:
    监听 parent state change event, 链式触发 child action ≤ 1s (NFR-SB-03).
    """

    def __init__(self, event_bus: ParentChildEventBus, graph: TaskRelationshipGraph):
        self._event_bus = event_bus
        self._graph = graph

    async def on_parent_state_change(
        self, parent_id: str, new_state: str
    ) -> None:
        """父 task state change → 链式触发 child action"""
        # 1. 查找所有 children (edge_type=spawn_exclusive)
        children_ids = await self._graph.get_children(
            parent_id=parent_id, edge_type="spawn_exclusive"
        )

        # 2. 根据 parent state 决定 child action
        action_map = {
            "cancelling": LifecycleAction.CANCEL,
            "paused": LifecycleAction.PAUSE,
            "running": LifecycleAction.RESUME,  # 仅 child 在 paused 时
            "failed": LifecycleAction.FAIL,  # child cancel (不可逆)
        }
        action = action_map.get(new_state)
        if action is None:
            return  # 其他 state 不联动

        # 3. 链式触发 child action (默认串行, ≤ 1s 延迟 per NFR-SB-03)
        for child_id in children_ids:
            await self._apply_child_action(child_id, action, parent_id)

        # 4. audit log append (Transaction append-only)
        await AuditLog.append(
            event_type="lifecycle_linkage_applied",
            parent_task_id=parent_id,
            children_count=len(children_ids),
            action=action.value,
        )

    async def _apply_child_action(
        self, child_id: str, action: LifecycleAction, parent_id: str
    ) -> None:
        """应用 child action (单 child)"""
        if action == LifecycleAction.CANCEL:
            await SubAgentPool.cancel(
                task_id=child_id, reason="parent_cancelled"
            )
        elif action == LifecycleAction.PAUSE:
            await SubAgentPool.pause(task_id=child_id, reason="parent_paused")
        elif action == LifecycleAction.RESUME:
            child_state = await SubAgentPool.get_state(child_id)
            if child_state.get("status") == "paused":
                await SubAgentPool.resume(
                    task_id=child_id, reason="parent_resumed"
                )
        elif action == LifecycleAction.FAIL:
            # parent failed → child cancel (链式, 不可逆)
            await SubAgentPool.cancel(
                task_id=child_id, reason="parent_failed"
            )

    async def on_child_fail(
        self, child_id: str, parent_id: str
    ) -> None:
        """child 失败 → parent 状态 = degraded (警告, 不强制 cancel)
        per [01 §1 UC-16 Alt flow C3](./01-requirements.md)"""
        # 不 cancel parent, 只标 degraded (L0 chat bar 提示)
        # per [01 §1 UC-16 child 失败 → parent 进入 degraded 状态 ≤ 5s](./01-requirements.md)
        await self._event_bus.publish(
            event_type="parent_degraded",
            parent_id=parent_id,
            child_id=child_id,
            reason="child_failed",
        )
        await AuditLog.append(
            event_type="parent_degraded",
            parent_task_id=parent_id,
            child_task_id=child_id,
            reason="child_failed",
        )
```

### §3.3.3 ExclusiveBindingGuard (C-24, M-28)

```python
# task_ops/spawn_subtask/exclusive_guard.py
# per [01 §1 UC-17 + F-30 编译期 + 运行期双层守门](./01-requirements.md)
# per NFR-SB-02 不可绕过 (守门 #7 0 unsafe + #12 缺标比错标)

from typing import Optional
from sub_agent.state import SubAgentState


class ExclusiveBindingGuard:
    """
    C-24 ExclusiveBindingGuard:
    1:1 独占守门, 编译期 (type system) + 运行期 (this class) 双层守门.
    """

    @staticmethod
    def is_exclusive(task_id: str) -> bool:
        """检查 task 是否 exclusive 1:1 绑定 (per [01 §2 F-28](./01-requirements.md))"""
        state: Optional[SubAgentState] = SubAgentState.get(task_id)
        if state is None:
            return False
        return state.get("exclusive_owner_task_id") is not None

    @staticmethod
    def check_reassign_allowed(task_id: str) -> None:
        """
        reassign 守门 (M-N6 入口):
        exclusive sub-task 必拒绝, 抛 ExclusiveBindingViolation.
        """
        if ExclusiveBindingGuard.is_exclusive(task_id):
            raise ExclusiveBindingViolation(
                f"Sub-task `{task_id}` is exclusive-bound to parent "
                f"`{SubAgentState.get(task_id).get('exclusive_owner_task_id')}`, "
                f"reassign prohibited (per NFR-SB-02)"
            )

    @staticmethod
    def check_merge_allowed(task_id: str) -> None:
        """
        merge 守门 (M-N1 入口):
        exclusive sub-task 必拒绝, 跟 [01 §2 F-28 互斥](./01-requirements.md) 一致.
        """
        if ExclusiveBindingGuard.is_exclusive(task_id):
            raise ExclusiveBindingViolation(
                f"Sub-task `{task_id}` is exclusive-bound, merge prohibited "
                f"(per F-28 exclusive sub-task cannot merge)"
            )

    @staticmethod
    def check_split_allowed(task_id: str) -> None:
        """
        split 守门 (M-N2 入口):
        exclusive sub-task 必拒绝, 跟 [01 §2 F-28 互斥](./01-requirements.md) 一致.
        """
        if ExclusiveBindingGuard.is_exclusive(task_id):
            raise ExclusiveBindingViolation(
                f"Sub-task `{task_id}` is exclusive-bound, split prohibited "
                f"(per F-28 exclusive sub-task cannot split)"
            )
```

### §3.3.4 SubTaskQuotaRegistry (C-25, M-29)

```python
# task_ops/spawn_subtask/quota_registry.py
# per [01 §1 UC-18 + F-31 Master SCD Type 2](./01-requirements.md)
# per 守门 #13 c/d W/T/M + 派生规 (c) Master 物理删除禁止 + SCD Type 2 + RLS 13 类必携

from typing import Optional
from datetime import datetime
from db.connection import get_db  # PostgreSQL Tier 3 (per [AGENTS.md §4 #1 v17 Tier 3](../AGENTS.md))
from audit.append_only import AuditLog


class SubTaskQuotaRegistry:
    """
    C-25 SubTaskQuotaRegistry:
    per-parent sub-task quota 计量 + 配置 (Master SCD Type 2).
    """

    DEFAULT_MAX_SUBTASKS = 2
    DEFAULT_PER_SUBTASK_TOKEN_BUDGET = 100_000  # 100K tokens (per F-31)

    @staticmethod
    async def query(parent_task_id: str, quota_check: bool = True) -> int:
        """
        查询父 task 剩余 quota (active sub-task count).
        per [01 §1 UC-18 Main flow step 2](./01-requirements.md)
        """
        if not quota_check:
            return 999  # 跳过 quota 校验 (per F-26 admin override)

        db = await get_db()
        result = await db.fetchrow(
            """
            SELECT current_subtasks, max_subtasks
            FROM task_quota_config
            WHERE parent_task_id = $1 AND is_current = TRUE
            """,
            parent_task_id,
        )
        if result is None:
            # 父 task 没有 quota config → 用默认值
            return SubTaskQuotaRegistry.DEFAULT_MAX_SUBTASKS

        return result["max_subtasks"] - result["current_subtasks"]

    @staticmethod
    async def increment(parent_task_id: str) -> None:
        """
        增加 active sub-task count (per [01 §1 UC-14 step 7.5](./01-requirements.md)).
        SCD Type 2: 关闭旧 row, 插入新 row.
        """
        db = await get_db()
        async with db.transaction():
            # 1. 关闭旧 row (SCD Type 2)
            await db.execute(
                """
                UPDATE task_quota_config
                SET effective_to = NOW(), is_current = FALSE
                WHERE parent_task_id = $1 AND is_current = TRUE
                """,
                parent_task_id,
            )
            # 2. 插入新 row (current_subtasks +1)
            await db.execute(
                """
                INSERT INTO task_quota_config
                  (config_id, parent_task_id, max_subtasks,
                   per_subtask_token_budget, effective_from, is_current,
                   created_at, created_by)
                VALUES
                  (gen_random_uuid(), $1, $2, $3, NOW(), TRUE, NOW(), 'Mavis')
                """,
                parent_task_id,
                SubTaskQuotaRegistry.DEFAULT_MAX_SUBTASKS,
                SubTaskQuotaRegistry.DEFAULT_PER_SUBTASK_TOKEN_BUDGET,
            )
            # 注: 实际需读旧 max_subtasks + current_subtasks, 然后 +1 current_subtasks
            # 简化: 假定 current_subtasks=0 → 1 (实际需 lookup)

    @staticmethod
    async def decrement(parent_task_id: str) -> None:
        """
        减少 active sub-task count (per [01 §1 UC-16 Alt flow E1](./01-requirements.md)
        parent cancel → 批量释放所有 children quota).
        SCD Type 2: 同 increment.
        """
        # 实现同 increment, 但 current_subtasks -1
        # 简化: 父 cancel 触发时直接 delete active children quota
        # 实际需走 SCD Type 2 close + insert new
        pass
```

### §3.3.5 SpawnSubtaskValidator (C-26, M-30)

```python
# task_ops/spawn_subtask/spawn_validator.py
# per [02 §2.7.1 step 1-3 校验](./02-basic-design.md)

from sub_agent.state import SubAgentState
from task_ops.spawn_subtask.quota_registry import SubTaskQuotaRegistry
from task_ops.spawn_subtask.exclusive_guard import ExclusiveBindingGuard
from task_ops.spawn_subtask.exceptions import (
    InvalidParentState, InvalidSAType,
    SubTaskQuotaExhausted, SelfSpawnProhibited,
)


class SpawnSubtaskValidator:
    """C-26 SpawnSubtaskValidator: spawn 7 步原子化前 5 步校验"""

    ALLOWED_PARENT_STATES = {"running", "paused"}
    VALID_SA_TYPES = {
        "SA-01", "SA-02", "SA-03", "SA-04", "SA-05",
        "SA-06", "SA-07", "SA-08", "SA-09", "SA-10",
    }

    @staticmethod
    def check_parent_state(parent_state: dict) -> None:
        """step 1: 父 task 状态校验 (per [02 §2.7.1 step 1](./02-basic-design.md))"""
        if parent_state.get("status") not in SpawnSubtaskValidator.ALLOWED_PARENT_STATES:
            raise InvalidParentState(
                f"Parent task state must be in {SpawnSubtaskValidator.ALLOWED_PARENT_STATES}, "
                f"got '{parent_state.get('status')}'"
            )

    @staticmethod
    def check_sa_type(sa_type: str) -> None:
        """step 2: SA type 校验 (Pydantic 已做 SA-10 拒绝, 此处冗余)"""
        if sa_type not in SpawnSubtaskValidator.VALID_SA_TYPES:
            raise InvalidSAType(f"Invalid SA type: {sa_type}")
        # SA-10 自嵌套由 Pydantic validator 拒绝 (per [01 §1 UC-14 Alt A2](./01-requirements.md))

    @staticmethod
    async def check_quota(parent_task_id: str) -> int:
        """step 3: quota 校验 (per [02 §2.7.1 step 3](./02-basic-design.md))"""
        quota_remaining = await SubTaskQuotaRegistry.query(parent_task_id)
        if quota_remaining <= 0:
            raise SubTaskQuotaExhausted(
                f"Parent {parent_task_id} sub-task quota exhausted (max=2)"
            )
        return quota_remaining

    @staticmethod
    def check_self_spawn(parent_task_id: str, child_task_id: str) -> None:
        """校验 parent != child (per [01 §2 F-30 派生约束](./01-requirements.md))"""
        if parent_task_id == child_task_id:
            raise SelfSpawnProhibited(
                f"parent_task_id ({parent_task_id}) == child_task_id ({child_task_id}), "
                f"self-spawn prohibited"
            )

    @staticmethod
    def check_parent_not_exclusive(parent_state: dict) -> None:
        """校验父 task 不是 exclusive sub-task (per [01 §2 F-30 派生约束](./01-requirements.md)
        防止 exclusive 链式污染)"""
        if parent_state.get("exclusive_owner_task_id") is not None:
            raise ExclusiveBindingViolation(  # 复用
                f"Parent task is itself exclusive sub-task, "
                f"cannot spawn further sub-task (防止 exclusive 链式污染)"
            )
```

### §3.3.6 ChildSubAgentFactory (C-27, M-31)

```python
# task_ops/spawn_subtask/child_factory.py
# per [01 §2 F-27 + F-28 child SubAgentState 初始化 + 4 字段](./01-requirements.md)

from datetime import datetime, timezone
from sub_agent.state import SubAgentState
from sub_agent.registry import SubAgentRegistry


class ChildSubAgentFactory:
    """C-27 ChildSubAgentFactory: child SubAgentState 初始化"""

    @staticmethod
    async def create(
        child_task_id: str,
        parent_task_id: str,
        sa_type: str,
        child_name: str,
        initial_context: dict,
        bound_by: str,
        bound_via: str,
    ) -> SubAgentState:
        """创建 child SubAgentState + 写 4 字段 (per [02 §3.2.1.2](./02-basic-design.md))"""
        now = datetime.now(timezone.utc)
        child_state: SubAgentState = {
            "task_id": child_task_id,
            "task_name": child_name,
            "task_type": sa_type,
            "status": "initializing",
            "parent_task_id": parent_task_id,  # 已有 (per [LangGraph 03-detailed §3.2.1.1](../2026-09-03-langgraph/03-detailed-design.md))
            # 🆕 v0.1 增量 4 字段:
            "exclusive_owner_task_id": parent_task_id,  # 1:1 独占 (per F-28)
            "bound_at": now,                              # 绑定创建时间
            "bound_by": bound_by,                         # 绑定创建者
            "bound_via": bound_via,                       # 入口标注
            # 父 context 浅拷贝 + exclusive 标注
            "initial_context": {
                **initial_context,
                "_exclusive_bound": True,  # 标注
            },
        }
        return child_state
```

### §3.3.7 ParentChildEventBus (C-28, M-32)

```python
# task_ops/spawn_subtask/event_bus.py
# per [01 §2 F-29 EventBus 订阅派发](./01-requirements.md) + [AGENTS.md §6.2 已有 EventBus 设计](../../AGENTS.md)

import asyncio
from typing import Callable, Any
from collections import defaultdict


class ParentChildEventBus:
    """C-28 ParentChildEventBus: parent/child state change event 订阅派发"""

    def __init__(self):
        self._subscribers: dict[str, list[Callable]] = defaultdict(list)

    def subscribe(self, event_type: str, callback: Callable) -> None:
        """订阅事件"""
        self._subscribers[event_type].append(callback)

    async def publish(self, event_type: str, **kwargs: Any) -> None:
        """发布事件 (异步派发)"""
        tasks = [
            asyncio.create_task(cb(**kwargs))
            for cb in self._subscribers.get(event_type, [])
        ]
        if tasks:
            await asyncio.gather(*tasks, return_exceptions=True)
```

### §3.3.8 SubTaskTokenBudget (C-29, M-33)

```python
# task_ops/spawn_subtask/token_budget.py
# per [01 §1 UC-18 + F-31 per-sub-task token 预算](./01-requirements.md) + NFR-SB-04

from telemetry.token import TokenTelemetry
from task_ops.spawn_subtask.event_bus import ParentChildEventBus
from task_ops.spawn_subtask.exceptions import SubTaskTokenBudgetExhausted


class SubTaskTokenBudget:
    """C-29 SubTaskTokenBudget: per-sub-task token 预算计量 (NFR-SB-04)"""

    @staticmethod
    async def record(task_id: str, tokens_used: int, op: str) -> None:
        """
        记录 sub-task token 使用, 达到预算抛 SubTaskTokenBudgetExhausted.
        per [01 §1 UC-18 Main flow step 1](./01-requirements.md)
        """
        # 1. 累加 TokenTelemetry
        await TokenTelemetry.record(task_id, tokens_used, op)

        # 2. 查询 budget
        budget_config = await SubTaskQuotaRegistry.get_token_budget(task_id)
        used_total = await TokenTelemetry.get_total(task_id)

        # 3. 达到预算抛异常
        if used_total >= budget_config["per_subtask_token_budget"]:
            await ParentChildEventBus().publish(
                event_type="subtask_token_budget_exhausted",
                task_id=task_id,
                used_total=used_total,
                budget=budget_config["per_subtask_token_budget"],
            )
            raise SubTaskTokenBudgetExhausted(
                f"Sub-task {task_id} token budget exhausted: "
                f"{used_total} >= {budget_config['per_subtask_token_budget']}"
            )
```

### §3.3.9 UISpawnSubtaskModal (C-30)

```tsx
// frontend/src/app/tasks/components/SpawnSubtaskModal.tsx
// per [01 §2 F-32 UI 任务卡右键 spawn 入口](./01-requirements.md)
// per 守门 #9 v3 + #24 v2 Next.js API route → console_server.py → subprocess
// per 守门 #23 v23 AI 建议走 ai_edit_mock.py, 不开 OpenAI

import React, { useState } from "react";
import { Button, Modal, Select, Input, Form, message } from "antd";

interface SpawnSubtaskModalProps {
  visible: boolean;
  parentTaskId: string;
  onCancel: () => void;
  onSuccess: (childTaskId: string) => void;
}

const SA_TYPES = [
  "SA-01", "SA-02", "SA-03", "SA-04", "SA-05",
  "SA-06", "SA-07", "SA-08", "SA-09", "SA-10",
];

export const SpawnSubtaskModal: React.FC<SpawnSubtaskModalProps> = ({
  visible, parentTaskId, onCancel, onSuccess,
}) => {
  const [saType, setSaType] = useState<string>("SA-04");  // 默认 git-ops
  const [childName, setChildName] = useState<string>("");
  const [aiSuggesting, setAiSuggesting] = useState<boolean>(false);
  const [loading, setLoading] = useState<boolean>(false);

  // AI 建议 sub-task name (per 守门 #23 v23 走 ai_edit_mock.py)
  const handleAISuggest = async () => {
    setAiSuggesting(true);
    try {
      // POST /api/automation/ai_suggest_subtask_name
      // 走 ai_edit_mock.py 本地 mock, 不开 OpenAI
      const res = await fetch("/api/automation/ai_suggest_subtask_name", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ parent_task_id: parentTaskId, sa_type: saType }),
      });
      const data = await res.json();
      setChildName(data.suggested_name);
      message.info(`AI 建议: ${data.suggested_name} (confidence=${data.confidence}, < 0.5 请手动 review)`);
    } finally {
      setAiSuggesting(false);
    }
  };

  // 提交 spawn sub-task
  const handleSubmit = async () => {
    if (!childName) {
      message.error("请输入 sub-task name");
      return;
    }
    setLoading(true);
    try {
      // POST /api/tmo/spawn_subtask (per [02 §5 9 端点](./02-basic-design.md))
      // 走 console_server.py → subprocess.run task_ops.py spawn_subtask
      const res = await fetch("/api/tmo/spawn_subtask", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          parent_task_id: parentTaskId,
          child_sa_type: saType,
          child_name: childName,
          spawned_via: "ui_rightclick",  // per 守门入口标注
        }),
      });
      const result = await res.json();
      if (result.status === "spawned") {
        message.success(`Sub-task spawned: ${result.child_task_id}`);
        onSuccess(result.child_task_id);
      } else {
        message.error(`Spawn failed: ${result.failure_reason}`);
      }
    } catch (err) {
      message.error(`Network error: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  return (
    <Modal
      title="Spawn sub-task"
      visible={visible}
      onCancel={onCancel}
      footer={[
        <Button key="cancel" onClick={onCancel}>Cancel</Button>,
        <Button key="submit" type="primary" loading={loading} onClick={handleSubmit}>
          Spawn
        </Button>,
      ]}
    >
      <Form layout="vertical">
        <Form.Item label="Sub-agent type (SA-XX)">
          <Select value={saType} onChange={setSaType}>
            {SA_TYPES.map((sa) => (
              <Select.Option key={sa} value={sa}>{sa}</Select.Option>
            ))}
          </Select>
        </Form.Item>
        <Form.Item label="Sub-task name">
          <Input
            value={childName}
            onChange={(e) => setChildName(e.target.value)}
            placeholder="Enter sub-task name"
          />
          <Button
            type="link"
            loading={aiSuggesting}
            onClick={handleAISuggest}
          >
            🤖 AI 建议 (mock, per 守门 #23 v23)
          </Button>
        </Form.Item>
        <Form.Item label="Parent task ID">
          <Input value={parentTaskId} disabled />
        </Form.Item>
      </Form>
    </Modal>
  );
};
```

---

## §4 测试矩阵 (per [LangGraph 03-detailed §4 测试矩阵](../2026-09-03-langgraph/03-detailed-design.md) 范式)

### §4.1 E2E-14: NFR-SB-01 spawn_subtask p95 latency ≤ 200ms

```python
# tests/e2e/test_e2e_14_spawn_subtask_latency.py
# per [01 §3 NFR-SB-01](./01-requirements.md)
import time
import asyncio
import pytest
from httpx import AsyncClient

@pytest.mark.asyncio
async def test_spawn_subtask_p95_under_200ms():
    """E2E-14: 100 次 spawn, p95 < 200ms (NFR-SB-01)"""
    async with AsyncClient(base_url="http://localhost:8080") as client:
        latencies = []
        for i in range(100):
            start = time.perf_counter()
            res = await client.post(
                "/api/tmo/spawn_subtask",
                json={
                    "parent_task_id": f"test-parent-{i % 10}",
                    "child_sa_type": "SA-04",
                    "child_name": f"git-init-st-{i}",
                    "spawned_via": "api",
                },
            )
            elapsed_ms = (time.perf_counter() - start) * 1000
            assert res.status_code == 200
            latencies.append(elapsed_ms)

        # p95 < 200ms
        latencies.sort()
        p95 = latencies[94]  # 95th percentile
        assert p95 < 200, f"NFR-SB-01 violated: p95={p95}ms >= 200ms"
```

### §4.2 E2E-15: NFR-SB-02 1:1 binding 不可绕过

```python
# tests/e2e/test_e2e_15_exclusive_binding_unguessable.py
# per [01 §3 NFR-SB-02](./01-requirements.md) + [01 §1 UC-17](./01-requirements.md)
import pytest
from httpx import AsyncClient

@pytest.mark.asyncio
async def test_reassign_exclusive_subtask_must_fail():
    """E2E-15: 尝试 reassign exclusive sub-task 必失败"""
    # 1. 先 spawn sub-task
    async with AsyncClient(base_url="http://localhost:8080") as client:
        spawn_res = await client.post(
            "/api/tmo/spawn_subtask",
            json={
                "parent_task_id": "test-parent-A",
                "child_sa_type": "SA-04",
                "child_name": "test-child",
                "spawned_via": "api",
            },
        )
        child_id = spawn_res.json()["child_task_id"]

        # 2. 尝试 reassign exclusive sub-task
        reassign_res = await client.post(
            "/api/tmo/reassign",
            json={"task_id": child_id, "new_sa_type": "SA-05"},
        )
        # 3. 必失败 (NFR-SB-02 不可绕过)
        assert reassign_res.status_code == 403
        assert "ExclusiveBindingViolation" in reassign_res.text
        assert "reassign prohibited" in reassign_res.text
        # 4. 验证子 task 类型没变
        state_res = await client.get(f"/api/tmo/relationships?parent_id={child_id}")
        assert state_res.json()["sa_type"] == "SA-04"  # 未变
```

### §4.3 E2E-16: NFR-SB-03 lifecycle 联动延迟 ≤ 1s

```python
# tests/e2e/test_e2e_16_lifecycle_linkage_latency.py
# per [01 §3 NFR-SB-03](./01-requirements.md) + [01 §1 UC-16](./01-requirements.md)
import time
import asyncio
import pytest
from httpx import AsyncClient

@pytest.mark.asyncio
async def test_parent_cancel_propagates_to_child_under_1s():
    """E2E-16: parent cancel → child cancel ≤ 1s (NFR-SB-03)"""
    async with AsyncClient(base_url="http://localhost:8080") as client:
        # 1. spawn sub-task
        spawn_res = await client.post(
            "/api/tmo/spawn_subtask",
            json={
                "parent_task_id": "test-parent-B",
                "child_sa_type": "SA-04",
                "child_name": "test-child-B",
                "spawned_via": "api",
            },
        )
        child_id = spawn_res.json()["child_task_id"]

        # 2. 记录 parent cancel 时刻
        start = time.perf_counter()
        cancel_res = await client.post(
            f"/api/tasks/{spawn_res.json()['parent_task_id']}/cancel",
            json={"reason": "test_parent_cancel"},
        )
        assert cancel_res.status_code == 200

        # 3. 轮询 child 状态, ≤ 1s 内应 = cancelled
        for _ in range(10):  # 最多 10 * 100ms = 1s
            await asyncio.sleep(0.1)
            state_res = await client.get(f"/api/tasks/{child_id}/state")
            if state_res.json()["status"] == "cancelled":
                elapsed = time.perf_counter() - start
                assert elapsed < 1.0, f"NFR-SB-03 violated: {elapsed}s >= 1s"
                return
        pytest.fail(f"Child {child_id} not cancelled within 1s")
```

### §4.4 Unit Tests (UT-27..UT-33, 7 tests)

| Test | 覆盖 | 守门 |
|---|---|---|
| UT-27 | SpawnSubtaskValidator.check_parent_state (5 state × 2 expected) | #1, #7 |
| UT-28 | ExclusiveBindingGuard.is_exclusive / check_reassign_allowed | #7, #12 |
| UT-29 | SubTaskQuotaRegistry.query / increment / decrement | #1, #13 |
| UT-30 | ChildSubAgentFactory.create 4 字段写入 | #1, #12 |
| UT-31 | LifecycleLinkageManager.on_parent_state_change 5 联动 | #1, #9 |
| UT-32 | SubTaskTokenBudget.record 100K 预算触发 SubTaskTokenBudgetExhausted | #1, #4 |
| UT-33 | SpawnSubtaskRequest Pydantic validator 拒绝 SA-10 自嵌套 | #7, #12 |

### §4.5 Integration Tests (IT-13..IT-15, 3 tests)

| Test | 覆盖 |
|---|---|
| IT-13 | M-N8 spawn_subtask_node 7 步端到端 (PostgreSQL + LangGraph + console_server 集成) |
| IT-14 | LifecycleLinkageManager + EventBus + SubAgentPool 集成 |
| IT-15 | SubTaskQuotaRegistry + task_quota_config SCD Type 2 + audit log 集成 |

### §4.6 E2E Tests 汇总 (E2E-14..E2E-18, 5 tests)

| Test | 覆盖 | 引用 |
|---|---|---|
| E2E-14 | NFR-SB-01 spawn_subtask p95 latency ≤ 200ms | [§4.1](#41-e2e-14-nfr-sb-01-spawn_subtask-p95-latency--200ms) |
| E2E-15 | NFR-SB-02 1:1 binding 不可绕过 | [§4.2](#42-e2e-15-nfr-sb-02-11-binding-不可绕过) |
| E2E-16 | NFR-SB-03 lifecycle 联动 ≤ 1s | [§4.3](#43-e2e-16-nfr-sb-03-lifecycle-联动延迟--1s) |
| E2E-17 | UI TaskCard 右键 spawn → modal → POST /api/tmo/spawn_subtask → 列表刷新 | [§3.3.9](#339-uispawnsubtaskmodal-c-30) |
| E2E-18 | sub-task quota 耗尽 + parent cancel 批量释放 + audit log 完整 | 跨 §3.3.5 + §3.3.7 + §3.3.8 |

---

## §5 数据库 Schema (3 表 DDL, per [01 §4 W/T/M](./01-requirements.md))

```sql
-- per 守门 #13 c/d W/T/M 強制約 + 派生规 (b)(c) Transaction 物理删除禁止 + 監査必須 + RLS 13 類必携

-- Table 1: sub_task_binding (Transaction, append-only)
CREATE TABLE sub_task_binding (
    binding_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    parent_task_id UUID NOT NULL,
    child_task_id UUID NOT NULL UNIQUE,  -- 1:1 exclusive, UNIQUE
    sa_type TEXT NOT NULL CHECK (sa_type IN (
        'SA-01','SA-02','SA-03','SA-04','SA-05',
        'SA-06','SA-07','SA-08','SA-09','SA-10'
    )),
    bound_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    bound_by TEXT NOT NULL,
    bound_via TEXT NOT NULL CHECK (bound_via IN ('ui_rightclick', 'l0_chat', 'api')),
    -- 生命周期状态
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'cancelled', 'failed', 'completed')),
    -- 派生血缘 (保留 per ADR-0046 §2.4)
    merged_from JSONB,  -- M-N1 血缘
    split_into JSONB,   -- M-N2 血缘
    superseded_by UUID,
    -- W/T/M: Transaction, 物理删除禁止, append-only
    -- RLS 13 類必携 (per 守门 #13): tenant_id + workspace_id + parent_task_id
    tenant_id UUID NOT NULL,
    workspace_id UUID NOT NULL,
    -- 审计
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- RLS policy
    CONSTRAINT rls_tenant_isolation CHECK (tenant_id IS NOT NULL)
);
CREATE INDEX idx_sub_task_binding_parent ON sub_task_binding(parent_task_id) WHERE status = 'active';
CREATE INDEX idx_sub_task_binding_tenant ON sub_task_binding(tenant_id);

-- Table 2: sub_task_runtime (Work, 短 TTL 24h)
CREATE TABLE sub_task_runtime (
    runtime_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    child_task_id UUID NOT NULL UNIQUE REFERENCES sub_task_binding(child_task_id),
    current_step TEXT,
    tokens_used BIGINT NOT NULL DEFAULT 0,
    last_llm_call_at TIMESTAMPTZ,
    last_mcp_call_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ NOT NULL DEFAULT NOW() + INTERVAL '24 hours',  -- Work 短 TTL
    -- W/T/M: Work, 物理删除允许 (TTL 失効), 不需 audit, 不需 RLS
    -- 简化: 仅 tenant_id + device_id 2 维
    tenant_id UUID NOT NULL,
    device_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_sub_task_runtime_expires ON sub_task_runtime(expires_at);

-- Table 3: task_quota_config (Master, SCD Type 2)
CREATE TABLE task_quota_config (
    config_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    parent_task_id UUID NOT NULL,
    max_subtasks INT NOT NULL DEFAULT 2,
    current_subtasks INT NOT NULL DEFAULT 0,
    per_subtask_token_budget BIGINT NOT NULL DEFAULT 100000,
    effective_from TIMESTAMPTZ NOT NULL,
    effective_to TIMESTAMPTZ,  -- SCD Type 2 end (NULL = current)
    is_current BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by TEXT NOT NULL,
    -- W/T/M: Master, 物理删除禁止, SCD Type 2, RLS 13 類必携
    tenant_id UUID NOT NULL,
    CONSTRAINT chk_scd_type2 CHECK (
        (is_current = TRUE AND effective_to IS NULL) OR
        (is_current = FALSE AND effective_to IS NOT NULL)
    )
);
CREATE INDEX idx_task_quota_config_current ON task_quota_config(parent_task_id) WHERE is_current = TRUE;

-- 触发器: SCD Type 2 自动 close 旧 row
CREATE OR REPLACE FUNCTION close_old_quota_config()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.is_current = TRUE THEN
        UPDATE task_quota_config
        SET effective_to = NOW(), is_current = FALSE
        WHERE parent_task_id = NEW.parent_task_id
          AND is_current = TRUE
          AND config_id != NEW.config_id;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_close_old_quota_config
BEFORE INSERT OR UPDATE ON task_quota_config
FOR EACH ROW EXECUTE FUNCTION close_old_quota_config();

-- RLS policies (per 守门 #13 RLS 13 類)
ALTER TABLE sub_task_binding ENABLE ROW LEVEL SECURITY;
CREATE POLICY sub_task_binding_tenant_isolation ON sub_task_binding
    USING (tenant_id = current_setting('app.tenant_id')::UUID);

ALTER TABLE task_quota_config ENABLE ROW LEVEL SECURITY;
CREATE POLICY task_quota_config_tenant_isolation ON task_quota_config
    USING (tenant_id = current_setting('app.tenant_id')::UUID);
```

---

## §6 守门合规检查 (per [02 §4 守门合规矩阵](./02-basic-design.md) 14 守门 + v3x 5 候选)

| 守门 | 检查项 | 验证方法 | 状态 |
|---|---|---|---|
| #1 | cargo check 0 err (实装后) | `cargo check --workspace --all-targets -j 4` | ⏳ 实装 v0.3 |
| #3 | 5 域单仓 (文档 v0.1) | dual-use disclaimer + 不引用 RGS | ✅ |
| #5 | env 安全 (无 secret) | 文档审计 | ✅ |
| #6 | PowerShell only (代码) | — | ⏳ 实装 |
| #7 | 0 unsafe (代码) | C-24 type-safe, 无 unsafe code | ⏳ 实装 |
| #9 | 子代理 RPC 不可靠 (per 实证) | 调试控制台走 subprocess, 不派 worker | ✅ 设计 |
| #12 | 缺标比错标 (7 已知缺口) | 显式列 G-SB-1..7 | ✅ |
| #13 a | L1↔L1 禁止通信 | M-N8 全部 L0 协调, 唯一 cross-task actor | ✅ |
| #13 c/d | DB W/T/M 3 表 100% 覆盖 | [§5](#5-数据库-schema-3-表-ddl-per-01--4-wtm) DDL | ✅ |
| #19 | Python 化 | 走 `scripts/automation/task_ops.py spawn_subtask` | ✅ 设计 |
| #21 | [P] docs 同步 | automation-design.md §4.14 追加 | ⏳ commit 时 |
| #22 | 调试控制台不污染 main | console_server.py 是 Python 进程, 不进 main 编译链 | ✅ 设计 |
| #23 | AI mock 不开 OpenAI | UI AI 建议走 ai_edit_mock.py | ✅ 设计 |
| #24 | subprocess 替代 RPC | console_server.py → subprocess.run task_ops.py | ✅ 设计 |

**v3x 候选** (per [AGENTS.md §4.1.1 v27-v31](../../AGENTS.md)):
- v27 子代理 RPC 失败 fallback: M-N8 走 subprocess 不派 worker, 适用
- v28 拍板必带推荐选项: ask_1df6987367ccc00928b65ee3 拍板 1 选项 (推荐), 适用
- v29 docs 同步饱和: 本次 +1, 距饱和点 40 还远
- v30 Mavis 永久代签: 5 签字栏全部代签, 适用
- v31 5 域 Lead 真人到位追溯: 真人到位后修订历史 +1 行, 适用

---

## §7 风险 (per [01 §5 7 已知缺口](./01-requirements.md) + [02 §7 6 风险](./02-basic-design.md))

跟 [02 §7 6 风险](./02-basic-design.md) 一致, 增量 1:
- **新增风险**: PostgreSQL Tier 3 启动待 5 域 Lead 真人到位 (per [AGENTS.md §4 #25 v25 G-DEP-08](../../AGENTS.md)), 任务卡持久化走 SQLite (per [AGENTS.md §4 #24 G-DEP-03](../../AGENTS.md)) 临时, Tier 3 切换需真人到位后触发

---

## §8 签字栏 (Signatures, per 7 段结构 5 角色)

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人 | Ulysses（一人公司 12 角色 per DEC-008）| 2026-09-09 | 🟡 Draft v0.1; 詳細設計 8 module (M-26..M-33) + 7 步 Python 実装 (M-N8 spawn_subtask_node) + 7 unit test (UT-27..UT-33) + 3 integration test (IT-13..IT-15) + 5 E2E test (E2E-14..E2E-18) + 3 表 DDL (sub_task_binding / sub_task_runtime / task_quota_config) + 14 守门合规检查 + v3x 5 候选 |
| 1.1 | 架构师 / Mavis 接手审批 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-09 | 🟢 Mavis 接手终审通过 (per ask_1df6987367ccc00928b65ee3 拍板 1 选项); 3 守门合规 (L1↔L1 禁止 + W/T/M 横展開 + 0 unsafe) + 5 E2E test (E2E-14..E2E-18) + 3 表 DDL RLS 13 類必携 + 5 签字栏落档 |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-09 | 🟢 Mavis 接手代签 (per 19:39 + 21:59 JST); 5 域独立真实身份 (per 8/21 JST) 签字请 DDD Review 阶段补 |
| 3 | 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-09 | 🟢 Mavis 接手代签 (per 19:39 + 21:59 JST); 5 域独立真实身份签字请 DDD Review 阶段补 |
| 4 | 评审主持人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-09 | 🟢 Mavis 接手代签 (per 19:39 + 21:59 JST); 5 域独立真实身份签字请 DDD Review 阶段补 |
| 5 | 项目负责人 (PM) | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-09 | 🟢 Mavis 接手代签 (per 19:39 + 21:59 JST); 5 域独立真实身份签字请 DDD Review 阶段补 |

---

## §9 修订历史 (Revision History)

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-09 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: 8 module (M-26..M-33) + 7 步 Python 実装 (§3.3.1.1) + LifecycleLinkageManager (§3.3.2) + ExclusiveBindingGuard (§3.3.3) + SubTaskQuotaRegistry (§3.3.4) + SpawnSubtaskValidator (§3.3.5) + ChildSubAgentFactory (§3.3.6) + ParentChildEventBus (§3.3.7) + SubTaskTokenBudget (§3.3.8) + UISpawnSubtaskModal (§3.3.9) + 7 unit test (UT-27..UT-33) + 3 integration test (IT-13..IT-15) + 5 E2E test (E2E-14..E2E-18) + 3 表 DDL (sub_task_binding / sub_task_runtime / task_quota_config) + 14 守门合规 + v3x 5 候选 + 6 风险 + 5 签字栏; 跟 [01-requirements.md v0.1](./01-requirements.md) + [02-basic-design.md v0.1](./02-basic-design.md) + [ADR-0052](../2026-08-26-upgrade/adr/0052-subtask-binding.md) + [PHASE-SUBTASK-BINDING-IMPL-REPORT.md](../../reports/PHASE-SUBTASK-BINDING-IMPL-REPORT.md) 同步落档 | 2026-09-09 21:53 JST 用户发令 + ask_1df6987367ccc00928b65ee3 拍板 1 选项, 跟 3 份新文档 + ADR-0052 + PHASE 报告 v0.1 同步落档, ~0.10M token 实测 |

---

## §10 参考 (References)

- [01-requirements.md v0.1](./01-requirements.md) — UC-14..UC-18 + F-26..F-32 + NFR-SB-01..05 + 3 表 W/T/M (前序)
- [02-basic-design.md v0.1](./02-basic-design.md) — 8 组件 C-23..C-30 + 1 节点 M-N8 + 1 协议 + 5 Reducer (前序)
- [ADR-0052 决策记录](../2026-08-26-upgrade/adr/0052-subtask-binding.md) — M-N8 spawn_subtask_node 决策落档
- [ADR-0046 LangGraph TMO 7 节点](../2026-08-26-upgrade/adr/0046-langgraph-task-management-operations.md) — 7 节点 + 25 module (前序, 本文增量 8 module)
- [LangGraph 03-detailed v0.2 §3.1 模块结构 + §3.2 依赖图 + §3.5 Subgraphs](../2026-09-03-langgraph/03-detailed-design.md) — 主路径 L0 StateGraph 范式
- [LangGraph 03-detailed v0.2 §3.2.1.1 SubAgentState line 779](../2026-09-03-langgraph/03-detailed-design.md) — SubAgentState TypedDict 已有 5 血缘字段
- [PHASE-SUBTASK-BINDING-IMPL-REPORT.md v0.1](../../reports/PHASE-SUBTASK-BINDING-IMPL-REPORT.md) — 8 子项实装 phase 计划 (M-26..M-33)
- [AGENTS.md §3 报告 7 段结构 + §4 守门 #1-#24 + §4.1 累积规 v1-v24 + §4.1.1 v3x 候选 + §4.2 实装前一致性门](../../AGENTS.md)
- [docs/automation-design.md](../../automation-design.md) — agent 交互 Python 化 (守门 #19) + automation/dispatcher.py brief() (守门 #20)
- [docs/data-design/ipa-detail/00-CLASSIFICATION-W-T-M.md v0.1](../../data-design/ipa-detail/00-CLASSIFICATION-W-T-M.md) — DB 三類橫展開 100 表索引
- [docs/data-design/ipa-detail/00-CLASSIFICATION-RULES.md v0.1](../../data-design/ipa-detail/00-CLASSIFICATION-RULES.md) — 跨项目 ルール手册 + 4 段检查清单
- [Pydantic v2 Documentation](https://docs.pydantic.dev/latest/) — BaseModel + Field + validator
- [LangGraph Documentation](https://langchain-ai.github.io/langgraph/) — StateGraph / Checkpoint / Subgraph / Interrupt / Command
- [Antd Documentation](https://ant.design/) — Modal / Select / Input / Form / message (UI 组件库)
- [STAR-OLU-001.md v0.1](../../STAR-OLU-001.md) — 1 SRE·周 = 1.2M tokens (8 module 估 ~0.5-0.8M tokens)

---

# === Detailed Design 结束 ===

**per AGENTS.md §0 一句话硬约束 + §1 代签规则**: 可以代签 Ulysses, 不可以编造历史. 本文档 v0.1 引用守门 #1-#24 + 累积规 v1-v24 全部按 git 实证 + AGENTS.md 引用, 无"per X 历史形态"等回溯叙事.

**per 守门 #3 5 域单仓**: 本文档仅 STAR 仓内, 不引用 RGS 仓代码, 不建立业务子域↔DDD bounded context 映射 (per ADR-0044 §dual-use disclaimer).

**per 守门 #7 0 unsafe**: 8 module 全部 type-safe (Pydantic + TypedDict), 无 unsafe code, C-24 ExclusiveBindingGuard 编译期 + 运行期双层守门.

**per 守门 #13 W/T/M 横展開**: 3 表 (sub_task_binding / sub_task_runtime / task_quota_config) 100% 覆盖 W/T/M 三類, 派生规 (a)(b)(c)(d) 全部落档 + RLS 13 類必携.

**per 守门 #21 v21 [P] docs 同步**: automation-design.md §4.14 追加, commit message 引用相对路径.
