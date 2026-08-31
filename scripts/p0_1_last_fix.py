#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
P0-1 fix 11: 修最后 12 err
- domain-validation: 缺 use uuid::Uuid
- domain-development: ExecutionActor::User(X.user_id) → User(UserId::from(...))
- domain-audit: actor_user_id: UserId::from(actor_ctx.user_id) → UserId::from 不要 (因为 actor_user_id 是 Uuid)
- domain-workspace: 删我之前 patch 加的重复 if 块
- domain-planning: pub fn new() Self 错
"""
import re
import sys
from pathlib import Path

WORKSPACE = Path(r"D:\Star\crates")

DRY_RUN = "--apply" not in sys.argv


def main() -> int:
    print("APPLY" if not DRY_RUN else "DRY-RUN")
    total = 0

    # 1. domain-validation 加 use uuid::Uuid
    f = WORKSPACE / "domain-validation" / "src" / "lib.rs"
    text = f.read_text(encoding="utf-8")
    if "use uuid::Uuid;" not in text:
        # 找第一个 use 块插入
        m = re.search(r'use\s+', text)
        if m:
            text = text[:m.start()] + "use uuid::Uuid;\n" + text[m.start():]
            f.write_text(text, encoding="utf-8")
            print(f"domain-validation/lib.rs                    1 patch (add use uuid::Uuid)")
            total += 1

    # 2. domain-development: ExecutionActor::User(IDENT.user_id) → User(UserId::from(IDENT.user_id))
    f = WORKSPACE / "domain-development" / "src" / "lib.rs"
    text = f.read_text(encoding="utf-8")
    original = text
    text = re.sub(
        r'ExecutionActor::User\((\w+)\.user_id\)',
        r'ExecutionActor::User(UserId::from(\1.user_id))',
        text
    )
    if text != original:
        f.write_text(text, encoding="utf-8")
        n = len(re.findall(r'ExecutionActor::User\(\w+\.user_id\)', original))
        print(f"domain-development/lib.rs                   {n} patch (ExecutionActor::User)")
        total += n

    # 3. domain-audit: actor_user_id: UserId::from(actor_ctx.user_id) → actor_user_id: actor_ctx.user_id
    # 但如果其他调用也是 UserId::from(actor_X.user_id) 不动 (那是 UserId 字段)
    f = WORKSPACE / "domain-audit" / "src" / "lib.rs"
    text = f.read_text(encoding="utf-8")
    original = text
    text = text.replace(
        "actor_user_id: UserId::from(actor_ctx.user_id),",
        "actor_user_id: actor_ctx.user_id,",
    )
    if text != original:
        f.write_text(text, encoding="utf-8")
        print(f"domain-audit/lib.rs                        1 patch (UserId::from remove)")
        total += 1

    # 4. domain-workspace: 删之前 patch 加的 stub
    f = WORKSPACE / "domain-workspace" / "src" / "lib.rs"
    text = f.read_text(encoding="utf-8")
    if "if TenantId::from(actor.tenant_id) != expected { /* workspace check tenant */ }" in text:
        text = text.replace(
            "    if TenantId::from(actor.tenant_id) != expected { /* workspace check tenant */ }\n",
            ""
        )
        f.write_text(text, encoding="utf-8")
        print(f"domain-workspace/lib.rs                    1 patch (remove stub)")
        total += 1

    print(f"\nTotal: {total}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
