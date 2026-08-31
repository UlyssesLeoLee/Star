#!/usr/bin/env python3
"""
P0-1c fix: star-mcp handlers ActorContext::new 参数从 Uuid 改 UserId/TenantId
- 适用于 feedback / permission / project / tenant / work_item / identity (6 handler)
- workspace / worktree / audit / board / collaboration 不动 (用 star_context 顶层 re-export)
"""
import re
import sys
from pathlib import Path

WORKSPACE = Path(r"D:\Star\crates\star-mcp\src\handlers")

HANDLERS_TO_FIX = [
    "feedback.rs", "permission.rs", "project.rs", "tenant.rs",
    "work_item.rs", "identity.rs",
]

# 找含 domain_xxx::context::ActorContext import 的 handler, 改 ActorContext::new(uuid::Uuid::nil(), ...) → (UserId::new(), TenantId::new())


def main() -> int:
    print("APPLY" if not DRY_RUN else "DRY-RUN")
    for h in HANDLERS_TO_FIX:
        f = WORKSPACE / h
        if not f.exists():
            continue
        c = f.read_text(encoding="utf-8")
        if "context::ActorContext" not in c:
            continue
        c2 = c
        # ActorContext::new(uuid::Uuid::nil(), uuid::Uuid::new_v4())
        c2 = re.sub(
            r'ActorContext::new\(uuid::Uuid::nil\(\), uuid::Uuid::new_v4\(\)\)',
            'ActorContext::new(UserId::new(), TenantId::new())',
            c2
        )
        # ActorContext::new(uuid::Uuid::nil(), tenant_id)  (强类型 tenant_id)
        c2 = re.sub(
            r'ActorContext::new\(uuid::Uuid::nil\(\), tenant_id\)',
            'ActorContext::new(UserId::new(), tenant_id)',
            c2
        )
        # ActorContext::new(uuid::Uuid::nil(), tid) — tid 是强类型 TenantId
        c2 = re.sub(
            r'ActorContext::new\(uuid::Uuid::nil\(\), tid\)',
            'ActorContext::new(UserId::new(), tid)',
            c2
        )
        # ActorContext::new(user_id, tenant_id.0) — tenant_id.0 (Uuid) 错, 应该是 tenant_id
        c2 = re.sub(
            r'ActorContext::new\(user_id, tenant_id\.0\)',
            'ActorContext::new(user_id, tenant_id)',
            c2
        )
        if c2 != c:
            f.write_text(c2, encoding="utf-8")
            print(f"{h} patched")
    return 0


if __name__ == "__main__":
    import sys
    DRY_RUN = "--apply" not in sys.argv
    sys.exit(main())
