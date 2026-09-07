#!/usr/bin/env python3
# _generate_uat_s21_s25.py - UAT 业务场景 21-25 (5 域 AC + 多租户 + RBAC + 审计 + 配额) fixture generator
# Author: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #10 + 19:39 JST 授权)
# 触发: 2026-09-07 14:30 JST user 发令
# 守门: #13 W/T/M 分类 + #19 Python 化 + #14 v2 5 域 Lead CONTENT 4 维

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


def gen_s21_5d_ac_acceptance() -> None:
    s_id = "S21-5d-ac-acceptance"
    _write(
        s_id,
        {
            "request.json": json.dumps(
                {
                    "fixture_version": "v1",
                    "scenario": s_id,
                    "class": "Transaction",
                    "method": "POST",
                    "endpoint": "/api/five-domain/audit/uat",
                    "request": {
                        "domains": ["player", "economy", "match", "social", "admin"],
                        "audit_type": "uat_acceptance",
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
                        "audit_id": "audit-uat-2026-09-07-001",
                        "domains_audited": 5,
                        "ac_pass": 25,
                        "ac_total": 25,
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
                        "transaction.uat_audit_log": {"insert_count": 1, "append_only": True, "audit_required": True},
                        "transaction.five_domain_audit": {"insert_count": 5, "domains": ["player", "economy", "match", "social", "admin"]},
                    },
                },
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "expected_events.json": json.dumps(
                {"fixture_version": "v1", "scenario": s_id, "events": [{"event_type": "uat.5d.audit.completed", "event_id": "evt-2026-09-07-024"}]},
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "ac_mapping.md": "# S21 AC 跨引\n- AC: docs/test-design.md v0.8 §2.6 + §7 (5 域跨域 AC 跨引)\n- 跨引: docs/uat-design.md §2 AC-UAT-021\n- 守门: #14 v2 5 域 Lead CONTENT 4 维 + Mavis 临时代签\n",
        },
    )


def gen_s22_multitenant_isolation() -> None:
    s_id = "S22-multitenant-isolation"
    _write(
        s_id,
        {
            "request.json": json.dumps(
                {
                    "fixture_version": "v1",
                    "scenario": s_id,
                    "class": "Transaction",
                    "method": "POST",
                    "endpoint": "/api/tenants/{id}/isolation-check",
                    "request": {
                        "tenant_id": "t-acme",
                        "cross_tenant_id": "t-beta",
                        "isolation_test": "rls_bypass_attempt",
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
                        "isolation_check_id": "iso-uat-2026-09-07-001",
                        "rls_bypass_succeeded": False,
                        "tenant_id": "t-acme",
                        "rows_accessed": 0,
                        "violation": None,
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
                    "tables": {"transaction.isolation_audit_log": {"insert_count": 1, "append_only": True}},
                },
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "expected_events.json": json.dumps(
                {"fixture_version": "v1", "scenario": s_id, "events": [{"event_type": "tenant.isolation.verified", "event_id": "evt-2026-09-07-025"}]},
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "ac_mapping.md": "# S22 AC 跨引\n- AC: docs/test-design.md v0.8 §9 (Security: RLS bypass attempt)\n- 跨引: docs/uat-design.md §2 AC-UAT-022\n- 守门: #13 d + RLS 13 類必携 (admin 域)\n",
        },
    )


def gen_s23_rbac_scheme() -> None:
    s_id = "S23-rbac-scheme"
    _write(
        s_id,
        {
            "request.json": json.dumps(
                {
                    "fixture_version": "v1",
                    "scenario": s_id,
                    "class": "Master",
                    "method": "POST",
                    "endpoint": "/api/rbac/roles",
                    "request": {
                        "tenant_id": "t-acme",
                        "role_name": "uat-reviewer",
                        "permissions": ["workitem.read", "workitem.write", "worktree.review"],
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
                        "role_id": "role-uat-reviewer-2026-09-07-001",
                        "tenant_id": "t-acme",
                        "role_name": "uat-reviewer",
                        "permissions": ["workitem.read", "workitem.write", "worktree.review"],
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
                    "tables": {"master.rbac_role": {"insert_count": 1, "scd_type": 2, "physical_delete_forbidden": True}},
                },
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "expected_events.json": json.dumps(
                {"fixture_version": "v1", "scenario": s_id, "events": [{"event_type": "rbac.role.created", "event_id": "evt-2026-09-07-026"}]},
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "ac_mapping.md": "# S23 AC 跨引\n- AC: docs/test-design.md v0.8 §9 (Security: RBAC scheme)\n- 跨引: docs/uat-design.md §2 AC-UAT-023\n- 守门: #13 c Master SCD Type 2 + RLS 13 類\n",
        },
    )


def gen_s24_audit_worm() -> None:
    s_id = "S24-audit-worm"
    _write(
        s_id,
        {
            "request.json": json.dumps(
                {
                    "fixture_version": "v1",
                    "scenario": s_id,
                    "class": "Transaction",
                    "method": "POST",
                    "endpoint": "/api/audit/event",
                    "request": {
                        "event_type": "audit.onboarding.failed",
                        "actor_session_id": "session-2026-09-07-001",
                        "worm_required": True,
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
                        "audit_event_id": "audit-evt-uat-2026-09-07-001",
                        "worm_locked": True,
                        "retention_period_seconds": 63072000,  # 2 years
                        "physical_delete_forbidden": True,
                        "tamper_proof": True,
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
                        "transaction.audit_event": {"insert_count": 1, "append_only": True, "worm_locked": True, "retention_period": "2y"},
                    },
                },
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "expected_events.json": json.dumps(
                {"fixture_version": "v1", "scenario": s_id, "events": [{"event_type": "audit.worm.locked", "event_id": "evt-2026-09-07-027"}]},
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "ac_mapping.md": "# S24 AC 跨引\n- AC: docs/architecture/2026-08-26-upgrade/adr/0043-audit-onboarding-failed.md (WORM audit)\n- 跨引: docs/uat-design.md §2 AC-UAT-024\n- 守门: #13 d WORM 物理删除禁止 + 2 年 retention\n",
        },
    )


def gen_s25_quota_exceeded() -> None:
    s_id = "S25-quota-exceeded"
    _write(
        s_id,
        {
            "request.json": json.dumps(
                {
                    "fixture_version": "v1",
                    "scenario": s_id,
                    "class": "Work",
                    "method": "POST",
                    "endpoint": "/api/quotas/check",
                    "request": {
                        "tenant_id": "t-acme",
                        "quota_type": "llm_tokens",
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
                    "response_429": {
                        "quota_check_id": "quota-uat-2026-09-07-001",
                        "quota_exceeded": True,
                        "current_usage": 1100000,
                        "quota_limit": 1000000,
                        "retry_after_seconds": 3600,
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
                    "tables": {"work.quota_counter": {"update_count": 1, "retention_period": "1d", "exceeded": True}},
                },
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "expected_events.json": json.dumps(
                {"fixture_version": "v1", "scenario": s_id, "events": [{"event_type": "quota.exceeded", "event_id": "evt-2026-09-07-028"}]},
                ensure_ascii=False,
                indent=2,
            ) + "\n",
            "ac_mapping.md": "# S25 AC 跨引\n- AC: docs/test-design.md v0.8 §9 (Security: quota)\n- 跨引: docs/uat-design.md §2 AC-UAT-025\n- 守门: #13 a Work 短 TTL + 429 retry_after\n",
        },
    )


def main():
    MOCK_DATA.mkdir(parents=True, exist_ok=True)
    gen_s21_5d_ac_acceptance()
    gen_s22_multitenant_isolation()
    gen_s23_rbac_scheme()
    gen_s24_audit_worm()
    gen_s25_quota_exceeded()
    print("S21-S25 fixtures generated (5 scenarios × 5 files = 25 files)")


if __name__ == "__main__":
    main()
