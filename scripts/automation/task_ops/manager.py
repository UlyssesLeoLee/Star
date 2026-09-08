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

# TMO 8 操作类型 → 节点 ID 映射 (per 02 §2.6.3 路由表 v0.3 + ADR-0049 加入 create)
OPERATION_TO_NODE: dict[str, str] = {
    "merge": "M-N1",
    "split": "M-N2",
    "dep_set": "M-N3",
    "bulk_action": "M-N4",
    "summarize": "M-N5",
    "reassign": "M-N6",
    "metadata": "M-N7",
    "create": "M-N8",  # per ADR-0049 + 2026-09-09 04:57 JST 用户拍板核心功能
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

    def has_agent(self, agent_id: str) -> bool:
        """检查 agent 是否在 L0 池中有效注册 (per 拍板: '存在有效 agent' 强约束)

        PoC mock 模式: 检查 _agents dict (per ADR-0049 v0.1 内存版 SubAgentPool)
        真实接入 star_context.sub_agent.registry 时, 走 registry.has(agent_id)

        Returns:
            True if agent_id 在 L0 池中注册, False otherwise
        """
        # PoC: 接受任何非空 agent_id (mock), 真实接入时替换为 registry.has()
        return bool(agent_id and agent_id.strip())

    async def dispatch(
        self,
        agent_id: str,
        sa_type: str,
        task_id: str,
        worktree_id: str,
        tenant_id: str,
    ) -> str:
        """dispatch SA-XX sub-agent (per ADR-0049 M-N8 拍板: 自动接管, 5s 内任务卡 in_progress)

        流程:
          1. 验证 agent_id 有效 (per 守门 #13 a L0 唯一入口)
          2. 创建 AgentSession (per INV-WT-07 1 Worktree → 0..N AgentSession)
          3. spawn L1 sub-agent (per M-N8 create_node 协调)
          4. 返 agent_session_id (L0 唯一签发, UI 端可轮询状态)

        PoC mock 模式: 内存版, 真实接入 star_context.sub_agent.pool 时替换

        Returns:
            agent_session_id (新签发, 格式 "ags-{uuid8}")
        """
        if not self.has_agent(agent_id):
            raise ValueError(
                f"SubAgentPool.dispatch: agent_id {agent_id!r} not registered "
                f"(per 拍板: '存在有效 agent' 强约束)"
            )

        agent_session_id = f"ags-{uuid.uuid4().hex[:8]}"
        # PoC: 内存版, 真实 AgentSession 走 G-WT-01 DB 接入
        context = {
            "agent_id": agent_id,
            "sa_type": sa_type,
            "task_id": task_id,
            "worktree_id": worktree_id,
            "tenant_id": tenant_id,
            "agent_session_id": agent_session_id,
            "status": "running",
            "started_at_ms": int(time.time() * 1000),
        }
        await self.spawn(task_type=sa_type, context=context, task_id=task_id)
        logger.info(
            f"SubAgentPool.dispatch: agent_session={agent_session_id} "
            f"sa={sa_type} task={task_id} wt={worktree_id} tenant={tenant_id}"
        )
        return agent_session_id


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
            elif node_id == "M-N2":
                from automation.task_ops.nodes.split_node import split_node
                result = await split_node(state=message, manager=self)
            elif node_id == "M-N5":
                from automation.task_ops.nodes.summarize_node import summarize_node
                result = await summarize_node(state=message, manager=self)
            elif node_id == "M-N6":
                from automation.task_ops.nodes.reassign_node import reassign_node
                result = await reassign_node(state=message, manager=self)
            elif node_id == "M-N7":
                from automation.task_ops.nodes.metadata_node import metadata_node
                result = await metadata_node(state=message, manager=self)
            elif node_id == "M-N8":
                # M-N8 create_node (per ADR-0049 + 2026-09-09 04:57 JST 用户拍板核心功能)
                # 走独立 _create_task 路径, 因为 create_node 需要 worktree_registry (TMO 7 节点无此依赖)
                result = await self._create_task(message)
            else:
                # M-N3 reorder_node + M-N4 bulk_node 走独立 factory 路径 (per wt-tmo-03 + wt-tmo-04)
                # 不经 manager.dispatch, 直接调 _REORDER_NODE / _bulk_queue
                raise NotImplementedError(
                    f"TMO node {node_id} ({op}) uses factory pattern, dispatch via manager not supported "
                    f"(per M-N3 reorder_node / M-N4 bulk_node factory pattern, wt-tmo-03 + wt-tmo-04 实装)"
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

    # ============================================================
    # M-N8 create_node 入口 (per ADR-0049 + 2026-09-09 04:57 JST 拍板)
    # ============================================================

    async def create(self, request: dict) -> dict:
        """M-N8 公开入口 — UI (sprint/+ New issue / board/+ New issue) 调用

        流程 (per 拍板 trigger-location_opt1):
          1. 校验 request (CreateTaskRequest 字段)
          2. 委托 dispatch() → create_node (M-N8)
          3. 返 CreateTaskResponse (task_id + worktree_id + agent_session_id + status 流)

        Args:
            request: CreateTaskRequest dict, 见 protocols.py

        Returns:
            {"ok": True, "node": "M-N8", "result": CreateTaskResponse, "duration_ms": int}
            或 {"ok": False, "node": "M-N8", "error": str, "duration_ms": int}
        """
        return await self.dispatch(request)

    async def _create_task(self, request: dict) -> dict:
        """M-N8 create_node 内部协调

        职责:
          1. 调 create_node 真实工作
          2. 协调 worktree_registry (PoC 内存版, 真实 DB 推 G-WT-01)
          3. 协调 metadata_registry (per M-N7 协同, 创建即有 metadata)
          4. 返 CreateTaskResponse

        Args:
            request: CreateTaskRequest dict (同 create())

        Returns:
            CreateTaskResponse dict, 见 protocols.py
        """
        from automation.task_ops.nodes.create_node import create_node
        # PoC 内存版 registry (per ADR-0049 v0.1, 真实 DB 推 G-WT-01)
        worktree_registry = self._get_or_create_worktree_registry()
        metadata_registry = self._get_or_create_metadata_registry()

        return await create_node(
            sub_agent_pool=self.sub_pool,
            worktree_registry=worktree_registry,
            metadata_registry=metadata_registry,
            request=request,
        )

    def _get_or_create_worktree_registry(self) -> "WorktreeRegistry":
        """获取或创建 worktree_registry (PoC 内存版)

        真实接入: per G-WT-01, 替换为 db.worktree_repo
        """
        if not hasattr(self, "_worktree_registry"):
            self._worktree_registry = WorktreeRegistry()
        return self._worktree_registry

    def _get_or_create_metadata_registry(self) -> "TaskMetadataRepository | None":
        """获取或创建 metadata_registry (PoC 内存版, per M-N7 协同)

        真实接入: per G-WT-01, 替换为 db.task_metadata_repo
        """
        if not hasattr(self, "_metadata_registry"):
            # TaskMetadataRepository 已存在 (per task_metadata_repo.py)
            try:
                from automation.task_ops.task_metadata_repo import TaskMetadataRepository
                # PoC: SQLite 内存版, 真实 DB 路径推 G-WT-01
                self._metadata_registry = TaskMetadataRepository(":memory:")
            except ImportError:
                # 真实 task_metadata_repo 不存在时, 走 None (create_node 内已容错)
                self._metadata_registry = None
        return self._metadata_registry


@dataclass
class WorktreeRegistry:
    """Worktree 内存注册表 (per ADR-0049 v0.1 + 守门 #13 d Transaction append-only)

    PoC: 内存版, 真实 DB 推 G-WT-01 (per HANDOFF-ST-001 H2 阻塞解除后启动)
    约束: 物理删除禁止 (per 守门 #13 d Transaction), 仅追加 + 状态机推进
    """
    _worktrees: dict = field(default_factory=dict)

    def add(self, worktree: dict) -> None:
        """添加 worktree (L0 唯一入口)"""
        wt_id = worktree["id"]
        if wt_id in self._worktrees:
            raise ValueError(f"worktree {wt_id} already exists (per 守门 #13 d 物理删除禁止)")
        self._worktrees[wt_id] = worktree

    def update(self, wt_id: str, patch: dict) -> None:
        """更新 worktree 状态 (per 17 状态机推进)"""
        if wt_id not in self._worktrees:
            raise KeyError(f"worktree {wt_id} not found")
        self._worktrees[wt_id].update(patch)

    def get(self, wt_id: str) -> dict:
        if wt_id not in self._worktrees:
            raise KeyError(f"worktree {wt_id} not found")
        return self._worktrees[wt_id]

    def list(self) -> list[dict]:
        return list(self._worktrees.values())
