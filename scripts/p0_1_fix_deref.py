#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
P0-1 fix 9: 修 *tenant_id / *user_id 解引用模式
- viewer.tenant_id != *tenant_id → viewer.tenant_id != tenant_id.0
- viewer.tenant_id != *cmd.tenant_id → viewer.tenant_id != cmd.tenant_id.0
- actor_ctx.user_id 包 UserId::from 不必要 → 还原
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
            # 1. *IDENT.tenant_id → IDENT.tenant_id.0 (访问 tuple field)
            # 排除已经是 .0 的
            text2 = re.sub(
                r'\*(?![\w])(\w+)\.tenant_id\b(?!\.0)',
                r'\1.tenant_id.0',
                text
            )
            if text2 != text:
                n2 = len(re.findall(r'\*(?![\w])\w+\.tenant_id\b(?!\.0)', text))
                n += n2
                text = text2
            # 2. *IDENT.user_id → IDENT.user_id.0
            text2 = re.sub(
                r'\*(?![\w])(\w+)\.user_id\b(?!\.0)',
                r'\1.user_id.0',
                text
            )
            if text2 != text:
                n2 = len(re.findall(r'\*(?![\w])\w+\.user_id\b(?!\.0)', text))
                n += n2
                text = text2
            # 3. UserId::from(IDENT.user_id) 在 star_context 调用（不是 .0 而是 Uuid）
            # 如果 IDENT.user_id 已经是 Uuid (来自 star_context), UserId::from 多余
            # 暂保留 — domain-* 内部用，UUID 强类型 From 仍需要

            if not DRY_RUN and text != original:
                f.write_text(text, encoding="utf-8")
            if n > 0:
                print(f"{crate+'/'+f.name:50} {n} patches")
                total += n
    print(f"\nTotal: {total}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
