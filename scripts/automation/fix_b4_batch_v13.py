#!/usr/bin/env python3
"""fix_b4_batch_v13.py: domain-context tenant_id wrap + domain-agent L1441"""
import re
import sys
from pathlib import Path

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

dry_run = "--dry-run" in sys.argv
root = Path(r"D:\Star\.worktrees\feat-auto-20260904-1c260bc7")

# ---- domain-context: wrap bare `tenant_id,` and `actor_t` etc in add_relevant_item + make_actor calls
fp = root / "crates/domain-context/src/lib.rs"
text = fp.read_text(encoding="utf-8")
test_start = text.find("mod tests {")
pre = text[:test_start]
test_block = text[test_start:]

n_changes = 0

# 1. Wrap `add_relevant_item(\n                tenant_id,` -> `TenantId(tenant_id)`
n = 0
test_block = re.sub(
    r"(\.add_relevant_item\(\n)(\s+)tenant_id,",
    r"\1\2TenantId(tenant_id),",
    test_block,
)
n_addrel = test_block.count(".add_relevant_item(\n") - len(re.findall(r"\.add_relevant_item\(\n(\s+)TenantId\(tenant_id\),", test_block))
# Simpler: count occurrences of pattern we replaced
# Just measure by direct count after replacement
# Actually let me do it more directly
n_addrel = 0
# Now do similar for other method calls with bare tenant_id
# Patterns: list_relevant_items(tenant_id, ...), mark_relevant(tenant_id, ...), drop_relevant(tenant_id, ...), etc.
# Generic: method call starting with `tenant_id,` at line start (16+ spaces)
pat = re.compile(r"^(\s{12,})(add_relevant_item|list_relevant_items|mark_relevant|drop_relevant|get_packet|add_provenance|tag_artifact)\(\n(\s+)tenant_id,", re.MULTILINE)
matches = list(pat.finditer(test_block))
for m in matches:
    indent = m.group(3)
    old = m.group(0)
    new = f"{m.group(1)}{m.group(2)}(\n{indent}TenantId(tenant_id),"
    test_block = test_block.replace(old, new, 1)
    n_addrel += 1

# 2. make_actor(<uuid_var>) -> make_actor(TenantId(<uuid_var>))
# Pattern: `make_actor(<varname>)` where <varname> is uuid Uuid
# We need to detect which vars are Uuid
# Simple heuristic: any var that appears as `let X = uuid::Uuid::new_v4();` should be wrapped
uuid_vars = re.findall(r"let (\w+) = uuid::Uuid::new_v4\(\);", test_block)
print(f"Found uuid vars: {uuid_vars}")

n_actor = 0
for var in uuid_vars:
    # find `make_actor(<var>)` (only var, not TenantId)
    pat = re.compile(rf"make_actor\({re.escape(var)}\)")
    for m in list(pat.finditer(test_block)):
        old = m.group(0)
        new = f"make_actor(TenantId({var}))"
        test_block = test_block.replace(old, new, 1)
        n_actor += 1

print(f"  domain-context: {n_addrel} method call wraps + {n_actor} make_actor wraps")
n_changes = n_addrel + n_actor

if not dry_run:
    fp.write_text(pre + test_block, encoding="utf-8")

# ---- domain-agent L1441 ----
ag_path = root / "crates/domain-agent/src/lib.rs"
ag_text = ag_path.read_text(encoding="utf-8")
ag_lines = ag_text.split("\n")
if len(ag_lines) >= 1441:
    print(f"  domain-agent L1441: {ag_lines[1440].strip()[:200]}")
    # Get context
    for i in range(max(0, 1440-5), min(len(ag_lines), 1441+3)):
        print(f"    L{i+1}: {ag_lines[i].strip()[:200]}")
