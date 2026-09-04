#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
scripts/automation/task_ops/nodes/reorder_node.py — M-N3 reorder_node
(per docs/architecture/2026-09-03-langgraph/03-detailed-design.md §3.2.1.1 + 02-basic-design.md §2.6.2)

M-N3 reorder_node 责任:
  1. 接收 dep_set: { task_id: [depends_on_task_ids] }
  2. 构造 TaskRelationshipGraph (含隐式 add_edge)
  3. 调 DAGValidator 检测 cycle
  4. 有环 → 抛 ReorderInterrupted (interrupt 协议), 拒绝 reorder
  5. 无环 → 调 topological_sort 输出执行顺序 + 调派 SA-10 task-orchestrator 跨任务编排

守门 #13 a 实证: 全部 L0 协调, 不直接调 L1 sub-agent
  - reorder_node 本身是 L0 派发层节点
  - 跨任务编排 (派 SA-10) 走 TaskOperationsManager.dispatch_l0() (在 wt-tmo-01 落地)
  - 本节点不持有 sub-agent context, 不读 sub-agent state

interrupt 协议 (per LangGraph interrupt 范式 + 02 §2.6.3 7 协议):
  - ReorderInterrupted(cycle_path, reason, source_node="M-N3")
  - 父图 (L0) 接到 interrupt 后可选择: reject / ignore / 强制推进 (人工拍板)
  - 强约束: cycle 永不静默通过, 必须显式 resolve

用法:
    from scripts.automation.task_ops.nodes.reorder_node import (
        ReorderNode, ReorderState, ReorderResult, ReorderInterrupted,
    )

    state = ReorderState(
        task_ids=["a", "b", "c"],
        dep_set={"a": [], "b": ["a"], "c": ["a", "b"]},
    )
    node = ReorderNode()
    result = node.execute(state)
    assert result.ok
    assert result.order == ["a", "b", "c"]
"""

from __future__ import annotations

import json
import time
from dataclasses import dataclass, field
from pathlib import Path
from typing import Dict, List, Optional, Tuple

# 局部 import (避免循环依赖 + 跨 worktree 兼容性)
from scripts.automation.task_ops.relationship_graph import TaskRelationshipGraph
from scripts.automation.task_ops.dag_validator import DAGValidator


# === 中断协议 (per LangGraph interrupt + 02 §2.6.3) ===


class ReorderInterrupted(Exception):
    """M-N3 reorder_node 检测到 cycle 时抛出的 interrupt 异常.

    字段:
      - cycle_path: 环路径 (节点 ID 列表, 首尾相同)
      - reason: 中断原因 (machine-readable code)
      - source_node: 来源节点 ID (固定 "M-N3")
      - proposed_dep_set: 用户提交的 dep_set (用于诊断)
    """

    def __init__(
        self,
        cycle_path: List[str],
        reason: str = "cycle_detected",
        source_node: str = "M-N3",
        proposed_dep_set: Optional[Dict[str, List[str]]] = None,
    ) -> None:
        self.cycle_path = cycle_path
        self.reason = reason
        self.source_node = source_node
        self.proposed_dep_set = proposed_dep_set or {}
        super().__init__(
            f"ReorderInterrupted: cycle={' -> '.join(cycle_path)} reason={reason}"
        )

    def to_dict(self) -> Dict:
        return {
            "interrupt_type": "ReorderInterrupted",
            "source_node": self.source_node,
            "reason": self.reason,
            "cycle_path": self.cycle_path,
            "proposed_dep_set": self.proposed_dep_set,
        }


# === State / Result dataclass (LangGraph 节点范式) ===


@dataclass
class ReorderState:
    """M-N3 输入 state.

    字段:
      - task_ids: 参与重排的任务 ID 列表 (顺序无要求, topological_sort 重排)
      - dep_set: { task_id: [depends_on_task_ids] } 用户声明的依赖
      - existing_graph: 已有的 TaskRelationshipGraph (可空, 用于增量 reorder)
      - session_id: 跨 session 续标识
    """

    task_ids: List[str]
    dep_set: Dict[str, List[str]] = field(default_factory=dict)
    existing_graph: Optional[TaskRelationshipGraph] = None
    session_id: Optional[str] = None


@dataclass
class ReorderResult:
    """M-N3 输出 result.

    字段:
      - ok: 是否成功 (无环 + 节点齐全)
      - order: topological order (成功时填, 失败时 [])
      - cycle_path: 环路径 (失败时填, 成功时 None)
      - reason: 失败原因 code (成功时 None)
      - graph_snapshot: 校验后 graph 的 to_dict() (便于 audit / 调试)
      - duration_ms: 处理耗时 (ms)
    """

    ok: bool
    order: List[str] = field(default_factory=list)
    cycle_path: Optional[List[str]] = None
    reason: Optional[str] = None
    graph_snapshot: Optional[Dict] = None
    duration_ms: float = 0.0

    def to_dict(self) -> Dict:
        return {
            "ok": self.ok,
            "order": self.order,
            "cycle_path": self.cycle_path,
            "reason": self.reason,
            "duration_ms": self.duration_ms,
            "graph_snapshot": self.graph_snapshot,
        }


# === M-N3 reorder_node 主类 ===


class ReorderNode:
    """M-N3 reorder_node — 依赖 DAG 编排 + cycle detection.

    守门 #13 a 实证:
      - 本节点是 L0 派发层, 不持有 L1 sub-agent state
      - dep_set 解析 + 邻接表构造 + cycle detection 全部在 L0 同步跑完
      - 成功时返 topological order 给 L0 协调器 (TaskOperationsManager in wt-tmo-01)
      - 失败时抛 ReorderInterrupted interrupt, 由 L0 决定 reject / 人工拍板
    """

    NODE_ID = "M-N3"
    NODE_NAME = "reorder_node"

    def __init__(self) -> None:
        self._graph: Optional[TaskRelationshipGraph] = None
        self._last_result: Optional[ReorderResult] = None

    def execute(self, state: ReorderState) -> ReorderResult:
        """同步执行 reorder (LangGraph 节点函数范式).

        Raises:
            ReorderInterrupted: 检测到 cycle (L0 接到后走 interrupt 协议)

        Returns:
            ReorderResult: 成功时含 topological order + graph snapshot

        复杂度: O(V + E) — 全部由 DAGValidator 兜底
        """
        t0 = time.perf_counter()
        self._audit(
            "reorder_node.start",
            {
                "task_ids": state.task_ids,
                "dep_set": state.dep_set,
                "session_id": state.session_id,
            },
        )

        # 1. 构造图
        graph = state.existing_graph or TaskRelationshipGraph()
        # 确保所有 task_id 都注册为节点
        for tid in state.task_ids:
            if not graph.has_node(tid):
                graph.add_task(tid)

        # 2. 解析 dep_set, 添加显式依赖边
        for task_id, deps in state.dep_set.items():
            if not graph.has_node(task_id):
                graph.add_task(task_id)
            for dep in deps:
                graph.add_edge(task_id, dep)  # task_id 依赖 dep

        self._graph = graph

        # 3. cycle detection
        cycle = DAGValidator.find_cycle(graph)
        if cycle is not None:
            duration_ms = round((time.perf_counter() - t0) * 1000, 4)
            result = ReorderResult(
                ok=False,
                cycle_path=cycle,
                reason="cycle_detected",
                graph_snapshot=graph.to_dict(),
                duration_ms=duration_ms,
            )
            self._last_result = result
            self._audit(
                "reorder_node.cycle_detected",
                {
                    "cycle_path": cycle,
                    "duration_ms": duration_ms,
                    "session_id": state.session_id,
                },
            )
            raise ReorderInterrupted(
                cycle_path=cycle,
                reason="cycle_detected",
                source_node=self.NODE_ID,
                proposed_dep_set=state.dep_set,
            )

        # 4. topological sort (Kahn O(V+E))
        order = DAGValidator.topological_sort(graph)
        if order is None:
            # 防御: find_cycle 没找到, topological_sort 报 cycle (理论上不会发生, 但留防御)
            duration_ms = round((time.perf_counter() - t0) * 1000, 4)
            result = ReorderResult(
                ok=False,
                reason="topological_sort_failed",
                graph_snapshot=graph.to_dict(),
                duration_ms=duration_ms,
            )
            self._last_result = result
            raise ReorderInterrupted(
                cycle_path=[],
                reason="topological_sort_failed",
                source_node=self.NODE_ID,
                proposed_dep_set=state.dep_set,
            )

        # 5. 限制 order 在 state.task_ids 范围内 (subgraph reorder)
        if state.task_ids:
            target = set(state.task_ids)
            filtered = [n for n in order if n in target]
            # Kahn 的稳定性: 同层按字母序, 这里保留
            order = filtered

        duration_ms = round((time.perf_counter() - t0) * 1000, 4)
        result = ReorderResult(
            ok=True,
            order=order,
            graph_snapshot=graph.to_dict(),
            duration_ms=duration_ms,
        )
        self._last_result = result
        self._audit(
            "reorder_node.success",
            {
                "order": order,
                "duration_ms": duration_ms,
                "node_count": graph.dependency_count.__self__.edge_count() if False else graph.edge_count(),
                "session_id": state.session_id,
            },
        )
        return result

    def execute_with_interrupt_handler(
        self,
        state: ReorderState,
        on_interrupt: Optional[callable] = None,
    ) -> Tuple[Optional[ReorderResult], Optional[ReorderInterrupted]]:
        """execute + interrupt 处理器.

        用法: M-N3 caller 想 try/except 自主处理 interrupt 时走这入口.
        Returns: (result, interrupt) 二选一非空.

        on_interrupt 签名: (interrupt: ReorderInterrupted) -> Optional[ReorderResult]
          - 返回 None: 继续 raise (默认)
          - 返回 ReorderResult: 替换 raise (人工拍板后强制推进)
        """
        try:
            result = self.execute(state)
            return (result, None)
        except ReorderInterrupted as intr:
            if on_interrupt is not None:
                replaced = on_interrupt(intr)
                return (replaced, intr)
            return (None, intr)

    @property
    def last_result(self) -> Optional[ReorderResult]:
        return self._last_result

    @property
    def last_graph(self) -> Optional[TaskRelationshipGraph]:
        return self._graph

    # === audit log ===

    def _audit(self, action: str, payload: Dict) -> None:
        try:
            log_path = Path("docs/reports/tmo.log")
            log_path.parent.mkdir(parents=True, exist_ok=True)
            entry = {
                "timestamp": time.time(),
                "phase": "task_ops.nodes.reorder_node",
                "node_id": self.NODE_ID,
                "action": action,
                "payload": payload,
            }
            with log_path.open("a", encoding="utf-8") as f:
                f.write(json.dumps(entry, ensure_ascii=False) + "\n")
        except Exception:
            import sys
            print(f"[WARN] tmo.log write failed for action={action}", file=sys.stderr)


__all__ = [
    "ReorderNode",
    "ReorderState",
    "ReorderResult",
    "ReorderInterrupted",
]
