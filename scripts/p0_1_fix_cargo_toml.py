#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
修 P0-1 第一次 apply 时 Cargo.toml 插入错位的 bug
- bug: 错位到 [lints] 段后, 导致 cargo 解析失败
- fix: 把误插的 star-context 行移到正确位置 ([dependencies] 段末尾, 下一段前)
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


def fix_cargo_toml(crate_dir: Path) -> str:
    cargo = crate_dir / "Cargo.toml"
    if not cargo.exists():
        return "no Cargo.toml"

    text = cargo.read_text(encoding="utf-8")

    # 1. 找误插的 star-context 行 (在 [lints] 段后)
    # 模式: [lints]\nworkspace = true\n\nstar-context = { path = "../star-context" }
    star_ctx_line = 'star-context = { path = "../star-context" }'
    if star_ctx_line not in text:
        return "no misplaced star-context (already correct or missing)"

    # 2. 移除所有 star-context 行 (不管位置, 重新插入)
    lines = text.split("\n")
    lines = [l for l in lines if l.strip() != star_ctx_line]
    text = "\n".join(lines)

    # 3. 找到正确插入位置: [dependencies] 段末尾 (下一段 [xxx] 之前)
    lines = text.split("\n")
    new_lines = []
    in_deps = False
    inserted = False

    for i, line in enumerate(lines):
        # 检测段头: 带前导空格的 [xxx]
        if re.match(r"^\s*\[", line):
            stripped = line.strip()
            if stripped == "[dependencies]":
                in_deps = True
                new_lines.append(line)
                continue
            elif in_deps and not inserted:
                # 进入下一段, 先插入 star-context
                new_lines.append(star_ctx_line)
                inserted = True
                in_deps = False
        new_lines.append(line)

    # 如果 [dependencies] 是最后一段, 在末尾插入
    if in_deps and not inserted:
        new_lines.append(star_ctx_line)
        inserted = True

    new_text = "\n".join(new_lines)
    if not DRY_RUN:
        cargo.write_text(new_text, encoding="utf-8")
    return f"fixed, inserted={inserted}"


def main() -> int:
    if DRY_RUN:
        print("DRY-RUN")
    else:
        print("APPLY")

    for crate in TARGETS:
        d = WORKSPACE / crate
        result = fix_cargo_toml(d)
        print(f"{crate:30} {result}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
