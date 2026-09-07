#!/usr/bin/env python3
# _generate_uat_s26_s35.py - UAT 业务场景 26-35 (3 incidents 404 + 4 mcp tools + 2 tsc err + 5 域异步 + 5 域 Saga + Mavis 追溯 + TMO 7 + L1↔L1) fixture generator
# Author: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #10 + 19:39 JST 授权)
# 触发: 2026-09-07 16:15 JST user 发令 "测试结果中是否存在404或者交互不符合预期，协作不符合预期，这些都要100%覆盖"
# 守门: #11 100% 覆盖 0 容忍 + #13 W/T/M 分类 (3 Master / 5 Transaction / 2 Work) + #14 v2 5 域 Lead CONTENT 4 维 + #19 Python 化 + #20 拆 commit 派生规

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


def gen_s26_notimplemented_incident_probe() -> None:
    s_id = "S26-notimplemented-incident-probe"
    _write(
        s_id,
        {
            "request.json": json.dumps(
                {
                    "fixture_version": "v1",
                    "scenario": s_id,
                    "class": "Work",
                    "method": "GET",
                    "endpoint": "/api/incidents/probe-production",
                    "request": {
                        "capability": "probe-production",
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
                    "response_404": {
                        "error": "Capability not implemented (per REQ-OPS-003 §30.6 boundary)",
                        "capability": "probe-production",
                        "note": "TBD: error message schema per basic-design §30.6",
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
                    "tables": {
                        "work.negative_result_cache": {
                            "insert_count": 1,
                            "retention_period": "1d",
                            "key": "probe-production:404",
                        }
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
                            "event_type": "capability.probe-production.rejected",
                            "event_id": "evt-2026-09-07-029",
                            "boundary": "REQ-OPS-003 §30.6",
                        }
                    ],
                },
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "ac_mapping.md": "# S26 AC 跨引\n- AC: docs/test-design.md v0.8 §27.3.2 (per REQ-OPS-003 §30.6 boundary)\n- 跨引: docs/uat-design.md §8.1 404 路径覆盖矩阵 (A.1)\n- 守门: #13 a Work 短 TTL (1d) + #11 100% 覆盖 0 容忍\n",
        },
    )


def gen_s27_notimplemented_incident_alert() -> None:
    s_id = "S27-notimplemented-incident-alert"
    _write(
        s_id,
        {
            "request.json": json.dumps(
                {
                    "fixture_version": "v1",
                    "scenario": s_id,
                    "class": "Work",
                    "method": "POST",
                    "endpoint": "/api/incidents/process-alert",
                    "request": {
                        "alert_id": "alert-2026-09-07-001",
                        "severity": "high",
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
                    "response_404": {
                        "error": "Capability not implemented (per REQ-OPS-003 §30.6 boundary)",
                        "capability": "process-alert",
                        "note": "TBD: error message schema per basic-design §30.6",
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
                    "tables": {
                        "work.negative_result_cache": {
                            "insert_count": 1,
                            "retention_period": "1d",
                            "key": "process-alert:404",
                        }
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
                            "event_type": "capability.process-alert.rejected",
                            "event_id": "evt-2026-09-07-030",
                            "boundary": "REQ-OPS-003 §30.6",
                        }
                    ],
                },
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "ac_mapping.md": "# S27 AC 跨引\n- AC: docs/test-design.md v0.8 §27.3.2 (per REQ-OPS-003 §30.6 boundary)\n- 跨引: docs/uat-design.md §8.1 404 路径覆盖矩阵 (A.2)\n- 守门: #13 a Work 短 TTL (1d) + #11 100% 覆盖 0 容忍\n",
        },
    )


def gen_s28_notimplemented_incident_rollback() -> None:
    s_id = "S28-notimplemented-incident-rollback"
    _write(
        s_id,
        {
            "request.json": json.dumps(
                {
                    "fixture_version": "v1",
                    "scenario": s_id,
                    "class": "Work",
                    "method": "POST",
                    "endpoint": "/api/incidents/inc-001/auto-rollback",
                    "request": {
                        "rollback_target": "v1.0.0",
                        "incident_id": "inc-001",
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
                    "response_404": {
                        "error": "Capability not implemented (per REQ-OPS-003 §30.6 boundary)",
                        "capability": "auto-rollback",
                        "note": "TBD: error message schema per basic-design §30.6",
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
                    "tables": {
                        "work.negative_result_cache": {
                            "insert_count": 1,
                            "retention_period": "1d",
                            "key": "auto-rollback:404",
                        }
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
                            "event_type": "capability.auto-rollback.rejected",
                            "event_id": "evt-2026-09-07-031",
                            "boundary": "REQ-OPS-003 §30.6",
                        }
                    ],
                },
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "ac_mapping.md": "# S28 AC 跨引\n- AC: docs/test-design.md v0.8 §27.3.2 (per REQ-OPS-003 §30.6 boundary)\n- 跨引: docs/uat-design.md §8.1 404 路径覆盖矩阵 (A.3)\n- 守门: #13 a Work 短 TTL (1d) + #11 100% 覆盖 0 容忍\n",
        },
    )


def gen_s29_mcp_tool_failed_empty_result() -> None:
    s_id = "S29-mcp-tool-failed-empty-result"
    _write(
        s_id,
        {
            "request.json": json.dumps(
                {
                    "fixture_version": "v1",
                    "scenario": s_id,
                    "class": "Master",
                    "method": "POST",
                    "endpoint": "/api/mcp/tools-invoke",
                    "request": {
                        "tools": [
                            "find_references",
                            "get_code_context",
                            "get_symbol",
                            "search_code",
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
                    "class": "Master",
                    "response_200": {
                        "tools_status": {
                            "find_references": {"result": None, "empty": True, "scd_version": 1},
                            "get_code_context": {"result": None, "empty": True, "scd_version": 1},
                            "get_symbol": {"result": None, "empty": True, "scd_version": 1},
                            "search_code": {"result": None, "empty": True, "scd_version": 1},
                        },
                        "all_empty": True,
                        "pre_existing_failed": True,
                        "scd_version": 1,
                        "valid_from": "2026-09-07T16:00:00Z",
                        "valid_to": None,
                        "physical_delete_forbidden": True,
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
                        "master.mcp_tool_state": {
                            "update_count": 4,
                            "scd_type": 2,
                            "physical_delete_forbidden": True,
                            "tools": ["find_references", "get_code_context", "get_symbol", "search_code"],
                        }
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
                            "event_type": "mcp.tool.empty_result.verified",
                            "event_id": "evt-2026-09-07-032",
                            "tools": ["find_references", "get_code_context", "get_symbol", "search_code"],
                        }
                    ],
                },
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "ac_mapping.md": "# S29 AC 跨引\n- AC: docs/test-design.md v0.8 §27.6 缺口 #1 (4 pre-existing star-mcp tools failed)\n- 跨引: docs/uat-design.md §8.1 404 路径覆盖矩阵 (B.1-B.4)\n- 守门: #13 c Master SCD Type 2 + physical_delete_forbidden + RLS 13 類必携\n",
        },
    )


def gen_s30_tsc_err_render_fallback() -> None:
    s_id = "S30-tsc-err-render-fallback"
    _write(
        s_id,
        {
            "request.json": json.dumps(
                {
                    "fixture_version": "v1",
                    "scenario": s_id,
                    "class": "Transaction",
                    "method": "GET",
                    "endpoint": "/agent-view",
                    "request": {
                        "actor_session_id": "session-2026-09-07-001",
                        "tsc_err_locations": [
                            "src/app/agent-view/page.tsx:398",
                            "src/lib/store.ts:562",
                        ],
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
                        "page": "agent-view",
                        "tsc_err_advisory_mode": True,
                        "guard_v2_advisory": True,
                        "ci_pr_12_pass": "9/9",
                        "error_boundary_triggered": False,
                        "fallback_ui": False,
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
                        "transaction.tsc_err_audit": {
                            "insert_count": 2,
                            "append_only": True,
                            "audit_required": True,
                            "locations": [
                                "src/app/agent-view/page.tsx:398",
                                "src/lib/store.ts:562",
                            ],
                        }
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
                            "event_type": "tsc.err.render_fallback.verified",
                            "event_id": "evt-2026-09-07-033",
                            "locations": [
                                "src/app/agent-view/page.tsx:398",
                                "src/lib/store.ts:562",
                            ],
                        }
                    ],
                },
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "ac_mapping.md": "# S30 AC 跨引\n- AC: docs/test-design.md v0.8 §27.6 缺口 #3 (2 pre-existing tsc err, per Worker 12 实证)\n- 跨引: docs/uat-design.md §8.2 交互预期覆盖矩阵 (C.1-C.2)\n- 守门: #6 v2 + #1 v26 + #1 v25 PR #12 9/9 CI 全 pass (advisory 模式 0 阻断)\n",
        },
    )


def gen_s31_async_timeout_5d_concurrent() -> None:
    s_id = "S31-async-timeout-5d-concurrent"
    _write(
        s_id,
        {
            "request.json": json.dumps(
                {
                    "fixture_version": "v1",
                    "scenario": s_id,
                    "class": "Transaction",
                    "method": "POST",
                    "endpoint": "/api/five-domain/concurrent-update",
                    "request": {
                        "domains": ["player", "economy", "match", "social", "admin"],
                        "timeout_warning_threshold_ms": 5000,
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
                        "concurrent_update_id": "concur-uat-2026-09-07-001",
                        "domains_updated": 5,
                        "elapsed_ms": 0,
                        "timeout_warning_triggered": False,
                        "coordination_consistent": True,
                        "l0_coordinated": True,
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
                        "transaction.five_domain_concurrent_audit": {
                            "insert_count": 5,
                            "append_only": True,
                            "audit_required": True,
                            "domains": ["player", "economy", "match", "social", "admin"],
                        }
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
                            "event_type": "five_domain.concurrent.completed",
                            "event_id": "evt-2026-09-07-034",
                            "domains": 5,
                            "l0_coordinated": True,
                        }
                    ],
                },
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "ac_mapping.md": "# S31 AC 跨引\n- AC: docs/test-design.md v0.8 §27.3 (5 域跨域并发 + 守门 #9 v2 + v3 实证)\n- 跨引: docs/uat-design.md §8.2 交互预期覆盖矩阵 (1 跨 session 异步)\n- 守门: #13 d Transaction append-only + audit 100% + 守门 #9 v2 + v3\n",
        },
    )


def gen_s32_5d_cross_domain_saga_coordination() -> None:
    s_id = "S32-5d-cross-domain-saga-coordination"
    _write(
        s_id,
        {
            "request.json": json.dumps(
                {
                    "fixture_version": "v1",
                    "scenario": s_id,
                    "class": "Transaction",
                    "method": "POST",
                    "endpoint": "/api/five-domain/saga/coordinate",
                    "request": {
                        "saga_id": "saga-uat-2026-09-07-001",
                        "domains": ["player", "economy", "match", "social", "admin"],
                        "steps": [
                            {"step": 1, "domain": "player", "action": "validate"},
                            {"step": 2, "domain": "economy", "action": "charge"},
                            {"step": 3, "domain": "match", "action": "dispatch"},
                            {"step": 4, "domain": "social", "action": "notify"},
                            {"step": 5, "domain": "admin", "action": "audit"},
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
                    "response_200": {
                        "saga_id": "saga-uat-2026-09-07-001",
                        "saga_status": "completed",
                        "steps_executed": 5,
                        "l0_coordinated": True,
                        "l1_to_l1_prohibited": True,
                        "raci_4_dim": {
                            "decision_scope": "跨域 + 域内 (Both, per 守门 #3 v2 派生规)",
                            "raci": "R+A+C 完整责任",
                            "timeline": "待定 (Mavis 长期代签)",
                            "mavis_sign_boundary": "全部代签",
                        },
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
                        "transaction.saga_log": {
                            "insert_count": 5,
                            "append_only": True,
                            "audit_required": True,
                            "saga_id": "saga-uat-2026-09-07-001",
                        }
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
                            "event_type": "saga.cross_domain.completed",
                            "event_id": "evt-2026-09-07-035",
                            "steps": 5,
                            "l0_coordinated": True,
                        }
                    ],
                },
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "ac_mapping.md": "# S32 AC 跨引\n- AC: docs/test-design.md v0.8 §2.3.1 #3 + §7 S-04 (5 域跨域 Saga)\n- 跨引: docs/uat-design.md §8.3 协作预期覆盖矩阵 (1 5 域跨域协调)\n- 守门: #13 a L0 唯一协调 + #14 v2 5 域 Lead CONTENT 4 维 + Mavis 临时代签\n",
        },
    )


def gen_s33_mavis_proxy_real_person_signoff() -> None:
    s_id = "S33-mavis-proxy-real-person-signoff"
    _write(
        s_id,
        {
            "request.json": json.dumps(
                {
                    "fixture_version": "v1",
                    "scenario": s_id,
                    "class": "Master",
                    "method": "POST",
                    "endpoint": "/api/five-domain/mavis-sign-off",
                    "request": {
                        "domains": ["player", "economy", "match", "social", "admin"],
                        "mavis_sign_type": "Mavis 临时代签",
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
                        "sign_off_id": "signoff-uat-2026-09-07-001",
                        "domains_signed": 5,
                        "mavis_proxy_sign": True,
                        "mavis_sign_type": "Mavis 临时代签",
                        "commit_author": "Ulysses <ulysses@mavis.local>",
                        "approval": "架构师 (Mavis 接手 agent per DEC-008)",
                        "revised_by": "Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手",
                        "real_person_signoff_pending": True,
                        "real_person_timeline": "待定 (per 9/3 19:35 JST 拍板 D 维持)",
                        "scd_version": 1,
                        "valid_from": "2026-09-07T16:00:00Z",
                        "valid_to": None,
                        "physical_delete_forbidden": True,
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
                        "master.mavis_signoff_history": {
                            "insert_count": 5,
                            "scd_type": 2,
                            "physical_delete_forbidden": True,
                            "domains": ["player", "economy", "match", "social", "admin"],
                        }
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
                            "event_type": "mavis.signoff.recorded",
                            "event_id": "evt-2026-09-07-036",
                            "domains": 5,
                            "real_person_pending": True,
                        }
                    ],
                },
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "ac_mapping.md": "# S33 AC 跨引\n- AC: docs/test-design.md v0.8 §25 (5 域 Lead 4 维) + docs/recruitment/5-business-domain-lead-referral.md v0.1\n- 跨引: docs/uat-design.md §8.3 协作预期覆盖矩阵 (2 Mavis 临时代签追溯)\n- 守门: #13 c Master SCD Type 2 + #14 v2 5 域 Lead CONTENT 4 维 + #1 禁回溯叙事\n",
        },
    )


def gen_s34_tmo_7node_orchestration_coordination() -> None:
    s_id = "S34-tmo-7node-orchestration-coordination"
    _write(
        s_id,
        {
            "request.json": json.dumps(
                {
                    "fixture_version": "v1",
                    "scenario": s_id,
                    "class": "Transaction",
                    "method": "POST",
                    "endpoint": "/api/tmo/orchestrate",
                    "request": {
                        "tmo_nodes": [
                            {"id": "M-N1", "op": "merge"},
                            {"id": "M-N2", "op": "split"},
                            {"id": "M-N3", "op": "reorder"},
                            {"id": "M-N4", "op": "bulk"},
                            {"id": "M-N5", "op": "summarize"},
                            {"id": "M-N6", "op": "reassign"},
                            {"id": "M-N7", "op": "metadata"},
                        ],
                        "l0_coordination": True,
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
                        "tmo_orchestration_id": "tmo-uat-2026-09-07-001",
                        "tmo_nodes_executed": 7,
                        "l0_coordinated": True,
                        "l1_to_l1_prohibited": True,
                        "nodes_status": {
                            "M-N1_merge": "ok",
                            "M-N2_split": "ok",
                            "M-N3_reorder": "ok",
                            "M-N4_bulk": "ok",
                            "M-N5_summarize": "ok",
                            "M-N6_reassign": "ok",
                            "M-N7_metadata": "ok",
                        },
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
                        "transaction.tmo_orchestration_log": {
                            "insert_count": 7,
                            "append_only": True,
                            "audit_required": True,
                            "nodes": ["M-N1", "M-N2", "M-N3", "M-N4", "M-N5", "M-N6", "M-N7"],
                        }
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
                            "event_type": "tmo.7node.orchestration.completed",
                            "event_id": "evt-2026-09-07-037",
                            "nodes": 7,
                            "l0_coordinated": True,
                        }
                    ],
                },
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "ac_mapping.md": "# S34 AC 跨引\n- AC: docs/architecture/2026-09-03-langgraph/02-basic-design.md v0.2 §2.6 (TMO 7 节点) + ADR-0046\n- 跨引: docs/uat-design.md §8.3 协作预期覆盖矩阵 (4 TMO 7 节点跨域编排)\n- 守门: #13 a L0 唯一协调 + 守门 #13 a L1↔L1 禁止 + ADR-0046\n",
        },
    )


def gen_s35_l1_l1_prohibition_l0_coordination() -> None:
    s_id = "S35-l1-l1-prohibition-l0-coordination"
    _write(
        s_id,
        {
            "request.json": json.dumps(
                {
                    "fixture_version": "v1",
                    "scenario": s_id,
                    "class": "Transaction",
                    "method": "POST",
                    "endpoint": "/api/tmo/l1-l1-check",
                    "request": {
                        "l1_pairs": [
                            ["SA-01", "SA-02"],
                            ["SA-03", "SA-04"],
                            ["SA-05", "SA-06"],
                            ["SA-07", "SA-08"],
                            ["SA-09", "SA-10"],
                        ],
                        "l0_coordination_required": True,
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
                        "l1_l1_check_id": "l1l1-uat-2026-09-07-001",
                        "l1_pairs_checked": 5,
                        "l1_to_l1_direct_prohibited": True,
                        "l0_coordination_enforced": True,
                        "l0_routed": 5,
                        "l1_direct_routed": 0,
                        "cycle_detected": False,
                        "complexity": "O(V+E)",
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
                        "transaction.l1_l1_prohibition_audit": {
                            "insert_count": 5,
                            "append_only": True,
                            "audit_required": True,
                            "l0_routed": 5,
                            "l1_direct_routed": 0,
                        }
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
                            "event_type": "l1_l1.prohibition.enforced",
                            "event_id": "evt-2026-09-07-038",
                            "l0_routed": 5,
                            "l1_direct_routed": 0,
                        }
                    ],
                },
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "ac_mapping.md": "# S35 AC 跨引\n- AC: docs/architecture/2026-09-03-langgraph/02-basic-design.md v0.2 §2.6 + ADR-0046 (L1↔L1 禁止, L0 协调)\n- 跨引: docs/uat-design.md §8.3 协作预期覆盖矩阵 (5 L1↔L1 禁止 L0 协调)\n- 守门: #13 a L0 唯一协调 + 守门 #13 a L1↔L1 禁止 (TMO-03 4 类 cycle + O(V+E))\n",
        },
    )


def main():
    MOCK_DATA.mkdir(parents=True, exist_ok=True)
    gen_s26_notimplemented_incident_probe()
    gen_s27_notimplemented_incident_alert()
    gen_s28_notimplemented_incident_rollback()
    gen_s29_mcp_tool_failed_empty_result()
    gen_s30_tsc_err_render_fallback()
    gen_s31_async_timeout_5d_concurrent()
    gen_s32_5d_cross_domain_saga_coordination()
    gen_s33_mavis_proxy_real_person_signoff()
    gen_s34_tmo_7node_orchestration_coordination()
    gen_s35_l1_l1_prohibition_l0_coordination()
    print("S26-S35 fixtures generated (10 scenarios × 5 files = 50 files)")


if __name__ == "__main__":
    main()
