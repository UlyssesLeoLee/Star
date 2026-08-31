#!/usr/bin/env python3
"""
P0-1 add uuid dep to domain Cargo.toml 缺 uuid 的
"""
import re
import sys
from pathlib import Path

WORKSPACE = Path(r"D:\Star\crates")
DOMAINS = [
    "domain-feedback", "domain-project", "domain-work-item", "domain-worktree",
]

DRY_RUN = "--apply" not in sys.argv


def main() -> int:
    print("APPLY" if not DRY_RUN else "DRY-RUN")
    for d in DOMAINS:
        f = WORKSPACE / d / "Cargo.toml"
        if not f.exists():
            continue
        text = f.read_text(encoding="utf-8")
        if "uuid = { workspace = true }" in text:
            continue
        if "[dependencies]" in text:
            lines = text.split("\n")
            in_deps = False
            insert_idx = None
            for i, line in enumerate(lines):
                if re.match(r"^\[dependencies\]", line):
                    in_deps = True
                    continue
                if in_deps:
                    if re.match(r"^\s*\[", line):
                        insert_idx = i
                        break
                    if line.strip():
                        insert_idx = i + 1
            if insert_idx is None:
                insert_idx = len(lines)
            lines.insert(insert_idx, "uuid = { workspace = true }")
            new_text = "\n".join(lines)
            if not DRY_RUN:
                f.write_text(new_text, encoding="utf-8")
            print(f"{d}/Cargo.toml 1 patch (add uuid)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
