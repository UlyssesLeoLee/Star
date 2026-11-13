# @cypher schema=1 source_sha256=dad6b5678aa9016180d297dfe726874e111c1a51672b1a3eb071c3fc65243ea1
# MERGE (self:File {path:"scripts/automation/task_ops/spawn_subtask/models.py"})
# @endcypher
# scripts/automation/task_ops/spawn_subtask/models.py
# spawn_subtask 入参/出参 model (per 02-basic-design.md §2.7.4 协议 + 03-detailed §3.3.1.1)
#
# 上游为 Pydantic v2 BaseModel + Field/validator; 本仓守门 #19 标准库 only
# (无第三方依赖) — 此处以 dataclass + __post_init__ 手工校验等价实现,
# 校验失败抛 ValueError (对应 Pydantic ValidationError 语义).
#
# 字段约束 (per 01-requirements §2 F-27):
#   - parent_task_id 必合 ^[a-z0-9-]{8,64}$
#   - child_sa_type ∈ SA-01..SA-10; SA-10 自嵌套拒绝 (per UC-14 Alt A2 + UT-33)
#   - child_name 非空, ≤ 200 chars
#   - bound_by ∈ {Ulysses, Mavis, *-subagent} (per 守门 #10 代签)
#   - spawned_via ∈ {ui_rightclick, l0_chat, api} (per 02-basic §2.7.4 入口标注)

from __future__ import annotations

import re
from dataclasses import dataclass, field
from datetime import datetime, timezone
from typing import Any, Dict, Optional, TypedDict

SAType = (
    "SA-01", "SA-02", "SA-03", "SA-04", "SA-05",
    "SA-06", "SA-07", "SA-08", "SA-09", "SA-10",
)

SA_TYPE_VALUES = frozenset(SAType)

SPAWN_VIA_VALUES = frozenset(("ui_rightclick", "l0_chat", "api"))

_PARENT_ID_RE = re.compile(r"^[a-z0-9-]{8,64}$")
_BOUND_BY_RE = re.compile(r"^(Ulysses|Mavis|.*-subagent)$")

SA10_SELF_SPAWN_MESSAGE = (
    "SA-10 task-orchestrator cannot spawn sub-task "
    "(per F-30 ExclusiveBindingGuard + NFR-SB-02 self-spawn prohibited)"
)


@dataclass(frozen=True)
class SpawnSubtaskRequest:
    """L0 → L1 spawn_subtask 请求 (per 02-basic §2.7.4)."""

    parent_task_id: str
    child_sa_type: str
    child_name: str
    initial_context: Dict[str, Any] = field(default_factory=dict)
    quota_check: bool = True
    bound_by: str = "Ulysses"
    spawned_via: str = "api"

    def __post_init__(self) -> None:
        # parent_task_id 格式 (per F-27 字段约束)
        if not _PARENT_ID_RE.match(self.parent_task_id):
            raise ValueError(
                f"parent_task_id must match ^[a-z0-9-]{{8,64}}$: {self.parent_task_id!r}"
            )
        # child_sa_type 合法性 (per F-27 + F-30)
        if self.child_sa_type not in SA_TYPE_VALUES:
            raise ValueError(
                f"Invalid SA type: {self.child_sa_type} (must be one of {SAType})"
            )
        # SA-10 自嵌套拒绝 (per UC-14 Alt A2 + UT-33)
        if self.child_sa_type == "SA-10":
            raise ValueError(SA10_SELF_SPAWN_MESSAGE)
        # child_name 非空 ≤ 200 chars (per F-27)
        if not 1 <= len(self.child_name) <= 200:
            raise ValueError(f"child_name must be 1..200 chars, got {len(self.child_name)}")
        # bound_by 合法性 (per 守门 #10 代签)
        if not _BOUND_BY_RE.match(self.bound_by):
            raise ValueError(
                f"bound_by must match ^(Ulysses|Mavis|.*-subagent)$: {self.bound_by!r}"
            )
        # spawned_via 入口标注 (per 02-basic §2.7.4)
        if self.spawned_via not in SPAWN_VIA_VALUES:
            raise ValueError(
                f"spawned_via must be one of {sorted(SPAWN_VIA_VALUES)}: {self.spawned_via!r}"
            )


@dataclass(frozen=True)
class SpawnSubtaskResult:
    """L1 → L0 spawn_subtask 响应 (per 02-basic §2.7.4)."""

    child_task_id: str
    parent_task_id: str
    sa_type: str
    status: str  # "spawned" | "failed" (守门 #12: 失败必填 failure_reason)
    failure_reason: Optional[str] = None
    spawned_at: datetime = field(default_factory=lambda: datetime.now(timezone.utc))
    exclusive_owner_task_id: str = ""  # = parent_task_id (per F-28 1:1 独占)
    quota_remaining: int = -1

    def to_dict(self) -> Dict[str, Any]:
        """JSON 序列化 (subprocess 出口 / API 响应用)."""
        return {
            "child_task_id": self.child_task_id,
            "parent_task_id": self.parent_task_id,
            "sa_type": self.sa_type,
            "status": self.status,
            "failure_reason": self.failure_reason,
            "spawned_at": self.spawned_at.isoformat(),
            "exclusive_owner_task_id": self.exclusive_owner_task_id,
            "quota_remaining": self.quota_remaining,
        }


class TopAgentState(TypedDict, total=False):
    """TopAgentState 增量字段 (per 02-basic §3.2.1.2)."""

    active_tmo_operation: Optional[Dict[str, Any]]
    last_spawn_subtask_result: Optional[SpawnSubtaskResult]
