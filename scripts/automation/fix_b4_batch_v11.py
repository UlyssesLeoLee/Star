#!/usr/bin/env python3
"""
scripts/automation/fix_b4_batch_v11.py v0.1
Phase B.4 sub-session #6 fixer v0.11: domain-feedback 7 tenant_id shorthand + 1 fix

Pattern: TransitionFeedbackStatusCommand { feedback_id: fid, tenant_id, ... }
  -> { feedback_id: fid, tenant_id: TenantId(tenant_id), ... }
"""
import re
import sys
from pathlib import Path

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

dry_run = "--dry-run" in sys.argv
root = Path(r"D:\Star\.worktrees\feat-auto-20260904-1c260bc7")

# Approach: in tests block, find all `let tenant_id = uuid::Uuid::new_v4();` declarations
# Then for each, in subsequent struct literal `tenant_id,` (no colon), change to `tenant_id: TenantId(tenant_id),`
# But we need to be careful: `let tenant_id = TenantId(...);` (already TenantId) - skip

# Simpler: find pattern in test block: `tenant_id,` at column 21+ preceded by struct brace,
# and the variable was defined as `let tenant_id = uuid::Uuid::new_v4()` in same function

# Most pragmatic: in test block, find ALL `tenant_id,` lines that appear in TransitionFeedbackStatusCommand
# and wrap them. Looking at the pattern:
#   TransitionFeedbackStatusCommand {
#       feedback_id: fid,
#       tenant_id,                      <-- need to wrap
#       from: ...
# We can do: in test block, after `let tenant_id = uuid::Uuid::new_v4();` (Uuid declaration),
# replace `\n(\s+)tenant_id,\n` with `\n\1tenant_id: TenantId(tenant_id),\n`

fp = root / "crates/domain-feedback/src/lib.rs"
text = fp.read_text(encoding="utf-8")
test_start = text.find("mod tests {")
if test_start < 0:
    print("ERR: tests block not found")
    sys.exit(1)
pre = text[:test_start]
test_block = text[test_start:]

# Strategy: scan test block by `#[test]` or `async fn` functions.
# Within each function, if `let tenant_id = uuid::Uuid::new_v4();` (or `let tenant_id = uuid::Uuid::new_v4();`)
# is present, and there's a struct literal with `tenant_id,` shorthand, replace.
# But simpler: just look for `tenant_id,` pattern inside TransitionFeedbackStatusCommand
# and the L641 issue is the special one.
# For now, do exact string replacement for each line.

# Find each `let tenant_id = uuid::Uuid::new_v4();` declaration, then
# find the following `tenant_id,` shorthand in the SAME function

# Pragmatic regex: in test_block, find all `                    tenant_id,\n` and wrap them.
# But this is too aggressive (could affect valid cases). Let's be specific.

# Get list of all `let tenant_id = uuid::Uuid::new_v4();` decls and their line numbers
decl_pattern = re.compile(r"^(\s+)let tenant_id = uuid::Uuid::new_v4\(\);", re.MULTILINE)
decls = list(decl_pattern.finditer(test_block))
print(f"Found {len(decls)} Uuid tenant_id declarations in tests block")

# Find all `tenant_id,` shorthand patterns at indentation ≥ 12 (struct field)
shorthand_pattern = re.compile(r"^(\s{16,})tenant_id,$", re.MULTILINE)

new_test_block = test_block
n_shorthand = 0
for m in list(shorthand_pattern.finditer(test_block)):
    line_start_in_block = m.start()
    # Get context above to see if there's a struct literal opening
    # For each `tenant_id,` shorthand, just wrap it.
    # This is a heuristic but should be OK for the 7 known locations.
    indent = m.group(1)
    old = m.group(0)
    new = f"{indent}tenant_id: TenantId(tenant_id),"
    if old in new_test_block:
        new_test_block = new_test_block.replace(old, new, 1)
        n_shorthand += 1

print(f"Wrapped {n_shorthand} `tenant_id,` shorthand -> `tenant_id: TenantId(tenant_id),`")

# Also fix L590: ActorContext::new(uuid::Uuid::new_v4(), *tenant_id.as_uuid()) -> .as_uuid() on TenantId
# But TenantId's as_uuid returns Uuid (owned) so * is wrong
# Find this pattern in test block
old_l590 = "                ActorContext::new(uuid::Uuid::new_v4(), *tenant_id.as_uuid())"
new_l590 = "                ActorContext::new(uuid::Uuid::new_v4(), tenant_id.as_uuid())"
if old_l590 in new_test_block:
    new_test_block = new_test_block.replace(old_l590, new_l590, 1)
    print("Fixed L590: removed * from *tenant_id.as_uuid()")
    n_shorthand += 1

# Write
if new_test_block != test_block:
    full_new = pre + new_test_block
    if not dry_run:
        fp.write_text(full_new, encoding="utf-8")
    print(f"WRITTEN: {n_shorthand} edits")
else:
    print("NO CHANGES")
