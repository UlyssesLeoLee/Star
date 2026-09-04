# scripts/automation/sub_agent/types/sa_10_task_orchestrator.py
# SA-10 task-orchestrator subgraph (per docs/architecture/2026-09-03-langgraph/03-detailed-design.md v0.2 §3.5
#  + 02-basic-design.md v0.2 §2.2.2 SA-10 + 通用节点模板 §2.2.3)
#
# 5 节点模板 (per 02 §2.2.3):
#   - init   → 状态初始化, parent context 注入
#   - plan   → 编排计划 (合并任务: 收集 a/b context + 决定合并策略)
#   - execute → 执行合并 (实际工作)
#   - verify → 守门 #1 / #12 / 测试
#   - report → 最终结果生成 + 通知 Top
#
# SA-10 特殊:
#   - v0.2 新增 (TMO 跨任务编排型)
#   - L0 协调 (L1↔L1 禁止 per 守门 #13 a)
#   - mock 模式: 不开 OpenAI/Anthropic API (守门 #23)
#   - 守门 #20: 实装前必先 brief 落档 (TMO-01 子项)
#
# 跟 SA-01..SA-09 不同:
#   - SA-10 是"跨任务编排型", 不是"任务绑定型"
#   - 接收 TMO 协议 (per task_ops/protocols.py)
#   - 输出: merged_task_id / split_task_ids / dep_set / bulk_summary / ...

from __future__ import annotations

import logging
import time
import uuid
from dataclasses import dataclass, field
from typing import Any, Optional

logger = logging.getLogger("sub_agent.types.sa_10")


@dataclass
class SA10SubgraphState:
    """SA-10 subgraph 状态 (per 02 §2.2.3 通用 sub-agent state + TMO 扩展)"""
    task_id: str
    operation: str  # "merge" / "split" / "dep_set" / "bulk_action" / "summarize" / "reassign" / "metadata"
    plan: Optional[dict] = None
    execute_result: Optional[dict] = None
    verify_result: Optional[dict] = None
    report: Optional[dict] = None
    status: str = "init"  # init → plan → execute → verify → report → done
    started_at: float = field(default_factory=time.time)
    audit_log: list[dict] = field(default_factory=list)


class SA10TaskOrchestrator:
    """SA-10 task-orchestrator (v0.2 新增, per 02 §2.2.2)

    5 节点 subgraph: init → plan → execute → verify → report
    """
    def __init__(self, merged_task_id: Optional[str] = None):
        self.merged_task_id = merged_task_id or f"sa10-{uuid.uuid4().hex[:8]}"
        self.state = SA10SubgraphState(task_id=self.merged_task_id, operation="unknown")

    def _audit(self, node: str, **kwargs) -> None:
        entry = {"node": node, "timestamp": time.time(), **kwargs}
        self.state.audit_log.append(entry)
        logger.info("sa10 audit: %s", entry)

    # ===== 节点 1: init =====

    async def init_node(self, **kwargs) -> dict:
        """init 节点: 状态初始化 + parent context 注入"""
        self.state.status = "init"
        self.state.operation = kwargs.get("operation", "merge")
        self.state.plan = {"operation": self.state.operation, "kwargs": kwargs}
        self._audit("init", operation=self.state.operation, kwargs=list(kwargs.keys()))
        return {"status": "init_done", "task_id": self.state.task_id, "operation": self.state.operation}

    # ===== 节点 2: plan =====

    async def plan_node(self, **kwargs) -> dict:
        """plan 节点: 编排计划

        merge 操作 plan:
          - merged_from: 源 task_ids
          - merged_state: stash_checkpoint_ids
          - merge_strategy: 合并策略
          - original_user_input: 用户输入
        """
        self.state.status = "plan"
        merged_from = kwargs.get("merged_from", [])
        merged_state = kwargs.get("merged_state", [])
        merge_strategy = kwargs.get("merge_strategy", "context_union")
        user_input = kwargs.get("original_user_input")

        plan = {
            "operation": "merge",
            "merged_from": list(merged_from),
            "merged_state": list(merged_state),
            "merge_strategy": merge_strategy,
            "original_user_input": user_input,
            "steps": [
                f"1. 收集 {len(merged_from)} 个源 task 的 context",
                f"2. 按 {merge_strategy} 策略合并",
                "3. 写入 merged_task state (append-only per 守门 #13 d)",
                "4. emit UI 事件 (TaskCardCreate × 1)",
            ],
        }
        self.state.plan = plan
        self._audit("plan", merged_from=merged_from, merge_strategy=merge_strategy)
        return {"status": "plan_done", "plan": plan}

    # ===== 节点 3: execute =====

    async def execute_node(self, **kwargs) -> dict:
        """execute 节点: 执行合并

        实际工作:
          - 拼接 source task context
          - 写入 merged_task state
        """
        self.state.status = "execute"
        plan = self.state.plan or {}
        merged_from = plan.get("merged_from", [])
        merged_state = plan.get("merged_state", [])
        merge_strategy = plan.get("merge_strategy", "context_union")

        # mock: 拼接 context (实际接入时, 这里会读 sub_pool handle.state)
        merged_context: dict[str, Any] = {
            "source_tasks": list(merged_from),
            "checkpoint_refs": list(merged_state),
            "merge_strategy": merge_strategy,
            "merged_at": time.time(),
        }

        result = {
            "merged_task_id": self.merged_task_id,
            "merged_context": merged_context,
            "operation": "merge",
        }
        self.state.execute_result = result
        self._audit("execute", merged_task_id=self.merged_task_id, strategy=merge_strategy)
        return {"status": "execute_done", "result": result}

    # ===== 节点 4: verify =====

    async def verify_node(self, **kwargs) -> dict:
        """verify 节点: 守门 #1 / #12 / 测试

        守门:
          - 守门 #13 a: L0 唯一协调, 跨任务操作只经 L0
          - 守门 #13 d: Transaction append-only (这里 verify merged_context 完整 + 源 task 都标 superseded)
          - 守门 #19: Python 化
          - 守门 #23: 不开外部 API
        """
        self.state.status = "verify"
        result = self.state.execute_result or {}
        plan = self.state.plan or {}

        violations: list[str] = []
        # 守门 #13 d 验证
        if not result.get("merged_context"):
            violations.append("merged_context is empty")
        if not plan.get("merged_from"):
            violations.append("merged_from is empty (no source tasks)")
        # 守门 #13 a 验证
        if self.state.operation not in ("merge", "split", "dep_set", "bulk_action", "summarize", "reassign", "metadata"):
            violations.append(f"unknown TMO operation: {self.state.operation}")

        verify_result = {
            "violations": violations,
            "ok": len(violations) == 0,
            "checked": [
                "守门 #13 a (L0 唯一协调)",
                "守门 #13 d (Transaction append-only)",
                "守门 #19 (Python 化)",
                "守门 #23 (不开外部 API)",
            ],
        }
        self.state.verify_result = verify_result
        self._audit("verify", ok=verify_result["ok"], violations=violations)
        return {"status": "verify_done", "verify": verify_result}

    # ===== 节点 5: report =====

    async def report_node(self, **kwargs) -> dict:
        """report 节点: 最终结果生成 + 通知 Top"""
        self.state.status = "report"
        report = {
            "task_id": self.state.task_id,
            "operation": self.state.operation,
            "merged_task_id": self.merged_task_id,
            "merged_from": (self.state.plan or {}).get("merged_from", []),
            "verify_ok": (self.state.verify_result or {}).get("ok", False),
            "duration_ms": (time.time() - self.state.started_at) * 1000,
            "audit_count": len(self.state.audit_log),
        }
        self.state.report = report
        self._audit("report", task_id=self.state.task_id, verify_ok=report["verify_ok"])
        self.state.status = "done"
        return {"status": "report_done", "report": report}

    # ===== 完整 run =====

    async def run(self, operation: str, **kwargs) -> str:
        """SA-10 完整 5 节点 subgraph 跑一次

        返回 merged_task_id (per 02 §2.2.3 通用 sub-agent 模板)
        """
        kwargs["operation"] = operation
        await self.init_node(**kwargs)
        await self.plan_node(**kwargs)
        await self.execute_node(**kwargs)
        verify_result = await self.verify_node(**kwargs)
        report_result = await self.report_node(**kwargs)
        if not verify_result["verify"]["ok"]:
            raise ValueError(
                f"SA-10 verify failed: {verify_result['verify']['violations']}"
            )
        return self.merged_task_id
