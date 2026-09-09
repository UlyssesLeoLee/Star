#!/usr/bin/env python3
r"""H3 as_uuid refactor: 4 macros return &uuid::Uuid, should return uuid::Uuid (Copy).

per WBS v0.33 §14.17 H3 as_uuid + HANDOFF-ST-001 §1 H3 + 守门 #1 v25 单 crate 模式.
4 files: domain-feedback, domain-integration, domain-theme, domain-validation.
"""
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(r"D:\Star")
TARGETS = [
    "crates/domain-feedback/src/macros.rs",
    "crates/domain-integration/src/macros.rs",
    "crates/domain-theme/src/macros.rs",
    "crates/domain-validation/src/macros.rs",
]
OLD = "pub fn as_uuid(&self) -> &uuid::Uuid {\n                &self.0\n            }"
NEW = "pub fn as_uuid(&self) -> uuid::Uuid {\n                self.0\n            }"


def main() -> int:
    changed = 0
    for rel in TARGETS:
        p = ROOT / rel
        t = p.read_text(encoding="utf-8")
        if OLD not in t:
            print(f"[skip] {rel} (no OLD pattern or already fixed)")
            continue
        if NEW in t:
            print(f"[skip] {rel} (already fixed)")
            continue
        new_t = t.replace(OLD, NEW, 1)
        p.write_text(new_t, encoding="utf-8")
        print(f"[ok]   {rel}: &uuid::Uuid -> uuid::Uuid (Copy)")
        changed += 1
    print(f"\nchanged: {changed}/{len(TARGETS)}")
    return 0 if changed > 0 else 0


if __name__ == "__main__":
    sys.exit(main())
