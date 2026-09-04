#!/usr/bin/env python3
"""
scripts/automation/fix_b4_batch_v6.py v0.1
Phase B.4 sub-session #6 fixer v0.6: --all-targets 80 err 4 file 4 pattern

Pattern map:
- domain-feedback (60 err):
  - P1: make_create_cmd(\n  indent tenant_id,\n  -> make_create_cmd(\n  indent TenantId(tenant_id),\n
  - P2: make_create_cmd(\n  indent tenant_a,\n  -> make_create_cmd(\n  indent TenantId(tenant_a),\n
  - P3: TransitionFeedbackStatusCommand { tenant_id,\n  -> { tenant_id: TenantId(tenant_id),\n
- domain-work-item (15 err):
  - P4: basic_cmd(tid) -> basic_cmd(TenantId(tid)) (单行/多行)
  - P5: create_work_item(basic_cmd(tid), ...) -> TenantId wrap
- domain-scm (4 err):
  - P6: *project_id.as_uuid() -> project_id.as_uuid()  (Uuid deref)
  - P7: tenant_id: tenant_a,  (struct literal field shorthand type mismatch) -> tenant_id: TenantId(tenant_a),
  - P8: tenant_id: UserId::new(),  (struct literal field value type mismatch) -> tenant_id: TenantId::new(),
- domain-audit (1 err):
  - P9: approver_user_id: Some(uuid::Uuid::new_v4()), -> Some(UserId::new()),

Usage: python fix_b4_batch_v6.py [--dry-run]
"""
import re
import sys
from pathlib import Path

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

dry_run = "--dry-run" in sys.argv
root = Path(r"D:\Star\.worktrees\feat-auto-20260904-1c260bc7")

stats = {"files_touched": 0, "changes": 0}


def patch_file(rel_path: str, edits: list[tuple[str, str, str]]) -> None:
    """Apply (label, old, new) edits to file. Report and track."""
    fp = root / rel_path
    text = fp.read_text(encoding="utf-8")
    n0 = text.count(edits[0][1]) if edits else 0
    new_text = text
    for label, old, new in edits:
        if old not in new_text:
            print(f"  MISS [{label}]: pattern not found in {rel_path}")
            continue
        cnt = new_text.count(old)
        new_text = new_text.replace(old, new)
        print(f"  OK  [{label}]: {cnt} replacements in {rel_path}")
        stats["changes"] += cnt
    if new_text != text:
        stats["files_touched"] += 1
        if not dry_run:
            fp.write_text(new_text, encoding="utf-8")


# ---------- domain-feedback 60 err ----------
# Build edits: for each make_create_cmd(\n    X,\n where X is a uuid var name, wrap it.
# Read the file once to construct edits.
fb_path = root / "crates/domain-feedback/src/lib.rs"
fb_text = fb_path.read_text(encoding="utf-8")

# P1+P2: make_create_cmd(\n    <indent>name,\n  ->  make_create_cmd(\n    <indent>TenantId(name),\n
p1p2 = re.compile(
    r"(make_create_cmd\(\n(\s+))(\w+),\n",
    re.MULTILINE,
)
fb_edits_p1p2: list[tuple[str, str, str]] = []
counter = {"n": 0}
def p1p2_repl(m: re.Match) -> str:
    counter["n"] += 1
    label = f"P1P2_{counter['n']:02d}"
    indent = m.group(2)
    name = m.group(3)
    fb_edits_p1p2.append((label, m.group(0), f"{m.group(1)}TenantId({name}),\n"))
    return f"__PLACEHOLDER_{label}__"
fb_text_tmp = p1p2.sub(p1p2_repl, fb_text)
# now fb_edits_p1p2 has the list, and fb_text_tmp has placeholders. Reconstruct the real text
# by applying the edits:
for label, old, new in fb_edits_p1p2:
    placeholder = f"__PLACEHOLDER_{label}__"
    fb_text_tmp = fb_text_tmp.replace(placeholder, new, 1)

# P3: TransitionFeedbackStatusCommand { ... tenant_id,\n ... from:
# We need to find lines like "                    tenant_id,\n"  that precede "from:" with 4 lines.
# Look for blocks: TransitionFeedbackStatusCommand { ... tenant_id, from:
p3 = re.compile(
    r"(\{\n(\s+)tenant_id,\n(\s+)from:)",
    re.MULTILINE,
)
fb_edits_p3: list[tuple[str, str, str]] = []
counter3 = {"n": 0}
def p3_repl(m: re.Match) -> str:
    counter3["n"] += 1
    label = f"P3_{counter3['n']:02d}"
    indent = m.group(2)
    from_indent = m.group(3)
    fb_edits_p3.append((label, m.group(0), f"{{\n{indent}tenant_id: TenantId(tenant_id),\n{from_indent}from:"))
    return f"__PLACEHOLDER_{label}__"
fb_text_tmp2 = p3.sub(p3_repl, fb_text_tmp)
for label, old, new in fb_edits_p3:
    placeholder = f"__PLACEHOLDER_{label}__"
    fb_text_tmp2 = fb_text_tmp2.replace(placeholder, new, 1)

# Write
if fb_text_tmp2 != fb_text:
    cnt1 = sum(1 for e in fb_edits_p1p2)
    cnt3 = sum(1 for e in fb_edits_p3)
    print(f"  domain-feedback: P1P2 x{cnt1} + P3 x{cnt3}")
    stats["changes"] += cnt1 + cnt3
    if not dry_run:
        fb_path.write_text(fb_text_tmp2, encoding="utf-8")
    stats["files_touched"] += 1

# ---------- domain-work-item 15 err ----------
wi_path = root / "crates/domain-work-item/src/lib.rs"
wi_text = wi_path.read_text(encoding="utf-8")
# P4: basic_cmd(tid) and basic_cmd(\n  tid\n) -> basic_cmd(TenantId(tid))
p4a = re.compile(r"basic_cmd\(tid\)")
wi_text_new = p4a.sub("basic_cmd(TenantId(tid))", wi_text)
cnt4a = wi_text.count("basic_cmd(tid)")
# also cmd.tid is a Uuid in some places? Let me also check `create_work_item` second arg patterns:
# We only fix basic_cmd(tid) explicitly.
print(f"  domain-work-item: P4A x{cnt4a}")
stats["changes"] += cnt4a
if wi_text_new != wi_text and not dry_run:
    wi_path.write_text(wi_text_new, encoding="utf-8")
    stats["files_touched"] += 1
elif cnt4a == 0:
    print("  WARN: domain-work-item P4A no matches found")

# ---------- domain-scm 4 err ----------
scm_path = root / "crates/domain-scm/src/lib.rs"
scm_text = scm_path.read_text(encoding="utf-8")
scm_edits = [
    ("P6a", "            .with_project(*project_id.as_uuid())", "            .with_project(project_id.as_uuid())"),
    ("P6b", "            .with_project(*project_a.as_uuid())", "            .with_project(project_a.as_uuid())"),
    ("P7",  "            tenant_id: tenant_a,\n", "            tenant_id: TenantId(tenant_a),\n"),
    ("P8",  "            tenant_id: UserId::new(),\n", "            tenant_id: TenantId::new(),\n"),
]
patch_file("crates/domain-scm/src/lib.rs", scm_edits)

# ---------- domain-audit 1 err ----------
audit_path = root / "crates/domain-audit/src/lib.rs"
audit_text = audit_path.read_text(encoding="utf-8")
audit_old = "                approver_user_id: Some(uuid::Uuid::new_v4()),\n"
audit_new = "                approver_user_id: Some(UserId::new()),\n"
if audit_old in audit_text:
    print("  domain-audit: P9 x1")
    stats["changes"] += 1
    if not dry_run:
        audit_path.write_text(audit_text.replace(audit_old, audit_new), encoding="utf-8")
    stats["files_touched"] += 1
else:
    print("  MISS P9: domain-audit pattern not found")

print(f"\nTOTAL: {stats['files_touched']} files, {stats['changes']} changes (dry_run={dry_run})")
