#!/usr/bin/env python3
"""fix_b4_batch_v15.py: domain-collaboration Whiteboard::new + others"""
import re
import sys
from pathlib import Path

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

dry_run = "--dry-run" in sys.argv
root = Path(r"D:\Star\.worktrees\feat-auto-20260904-1c260bc7")

fp = root / "crates/domain-collaboration/src/lib.rs"
text = fp.read_text(encoding="utf-8")
test_start = text.find("mod tests {")
pre = text[:test_start]
test_block = text[test_start:]

# Pattern: `Whiteboard::new(<uuid_var>, <project_var>, ...)`  -> `Whiteboard::new(TenantId(<uuid_var>), <project_var>, ...)`
# Find: `Whiteboard::new(<name>, ` where <name> is a uuid var
n = 0
pat = re.compile(r"Whiteboard::new\((\w+), (project),")
for m in list(pat.finditer(test_block)):
    var = m.group(1)
    proj = m.group(2)
    old = m.group(0)
    new = f"Whiteboard::new(TenantId({var}), {proj},"
    test_block = test_block.replace(old, new, 1)
    n += 1

print(f"Whiteboard::new wrapped {n} occurrences")
if not dry_run:
    fp.write_text(pre + test_block, encoding="utf-8")

# Also look at remaining err contexts
col_path = root / "crates/domain-collaboration/src/lib.rs"
col_text = col_path.read_text(encoding="utf-8")
col_lines = col_text.split("\n")
for ln in [1700, 1744, 1814, 1851, 1871, 1889, 1936, 1978]:
    if len(col_lines) >= ln:
        print(f"  L{ln}: {col_lines[ln-1].strip()[:200]}")
