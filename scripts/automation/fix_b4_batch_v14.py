#!/usr/bin/env python3
"""fix_b4_batch_v14.py: Final wrap of `tenant_id,` shorthand in domain-context."""
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
pre = text[:test_start]
test_block = text[test_start:]

n = 0
# Match `                    tenant_id,` (16+ spaces, just tenant_id, line end)
# Don't match `tenant_id: TenantId(tenant_id),` (already done)
pat = re.compile(r"^(\s{12,})tenant_id,$", re.MULTILINE)
for m in list(pat.finditer(test_block)):
    indent = m.group(1)
    old = m.group(0)
    new = f"{indent}tenant_id: TenantId(tenant_id),"
    test_block = test_block.replace(old, new, 1)
    n += 1

# Also: bare tenant_id as fn arg, like `add_relevant_item(\n    tenant_id,\n    `
# Match indent + `tenant_id,` at line end where indent < 12 (because next line is deeper)
# Already covered above (12+ spaces)

# Method call patterns with bare tenant_id: replace `method(\n  tenant_id,` etc
# Actually we need to match where tenant_id is the FIRST argument
# Pattern: `<method>(\n  ...\n  tenant_id,\n` where ... is anything except `TenantId(`
# Simpler: find all lines like `(\s+)tenant_id,` where the previous line ends with `(\n`
pat2 = re.compile(r"(method|_item|list_|mark_|drop_|get_|add_|tag_)\(\n(\s+)tenant_id,", re.MULTILINE)
# This is hard to do generically. Just do the L1066 pattern specifically:
# `CreateContextPacketCommand {\n` ... `\n                    tenant_id,` -> `tenant_id: TenantId(tenant_id),`
# Already covered by pat above

# Also: other method calls like `add_relevant_item(\n                tenant_id,`
# The fix is the same.

# Look for any remaining bare `tenant_id,` (16+ spaces):
remaining = []
for line in test_block.split("\n"):
    if re.match(r"^\s{12,}tenant_id,$", line):
        remaining.append(line)
print(f"Wrapped {n} bare tenant_id, shorthand in domain-context")
if remaining:
    print(f"Remaining bare tenant_id,: {len(remaining)}")
    for r in remaining[:5]:
        print(f"  {r[:100]}")

if not dry_run:
    fp.write_text(pre + test_block, encoding="utf-8")

# Also fix domain-notification
nf_path = root / "crates/domain-notification/src/lib.rs"
nf_text = nf_path.read_text(encoding="utf-8")
nf_lines = nf_text.split("\n")
if len(nf_lines) >= 1130:
    for i in range(max(0, 1128), min(len(nf_lines), 1132)):
        print(f"  domain-notification L{i+1}: {nf_lines[i].strip()[:200]}")
