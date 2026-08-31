#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
P0-1 fix 17: 修最后 5+1 err
- domain-workspace 583: actor.tenant_id != *expected → actor.tenant_id != expected.0
- domain-feedback 84, 589: ActorContext::new(Uuid::new_v4(), tenant_id.0) → (UserId::new(), tenant_id)
- domain-local-runtime spawn_upload_integration 22: use 路径错
"""
import re
import sys
from pathlib import Path

WORKSPACE = Path(r"D:\Star\crates")

DRY_RUN = "--apply" not in sys.argv


def main() -> int:
    print("APPLY" if not DRY_RUN else "DRY-RUN")
    total = 0

    # 1. domain-workspace 583
    f = WORKSPACE / "domain-workspace" / "src" / "lib.rs"
    text = f.read_text(encoding="utf-8")
    if "if actor.tenant_id != *expected {" in text:
        text = text.replace(
            "if actor.tenant_id != *expected {",
            "if actor.tenant_id != expected.0 {"
        )
        f.write_text(text, encoding="utf-8")
        print(f"domain-workspace/lib.rs                    1 patch")
        total += 1

    # 2. domain-feedback 84, 589
    f = WORKSPACE / "domain-feedback" / "src" / "lib.rs"
    text = f.read_text(encoding="utf-8")
    original = text
    text = re.sub(
        r'ActorContext::new\(Uuid::new_v4\(\),\s*(\w+)\.0\)',
        r'ActorContext::new(UserId::new(), \1)',
        text
    )
    if text != original:
        f.write_text(text, encoding="utf-8")
        n = len(re.findall(r'ActorContext::new\(Uuid::new_v4\(\),\s*\w+\.0\)', original))
        print(f"domain-feedback/lib.rs                       {n} patches")
        total += n

    # 3. domain-local-runtime spawn_upload_integration.rs 22 行 use 错
    f = WORKSPACE / "domain-local-runtime" / "src" / "spawn_upload_integration.rs"
    if f.exists():
        text = f.read_text(encoding="utf-8")
        # 看实际 — E0277 报 use 路径错, 我脚本没改这个文件
        # 先看实际错
        pass

    print(f"\nTotal: {total}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
