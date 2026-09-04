#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
tests/unit/test_task_ops/test_reorder_node.py — M-N3 reorder_node unit tests
(per docs/architecture/2026-09-03-langgraph/03-detailed-design.md §3.5 UT-22)

UT-22 实证:
  - reorder_node 输入 dep_set, 输出 topological order
  - cycle 检测 → 抛 ReorderInterrupted (HTTP 409 interrupt 协议)
  - 4 字段关系 (parent / merged_from / split_into / superseded_by) 正确入图
  - 跨 subgraph reorder (state.task_ids 限定范围)
  - execute_with_interrupt_handler 人工拍板路径

守门 #13 a 实证:
  - reorder_node 是 L0 协调层节点, 不持有 L1 sub-agent state
  - interrupt 协议 (ReorderInterrupted) 用于 L0 通知 L0 协调器 (TaskOperationsManager)
"""

from __future__ import annotations

import pytest

from scripts.automation.task_ops.relationship_graph import TaskRelationshipGraph
from scripts.automation.task_ops.dag_validator import DAGValidator
from scripts.automation.task_ops.nodes.reorder_node import (
    ReorderNode,
    ReorderState,
    ReorderResult,
    ReorderInterrupted,
)


# === Happy path 跑通 (无环 reorder) ===


class TestReorderNodeHappyPath:
    """M-N3 reorder_node 无环 reorder 全过."""

    def test_simple_chain(self) -> None:
        """链状 dep: b→a, c→b, d→c (d 依赖 c, c 依赖 b, b 依赖 a)."""
        node = ReorderNode()
        state = ReorderState(
            task_ids=["a", "b", "c", "d"],
            dep_set={
                "b": ["a"],
                "c": ["b"],
                "d": ["c"],
            },
        )
        result = node.execute(state)
        assert result.ok
        assert result.cycle_path is None
        order = result.order
        # 顺序: a 在最前, d 在最后
        assert order.index("a") < order.index("b") < order.index("c") < order.index("d")

    def test_diamond(self) -> None:
        """菱形: d 依赖 b 和 c, b/c 都依赖 a."""
        node = ReorderNode()
        state = ReorderState(
            task_ids=["a", "b", "c", "d"],
            dep_set={
                "b": ["a"],
                "c": ["a"],
                "d": ["b", "c"],
            },
        )
        result = node.execute(state)
        assert result.ok
        order = result.order
        assert order[0] == "a"
        assert order[-1] == "d"
        assert order.index("b") < order.index("d")
        assert order.index("c") < order.index("d")

    def test_independent_tasks(self) -> None:
        """无依赖任务, 全部入 order (顺序按字母)."""
        node = ReorderNode()
        state = ReorderState(
            task_ids=["a", "b", "c"],
            dep_set={},
        )
        result = node.execute(state)
        assert result.ok
        assert sorted(result.order) == ["a", "b", "c"]

    def test_empty_task_list(self) -> None:
        """空 task_ids: 返 ok=True + 空 order."""
        node = ReorderNode()
        state = ReorderState(task_ids=[], dep_set={})
        result = node.execute(state)
        assert result.ok
        assert result.order == []


# === 4 字段关系 → reorder 正确性 ===


class TestReorderNodeFieldRelationships:
    """M-N3 接受 4 字段关系 (parent / merged_from / split_into / superseded_by)."""

    def test_parent_task_id_reorder(self) -> None:
        """child.parent = parent → parent 先."""
        g = TaskRelationshipGraph()
        g.add_task("parent")
        g.add_task("child", parent_task_id="parent")
        state = ReorderState(
            task_ids=["parent", "child"],
            dep_set={},
            existing_graph=g,
        )
        node = ReorderNode()
        result = node.execute(state)
        assert result.ok
        assert result.order.index("parent") < result.order.index("child")

    def test_merged_from_reorder(self) -> None:
        """merged.merged_from = [a, b] → a, b 都先."""
        g = TaskRelationshipGraph()
        g.add_task("a")
        g.add_task("b")
        g.add_task("merged", merged_from=["a", "b"])
        state = ReorderState(
            task_ids=["a", "b", "merged"],
            dep_set={},
            existing_graph=g,
        )
        node = ReorderNode()
        result = node.execute(state)
        assert result.ok
        order = result.order
        assert order.index("a") < order.index("merged")
        assert order.index("b") < order.index("merged")

    def test_split_into_reorder(self) -> None:
        """A.split_into = [b, c] → A 先, b/c 后."""
        g = TaskRelationshipGraph()
        g.add_task("A")
        g.add_task("b")
        g.add_task("c")
        g.add_task("A", split_into=["b", "c"])
        state = ReorderState(
            task_ids=["A", "b", "c"],
            dep_set={},
            existing_graph=g,
        )
        node = ReorderNode()
        result = node.execute(state)
        assert result.ok
        order = result.order
        assert order.index("A") < order.index("b")
        assert order.index("A") < order.index("c")


# === Cycle interrupt 协议 (守门 #13 a 强约束实证) ===


class TestReorderNodeCycleInterrupt:
    """4 类 cycle 全部触发 ReorderInterrupted interrupt."""

    def test_3node_cycle_raises_interrupt(self) -> None:
        """3-node cycle a → b → c → a (用户 demo 实证用例)."""
        node = ReorderNode()
        state = ReorderState(
            task_ids=["a", "b", "c"],
            dep_set={
                "a": ["b"],
                "b": ["c"],
                "c": ["a"],
            },
        )
        with pytest.raises(ReorderInterrupted) as exc_info:
            node.execute(state)
        intr = exc_info.value
        assert intr.source_node == "M-N3"
        assert intr.reason == "cycle_detected"
        assert intr.cycle_path is not None
        assert intr.cycle_path[0] == intr.cycle_path[-1]
        # 环必含 a, b, c
        assert set(intr.cycle_path[:-1]) == {"a", "b", "c"}

    def test_self_loop_raises_interrupt(self) -> None:
        """self-loop: a 依赖 a."""
        node = ReorderNode()
        state = ReorderState(
            task_ids=["a"],
            dep_set={"a": ["a"]},
        )
        with pytest.raises(ReorderInterrupted) as exc_info:
            node.execute(state)
        assert "a" in exc_info.value.cycle_path

    def test_2node_cycle_raises_interrupt(self) -> None:
        node = ReorderNode()
        state = ReorderState(
            task_ids=["a", "b"],
            dep_set={"a": ["b"], "b": ["a"]},
        )
        with pytest.raises(ReorderInterrupted) as exc_info:
            node.execute(state)
        assert set(exc_info.value.cycle_path[:-1]) == {"a", "b"}

    def test_long_cycle_raises_interrupt(self) -> None:
        """6 节点环."""
        node = ReorderNode()
        state = ReorderState(
            task_ids=["a", "b", "c", "d", "e", "f"],
            dep_set={
                "a": ["b"],
                "b": ["c"],
                "c": ["d"],
                "d": ["e"],
                "e": ["f"],
                "f": ["a"],
            },
        )
        with pytest.raises(ReorderInterrupted) as exc_info:
            node.execute(state)
        intr = exc_info.value
        assert set(intr.cycle_path[:-1]) == {"a", "b", "c", "d", "e", "f"}

    def test_interrupt_to_dict(self) -> None:
        """interrupt 协议 serialization (HTTP 409 detail 用)."""
        node = ReorderNode()
        state = ReorderState(
            task_ids=["a", "b"],
            dep_set={"a": ["b"], "b": ["a"]},
        )
        with pytest.raises(ReorderInterrupted) as exc_info:
            node.execute(state)
        d = exc_info.value.to_dict()
        assert d["interrupt_type"] == "ReorderInterrupted"
        assert d["source_node"] == "M-N3"
        assert d["reason"] == "cycle_detected"
        assert "cycle_path" in d
        assert d["proposed_dep_set"] == {"a": ["b"], "b": ["a"]}

    def test_interrupt_proposed_dep_set_preserved(self) -> None:
        """interrupt 内 proposed_dep_set 保留用户输入, 便于诊断."""
        node = ReorderNode()
        original = {"x": ["y"], "y": ["x"], "z": ["x"]}
        state = ReorderState(task_ids=["x", "y", "z"], dep_set=original)
        with pytest.raises(ReorderInterrupted) as exc_info:
            node.execute(state)
        assert exc_info.value.proposed_dep_set == original


# === execute_with_interrupt_handler (人工拍板路径) ===


class TestReorderNodeInterruptHandler:
    """execute_with_interrupt_handler 允许 L0 caller 自主处理 interrupt."""

    def test_default_propagates_interrupt(self) -> None:
        """不传 on_interrupt: 跟 execute 行为一致, raise."""
        node = ReorderNode()
        state = ReorderState(
            task_ids=["a", "b"],
            dep_set={"a": ["b"], "b": ["a"]},
        )
        result, intr = node.execute_with_interrupt_handler(state)
        assert result is None
        assert intr is not None
        assert intr.cycle_path is not None

    def test_on_interrupt_replaces_with_force_result(self) -> None:
        """on_interrupt 返 ReorderResult: 替换 raise (人工拍板强制推进)."""
        node = ReorderNode()
        state = ReorderState(
            task_ids=["a", "b"],
            dep_set={"a": ["b"], "b": ["a"]},
        )

        def force_through(intr: ReorderInterrupted) -> ReorderResult:
            # 人工拍板: 强制按 a → b 推进 (忽略 cycle)
            return ReorderResult(
                ok=True,
                order=["a", "b"],
                reason="force_pushed_by_human",
                graph_snapshot=None,
            )

        result, intr = node.execute_with_interrupt_handler(state, on_interrupt=force_through)
        assert result is not None
        assert result.ok
        assert result.reason == "force_pushed_by_human"
        assert result.order == ["a", "b"]
        assert intr is not None  # interrupt 仍被记录

    def test_on_interrupt_returning_none_keeps_propagation(self) -> None:
        """on_interrupt 返 None: 视为不处理, 但 return 路径仍记录 interrupt."""
        node = ReorderNode()
        state = ReorderState(
            task_ids=["a", "b"],
            dep_set={"a": ["b"], "b": ["a"]},
        )

        def no_op(intr: ReorderInterrupted) -> None:
            return None

        result, intr = node.execute_with_interrupt_handler(state, on_interrupt=no_op)
        assert result is None
        assert intr is not None


# === 子图 reorder (state.task_ids 限定范围) ===


class TestReorderNodeSubgraph:
    """order 仅返回 state.task_ids 内的节点, 外部节点不进 order."""

    def test_subgraph_reorder(self) -> None:
        """主图含 a→b→c, state.task_ids=[b, c] → order=[b, c]."""
        g = TaskRelationshipGraph()
        g.add_task("a")
        g.add_task("b", parent_task_id="a")
        g.add_task("c", parent_task_id="b")
        state = ReorderState(
            task_ids=["b", "c"],
            dep_set={},
            existing_graph=g,
        )
        node = ReorderNode()
        result = node.execute(state)
        assert result.ok
        assert set(result.order) == {"b", "c"}
        # c 依赖 b → c 在 b 后
        assert result.order.index("b") < result.order.index("c")


# === graph_snapshot (audit 实证) ===


class TestReorderNodeSnapshot:
    """result.graph_snapshot 必含完整图, 便于 audit log 落地."""

    def test_snapshot_contains_all_nodes(self) -> None:
        node = ReorderNode()
        state = ReorderState(
            task_ids=["a", "b", "c"],
            dep_set={"b": ["a"], "c": ["b"]},
        )
        result = node.execute(state)
        assert result.graph_snapshot is not None
        snap = result.graph_snapshot
        assert "nodes" in snap
        assert "edges" in snap
        assert len(snap["nodes"]) == 3
        # edges 至少 2 条
        assert len(snap["edges"]) >= 2

    def test_duration_ms_recorded(self) -> None:
        node = ReorderNode()
        state = ReorderState(
            task_ids=["a", "b"],
            dep_set={"b": ["a"]},
        )
        result = node.execute(state)
        assert result.duration_ms > 0
        assert result.duration_ms < 1000  # < 1s


# === last_result / last_graph 探针 (调试用) ===


class TestReorderNodeProbes:
    def test_last_result_after_success(self) -> None:
        node = ReorderNode()
        state = ReorderState(task_ids=["a"], dep_set={})
        node.execute(state)
        assert node.last_result is not None
        assert node.last_result.ok

    def test_last_graph_populated(self) -> None:
        node = ReorderNode()
        state = ReorderState(task_ids=["a", "b"], dep_set={"b": ["a"]})
        node.execute(state)
        assert node.last_graph is not None
        assert "a" in node.last_graph.nodes()
        assert "b" in node.last_graph.nodes()
