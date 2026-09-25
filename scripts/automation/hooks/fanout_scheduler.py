"""FS-3: Fan-out Scheduler — per-event 多 hook 調度 + 串行/並行 + 優先級排序 (per BD §4.4 + DD §2.3).

调度算法 (per SRS §FR-2.3 + §FR-4.2):
    1. 從 HR-2.list_by_event(event.event_type) 獲取 hook 列表
    2. 按 priority 升序分組
    3. 同 priority 串行或並行 (per parallel 標誌)
    4. 對每個 hook 調用 HR-4.run()
    5. 任一 BLOCK/ASK → 立即停止
    6. 累積 transform
    7. 聚合最終決策

决策优先级 (per BD §4.4): BLOCK (1) > ASK (2) > WARN (3) > PASS (4), transform 累积不参与优先级.
"""
# SPDX-License-Identifier: MIT OR Apache-2.0

from __future__ import annotations

import logging
from collections import defaultdict
from concurrent.futures import ThreadPoolExecutor, as_completed
from typing import Any, Dict, List, Optional


logger = logging.getLogger(__name__)


# 决策优先级 (per BD §4.4)
DECISION_PRIORITY: Dict[str, int] = {
    "BLOCK": 1,
    "ASK": 2,
    "WARN": 3,
    "PASS": 4,
    "transform": 99,  # 不参与决策优先级, 累积 transformed_args
}


class FanoutScheduler:
    """Fan-out Scheduler 主類 (per BD §4.4 + DD §2.3)."""

    def __init__(
        self,
        registry: "HookRegistry",
        runner: "HookRunner",
        max_workers: int = 4,
    ):
        self._registry = registry
        self._runner = runner
        self._executor = ThreadPoolExecutor(
            max_workers=max_workers,
            thread_name_prefix="hook-fanout",
        )

    def fanout(self, event: "Event") -> "EventResult":
        """per-event fan-out 调度.

        调度算法 (per BD §4.4):
        1. 從 HR-2.list_by_event(event.event_type) 獲取 hook 列表
        2. 按 priority 升序分組
        3. 同 priority 串行或並行 (per parallel 標誌)
        4. 對每個 hook 調用 HR-4.run()
        5. 任一 BLOCK/ASK → 立即停止
        6. 累積 transform
        7. 聚合最終決策
        """
        hooks = self._registry.list_by_event(event.event_type)
        if not hooks:
            return EventResult(decision="PASS", reason="no hooks matched")

        # 按 priority 分組
        priority_groups: Dict[int, List["Hook"]] = defaultdict(list)
        for hook in hooks:
            priority_groups[hook.priority].append(hook)

        aggregated_decision = "PASS"
        aggregated_reason = "no decisions"
        transformed_args: Optional[Dict[str, Any]] = None
        hooks_executed: List[str] = []
        total_latency = 0.0

        # 按 priority 升序遍历
        for priority in sorted(priority_groups.keys()):
            group = priority_groups[priority]

            # 同 priority 串行或並行 (per parallel 標誌)
            if len(group) == 1 or not any(h.parallel for h in group):
                # 串行
                results = [self._runner.run(h, event) for h in group]
            else:
                # 並行 (max_workers 上限 per SRS §FR-4.3)
                futures = {
                    self._executor.submit(self._runner.run, h, event): h
                    for h in group
                }
                results_by_hook: Dict[str, "HookResult"] = {}
                for future in as_completed(futures):
                    hook = futures[future]
                    try:
                        results_by_hook[hook.name] = future.result()
                    except Exception as e:
                        # 並行单 hook 异常 (per SRS §NFR-A-3): 视為 WARN
                        logger.error(
                            "并行 hook %s 拋异常: %s",
                            hook.name, e,
                        )
                        results_by_hook[hook.name] = HookResult(
                            decision="WARN",
                            reason=f"parallel exception: {type(e).__name__}: {e}",
                            rule_id=hook.name,
                            latency_ms=0.0,
                        )
                # 按 group 顺序保留 (per 文档)
                results = [results_by_hook[h.name] for h in group]

            # 处理结果
            for hook, result in zip(group, results):
                hooks_executed.append(hook.name)
                total_latency += result.latency_ms

                # 累积 transform (per SRS §FR-4.4: 后覆盖前)
                if result.decision == "transform" and result.transformed_args:
                    transformed_args = result.transformed_args

                # 聚合决策
                cur_prio = DECISION_PRIORITY.get(result.decision, 99)
                agg_prio = DECISION_PRIORITY.get(aggregated_decision, 99)
                if cur_prio < agg_prio:
                    aggregated_decision = result.decision
                    if result.reason:
                        aggregated_reason = result.reason

                # 立即停止 fan-out (per SRS §FR-2.3 BLOCK/ASK → stop)
                if result.decision in ("BLOCK", "ASK"):
                    return EventResult(
                        decision=result.decision,
                        transformed_args=transformed_args,
                        reason=result.reason or aggregated_reason,
                        latency_ms=total_latency,
                        hooks_executed=hooks_executed,
                    )

        return EventResult(
            decision=aggregated_decision,
            transformed_args=transformed_args,
            reason=aggregated_reason,
            latency_ms=total_latency,
            hooks_executed=hooks_executed,
        )

    def shutdown(self, wait: bool = True) -> None:
        """关闭线程池 (优雅停机)."""
        self._executor.shutdown(wait=wait)


# Late imports
from .event_emitter import Event, EventResult, HookResult  # noqa: E402
from .hook_registry import Hook, HookRegistry  # noqa: E402
from .hook_runner import HookRunner  # noqa: E402