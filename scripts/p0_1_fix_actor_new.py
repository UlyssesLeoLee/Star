#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
P0-1 fix 10: 修 ActorContext::new 调用模式
- ActorContext::new(UserId::new(), IDENT) → ActorContext::new(Uuid::new_v4(), IDENT.0)
- ActorContext::new(Uuid::new_v4(), IDENT_TENANT) → ActorContext::new(Uuid::new_v4(), IDENT_TENANT.0)
- UserId::from_uuid(UserId::from(X.user_id)) → UserId::from_uuid(X.user_id) (actor.user_id 已经是 Uuid)
"""
import re
import sys
from pathlib import Path

WORKSPACE = Path(r"D:\Star\crates")

DRY_RUN = "--apply" not in sys.argv


def main() -> int:
    print("APPLY" if not DRY_RUN else "DRY-RUN")
    targets = [
        "domain-permission", "domain-project", "domain-workflow", "domain-tenant",
        "domain-work-item", "domain-worktree", "domain-workspace", "domain-search",
        "domain-scm", "domain-notification", "domain-automation", "domain-audit",
        "domain-collaboration", "domain-comment", "domain-context", "domain-development",
        "domain-feedback", "domain-relation", "domain-board", "domain-identity",
        "domain-local-runtime", "application", "infrastructure", "api",
        "domain-agent", "domain-kms", "domain-integration", "domain-validation",
        "domain-planning", "domain-form", "domain-ai", "domain-dashboard",
        "domain-theme", "domain-report", "domain-cli", "domain-agent-windows",
    ]
    total = 0
    for crate in targets:
        for f in (WORKSPACE / crate / "src").rglob("*.rs"):
            text = f.read_text(encoding="utf-8")
            original = text
            n = 0
            # 1. ActorContext::new(UserId::new(), IDENT) → Uuid::new_v4() + IDENT.0
            text2 = re.sub(
                r'ActorContext::new\(UserId::new\(\),\s*(\w+)\)',
                r'ActorContext::new(Uuid::new_v4(), \1.0)',
                text
            )
            if text2 != text:
                n2 = len(re.findall(r'ActorContext::new\(UserId::new\(\),\s*\w+\)', text))
                n += n2
                text = text2
            # 2. ActorContext::new(Uuid::new_v4(), IDENT) (IDENT 不是 Uuid) → IDENT.0
            text2 = re.sub(
                r'ActorContext::new\(Uuid::new_v4\(\),\s*(?!Uuid::)(\w+)\)',
                r'ActorContext::new(Uuid::new_v4(), \1.0)',
                text
            )
            if text2 != text:
                n2 = len(re.findall(r'ActorContext::new\(Uuid::new_v4\(\),\s*(?!Uuid::)\w+\)', text))
                n += n2
                text = text2
            # 3. UserId::from_uuid(UserId::from(X.user_id)) → UserId::from_uuid(X.user_id) (去掉 from)
            text2 = re.sub(
                r'UserId::from_uuid\(UserId::from\((\w+)\.user_id\)\)',
                r'UserId::from_uuid(\1.user_id)',
                text
            )
            if text2 != text:
                n2 = len(re.findall(r'UserId::from_uuid\(UserId::from\(\w+\.user_id\)\)', text))
                n += n2
                text = text2
            # 4. TenantId::from_uuid(TenantId::from(X.tenant_id)) → TenantId::from_uuid(X.tenant_id)
            text2 = re.sub(
                r'TenantId::from_uuid\(TenantId::from\((\w+)\.tenant_id\)\)',
                r'TenantId::from_uuid(\1.tenant_id)',
                text
            )
            if text2 != text:
                n2 = len(re.findall(r'TenantId::from_uuid\(TenantId::from\(\w+\.tenant_id\)\)', text))
                n += n2
                text = text2
            # 5. TenantId::from(IDENT_TENANT) 在 ActorContext::new 第二参模式 (IDENT 不是 Uuid, 强类型)
            #    ActorContext::new(Uuid::new_v4(), TenantId::from(IDENT)) → ActorContext::new(Uuid::new_v4(), IDENT.0)
            #    等等, 这是相同模式

            if not DRY_RUN and text != original:
                f.write_text(text, encoding="utf-8")
            if n > 0:
                print(f"{crate+'/'+f.name:50} {n} patches")
                total += n
    print(f"\nTotal: {total}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
