#!/usr/bin/env python3
# -*- coding: utf-8 -*-
# @cypher schema=1 source_sha256=f099d57b2c43e650b1a1565b22b92dbd575df25e16a7455ec07757de807f6b28
# MERGE (self:File {path:"scripts/automation/agent_runtime/tests/test_taskgraph.py"})
# @endcypher
"""test_taskgraph — LangGraph 语义任务图 (per LangGraph 9/3 02 + TASK WBS 6.2)。"""

import unittest

from automation.agent_runtime.taskgraph import (
    CycleError,
    GraphNode,
    GraphState,
    TaskGraph,
    TaskGraphError,
    reducer_add,
    reducer_append,
    reducer_merge_dict,
    reducer_overwrite,
)


class TestReducers(unittest.TestCase):
    def test_overwrite(self):
        self.assertEqual(reducer_overwrite(1, 2), 2)
        self.assertIsNone(reducer_overwrite(None, None))

    def test_add(self):
        self.assertEqual(reducer_add(0, 5), 5)
        self.assertEqual(reducer_add(None, 3), 3)
        self.assertEqual(reducer_add(2, 3), 5)

    def test_append(self):
        self.assertEqual(reducer_append(None, [1]), [1])
        self.assertEqual(reducer_append([1], [2, 3]), [1, 2, 3])

    def test_merge_dict(self):
        self.assertEqual(reducer_merge_dict({"a": 1}, {"b": 2}), {"a": 1, "b": 2})
        self.assertEqual(reducer_merge_dict(None, {"b": 2}), {"b": 2})


class TestGraphStructure(unittest.TestCase):
    def test_node_kind_validation(self):
        GraphNode(name="n1", kind="SA-01")
        with self.assertRaises(TaskGraphError):
            GraphNode(name="n2", kind="SA-99")

    def test_duplicate_node_rejected(self):
        g = TaskGraph()
        g.add_node(GraphNode(name="n1", kind="SA-01"))
        with self.assertRaises(TaskGraphError):
            g.add_node(GraphNode(name="n1", kind="SA-02"))

    def test_edge_to_unknown_node_rejected(self):
        g = TaskGraph()
        g.add_node(GraphNode(name="n1", kind="SA-01"))
        with self.assertRaises(TaskGraphError):
            g.add_edge("n1", "ghost")

    def test_self_loop_rejected(self):
        g = TaskGraph()
        g.add_node(GraphNode(name="n1", kind="SA-01"))
        with self.assertRaises(TaskGraphError):
            g.add_edge("n1", "n1")

    def test_compile_topological_order(self):
        g = TaskGraph()
        for name in ("a", "b", "c", "d"):
            g.add_node(GraphNode(name=name, kind="SA-01"))
        g.add_edge("a", "b")
        g.add_edge("b", "c")
        g.add_edge("a", "d")
        order = g.compile()
        self.assertEqual(order, ["a", "b", "d", "c"])
        self.assertEqual(set(order), {"a", "b", "c", "d"})

    def test_cycle_detected(self):
        g = TaskGraph()
        for name in ("a", "b", "c"):
            g.add_node(GraphNode(name=name, kind="SA-01"))
        g.add_edge("a", "b")
        g.add_edge("b", "c")
        g.add_edge("c", "a")
        with self.assertRaises(CycleError):
            g.compile()


class TestGraphRun(unittest.TestCase):
    def test_run_merges_state_via_reducers(self):
        """LangGraph semantics: 节点返回 partial, reducer 合并。"""
        g = TaskGraph()
        g.add_node(GraphNode(name="step1", kind="SA-04", fn=lambda s, c: {"n": 1}))
        g.add_node(GraphNode(name="step2", kind="SA-06", fn=lambda s, c: {"n": 2, "log": "x"}))
        g.add_edge("step1", "step2")
        state = GraphState(
            values={"n": 0, "log": ""},
            reducers={"n": reducer_add, "log": reducer_append},
        )
        g.run(state, ctx={})
        self.assertEqual(state.values["n"], 3)  # reducer_add: 0+1+2
        self.assertEqual(state.values["log"], ["x"])  # reducer_append: "" → ["x"]

    def test_run_dependency_order(self):
        executed = []
        g = TaskGraph()
        g.add_node(GraphNode(name="a", kind="SA-05", fn=lambda s, c: executed.append("a")))
        g.add_node(GraphNode(name="b", kind="SA-02", fn=lambda s, c: executed.append("b")))
        g.add_node(GraphNode(name="c", kind="SA-09", fn=lambda s, c: executed.append("c")))
        g.add_edge("a", "c")
        g.add_edge("b", "c")
        g.run(GraphState(), ctx={})
        self.assertEqual(executed[:2], ["a", "b"])
        self.assertEqual(executed[2], "c")

    def test_run_concurrent_bounded(self):
        """max_concurrency > 1 走 bounded ThreadPoolExecutor (per SRS §44)。"""
        g = TaskGraph()
        for i in range(4):
            g.add_node(GraphNode(name=f"n{i}", kind="SA-08",
                                 fn=lambda s, c, i=i: {f"n{i}": i}))
        g.run(GraphState(), ctx={}, max_concurrency=2)
        self.assertEqual(len(g.nodes), 4)

    def test_run_without_fn_is_noop(self):
        g = TaskGraph()
        g.add_node(GraphNode(name="a", kind="SA-03"))
        state = GraphState(values={"k": "v"})
        g.run(state, ctx={})
        self.assertEqual(state.values["k"], "v")

    def test_merge_dict_state(self):
        g = TaskGraph()
        g.add_node(GraphNode(name="a", kind="SA-01",
                             fn=lambda s, c: {"config": {"x": 1}}))
        g.add_node(GraphNode(name="b", kind="SA-01",
                             fn=lambda s, c: {"config": {"y": 2}}))
        g.add_edge("a", "b")
        state = GraphState(reducers={"config": reducer_merge_dict})
        g.run(state, ctx={})
        self.assertEqual(state.values["config"], {"x": 1, "y": 2})


if __name__ == "__main__":
    unittest.main()
