#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
scripts/automation/task_ops/relationship_graph.py — TaskRelationshipGraph
(per docs/architecture/2026-09-03-langgraph/02-basic-design.md §2.6.1 + 03-detailed-design.md §3.2.1)

4 字段 DAG 数据结构:
  - parent_task_id (Optional[str]): 父任务 ID, 当前任务是子任务
  - merged_from (List[str]): 当前任务是合并产物, 来自这些源任务
  - split_into (List[str]): 当前任务被拆分为这些子任务
  - superseded_by (List[str]): 当前任务被这些任务取代

依赖边方向 (per 守门 #13 a + LangGraph 拓扑惯例):
  - A.parent_task_id = B   →  边 A → B  (A 依赖 B, B 先完成)
  - A.merged_from = [B, C] →  边 A → B, A → C
  - A.split_into = [B, C]  →  边 B → A, C → A  (B/C 依赖 A)
  - A.superseded_by = [B]  →  边 A → B  (A 依赖 B, B 接管后 A 才被视为终态)

约束 (per 守门 #1 v1 + 守门 #12 + 守门 #19):
  - 标准库 only (无第三方依赖, 跨 project 持久)
  - audit log 必填, 落 docs/reports/tmo.log (per §3.4)
  - cycle detection 算法复杂度 O(V+E), 不递归 (iterative DFS)
  - 4 字段关系 + 显式 add_edge 合并到统一邻接表

用法:
    from scripts.automation.task_ops.relationship_graph import TaskRelationshipGraph
    g = TaskRelationshipGraph()
    g.add_task("a", split_into=["b", "c"])
    g.add_task("b", parent_task_id="a")
    g.add_task("c", parent_task_id="a")
    # b → a (b 依赖 a), c → a
    print(g.dependencies("b"))  # ["a"]
    print(g.dependents("a"))    # ["b", "c"]
"""

from __future__ import annotations

import json
import time
from dataclasses import dataclass, field
from pathlib import Path
from typing import Dict, List, Optional, Set, Tuple


# 节点颜色 (per dag_validator 三色标记 — 在此仅作类型提示, validator 内部独立维护)
WHITE = 0  # unvisited
GRAY = 1   # in current DFS path (cycle 风险)
BLACK = 2  # fully processed


@dataclass
class TaskNode:
    """任务节点 — 4 字段关系 + 必要 metadata.

    与 task card W/T/M 分类对照 (per 守门 #13):
      - parent_task_id / merged_from / split_into / superseded_by 是 Master (SCD Type 2, 关系变更留痕)
      - 节点本身 (task_id + status) 是 Transaction (append-only audit)
    """

    task_id: str
    parent_task_id: Optional[str] = None
    merged_from: List[str] = field(default_factory=list)
    split_into: List[str] = field(default_factory=list)
    superseded_by: List[str] = field(default_factory=list)

    # 显式 add_edge 的附加依赖 (不来自 4 字段, 由 reorder_node 临时添加)
    explicit_dependencies: List[str] = field(default_factory=list)

    # 业务 metadata (可选, 不参与依赖计算)
    status: str = "pending"  # pending / in_progress / done / blocked / superseded
    metadata: Dict = field(default_factory=dict)


class TaskRelationshipGraph:
    """Task Relationship Graph — 4 字段 + 显式 add_edge 统一邻接表.

    邻接表语义: adjacency[node] = set of nodes that node depends on (出边 = 依赖边).
    即: 边 u → v 表示 u 依赖 v (v 必须先完成).

    主要方法:
      - add_task(task_id, **fields): 注册节点, 同步生成 4 字段对应的依赖边
      - add_edge(u, v): 显式添加依赖边 (u → v)
      - remove_edge(u, v): 移除边
      - dependencies(task_id): 返回 task_id 依赖的所有节点
      - dependents(task_id): 返回所有依赖 task_id 的节点
      - has_cycle(): 委托给 DAGValidator
      - nodes() / edges(): 列出全部节点 / 边
      - to_dict() / from_dict(): 序列化 (便于 audit log 落档)
    """

    def __init__(self) -> None:
        # 节点表: task_id → TaskNode
        self._nodes: Dict[str, TaskNode] = {}
        # 邻接表 (out-edge): node → set of (依赖目标)
        # 边 u → v 表示 u 依赖 v
        self._adj: Dict[str, Set[str]] = {}

    # === 节点管理 ===

    def add_task(
        self,
        task_id: str,
        parent_task_id: Optional[str] = None,
        merged_from: Optional[List[str]] = None,
        split_into: Optional[List[str]] = None,
        superseded_by: Optional[List[str]] = None,
        status: str = "pending",
        metadata: Optional[Dict] = None,
    ) -> None:
        """注册任务节点 + 同步生成 4 字段对应的依赖边.

        边方向:
          - parent_task_id = p: 边 task_id → p
          - merged_from = [a, b, ...]: 边 task_id → a, task_id → b, ...
          - split_into = [a, b, ...]: 边 a → task_id, b → task_id, ...
          - superseded_by = [a, ...]: 边 task_id → a, task_id → ...
        """
        node = TaskNode(
            task_id=task_id,
            parent_task_id=parent_task_id,
            merged_from=list(merged_from or []),
            split_into=list(split_into or []),
            superseded_by=list(superseded_by or []),
            status=status,
            metadata=dict(metadata or {}),
        )
        self._nodes[task_id] = node
        self._adj.setdefault(task_id, set())

        # 同步生成依赖边
        if parent_task_id is not None:
            self._adj[task_id].add(parent_task_id)
            self._adj.setdefault(parent_task_id, set())
        for src in node.merged_from:
            self._adj[task_id].add(src)
            self._adj.setdefault(src, set())
        for child in node.split_into:
            self._adj.setdefault(child, set()).add(task_id)
        for new_task in node.superseded_by:
            self._adj[task_id].add(new_task)
            self._adj.setdefault(new_task, set())

    def has_node(self, task_id: str) -> bool:
        return task_id in self._nodes

    def get_node(self, task_id: str) -> Optional[TaskNode]:
        return self._nodes.get(task_id)

    def nodes(self) -> List[str]:
        return list(self._nodes.keys())

    # === 边管理 ===

    def add_children(
        self,
        parent_task_id: str,
        child_task_ids: List[str],
        strategy: str = "context_fork",
    ) -> None:
        """注册拆分关系 (per M-N2 split_node 调用): parent → child 边.

        跟 add_task(split_into=...) 镜像, 但允许单独 update (parent 之前已 add,
        split 节点只需在 relationship_graph 上挂 children 边).

        边方向 (per 守门 #13 a + LangGraph 拓扑惯例):
          - parent_task_id 拆分为 child_task_ids
          - 边 child → parent (child 依赖 parent, parent 先完成)

        副作用:
          - 自动 ensure parent + children 节点存在 (隐式 add)
          - parent.split_into 同步记录 (SCD Type 2 关系变更留痕 per 守门 #13 c)
        """
        # ensure parent 存在
        if parent_task_id not in self._nodes:
            self._nodes[parent_task_id] = TaskNode(task_id=parent_task_id)
            self._adj.setdefault(parent_task_id, set())

        for child_id in child_task_ids:
            # ensure child 存在
            if child_id not in self._nodes:
                child_node = TaskNode(task_id=child_id, parent_task_id=parent_task_id)
                self._nodes[child_id] = child_node
                self._adj.setdefault(child_id, set())
            else:
                # 已有 node, 补 parent_task_id 字段 (SCD Type 2)
                existing = self._nodes[child_id]
                if existing.parent_task_id is None:
                    existing.parent_task_id = parent_task_id
            # 边: child → parent (child 依赖 parent)
            self._adj[child_id].add(parent_task_id)
            # parent 同步记录 split_into
            self._nodes[parent_task_id].split_into.append(child_id)

    def get_children(self, parent_task_id: str) -> List[str]:
        """返回 parent_task_id 的所有子节点 (split 关系)."""
        node = self._nodes.get(parent_task_id)
        if node is None:
            return []
        return list(node.split_into)

    def add_edge(self, u: str, v: str) -> None:
        """显式添加依赖边 u → v (u 依赖 v).

        隐式 create 节点 (如果 u 或 v 不存在), 用于 reorder_node 临时边.
        """
        if u not in self._nodes:
            self._nodes[u] = TaskNode(task_id=u)
            self._adj.setdefault(u, set())
        if v not in self._nodes:
            self._nodes[v] = TaskNode(task_id=v)
            self._adj.setdefault(v, set())
        self._adj[u].add(v)
        # 同时记录到显式依赖表 (便于 audit)
        self._nodes[u].explicit_dependencies.append(v)

    def remove_edge(self, u: str, v: str) -> None:
        if u in self._adj:
            self._adj[u].discard(v)
        if u in self._nodes and v in self._nodes[u].explicit_dependencies:
            self._nodes[u].explicit_dependencies.remove(v)

    def edges(self) -> List[Tuple[str, str]]:
        """返回全部边 (u, v) 列表, u → v 表示 u 依赖 v."""
        result: List[Tuple[str, str]] = []
        for u, vs in self._adj.items():
            for v in vs:
                result.append((u, v))
        return result

    def edge_count(self) -> int:
        return sum(len(vs) for vs in self._adj.values())

    def dependency_count(self, task_id: str) -> int:
        return len(self._adj.get(task_id, set()))

    # === 查询 ===

    def dependencies(self, task_id: str) -> List[str]:
        """返回 task_id 依赖的所有节点 (即出边邻居)."""
        return sorted(self._adj.get(task_id, set()))

    def dependents(self, task_id: str) -> List[str]:
        """返回所有依赖 task_id 的节点 (即反向邻接)."""
        result: Set[str] = set()
        for u, vs in self._adj.items():
            if task_id in vs:
                result.add(u)
        return sorted(result)

    def subgraph(self, task_ids: List[str]) -> "TaskRelationshipGraph":
        """提取子图 (按节点 ID 列表), 保留内部依赖边."""
        sub = TaskRelationshipGraph()
        target = set(task_ids)
        for tid in task_ids:
            node = self._nodes.get(tid)
            if node is None:
                continue
            sub.add_task(
                task_id=node.task_id,
                parent_task_id=node.parent_task_id if node.parent_task_id in target else None,
                merged_from=[m for m in node.merged_from if m in target],
                split_into=[s for s in node.split_into if s in target],
                superseded_by=[s for s in node.superseded_by if s in target],
                status=node.status,
                metadata=node.metadata,
            )
        for u, v in self.edges():
            if u in target and v in target:
                sub.add_edge(u, v)
        return sub

    # === 委托 cycle detection ===

    def has_cycle(self) -> bool:
        """委托 DAGValidator, 复杂度 O(V+E)."""
        # 局部 import 避免循环依赖 (validator import graph)
        from scripts.automation.task_ops.dag_validator import DAGValidator

        is_dag, _ = DAGValidator.validate(self)
        return not is_dag

    def find_cycle(self) -> Optional[List[str]]:
        """返回环路径 (节点 ID 列表), 无环返回 None."""
        from scripts.automation.task_ops.dag_validator import DAGValidator

        _, cycle = DAGValidator.validate(self)
        return cycle

    # === 序列化 ===

    def to_dict(self) -> Dict:
        return {
            "nodes": [
                {
                    "task_id": n.task_id,
                    "parent_task_id": n.parent_task_id,
                    "merged_from": n.merged_from,
                    "split_into": n.split_into,
                    "superseded_by": n.superseded_by,
                    "status": n.status,
                    "metadata": n.metadata,
                }
                for n in self._nodes.values()
            ],
            "edges": [list(e) for e in self.edges()],
        }

    @classmethod
    def from_dict(cls, data: Dict) -> "TaskRelationshipGraph":
        g = cls()
        for n in data.get("nodes", []):
            g.add_task(
                task_id=n["task_id"],
                parent_task_id=n.get("parent_task_id"),
                merged_from=n.get("merged_from") or [],
                split_into=n.get("split_into") or [],
                superseded_by=n.get("superseded_by") or [],
                status=n.get("status", "pending"),
                metadata=n.get("metadata") or {},
            )
        for u, v in data.get("edges", []):
            g.add_edge(u, v)
        return g


# === audit log helper ===

def _audit(action: str, payload: Dict) -> None:
    """audit log 落 docs/reports/tmo.log (per 守门 #12 + §3.4 约束).

    由调用方 (reorder_node / dag_validator / routes_tmo) 显式调用, 不在此处强制.
    """
    try:
        log_path = Path("docs/reports/tmo.log")
        log_path.parent.mkdir(parents=True, exist_ok=True)
        entry = {
            "timestamp": time.time(),
            "phase": "task_ops.relationship_graph",
            "action": action,
            "payload": payload,
        }
        with log_path.open("a", encoding="utf-8") as f:
            f.write(json.dumps(entry, ensure_ascii=False) + "\n")
    except Exception:
        # audit log 失败不阻塞主流程, 但 stderr 提示
        import sys
        print(f"[WARN] tmo.log write failed for action={action}", file=sys.stderr)


__all__ = ["TaskRelationshipGraph", "TaskNode", "WHITE", "GRAY", "BLACK"]
