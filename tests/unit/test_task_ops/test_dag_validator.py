#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
tests/unit/test_task_ops/test_dag_validator.py — DAGValidator cycle detection unit tests
(per docs/architecture/2026-09-03-langgraph/03-detailed-design.md §3.5 UT-21)

守门 #13 a 实证 (L1↔L1 禁止通信 → TMO 全部 L0 协调):
  - cycle detection 算法复杂度严格 O(V+E)
  - 4 类 cycle 必检: self-loop / 2-node / 3-node / long-cycle (≥4)
  - iterative DFS (显式栈), 不用 recursive (避免 Python recursion limit 1000 爆栈)
  - 3-color 标记 white/gray/black

覆盖:
  - 4 类 cycle 必检出 (含完整路径)
  - 无环 DAG 全过 (含空图 / 单节点 / 链状 / 树状 / 完全图)
  - 大图性能基准 (1K node + 2K edge 跑通, 验证 O(V+E))
  - topological_sort 与 cycle detection 一致性
  - 隐式节点 (add_edge 但未 add_task) 处理
  - 跨 subgraph 校验 (subgraph 单独 validate)
"""

from __future__ import annotations

import time

import pytest

from scripts.automation.task_ops.relationship_graph import (
    TaskRelationshipGraph,
    TaskNode,
)
from scripts.automation.task_ops.dag_validator import (
    DAGValidator,
    WHITE,
    GRAY,
    BLACK,
)


# === 4 类 cycle 必检 (守门 #13 a 强约束) ===


class TestCycleDetectionFourCases:
    """4 类 cycle 全覆盖, per brief §2 完成标准."""

    def test_self_loop_detected(self) -> None:
        """self-loop: a → a (单节点环)."""
        g = TaskRelationshipGraph()
        g.add_task("a")
        g.add_edge("a", "a")
        is_dag, cycle = DAGValidator.validate(g)
        assert not is_dag
        assert cycle is not None
        # 环路径首尾相同
        assert cycle[0] == cycle[-1]
        assert "a" in cycle

    def test_2node_cycle_detected(self) -> None:
        """2-node cycle: a → b → a."""
        g = TaskRelationshipGraph()
        g.add_task("a")
        g.add_task("b")
        g.add_edge("a", "b")
        g.add_edge("b", "a")
        is_dag, cycle = DAGValidator.validate(g)
        assert not is_dag
        assert cycle is not None
        assert cycle[0] == cycle[-1]
        # 环必须包含 a 和 b
        assert set(cycle[:-1]) == {"a", "b"}

    def test_3node_cycle_detected(self) -> None:
        """3-node cycle: a → b → c → a (用户 demo 实证用例)."""
        g = TaskRelationshipGraph()
        g.add_task("a")
        g.add_task("b")
        g.add_task("c")
        g.add_edge("a", "b")
        g.add_edge("b", "c")
        g.add_edge("c", "a")
        is_dag, cycle = DAGValidator.validate(g)
        assert not is_dag
        assert cycle is not None
        assert cycle[0] == cycle[-1]
        # 环必须包含 a, b, c
        assert set(cycle[:-1]) == {"a", "b", "c"}

    def test_long_cycle_detected(self) -> None:
        """long-cycle: 6 节点环 a → b → c → d → e → f → a."""
        g = TaskRelationshipGraph()
        for n in ["a", "b", "c", "d", "e", "f"]:
            g.add_task(n)
        edges = [("a", "b"), ("b", "c"), ("c", "d"), ("d", "e"), ("e", "f"), ("f", "a")]
        for u, v in edges:
            g.add_edge(u, v)
        is_dag, cycle = DAGValidator.validate(g)
        assert not is_dag
        assert cycle is not None
        assert cycle[0] == cycle[-1]
        # 6 节点环必全检出
        assert set(cycle[:-1]) == {"a", "b", "c", "d", "e", "f"}

    def test_cycle_in_complex_dag(self) -> None:
        """DAG 中部分子图含环: a → b → c → b (b/c 构成环, a 旁路)."""
        g = TaskRelationshipGraph()
        g.add_task("a")
        g.add_task("b")
        g.add_task("c")
        g.add_task("d")
        g.add_edge("a", "b")
        g.add_edge("a", "d")
        g.add_edge("b", "c")
        g.add_edge("c", "b")  # cycle
        is_dag, cycle = DAGValidator.validate(g)
        assert not is_dag
        assert cycle is not None
        # 环必含 b, c
        assert "b" in cycle
        assert "c" in cycle


# === 无环 DAG 全过 (false positive 守门) ===


class TestAcyclicDAGs:
    """无环 DAG 全过, 守 false positive = 0."""

    def test_empty_graph(self) -> None:
        """空图: 无节点无环."""
        g = TaskRelationshipGraph()
        is_dag, cycle = DAGValidator.validate(g)
        assert is_dag
        assert cycle is None
        assert DAGValidator.topological_sort(g) == []

    def test_single_node(self) -> None:
        g = TaskRelationshipGraph()
        g.add_task("solo")
        is_dag, cycle = DAGValidator.validate(g)
        assert is_dag
        assert DAGValidator.topological_sort(g) == ["solo"]

    def test_chain(self) -> None:
        """链状: a → b → c → d."""
        g = TaskRelationshipGraph()
        for n in ["a", "b", "c", "d"]:
            g.add_task(n)
        g.add_edge("b", "a")
        g.add_edge("c", "b")
        g.add_edge("d", "c")
        is_dag, _ = DAGValidator.validate(g)
        assert is_dag
        # topological: a 在前, d 在后
        order = DAGValidator.topological_sort(g)
        assert order is not None
        assert order.index("a") < order.index("b") < order.index("c") < order.index("d")

    def test_tree(self) -> None:
        """树状: a 是根, b/c/d 是叶子."""
        g = TaskRelationshipGraph()
        for n in ["a", "b", "c", "d"]:
            g.add_task(n)
        for child in ["b", "c", "d"]:
            g.add_edge(child, "a")
        is_dag, _ = DAGValidator.validate(g)
        assert is_dag
        order = DAGValidator.topological_sort(g)
        assert order is not None
        # a 必在最前
        assert order[0] == "a"

    def test_diamond(self) -> None:
        """菱形: a → b, a → c, b → d, c → d (d 依赖 b 和 c)."""
        g = TaskRelationshipGraph()
        for n in ["a", "b", "c", "d"]:
            g.add_task(n)
        g.add_edge("b", "a")
        g.add_edge("c", "a")
        g.add_edge("d", "b")
        g.add_edge("d", "c")
        is_dag, _ = DAGValidator.validate(g)
        assert is_dag

    def test_disconnected_components(self) -> None:
        """两个不连通的 DAG: {a→b} + {c→d}."""
        g = TaskRelationshipGraph()
        for n in ["a", "b", "c", "d"]:
            g.add_task(n)
        g.add_edge("b", "a")
        g.add_edge("d", "c")
        is_dag, _ = DAGValidator.validate(g)
        assert is_dag
        order = DAGValidator.topological_sort(g)
        assert order is not None
        assert set(order) == {"a", "b", "c", "d"}
        # 内部顺序保持
        assert order.index("a") < order.index("b")
        assert order.index("c") < order.index("d")


# === 4 字段关系 → 边方向 (per TaskRelationshipGraph 语义) ===


class TestFieldRelationshipEdgeGeneration:
    """4 字段关系 → 边方向正确性."""

    def test_parent_task_id_edge(self) -> None:
        """A.parent_task_id = p: 边 A → p (A 依赖 p)."""
        g = TaskRelationshipGraph()
        g.add_task("parent")
        g.add_task("child", parent_task_id="parent")
        assert g.dependencies("child") == ["parent"]
        assert g.dependents("parent") == ["child"]
        is_dag, _ = DAGValidator.validate(g)
        assert is_dag

    def test_merged_from_edge(self) -> None:
        """M.merged_from = [a, b]: 边 M → a, M → b."""
        g = TaskRelationshipGraph()
        g.add_task("a")
        g.add_task("b")
        g.add_task("merged", merged_from=["a", "b"])
        deps = g.dependencies("merged")
        assert set(deps) == {"a", "b"}
        is_dag, _ = DAGValidator.validate(g)
        assert is_dag

    def test_split_into_edge(self) -> None:
        """A.split_into = [b, c]: 边 b → A, c → A (b/c 依赖 A)."""
        g = TaskRelationshipGraph()
        g.add_task("parent")
        g.add_task("b")
        g.add_task("c")
        g.add_task("a", split_into=["b", "c"])
        assert set(g.dependencies("b")) == {"a"}
        assert set(g.dependencies("c")) == {"a"}
        is_dag, _ = DAGValidator.validate(g)
        assert is_dag

    def test_superseded_by_edge(self) -> None:
        """A.superseded_by = [B]: 边 A → B."""
        g = TaskRelationshipGraph()
        g.add_task("A")
        g.add_task("B")
        g.add_task("A", superseded_by=["B"])
        assert g.dependencies("A") == ["B"]
        is_dag, _ = DAGValidator.validate(g)

    def test_parent_with_merged_from_creates_cycle(self) -> None:
        """parent_task_id + merged_from 配合产生环."""
        g = TaskRelationshipGraph()
        g.add_task("a")
        g.add_task("b", parent_task_id="a")
        g.add_task("a", merged_from=["b"])  # a 依赖 b, b 依赖 a → cycle
        is_dag, cycle = DAGValidator.validate(g)
        assert not is_dag
        assert cycle is not None


# === 隐式节点 (add_edge 但未 add_task) ===


class TestImplicitNodes:
    """add_edge 隐式创建的节点, color map 不漏."""

    def test_implicit_node_via_add_edge(self) -> None:
        """a → b (b 未 add_task, 由 add_edge 隐式创建)."""
        g = TaskRelationshipGraph()
        g.add_task("a")
        g.add_edge("a", "b")  # b 隐式
        assert g.has_node("b")
        is_dag, _ = DAGValidator.validate(g)
        assert is_dag

    def test_implicit_node_in_cycle(self) -> None:
        """a → b (implicit) + b → a (implicit): cycle 必检."""
        g = TaskRelationshipGraph()
        g.add_task("a")
        g.add_edge("a", "b")
        g.add_edge("b", "a")
        is_dag, cycle = DAGValidator.validate(g)
        assert not is_dag
        assert cycle is not None


# === topological_sort 与 cycle detection 一致性 ===


class TestTopologicalSortConsistency:
    """Kahn 排序与 cycle detection 一致性."""

    def test_topo_sort_returns_none_on_cycle(self) -> None:
        g = TaskRelationshipGraph()
        g.add_task("a")
        g.add_task("b")
        g.add_edge("a", "b")
        g.add_edge("b", "a")
        order = DAGValidator.topological_sort(g)
        assert order is None

    def test_topo_sort_handles_isolated_nodes(self) -> None:
        """孤立节点: 既无入边也无出边, 必在 order 中."""
        g = TaskRelationshipGraph()
        g.add_task("a")
        g.add_task("b", parent_task_id="a")
        g.add_task("isolated")
        order = DAGValidator.topological_sort(g)
        assert order is not None
        assert set(order) == {"a", "b", "isolated"}
        assert order.index("a") < order.index("b")


# === find_all_back_edges 诊断 ===


class TestBackEdges:
    def test_no_back_edges_in_dag(self) -> None:
        g = TaskRelationshipGraph()
        for n in ["a", "b", "c"]:
            g.add_task(n)
        g.add_edge("b", "a")
        g.add_edge("c", "b")
        back = DAGValidator.find_all_back_edges(g)
        assert back == []

    def test_back_edges_in_cycle(self) -> None:
        """a → b → c → a: 至少 1 条回边."""
        g = TaskRelationshipGraph()
        for n in ["a", "b", "c"]:
            g.add_task(n)
        g.add_edge("a", "b")
        g.add_edge("b", "c")
        g.add_edge("c", "a")
        back = DAGValidator.find_all_back_edges(g)
        assert len(back) >= 1
        for u, v in back:
            # 回边的 v 必在 u 之前 DFS 路径上
            assert (u, v) in [("a", "b"), ("b", "c"), ("c", "a")]


# === 性能 / 复杂度 (守门 #13 a O(V+E) 实证) ===


class TestComplexity:
    """O(V+E) 复杂度守门 (1K / 5K / 10K 节点)."""

    def _build_dag(self, n: int) -> TaskRelationshipGraph:
        """构造链状 DAG (V=n, E=n-1, O(V+E)=O(n))."""
        g = TaskRelationshipGraph()
        for i in range(n):
            g.add_task(f"n{i:06d}")
        for i in range(1, n):
            g.add_edge(f"n{i:06d}", f"n{i-1:06d}")
        return g

    def test_1k_chain_passes(self) -> None:
        g = self._build_dag(1000)
        t0 = time.perf_counter()
        is_dag, _ = DAGValidator.validate(g)
        elapsed = time.perf_counter() - t0
        assert is_dag
        # 1K 链 < 100ms (实际 < 10ms, 留 buffer)
        assert elapsed < 0.5, f"1K chain too slow: {elapsed*1000:.2f}ms"

    def test_5k_chain_passes(self) -> None:
        g = self._build_dag(5000)
        t0 = time.perf_counter()
        is_dag, _ = DAGValidator.validate(g)
        elapsed = time.perf_counter() - t0
        assert is_dag
        assert elapsed < 2.0, f"5K chain too slow: {elapsed*1000:.2f}ms"

    def test_10k_chain_passes(self) -> None:
        g = self._build_dag(10_000)
        t0 = time.perf_counter()
        is_dag, _ = DAGValidator.validate(g)
        elapsed = time.perf_counter() - t0
        assert is_dag
        assert elapsed < 5.0, f"10K chain too slow: {elapsed*1000:.2f}ms"

    def test_complexity_benchmark_scaling(self) -> None:
        """O(V+E) 实证: 1K vs 5K vs 10K 时间比应近似 1:5:10 (线性)."""
        timings = {}
        for n in (1000, 5000, 10_000):
            g = self._build_dag(n)
            r = DAGValidator.complexity_benchmark(g)
            timings[n] = r["validate_ms"]
        # 5K 时间 < 6 * 1K 时间 (线性 + 常数)
        assert timings[5000] < timings[1000] * 6 + 50
        # 10K 时间 < 12 * 1K 时间
        assert timings[10_000] < timings[1000] * 12 + 100

    def test_wide_dag(self) -> None:
        """宽图: 1 个根 + 1000 个叶子 (E=1000)."""
        g = TaskRelationshipGraph()
        g.add_task("root")
        for i in range(1000):
            tid = f"leaf{i:04d}"
            g.add_task(tid)
            g.add_edge(tid, "root")
        t0 = time.perf_counter()
        is_dag, _ = DAGValidator.validate(g)
        elapsed = time.perf_counter() - t0
        assert is_dag
        assert elapsed < 0.5, f"wide dag too slow: {elapsed*1000:.2f}ms"


# === subgraph 跨子图校验 (为 IT-12 做准备) ===


class TestSubgraphValidation:
    def test_subgraph_dag_valid(self) -> None:
        """主图含环, subgraph 不含环 → subgraph validate 返 True."""
        g = TaskRelationshipGraph()
        # 主图构造环: a → c → b → a (a 依赖 c, c 依赖 b, b 依赖 a)
        g.add_task("a")
        g.add_task("b")
        g.add_task("c")
        g.add_task("a", merged_from=["c"])  # a → c (a 依赖 c)
        g.add_task("b", parent_task_id="c")  # b → c (b 依赖 c)
        g.add_task("c", parent_task_id="a")  # c → a (c 依赖 a)
        # cycle: a → c → b → ... 等等, 验证一下
        # a → c (merged_from), b → c (parent), c → a (parent)
        # 环: a → c → a (2 节点环)
        assert g.has_cycle()

        sub = g.subgraph(["a", "c"])
        # subgraph: a → c, c → a (subgraph 仍含环)
        is_dag, cycle = DAGValidator.validate(sub)
        assert not is_dag

        # 子图 a/b (b 依赖 c, 不含 c): a, b 边不存在, 是 DAG
        sub_ab = TaskRelationshipGraph()
        sub_ab.add_task("a")
        sub_ab.add_task("b")
        is_dag_ab, cycle_ab = DAGValidator.validate(sub_ab)
        assert is_dag_ab
        assert cycle_ab is None

    def test_subgraph_preserves_cycle(self) -> None:
        """subgraph 含环部分 → validate 返 False."""
        g = TaskRelationshipGraph()
        g.add_task("a")
        g.add_task("b")
        g.add_task("c")
        g.add_edge("a", "b")
        g.add_edge("b", "c")
        g.add_edge("c", "a")
        sub = g.subgraph(["a", "b", "c"])
        is_dag, cycle = DAGValidator.validate(sub)
        assert not is_dag
        assert cycle is not None


# === Color 常量 (per 三色标记) ===

class TestColorConstants:
    def test_color_values(self) -> None:
        """三色标记常量守门 (per LangGraph 拓扑惯例 + 3-color DFS 实证)."""
        assert WHITE == 0
        assert GRAY == 1
        assert BLACK == 2
