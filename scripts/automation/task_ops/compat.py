# @cypher schema=1 source_sha256=706f738172c0341302a88f4b83dab4d2cc0c202f8ad787fc2af3c92e2f298dda
# MERGE (self:File {path:"scripts/automation/task_ops/compat.py"})
# @endcypher
# scripts/automation/task_ops/compat.py
# 既有模块最小接口 stub (WBS 6.3 Sub-task Binding 实装)
#
# dev 分支不存在以下历史模块 (历史实现位于未合入孤儿 worktree cp-orchestrator,
# per TASK 前置注意) — spawn_subtask 8 模块 (M-26..M-33) 依赖它们, 此处建最小接口
# stub 供编译期 + 单测运行期使用, 生产接入时替换为真实实现 (接口保持 async 签名不变):
#
#   - sub_agent.state.SubAgentState        (M-06)  → class SubAgentState + 内存 store
#   - sub_agent.registry.SubAgentRegistry  (M-07)  → class SubAgentRegistry
#   - sub_agent.pool.SubAgentPool          (L1 pool) → class SubAgentPool
#   - audit.append_only.AuditLog           (Transaction append-only, 守门 #13 b)
#   - telemetry.token.TokenTelemetry       (守门 #4 token-OLU)
#   - db.connection.get_db + task_quota_config 表 (PostgreSQL Tier 3, 003 DDL)
#     → class QuotaConfigStore (内存版 SCD Type 2, 语义对齐 003_task_quota_config.sql)
#
# 注意: SubAgentState 是 dict 子类, 不得在其上定义 classmethod get/clear
# (会遮蔽 dict.get / dict.clear 实例方法) — store 操作走模块级函数.
#
# 约束 (per 守门 #19 v19): 标准库 only (无第三方依赖).
# 约束 (per 守门 #13 d): AuditLog append-only, 不提供 delete 之外的物理删除语义.
#
# 所有 stub 状态均为模块级内存单例, 测试通过 reset 函数隔离 (见各 section).

from __future__ import annotations

import uuid
from datetime import datetime, timezone
from typing import Any, Dict, List, Optional


# ===== M-06 sub_agent.state.SubAgentState stub =====
#
# 上游形态 (per docs/architecture/2026-09-03-langgraph/03-detailed-design.md §3.2.1.1
# + docs/architecture/2026-09-09-subtask-binding/02-basic-design.md §3.2.1.2):
#   TypedDict (total=False) — 已有 5 血缘字段 + v0.1 增量 4 binding 字段.
# 此处以 dict 子类承载 (支持 state.get("x") / state["x"] 双访问), store 由
# 模块级函数管理 (对应上游 SubAgentState.get(task_id) 的读取语义).

class SubAgentState(dict):
    """SubAgentState (M-06 stub, per 02-basic §3.2.1.2).

    字段 (total=False 语义):
      已有: task_id / task_name / task_type / status / parent_task_id /
            merged_from / split_into / superseded_by / checkpoint_snapshot /
            initial_context
      v0.1 增量 4 binding 字段 (per F-28):
            exclusive_owner_task_id (Optional[str], write-once per R-12)
            bound_at (Optional[datetime])
            bound_by (Optional[str])
            bound_via (Optional[str])
    """


_SUB_AGENT_STATES: Dict[str, SubAgentState] = {}


def get_sub_agent_state(task_id: str) -> Optional[SubAgentState]:
    """读取 task 状态 (None = 不存在), 对应上游 SubAgentState.get(task_id)."""
    return _SUB_AGENT_STATES.get(task_id)


def put_sub_agent_state(state: SubAgentState) -> None:
    """写入 task 状态 (spawn 入口调用)."""
    _SUB_AGENT_STATES[state["task_id"]] = state


def list_sub_agent_task_ids() -> List[str]:
    """列出全部已注册 task_id."""
    return list(_SUB_AGENT_STATES.keys())


def clear_sub_agent_states() -> None:
    """测试隔离用 reset (仅测试使用, 非生产语义)."""
    _SUB_AGENT_STATES.clear()


# ===== M-07 sub_agent.registry.SubAgentRegistry stub =====

class SubAgentRegistry:
    """SubAgentRegistry (M-07 stub): L1 子代理注册入口.

    spawn() 后 child status = running (per UC-14 post-condition), 重复 spawn
    同 task_id 抛 ValueError (1:1 独占, 见 M-28 ExclusiveBindingGuard).
    """

    @staticmethod
    async def spawn(task_id: str, sa_type: str, state: SubAgentState) -> None:
        """注册并启动 L1 子代理 (in-process subgraph stub)."""
        if get_sub_agent_state(task_id) is not None:
            raise ValueError(f"task {task_id} already spawned (1:1 exclusive)")
        state["status"] = "running"  # per UC-14 post-condition: child = running
        put_sub_agent_state(state)

    @staticmethod
    async def list_task_ids() -> List[str]:
        """列出全部已注册 task_id (测试/审计用)."""
        return list_sub_agent_task_ids()


# ===== sub_agent.pool.SubAgentPool stub (L1 pool) =====

class SubAgentPool:
    """SubAgentPool stub (L1 sub_pool): L0 唯一访问入口 (per 守门 #13 a)."""

    @staticmethod
    async def get_state(task_id: str) -> Optional[SubAgentState]:
        """读取 task 状态 (None = 不存在, per M-30 check_parent_state 处理)."""
        return get_sub_agent_state(task_id)

    @staticmethod
    async def cancel(task_id: str, reason: str) -> None:
        """cancel child (per UC-16: cancelling → cancelled)."""
        state = get_sub_agent_state(task_id)
        if state is None:
            raise KeyError(f"task {task_id} not found in sub_agent_pool")
        state["status"] = "cancelled"
        state["cancel_reason"] = reason

    @staticmethod
    async def pause(task_id: str, reason: str) -> None:
        """pause child (per UC-16 Alt flow C1)."""
        state = get_sub_agent_state(task_id)
        if state is None:
            raise KeyError(f"task {task_id} not found in sub_agent_pool")
        state["status"] = "paused"
        state["pause_reason"] = reason

    @staticmethod
    async def resume(task_id: str, reason: str) -> None:
        """resume child (per UC-16 联动表 parent resume → child resume)."""
        state = get_sub_agent_state(task_id)
        if state is None:
            raise KeyError(f"task {task_id} not found in sub_agent_pool")
        state["status"] = "running"
        state["resume_reason"] = reason


# ===== audit.append_only.AuditLog stub (守门 #13 b Transaction append-only) =====

class AuditLog:
    """AuditLog stub (audit.append_only): Transaction append-only.

    生产实现落 sub_task_binding 表 / PG audit (per 001 DDL + 守门 #13 b);
    此处内存 append, 只增不删 (物理删除禁止).
    """

    _events: List[Dict[str, Any]] = []

    @staticmethod
    async def append(event_type: str, **fields: Any) -> None:
        """追加审计事件 (append-only, 物理删除禁止)."""
        event: Dict[str, Any] = {
            "event_type": event_type,
            "event_id": uuid.uuid4().hex[:16],
            "recorded_at": datetime.now(timezone.utc).isoformat(),
        }
        event.update(fields)
        AuditLog._events.append(event)

    @staticmethod
    def get_events() -> List[Dict[str, Any]]:
        """读取全部审计事件 (只读)."""
        return list(AuditLog._events)

    @staticmethod
    def clear() -> None:
        """测试隔离用 reset (仅测试使用, 非生产语义)."""
        AuditLog._events.clear()


# ===== telemetry.token.TokenTelemetry stub (守门 #4 token-OLU) =====

class TokenTelemetry:
    """TokenTelemetry stub (telemetry.token): per-task token 累加计量."""

    _totals: Dict[str, int] = {}
    _ops: Dict[str, List[str]] = {}

    @staticmethod
    async def record(task_id: str, tokens_used: int, op: str) -> None:
        """累加 task token 使用 (per UC-18 step 1)."""
        TokenTelemetry._totals[task_id] = TokenTelemetry._totals.get(task_id, 0) + tokens_used
        TokenTelemetry._ops.setdefault(task_id, []).append(op)

    @staticmethod
    async def get_total(task_id: str) -> int:
        """查询 task 累计 token (0 = 无记录)."""
        return TokenTelemetry._totals.get(task_id, 0)

    @staticmethod
    def reset() -> None:
        """测试隔离用 reset (仅测试使用, 非生产语义)."""
        TokenTelemetry._totals.clear()
        TokenTelemetry._ops.clear()


# ===== db.connection.get_db + task_quota_config 表 stub (Tier 3 PG) =====
#
# 生产实现 = 003_task_quota_config.sql (Master, SCD Type 2, RLS 13 类必携,
# close_old_quota_config 触发器). 此处内存版对齐同语义:
#   - 每 parent_task_id 仅 1 行 is_current=TRUE
#   - 变更 = close 旧行 (effective_to=now, is_current=False) + insert 新行

class QuotaConfigStore:
    """task_quota_config 内存版 (db.connection stub, 语义对齐 003 DDL)."""

    _current: Dict[str, Dict[str, Any]] = {}
    _history: List[Dict[str, Any]] = []  # closed rows (SCD Type 2 留痕)

    @classmethod
    def current(cls, parent_task_id: str) -> Optional[Dict[str, Any]]:
        """读取当前有效配置行 (None = 无配置 → 用默认值)."""
        return cls._current.get(parent_task_id)

    @classmethod
    def upsert_scd2(
        cls,
        parent_task_id: str,
        *,
        max_subtasks: Optional[int] = None,
        current_subtasks: Optional[int] = None,
        per_subtask_token_budget: Optional[int] = None,
        created_by: str = "Mavis",
    ) -> Dict[str, Any]:
        """SCD Type 2 变更: close 旧行 + insert 新行 (语义对齐 003 触发器)."""
        old = cls._current.get(parent_task_id)
        if old is not None:
            old["effective_to"] = datetime.now(timezone.utc)
            old["is_current"] = False
            cls._history.append(old)
        new_row: Dict[str, Any] = {
            "config_id": f"qc-{uuid.uuid4().hex[:16]}",
            "parent_task_id": parent_task_id,
            "max_subtasks": max_subtasks if max_subtasks is not None else (
                old["max_subtasks"] if old else 2
            ),
            "current_subtasks": current_subtasks if current_subtasks is not None else (
                old["current_subtasks"] if old else 0
            ),
            "per_subtask_token_budget": (
                per_subtask_token_budget
                if per_subtask_token_budget is not None
                else (old["per_subtask_token_budget"] if old else 100_000)
            ),
            "effective_from": datetime.now(timezone.utc),
            "effective_to": None,
            "is_current": True,
            "created_at": datetime.now(timezone.utc),
            "created_by": created_by,
        }
        cls._current[parent_task_id] = new_row
        return new_row

    @classmethod
    def history(cls, parent_task_id: Optional[str] = None) -> List[Dict[str, Any]]:
        """读取已关闭历史行 (SCD Type 2 留痕, 只读)."""
        if parent_task_id is None:
            return list(cls._history)
        return [row for row in cls._history if row["parent_task_id"] == parent_task_id]

    @classmethod
    def reset(cls) -> None:
        """测试隔离用 reset (仅测试使用, 非生产语义)."""
        cls._current.clear()
        cls._history.clear()
