#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
P0-1 fix 19: 修 ActorContext::new 调用方
- UserId::from(uuid::Uuid::nil()) → uuid::Uuid::nil()
- TenantId::new() → uuid::Uuid::new_v4()
- domain_xxx::UserId::from(uuid::Uuid::nil()) → uuid::Uuid::nil()
- domain_xxx::TenantId::new() → uuid::Uuid::new_v4()
"""
import re
import sys
from pathlib import Path

WORKSPACE = Path(r"D:\Star\crates")

DRY_RUN = "--apply" not in sys.argv


def main() -> int:
    print("APPLY" if not DRY_RUN else "DRY-RUN")
    total = 0
    targets = [
        "api", "application", "infrastructure", "star-mcp", "star-cli", "star-saga", "star-cache",
        "star-context", "star-sa", "star-sse", "star-webhook",
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
            # 1. UserId::from(uuid::Uuid::nil()) → uuid::Uuid::nil()
            n2 = text.count("UserId::from(uuid::Uuid::nil())")
            text = text.replace("UserId::from(uuid::Uuid::nil())", "uuid::Uuid::nil()")
            n += n2
            # 2. domain_xxx::UserId::from(uuid::Uuid::nil()) → uuid::Uuid::nil()
            n2 = len(re.findall(r'\w+::UserId::from\(uuid::Uuid::nil\(\)\)', text))
            text = re.sub(r'\w+::UserId::from\(uuid::Uuid::nil\(\)\)', 'uuid::Uuid::nil()', text)
            n += n2
            # 3. domain_xxx::TenantId::new() → uuid::Uuid::new_v4() (在 ActorContext::new 第二参)
            n2 = len(re.findall(r'\w+::TenantId::new\(\)', text))
            text = re.sub(r'\w+::TenantId::new\(\)', 'uuid::Uuid::new_v4()', text)
            n += n2
            # 4. 单独的 TenantId::new() 在 ActorContext::new 第二参 → Uuid::new_v4()
            # 上下文: ActorContext::new(...,\n        TenantId::new(),\n)
            n2 = text.count("TenantId::new()")
            text = re.sub(r'TenantId::new\(\)', 'uuid::Uuid::new_v4()', text)
            n += n2
            # 5. 单独的 UserId::new() 在 ActorContext::new 第一参 → uuid::Uuid::new_v4()
            n2 = text.count("UserId::new()")
            text = re.sub(r'UserId::new\(\)', 'uuid::Uuid::new_v4()', text)
            n += n2

            if not DRY_RUN and text != original:
                f.write_text(text, encoding="utf-8")
            if n > 0:
                print(f"{str(f.relative_to(WORKSPACE)):50} {n} patches")
                total += n
    print(f"\nTotal: {total}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
