#!/usr/bin/env python3
# _generate_uat_s01_s05.py - UAT 业务场景 1-5 fixture generator
# Author: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #10 + 19:39 JST 授权)
# 触发: 2026-09-07 14:30 JST user 发令 "补充更新 playwright 测试脚本, 专门增设 UAT 测试的 mock 项目内容以及配套文档"
# 守门: #13 W/T/M 分类 (Master SCD Type 2 / Transaction append-only / Work 短 TTL) + #19 Python 化

from __future__ import annotations

import json
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[5]
MOCK_DATA = REPO_ROOT / "tools" / "star-flash-mock" / "mock_data" / "uat" / "scenarios"


def _write(scenario_id: str, scenario_name: str, wt_m_class: str, files: dict[str, str]) -> None:
    """写入一个场景的 5 个文件."""
    s_dir = MOCK_DATA / scenario_id
    s_dir.mkdir(parents=True, exist_ok=True)
    for fname, content in files.items():
        path = s_dir / fname
        path.write_text(content, encoding="utf-8")


def gen_s01_workitem_create() -> None:
    """S01: WorkItem 创建 — Transaction (append-only, 业务事件)."""
    s_id = "S01-workitem-create"
    _write(
        s_id,
        "WorkItem 创建",
        "Transaction",
        {
            "request.json": json.dumps(
                {
                    "fixture_version": "v1",
                    "scenario": s_id,
                    "class": "Transaction",
                    "method": "POST",
                    "endpoint": "/api/mcp/workitem-create",
                    "request": {
                        "title": "[UAT-S01] 5 域 cross-domain 跨域反馈",
                        "description": "player → social → admin 跨域",
                        "domain": "social",
                        "actor_session_id": "session-2026-09-07-001",
                    },
                },
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "expected_response.json": json.dumps(
                {
                    "fixture_version": "v1",
                    "scenario": s_id,
                    "class": "Transaction",
                    "response_201": {
                        "id": "wi-uat-2026-09-07-001",
                        "title": "[UAT-S01] 5 域 cross-domain 跨域反馈",
                        "status": "open",
                        "domain": "social",
                        "created_at": "2026-09-07T15:00:00Z",
                        "creator": "session-2026-09-07-001",
                        "audit_logged": True,
                    },
                },
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "expected_db_state.json": json.dumps(
                {
                    "fixture_version": "v1",
                    "scenario": s_id,
                    "class": "Transaction",
                    "tables": {
                        "transaction.workitem": {"insert_count": 1, "append_only": True, "audit_required": True},
                        "transaction.workitem_event": {"insert_count": 1, "event_type": "workitem.created"},
                    },
                },
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "expected_events.json": json.dumps(
                {
                    "fixture_version": "v1",
                    "scenario": s_id,
                    "events": [
                        {
                            "event_type": "workitem.created",
                            "event_id": "evt-2026-09-07-001",
                            "actor_session_id": "session-2026-09-07-001",
                            "timestamp": "2026-09-07T15:00:00Z",
                        }
                    ],
                },
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "ac_mapping.md": (
                "# S01 AC 跨引\n\n"
                "- AC: docs/test-design.md v0.8 §6.1 MVP 测试矩阵 AC-WI-001 (WorkItem 创建)\n"
                "- 跨引: docs/uat-design.md §2 AC-UAT-001 (UAT 件套 1 + WorkItem 流程)\n"
                "- 守门: #13 d Transaction append-only (audit 100%)\n"
            ),
        },
    )


def gen_s02_worktree_create() -> None:
    """S02: Worktree 创建 — Transaction + Master (worktree 是 Master SCD Type 2)."""
    s_id = "S02-worktree-create"
    _write(
        s_id,
        "Worktree 创建",
        "Master",
        {
            "request.json": json.dumps(
                {
                    "fixture_version": "v1",
                    "scenario": s_id,
                    "class": "Master",
                    "method": "POST",
                    "endpoint": "/api/worktrees",
                    "request": {
                        "project_id": "proj-uat-001",
                        "branch": "feat/uat-s02",
                        "work_item_id": "wi-uat-2026-09-07-001",
                        "actor_session_id": "session-2026-09-07-001",
                    },
                },
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "expected_response.json": json.dumps(
                {
                    "fixture_version": "v1",
                    "scenario": s_id,
                    "class": "Master",
                    "response_201": {
                        "id": "wt-uat-2026-09-07-001",
                        "project_id": "proj-uat-001",
                        "branch": "feat/uat-s02",
                        "work_item_id": "wi-uat-2026-09-07-001",
                        "status": "active",
                        "scd_version": 1,
                        "valid_from": "2026-09-07T15:00:00Z",
                        "valid_to": None,
                        "rls_13_classes_attached": True,
                    },
                },
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "expected_db_state.json": json.dumps(
                {
                    "fixture_version": "v1",
                    "scenario": s_id,
                    "class": "Master",
                    "tables": {
                        "master.worktree": {"insert_count": 1, "scd_type": 2, "physical_delete_forbidden": True},
                        "transaction.worktree_event": {"insert_count": 1, "event_type": "worktree.created"},
                    },
                },
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "expected_events.json": json.dumps(
                {
                    "fixture_version": "v1",
                    "scenario": s_id,
                    "events": [
                        {
                            "event_type": "worktree.created",
                            "event_id": "evt-2026-09-07-002",
                            "worktree_id": "wt-uat-2026-09-07-001",
                            "timestamp": "2026-09-07T15:00:00Z",
                        }
                    ],
                },
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "ac_mapping.md": (
                "# S02 AC 跨引\n\n"
                "- AC: docs/test-design.md v0.8 §6.1 MVP AC-WT-001 (Worktree 创建)\n"
                "- 跨引: docs/uat-design.md §2 AC-UAT-002 (UAT 件套 1 + Worktree 流程)\n"
                "- 守门: #13 c Master SCD Type 2 + RLS 13 類必携\n"
            ),
        },
    )


def gen_s03_agent_running() -> None:
    """S03: Agent 运行 — Transaction (状态机事件流)."""
    s_id = "S03-agent-running"
    _write(
        s_id,
        "Agent 运行",
        "Transaction",
        {
            "request.json": json.dumps(
                {
                    "fixture_version": "v1",
                    "scenario": s_id,
                    "class": "Transaction",
                    "method": "POST",
                    "endpoint": "/api/agents/{id}/run",
                    "request": {
                        "agent_id": "ag-uat-001",
                        "worktree_id": "wt-uat-2026-09-07-001",
                        "actor_session_id": "session-2026-09-07-001",
                    },
                },
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "expected_response.json": json.dumps(
                {
                    "fixture_version": "v1",
                    "scenario": s_id,
                    "class": "Transaction",
                    "response_200": {
                        "agent_session_id": "ag-session-uat-2026-09-07-001",
                        "status": "running",
                        "started_at": "2026-09-07T15:00:05Z",
                        "worktree_id": "wt-uat-2026-09-07-001",
                    },
                },
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "expected_db_state.json": json.dumps(
                {
                    "fixture_version": "v1",
                    "scenario": s_id,
                    "class": "Transaction",
                    "tables": {
                        "transaction.agent_session": {"insert_count": 1, "append_only": True},
                        "transaction.agent_session_event": {"insert_count": 2, "events": ["agent.started", "agent.running"]},
                    },
                },
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "expected_events.json": json.dumps(
                {
                    "fixture_version": "v1",
                    "scenario": s_id,
                    "events": [
                        {"event_type": "agent.started", "event_id": "evt-2026-09-07-003"},
                        {"event_type": "agent.running", "event_id": "evt-2026-09-07-004"},
                    ],
                },
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "ac_mapping.md": (
                "# S03 AC 跨引\n\n"
                "- AC: docs/test-design.md v0.8 §6.1 MVP AC-AG-001 (Agent 启动)\n"
                "- 跨引: docs/uat-design.md §2 AC-UAT-003\n"
                "- 守门: #13 d Transaction append-only\n"
            ),
        },
    )


def gen_s04_feedback_loop() -> None:
    """S04: 5 域 Feedback 环 — Transaction (跨域事件流)."""
    s_id = "S04-feedback-loop"
    _write(
        s_id,
        "5 域 Feedback 环",
        "Transaction",
        {
            "request.json": json.dumps(
                {
                    "fixture_version": "v1",
                    "scenario": s_id,
                    "class": "Transaction",
                    "method": "POST",
                    "endpoint": "/api/feedback",
                    "request": {
                        "from_domain": "player",
                        "to_domain": "social",
                        "worktree_id": "wt-uat-2026-09-07-001",
                        "feedback_type": "code_review",
                        "actor_session_id": "session-2026-09-07-001",
                    },
                },
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "expected_response.json": json.dumps(
                {
                    "fixture_version": "v1",
                    "scenario": s_id,
                    "class": "Transaction",
                    "response_201": {
                        "feedback_id": "fb-uat-2026-09-07-001",
                        "from_domain": "player",
                        "to_domain": "social",
                        "status": "received",
                        "racy": "R(玩家域 Lead) + A(玩家域 Lead) + C(社会域 Lead) + I(其他域 Lead)",
                        "raci_complete": True,
                    },
                },
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "expected_db_state.json": json.dumps(
                {
                    "fixture_version": "v1",
                    "scenario": s_id,
                    "class": "Transaction",
                    "tables": {
                        "transaction.feedback": {"insert_count": 1, "audit_required": True},
                        "transaction.feedback_event": {"insert_count": 1, "cross_domain": True},
                    },
                },
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "expected_events.json": json.dumps(
                {
                    "fixture_version": "v1",
                    "scenario": s_id,
                    "events": [
                        {
                            "event_type": "feedback.cross_domain",
                            "from": "player",
                            "to": "social",
                            "event_id": "evt-2026-09-07-005",
                        }
                    ],
                },
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "ac_mapping.md": (
                "# S04 AC 跨引\n\n"
                "- AC: docs/test-design.md v0.8 §7 S2-S5 (5 域跨域 Feedback)\n"
                "- 跨引: docs/uat-design.md §2 AC-UAT-004 + 守门 #14 v2 5 域 Lead RACI 4 维\n"
                "- 守门: #13 d + #14 v2 RACI R+A+C 完整\n"
            ),
        },
    )


def gen_s05_validation_pass() -> None:
    """S05: 验证通过 — Work (短 TTL, 验证任务结果)."""
    s_id = "S05-validation-pass"
    _write(
        s_id,
        "验证通过",
        "Work",
        {
            "request.json": json.dumps(
                {
                    "fixture_version": "v1",
                    "scenario": s_id,
                    "class": "Work",
                    "method": "POST",
                    "endpoint": "/api/validation",
                    "request": {
                        "worktree_id": "wt-uat-2026-09-07-001",
                        "validation_type": "ci",
                        "actor_session_id": "session-2026-09-07-001",
                    },
                },
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "expected_response.json": json.dumps(
                {
                    "fixture_version": "v1",
                    "scenario": s_id,
                    "class": "Work",
                    "response_200": {
                        "validation_id": "val-uat-2026-09-07-001",
                        "result": "pass",
                        "retention_period_seconds": 86400,
                        "ttl": "short",
                    },
                },
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "expected_db_state.json": json.dumps(
                {
                    "fixture_version": "v1",
                    "scenario": s_id,
                    "class": "Work",
                    "tables": {
                        "work.validation_result": {"insert_count": 1, "retention_period": "1d", "physical_delete_after_ttl": True},
                    },
                },
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "expected_events.json": json.dumps(
                {
                    "fixture_version": "v1",
                    "scenario": s_id,
                    "events": [{"event_type": "validation.passed", "event_id": "evt-2026-09-07-006"}],
                },
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "ac_mapping.md": (
                "# S05 AC 跨引\n\n"
                "- AC: docs/test-design.md v0.8 §6.3 VAL-001 (CI 验证通过)\n"
                "- 跨引: docs/uat-design.md §2 AC-UAT-005\n"
                "- 守门: #13 a Work 短 TTL + タイマー失効\n"
            ),
        },
    )


def main():
    """主入口: S01-S05 fixture 生成."""
    MOCK_DATA.mkdir(parents=True, exist_ok=True)
    gen_s01_workitem_create()
    gen_s02_worktree_create()
    gen_s03_agent_running()
    gen_s04_feedback_loop()
    gen_s05_validation_pass()
    print("S01-S05 fixtures generated (5 scenarios × 5 files = 25 files)")


if __name__ == "__main__":
    main()
