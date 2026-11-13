# scripts/automation/task_ops/spawn_subtask/event_bus.py
# M-32 ParentChildEventBus (C-28)
# per docs/architecture/2026-09-09-subtask-binding/03-detailed-design.md §3.3.7
# + 01-requirements.md §2 F-29 (EventBus per AGENTS.md §6.2 已有 EventBus 设计)
#
# parent/child state change event 订阅 + 派发 (异步, return_exceptions 隔离).

from __future__ import annotations

import asyncio
from collections import defaultdict
from typing import Any, Callable, DefaultDict, List


class ParentChildEventBus:
    """C-28 ParentChildEventBus: parent/child state change event 订阅派发."""

    def __init__(self) -> None:
        self._subscribers: DefaultDict[str, List[Callable[..., Any]]] = defaultdict(list)

    def subscribe(self, event_type: str, callback: Callable[..., Any]) -> None:
        """订阅事件 (同一 event_type 可多个 callback, 按订阅顺序派发)."""
        self._subscribers[event_type].append(callback)

    async def publish(self, event_type: str, **kwargs: Any) -> None:
        """发布事件 (异步并发派发, 单个订阅者异常不阻断其余, per DD §3.3.7)."""
        tasks = [
            asyncio.create_task(cb(**kwargs))
            for cb in self._subscribers.get(event_type, [])
        ]
        if tasks:
            await asyncio.gather(*tasks, return_exceptions=True)
