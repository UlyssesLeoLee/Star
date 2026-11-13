# scripts/automation/task_ops/spawn_subtask/cli.py
# spawn_subtask subprocess 入口 (per 守门 #19 v19 agent 交互 Python 化 + 守门 #24 v2
# subprocess 替代 RPC: console_server.py → subprocess.run 本模块)
#
# 用法:
#   python -m scripts.automation.task_ops.spawn_subtask.cli spawn_subtask \
#     --parent-task-id <id> --child-sa-type SA-04 --child-name <name> \
#     [--bound-by Ulysses] [--spawned-via api] [--no-quota-check] \
#     [--initial-context-json '{}']
#
# 输出: 单行 JSON (SpawnSubtaskResult.to_dict()), exit 0 = spawned, exit 2 = failed.
# (设计文档的入口为 scripts/automation/task_ops.py spawn_subtask; dev 分支无该
# 文件 (per TASK 前置注意), 入口落在 task_ops 包内本模块, 报告已列明.)

from __future__ import annotations

import argparse
import asyncio
import json
import sys
from typing import List, Optional

from scripts.automation.task_ops.spawn_subtask.models import SpawnSubtaskRequest, TopAgentState
from scripts.automation.task_ops.spawn_subtask.spawn_subtask_node import spawn_subtask_node


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="spawn_subtask",
        description="M-N8 spawn_subtask: 1 派生 1 创建绑定子任务 (WBS 6.3)",
    )
    parser.add_argument(
        "command",
        nargs="?",
        default="spawn_subtask",
        choices=["spawn_subtask"],
        help="子命令 (固定 spawn_subtask, per task_ops.py spawn_subtask 入口形态)",
    )
    parser.add_argument("--parent-task-id", required=True, help="父 task ID")
    parser.add_argument(
        "--child-sa-type",
        required=True,
        help="child SA 类型 (SA-01..SA-09; SA-10 自嵌套禁止)",
    )
    parser.add_argument("--child-name", required=True, help="子任务名 (1..200 chars)")
    parser.add_argument(
        "--bound-by", default="Ulysses", help="绑定创建者 (per 守门 #10 代签)"
    )
    parser.add_argument(
        "--spawned-via",
        default="api",
        choices=["ui_rightclick", "l0_chat", "api"],
        help="入口标注",
    )
    parser.add_argument(
        "--no-quota-check", action="store_true", help="跳过 quota 校验 (admin override)"
    )
    parser.add_argument(
        "--initial-context-json", default="{}", help="父 context 浅拷贝 (JSON)"
    )
    return parser


def run(args: argparse.Namespace) -> int:
    """构建请求 → 执行 M-N8 节点 → 打印 JSON 结果, 返回 exit code."""
    try:
        initial_context = json.loads(args.initial_context_json)
        if not isinstance(initial_context, dict):
            raise ValueError("--initial-context-json 必须是 JSON object")
        req = SpawnSubtaskRequest(
            parent_task_id=args.parent_task_id,
            child_sa_type=args.child_sa_type,
            child_name=args.child_name,
            initial_context=initial_context,
            quota_check=not args.no_quota_check,
            bound_by=args.bound_by,
            spawned_via=args.spawned_via,
        )
    except (ValueError, json.JSONDecodeError) as e:
        print(json.dumps({
            "status": "failed",
            "failure_reason": f"InvalidRequest: {e}",
        }, ensure_ascii=False))
        return 2

    state: TopAgentState = {"active_tmo_operation": {"request": req}}
    result = asyncio.run(spawn_subtask_node(state))["last_spawn_subtask_result"]
    print(json.dumps(result.to_dict(), ensure_ascii=False))
    return 0 if result.status == "spawned" else 2


def main(argv: Optional[List[str]] = None) -> int:
    """CLI 入口 (argv=None → sys.argv[1:])."""
    parser = _build_parser()
    args = parser.parse_args(argv)
    return run(args)


if __name__ == "__main__":
    sys.exit(main())
