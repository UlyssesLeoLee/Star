#!/usr/bin/env python3
# _generate_10_mcp_fixtures.py - MCP 16 tool 扩 10 份 (6 → 16) fixture generator
# Author: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #10 + 19:39 JST 授权)
# 触发: 2026-09-05 07:18 JST user 拍板 "完成剩余轮次" + P3-C MCP 16 tool 扩 10 份
# 守门: AGENTS.md §7 #1 16 tool 真实接入 e2e + ADR-0032 MCP Transport stdio

from __future__ import annotations

import json
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[3]
MCP_DIR = REPO_ROOT / "tools" / "star-flash-mock" / "mock_data" / "mcp"

# 10 缺标 MCP tool (per AGENTS.md §7 #1 + §26 README v0.2 缺口 #1)
TOOLS = [
    {"name": "audit_event", "method": "GET", "scenario": "list-audit-events", "table": "audit.audit_event", "description": "MCP tool: audit_event list (per 守门 #13 b T 物理删除禁止 + audit_logged)", "wtm_class": "Transaction"},
    {"name": "scm", "method": "GET", "scenario": "list-repositories", "table": "scm.repository", "description": "MCP tool: scm list repositories (per 5 域 match)", "wtm_class": "Master"},
    {"name": "workspace", "method": "GET", "scenario": "list-workspaces", "table": "workspace.workspace", "description": "MCP tool: workspace list (per M/T 混合)", "wtm_class": "Master"},
    {"name": "feedback", "method": "POST", "scenario": "create-feedback", "table": "feedback.feedback", "description": "MCP tool: feedback create (per 5 域 social + 状态机 6 态)", "wtm_class": "Transaction"},
    {"name": "inbox", "method": "GET", "scenario": "list-inbox-items", "table": "notification.notification", "description": "MCP tool: inbox list (per 5 域 social + 短 TTL)", "wtm_class": "Transaction"},
    {"name": "project", "method": "GET", "scenario": "list-projects", "table": "project.project", "description": "MCP tool: project list (per 5 域 match)", "wtm_class": "Transaction"},
    {"name": "permission", "method": "GET", "scenario": "list-permissions", "table": "permission.permission", "description": "MCP tool: permission list (per 5 域 admin + 13 類 RLS)", "wtm_class": "Master"},
    {"name": "kms", "method": "GET", "scenario": "list-kms-keys", "table": "identity.credential", "description": "MCP tool: kms list (per 5 域 player + 凭据管理)", "wtm_class": "Transaction"},
    {"name": "form", "method": "GET", "scenario": "list-forms", "table": "workflow.workflow_definition", "description": "MCP tool: form list (per workflow 表单定义)", "wtm_class": "Master"},
    {"name": "search", "method": "GET", "scenario": "search-query", "table": "search.search_index", "description": "MCP tool: search query (per W 类 派生重建)", "wtm_class": "Work"},
]


def build_fixture(tool: dict) -> dict:
    """生成 16 tool 扩 fixture。"""
    return {
        "fixture_version": "v1",
        "module": "mcp",
        "tool": tool["name"],
        "method": tool["method"],
        "scenario": tool["scenario"],
        "description": tool["description"],
        "schema_ref": f"crates/star-mcp/src/{tool['name']}.rs (per 16 tool 真实接入 e2e, AGENTS.md §7 #1)",
        "守门": [
            "AGENTS.md §7 #1 16 tool 真实接入 e2e",
            f"#13 {tool['wtm_class'][0]} {tool['wtm_class']} ({'物理删除禁止' if tool['wtm_class'] in ['Master', 'Transaction'] else '物理删除'})",
            "ADR-0032 MCP Transport stdio",
        ],
        "table": tool["table"],
        "wtm_class": tool["wtm_class"],
        "request": {
            "tool_name": tool["name"],
            "params": {"tenant_id": "tenant-001", "limit": 20},
        },
        "response_200": {
            "tool_invocation_id": f"invoke-uuid-{tool['name']}-001",
            "tool_result": {
                "items": [{"id": f"{tool['name']}-001", "name": f"{tool['name']}_item_1"}],
                "total": 1,
                "elapsed_ms": 25,
            },
            "elapsed_ms": 25,
        },
        "fixture_assertion": {
            "tool_16_listed": True,
            "wtm_class": tool["wtm_class"],
            "tenant_id_present": True,
            "rls_13_classes": True,
        },
    }


def main():
    """主入口: 10 MCP tool fixture 生成 (跳过已存在)."""
    MCP_DIR.mkdir(parents=True, exist_ok=True)
    created = 0
    skipped = 0
    for tool in TOOLS:
        filename = f"v1--mcp--{tool['name'].replace('_', '-')}--{tool['method']}.json"
        path = MCP_DIR / filename
        if path.exists():
            skipped += 1
            continue
        path.write_text(json.dumps(build_fixture(tool), ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        created += 1
    print(f"created: {created}, skipped (idempotent): {skipped}, total: {len(TOOLS)}")


if __name__ == "__main__":
    main()
