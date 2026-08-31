#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
P0-1 fix 20: 撤销 fix_actor_args 破坏性改动
- 把 struct literal 里的 `uuid::Uuid::new_v4()` 改回强类型 ID new()
- 保留 ActorContext::new 调用里的 `uuid::Uuid::new_v4()` (这些是对的)
- 9 个字段名: id / tenant_id / user_id / actor_user_id / granted_by / actor / tenant / project_id / owner_user_id
"""
import re
import sys
from pathlib import Path

WORKSPACE = Path(r"D:\Star\crates")

DRY_RUN = "--apply" not in sys.argv

# 字段 → 强类型 ID
FIELD_TO_TYPE = {
    "id": "UserId",  # 多数 id 字段是 UserId, domain-tenant 的 id 是 TenantId 需要单独修
    "tenant_id": "TenantId",
    "user_id": "UserId",
    "actor_user_id": "UserId",
    "granted_by": "UserId",
    "actor": "UserId",  # 多数 actor 字段是 UserId 强类型
    "tenant": "TenantId",
    "project_id": "ProjectId",
    "owner_user_id": "UserId",
    "reporter_user_id": "UserId",
    "executed_by_user_id": "UserId",
    "created_by": "UserId",
    "updated_by": "UserId",
    "resolved_by": "UserId",
    "closed_by": "UserId",
    "author_user_id": "UserId",
    "target_user_id": "UserId",
    "owner_id": "UserId",
    "member_user_id": "UserId",
    "subject_id": "UserId",
    "agent_id": "UserId",
}


def main() -> int:
    print("APPLY" if not DRY_RUN else "DRY-RUN")
    total = 0
    targets = [
        "star-mcp", "star-cli", "star-saga", "star-cache", "star-context", "star-sa", "star-sse", "star-webhook",
        "api", "application", "infrastructure",
    ] + [f"domain-{n}" for n in [
        "agent", "agent-windows", "ai", "audit", "automation", "board",
        "cli", "collaboration", "comment", "context", "dashboard",
        "development", "feedback", "form", "identity", "integration",
        "kms", "local-runtime", "notification", "permission", "planning",
        "project", "relation", "report", "scm", "search", "tenant",
        "theme", "validation", "work-item", "workflow", "workspace", "worktree",
    ]]

    for crate in targets:
        for f in (WORKSPACE / crate / "src").rglob("*.rs"):
            text = f.read_text(encoding="utf-8")
            original = text
            n = 0
            for field, type_name in FIELD_TO_TYPE.items():
                # 模式: `field: uuid::Uuid::new_v4(),` 在 struct literal
                # 不在 ActorContext::new 调用内 (那个有完整括号, 不一样)
                # 简化: 找 `field: uuid::Uuid::new_v4()` (没 `,` 后面)
                pattern = f"{field}: uuid::Uuid::new_v4()"
                if pattern in text:
                    new_val = f"{field}: {type_name}.new()"
                    text = text.replace(pattern, new_val)
                    n += 1
            if not DRY_RUN and text != original:
                f.write_text(text, encoding="utf-8")
            if n > 0:
                print(f"{str(f.relative_to(WORKSPACE)):50} {n} patches")
                total += n
    print(f"\nTotal: {total}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
