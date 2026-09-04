#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
scripts/automation/task_ops/dag_validator.py — DAGValidator
(per docs/architecture/2026-09-03-langgraph/03-detailed-design.md §3.2.1.1 cycle detection)

算法: 3-color DFS iterative
  - WHITE (0): unvisited
  - GRAY  (1): on current DFS path (有回到自身的环风险)
  - BLACK (2): fully processed (从该节点出发的所有路径都已探明)

复杂度严格 O(V + E):
  - 每个节点恰好入栈 / 出栈一次 (push / pop)
  - 每条边恰好被 examine 一次 (遍历邻接表)
  - 颜色数组 O(V) 空间, 显式栈空间 O(V) 最坏 (链状 DAG)

iterative DFS (不用 recursive) 原因:
  - 跨平台避免 Python 默认 recursion limit (1000), 1M 任务场景会爆栈
  - 显式栈可被 audit log 完整记录 (回溯 + 路径回放)
  - 跟守门 #13 a L0 协调一致 (L0 派发层禁止递归下钻到 L1 sub-agent)

主要 API:
  - DAGValidator.validate(graph) -> (is_dag: bool, cycle: Optional[List[str]])
  - DAGValidator.find_cycle(graph) -> Optional[List[str]]
  - DAGValidator.topological_sort(graph) -> Optional[List[str]]  (无环时返回 Kahn / DFS 序, 有环时 None)
  - DAGValidator.find_all_back_edges(graph) -> List[(u, v)]  (环入口边集合, 用于诊断)

用法:
    from scripts.automation.task_ops.relationship_graph import TaskRelationshipGraph
    from scripts.automation.task_ops.dag_validator import DAGValidator

    g = TaskRelationshipGraph()
    g.add_task("a")
    g.add_task("b", parent_task_id="a")  # b → a
    is_dag, cycle = DAGValidator.validate(g)
    assert is_dag
    order = DAGValidator.topological_sort(g)
    assert order == ["a", "b"]
"""

from __future__ import annotations

import json
import time
from pathlib import Path
from typing import Dict, List, Optional, Set, Tuple


# 3-color 标记 (per relationship_graph 中常量一致)
WHITE = 0  # unvisited
GRAY = 1   # on current DFS path
BLACK = 2  # fully processed


class DAGValidator:
    """DAG Validator — cycle detection O(V+E) + 拓扑排序.

    静态方法入口, 内部状态 (color / parent / stack) 局部维护, 不依赖单例.
    """

    @staticmethod
    def validate(graph) -> Tuple[bool, Optional[List[str]]]:
        """校验 graph 是否是 DAG.

        Returns:
            (is_dag, cycle_path):
              - is_dag = True  → DAG, cycle_path = None
              - is_dag = False → 含环, cycle_path 是环上节点 ID 列表 (u → ... → u, 首尾相同)

        复杂度: O(V + E)
        """
        cycle = DAGValidator.find_cycle(graph)
        return (cycle is None, cycle)

    @staticmethod
    def find_cycle(graph) -> Optional[List[str]]:
        """找出一个环 (节点路径), 无环返回 None.

        策略: iterative DFS + 3-color
          - 遇到 GRAY 邻居 = 回边, 立即沿 parent[] 链回溯到该邻居, 得到环路径
          - 节点首次访问推入栈, 邻居遍历完后标 BLACK + 弹栈
        """
        nodes: List[str] = graph.nodes()
        if not nodes:
            return None

        color: Dict[str, int] = {n: WHITE for n in nodes}
        # parent[u] = DFS 树中到达 u 的前驱 (用于环路径回溯)
        parent: Dict[str, Optional[str]] = {n: None for n in nodes}

        # 显式栈: (node, iterator_over_neighbors) — 模拟 recursive DFS 栈帧
        # 每个栈帧: 当前节点 + 已访问邻居的迭代器
        # 我们用 list index 追踪 "下一次要看的邻居" 避免每次重建 iterator
        for start in nodes:
            if color[start] != WHITE:
                continue
            # 显式 DFS 栈: list of (node, pending_neighbors_list)
            stack: List[Tuple[str, List[str]]] = [(start, sorted(graph.dependencies(start)))]
            color[start] = GRAY

            while stack:
                u, pending = stack[-1]
                if pending:
                    v = pending.pop(0)  # 取下一个邻居
                    if v not in color:
                        # 隐式节点 (add_edge 创建但未 add_task) — 视为 WHITE
                        color[v] = WHITE
                        parent[v] = None
                    if color[v] == GRAY:
                        # 回边 u → v (v 在当前路径上), 沿 parent 链回溯到 v
                        cycle = DAGValidator._reconstruct_cycle(parent, u, v)
                        DAGValidator._audit(
                            "find_cycle.detected",
                            {"start": start, "back_edge": [u, v], "cycle": cycle},
                        )
                        return cycle
                    if color[v] == WHITE:
                        color[v] = GRAY
                        parent[v] = u
                        stack.append((v, sorted(graph.dependencies(v))))
                else:
                    # u 的所有邻居都访问完, 标 BLACK + 弹栈
                    color[u] = BLACK
                    stack.pop()

        return None

    @staticmethod
    def _reconstruct_cycle(parent: Dict[str, Optional[str]], u: str, v: str) -> List[str]:
        """回溯环路径: 从 u 沿 parent[] 走到 v, 输出 [v, ..., u, v].

        语义:
          - v 是 DFS 树中 u 的祖先, 边 u → v 是回边
          - DFS 树路径 v → ... → u, 加上回边 u → v 构成完整环
          - 输出: [v, parent_chain_reversed_until_u, u, v] (首尾相同, 标识完整环)

        例子:
          - 2-node: parent={b:a}, u=b, v=a → [a, b, a]
          - 3-node: parent={b:a, c:b}, u=c, v=a → [a, c, b, a]
          - self-loop: u=v=a → [a, a]
        """
        path: List[str] = [v]
        cur: Optional[str] = u
        while cur != v:
            path.append(cur)
            cur = parent.get(cur)
            if cur is None:
                # 防御: parent 链断, 直接闭合到 v
                break
        path.append(v)
        return path

    @staticmethod
    def topological_sort(graph) -> Optional[List[str]]:
        """拓扑排序 (Kahn's algorithm on reversed graph), 无环返回 list, 有环返回 None.

        边方向约定 (per TaskRelationshipGraph):
          - u → v 表示 u 依赖 v (v 是 u 的 prerequisite)
          - 期望执行序: v 先, u 后

        算法: 在反向图 (v → u, 即 v 是 u 的 prerequisite) 上跑 Kahn
          - 反向图 in_degree[v] = 多少 u 满足 v → u (i.e., 多少任务把 v 当作 prerequisite)
          - 反向图 in_degree = 0 = 没有任务把该节点当作 prerequisite = 该节点无依赖 (leaf)
          - 从 leaf 开始处理, 即 "先执行无依赖任务"

        时间复杂度 O(V + E).
        跟 DFS-based 拓扑一致: 任一稳定的 topological order 即可.
        """
        nodes: List[str] = graph.nodes()
        if not nodes:
            return []

        # 反向图: rev_adj[v] = {u : u → v in original, i.e., u depends on v}
        # 反向图入度: rev_in_degree[v] = |{u : v → u in rev, i.e., u depends on v}|
        #           = |{u : v in graph.dependencies(u)}| = graph.dependent_count(v)
        rev_in_degree: Dict[str, int] = {n: 0 for n in nodes}
        rev_adj: Dict[str, List[str]] = {n: [] for n in nodes}
        for u, vs in graph._adj.items():  # type: ignore[attr-defined]
            for v in vs:
                if v not in rev_in_degree:
                    rev_in_degree[v] = 0
                # u → v in original → v → u in reverse
                rev_in_degree[u] = rev_in_degree.get(u, 0) + 1
                rev_adj.setdefault(v, []).append(u)

        # Kahn: 反向图入度 0 = 原图出度 0 = 无依赖任务 (leaf in dep graph)
        from collections import deque

        queue: deque = deque(sorted([n for n, d in rev_in_degree.items() if d == 0]))
        order: List[str] = []
        while queue:
            v = queue.popleft()
            order.append(v)
            for u in sorted(rev_adj.get(v, [])):
                rev_in_degree[u] -= 1
                if rev_in_degree[u] == 0:
                    queue.append(u)

        if len(order) != len(rev_in_degree):
            DAGValidator._audit(
                "topological_sort.cycle",
                {"expected": len(rev_in_degree), "got": len(order)},
            )
            return None
        return order

    @staticmethod
    def find_all_back_edges(graph) -> List[Tuple[str, str]]:
        """找全部回边 (u, v), v 在 u 之前的 DFS 路径上.

        用于 cycle 诊断, 列出所有环入口边.
        """
        nodes: List[str] = graph.nodes()
        if not nodes:
            return []

        color: Dict[str, int] = {n: WHITE for n in nodes}
        back_edges: List[Tuple[str, str]] = []
        for start in nodes:
            if color[start] != WHITE:
                continue
            stack: List[Tuple[str, List[str]]] = [(start, sorted(graph.dependencies(start)))]
            color[start] = GRAY
            while stack:
                u, pending = stack[-1]
                if pending:
                    v = pending.pop(0)
                    if v not in color:
                        color[v] = WHITE
                    if color[v] == GRAY:
                        back_edges.append((u, v))
                    elif color[v] == WHITE:
                        color[v] = GRAY
                        stack.append((v, sorted(graph.dependencies(v))))
                else:
                    color[u] = BLACK
                    stack.pop()
        return back_edges

    # === 性能基准 (per 守门 #13 a 算法复杂度严格 O(V+E) 实证) ===

    @staticmethod
    def complexity_benchmark(graph) -> Dict:
        """跑 validate / topological_sort / find_all_back_edges, 报告耗时.

        用于 unit test 守门 #13 a 实证: O(V+E) 比例验证.
        """
        results: Dict = {"nodes": len(graph.nodes()), "edges": graph.edge_count()}
        t0 = time.perf_counter()
        is_dag, cycle = DAGValidator.validate(graph)
        results["validate_ms"] = round((time.perf_counter() - t0) * 1000, 4)
        t0 = time.perf_counter()
        order = DAGValidator.topological_sort(graph)
        results["topological_sort_ms"] = round((time.perf_counter() - t0) * 1000, 4)
        t0 = time.perf_counter()
        back = DAGValidator.find_all_back_edges(graph)
        results["find_all_back_edges_ms"] = round((time.perf_counter() - t0) * 1000, 4)
        results["is_dag"] = is_dag
        results["has_cycle"] = cycle is not None
        results["topological_sort_returned"] = order is not None
        return results

    # === audit log ===

    @staticmethod
    def _audit(action: str, payload: Dict) -> None:
        try:
            log_path = Path("docs/reports/tmo.log")
            log_path.parent.mkdir(parents=True, exist_ok=True)
            entry = {
                "timestamp": time.time(),
                "phase": "task_ops.dag_validator",
                "action": action,
                "payload": payload,
            }
            with log_path.open("a", encoding="utf-8") as f:
                f.write(json.dumps(entry, ensure_ascii=False) + "\n")
        except Exception:
            import sys
            print(f"[WARN] tmo.log write failed for action={action}", file=sys.stderr)


__all__ = ["DAGValidator", "WHITE", "GRAY", "BLACK"]
