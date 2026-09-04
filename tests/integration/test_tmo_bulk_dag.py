#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
tests/integration/test_tmo_bulk_dag.py — IT-12 partial: TMO DAG 跨 subgraph 集成测试
(per docs/architecture/2026-09-03-langgraph/03-detailed-design.md §3.5 IT-12)

IT-12 partial (本子代理 wt-tmo-03 范围):
  - 跨 subgraph cycle detection (主图 + 子图各自校验)
  - TaskRelationshipGraph + DAGValidator + ReorderNode 端到端
  - 4 字段关系 (parent / merged_from / split_into / superseded_by) 完整
  - /api/tmo/* FastAPI 端点 (in-process TestClient)

不在本子代理范围 (跨 wt-tmo-01 / wt-tmo-04):
  - BulkOperationQueue 实际跑通 (wt-tmo-04 落地)
  - merge_node dispatch 跑通 (wt-tmo-01 落地)
  - 这里只做 cross-subgraph DAG 校验部分
"""

from __future__ import annotations

import pytest

# FastAPI TestClient (可选: 若 fastapi 不可用, 跳过 API 集成)
try:
    from fastapi import FastAPI
    from fastapi.testclient import TestClient
    HAS_FASTAPI = True
except ImportError:
    HAS_FASTAPI = False

from scripts.automation.task_ops.relationship_graph import TaskRelationshipGraph
from scripts.automation.task_ops.dag_validator import DAGValidator
from scripts.automation.task_ops.nodes.reorder_node import (
    ReorderNode,
    ReorderState,
    ReorderInterrupted,
)


# === 跨 subgraph DAG 校验 (守门 #13 a 实证) ===


class TestCrossSubgraphDAGValidation:
    """IT-12 partial: 主图 + 子图各自 cycle detection 跑通."""

    def test_main_graph_cycle_subgraph_clean(self) -> None:
        """主图含环 (c1 区), 子图 (c2 区) 无环 → 各自分别校验."""
        g = TaskRelationshipGraph()
        # c1 区: a → b → c → a (cycle)
        for n in ["a", "b", "c"]:
            g.add_task(n)
        g.add_edge("a", "b")
        g.add_edge("b", "c")
        g.add_edge("c", "a")
        # c2 区: d → e (DAG)
        for n in ["d", "e"]:
            g.add_task(n)
        g.add_edge("e", "d")

        # 主图 cycle
        assert g.has_cycle()

        # c2 子图无环
        sub_c2 = g.subgraph(["d", "e"])
        is_dag, cycle = DAGValidator.validate(sub_c2)
        assert is_dag
        assert cycle is None
        order = DAGValidator.topological_sort(sub_c2)
        assert order is not None
        assert order.index("d") < order.index("e")

    def test_subgraph_isolates_cycle(self) -> None:
        """subgraph 只取 cycle 部分 → cycle 保留."""
        g = TaskRelationshipGraph()
        for n in ["a", "b", "c", "d"]:
            g.add_task(n)
        g.add_edge("a", "b")
        g.add_edge("b", "c")
        g.add_edge("c", "a")  # cycle
        g.add_edge("d", "a")  # d 旁路

        sub = g.subgraph(["a", "b", "c"])
        is_dag, cycle = DAGValidator.validate(sub)
        assert not is_dag
        assert cycle is not None

    def test_reorder_subgraph_only(self) -> None:
        """reorder_node subgraph 模式: 只 reorder state.task_ids 范围."""
        g = TaskRelationshipGraph()
        g.add_task("root")
        for i, n in enumerate(["m1", "m2", "m3"]):
            g.add_task(n, parent_task_id="root")
        # m1, m2, m3 都依赖 root

        # subgraph reorder: 只 m1/m2/m3
        state = ReorderState(
            task_ids=["m1", "m2", "m3"],
            dep_set={},
            existing_graph=g,
        )
        node = ReorderNode()
        result = node.execute(state)
        assert result.ok
        # root 不在 order
        assert "root" not in result.order
        assert set(result.order) == {"m1", "m2", "m3"}


# === 4 字段关系端到端 (parent / merged_from / split_into / superseded_by) ===


class TestFourFieldsEndToEnd:
    """4 字段 → 边 → DAG → reorder 端到端跑通."""

    def test_split_then_reorder(self) -> None:
        """A.split_into=[B, C] + B.parent=A + C.parent=A → reorder A → B/C."""
        g = TaskRelationshipGraph()
        g.add_task("A")
        g.add_task("B", parent_task_id="A")
        g.add_task("C", parent_task_id="A")
        g.add_task("A", split_into=["B", "C"])  # B/C 依赖 A
        state = ReorderState(
            task_ids=["A", "B", "C"],
            dep_set={},
            existing_graph=g,
        )
        node = ReorderNode()
        result = node.execute(state)
        assert result.ok
        assert result.order[0] == "A"

    def test_merge_then_reorder(self) -> None:
        """M.merged_from=[A, B] + A/B done → M 在 A/B 之后."""
        g = TaskRelationshipGraph()
        g.add_task("A", status="done")
        g.add_task("B", status="done")
        g.add_task("M", merged_from=["A", "B"])
        state = ReorderState(
            task_ids=["A", "B", "M"],
            dep_set={},
            existing_graph=g,
        )
        node = ReorderNode()
        result = node.execute(state)
        assert result.ok
        order = result.order
        assert order.index("A") < order.index("M")
        assert order.index("B") < order.index("M")

    def test_supersede_creates_dependency(self) -> None:
        """A.superseded_by=[B] → A 依赖 B (B 接管后 A 才视为终态)."""
        g = TaskRelationshipGraph()
        g.add_task("A")
        g.add_task("B")
        g.add_task("A", superseded_by=["B"])
        state = ReorderState(
            task_ids=["A", "B"],
            dep_set={},
            existing_graph=g,
        )
        node = ReorderNode()
        result = node.execute(state)
        assert result.ok
        assert result.order.index("B") < result.order.index("A")


# === 跨 subgraph cycle interrupt (完整 4 字段链) ===


class TestCrossSubgraphCycleInterrupt:
    """跨 subgraph cycle 必被 reorder_node interrupt 协议捕获."""

    def test_complex_4field_cycle(self) -> None:
        """复杂 4 字段链形成 cycle: parent + merged_from 配合."""
        g = TaskRelationshipGraph()
        g.add_task("a")
        g.add_task("b", parent_task_id="a")
        g.add_task("a", merged_from=["b"])  # a → b + b → a = cycle
        state = ReorderState(
            task_ids=["a", "b"],
            dep_set={},
            existing_graph=g,
        )
        node = ReorderNode()
        with pytest.raises(ReorderInterrupted) as exc_info:
            node.execute(state)
        assert exc_info.value.source_node == "M-N3"

    def test_5node_cycle_via_fields(self) -> None:
        """5 节点环: A.parent=B, B.parent=C, ..., E.parent=A."""
        g = TaskRelationshipGraph()
        ids = ["a", "b", "c", "d", "e"]
        for n in ids:
            g.add_task(n)
        # a.parent = b → a → b
        g.add_task("a", parent_task_id="b")
        g.add_task("b", parent_task_id="c")
        g.add_task("c", parent_task_id="d")
        g.add_task("d", parent_task_id="e")
        g.add_task("e", parent_task_id="a")
        # cycle: a → b → c → d → e → a
        state = ReorderState(
            task_ids=ids,
            dep_set={},
            existing_graph=g,
        )
        node = ReorderNode()
        with pytest.raises(ReorderInterrupted) as exc_info:
            node.execute(state)
        assert set(exc_info.value.cycle_path[:-1]) == set(ids)


# === /api/tmo/* FastAPI 端点 (in-process TestClient) ===


@pytest.mark.skipif(not HAS_FASTAPI, reason="fastapi not installed")
class TestAPIDependenciesEndpoint:
    """POST /api/tmo/dependencies 端到端跑通."""

    @pytest.fixture
    def client(self):
        # 重新创建 router (独立 app, 避免 state 污染)
        from scripts.automation.api.routes_tmo import router as tmo_router
        app = FastAPI()
        app.include_router(tmo_router)
        return TestClient(app)

    def test_post_dependencies_success(self, client) -> None:
        """POST /api/tmo/dependencies 200 + ok=True."""
        resp = client.post(
            "/api/tmo/dependencies",
            json={
                "edges": [
                    {"task_id": "b", "depends_on": "a"},
                    {"task_id": "c", "depends_on": "b"},
                ],
            },
        )
        assert resp.status_code == 200
        data = resp.json()
        assert data["ok"] is True
        assert data["total_edges"] == 2
        assert data["total_nodes"] == 3
        assert data["cycle_detected"] is False

    def test_post_dependencies_409_on_cycle(self, client) -> None:
        """POST cycle 必 409 + cycle_path."""
        resp = client.post(
            "/api/tmo/dependencies",
            json={
                "edges": [
                    {"task_id": "a", "depends_on": "b"},
                    {"task_id": "b", "depends_on": "a"},
                ],
            },
        )
        assert resp.status_code == 409
        data = resp.json()
        assert "detail" in data
        assert data["detail"]["error"] == "cycle_detected"
        assert data["detail"]["cycle_path"] is not None

    def test_get_dependencies_list(self, client) -> None:
        """GET /api/tmo/dependencies 列表."""
        # 先 add
        client.post(
            "/api/tmo/dependencies",
            json={
                "edges": [
                    {"task_id": "b", "depends_on": "a"},
                ],
            },
        )
        resp = client.get("/api/tmo/dependencies")
        assert resp.status_code == 200
        data = resp.json()
        assert data["total_edges"] >= 1
        assert "a" in data["nodes"]
        assert "b" in data["nodes"]
        assert data["is_dag"] is True

    def test_get_dependencies_for_task(self, client) -> None:
        """GET /api/tmo/dependencies/{task_id}."""
        client.post(
            "/api/tmo/dependencies",
            json={"edges": [{"task_id": "child", "depends_on": "parent"}]},
        )
        resp = client.get("/api/tmo/dependencies/child")
        assert resp.status_code == 200
        data = resp.json()
        assert data["task_id"] == "child"
        assert data["dependencies"] == ["parent"]
        assert data["dependents"] == []

    def test_get_dependencies_404_unknown_task(self, client) -> None:
        """GET 不存在 task_id → 404."""
        # 先 reset
        client.delete("/api/tmo/dependencies")
        resp = client.get("/api/tmo/dependencies/ghost")
        assert resp.status_code == 404

    def test_delete_dependencies_clears_graph(self, client) -> None:
        """DELETE /api/tmo/dependencies → reset."""
        client.post(
            "/api/tmo/dependencies",
            json={"edges": [{"task_id": "x", "depends_on": "y"}]},
        )
        resp = client.delete("/api/tmo/dependencies")
        assert resp.status_code == 200
        # 验证清空
        resp2 = client.get("/api/tmo/dependencies")
        assert resp2.json()["total_nodes"] == 0
        assert resp2.json()["total_edges"] == 0

    def test_validate_endpoint_does_not_modify(self, client) -> None:
        """POST /api/tmo/dependencies/validate 不写入主图."""
        # 先 add 一些边
        client.post(
            "/api/tmo/dependencies",
            json={"edges": [{"task_id": "a", "depends_on": "b"}]},
        )
        before = client.get("/api/tmo/dependencies").json()

        # validate 一个会成环的提案
        resp = client.post(
            "/api/tmo/dependencies/validate",
            json={
                "edges": [
                    {"task_id": "b", "depends_on": "a"},
                ],
            },
        )
        assert resp.status_code == 200
        data = resp.json()
        assert data["is_dag"] is False
        assert data["cycle_path"] is not None

        # 主图未变
        after = client.get("/api/tmo/dependencies").json()
        assert before == after


@pytest.mark.skipif(not HAS_FASTAPI, reason="fastapi not installed")
class TestAPIReorderEndpoint:
    """POST /api/tmo/reorder 端到端跑通."""

    @pytest.fixture
    def client(self):
        from scripts.automation.api.routes_tmo import router as tmo_router
        app = FastAPI()
        app.include_router(tmo_router)
        return TestClient(app)

    def test_reorder_success(self, client) -> None:
        """POST /api/tmo/reorder 无环 → 200 + order."""
        resp = client.post(
            "/api/tmo/reorder",
            json={
                "task_ids": ["a", "b", "c"],
                "dep_set": {"b": ["a"], "c": ["b"]},
            },
        )
        assert resp.status_code == 200
        data = resp.json()
        assert data["ok"] is True
        order = data["order"]
        assert order.index("a") < order.index("b") < order.index("c")

    def test_reorder_409_on_cycle(self, client) -> None:
        """POST /api/tmo/reorder cycle → 409 + cycle_path."""
        resp = client.post(
            "/api/tmo/reorder",
            json={
                "task_ids": ["a", "b", "c"],
                "dep_set": {
                    "a": ["b"],
                    "b": ["c"],
                    "c": ["a"],
                },
            },
        )
        assert resp.status_code == 409
        data = resp.json()
        assert "detail" in data
        detail = data["detail"]
        assert detail["reason"] == "cycle_detected"
        assert detail["source_node"] == "M-N3"
        assert detail["cycle_path"] is not None
        assert detail["cycle_path"][0] == detail["cycle_path"][-1]


# === End-to-end (E2E 守门 #13 a 实证) ===


class TestEndToEndReorderPipeline:
    """完整 pipeline: graph build → validate → reorder → API."""

    def test_3node_cycle_full_pipeline(self) -> None:
        """构造 3-node cycle, 跑完整 pipeline, 验证 reject."""
        # 1. 构造图
        g = TaskRelationshipGraph()
        for n in ["a", "b", "c"]:
            g.add_task(n)
        g.add_edge("a", "b")
        g.add_edge("b", "c")
        g.add_edge("c", "a")

        # 2. cycle detection 必报
        assert g.has_cycle()
        cycle = g.find_cycle()
        assert cycle is not None
        assert set(cycle[:-1]) == {"a", "b", "c"}

        # 3. reorder_node interrupt 协议
        node = ReorderNode()
        state = ReorderState(
            task_ids=["a", "b", "c"],
            dep_set={"a": ["b"], "b": ["c"], "c": ["a"]},
        )
        with pytest.raises(ReorderInterrupted) as exc_info:
            node.execute(state)
        intr = exc_info.value
        # 4. interrupt 协议必含完整 cycle 路径
        assert intr.cycle_path[0] == intr.cycle_path[-1]
        assert set(intr.cycle_path[:-1]) == {"a", "b", "c"}
        # 5. 序列化 (HTTP 409 detail 用)
        d = intr.to_dict()
        assert d["interrupt_type"] == "ReorderInterrupted"
        assert d["source_node"] == "M-N3"

    def test_dag_5node_full_pipeline(self) -> None:
        """5 节点 DAG, 端到端 reorder 跑通."""
        g = TaskRelationshipGraph()
        for n in ["a", "b", "c", "d", "e"]:
            g.add_task(n)
        g.add_edge("b", "a")
        g.add_edge("c", "a")
        g.add_edge("d", "b")
        g.add_edge("d", "c")
        g.add_edge("e", "d")

        # 1. 校验
        is_dag, _ = DAGValidator.validate(g)
        assert is_dag

        # 2. reorder
        state = ReorderState(
            task_ids=["a", "b", "c", "d", "e"],
            dep_set={},
            existing_graph=g,
        )
        node = ReorderNode()
        result = node.execute(state)
        assert result.ok
        order = result.order
        # a 在最前, e 在最后
        assert order[0] == "a"
        assert order[-1] == "e"
        # d 在 b 和 c 之后, e 在 d 之后
        assert order.index("d") > order.index("b")
        assert order.index("d") > order.index("c")
        assert order.index("e") > order.index("d")
