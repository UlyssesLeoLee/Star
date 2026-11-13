#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""agent_runtime — STAR Agent Runtime 双模式 Runtime 的 Python 侧实装
(per docs/architecture/2026-09-03-agent-runtime/02-basic-design.md + 03-detailed-design.md)。

WBS 6.2 目标: LangGraph Python (任务图模式) + Rust ECS (crates/star-agent-runtime)
双模式 Runtime。本包落地 Python 侧 — L0 派发层 (SQLite WAL TaskQueue / Scheduler /
Backpressure / RateLimiter / Retry / ProcessPool / Observer) + LangGraph 语义任务图
(taskgraph, 9 SA Type 节点 + reducer) + Runtime 双模式管理 (Lightweight < 10 /
ECS ≥ 12 + 迟滞区 10-11, per SRS-STAR-AGENT-RUNTIME-001 §7)。

架构原则 (per SRS §3 + §8):
    - Agent = Identity + State + References + Policies (绝不持有 Full Context /
      HTTP Client / LLM Client / MCP Client / OS Thread, per SRS §8 不得存放)
    - Agent ≠ Process / Thread / Runtime (per SRS §110)
    - < 10 Resident Agent 禁止启用完整 ECS Mode (per SRS §3.2 硬约束);
      本包 ECS 模式仅切换运行时标志与指标, L1 ECS 实体执行在 Rust 侧
      (crates/star-agent-runtime), 边界见 runtime.py
    - 所有队列/并发有界 + Fail Safe 默认 (per SRS §44 + §101)

约束 (per 守门 #1 v1 + 守门 #6 + 守门 #24):
    - 标准库 only: dataclasses / enum / sqlite3 / subprocess / threading / heapq
    - 跨平台: subprocess list-mode, shell=False (per 守门 #6 PowerShell only)
    - 复用现役模块 (AGENTS.md §0): automation.dispatcher.SubagentDispatcher
      (守门 #24 subprocess 路径) 在 process_pool 中以软依赖接入

已知缺口 (per 守门 #12 缺标比错标):
    1. LangGraph 真实包未安装 → taskgraph.py 以标准库实现 LangGraph 语义
       (节点/边/状态 reducer/拓扑执行), 真实 langgraph 依赖接入待 P3-B 续
    2. 真实 LLM/Tool/MCP Pool 待 P3-C (per SRS G-4), 本包仅骨架接口
    3. 跨机分布式 ❌ N/A (per 守门 #3 5 域单仓)
"""

from __future__ import annotations

__version__ = "0.1.0"

from .config import ConfigError, RuntimeConfig
from .events import EventBus, Mailbox
from .models import (
    Agent,
    AgentEvent,
    AgentIdentity,
    AgentState,
    AgentType,
    ContextRef,
    ContextTier,
    EventType,
    LifecycleState,
    MailboxMessage,
    MailboxRef,
    MemoryRef,
    MemoryType,
    ModelRef,
    PayloadRef,
    PermissionRef,
    Priority,
    Task,
    TokenBudget,
    TokenUsage,
    ToolPolicyRef,
    McpPolicyRef,
    WorkflowRef,
)
from .runtime import AgentRuntime
from .runtime_mode import RuntimeMode, RuntimeModeManager
from .scheduler import Scheduler, ScheduleResult
from .state_machine import AgentStateMachine, LifecycleMachine

__all__ = [
    "Agent",
    "AgentEvent",
    "AgentIdentity",
    "AgentRuntime",
    "AgentState",
    "AgentStateMachine",
    "AgentType",
    "ConfigError",
    "ContextRef",
    "ContextTier",
    "EventBus",
    "EventType",
    "LifecycleMachine",
    "LifecycleState",
    "Mailbox",
    "MailboxMessage",
    "MailboxRef",
    "McpPolicyRef",
    "MemoryRef",
    "MemoryType",
    "ModelRef",
    "PayloadRef",
    "PermissionRef",
    "Priority",
    "RuntimeConfig",
    "RuntimeMode",
    "RuntimeModeManager",
    "ScheduleResult",
    "Scheduler",
    "Task",
    "TokenBudget",
    "TokenUsage",
    "ToolPolicyRef",
    "WorkflowRef",
    "__version__",
]
