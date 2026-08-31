#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
P0-1 fix 5: 重新加 from 转换 (恢复 from-wrap 状态, 回到 65 err)
- 把 actor.tenant_id 全部加 TenantId::from(actor.tenant_id)
- 把 actor.user_id 全部加 UserId::from(actor.user_id)
- 65 err (q.tenant_id Uuid 不匹配) 单独手工修
"""
import re
import sys
from pathlib import Path

WORKSPACE = Path(r"D:\Star\crates")

DOMAINS_WITH_ACTOR_CTX = [
    "domain-agent", "domain-audit", "domain-automation", "domain-board",
    "domain-collaboration", "domain-comment", "domain-context", "domain-development",
    "domain-identity", "domain-local-runtime", "domain-notification", "domain-permission",
    "domain-planning", "domain-project", "domain-relation", "domain-scm", "domain-search",
    "domain-tenant", "domain-work-item", "domain-workflow", "domain-workspace", "domain-worktree",
]
SUPPORTING_CRATES = ["api", "application", "infrastructure"]
TARGETS = DOMAINS_WITH_ACTOR_CTX + SUPPORTING_CRATES

DRY_RUN = "--apply" not in sys.argv


def reapply_from(crate_dir: Path) -> tuple[int, int]:
    lib_rs = crate_dir / "src" / "lib.rs"
    if not lib_rs.exists():
        return (0, 0)
    text = lib_rs.read_text(encoding="utf-8")
    original = text

    # 把 actor.tenant_id 替换为 TenantId::from(actor.tenant_id) — 排除已经包过的
    # 模式: \bactor\.tenant_id\b 后面不是 (::)
    # 如果前面不是 TenantId::from(, 加 from
    text = re.sub(
        r'(?<!TenantId::from\()(?<!UserId::from\()(?<!ProjectId::from\()\bactor\.tenant_id\b(?!\()',
        'TenantId::from(actor.tenant_id)',
        text
    )
    text = re.sub(
        r'(?<!TenantId::from\()(?<!UserId::from\()(?<!ProjectId::from\()\bactor\.user_id\b(?!\()',
        'UserId::from(actor.user_id)',
        text
    )

    if not DRY_RUN and text != original:
        lib_rs.write_text(text, encoding="utf-8")

    return (
        original.count('actor.tenant_id') - original.count('TenantId::from(actor.tenant_id)'),
        original.count('actor.user_id') - original.count('UserId::from(actor.user_id)'),
    )


def main() -> int:
    if DRY_RUN:
        print("DRY-RUN")
    else:
        print("APPLY")
    total_t = 0
    total_u = 0
    for crate in TARGETS:
        d = WORKSPACE / crate
        t, u = reapply_from(d)
        total_t += t
        total_u += u
        print(f"{crate:30} tenant_id新增={t} user_id新增={u}")
    print(f"\n总: tenant_id={total_t}, user_id={total_u}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
