#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""agent_runtime.scheduler — L0/L1 调度器 (per SRS §42-§44 + 03-DD §6.1/§6.4)。

职责 (per SRS §42 REQ-SCH-001):
    Ready Queue / Priority / Fairness / HOT Slot / Tenant Quota 的 Python 侧骨架。

调度算法 (per 03-DD §6.1):
    1. Ready Queue 按 (priority ASC, seq ASC) 取
    2. Critical/High 立即调度; Normal/Low/Background 走 per-tenant round-robin
       fair share (防止 Background 饿死 + 防单 tenant 占满, per SRS §57)
    3. HOT Slot 限制 (per 03-DD §6.4 check_hot_slot_available):
       hot_count < max_hot_agents, 否则排队等待 (禁止 unlimited spawn, per SRS §44)

复杂度: heapq push/pop O(log N)。
约束: 标准库 only。
"""

from __future__ import annotations

import heapq
import itertools
import time
from dataclasses import dataclass, field
from typing import Dict, List, Optional

from .models import Priority, Task


@dataclass
class ScheduleResult:
    """一次调度结果 (per 02-BD §3.1 Dispatcher.dispatch 返)。"""

    scheduled: List[str] = field(default_factory=list)
    deferred: List[str] = field(default_factory=list)
    dropped_low: List[str] = field(default_factory=list)
    reason: Optional[str] = None


class Scheduler:
    """Agent Ready Queue + Priority + Fairness + HOT Slot (per SRS §42-§44)。"""

    def __init__(self, max_hot_agents: int = 100, fair_share: int = 2):
        if max_hot_agents < 1:
            raise ValueError("max_hot_agents 必须 >= 1 (禁止无 HOT 上限, per SRS §30)")
        self.max_hot_agents = max_hot_agents
        self.fair_share = fair_share
        self._seq = itertools.count()
        #: ready heap: (priority, seq, task_id)
        self._ready: List[tuple] = []
        self._by_id: Dict[str, tuple] = {}
        self._hot: Dict[str, float] = {}  # task_id → 进入 HOT 时间
        self._tenant_turn: Dict[str, int] = {}

    # === Ready Queue ===

    def enqueue_ready(self, task: Task) -> None:
        """任务入 Ready Queue (幂等: 同 task_id 不重复入)。"""
        if task.task_id in self._by_id:
            return
        entry = (int(task.priority), next(self._seq), task.task_id)
        self._by_id[task.task_id] = entry
        heapq.heappush(self._ready, entry)

    def ready_queue_depth(self) -> int:
        return len(self._ready)

    def drop_low_priority(self, count: int = 1) -> List[str]:
        """溢出策略 DropLow (per 03-DD §6.3): 丢最低优先级 (Background)。"""
        dropped = []
        # Background(4) > Low(3) > ... → 从大到小删
        by_priority_desc = sorted(self._ready, reverse=True)
        for entry in by_priority_desc[:count]:
            if entry in self._ready:
                self._ready.remove(entry)
                heapq.heapify(self._ready)
                self._by_id.pop(entry[2], None)
                dropped.append(entry[2])
        return dropped

    # === HOT Slot (per 03-DD §6.4) ===

    def hot_count(self) -> int:
        return len(self._hot)

    def hot_slot_usage(self) -> float:
        """0.0-1.0 (per SRS §74 hot_slot_usage)。"""
        if self.max_hot_agents <= 0:
            return 0.0
        return len(self._hot) / self.max_hot_agents

    def hot_slot_available(self) -> bool:
        """per 03-DD §6.4 check_hot_slot_available。"""
        return len(self._hot) < self.max_hot_agents

    def drain_deferred(self) -> List[str]:
        """清空 Ready Queue 并返回未调度 (deferred) 的 task_id 列表。

        调用方据此把 task requeue 回 TaskQueue (per 03-DD §6.1 re-enqueue)。
        """
        out = [entry[2] for entry in self._ready]
        for entry in self._ready:
            self._by_id.pop(entry[2], None)
        self._ready = []
        return out

    # === 调度 ===

    def _fair_ok(self, task: Task) -> bool:
        """Normal/Low/Background per-tenant round-robin (per SRS §57)。"""
        if task.priority <= Priority.High:
            return True  # Critical/High 立即调度 (per 03-DD §6.1)
        turn = self._tenant_turn.get(task.tenant_id, 0)
        return turn < self.fair_share

    def schedule(self, ready_tasks: Optional[List[Task]] = None) -> ScheduleResult:
        """执行一轮调度 (per 03-DD §6.1)。

        ready_tasks: 外部注入的候选任务 (来自 TaskQueue.dequeue); 缺省用
        enqueue_ready 的内部堆。每轮公平份额重置 (per-dispatch-round 语义),
        未满足 fair share 的任务重排队尾, 二次碰到即停止 (防死循环)。
        """
        if ready_tasks is not None:
            for t in ready_tasks:
                self.enqueue_ready(t)
        self._tenant_turn = {}  # 每轮重置 (per SRS §57 round 语义)
        result = ScheduleResult()
        deferred_once: List[str] = []
        while self._ready:
            if not self.hot_slot_available():
                result.reason = f"HotSlotFull ({self.hot_count()}/{self.max_hot_agents})"
                break
            priority, _seq, task_id = heapq.heappop(self._ready)
            self._by_id.pop(task_id, None)
            task = self._resolve_task(task_id, priority)
            if task is None:
                continue
            if not self._fair_ok(task):
                # Re-enqueue at tail (per 03-DD §6.1), 二次碰到即停
                if task_id in deferred_once:
                    self.enqueue_ready(task)
                    break
                deferred_once.append(task_id)
                self.enqueue_ready(task)
                continue
            if task.priority > Priority.High:
                # 只有受 fair share 约束的 Normal/Low/Background 计入配额
                # (Critical/High 立即调度, 不计 fair 份额, per 03-DD §6.1)
                self._tenant_turn[task.tenant_id] = (
                    self._tenant_turn.get(task.tenant_id, 0) + 1
                )
            self.acquire_hot(task_id)
            result.scheduled.append(task_id)
        return result

    def _resolve_task(self, task_id: str, priority: int) -> Optional[Task]:
        """由 task_id 重建 Task (注入场景下调用方提供原任务表)。"""
        task = self._task_registry.get(task_id) if hasattr(self, "_task_registry") else None
        if task is not None:
            return task
        # 内部堆场景: 调度器不持有完整 Task, 用最小投影 (优先级已含在 key)
        return self._minimal_task(task_id, priority)

    @staticmethod
    def _minimal_task(task_id: str, priority: int) -> Task:
        return Task(
            task_id=task_id,
            brief="",
            agent_type="SA-N",
            tenant_id="",
            priority=Priority(priority),
        )

    def register_task(self, task: Task) -> None:
        """登记完整任务对象 (供 _resolve_task 还原)。"""
        if not hasattr(self, "_task_registry"):
            self._task_registry = {}
        self._task_registry[task.task_id] = task

    # === HOT 生命周期 ===

    def acquire_hot(self, task_id: str) -> bool:
        """dispatch_to_hot (per 03-DD §6.4): 有槽 → 计数+1; 无槽 → False。"""
        if not self.hot_slot_available():
            return False
        self._hot[task_id] = time.monotonic()
        return True

    def release_hot(self, task_id: str) -> None:
        self._hot.pop(task_id, None)

    def agent_wait_time(self, task_id: str) -> Optional[float]:
        """agent_wait_time 指标 (per SRS §74)。"""
        if task_id not in self._hot:
            return None
        return time.monotonic() - self._hot[task_id]
