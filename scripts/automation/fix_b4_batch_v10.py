#!/usr/bin/env python3
"""
scripts/automation/fix_b4_batch_v10.py v0.1
Phase B.4 sub-session #6 fixer v0.10: domain-search 18 err + domain-planning 8 err

Pattern (domain-search):
- S1: projector_actor(<uuid_var>) -> projector_actor(TenantId(<uuid_var>))
- S2: sample_index_cmd(<uuid_var>, ...) -> sample_index_cmd(TenantId(<uuid_var>), ...)
- S3: make_actor(TenantId(<uuid_var>), <uuid_var>) -> make_actor(TenantId(<uuid_var>), UserId(<uuid_var>))
- S4: `user_id: me,` (struct field, 期望 UserId) -> `user_id: UserId(me),`
- S5: `actor_user_id: me,` -> `actor_user_id: UserId(me),`
- S6: `tenant_id: t1,` (struct field, 期望 TenantId) -> `tenant_id: TenantId(t1),`
- S7: `tenant_id: tenant_id,` -> 保留 (已经是 TenantId, 仅当 var 是 TenantId 类型) - skip
- S8: list_saved(tenant_id, ...) -> list_saved(TenantId(tenant_id), ...) (tenant_id 期望 TenantId)
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
    fp = root / rel
    text = fp.read_text(encoding="utf-8")
    if old not in text:
        print(f"  MISS {label}: not found")
        return
    n = text.count(old)
    text2 = text.replace(old, new, 1)
    print(f"  OK   {label}: {n}")
    stats["edits"] += n
    if not dry_run:
        fp.write_text(text2, encoding="utf-8")
    stats["files"] += 1


# ---- domain-search ----
s_path = root / "crates/domain-search/src/lib.rs"
s_text = s_path.read_text(encoding="utf-8")
s_new = s_text

# S1: projector_actor(<uuid_var>)  -> projector_actor(TenantId(<uuid_var>))
# Only for tests block, lines starting with whitespace + `let p1 = projector_actor(`
# safer: replace specific patterns
patch(
    "crates/domain-search/src/lib.rs",
    "        let p1 = projector_actor(t1);",
    "        let p1 = projector_actor(TenantId(t1));",
    "search S1a",
)
patch(
    "crates/domain-search/src/lib.rs",
    "        let p2 = projector_actor(t2);",
    "        let p2 = projector_actor(TenantId(t2));",
    "search S1b",
)

# S2: sample_index_cmd(t1, ResourceType::WorkItem, "tenant1 doc") -> TenantId(t1, ...)
patch(
    "crates/domain-search/src/lib.rs",
    '            sample_index_cmd(t1, ResourceType::WorkItem, "tenant1 doc"),',
    '            sample_index_cmd(TenantId(t1), ResourceType::WorkItem, "tenant1 doc"),',
    "search S2a",
)
patch(
    "crates/domain-search/src/lib.rs",
    '            sample_index_cmd(t2, ResourceType::WorkItem, "tenant2 doc"),',
    '            sample_index_cmd(TenantId(t2), ResourceType::WorkItem, "tenant2 doc"),',
    "search S2b",
)

# S3: make_actor(TenantId(t1), uuid::Uuid::new_v4()) -> make_actor(TenantId(t1), UserId(uuid::Uuid::new_v4()))
# Don't touch - already uses uuid::Uuid::new_v4() in some places, just need UserId wrap
# Actually, lines look like: `let user1 = make_actor(TenantId(t1), uuid::Uuid::new_v4());`
# We need to wrap the second arg with UserId
# Pattern: make_actor(TenantId(<t>), uuid::Uuid::new_v4()) -> make_actor(TenantId(<t>), UserId(uuid::Uuid::new_v4()))
patch(
    "crates/domain-search/src/lib.rs",
    "        let user1 = make_actor(TenantId(t1), uuid::Uuid::new_v4());",
    "        let user1 = make_actor(TenantId(t1), UserId(uuid::Uuid::new_v4()));",
    "search S3a",
)
patch(
    "crates/domain-search/src/lib.rs",
    "        let user2 = make_actor(TenantId(t2), uuid::Uuid::new_v4());",
    "        let user2 = make_actor(TenantId(t2), UserId(uuid::Uuid::new_v4()));",
    "search S3b",
)

# S3b: make_actor(TenantId(tenant_id), me)  where me is uuid var
patch(
    "crates/domain-search/src/lib.rs",
    "        let actor = make_actor(TenantId(tenant_id), me);",
    "        let actor = make_actor(TenantId(tenant_id), UserId(me));",
    "search S3c",
)
patch(
    "crates/domain-search/src/lib.rs",
    "        let actor_me = make_actor(TenantId(tenant_id), me);",
    "        let actor_me = make_actor(TenantId(tenant_id), UserId(me));",
    "search S3d",
)
patch(
    "crates/domain-search/src/lib.rs",
    "        let actor_other = make_actor(TenantId(tenant_id), other);",
    "        let actor_other = make_actor(TenantId(tenant_id), UserId(other));",
    "search S3e",
)

# S4 + S5: in struct literal
# Look for `user_id: me,` in UpsertIndexCommand / SearchQuery / SaveSearchCommand (where me: Uuid)
# These all have `me: uuid::Uuid::new_v4()` so we need `user_id: UserId(me)`
# Apply globally only in test block (lines 900+ which is after mod tests {)

s_path = root / "crates/domain-search/src/lib.rs"
s_text = s_path.read_text(encoding="utf-8")
test_start = s_text.find("mod tests {")
if test_start > 0:
    pre = s_text[:test_start]
    test_block = s_text[test_start:]
    # S4: in struct literal, `user_id: me,` -> `user_id: UserId(me),`
    test_block_new = re.sub(r"(\s+)user_id: me,", r"\1user_id: UserId(me),", test_block)
    n_s4 = test_block.count("user_id: me,") - test_block_new.count("user_id: UserId(me),")
    # wait: count was wrong. Let's count originals:
    n_s4_orig = test_block.count("user_id: me,")
    n_s4_new = test_block_new.count("user_id: UserId(me),")
    print(f"  OK   search S4: {n_s4_orig} -> {n_s4_new} (期望 +{n_s4_orig})")
    # S5: `actor_user_id: me,` -> `actor_user_id: UserId(me),`
    test_block_new = re.sub(r"(\s+)actor_user_id: me,", r"\1actor_user_id: UserId(me),", test_block_new)
    n_s5 = test_block.count("actor_user_id: me,")
    print(f"  OK   search S5: {n_s5}")
    # S6: `tenant_id: t1,` in struct literal -> `tenant_id: TenantId(t1),`
    test_block_new = re.sub(r"(\s+)tenant_id: t1,", r"\1tenant_id: TenantId(t1),", test_block_new)
    n_s6 = test_block.count("tenant_id: t1,")
    print(f"  OK   search S6: {n_s6}")
    # S8: list_saved(tenant_id, ...) -> list_saved(TenantId(tenant_id), ...)
    test_block_new = re.sub(
        r"list_saved\(tenant_id,",
        "list_saved(TenantId(tenant_id),",
        test_block_new,
    )
    n_s8 = test_block.count("list_saved(tenant_id,")
    print(f"  OK   search S8: {n_s8}")
    # write
    if test_block_new != test_block:
        s_text_new = pre + test_block_new
        stats["edits"] += n_s4_orig + n_s5 + n_s6 + n_s8
        if not dry_run:
            s_path.write_text(s_text_new, encoding="utf-8")
        stats["files"] += 1

# ---- domain-planning ----
print("\n--- domain-planning ---")
patch(
    "crates/domain-planning/src/lib.rs",
    "    fn make_admin_actor(tenant_id: TenantId, project_id: ProjectId) -> ActorContext {\n        ActorContext::new(Uuid::new_v4(), tenant_id.0)\n            .with_role(roles::PROJECT_ADMIN)\n            .with_project(*project_id.as_uuid())\n    }",
    "    fn make_admin_actor(tenant_id: TenantId, project_id: ProjectId) -> ActorContext {\n        ActorContext::new(Uuid::new_v4(), tenant_id.0)\n            .with_role(roles::PROJECT_ADMIN)\n            .with_project(project_id.as_uuid())\n    }",
    "plan P1a",
)
patch(
    "crates/domain-planning/src/lib.rs",
    "    fn make_dev_actor(tenant_id: TenantId, project_id: ProjectId) -> ActorContext {\n        ActorContext::new(Uuid::new_v4(), tenant_id.0)\n            .with_role(roles::DEVELOPER)\n            .with_project(*project_id.as_uuid())\n    }",
    "    fn make_dev_actor(tenant_id: TenantId, project_id: ProjectId) -> ActorContext {\n        ActorContext::new(Uuid::new_v4(), tenant_id.0)\n            .with_role(roles::DEVELOPER)\n            .with_project(project_id.as_uuid())\n    }",
    "plan P1b",
)
# P2: cross_tenant_sprint_denied - actor_tenant.0 -> actor_tenant (Uuid direct), *project_id -> project_id
patch(
    "crates/domain-planning/src/lib.rs",
    "        let actor = ActorContext::new(Uuid::new_v4(), actor_tenant.0)\n            .with_role(roles::PROJECT_ADMIN)\n            .with_project(*project_id.as_uuid());",
    "        let actor = ActorContext::new(Uuid::new_v4(), actor_tenant)\n            .with_role(roles::PROJECT_ADMIN)\n            .with_project(project_id.as_uuid());",
    "plan P2a",
)
# P3: tenant_id: cmd_tenant, (shorthand) -> tenant_id: TenantId(cmd_tenant),
patch(
    "crates/domain-planning/src/lib.rs",
    "            tenant_id: cmd_tenant,\n            project_id,",
    "            tenant_id: TenantId(cmd_tenant),\n            project_id,",
    "plan P3a",
)
# P4: L1645 actor_b - tenant_b.0 -> tenant_b (Uuid direct)
patch(
    "crates/domain-planning/src/lib.rs",
    "        let actor_b = ActorContext::new(Uuid::new_v4(), tenant_b.0).with_role(roles::PROJECT_ADMIN);",
    "        let actor_b = ActorContext::new(Uuid::new_v4(), tenant_b).with_role(roles::PROJECT_ADMIN);",
    "plan P4a",
)
# P5: L1649 tenant_id: tenant_b, -> TenantId wrap
patch(
    "crates/domain-planning/src/lib.rs",
    "                    tenant_id: tenant_b,\n                    sprint_id:",
    "                    tenant_id: TenantId(tenant_b),\n                    sprint_id:",
    "plan P5a",
)

print(f"\nTOTAL: {stats['files']} files, {stats['edits']} edits (dry_run={dry_run})")
