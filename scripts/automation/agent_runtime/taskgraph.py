#!/usr/bin/env python3
# -*- coding: utf-8 -*-
# @cypher schema=1 source_sha256=19752bf8ce036a671a967540c37a77e8b2bf69a0e08955ed41798ce1af87f623
# MERGE (self:File {path:"scripts/automation/agent_runtime/taskgraph.py"})
# @endcypher
"""agent_runtime.taskgraph — LangGraph 任务图模式 (per LangGraph 9/3 02 + TASK WBS 6.2)。

LangGraph view 跟 Agent Runtime view 平行 (per 02-BD §1.3):
    - LangGraph subgraph 实现 SA-XX 业务逻辑 (9 SA Type 节点)
    - Agent Runtime 提供底层 Runtime (本包其他模块)
    两者通过 Adapter 连接 (per 02-BD §1.3 关系段)。

本模块以标准库实现 LangGraph 语义子集 (真实 langgraph 依赖接入待 P3-B 续,
per 已知缺口 #1):
    - 节点: GraphNode (9 SA Type 节点 kind, per LangGraph 9/3 02 §3 图示)
    - 边: GraphEdge (src → dst, per LangGraph 状态 + 边 + reducer)
    - 状态: GraphState (dict + reducer 函数, LangGraph semantics: 节点返回
      partial update, 由 reducer 合并进全局 state)
    - 执行: compile() 拓扑排序 (Kahn, 循环检测) → run() 按依赖顺序执行,
      支持 bounded 并发 (per SRS §44 禁止 unlimited spawn)

参考实现: scripts/automation/task_ops/dag_validator.py (现役 DAG 校验),
本模块独立重写为可执行图 (能 import 的绝不重写, 但执行语义不重复其静态校验)。

约束: 标准库 only。
"""

from __future__ import annotations

from collections import defaultdict, deque
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass, field
from typing import Any, Callable, Dict, List, Optional, Set

from .models import SA_TYPES


class TaskGraphError(RuntimeError):
    """图结构 / 执行异常。"""


class CycleError(TaskGraphError):
    """拓扑排序检出环 (禁止环, per LangGraph DAG 语义)。"""


# ============================================================================
# §1 Reducer (LangGraph 状态合并语义)
# ============================================================================


def reducer_overwrite(current: Any, update: Any) -> Any:
    """overwrite: 覆盖 (LangGraph default reducer)。"""
    return update


def reducer_add(current: Any, update: Any) -> Any:
    """add: 数值累加。"""
    return (current or 0) + (update or 0)


def reducer_append(current: Any, update: Any) -> Any:
    """append: 列表追加 (不修改原列表)。"""
    return list(current or []) + list(update or [])


def reducer_merge_dict(current: Any, update: Any) -> Any:
    """merge: dict 浅合并, update 优先。"""
    out = dict(current or {})
    out.update(update or {})
    return out


# ============================================================================
# §2 Graph 元素 (per LangGraph 9/3 02: 节点 + 边 + state)
# ============================================================================


@dataclass
class GraphNode:
    """任务图节点。kind 对应 9 SA Type (per LangGraph 9/3 02 §3)。"""

    name: str
    kind: str  # SA-01..SA-09 / SA-N
    fn: Optional[Callable[["GraphState", Dict[str, Any]], Optional[dict]]] = None

    def __post_init__(self) -> None:
        if self.kind not in SA_TYPES and self.kind != "SA-N":
            raise TaskGraphError(f"非法节点 kind: {self.kind!r} (允许 {sorted(SA_TYPES)} + SA-N)")


@dataclass(frozen=True)
class GraphEdge:
    """有向边 src → dst (per LangGraph 边语义)。"""

    src: str
    dst: str


@dataclass
class GraphState:
    """图状态: values + reducer 表 (LangGraph state semantics)。"""

    values: Dict[str, Any] = field(default_factory=dict)
    reducers: Dict[str, Callable[[Any, Any], Any]] = field(default_factory=dict)

    def update(self, partial: Optional[Dict[str, Any]]) -> "GraphState":
        """按 reducer 合并 partial (节点返回 partial update)。"""
        for key, val in (partial or {}).items():
            reducer = self.reducers.get(key, reducer_overwrite)
            self.values[key] = reducer(self.values.get(key), val)
        return self


# ============================================================================
# §3 TaskGraph (compile + run, per LangGraph 拓扑执行)
# ============================================================================


class TaskGraph:
    """LangGraph 语义任务图 (per LangGraph 9/3 02, 标准库实现)。"""

    def __init__(self, name: str = "taskgraph"):
        self.name = name
        self.nodes: Dict[str, GraphNode] = {}
        self.edges: List[GraphEdge] = []
        self._order: List[str] = []
        self._deps: Dict[str, Set[str]] = {}

    def add_node(self, node: GraphNode) -> "TaskGraph":
        if node.name in self.nodes:
            raise TaskGraphError(f"节点重名: {node.name}")
        self.nodes[node.name] = node
        return self

    def add_edge(self, src: str, dst: str) -> "TaskGraph":
        if src not in self.nodes or dst not in self.nodes:
            raise TaskGraphError(f"边引用未注册节点: {src} -> {dst}")
        if src == dst:
            raise TaskGraphError(f"自环禁止: {src}")
        self.edges.append(GraphEdge(src=src, dst=dst))
        return self

    def compile(self) -> List[str]:
        """Kahn 拓扑排序 (per 03-DD §6 算法风格), 检出环 → CycleError。

        Returns: 执行顺序 (单链拓扑序)。
        """
        indegree = {name: 0 for name in self.nodes}
        dependents: Dict[str, List[str]] = defaultdict(list)
        for edge in self.edges:
            indegree[edge.dst] += 1
            dependents[edge.src].append(edge.dst)
        ready = deque(sorted(n for n, d in indegree.items() if d == 0))
        order: List[str] = []
        while ready:
            node = ready.popleft()
            order.append(node)
            for nxt in dependents[node]:
                indegree[nxt] -= 1
                if indegree[nxt] == 0:
                    ready.append(nxt)
        if len(order) != len(self.nodes):
            cyclic = sorted(n for n, d in indegree.items() if d > 0)
            raise CycleError(f"任务图存在环, 节点: {cyclic}")
        self._order = order
        self._deps = {
            name: {e.src for e in self.edges if e.dst == name} for name in self.nodes
        }
        return list(order)

    def run(
        self,
        state: GraphState,
        ctx: Optional[Dict[str, Any]] = None,
        max_concurrency: int = 1,
    ) -> GraphState:
        """执行图 (per LangGraph: 依赖满足才执行节点, 返回状态更新)。

        max_concurrency > 1 → ThreadPoolExecutor (bounded, per SRS §44)。
        """
        if not self._order:
            self.compile()
        ctx = dict(ctx or {})
        if max_concurrency < 1:
            raise TaskGraphError("max_concurrency 必须 >= 1 (禁止 unlimited, per SRS §44)")

        done: Set[str] = set()
        remaining = list(self._order)
        executor = None
        if max_concurrency > 1:
            executor = ThreadPoolExecutor(max_workers=max_concurrency)

        def _run_node(node: GraphNode) -> Optional[dict]:
            if node.fn is None:
                return None
            return node.fn(state, ctx)

        try:
            while remaining:
                runnable = [n for n in remaining if self._deps[n] <= done]
                if not runnable:
                    raise CycleError(f"依赖无法满足 (环或缺失节点): {remaining}")
                if executor is None:
                    for n in runnable:
                        partial = _run_node(self.nodes[n])
                        state.update(partial)
                        done.add(n)
                        remaining.remove(n)
                else:
                    futures = {executor.submit(_run_node, self.nodes[n]): n for n in runnable}
                    for fut, n in futures.items():
                        partial = fut.result()  # 节点异常向上抛 (fail fast)
                        state.update(partial)
                        done.add(n)
                        remaining.remove(n)
        finally:
            if executor is not None:
                executor.shutdown(wait=True)
        return state
