#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""agent_runtime.runtime — AgentRuntime facade (per 02-BD §5.1 Runtime API)。

Runtime API (per SRS §85 REQ-API-001 + 02-BD §5.1):
    create_agent / delete_agent / get_agent / send_event / suspend_agent /
    resume_agent / cancel_agent / get_agent_state / get_runtime_metrics

dispatch 流程 (per 03-DD §4.1 UC-01):
    rate_limit.acquire → backpressure.await_slot → queue.enqueue
    → scheduler (HOT slot + fair) → pool.submit (守门 #24 subprocess)
    → observer.metrics

模式切换 (per SRS §83 一致性): 切换过程不丢 Event / 不重复 Tool / 不丢
Agent State (Python 侧 resident registry 全程在内存, 切换只动模式标志与
指标; L1 ECS 实体执行在 Rust 侧 crates/star-agent-runtime, 边界:
本 Runtime 只产出 L1 调度决策, 大世界执行由 Rust ECS 承接)。

约束: 标准库 only; 线程安全 (registry lock)。
"""

from __future__ import annotations

import threading
import time
from typing import Dict, Optional

from .backpressure import Backpressure, BackpressureDecision
from .config import ConfigError, RuntimeConfig
from .events import EventBus
from .metrics import Observer, RuntimeMetrics
from .models import (
    Agent,
    AgentEvent,
    AgentIdentity,
    AgentStateInfo,
    AgentType,
    LifecycleState,
    Task,
)
from .rate_limiter import TenantRateLimiter
from .runtime_mode import RuntimeMode, RuntimeModeManager
from .scheduler import Scheduler
from .state_machine import AgentStateMachine, LifecycleMachine
from .task_queue import TaskQueue


class AgentRuntime:
    """双模式 Runtime facade (Lightweight / ECS, per SRS §6)。"""

    def __init__(self, config: Optional[RuntimeConfig] = None, db_path: Optional[str] = None):
        self.config = (config or RuntimeConfig()).validate()
        self.mode_manager = RuntimeModeManager(self.config)
        self.queue = TaskQueue(
            db_path=db_path,
            retention_seconds=self.config.queue_retention_seconds,
            dlq_retention_seconds=self.config.dlq_retention_seconds,
        )
        self.scheduler = Scheduler(max_hot_agents=self.config.max_hot_agents)
        self.event_bus = EventBus(mailbox_max_size=self.config.mailbox_max_size)
        self.rate_limiter = TenantRateLimiter(
            capacity=self.config.rate_limit_capacity,
            refill_per_sec=self.config.rate_limit_refill_per_sec,
        )
        self.lifecycle = LifecycleMachine(
            hot_timeout=self.config.hot_to_warm_timeout,
            cold_idle_timeout=self.config.warm_to_cold_idle_timeout,
        )
        self.observer = Observer()
        self.backpressure = Backpressure(
            max_size=self.config.max_queue_size,
            overflow_policy=self.config.overflow_policy,
            throttle_retry_after=self.config.throttle_retry_after,
            delay_ms=self.config.delay_overflow_ms,
            size_provider=lambda: self.queue.size(),
            drop_low=lambda count: self.scheduler.drop_low_priority(count),
        )
        self._agents: Dict[str, Agent] = {}
        self._lock = threading.Lock()

    # === Runtime API (per SRS §85) ===

    def create_agent(self, tenant_id: str, agent_type: str, agent_id: Optional[str] = None,
                     **kwargs) -> Agent:
        """create_agent (per SRS §85): 注册 resident agent (WARM 起步)。

        agent_id: 显式指定 (测试/恢复场景); 缺省自动生成。
        同 id 重复注册 → ConfigError (per SRS §95 Single Active Ownership 雏形)。
        """
        if agent_id is not None:
            identity = AgentIdentity(agent_id=agent_id, tenant_id=tenant_id,
                                     agent_type=AgentType(agent_type))
        else:
            identity = AgentIdentity.create(tenant_id=tenant_id, agent_type=agent_type)
        agent = Agent(identity=identity, **kwargs)
        agent.lifecycle.current = LifecycleState.Warm
        with self._lock:
            if identity.agent_id in self._agents:
                raise ConfigError([f"agent 已存在: {identity.agent_id}"])
            self._agents[identity.agent_id] = agent
        self.mode_manager.set_resident_count(len(self._agents))
        self.event_bus.subscribe(identity.agent_id)
        return agent

    def delete_agent(self, agent_id: str) -> bool:
        """delete_agent (per SRS §85)。"""
        with self._lock:
            existed = self._agents.pop(agent_id, None) is not None
        if existed:
            self.mode_manager.set_resident_count(len(self._agents))
        return existed

    def get_agent(self, agent_id: str) -> Optional[Agent]:
        with self._lock:
            return self._agents.get(agent_id)

    def get_agent_state(self, agent_id: str) -> Optional[AgentStateInfo]:
        agent = self.get_agent(agent_id)
        return agent.state if agent else None

    def send_event(self, event: AgentEvent) -> str:
        """send_event (per SRS §85): Event → Bus → Mailbox → 激活 COLD agent。"""
        agent = self.get_agent(event.target_agent_id)
        if agent is not None and agent.lifecycle.current is LifecycleState.Cold:
            self.lifecycle.restore(agent)
        return self.event_bus.publish(event)

    def suspend_agent(self, agent_id: str) -> bool:
        agent = self.get_agent(agent_id)
        return agent is not None and AgentStateMachine.suspend(agent)

    def resume_agent(self, agent_id: str) -> bool:
        agent = self.get_agent(agent_id)
        return agent is not None and AgentStateMachine.resume(agent)

    def cancel_agent(self, agent_id: str) -> bool:
        """cancel_agent (per SRS §50): Cancellation 向下传播 — 同 id 任务一并取消。"""
        agent = self.get_agent(agent_id)
        if agent is None:
            return False
        self.queue.cancel(agent_id)
        return AgentStateMachine.cancel(agent)

    # === dispatch (per 03-DD §4.1 UC-01) ===

    def dispatch(self, task: Task, execute: bool = False) -> str:
        """L0 dispatch 流程 (per 03-DD §4.1 UC-01)。

        execute=True: 实际跑 subprocess executor (process_pool, 守门 #24);
        默认 False: 只落库 + 调度 (HOT slot), 执行由调用方 worker 承接。
        """
        # 1. Rate limit (per 03-DD §6.1 step 1)
        if not self.rate_limiter.acquire(task.tenant_id):
            raise RuntimeError(f"rate limit exceeded for tenant {task.tenant_id}")
        # 2. Backpressure (per 03-DD §6.1 step 2)
        decision: BackpressureDecision = self.backpressure.await_slot()
        if decision.action == "Delay":
            time.sleep(decision.delay_ms / 1000.0)
        # 3. Enqueue (per 03-DD §6.1 step 3)
        self.queue.enqueue(task)
        # 4. Dequeue + Schedule → HOT slot (per 03-DD §6.1 step 4-5 + §2.2 dequeue)
        dequeued = self.queue.dequeue(limit=self.config.max_hot_agents)
        for t in dequeued:
            self.scheduler.register_task(t)
        result = self.scheduler.schedule(ready_tasks=dequeued)
        # 未调度 (HOT slot 满 / fair share) → requeue 回 pending (per 03-DD §6.1)
        for deferred_id in self.scheduler.drain_deferred():
            self.queue.requeue(deferred_id)
        # 5. Execute (per UC-01 worker.execute, 守门 #24 subprocess 路径)
        if execute and result.scheduled:
            from .process_pool import ProcessPool

            pool = ProcessPool(size=self.config.process_pool_size)
            try:
                for task_id in result.scheduled:
                    pool.submit([task_id])  # 执行骨架: 真实 cmd 由业务注入
            finally:
                pool.shutdown()
        # 6. Observe (per UC-01 observer.metrics)
        self.observer.inc_dispatched()
        return task.task_id

    # === tick / lifecycle (per 03-DD §3.2) ===

    def tick(self, now: Optional[float] = None) -> RuntimeMode:
        """事件循环入口: 模式判定 + lifecycle sweep + 指标合并。"""
        t = time.monotonic() if now is None else now
        with self._lock:
            agents = list(self._agents.values())
        for agent in agents:
            self.lifecycle.sweep(agent, now=t)
        mode = self.mode_manager.decide(now=t, count=len(agents))
        self.mode_manager.set_resident_count(len(agents))
        return mode

    # === metrics (per SRS §61 + §85 get_runtime_metrics) ===

    def get_runtime_metrics(self) -> RuntimeMetrics:
        with self._lock:
            agents = list(self._agents.values())
        hot = sum(1 for a in agents if a.lifecycle.current is LifecycleState.Hot)
        warm = sum(1 for a in agents if a.lifecycle.current is LifecycleState.Warm)
        cold = sum(1 for a in agents if a.lifecycle.current is LifecycleState.Cold)
        m = RuntimeMetrics(
            logical_agent_count=len(agents),
            resident_agent_count=len(agents),
            hot_agent_count=hot,
            warm_agent_count=warm,
            cold_agent_count=cold,
            runtime_mode=self.mode_manager.get_mode().value,
            ecs_switch_count=self.mode_manager.switch_count,
            ready_queue_depth=self.scheduler.ready_queue_depth(),
            hot_slot_usage=self.scheduler.hot_slot_usage(),
            llm_queue_depth=self.queue.pending_count(),
            tool_queue_depth=0,
        )
        return self.observer.snapshot(m)

    # === ECS 边界 (per 02-BD §1.3 Adapter) ===

    def ecs_mode_active(self) -> bool:
        """L1 是否切换 ECS (per SRS §7)。ECS 实体执行在 Rust 侧。"""
        return self.mode_manager.get_mode() is RuntimeMode.Ecs

    def close(self) -> None:
        self.queue.close()
