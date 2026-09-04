# scripts/automation/task_ops/nodes/summarize_node.py
# M-N5 summarize_node (TMO-05, per 02-basic-design.md v0.2 §2.6)
#
# 职责:
#   - 汇总 N 个 task 的结果 + context 到 1 个 summary
#   - 守门 #13 a: L0 唯一汇总入口
#   - 守门 #13 d: 汇总后 task 不 supersede, summary 单独存 (Work 类型, 短 TTL)
#   - 守门 #19: Python 化
#
# 输入: SummarizeRequest (per protocols.py)
# 输出: SummarizeResponse (summary + per_task_highlights + TTL)
#
# 阻塞 (per G-DEP-02): P1 工具 (search_code / get_symbol / find_references / get_code_context)
# 真实 context 汇总需 code search 集成, PoC stub 用 mock context aggregation

from __future__ import annotations

import logging
import time
import uuid
from typing import Optional

logger = logging.getLogger("task_ops.summarize")

# 汇总策略 (per 02 §2.6.7)
SUMMARIZE_STRATEGIES = ("concatenate", "deduplicate", "extract_keywords")


async def summarize_node(
    sub_agent_pool,  # SubAgentPool 实例 (L0 唯一入口)
    request: dict,
) -> dict:
    """M-N5 summarize_node stub (per TMO-05 7 子项实装 phase)

    最小可行骨架 (per V2-6 5 子代理 + Mavis 跨域协调模式):
      1. 验证 task_ids 至少 1 个
      2. 收集每个 task 的 context + last_state
      3. 简单 concatenation / keyword extraction (per strategy)
      4. 返回 summary + per_task_highlights + TTL (Work 类型 短 TTL per 守门 #13 d)
    """
    task_ids = request.get("task_ids", [])
    summarize_strategy = request.get("summarize_strategy", "concatenate")
    actor_session_id = request.get("actor_session_id")

    if not task_ids or len(task_ids) < 1:
        raise ValueError("summarize_request: task_ids required (>= 1)")
    if summarize_strategy not in SUMMARIZE_STRATEGIES:
        raise ValueError(f"summarize_request: summarize_strategy must be one of {SUMMARIZE_STRATEGIES}")

    # 收集每个 task 的 context
    per_task_highlights = []
    aggregated_context = {}
    for tid in task_ids:
        try:
            task = sub_agent_pool.get(tid)
            state = task.state
            ctx = state.get("context", {})
            per_task_highlights.append({
                "task_id": tid,
                "task_type": task.task_type,
                "status": state.get("status", "unknown"),
                "key": f"{tid}:{state.get('status', 'unknown')}",
            })
            # 简单 context merge
            for k, v in ctx.items():
                if k not in aggregated_context:
                    aggregated_context[k] = v
        except KeyError:
            # task 不存在, 跳过 (per 守门 #13 a L0 容错)
            per_task_highlights.append({"task_id": tid, "error": "not_found"})

    # 简单 summary (mock)
    if summarize_strategy == "concatenate":
        summary = f"汇总 {len(task_ids)} 个 task: " + ", ".join(h.get("task_id", "?") for h in per_task_highlights)
    elif summarize_strategy == "deduplicate":
        unique = set(h.get("task_type", "?") for h in per_task_highlights)
        summary = f"汇总 {len(task_ids)} 个 task ({len(unique)} 类型去重)"
    else:  # "extract_keywords"
        keywords = list(aggregated_context.keys())[:10]
        summary = f"关键词: {', '.join(keywords)}"

    # Work 类型 短 TTL (per 守门 #13 d Work 100% retention: 短 TTL)
    ttl_seconds = 3600  # 1 小时

    summary_id = f"summary-{uuid.uuid4().hex[:8]}"

    logger.info(f"M-N5 summarize_node: {len(task_ids)} tasks -> {summary_id}")

    return {
        "operation": "summarize",
        "summary_id": summary_id,
        "summary": summary,
        "per_task_highlights": per_task_highlights,
        "aggregated_context": aggregated_context,
        "summarize_strategy": summarize_strategy,
        "ttl_seconds": ttl_seconds,
        "actor_session_id": actor_session_id,
        "completed_at_ms": int(time.time() * 1000),
    }
