#!/usr/bin/env python3
"""
scripts/automation/fix_b4_batch_v8.py v0.1
Phase B.4 sub-session #6 fixer v0.8: domain-collaboration 47 err + domain-agent + domain-workspace

Pattern:
- domain-collaboration:
  - Q1: make_actor/make_admin_actor 内部: ActorContext::new(user.0, tenant.0) -> ActorContext::new(user, tenant)
  - Q2: .with_project(project.as_uuid()) -> .with_project(project)
  - Q3: 调用点: make_actor(uuid::Uuid::new_v4(), TenantId(tenant), ProjectId(project)) -> make_actor(UserId::new(), TenantId(tenant), project)
  - Q4: make_actor(other, ...) where other: Uuid -> make_actor(UserId(other), ...)
"""
import re
import sys
from pathlib import Path

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

dry_run = "--dry-run" in sys.argv
root = Path(r"D:\Star\.worktrees\feat-auto-20260904-1c260bc7")

stats = {"files": 0, "edits": 0}

# ---- domain-collaboration ----
col_path = root / "crates/domain-collaboration/src/lib.rs"
col_text = col_path.read_text(encoding="utf-8")
col_new = col_text

# Q1+Q2: make_actor / make_admin_actor 内部修复
helper_old1 = "    fn make_actor(user: UserId, tenant: TenantId, project: ProjectId) -> ActorContext {\n        ActorContext::new(user.0, tenant.0)\n            .with_project(project.as_uuid())\n            .with_role(\"developer\")\n    }"
helper_new1 = "    fn make_actor(user: UserId, tenant: TenantId, project: ProjectId) -> ActorContext {\n        ActorContext::new(user, tenant)\n            .with_project(project)\n            .with_role(\"developer\")\n    }"
if helper_old1 in col_new:
    col_new = col_new.replace(helper_old1, helper_new1, 1)
    print("  OK   domain-collaboration: make_actor fix")
    stats["edits"] += 1

helper_old2 = "    fn make_admin_actor(user: UserId, tenant: TenantId, project: ProjectId) -> ActorContext {\n        ActorContext::new(user.0, tenant.0)\n            .with_project(project.as_uuid())\n            .with_role(\"tenant_admin\")\n    }"
helper_new2 = "    fn make_admin_actor(user: UserId, tenant: TenantId, project: ProjectId) -> ActorContext {\n        ActorContext::new(user, tenant)\n            .with_project(project)\n            .with_role(\"tenant_admin\")\n    }"
if helper_old2 in col_new:
    col_new = col_new.replace(helper_old2, helper_new2, 1)
    print("  OK   domain-collaboration: make_admin_actor fix")
    stats["edits"] += 1

# Q3: call sites with `make_actor(uuid::Uuid::new_v4(), TenantId(tenant), ProjectId(project))`
# This pattern: where `let project = ProjectId::new();` then `ProjectId(project)` re-wraps a Uuid
# Solution: use UserId::new() and bare project.
# Use a regex to find the call sites.
# Pattern: make_actor(uuid::Uuid::new_v4(), TenantId(<var>), ProjectId(<var>))
q3_pat = re.compile(
    r"make_actor\(uuid::Uuid::new_v4\(\), TenantId\((\w+)\), ProjectId\((\w+)\)\)"
)
q3_new = "make_actor(UserId::new(), TenantId(\\1), \\2)"
before_q3 = col_new
col_new = q3_pat.sub(q3_new, col_new)
n_q3 = (before_q3.count(q3_pat.pattern) if False else len(q3_pat.findall(before_q3)))
print(f"  OK   domain-collaboration: make_actor Q3 x{n_q3}")
stats["edits"] += n_q3

# Same for make_admin_actor
q3b_pat = re.compile(
    r"make_admin_actor\(uuid::Uuid::new_v4\(\), TenantId\((\w+)\), ProjectId\((\w+)\)\)"
)
q3b_new = "make_admin_actor(UserId::new(), TenantId(\\1), \\2)"
before_q3b = col_new
col_new = q3b_pat.sub(q3b_new, col_new)
n_q3b = len(q3b_pat.findall(before_q3b))
print(f"  OK   domain-collaboration: make_admin_actor Q3b x{n_q3b}")
stats["edits"] += n_q3b

# Q4: make_actor(other, TenantId(tenant), ProjectId(project))  where `other: Uuid`
# This is when other was declared as Uuid earlier. Use UserId(other) to wrap.
q4_pat = re.compile(
    r"make_actor\(other, TenantId\((\w+)\), ProjectId\((\w+)\)\)"
)
q4_new = "make_actor(UserId(other), TenantId(\\1), \\2)"
before_q4 = col_new
col_new = q4_pat.sub(q4_new, col_new)
n_q4 = len(q4_pat.findall(before_q4))
print(f"  OK   domain-collaboration: make_actor Q4 x{n_q4}")
stats["edits"] += n_q4

# Same for make_admin_actor
q4b_pat = re.compile(
    r"make_admin_actor\(other, TenantId\((\w+)\), ProjectId\((\w+)\)\)"
)
q4b_new = "make_admin_actor(UserId(other), TenantId(\\1), \\2)"
before_q4b = col_new
col_new = q4b_pat.sub(q4b_new, col_new)
n_q4b = len(q4b_pat.findall(before_q4b))
print(f"  OK   domain-collaboration: make_admin_actor Q4b x{n_q4b}")
stats["edits"] += n_q4b

# write
if col_new != col_text:
    stats["files"] += 1
    if not dry_run:
        col_path.write_text(col_new, encoding="utf-8")

# ---- domain-agent 1 err ----
ag_path = root / "crates/domain-agent/src/lib.rs"
ag_text = ag_path.read_text(encoding="utf-8")
# Look at L1441
lines = ag_text.split("\n")
if len(lines) >= 1441:
    line_1441 = lines[1440]
    print(f"  domain-agent L1441: {line_1441.strip()[:200]}")

print(f"\nTOTAL: {stats['files']} files, {stats['edits']} edits (dry_run={dry_run})")
