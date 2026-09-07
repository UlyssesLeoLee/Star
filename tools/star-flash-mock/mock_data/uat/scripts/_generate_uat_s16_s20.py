#!/usr/bin/env python3
# _generate_uat_s16_s20.py - UAT 业务场景 16-20 (TMO M-N7 + Streamable HTTP 4) fixture generator
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


def gen_s16_tmo_metadata() -> None:
    s_id = "S16-tmo-metadata"
    _write(
        s_id,
        {
            "request.json": json.dumps(
                {
                    "fixture_version": "v1",
                    "scenario": s_id,
                    "class": "Master",
                    "node": "M-N7",
                    "method": "POST",
                    "endpoint": "/api/tmo/metadata",
                    "request": {
                        "operation": "add",
                        "target_task_id": "task-uat-002",
                        "metadata": {"priority": "high", "tags": ["uat", "cross-domain"]},
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
                    "node": "M-N7",
                    "response_200": {
                        "task_id": "task-uat-002",
                        "scd_version": 2,
                        "metadata": {"priority": "high", "tags": ["uat", "cross-domain"]},
                        "valid_from": "2026-09-07T15:00:00Z",
                        "valid_to": None,
                        "active": True,
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
                    "tables": {"master.task_metadata": {"insert_count": 1, "scd_type": 2, "physical_delete_forbidden": True}},
                },
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "expected_events.json": json.dumps(
                {"fixture_version": "v1", "scenario": s_id, "events": [{"event_type": "tmo.metadata.added", "node": "M-N7", "event_id": "evt-2026-09-07-017"}]},
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "ac_mapping.md": "# S16 AC 跨引\n- AC: docs/architecture/2026-09-03-langgraph/02-basic-design.md v0.2 §2.6.9 (M-N7 metadata)\n- 跨引: docs/uat-design.md §2 AC-UAT-016\n- 守门: #13 c Master SCD Type 2\n",
        },
    )


def gen_s17_streamable_connect() -> None:
    s_id = "S17-streamable-connect"
    _write(
        s_id,
        {
            "request.json": json.dumps(
                {
                    "fixture_version": "v1",
                    "scenario": s_id,
                    "class": "Transaction",
                    "method": "POST",
                    "endpoint": "/api/mcp/streamable/session",
                    "request": {"actor_session_id": "session-2026-09-07-001"},
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
                        "session_id": "streamable-uat-2026-09-07-001",
                        "created_at": "2026-09-07T15:00:00Z",
                        "transport": "streamable-http",
                        "sse_supported": True,
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
                    "tables": {"transaction.streamable_session": {"insert_count": 1, "append_only": True}},
                },
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "expected_events.json": json.dumps(
                {"fixture_version": "v1", "scenario": s_id, "events": [{"event_type": "streamable.session.created", "event_id": "evt-2026-09-07-018"}]},
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "ac_mapping.md": "# S17 AC 跨引\n- AC: docs/architecture/2026-08-26-upgrade/adr/0032-mcp-transport-stdio.md + AGENTS §7 #3 (Streamable HTTP)\n- 跨引: docs/uat-design.md §2 AC-UAT-017\n- 守门: #13 d Transaction audit\n",
        },
    )


def gen_s18_streamable_push() -> None:
    s_id = "S18-streamable-push"
    _write(
        s_id,
        {
            "request.json": json.dumps(
                {
                    "fixture_version": "v1",
                    "scenario": s_id,
                    "class": "Transaction",
                    "method": "GET",
                    "endpoint": "/api/mcp/streamable/session/{id}/events",
                    "request": {
                        "session_id": "streamable-uat-2026-09-07-001",
                        "headers": {"Accept": "text/event-stream"},
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
                    "response_200_sse": {
                        "content_type": "text/event-stream",
                        "stream_format": "id: <evt_id>\\nevent: <type>\\ndata: <json>\\n\\n",
                        "event_types_pushed": ["tool_call", "agent_dispatch", "interrupt"],
                        "keepalive_interval_seconds": 30,
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
                    "tables": {"transaction.streamable_event": {"insert_count": "N (continuous)", "append_only": True}},
                },
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "expected_events.json": json.dumps(
                {
                    "fixture_version": "v1",
                    "scenario": s_id,
                    "events": [
                        {"event_type": "streamable.push.tool_call", "event_id": "evt-2026-09-07-019"},
                        {"event_type": "streamable.push.agent_dispatch", "event_id": "evt-2026-09-07-020"},
                    ],
                },
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "ac_mapping.md": "# S18 AC 跨引\n- AC: docs/test-design.md v0.8 §23 (Streamable HTTP server-push)\n- 跨引: docs/uat-design.md §2 AC-UAT-018\n- 守门: #13 d + sse_format_compliant\n",
        },
    )


def gen_s19_streamable_reconnect() -> None:
    s_id = "S19-streamable-reconnect"
    _write(
        s_id,
        {
            "request.json": json.dumps(
                {
                    "fixture_version": "v1",
                    "scenario": s_id,
                    "class": "Transaction",
                    "method": "GET",
                    "endpoint": "/api/mcp/streamable/session/{id}/events",
                    "request": {
                        "session_id": "streamable-uat-2026-09-07-001",
                        "last_event_id": "evt-2026-09-07-019",
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
                        "last_event_id": "evt-2026-09-07-019",
                        "resume_from": "evt-2026-09-07-019",
                        "events": [
                            {"id": "evt-2026-09-07-020", "type": "agent_dispatch"},
                            {"id": "evt-2026-09-07-021", "type": "interrupt"},
                        ],
                        "transport": "streamable-http",
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
                    "tables": {"transaction.streamable_resume_log": {"insert_count": 1, "last_event_id": "evt-2026-09-07-019"}},
                },
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "expected_events.json": json.dumps(
                {"fixture_version": "v1", "scenario": s_id, "events": [{"event_type": "streamable.resume", "event_id": "evt-2026-09-07-022"}]},
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "ac_mapping.md": "# S19 AC 跨引\n- AC: docs/test-design.md v0.8 §23 (Streamable HTTP reconnect)\n- 跨引: docs/uat-design.md §2 AC-UAT-019\n- 守门: #13 d + last_event_id 续传\n",
        },
    )


def gen_s20_streamable_delete() -> None:
    s_id = "S20-streamable-delete"
    _write(
        s_id,
        {
            "request.json": json.dumps(
                {
                    "fixture_version": "v1",
                    "scenario": s_id,
                    "class": "Transaction",
                    "method": "DELETE",
                    "endpoint": "/api/mcp/streamable/session/{id}",
                    "request": {"session_id": "streamable-uat-2026-09-07-001"},
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
                        "session_id": "streamable-uat-2026-09-07-001",
                        "status": "deleted",
                        "deleted_at": "2026-09-07T15:05:00Z",
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
                    "tables": {"transaction.streamable_session": {"update_count": 1, "active": False, "physical_delete_forbidden": True}},
                },
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "expected_events.json": json.dumps(
                {"fixture_version": "v1", "scenario": s_id, "events": [{"event_type": "streamable.session.deleted", "event_id": "evt-2026-09-07-023"}]},
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "ac_mapping.md": "# S20 AC 跨引\n- AC: docs/test-design.md v0.8 §23 (Streamable HTTP delete)\n- 跨引: docs/uat-design.md §2 AC-UAT-020\n- 守门: #13 d 软删除 + 审计\n",
        },
    )


def main():
    MOCK_DATA.mkdir(parents=True, exist_ok=True)
    gen_s16_tmo_metadata()
    gen_s17_streamable_connect()
    gen_s18_streamable_push()
    gen_s19_streamable_reconnect()
    gen_s20_streamable_delete()
    print("S16-S20 fixtures generated (5 scenarios × 5 files = 25 files)")


if __name__ == "__main__":
    main()
