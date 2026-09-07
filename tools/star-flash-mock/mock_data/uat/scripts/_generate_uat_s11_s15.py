#!/usr/bin/env python3
# _generate_uat_s11_s15.py - UAT 业务场景 11-15 (TMO M-N2..M-N6) fixture generator
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


def gen_s11_tmo_split() -> None:
    s_id = "S11-tmo-split"
    _write(
        s_id,
        {
            "request.json": json.dumps(
                {
                    "fixture_version": "v1",
                    "scenario": s_id,
                    "class": "Transaction",
                    "node": "M-N2",
                    "method": "POST",
                    "endpoint": "/api/tmo/split",
                    "request": {
                        "operation": "split",
                        "target_task_id": "task-large-001",
                        "split_count": 2,
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
                    "node": "M-N2",
                    "response_200": {
                        "original_task_id": "task-large-001",
                        "new_task_ids": ["task-split-001-a", "task-split-001-b"],
                        "split_strategy": "context_half",
                        "stash_checkpoint_ids": ["stash-task-large-001-split-2-way"],
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
                        "transaction.tmo_split_log": {"insert_count": 1, "append_only": True},
                        "transaction.stash_checkpoint": {"insert_count": 1, "append_only": True},
                    },
                },
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "expected_events.json": json.dumps(
                {"fixture_version": "v1", "scenario": s_id, "events": [{"event_type": "tmo.split.completed", "node": "M-N2", "event_id": "evt-2026-09-07-012"}]},
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "ac_mapping.md": "# S11 AC 跨引\n- AC: docs/architecture/2026-09-03-langgraph/02-basic-design.md v0.2 §2.6.4 (M-N2 split)\n- 跨引: docs/uat-design.md §2 AC-UAT-011\n- 守门: #13 a L1↔L1 禁止 (L0 唯一协调) + #13 d stash_append_only\n",
        },
    )


def gen_s12_tmo_reorder() -> None:
    s_id = "S12-tmo-reorder"
    _write(
        s_id,
        {
            "request.json": json.dumps(
                {
                    "fixture_version": "v1",
                    "scenario": s_id,
                    "class": "Transaction",
                    "node": "M-N3",
                    "method": "POST",
                    "endpoint": "/api/tmo/reorder",
                    "request": {
                        "operation": "dep_set",
                        "edges": [
                            {"from": "task-A", "to": "task-B"},
                            {"from": "task-B", "to": "task-C"},
                        ],
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
                    "node": "M-N3",
                    "response_200": {
                        "edges_added": 2,
                        "cycle_detected": False,
                        "topo_order": ["task-A", "task-B", "task-C"],
                        "stash_checkpoint_ids": ["stash-dep-set-2-edges"],
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
                        "transaction.tmo_reorder_log": {"insert_count": 1, "append_only": True},
                        "master.task_dep": {"update_count": 2, "scd_type": 2},
                    },
                },
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "expected_events.json": json.dumps(
                {"fixture_version": "v1", "scenario": s_id, "events": [{"event_type": "tmo.reorder.completed", "node": "M-N3", "event_id": "evt-2026-09-07-013"}]},
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "ac_mapping.md": "# S12 AC 跨引\n- AC: docs/architecture/2026-09-03-langgraph/02-basic-design.md v0.2 §2.6.5 (M-N3 reorder)\n- 跨引: docs/uat-design.md §2 AC-UAT-012\n- 守门: #13 a L1↔L1 禁止 + cycle_detected=False\n",
        },
    )


def gen_s13_tmo_bulk() -> None:
    s_id = "S13-tmo-bulk"
    _write(
        s_id,
        {
            "request.json": json.dumps(
                {
                    "fixture_version": "v1",
                    "scenario": s_id,
                    "class": "Transaction",
                    "node": "M-N4",
                    "method": "POST",
                    "endpoint": "/api/tmo/bulk",
                    "request": {
                        "operation": "merge",
                        "target_task_ids": ["task-001", "task-002", "task-003", "task-004", "task-005"],
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
                    "node": "M-N4",
                    "response_207": {
                        "succeeded_count": 3,
                        "failed_count": 2,
                        "succeeded": ["task-001", "task-002", "task-003"],
                        "failed": ["task-004", "task-005"],
                        "failure_reasons": ["task-004: cyclic dep", "task-005: invalid SA type"],
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
                    "tables": {"transaction.tmo_bulk_log": {"insert_count": 1, "append_only": True}},
                },
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "expected_events.json": json.dumps(
                {"fixture_version": "v1", "scenario": s_id, "events": [{"event_type": "tmo.bulk.partial_failure", "node": "M-N4", "event_id": "evt-2026-09-07-014"}]},
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "ac_mapping.md": "# S13 AC 跨引\n- AC: docs/architecture/2026-09-03-langgraph/02-basic-design.md v0.2 §2.6.6 (M-N4 bulk)\n- 跨引: docs/uat-design.md §2 AC-UAT-013\n- 守门: #13 a L1↔L1 禁止 + partial_failure 3-of-5\n",
        },
    )


def gen_s14_tmo_summarize() -> None:
    s_id = "S14-tmo-summarize"
    _write(
        s_id,
        {
            "request.json": json.dumps(
                {
                    "fixture_version": "v1",
                    "scenario": s_id,
                    "class": "Transaction",
                    "node": "M-N5",
                    "method": "POST",
                    "endpoint": "/api/tmo/summarize",
                    "request": {
                        "operation": "summarize",
                        "target_task_ids": ["task-summary-001", "task-summary-002", "task-summary-003"],
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
                    "node": "M-N5",
                    "response_200": {
                        "summarized_count": 3,
                        "summary_text": "3 个 L1 任务摘要: 跨域反馈 + 验证 + 合并",
                        "original_token_count": 4500,
                        "summary_token_count": 320,
                        "compression_ratio": 0.071,
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
                    "tables": {"transaction.tmo_summarize_log": {"insert_count": 1, "append_only": True}},
                },
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "expected_events.json": json.dumps(
                {"fixture_version": "v1", "scenario": s_id, "events": [{"event_type": "tmo.summarize.completed", "node": "M-N5", "event_id": "evt-2026-09-07-015"}]},
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "ac_mapping.md": "# S14 AC 跨引\n- AC: docs/architecture/2026-09-03-langgraph/02-basic-design.md v0.2 §2.6.7 (M-N5 summarize)\n- 跨引: docs/uat-design.md §2 AC-UAT-014\n- 守门: #13 a L1↔L1 禁止 (L0 唯一协调)\n",
        },
    )


def gen_s15_tmo_reassign() -> None:
    s_id = "S15-tmo-reassign"
    _write(
        s_id,
        {
            "request.json": json.dumps(
                {
                    "fixture_version": "v1",
                    "scenario": s_id,
                    "class": "Transaction",
                    "node": "M-N6",
                    "method": "POST",
                    "endpoint": "/api/tmo/reassign",
                    "request": {
                        "operation": "reassign",
                        "target_task_id": "task-uat-001",
                        "from_sa": "SA-09",
                        "to_sa": "SA-02",
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
                    "node": "M-N6",
                    "response_200": {
                        "task_id": "task-uat-001",
                        "from_sa": "SA-09",
                        "to_sa": "SA-02",
                        "reassignment_status": "ok",
                        "checkpoint_handoff": True,
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
                    "tables": {"transaction.tmo_reassign_log": {"insert_count": 1, "append_only": True, "sa_handoff": True}},
                },
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "expected_events.json": json.dumps(
                {"fixture_version": "v1", "scenario": s_id, "events": [{"event_type": "tmo.reassign.completed", "node": "M-N6", "event_id": "evt-2026-09-07-016"}]},
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "ac_mapping.md": "# S15 AC 跨引\n- AC: docs/architecture/2026-09-03-langgraph/02-basic-design.md v0.2 §2.6.8 (M-N6 reassign)\n- 跨引: docs/uat-design.md §2 AC-UAT-015\n- 守门: #13 a L1↔L1 禁止 + sa_handoff=True\n",
        },
    )


def main():
    MOCK_DATA.mkdir(parents=True, exist_ok=True)
    gen_s11_tmo_split()
    gen_s12_tmo_reorder()
    gen_s13_tmo_bulk()
    gen_s14_tmo_summarize()
    gen_s15_tmo_reassign()
    print("S11-S15 fixtures generated (5 scenarios × 5 files = 25 files)")


if __name__ == "__main__":
    main()
