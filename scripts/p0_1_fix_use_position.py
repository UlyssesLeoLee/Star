#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
P0-1 fix 3: 把所有 crate 的 `pub use star_context::ActorContext;` 移到顶部
- bug: 上次脚本的"找最后 use 之后插入"逻辑错位
- fix: 直接在文件顶部 (前 30 行内), 紧跟其他 use 之后, 插入
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


def fix_use_position(crate_dir: Path) -> str:
    lib_rs = crate_dir / "src" / "lib.rs"
    if not lib_rs.exists():
        return "no lib.rs"

    text = lib_rs.read_text(encoding="utf-8")

    # 1. 删除所有现有的 pub use star_context::ActorContext;
    text = re.sub(r'^\s*pub use star_context::ActorContext;\s*\n', '', text, flags=re.MULTILINE)

    # 2. 找第一个 use 语句的位置, 插入到最后一个 use 后
    lines = text.split("\n")

    # 找顶部 import 块的"最后一个连续 use 行"
    last_use_idx = -1
    for i, line in enumerate(lines):
        if re.match(r'^\s*(pub\s+)?use\s+', line):
            last_use_idx = i
        elif last_use_idx >= 0:
            # 离开 use 块 (空行或非 use 行)
            if line.strip() == '':
                continue
            break

    if last_use_idx < 0:
        return "no use block found"

    # 插入到 last_use_idx 之后
    lines.insert(last_use_idx + 1, 'pub use star_context::ActorContext;')
    new_text = "\n".join(lines)

    if not DRY_RUN:
        lib_rs.write_text(new_text, encoding="utf-8")
    return f"inserted at line {last_use_idx + 2}"


def main() -> int:
    if DRY_RUN:
        print("DRY-RUN")
    else:
        print("APPLY")

    for crate in TARGETS:
        d = WORKSPACE / crate
        result = fix_use_position(d)
        print(f"{crate:30} {result}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
