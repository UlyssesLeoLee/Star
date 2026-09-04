#!/usr/bin/env python3
"""
scripts/automation/fix_b4_batch_v9.py v0.1
Phase B.4 sub-session #6 fixer v0.9: --all-targets 13 err 3 file

Pattern:
- domain-permission (9 err):
  - R1: `.with_project(*project.as_uuid())` -> `.with_project(project.as_uuid())` (5 处)
  - R2: `tenant_id: tenant_a,` (struct literal field shorthand) -> `tenant_id: TenantId(tenant_a),` (2 处)
  - R3: L1284/1297 mismatched types - struct literal field shorthand (类似 R2)
- domain-automation (3 err):
  - A1: L1098, 1104: `*project.as_uuid()` deref -> remove *
  - A2: L1199: mismatched types - field value type mismatch
- api (1 err):
  - I1: ActorContext { ... } struct literal 缺 `is_agent_session`, `tenant_policy_id`, `workspace_ids` 字段
"""
import re
import sys
from pathlib import Path

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

dry_run = "--dry-run" in sys.argv
root = Path(r"D:\Star\.worktrees\feat-auto-20260904-1c260bc7")

stats = {"files": 0, "edits": 0}


def patch(rel: str, old: str, new: str, label: str) -> None:
    global stats
    fp = root / rel
    text = fp.read_text(encoding="utf-8")
    if old not in text:
        print(f"  MISS {label}: pattern not found in {rel}")
        return
    n = text.count(old)
    text = text.replace(old, new, 1)
    print(f"  OK   {label}: {n} replacement in {rel}")
    stats["edits"] += n
    if not dry_run:
        fp.write_text(text, encoding="utf-8")
    stats["files"] += 1


# ---- domain-permission 9 err ----
# R1: 5 处 *project.as_uuid() -> project.as_uuid()
patch(
    "crates/domain-permission/src/lib.rs",
    "                    .with_project(*project.as_uuid()),",
    "                    .with_project(project.as_uuid()),",
    "perm R1a",
)
patch(
    "crates/domain-permission/src/lib.rs",
    "                        .with_project(*project.as_uuid()),",
    "                        .with_project(project.as_uuid()),",
    "perm R1b",
)
patch(
    "crates/domain-permission/src/lib.rs",
    "            .with_project(*project.as_uuid());",
    "            .with_project(project.as_uuid());",
    "perm R1c",
)
# R2: 2 处 tenant_id: tenant_a, in struct literal -> TenantId wrap
patch(
    "crates/domain-permission/src/lib.rs",
    "                tenant_id: tenant_a,\n                name:",
    "                tenant_id: TenantId(tenant_a),\n                name:",
    "perm R2a",
)
patch(
    "crates/domain-permission/src/lib.rs",
    "                    tenant_id: tenant_a,\n                    name:",
    "                    tenant_id: TenantId(tenant_a),\n                    name:",
    "perm R2b",
)
# R3: L1284/1297 - need to look at these
# They are in CreateSchemeCommand { tenant_id: tenant_a, ... } pattern
# Let me patch them by reading the file
# Actually we already covered those above. Let me also check the GrantRoleCommand
perm_path = root / "crates/domain-permission/src/lib.rs"
perm_text = perm_path.read_text(encoding="utf-8")
# Find lines with "tenant_id: tenant_a," OR "tenant_id: tenant_b,"
# These are field shorthand for Uuid variables
r3a_pat = re.compile(r"(\s+)tenant_id: (tenant_a|tenant_b),")
matches = list(r3a_pat.finditer(perm_text))
print(f"  perm R3 candidates: {len(matches)}")
for m in matches:
    line_start = perm_text[:m.start()].count("\n") + 1
    print(f"    L{line_start}: {m.group(0).strip()}")
    if not dry_run and "TenantId(" not in m.group(0):
        # wrap
        var = m.group(2)
        new_line = f"{m.group(1)}tenant_id: TenantId({var}),"
        perm_text = perm_text.replace(m.group(0), new_line, 1)
        stats["edits"] += 1
if not dry_run:
    perm_path.write_text(perm_text, encoding="utf-8")
    stats["files"] += 1

# ---- domain-automation 3 err ----
patch(
    "crates/domain-automation/src/lib.rs",
    "                .with_project(*project.as_uuid()),",
    "                .with_project(project.as_uuid()),",
    "auto A1a",
)
patch(
    "crates/domain-automation/src/lib.rs",
    "                    .with_project(*project.as_uuid()),",
    "                    .with_project(project.as_uuid()),",
    "auto A1b",
)
# A2: L1199 mismatched - need to look
auto_path = root / "crates/domain-automation/src/lib.rs"
auto_text = auto_path.read_text(encoding="utf-8")
auto_lines = auto_text.split("\n")
if len(auto_lines) >= 1199:
    print(f"  auto A2 L1199: {auto_lines[1198].strip()[:200]}")

# ---- api 1 err ----
# Add is_agent_session, tenant_policy_id, workspace_ids to ActorContext literal
api_old = """        let actor = ActorContext {
            user_id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            device_id: None,
            is_local_runtime: false,
            is_platform_admin: false,
            project_ids: vec![],
            roles: vec!["developer".to_string()],
        };"""
api_new = """        let actor = ActorContext {
            user_id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            device_id: None,
            is_local_runtime: false,
            is_platform_admin: false,
            is_agent_session: false,
            tenant_policy_id: None,
            project_ids: vec![],
            workspace_ids: vec![],
            roles: vec!["developer".to_string()],
        };"""
patch("crates/api/src/lib.rs", api_old, api_new, "api I1")

print(f"\nTOTAL: {stats['files']} files, {stats['edits']} edits (dry_run={dry_run})")
