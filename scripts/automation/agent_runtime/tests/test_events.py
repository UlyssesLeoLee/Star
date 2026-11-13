#!/usr/bin/env python3
# -*- coding: utf-8 -*-
# @cypher schema=1 source_sha256=4cf283860b5ab8241942e91c700414e20be0c24aee11fc7273384c8e62bfbd8e
# MERGE (self:File {path:"scripts/automation/agent_runtime/tests/test_events.py"})
# @endcypher
"""test_events — EventBus + Mailbox (per SRS §26-§28 + 02-BD §5.3)。"""

import unittest

from automation.agent_runtime.events import EventBus, EventRouter, Mailbox
from automation.agent_runtime.models import AgentEvent, EventType, PayloadRef


def make_event(source, target, tenant="t1", payload=None):
    return AgentEvent.create(
        source_agent_id=source,
        target_agent_id=target,
        tenant_id=tenant,
        event_type=EventType.AgentMessage,
        payload_ref=payload,
    )


class TestMailbox(unittest.TestCase):
    def test_push_and_unread(self):
        mb = Mailbox(max_size=8)
        mb.push(make_event("a", "b"))
        self.assertEqual(mb.unread_count(), 1)

    def test_drain_marks_read(self):
        mb = Mailbox(max_size=8)
        mb.push(make_event("a", "b"))
        out = mb.drain_unread()
        self.assertEqual(len(out), 1)
        self.assertEqual(mb.unread_count(), 0)

    def test_bounded_drops_oldest(self):
        """per SRS §44: 禁止无界 — 满时丢最旧。"""
        mb = Mailbox(max_size=2)
        mb.push(make_event("a", "b"))
        mb.push(make_event("c", "b"))
        mb.push(make_event("d", "b"))
        out = mb.drain_unread()
        self.assertEqual(len(out), 2)

    def test_zero_max_size_rejected(self):
        with self.assertRaises(ValueError):
            Mailbox(max_size=0)

    def test_ref_unread_count(self):
        mb = Mailbox(max_size=8)
        mb.push(make_event("a", "b"))
        ref = mb.ref("mb1")
        self.assertEqual(ref.unread_count, 1)


class TestEventBus(unittest.TestCase):
    def test_publish_routes_to_target_mailbox(self):
        bus = EventBus(mailbox_max_size=8)
        bus.subscribe("target")
        bus.publish(make_event("src", "target"))
        self.assertEqual(bus.published_count("target"), 1)
        # 非目标不受影响
        self.assertEqual(bus.published_count("other"), 0)

    def test_publish_returns_event_id(self):
        bus = EventBus()
        event = make_event("src", "dst")
        self.assertEqual(bus.publish(event), event.event_id)

    def test_subscribe_unsubscribe(self):
        bus = EventBus()
        sub_id = bus.subscribe("agent1")
        self.assertTrue(bus.unsubscribe("agent1", sub_id))
        self.assertFalse(bus.unsubscribe("agent1", sub_id))
        self.assertFalse(bus.unsubscribe("ghost", "x"))

    def test_router_routes_to_target(self):
        event = make_event("src", "dst")
        self.assertEqual(EventRouter.route(event), "dst")

    def test_payload_ref_not_cloned(self):
        """per SRS §28: 大消息经 PayloadRef, 不在 Agent 间 Clone。"""
        bus = EventBus()
        ref = PayloadRef(store_id="s1", key="big.json", size_bytes=1_000_000)
        bus.publish(make_event("a", "b", payload=ref))
        msgs = bus.drain_unread("b")
        self.assertEqual(len(msgs), 1)
        ref2 = msgs[0].payload_ref
        assert ref2 is not None
        self.assertEqual(ref2.key, "big.json")
        self.assertEqual(ref2.size_bytes, 1_000_000)


if __name__ == "__main__":
    unittest.main()
