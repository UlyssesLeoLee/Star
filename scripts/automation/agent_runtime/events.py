#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""agent_runtime.events — EventBus + Mailbox (per SRS §26-§28 + 03-DD §4.3)。

Event Driven (per SRS §26 REQ-EVT-001): Agent 不通过持续轮询保持活跃,
Event → Event Bus → Event Router → Mailbox → Scheduler → Agent Activation。

Agent 间通信 (per SRS §27 REQ-EVT-002): 禁止 Agent A 持有 Agent B Object
Reference, 通过 Event 交互。本实现 Mailbox 只存信封 (MailboxMessage + PayloadRef),
大消息不内联 (per SRS §28 REQ-PAYLOAD-001)。

约束: 标准库 only; bounded mailbox (per SRS §44 禁止无界)。
"""

from __future__ import annotations

import threading
from collections import deque
from typing import Dict, List, Optional

from .models import AgentEvent, MailboxMessage, MailboxRef, new_id


class Mailbox:
    """bounded 邮箱 (per 03-DD §4.3 + SRS §19 REQ-ECS-011)。

    只存信封 (event_id + payload_ref), 大消息走 Payload Store。
    满时丢弃最旧消息 (bounded, per SRS §44 禁止无界 queue)。
    """

    def __init__(self, max_size: int = 256):
        if max_size < 1:
            raise ValueError("max_size 必须 >= 1 (禁止无界 mailbox, per SRS §44)")
        self.max_size = max_size
        self._messages: deque = deque(maxlen=max_size)
        self._lock = threading.Lock()

    def push(self, event: AgentEvent) -> MailboxMessage:
        """投递事件 → 信封 (per 03-DD §4.3)。"""
        msg = MailboxMessage(
            msg_id=new_id(),
            event_id=event.event_id,
            payload_ref=event.payload_ref,
            timestamp=event.timestamp,
        )
        with self._lock:
            self._messages.append(msg)  # deque(maxlen) 自动丢最旧 (bounded)
        return msg

    def unread_count(self) -> int:
        with self._lock:
            return sum(1 for m in self._messages if not m.read)

    def drain_unread(self) -> List[MailboxMessage]:
        """取未读并标记已读 (per MailboxRef.unread_count 归零)。"""
        with self._lock:
            out = [m for m in self._messages if not m.read]
            for m in self._messages:
                object.__setattr__(m, "read", True)
            return out

    def ref(self, mailbox_id: str) -> MailboxRef:
        return MailboxRef(mailbox_id=mailbox_id, unread_count=self.unread_count(), max_size=self.max_size)


class EventRouter:
    """Event Router (per 02-BD §5.3): 路由到 target agent。"""

    @staticmethod
    def route(event: AgentEvent) -> str:
        """per 02-BD §5.3: route(event) → target agent id。"""
        return event.target_agent_id


class EventBus:
    """进程内 Event Bus (per 02-BD §5.3 EventBus 接口)。

    publish → router → target mailbox; subscribe/unsubscribe 管理订阅表。
    """

    def __init__(self, mailbox_max_size: int = 256):
        self.mailbox_max_size = mailbox_max_size
        self._mailboxes: Dict[str, Mailbox] = {}
        self._subscribers: Dict[str, set] = {}  # agent_id → {sub_id}
        self._lock = threading.Lock()

    def subscribe(self, agent_id: str, sub_id: Optional[str] = None) -> str:
        """subscribe(agent_id) → sub_id (per 02-BD §5.3)。"""
        sid = sub_id or new_id()
        with self._lock:
            self._mailboxes.setdefault(agent_id, Mailbox(self.mailbox_max_size))
            self._subscribers.setdefault(agent_id, set()).add(sid)
        return sid

    def unsubscribe(self, agent_id: str, sub_id: str) -> bool:
        with self._lock:
            subs = self._subscribers.get(agent_id)
            if not subs or sub_id not in subs:
                return False
            subs.discard(sub_id)
            if not subs:
                self._subscribers.pop(agent_id, None)
            return True

    def publish(self, event: AgentEvent) -> str:
        """publish(event) → event_id (per 02-BD §5.3)。"""
        target = EventRouter.route(event)
        with self._lock:
            mailbox = self._mailboxes.get(target)
            if mailbox is None:
                mailbox = Mailbox(self.mailbox_max_size)
                self._mailboxes[target] = mailbox
        mailbox.push(event)
        return event.event_id

    def mailbox(self, agent_id: str) -> Mailbox:
        with self._lock:
            return self._mailboxes.setdefault(agent_id, Mailbox(self.mailbox_max_size))

    def drain_unread(self, agent_id: str) -> List[MailboxMessage]:
        return self.mailbox(agent_id).drain_unread()

    def published_count(self, agent_id: str) -> int:
        return self.mailbox(agent_id).unread_count()
