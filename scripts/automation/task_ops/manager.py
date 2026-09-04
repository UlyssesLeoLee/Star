# scripts/automation/task_ops/manager.py
# TaskOperationsManager (C-16, per docs/architecture/2026-09-03-langgraph/02-basic-design.md v0.2 §2.6)
#
# 职责:
#   - TMO 集中管理: 7 节点 (M-N1..M-N7) 调度
#   - 唯一 cross-task actor (per 守门 #13 a L1↔L1 禁止)
#   - 7 协议 (MergeRequest / SplitRequest / DepSet / BulkAction / ReassignRequest / MetadataUpdate / SummarizeResult) 路由
#   - 7 API 端点 (per 02 §2.6 8 API 端点) 委托给对应 node
#
# 设计原则 (per 守门):
#   - 守门 #13 a: 7 节点全部 L0 协调, 跨任务操作只经 L0
#   - 守门 #13 d: task card 状态 = Work, checkpoint = Transaction (append-only)
#   - 守门 #19: Python 化, 标准库 + 第三方 (无 .rs)
#   - 守门 #22: 调试控制台 (port 8080) 不污染 main 编译链
#
# 依赖 (注入):
#   - sub_agent_pool: SubAgentPool 实例 (管理 L1 sub-agents)
#   - relationship_graph: TaskRelationshipGraph (DAG, M-N3 后续子项)
#   - bulk_queue: BulkOperationQueue (M-N4 后续子项)

from __future__ import annotations

import asyncio
import logging
import time
import uuid
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Awaitable, Callable, Optional

logger = logging.getLogger("task_ops.manager")

# TMO 7 操作类型 → 节点 ID 映射 (per 02 §2.6.3 路由表)
OPERATION_TO_NODE: dict[str, str] = {
    "merge": "M-N1",
    "split": "M-N2",
    "dep_set": "M-N3",
    "bulk_action": "M-N4",
    "summarize": "M-N5",
    "reassign": "M-N6",
    "metadata": "M-N7",
}


@dataclass
class SubAgentHandle:
    """L1 sub-agent handle (mock 用, 真实接入时替换为 star_context SubAgentHandle)"""
    task_id: str
    task_type: str  # SA-01..SA-10
    state: dict = field(default_factory=dict)
    checkpoints: list[dict] = field(default_factory=list)


@dataclass
class SubAgentPool:
    """L1 sub-agent 池 (mock 用, 真实接入 star_context.sub_agent.pool)

    约束 (per 守门 #13 a L1↔L1 禁止): L0 唯一访问入口
    """
    _handles: dict[str, SubAgentHandle] = field(default_factory=dict)

    def get(self, task_id: str) -> SubAgentHandle:
        if task_id not in self._handles:
            raise KeyError(f"task {task_id} not found in sub_agent_pool")
        return self._handles[task_id]

    def add(self, task_type: str, task_id: Optional[str] = None, initial_state: Optional[dict] = None) -> SubAgentHandle:
        """添加 L1 sub-agent (mock 模式)"""
        tid = task_id or f"task-{uuid.uuid4().hex[:8]}"
        handle = SubAgentHandle(
            task_id=tid,
            task_type=task_type,
            state=initial_state or {"status": "running", "context": {}},
        )
        self._handles[tid] = handle
        return handle

    async def checkpoint(self, task_id: str, label: str) -> str:
        """stash_state (Transaction append-only per 守门 #13 d)"""
        handle = self.get(task_id)
        checkpoint_id = f"cp-{uuid.uuid4().hex[:8]}"
        handle.checkpoints.append({
            "id": checkpoint_id,
            "label": label,
            "snapshot": dict(handle.state),
            "timestamp": time.time(),
        })
        return checkpoint_id

    async def update(self, task_id: str, patch: dict) -> None:
        """update L1 state (L0 唯一入口 per 守门 #13 a)

        特殊: patch 含顶层 task_type 键时, 同步改 handle.task_type
        (per M-N6 reassign_node 跨 SA 类型切换需求, dataclass 字段 + state 镜像)
        """
        handle = self.get(task_id)
        if "task_type" in patch and patch["task_type"] != handle.task_type:
            handle.task_type = patch["task_type"]
        handle.state.update(patch)

    async def spawn(self, task_type: str, context: dict, task_id: Optional[str] = None) -> SubAgentHandle:
        """spawn 新 L1 sub-agent (L0 唯一入口 per 守门 #13 a)"""
        # 真实接入时, 这里会 dispatch SA-XX subgraph
        # mock 模式: 同步 add + 设置 context
        handle = self.add(task_type=task_type, task_id=task_id, initial_state={
            "status": "running",
            "context": context,
        })
        return handle


@dataclass
class TaskOperationsManager:
    """TMO 集中管理 (C-16 per 02 §2.6)

    7 节点全部 L0 协调, 跨任务操作只经本类 (per 守门 #13 a L1↔L1 禁止)
    """
    sub_pool: SubAgentPool = field(default_factory=SubAgentPool)
    audit_log: list[dict] = field(default_factory=list)
    state_root: Path = field(default_factory=lambda: Path("docs/reports/task_ops_state.json"))

    def audit(self, operation: str, params: dict, result: dict) -> None:
        """audit log (per 守门 #13 d Transaction 100% audit)"""
        entry = {
            "timestamp": time.time(),
            "operation": operation,
            "params": params,
            "result": result,
        }
        self.audit_log.append(entry)
        logger.info("tmo audit: %s", entry)

    def route(self, message: dict) -> str:
        """TMO 路由 (per 02 §2.6.3 路由表)"""
        op = message.get("operation")
        if op not in OPERATION_TO_NODE:
            raise ValueError(
                f"unknown TMO operation: {op!r}, expected one of {tuple(OPERATION_TO_NODE.keys())}"
            )
        return OPERATION_TO_NODE[op]

    async def dispatch(self, message: dict) -> dict:
        """统一 TMO 入口 (L0 唯一, 跨任务操作)

        流程:
          1. 路由到对应节点 (M-N1..M-N7)
          2. 调用对应 node 函数
          3. audit 落档
        """
        node_id = self.route(message)
        op = message["operation"]
        start = time.time()
        try:
            if node_id == "M-N1":
                from automation.task_ops.nodes.merge_node import merge_node
                result = await merge_node(state=message, manager=self)
            else:
                # M-N2..M-N7 待后续子项 (TMO-02..TMO-07) 实装
                raise NotImplementedError(
                    f"TMO node {node_id} ({op}) not yet implemented (per PHASE-LANGGRAPH-TMO-IMPL-REPORT v0.1, 7 子项 phase 计划)"
                )
            duration_ms = (time.time() - start) * 1000
            self.audit(op, message, {"node": node_id, "result": result, "duration_ms": duration_ms})
            return {"ok": True, "node": node_id, "result": result, "duration_ms": duration_ms}
        except Exception as e:
            duration_ms = (time.time() - start) * 1000
            self.audit(op, message, {"node": node_id, "error": str(e), "duration_ms": duration_ms})
            return {"ok": False, "node": node_id, "error": str(e), "duration_ms": duration_ms}

    def get_state_snapshot(self) -> dict:
        """状态快照 (mock mode 用于 test 验证)"""
        return {
            "sub_pool": {tid: {"task_type": h.task_type, "state": h.state, "checkpoint_count": len(h.checkpoints)} for tid, h in self.sub_pool._handles.items()},
            "audit_count": len(self.audit_log),
        }
