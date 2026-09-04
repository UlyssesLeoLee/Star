# scripts/automation/task_ops/nodes/summarize_node.py
# TMO M-N5 summarize_node (per docs/architecture/2026-09-03-langgraph/03-detailed-design.md v0.2 §3.2.1.1)
#
# 职责: 跨 N 个 task 汇总 (LLM 生成 summary, token_usage, status)
# 方向: L0 → UI (per 02-basic-design.md §2.6.2)
# 协议: SummarizeResult TypedDict (per protocols.py)
#
# 流程:
#   1. validate: target_task_ids 非空 + 全部存在
#   2. 遍历每个 task, 读 L1 状态 (status / context / checkpoints)
#   3. 调用 LLM (mock fallback) 生成 summary (per 守门 #5 mock 备选, 9/3 11:35 拍板 A)
#   4. 收集 token_usage (input / output / total)
#   5. ui_streamer.push × N (N × TaskCardUpdate summary)
#
# 守门 (per AGENTS.md §4):
#   - 守门 #13 a: L0 唯一协调入口 (跨任务汇总只经 L0)
#   - 守门 #13 d: token_usage 记账 (Transaction append-only)
#   - 守门 #19: Python 化, 不写 .rs
#   - 守门 #22: 调试控制台走 port 8080 console_server.py, 不污染 main 编译链
#   - 守门 #23: AI 修改 mock, 不开 OpenAI/Anthropic API
#
# 派发 (per 守门 #20): 实装前必先 brief 落档 (TMO-05 父会话 Mavis 委托)

from __future__ import annotations

import logging
import time
from typing import Any, Dict, List, Optional, Sequence

logger = logging.getLogger("task_ops.nodes.summarize_node")


# ===== 常量 (per 03 §3.2.1.1 + 02 §2.6.4) =====

MIN_SUMMARIZE_TARGETS: int = 1
MAX_SUMMARIZE_TARGETS: int = 50
"""M-N5 summarize 批量上限 (per 03 §3.2.1.1 注释): 防止一次性汇总过多.

守门: 防止 O(N) LLM 阻塞 UI / 一次性 token 爆量.
"""

DEFAULT_SUMMARIZE_PROMPT_TEMPLATE: str = (
    "请用 1-2 句中文总结以下任务执行情况: "
    "task_id={task_id} task_type={task_type} status={status} "
    "context_keys={context_keys}"
)
"""M-N5 mock 备选 LLM prompt 模板 (per 守门 #5 + 守门 #23).

实装 LLM 后此模板用作 fallback / 调试对照.
"""


# ===== Mock LLM 摘要生成 (per 守门 #5 + 守门 #23) =====

def _mock_llm_summarize(
    task_id: str,
    task_type: str,
    status: str,
    context_keys: Sequence[str],
) -> tuple[str, dict]:
    """Mock LLM 摘要生成 (per 守门 #5 9/3 11:35 拍板 A).

    不调用 OpenAI / Anthropic / 任何外部 API.
    返回 (summary_text, token_usage).

    守门:
      - 守门 #5: 不读环境变量内容, 不泄露 secret
      - 守门 #23: AI mock 模式, 永远 confidence < 0.5
    """
    # 守门 #23: mock 永远不调外部 API
    # 守门 #19: 不读 .env, 不读 $env:*, 不泄露 secret
    context_keys_str = ",".join(sorted(context_keys)) if context_keys else "(empty)"
    summary = (
        f"任务 {task_id} (类型 {task_type}) 当前状态 {status}, "
        f"已记录 context 字段: {context_keys_str}. "
        f"由 mock 模板生成 (per 守门 #5 9/3 11:35 拍板 A + 守门 #23 mock 模式)."
    )
    # 估算 token_usage (简化: 字符数 / 4)
    input_chars = len(DEFAULT_SUMMARIZE_PROMPT_TEMPLATE.format(
        task_id=task_id, task_type=task_type, status=status, context_keys=context_keys_str
    ))
    output_chars = len(summary)
    token_usage = {
        "input": max(1, input_chars // 4),
        "output": max(1, output_chars // 4),
        "total": max(2, (input_chars + output_chars) // 4),
    }
    return summary, token_usage


# ===== 辅助函数 =====

def _validate_summarize_request(
    target_task_ids: Sequence[str],
    sub_pool,
) -> None:
    """M-N5 步骤 1: validate

    守门:
      - target_task_ids 非空
      - len(target_task_ids) ∈ [MIN_SUMMARIZE_TARGETS, MAX_SUMMARIZE_TARGETS]
      - 每个 task_id 存在
    """
    if not target_task_ids:
        raise ValueError("summarize_node: target_task_ids is required (non-empty)")

    if len(target_task_ids) > MAX_SUMMARIZE_TARGETS:
        raise ValueError(
            f"summarize_node: target_task_ids count {len(target_task_ids)} > MAX {MAX_SUMMARIZE_TARGETS}"
        )

    for tid in target_task_ids:
        try:
            sub_pool.get(tid)
        except KeyError:
            raise ValueError(
                f"summarize_node: target_task_id {tid!r} not found in sub_pool"
            )


def _collect_task_state(
    target_task_ids: Sequence[str],
    sub_pool,
) -> List[dict]:
    """M-N5 步骤 2: 遍历每个 task, 读 L1 状态.

    Returns:
        list of {task_id, task_type, status, context, checkpoint_count}
    """
    states: List[dict] = []
    for tid in target_task_ids:
        handle = sub_pool.get(tid)
        states.append({
            "task_id": tid,
            "task_type": handle.task_type,
            "status": handle.state.get("status", "pending"),
            "context_keys": sorted((handle.state.get("context") or {}).keys()),
            "checkpoint_count": len(handle.checkpoints),
        })
    return states


def _generate_summaries(
    task_states: Sequence[dict],
) -> List[dict]:
    """M-N5 步骤 3+4: 调 LLM (mock) 生成 summary + 收集 token_usage.

    守门 #5 + 守门 #23: mock 模式, 不开外部 API.

    Returns:
        list of TaskSummary TypedDict
    """
    summaries: List[dict] = []
    for ts in task_states:
        summary, token_usage = _mock_llm_summarize(
            task_id=ts["task_id"],
            task_type=ts["task_type"],
            status=ts["status"],
            context_keys=ts["context_keys"],
        )
        summaries.append({
            "task_id": ts["task_id"],
            "task_type": ts["task_type"],
            "status": ts["status"],
            "summary": summary,
            "token_usage": token_usage,
        })
    return summaries


def _emit_ui_events(
    target_task_ids: Sequence[str],
    summaries: Sequence[dict],
) -> List[dict]:
    """M-N5 步骤 5: emit UI events (N × TaskCardUpdate summary 字段)

    守门 #24: 调试控制台走 subprocess, 不直接 RPC.
    本步骤 emit 事件到 mock UI stream.
    """
    events: List[dict] = []
    for s in summaries:
        events.append({
            "type": "TaskCardUpdate",
            "task_id": s["task_id"],
            "patch": {
                "summary": s["summary"],
                "summary_token_usage": s["token_usage"],
            },
        })
    logger.info(
        "summarize_node emit_ui_events: %d events (1 update per task)", len(events),
    )
    return events


# ===== 主函数: summarize_node =====

async def summarize_node(state: dict, manager) -> dict:
    """TMO M-N5: 跨 N 个 task 汇总 (per 03 §3.2.1.1)

    输入 (state = SummarizeResult TypedDict, per protocols.py):
      operation: "summarize"
      target_task_ids: 至少 1 个 task_id (守门 ≤ MAX_SUMMARIZE_TARGETS)
      actor_session_id: 发起者 session_id (L0 chat bar)

    输出 (TopAgentState 增量更新, per 03 §3.2.1.1):
      superseded_tasks: [] (无副作用)
      active_tmo_operation: None
      global_context: {last_tmo_result: {operation, target_task_ids, task_summaries, ...}}
      ui_events: N 个 TaskCardUpdate (summary 字段)
      task_summaries: list[TaskSummary]
      total_token_usage: {input, output, total} (聚合)

    守门:
      - 守门 #13 a: L0 唯一入口, 跨任务汇总只经 L0
      - 守门 #13 d: token_usage 记账 (Transaction append-only)
      - 守门 #19: Python 化, 不写 .rs
      - 守门 #23: AI mock 模式, 不开 OpenAI/Anthropic API
    """
    target_task_ids: List[str] = list(state.get("target_task_ids") or [])
    actor_session_id: Optional[str] = state.get("actor_session_id")

    logger.info(
        "summarize_node start: targets=%d actor=%s",
        len(target_task_ids), actor_session_id,
    )

    sub_pool = manager.sub_pool

    # 步骤 1: validate
    _validate_summarize_request(target_task_ids, sub_pool)

    # 步骤 2: collect task state
    task_states = _collect_task_state(target_task_ids, sub_pool)

    # 步骤 3+4: generate summaries (mock LLM) + collect token_usage
    started = time.time()
    task_summaries = _generate_summaries(task_states)

    # 步骤 5: emit UI events
    ui_events = _emit_ui_events(target_task_ids, task_summaries)

    # 聚合 token_usage
    total_token_usage = {
        "input": sum(s["token_usage"]["input"] for s in task_summaries),
        "output": sum(s["token_usage"]["output"] for s in task_summaries),
        "total": sum(s["token_usage"]["total"] for s in task_summaries),
    }

    duration_ms = (time.time() - started) * 1000

    result: dict = {
        "operation": "summarize",
        "target_task_ids": list(target_task_ids),
        "task_summaries": task_summaries,
        "total_token_usage": total_token_usage,
        "ui_events": ui_events,
        "active_tmo_operation": None,
        "duration_ms": round(duration_ms, 2),
    }
    logger.info(
        "summarize_node done: %d summaries, %d total tokens, %.2fms",
        len(task_summaries), total_token_usage["total"], duration_ms,
    )
    return result
