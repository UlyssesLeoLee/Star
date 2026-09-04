#!/usr/bin/env python3
"""fix_b4_batch_v12.py: Wrap `tenant_id,` shorthand in domain-context tests block.
   Same pattern as fix_b4_batch_v11.py but for domain-context."""
import re
import sys
from pathlib import Path

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

dry_run = "--dry-run" in sys.argv
root = Path(r"D:\Star\.worktrees\feat-auto-20260904-1c260bc7")

fp = root / "crates/domain-context/src/lib.rs"
text = fp.read_text(encoding="utf-8")
test_start = text.find("mod tests {")
if test_start < 0:
    print("ERR: tests block not found")
    sys.exit(1)
pre = text[:test_start]
test_block = text[test_start:]

# Find all `                    tenant_id,` (16+ spaces indent, struct field shorthand)
# But be careful: there may be a `tenant_id: TenantId(tenant_id),` already
n = 0
shorthand = re.compile(r"^(\s{16,})tenant_id,$", re.MULTILINE)
for m in list(shorthand.finditer(test_block)):
    indent = m.group(1)
    old = m.group(0)
    if f"{indent}tenant_id: TenantId(tenant_id)," in test_block:
        continue  # already done
    new = f"{indent}tenant_id: TenantId(tenant_id),"
    test_block = test_block.replace(old, new, 1)
    n += 1

# Also handle 8-space indent cases for the cmd_t / actor_t (if any)
short_shorthand = re.compile(r"^(\s{12,15})tenant_id,$", re.MULTILINE)
for m in list(short_shorthand.finditer(test_block)):
    indent = m.group(1)
    old = m.group(0)
    new = f"{indent}tenant_id: TenantId(tenant_id),"
    test_block = test_block.replace(old, new, 1)
    n += 1

print(f"Wrapped {n} tenant_id shorthand in domain-context")
if not dry_run:
    fp.write_text(pre + test_block, encoding="utf-8")
