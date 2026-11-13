#!/usr/bin/env python3
# -*- coding: utf-8 -*-
# @cypher schema=1 source_sha256=bc96740089570e8ec45ccaff3b2b08324fe34fbe8e67c3324ccd54c49c9c978d
# MERGE (self:File {path:"scripts/automation/agent_runtime/tests/test_models.py"})
# @endcypher
"""test_models — 数据模型 (per SRS §8-§28 + 03-DD §2.1)。"""

import unittest

from automation.agent_runtime.models import (
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
    SA_TYPES,
    Task,
    TokenBudget,
    ToolPolicyRef,
    McpPolicyRef,
    WorkflowRef,
)


class TestAgentType(unittest.TestCase):
    def test_all_9_sa_types_valid(self):
        for code in ("SA-01", "SA-02", "SA-03", "SA-04", "SA-05",
                     "SA-06", "SA-07", "SA-08", "SA-09"):
            t = AgentType(code)
            self.assertEqual(t.value, code)
            self.assertEqual(t.label(), SA_TYPES[code])

    def test_sa_n_placeholder(self):
        self.assertEqual(AgentType("SA-N").label(), "new-agent-type")

    def test_invalid_rejected(self):
        with self.assertRaises(ValueError):
            AgentType("SA-10")
        with self.assertRaises(ValueError):
            AgentType("foo")

    def test_equality_and_hash(self):
        self.assertEqual(AgentType("SA-01"), AgentType("SA-01"))
        self.assertEqual(len({AgentType("SA-01"), AgentType("SA-01")}), 1)


class TestAgent(unittest.TestCase):
    def test_create(self):
        agent = Agent.create(tenant_id="t1", agent_type="SA-01")
        self.assertEqual(agent.identity.tenant_id, "t1")
        self.assertIsInstance(agent.identity, AgentIdentity)
        self.assertEqual(agent.state.current, AgentState.Idle)
        self.assertEqual(agent.lifecycle.current, LifecycleState.Cold)

    def test_agent_holds_refs_only(self):
        """per SRS §8: Agent 结构上只有引用, 无 Full Context/Client。"""
        agent = Agent.create(tenant_id="t1", agent_type="SA-01")
        agent.context_ref = ContextRef(context_id="c1", tier=ContextTier.L3Full)
        agent.memory_ref = MemoryRef(memory_id="m1", memory_type=MemoryType.Semantic)
        agent.model_ref = ModelRef(provider="mock", model="mock")
        agent.tool_policy_ref = ToolPolicyRef(policy_id="p1", tool_allowlist=("git",))
        agent.mcp_policy_ref = McpPolicyRef(policy_id="p2", server_allowlist=("filesystem",))
        agent.permission_ref = PermissionRef(acl_id="a1", tenant_id="t1")
        agent.token_budget = TokenBudget(max_context_tokens=8000)
        agent.mailbox_ref = MailboxRef(mailbox_id="mb1")
        agent.workflow_ref = WorkflowRef(workflow_id="wf1", step=1, total_steps=10)
        # 断言无客户端/线程/全量上下文字段 (结构约束, per SRS §8 不得存放)
        for forbidden in ("http_client", "llm_client", "mcp_client", "retriever",
                          "tool_registry", "full_context", "thread"):
            self.assertFalse(hasattr(agent, forbidden))


class TestPriority(unittest.TestCase):
    def test_ordering(self):
        self.assertLess(Priority.Critical, Priority.High)
        self.assertLess(Priority.High, Priority.Normal)
        self.assertLess(Priority.Normal, Priority.Low)
        self.assertLess(Priority.Low, Priority.Background)

    def test_from_int(self):
        self.assertEqual(Priority(2), Priority.Normal)


class TestEvent(unittest.TestCase):
    def test_agent_event_fields(self):
        event = AgentEvent.create(
            source_agent_id="a1",
            target_agent_id="a2",
            tenant_id="t1",
            event_type=EventType.AgentMessage,
        )
        self.assertEqual(event.target_agent_id, "a2")
        self.assertEqual(event.event_type, EventType.AgentMessage)
        self.assertTrue(event.event_id)
        self.assertTrue(event.trace_id)

    def test_mailbox_message_holds_payload_ref(self):
        ref = PayloadRef(store_id="s1", key="k1", size_bytes=10)
        msg = MailboxMessage(msg_id="m1", event_id="e1", payload_ref=ref)
        self.assertIsNotNone(msg.payload_ref)
        self.assertEqual(msg.payload_ref.size_bytes, 10)
        self.assertFalse(msg.read)


class TestTask(unittest.TestCase):
    def test_task_create_has_idempotency_keys(self):
        """per SRS §52 REQ-IDEMPOTENT-001。"""
        task = Task.create(brief="b", agent_type="SA-01", tenant_id="t1")
        self.assertTrue(task.task_id)
        self.assertTrue(task.trace_id)
        self.assertTrue(task.operation_id)

    def test_priority_default(self):
        self.assertEqual(Task.create(brief="b", agent_type="SA-01", tenant_id="t1").priority,
                         Priority.Normal)


if __name__ == "__main__":
    unittest.main()
