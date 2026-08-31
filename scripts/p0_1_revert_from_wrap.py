#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
P0-1 fix 4: 撤销错误的 from() 转换
- bug: 之前脚本对所有 actor.tenant_id 加 TenantId::from(actor.tenant_id)
- 实际: star_context::ActorContext.tenant_id 已经是 Uuid, 不需要 from
- 实际原代码: q.tenant_id / q.user_id 等是 Uuid, 加 from 反而破坏语义
- 修正: 撤销所有 TenantId::from(actor.tenant_id) / UserId::from(actor.user_id) → actor.tenant_id / actor.user_id
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


def revert_from(crate_dir: Path) -> tuple[int, int]:
    lib_rs = crate_dir / "src" / "lib.rs"
    if not lib_rs.exists():
        return (0, 0)

    text = lib_rs.read_text(encoding="utf-8")
    original = text

    # 1. TenantId::from(actor.tenant_id) → actor.tenant_id
    text = re.sub(r'TenantId::from\(actor\.tenant_id\)', 'actor.tenant_id', text)
    # 2. UserId::from(actor.user_id) → actor.user_id
    text = re.sub(r'UserId::from\(actor\.user_id\)', 'actor.user_id', text)

    # 3. is_platform_admin() 改回 is_platform_admin 字段 (避免方法调用错)
    #    (因为 domain 原代码是 `actor.is_platform_admin` 字段访问, 不是方法调用)
    #    但 star_context::ActorContext 同时有字段和方法, 所以方法调用也 OK
    #    我们撤销: actor.is_platform_admin() → actor.is_platform_admin (字段访问, 等价)
    #    这样跟原 domain 习惯一致 (字段访问)
    text = re.sub(r'\bactor\.is_platform_admin\(\)', 'actor.is_platform_admin', text)
    # is_local_runtime 同理
    text = re.sub(r'\bactor\.is_local_runtime\(\)', 'actor.is_local_runtime', text)

    if not DRY_RUN and text != original:
        lib_rs.write_text(text, encoding="utf-8")

    # 统计改动
    tt = original.count('TenantId::from(actor.tenant_id)')
    uu = original.count('UserId::from(actor.user_id)')
    return (tt, uu)


def main() -> int:
    if DRY_RUN:
        print("DRY-RUN")
    else:
        print("APPLY")

    total_t = 0
    total_u = 0
    for crate in TARGETS:
        d = WORKSPACE / crate
        t, u = revert_from(d)
        total_t += t
        total_u += u
        print(f"{crate:30} TenantId_from={t} UserId_from={u}")
    print(f"\n总撤销: TenantId={total_t}, UserId={total_u}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
