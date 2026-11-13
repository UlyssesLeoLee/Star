#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""agent_runtime.models — 核心数据模型 (per SRS §8-§20 + 03-DD §2.1 关键类型)。

Agent = Identity + State + References + Policies (per SRS §8):
    Agent 是逻辑实体, 不是 Process / Thread / Runtime (per SRS §110)。

不得存放 (per SRS §8 不得存放):
    Full Context / Full Memory / HTTP Client / LLM Client / MCP Client /
    Retriever / Tool Registry / OS Thread — 本模块结构上只放引用 (Ref)。

约束: 标准库 only。
"""

from __future__ import annotations

import time
import uuid
from dataclasses import dataclass, field
from enum import Enum, IntEnum
from typing import Optional


def new_id() -> str:
    """UUID4 字符串 (per 03-DD §2.1 类型别名, 标准库 uuid)。"""
    return str(uuid.uuid4())


# ============================================================================
# §1 AgentType (9 SA Type, 引用 LangGraph 9/3 02 §3-§4, 不重写)
# ============================================================================

SA_TYPES = {
    "SA-01": "code-review",
    "SA-02": "test-gen",
    "SA-03": "5-域-lead-audit",
    "SA-04": "git-ops",
    "SA-05": "doc-sync",
    "SA-06": "refactor",
    "SA-07": "db-migration",
    "SA-08": "domain-dev",
    "SA-09": "free-form",
}


class AgentType:
    """SA-01..SA-09 类型标识 (引用 LangGraph 9/3 §6.1, 不重写定义)。

    合法值: SA-01..SA-09 或 SA-N (新类型占位, per LangGraph 9/3 02 §3 图示)。
    """

    def __init__(self, value: str):
        v = value.strip()
        if not (v in SA_TYPES or v == "SA-N"):
            raise ValueError(f"非法 AgentType: {value!r} (允许 {sorted(SA_TYPES)} + SA-N)")
        self.value = v

    def label(self) -> str:
        return SA_TYPES.get(self.value, "new-agent-type")

    def __str__(self) -> str:
        return self.value

    def __repr__(self) -> str:
        return f"AgentType({self.value!r})"

    def __eq__(self, other) -> bool:
        return isinstance(other, AgentType) and other.value == self.value

    def __hash__(self) -> int:
        return hash(self.value)


# ============================================================================
# §2 状态枚举 (per SRS §10 / §11 / §18, 03-DD §2.1)
# ============================================================================


class AgentState(Enum):
    """Agent 12 状态 (per SRS §10 REQ-ECS-002)。"""

    Idle = "Idle"
    Ready = "Ready"
    Scheduled = "Scheduled"
    Planning = "Planning"
    WaitingLlm = "WaitingLlm"
    WaitingTool = "WaitingTool"
    WaitingEvent = "WaitingEvent"
    Processing = "Processing"
    Completed = "Completed"
    Failed = "Failed"
    Suspended = "Suspended"
    Cancelled = "Cancelled"


class LifecycleState(Enum):
    """HOT / WARM / COLD (per SRS §11 REQ-ECS-003)。"""

    Hot = "Hot"
    Warm = "Warm"
    Cold = "Cold"


class Priority(IntEnum):
    """优先级 (per SRS §18 REQ-ECS-010), 数值越小越优先 (per 03-DD §2.1)。"""

    Critical = 0
    High = 1
    Normal = 2
    Low = 3
    Background = 4


class EventType(Enum):
    """9 类事件源 (per SRS §26 REQ-EVT-001)。"""

    UserMessage = "UserMessage"
    AgentMessage = "AgentMessage"
    ToolResult = "ToolResult"
    LlmResult = "LlmResult"
    McpEvent = "McpEvent"
    Timer = "Timer"
    WorkflowEvent = "WorkflowEvent"
    SystemEvent = "SystemEvent"
    ExternalEvent = "ExternalEvent"


class ContextTier(Enum):
    """Context 分层 (per SRS §37 REQ-CTX-002)。"""

    L1Hot = "L1Hot"        # RAM, MB 级
    L2Recent = "L2Recent"  # Redis / Local Cache, GB 级
    L3Full = "L3Full"      # DB / Object Store, TB 级


class MemoryType(Enum):
    """长期记忆类型 (per SRS §41 REQ-MEM-001)。"""

    Semantic = "Semantic"
    Episodic = "Episodic"
    User = "User"
    Workflow = "Workflow"
    Knowledge = "Knowledge"


# ============================================================================
# §3 引用组件 (per SRS §9-§20: Agent 只存 Ref, 不存实体)
# ============================================================================


@dataclass(frozen=True)
class AgentIdentity:
    """REQ-ECS-001: 唯一 ID + tenant + agent_type。"""

    agent_id: str
    tenant_id: str
    agent_type: AgentType

    @classmethod
    def create(cls, tenant_id: str, agent_type: str) -> "AgentIdentity":
        return cls(agent_id=new_id(), tenant_id=tenant_id, agent_type=AgentType(agent_type))


@dataclass
class AgentStateInfo:
    """AgentState Component (per 03-DD §2.1): current/prev/since/retry_count。"""

    current: AgentState = AgentState.Idle
    prev: AgentState = AgentState.Idle
    since: float = field(default_factory=time.monotonic)
    retry_count: int = 0


@dataclass
class LifecycleInfo:
    """LifecycleState Component (per 03-DD §2.1): current/last_active。"""

    current: LifecycleState = LifecycleState.Cold
    last_active: float = field(default_factory=time.monotonic)


@dataclass(frozen=True)
class ContextRef:
    """REQ-ECS-004: Context 引用, 不含 Full Context (per SRS §8)。"""

    context_id: str
    tier: ContextTier = ContextTier.L3Full
    loaded: bool = False
    refs: tuple = ()  # 共享 Context + Agent Delta (per SRS §40)


@dataclass(frozen=True)
class MemoryRef:
    """REQ-ECS-005: 长期记忆外置, Agent → MemoryRef → Memory Store。"""

    memory_id: str
    memory_type: MemoryType = MemoryType.Semantic


@dataclass(frozen=True)
class ModelRef:
    """REQ-ECS-006: 模型配置, 实际连接走 Shared LLM Pool (per SRS §18)。"""

    provider: str = "mock"
    model: str = "mock"
    profile: str = "default"
    temperature: float = 0.0
    max_tokens: int = 4096


@dataclass(frozen=True)
class ToolPolicyRef:
    """REQ-ECS-007: 不持有 Tool 实例, 权限由 Tool System 解析。"""

    policy_id: str
    tool_allowlist: tuple = ()


@dataclass(frozen=True)
class McpPolicyRef:
    """REQ-ECS (03-DD §2.1): MCP 权限引用。"""

    policy_id: str
    server_allowlist: tuple = ()


@dataclass(frozen=True)
class PermissionRef:
    """REQ-ECS-008: ACL 引用, 不复制完整 ACL。"""

    acl_id: str
    tenant_id: str


@dataclass
class TokenBudget:
    """REQ-ECS-009: token 配额 (per SRS §17)。"""

    max_context_tokens: int = 8000
    max_output_tokens: int = 2048
    remaining_tokens: int = 8000
    cost_budget_cents: int = 0


@dataclass
class MailboxRef:
    """REQ-ECS-011: Mailbox 引用, 不存大消息 (per SRS §19)。"""

    mailbox_id: str
    unread_count: int = 0
    max_size: int = 256


@dataclass
class WorkflowRef:
    """REQ-ECS-012: 长生命周期 Workflow 状态不全嵌入 Agent。"""

    workflow_id: str
    step: int = 0
    total_steps: int = 0
    checkpoint_id: Optional[str] = None


@dataclass(frozen=True)
class PayloadRef:
    """REQ-PAYLOAD-001: 大消息不在 Agent 间 Clone, 走 Payload Store。"""

    store_id: str
    key: str
    size_bytes: int = 0
    ttl_seconds: float = 3600.0


# ============================================================================
# §4 Event / Mailbox (per SRS §26-§28 + 03-DD §4.2-§4.3)
# ============================================================================


@dataclass(frozen=True)
class AgentEvent:
    """REQ-EVT-002 标准事件 (per SRS §27)。"""

    event_id: str
    source_agent_id: Optional[str]
    target_agent_id: str
    tenant_id: str
    event_type: EventType
    payload_ref: Optional[PayloadRef] = None
    timestamp: float = field(default_factory=time.time)
    trace_id: str = field(default_factory=new_id)
    correlation_id: Optional[str] = None

    @classmethod
    def create(
        cls,
        source_agent_id: Optional[str],
        target_agent_id: str,
        tenant_id: str,
        event_type: EventType,
        **kwargs,
    ) -> "AgentEvent":
        return cls(
            event_id=new_id(),
            source_agent_id=source_agent_id,
            target_agent_id=target_agent_id,
            tenant_id=tenant_id,
            event_type=event_type,
            **kwargs,
        )


@dataclass(frozen=True)
class MailboxMessage:
    """Mailbox 消息 (per 03-DD §4.3): 大消息不内联, 只带 PayloadRef。"""

    msg_id: str
    event_id: str
    payload_ref: Optional[PayloadRef] = None
    timestamp: float = field(default_factory=time.time)
    read: bool = False


@dataclass(frozen=True)
class TokenUsage:
    """Token 计量 (per SRS §17 + 03-DD §4.5)。"""

    input_tokens: int = 0
    output_tokens: int = 0
    cost_cents: int = 0
    timestamp: float = field(default_factory=time.time)


# ============================================================================
# §5 Agent 复合体 (per SRS §8: Identity + State + References + Policies)
# ============================================================================


@dataclass
class Agent:
    """逻辑 Agent (per SRS §8)。

    只含 Identity / State / Lifecycle / Refs / Policies。禁止:
    Full Context / Memory / HTTP Client / LLM Client / MCP Client /
    Retriever / Tool Registry / OS Thread (per SRS §8 不得存放)。
    """

    identity: AgentIdentity
    state: AgentStateInfo = field(default_factory=AgentStateInfo)
    lifecycle: LifecycleInfo = field(default_factory=LifecycleInfo)
    priority: Priority = Priority.Normal
    context_ref: Optional[ContextRef] = None
    memory_ref: Optional[MemoryRef] = None
    model_ref: Optional[ModelRef] = None
    tool_policy_ref: Optional[ToolPolicyRef] = None
    mcp_policy_ref: Optional[McpPolicyRef] = None
    permission_ref: Optional[PermissionRef] = None
    token_budget: Optional[TokenBudget] = None
    mailbox_ref: Optional[MailboxRef] = None
    workflow_ref: Optional[WorkflowRef] = None

    @classmethod
    def create(cls, tenant_id: str, agent_type: str, **kwargs) -> "Agent":
        """创建 Agent (初始 WARM, per 03-DD §3.2: Event → HOT)。"""
        return cls(identity=AgentIdentity.create(tenant_id, agent_type), **kwargs)


# ============================================================================
# §6 L0 Task (per 03-DD §5.1 task_queue 列)
# ============================================================================


@dataclass(frozen=True)
class Task:
    """L0 派发任务 (per 03-DD §5.1 schema 的业务投影)。"""

    task_id: str
    brief: str
    agent_type: str
    tenant_id: str
    priority: Priority = Priority.Normal
    payload: Optional[str] = None
    trace_id: str = field(default_factory=new_id)
    correlation_id: Optional[str] = None
    created_at: float = field(default_factory=time.time)
    operation_id: str = field(default_factory=new_id)  # 幂等 key (per SRS §52)

    @classmethod
    def create(cls, brief: str, agent_type: str, tenant_id: str, **kwargs) -> "Task":
        return cls(
            task_id=new_id(),
            brief=brief,
            agent_type=agent_type,
            tenant_id=tenant_id,
            **kwargs,
        )
