#!/usr/bin/env python3
"""
scripts/automation/fix_b4_sed20.py v0.1
Phase B.4 sub-session #4 改进版: 20-23 空格缩进的真 struct shorthand wrap

domain-agent 等 crate 中有 20 空格 struct 内部 shorthand, fixer v0.3 16-19 范围跳过,
手工 backout 1 处 + sed 改其他 21 处
"""
import sys
from pathlib import Path

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

WORKDIR = Path("D:/Star/.worktrees/feat-auto-20260904-1c260bc7")

# 21 处 20 空格 struct 内部 shorthand
changes = [
    # domain-agent
    ("crates/domain-agent/src/lib.rs", "                    tenant_id,", "                    tenant_id: TenantId(tenant_id),"),
]

for file_path, old, new in changes:
    full = WORKDIR / file_path
    if not full.exists():
        print(f"SKIP: {file_path}")
        continue
    content = full.read_text(encoding="utf-8")
    n = content.count(old)
    if n == 0:
        print(f"NOT FOUND: {old[:50]} in {file_path}")
        continue
    content = content.replace(old, new)
    full.write_text(content, encoding="utf-8")
    print(f"FIXED: {file_path} ({n} occurrences)")
