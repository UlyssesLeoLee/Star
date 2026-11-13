#!/usr/bin/env python3
# -*- coding: utf-8 -*-
# @cypher schema=1 source_sha256=d2645692df519b248b0b1e17fb252e3cbd2ddb500e8b356b2eab4681888285d9
# MERGE (self:File {path:"scripts/automation/agent_runtime/state_machine.py"})
# @endcypher
"""agent_runtime.state_machine — Agent 状态机 + Lifecycle 状态机
(per 03-DD §3.1 / §3.2 + SRS §48 REQ-STATE-001 / §24 REQ-LC-TRANS)。

Agent 状态机 (per 03-DD §3.1):
    Idle → Ready → Scheduled → Planning → WaitingLlm/Tool/Event → Processing
    → Completed → Idle; 异常: Failed → Ready, Suspended → Scheduled,
    Cancelled 终态。

Lifecycle 状态机 (per 03-DD §3.2):
    Event → HOT; HOT → WARM (60s 无活动); WARM → COLD (600s idle);
    COLD → (Event/Restore) → WARM → (Schedule) → HOT。超时可配置 (per SRS §7.4)。

约束: 标准库 only; 时间注入 (now 参数) 保证测试确定性。
"""

from __future__ import annotations

import time
from typing import Optional, Set

from .models import Agent, AgentState, LifecycleState

# ============================================================================
# §1 Agent 状态机 (per 03-DD §3.1 转换图)
# ============================================================================

#: 合法转换表 (per 03-DD §3.1)。Waiting* 三个等待态合并为 wait 组。
AGENT_TRANSITIONS: dict = {
    AgentState.Idle: {AgentState.Ready},
    AgentState.Ready: {AgentState.Scheduled},
    AgentState.Scheduled: {AgentState.Planning},
    AgentState.Planning: {
        AgentState.WaitingLlm,
        AgentState.WaitingTool,
        AgentState.WaitingEvent,
    },
    AgentState.WaitingLlm: {AgentState.Processing},
    AgentState.WaitingTool: {AgentState.Processing},
    AgentState.WaitingEvent: {AgentState.Processing},
    AgentState.Processing: {AgentState.Completed},
    AgentState.Completed: {AgentState.Idle},
    AgentState.Failed: {AgentState.Ready},
    AgentState.Suspended: {AgentState.Scheduled},
    AgentState.Cancelled: frozenset(),  # 终态
}

#: 任何非终态都可进入的异常态 (per 03-DD §3.1 异常: Failed / Suspended / Cancelled)
TERMINAL_STATES = frozenset({AgentState.Cancelled})

CANCEL_FROM: Set[AgentState] = {
    AgentState.Idle,
    AgentState.Ready,
    AgentState.Scheduled,
    AgentState.Planning,
    AgentState.WaitingLlm,
    AgentState.WaitingTool,
    AgentState.WaitingEvent,
    AgentState.Processing,
    AgentState.Failed,
    AgentState.Suspended,
}
SUSPEND_FROM: Set[AgentState] = CANCEL_FROM - {AgentState.Suspended}
FAIL_FROM: Set[AgentState] = CANCEL_FROM - {AgentState.Failed}


class AgentStateMachine:
    """Agent 12 状态转换器 (per 03-DD §3.1)。"""

    @staticmethod
    def can_transition(current: AgentState, target: AgentState) -> bool:
        """target 是否为 current 的合法后继。"""
        if target in (AgentState.Failed, AgentState.Suspended, AgentState.Cancelled):
            if target is AgentState.Failed:
                return current in FAIL_FROM
            if target is AgentState.Suspended:
                return current in SUSPEND_FROM
            if current in CANCEL_FROM:
                return True
            return False
        return target in AGENT_TRANSITIONS.get(current, frozenset())

    @staticmethod
    def transition(agent: Agent, target: AgentState, now: Optional[float] = None) -> bool:
        """转换 agent.state; 非法转换返回 False 不改状态 (per SRS §48 异常态)。"""
        if not AgentStateMachine.can_transition(agent.state.current, target):
            return False
        agent.state.prev = agent.state.current
        agent.state.current = target
        agent.state.since = time.monotonic() if now is None else now
        return True

    @staticmethod
    def cancel(agent: Agent, now: Optional[float] = None) -> bool:
        """取消 (per SRS §50 REQ-CANCEL-001), Cancelled 为终态。"""
        return AgentStateMachine.transition(agent, AgentState.Cancelled, now)

    @staticmethod
    def suspend(agent: Agent, now: Optional[float] = None) -> bool:
        """挂起 (per SRS §50), resume 回 Scheduled。"""
        return AgentStateMachine.transition(agent, AgentState.Suspended, now)

    @staticmethod
    def resume(agent: Agent, now: Optional[float] = None) -> bool:
        return AgentStateMachine.transition(agent, AgentState.Scheduled, now)

    @staticmethod
    def fail(agent: Agent, now: Optional[float] = None) -> bool:
        agent.state.retry_count += 1
        return AgentStateMachine.transition(agent, AgentState.Failed, now)

    @staticmethod
    def retry(agent: Agent, now: Optional[float] = None) -> bool:
        """Failed → Ready (per 03-DD §3.1 retry)。"""
        return AgentStateMachine.transition(agent, AgentState.Ready, now)


# ============================================================================
# §2 Lifecycle 状态机 (per 03-DD §3.2 + SRS §24)
# ============================================================================


class LifecycleMachine:
    """HOT/WARM/COLD 转换器 (per 03-DD §3.2)。

    不得由 Agent 自己决定长期生命周期 (per SRS §25 REQ-LC-MGR)。
    """

    def __init__(self, hot_timeout: float = 60.0, cold_idle_timeout: float = 600.0):
        # 配置化 (per SRS §7.4): 不硬编码进业务逻辑
        self.hot_timeout = hot_timeout
        self.cold_idle_timeout = cold_idle_timeout

    def activate(self, agent: Agent, now: Optional[float] = None) -> None:
        """Event / Schedule → HOT (per 03-DD §3.2)。COLD 先恢复 (restore)。"""
        agent.lifecycle.current = LifecycleState.Hot
        agent.lifecycle.last_active = time.monotonic() if now is None else now

    def touch(self, agent: Agent, now: Optional[float] = None) -> None:
        """记录活动时间 (HOT 保持)。"""
        agent.lifecycle.last_active = time.monotonic() if now is None else now

    def sweep(self, agent: Agent, now: Optional[float] = None) -> Optional[LifecycleState]:
        """周期扫描: HOT → WARM → COLD (per 03-DD §3.2 转换规则)。

        Returns: 本次扫出的新状态 (无变化返回 None)。
        """
        t = time.monotonic() if now is None else now
        idle = t - agent.lifecycle.last_active
        current = agent.lifecycle.current
        if current is LifecycleState.Hot and idle > self.hot_timeout:
            agent.lifecycle.current = LifecycleState.Warm
            return LifecycleState.Warm
        if current is LifecycleState.Warm and idle > self.cold_idle_timeout:
            agent.lifecycle.current = LifecycleState.Cold
            return LifecycleState.Cold
        return None

    def restore(self, agent: Agent, now: Optional[float] = None) -> None:
        """COLD → WARM (per SRS §23: 收到事件 Load → WARM → Schedule → HOT)。"""
        if agent.lifecycle.current is LifecycleState.Cold:
            agent.lifecycle.current = LifecycleState.Warm
            agent.lifecycle.last_active = time.monotonic() if now is None else now
