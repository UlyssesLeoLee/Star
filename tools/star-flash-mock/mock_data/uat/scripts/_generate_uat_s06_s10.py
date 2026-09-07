#!/usr/bin/env python3
# _generate_uat_s06_s10.py - UAT 业务场景 6-10 fixture generator
# Author: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #10 + 19:39 JST 授权)
# 触发: 2026-09-07 14:30 JST user 发令
# 守门: #13 W/T/M 分类 + #19 Python 化

from __future__ import annotations

import json
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[5]
MOCK_DATA = REPO_ROOT / "tools" / "star-flash-mock" / "mock_data" / "uat" / "scenarios"


def _write(scenario_id: str, files: dict[str, str]) -> None:
    s_dir = MOCK_DATA / scenario_id
    s_dir.mkdir(parents=True, exist_ok=True)
    for fname, content in files.items():
        path = s_dir / fname
        path.write_text(content, encoding="utf-8")


def gen_s06_validation_fail() -> None:
    s_id = "S06-validation-fail"
    _write(
        s_id,
        {
            "request.json": json.dumps(
                {
                    "fixture_version": "v1",
                    "scenario": s_id,
                    "class": "Work",
                    "method": "POST",
                    "endpoint": "/api/validation",
                    "request": {
                        "worktree_id": "wt-uat-2026-09-07-002",
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
                        "validation_id": "val-uat-2026-09-07-002",
                        "result": "fail",
                        "failure_reasons": ["test_failed:5", "lint_error:2"],
                        "retention_period_seconds": 86400,
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
                    "tables": {"work.validation_result": {"insert_count": 1, "retention_period": "1d"}},
                },
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "expected_events.json": json.dumps(
                {"fixture_version": "v1", "scenario": s_id, "events": [{"event_type": "validation.failed", "event_id": "evt-2026-09-07-007"}]},
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "ac_mapping.md": "# S06 AC 跨引\n- AC: docs/test-design.md v0.8 §6.3 VAL-002 (CI 失败)\n- 跨引: docs/uat-design.md §2 AC-UAT-006\n- 守门: #13 a Work 短 TTL\n",
        },
    )


def gen_s07_conflict_detect() -> None:
    s_id = "S07-conflict-detect"
    _write(
        s_id,
        {
            "request.json": json.dumps(
                {
                    "fixture_version": "v1",
                    "scenario": s_id,
                    "class": "Work",
                    "method": "POST",
                    "endpoint": "/api/worktrees/{id}/conflict-detect",
                    "request": {
                        "worktree_id": "wt-uat-2026-09-07-003",
                        "target_branch": "main",
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
                        "conflict_detected": True,
                        "conflict_files": ["src/lib/store.ts", "src/mocks/handlers/agents.ts"],
                        "conflict_count": 2,
                        "retention_period_seconds": 3600,
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
                    "tables": {"work.conflict_report": {"insert_count": 1, "retention_period": "1h"}},
                },
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "expected_events.json": json.dumps(
                {"fixture_version": "v1", "scenario": s_id, "events": [{"event_type": "conflict.detected", "event_id": "evt-2026-09-07-008"}]},
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "ac_mapping.md": "# S07 AC 跨引\n- AC: docs/test-design.md v0.8 §6.1 (Worktree conflict)\n- 跨引: docs/uat-design.md §2 AC-UAT-007\n- 守门: #13 a Work 短 TTL\n",
        },
    )


def gen_s08_rebase_merge() -> None:
    s_id = "S08-rebase-merge"
    _write(
        s_id,
        {
            "request.json": json.dumps(
                {
                    "fixture_version": "v1",
                    "scenario": s_id,
                    "class": "Transaction",
                    "method": "POST",
                    "endpoint": "/api/worktrees/{id}/rebase-merge",
                    "request": {
                        "worktree_id": "wt-uat-2026-09-07-004",
                        "target_branch": "main",
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
                        "rebase_status": "ok",
                        "merge_commit": "abc123def",
                        "scd_version": 2,
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
                        "master.worktree": {"update_count": 1, "scd_version_after": 2, "physical_delete_forbidden": True},
                        "transaction.worktree_event": {"insert_count": 1, "event_type": "worktree.merged"},
                    },
                },
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "expected_events.json": json.dumps(
                {"fixture_version": "v1", "scenario": s_id, "events": [{"event_type": "worktree.merged", "event_id": "evt-2026-09-07-009"}]},
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "ac_mapping.md": "# S08 AC 跨引\n- AC: docs/test-design.md v0.8 §6.1 (rebase + merge)\n- 跨引: docs/uat-design.md §2 AC-UAT-008\n- 守门: #13 c/d Master SCD + Transaction audit\n",
        },
    )


def gen_s09_merge_request() -> None:
    s_id = "S09-merge-request"
    _write(
        s_id,
        {
            "request.json": json.dumps(
                {
                    "fixture_version": "v1",
                    "scenario": s_id,
                    "class": "Transaction",
                    "method": "POST",
                    "endpoint": "/api/merge-requests",
                    "request": {
                        "worktree_id": "wt-uat-2026-09-07-005",
                        "title": "[UAT-S09] 5 域跨域合并",
                        "reviewers": ["ag-001", "ag-002"],
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
                        "merge_request_id": "mr-uat-2026-09-07-001",
                        "status": "open",
                        "reviewers": ["ag-001", "ag-002"],
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
                        "transaction.merge_request": {"insert_count": 1, "append_only": True},
                        "transaction.merge_request_event": {"insert_count": 1, "event_type": "mr.opened"},
                    },
                },
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "expected_events.json": json.dumps(
                {"fixture_version": "v1", "scenario": s_id, "events": [{"event_type": "mr.opened", "event_id": "evt-2026-09-07-010"}]},
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "ac_mapping.md": "# S09 AC 跨引\n- AC: docs/test-design.md v0.8 §6.1 (Merge request)\n- 跨引: docs/uat-design.md §2 AC-UAT-009\n- 守门: #13 d Transaction audit\n",
        },
    )


def gen_s10_tmo_merge() -> None:
    s_id = "S10-tmo-merge"
    _write(
        s_id,
        {
            "request.json": json.dumps(
                {
                    "fixture_version": "v1",
                    "scenario": s_id,
                    "class": "Transaction",
                    "node": "M-N1",
                    "method": "POST",
                    "endpoint": "/api/tmo/merge",
                    "request": {
                        "operation": "merge",
                        "target_task_ids": ["task-alpha-7f8a9b", "task-beta-2c3d4e"],
                        "merge_strategy": "context_union",
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
                    "node": "M-N1",
                    "response_200": {
                        "superseded_tasks": ["task-alpha-7f8a9b", "task-beta-2c3d4e"],
                        "merged_task_id": "task-merged-uat-f5a7b2",
                        "active_tmo_operation": None,
                        "stash_checkpoint_ids": ["stash-alpha-7f8a9b-3a", "stash-beta-2c3d4e-3a"],
                        "stash_append_only": True,
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
                        "transaction.tmo_merge_log": {"insert_count": 1, "append_only": True, "audit_required": True},
                        "transaction.stash_checkpoint": {"insert_count": 2, "append_only": True},
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
                        {"event_type": "tmo.merge.completed", "node": "M-N1", "event_id": "evt-2026-09-07-011"}
                    ],
                },
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "ac_mapping.md": "# S10 AC 跨引\n- AC: docs/architecture/2026-09-03-langgraph/02-basic-design.md v0.2 §2.6.3 (M-N1 merge)\n- 跨引: docs/uat-design.md §2 AC-UAT-010\n- 守门: #13 a L1↔L1 禁止 + #13 d stash_append_only\n",
        },
    )


def main():
    MOCK_DATA.mkdir(parents=True, exist_ok=True)
    gen_s06_validation_fail()
    gen_s07_conflict_detect()
    gen_s08_rebase_merge()
    gen_s09_merge_request()
    gen_s10_tmo_merge()
    print("S06-S10 fixtures generated (5 scenarios × 5 files = 25 files)")


if __name__ == "__main__":
    main()
