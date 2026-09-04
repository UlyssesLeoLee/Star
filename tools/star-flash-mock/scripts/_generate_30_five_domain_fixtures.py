#!/usr/bin/env python3
# _generate_30_five_domain_fixtures.py - 5 域 × 6 case = 30 fixture generator
# Author: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #10 + 19:39 JST 授权)
# 触发: 2026-09-05 07:18 JST user 拍板 "完成剩余轮次的内容" + P3-B 5 域 Lead 30 fixture (推荐)
# 守门: #3 5 域独立 Lead (不映射 DDD, per 仓库拓扑 disclaimer) + #14 4 维 RACI

from __future__ import annotations

import json
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[3]
MOCK_DATA = REPO_ROOT / "tools" / "star-flash-mock" / "mock_data" / "five-domain"

# 5 域 (per 守门 #3 历史治理命名, 不映射 DDD, per AGENTS.md §5 仓库拓扑 disclaimer)
DOMAINS = [
    {"name": "player", "lead_status": "Mavis 代签", "scd_version": 1,
     "subdomain_list": ["user", "identity", "workspace", "session"],
     "primary_table": "identity.user", "primary_op": "GET"},
    {"name": "economy", "lead_status": "Mavis 代签", "scd_version": 1,
     "subdomain_list": ["billing", "pricing", "cost", "token_usage"],
     "primary_table": "billing.usage", "primary_op": "GET"},
    {"name": "match", "lead_status": "Mavis 代签", "scd_version": 1,
     "subdomain_list": ["workflow", "state_machine", "saga"],
     "primary_table": "workflow.definition", "primary_op": "GET"},
    {"name": "social", "lead_status": "Mavis 代签", "scd_version": 1,
     "subdomain_list": ["collaboration", "notification", "comment"],
     "primary_table": "notification.notification", "primary_op": "GET"},
    {"name": "admin", "lead_status": "Mavis 代签", "scd_version": 1,
     "subdomain_list": ["rbac", "permission", "tenant", "audit"],
     "primary_table": "permission.role", "primary_op": "GET"},
]

# 6 case per 域 (per 守门 #14 4 维 + 业务实装)
CASES = [
    {"name": "01-list", "method": "GET", "op": "list", "description_suffix": "list scope per 守门 #14 决策 scope"},
    {"name": "02-create", "method": "POST", "op": "create", "description_suffix": "new entity per 守门 #14 RACI R+A 完整责任"},
    {"name": "03-update", "method": "PUT", "op": "update", "description_suffix": "modify per 守门 #14 到位 timeline"},
    {"name": "04-soft-delete", "method": "DELETE", "op": "soft_delete", "description_suffix": "Mavis 代签边界 (SCD v + 1, 不物理删)"},
    {"name": "05-raci", "method": "POST", "op": "raci_check", "description_suffix": "RACI 4 维 R+A+C 完整责任 (per 守门 #14 + #3 v2 派生规)"},
    {"name": "06-mavis-sign", "method": "POST", "op": "mavis_sign", "description_suffix": "Mavis 临时代签 author + 修订人 + 审批 (per 守门 #10 + 19:39 JST 授权)"},
]


def build_fixture(domain: dict, case: dict) -> dict:
    """生成 5 域 × 6 case fixture."""
    base = {
        "fixture_version": "v1",
        "module": "five-domain",
        "domain": domain["name"],
        "case": case["name"],
        "method": case["method"],
        "scenario": f"{domain['name']}-{case['name']}",
        "description": f"5 域业务 {domain['name']} 域 {case['op']} 操作 (per docs/test-design.md v0.7 §25 + AGENTS.md §4 #3 5 域独立 Lead + #14 4 维 RACI, {case['description_suffix']})",
        "schema_ref": f"frontend/src/mocks/handlers/{domain['name']}.ts (per test-design v0.6 §17.4 5 域业务 mock 完整化)",
        "守门": [
            "#3 5 域独立 Lead (不映射 DDD, per AGENTS.md §5 仓库拓扑 disclaimer)",
            "#14 5 域 Lead CONTENT 4 维 (决策 scope / RACI / 到位 timeline / Mavis 代签边界)",
        ],
        "lead_status": domain["lead_status"],
        "subdomain_list": domain["subdomain_list"],
        "primary_table": domain["primary_table"],
        "raci_4_dim": {
            "决策_scope": "跨域 + 域内 (Both, per 守门 #3 v2 派生规)",
            "RACI": "R+A+C 完整责任 (Lead 自执行 R + 负责 A + 接受域内 C 咨询, 域外 I 通知)",
            "到位_timeline": "待定 (Mavis 长期代签, per 9/3 19:35 JST 拍板 D 维持)",
            "Mavis_代签边界": "全部代签 (commit author + 修订人 + 审批, per 守门 #10 + 19:39 JST 授权)",
        },
        "five_lead_real_audit": {
            "audit_status": "Mavis 临时代签",
            "真人_到位": False,
            "sign_count": {"player": 5, "economy": 3, "match": 2, "social": 4, "admin": 6}[domain["name"]],
            "racy_completeness": "R+A+C 完整",
        },
    }
    op = case["op"]
    if op == "list":
        base.update({
            "request": {"domain": domain["name"], "limit": 20},
            "response_200": {
                "items": [{"id": f"{domain['name']}-{i:03d}", "name": f"{domain['name']}_item_{i}"} for i in range(1, 4)],
                "total": 3,
                "scope": "域内 + 跨域",
                "racy": "R(Lead) + A(Lead) + C(域内) + I(域外)",
            },
            "fixture_assertion": {"case": "list", "decision_scope": "Both", "real_lead": False},
        })
    elif op == "create":
        base.update({
            "request": {"domain": domain["name"], "name": f"new_{domain['name']}_entity", "actor_session_id": "session-2026-09-05-001"},
            "response_201": {
                "id": f"{domain['name']}-new-001",
                "created_at": "2026-09-05T07:30:00Z",
                "creator": "Mavis (临时代签)",
                "audit_logged": True,
                "raci": "R(Mavis) + A(Mavis) + C(域内) + I(域外)",
            },
            "fixture_assertion": {"case": "create", "raci_complete": True, "mavis_proxy_sign": True},
        })
    elif op == "update":
        base.update({
            "request": {"domain": domain["name"], "id": f"{domain['name']}-001", "name": "updated_name"},
            "response_200": {
                "id": f"{domain['name']}-001",
                "scd_version": domain["scd_version"] + 1,
                "valid_from": "2026-09-05T07:30:00Z",
                "previous_version": {"scd_version": domain["scd_version"], "valid_to": "2026-09-05T07:30:00Z", "physical_delete": False},
            },
            "fixture_assertion": {"case": "update", "scd_type": 2, "soft_delete": True},
        })
    elif op == "soft_delete":
        base.update({
            "request": {"domain": domain["name"], "id": f"{domain['name']}-001"},
            "response_200": {
                "id": f"{domain['name']}-001",
                "active": False,
                "valid_to": "2026-09-05T07:30:00Z",
                "physical_delete": False,
                "scd_version": domain["scd_version"] + 2,
            },
            "fixture_assertion": {"case": "soft_delete", "physical_delete": False, "scd_type": 2},
        })
    elif op == "raci_check":
        base.update({
            "request": {"domain": domain["name"], "racy_action": "audit", "actor_session_id": "session-2026-09-05-001"},
            "response_200": {
                "domain": domain["name"],
                "racy": {"R": "Mavis 临时代签", "A": "Mavis 临时代签", "C": "域内 Lead 待定", "I": "域外 Lead 通知"},
                "complete": True,
                "violation": None,
            },
            "fixture_assertion": {"case": "raci_check", "racy_complete": True, "violation": None},
        })
    elif op == "mavis_sign":
        base.update({
            "request": {"domain": domain["name"], "action": "sign", "actor_session_id": "session-2026-09-05-001"},
            "response_200": {
                "domain": domain["name"],
                "sign_type": "Mavis 临时代签",
                "commit_author": "Ulysses <ulysses@mavis.local>",
                "修订人": "Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手",
                "审批": "架构师 (Mavis 接手 agent per DEC-008)",
                "sign_timestamp": "2026-09-05T07:30:00Z",
            },
            "fixture_assertion": {"case": "mavis_sign", "author": "Ulysses", "Mavis_proxy": True},
        })
    return base


def main():
    """主入口: 5 域 × 6 case = 30 fixture 生成。"""
    MOCK_DATA.mkdir(parents=True, exist_ok=True)
    created = 0
    skipped = 0
    for domain in DOMAINS:
        d_dir = MOCK_DATA / domain["name"]
        d_dir.mkdir(exist_ok=True)
        for case in CASES:
            filename = f"v1--five-domain--{domain['name']}--{case['name']}--{case['method']}.json"
            path = d_dir / filename
            if path.exists():
                skipped += 1
                continue
            path.write_text(json.dumps(build_fixture(domain, case), ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
            created += 1
    print(f"created: {created}, skipped (idempotent): {skipped}, total: {len(DOMAINS) * len(CASES)}")


if __name__ == "__main__":
    main()
