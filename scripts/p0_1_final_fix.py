#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
P0-1 fix 8: 最终扫尾
- actor: actor_X.user_id / actor: IDENT.user_id → UserId::from(IDENT.user_id)
- actor_user_id: IDENT.user_id (没匹配的) → UserId::from
- TenantId::from(actor.tenant_id) 在 fn check_tenant 链
"""
import re
import sys
from pathlib import Path

WORKSPACE = Path(r"D:\Star\crates")

DRY_RUN = "--apply" not in sys.argv


def main() -> int:
    print("APPLY" if not DRY_RUN else "DRY-RUN")
    targets = [
        "domain-workflow", "domain-project", "domain-development",
        "domain-workspace", "domain-audit", "domain-collaboration",
        "domain-comment", "domain-context", "domain-search", "domain-scm",
        "domain-notification", "domain-relation", "domain-board",
        "domain-tenant", "domain-worktree", "application", "infrastructure",
    ]
    total = 0
    for crate in targets:
        for f in (WORKSPACE / crate / "src").rglob("*.rs"):
            text = f.read_text(encoding="utf-8")
            original = text
            n = 0
            # 1. actor: IDENT.user_id (在 struct literal 里)
            text2 = re.sub(
                r'actor:\s*(?!UserId::from\()(\w+)\.user_id\b',
                r'actor: UserId::from(\1.user_id)',
                text
            )
            if text2 != text:
                n2 = len(re.findall(r'actor:\s*(?!UserId::from\()\w+\.user_id', text))
                n += n2
                text = text2
            # 2. tenant: IDENT.tenant_id 类似
            text2 = re.sub(
                r'tenant:\s*(?!TenantId::from\()(\w+)\.tenant_id\b',
                r'tenant: TenantId::from(\1.tenant_id)',
                text
            )
            if text2 != text:
                n2 = len(re.findall(r'tenant:\s*(?!TenantId::from\()\w+\.tenant_id', text))
                n += n2
                text = text2
            # 3. user_id: IDENT.user_id 模式
            text2 = re.sub(
                r'user_id:\s*(?!UserId::from\()(\w+)\.user_id\b',
                r'user_id: UserId::from(\1.user_id)',
                text
            )
            if text2 != text:
                n2 = len(re.findall(r'user_id:\s*(?!UserId::from\()\w+\.user_id', text))
                n += n2
                text = text2

            if not DRY_RUN and text != original:
                f.write_text(text, encoding="utf-8")
            if n > 0:
                print(f"{crate+'/'+f.name:50} {n} patches")
                total += n
    print(f"\nTotal: {total}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
