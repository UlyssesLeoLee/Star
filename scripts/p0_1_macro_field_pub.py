#!/usr/bin/env python3
"""
P0-1 macro fix: define_uuid_id! 字段改为 pub
- per: value_object::UserId(uuid::Uuid::new_v4()) tuple 构造需要字段 pub
- 修改 22 domain + 3 supporting (但是 supporting 没用 define_uuid_id!)
"""
import re
import sys
from pathlib import Path

WORKSPACE = Path(r"D:\Star\crates")

DOMAINS = [
    "domain-agent", "domain-agent-windows", "domain-ai", "domain-audit", "domain-automation",
    "domain-board", "domain-cli", "domain-collaboration", "domain-comment", "domain-context",
    "domain-dashboard", "domain-development", "domain-feedback", "domain-form", "domain-identity",
    "domain-integration", "domain-kms", "domain-local-runtime", "domain-notification",
    "domain-permission", "domain-planning", "domain-project", "domain-relation", "domain-report",
    "domain-scm", "domain-search", "domain-tenant", "domain-theme", "domain-validation",
    "domain-work-item", "domain-workflow", "domain-workspace", "domain-worktree",
]

DRY_RUN = "--apply" not in sys.argv


def main() -> int:
    print("APPLY" if not DRY_RUN else "DRY-RUN")
    for d in DOMAINS:
        f = WORKSPACE / d / "src" / "macros.rs"
        if not f.exists():
            continue
        text = f.read_text(encoding="utf-8")
        if "pub struct $name(uuid::Uuid);" in text:
            new_text = text.replace(
                "pub struct $name(uuid::Uuid);",
                "pub struct $name(pub uuid::Uuid);",
                1
            )
            if not DRY_RUN:
                f.write_text(new_text, encoding="utf-8")
            print(f"{d}/src/macros.rs patched")
    # 还要修 lib.rs 顶部直接定义 macro_rules! 的 (e.g. domain-validation, domain-context)
    for d in DOMAINS:
        f = WORKSPACE / d / "src" / "lib.rs"
        if not f.exists():
            continue
        text = f.read_text(encoding="utf-8")
        if "pub struct $name(uuid::Uuid);" in text:
            new_text = text.replace(
                "pub struct $name(uuid::Uuid);",
                "pub struct $name(pub uuid::Uuid);",
                1
            )
            if not DRY_RUN:
                f.write_text(new_text, encoding="utf-8")
            print(f"{d}/src/lib.rs patched")
    return 0


if __name__ == "__main__":
    sys.exit(main())
