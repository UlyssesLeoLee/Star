#!/usr/bin/env python3
"""
scripts/automation/fix_b4_batch_v7.py v0.1
Phase B.4 sub-session #6 fixer v0.7: ActorContext 类型冲突 (45+29 err)

根因: crate 顶部 `pub use star_context::ActorContext;` 收敛后, 但 crate 内部 port.rs
      仍 `use crate::context::ActorContext;` (本地). tests 模块 `use super::*;` 拿到
      star_context 版本, 导致 `actor` 变量是 star_context 类型, 不能传给 port.rs
      期望的 context::ActorContext.

修法: 在 `mod tests` 顶部显式 `use crate::context::ActorContext;` 覆盖 super::* 的
      star_context 命名.

适用:
- domain-feedback (45 err)
- domain-integration (29 err)
"""
import re
import sys
from pathlib import Path

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

dry_run = "--dry-run" in sys.argv
root = Path(r"D:\Star\.worktrees\feat-auto-20260904-1c260bc7")

# Pattern: find `mod tests {` followed by `use super::*;` and insert
# `use crate::context::ActorContext;` after the use lines.
P = re.compile(
    r"(#\[cfg\(test\)\]\nmod tests \{\n    use super::\*;)([^\n]*\n)*?",
    re.MULTILINE,
)

changes = {"files": 0, "edits": 0}

for crate_name in ["domain-feedback", "domain-integration"]:
    fp = root / f"crates/{crate_name}/src/lib.rs"
    text = fp.read_text(encoding="utf-8")

    # Find the tests module start
    m = re.search(r"#\[cfg\(test\)\]\nmod tests \{\n    use super::\*;", text)
    if not m:
        print(f"  MISS {crate_name}: tests module + use super::* not found")
        continue
    if "use crate::context::ActorContext;" in text:
        print(f"  SKIP {crate_name}: already has use crate::context::ActorContext")
        continue
    # Insert right after `use super::*;`
    insertion = "    use crate::context::ActorContext; // P0-1 兼容: 显式覆盖 super::* 的 star_context 命名\n"
    new_text = text[:m.end()] + insertion + text[m.end():]
    print(f"  OK   {crate_name}: inserted use crate::context::ActorContext;")
    changes["edits"] += 1
    if not dry_run:
        fp.write_text(new_text, encoding="utf-8")
        changes["files"] += 1

# Also fix domain-integration's make_test_actor: Uuid deref issue
# L91: ActorContext::new(uuid::Uuid::new_v4(), *tenant_id.as_uuid())
#      -> ActorContext::new(UserId::new(), tenant_id)
# L93: .with_project(*ProjectId::new().as_uuid()) -> .with_project(ProjectId::new())
if False:  # disabled for now, may not be needed if tests use crate::context::ActorContext
    pass

# Also fix domain-feedback's make_actor: same issue
# L84: ActorContext::new(uuid::Uuid::new_v4(), *tenant_id.as_uuid()).with_role(...)
#      -> ActorContext::new(UserId::new(), tenant_id).with_role(...)
# context::ActorContext::new signature: (user_id: UserId, tenant_id: TenantId) -> Self
# So we can just use UserId::new() and tenant_id directly.

fb_path = root / "crates/domain-feedback/src/lib.rs"
fb_text = fb_path.read_text(encoding="utf-8")
old = "    fn make_actor(tenant_id: TenantId) -> ActorContext {\n        ActorContext::new(uuid::Uuid::new_v4(), *tenant_id.as_uuid()).with_role(roles::DEVELOPER)\n    }"
new = "    fn make_actor(tenant_id: TenantId) -> ActorContext {\n        ActorContext::new(UserId::new(), tenant_id).with_role(roles::DEVELOPER)\n    }"
if old in fb_text:
    fb_text = fb_text.replace(old, new, 1)
    print("  OK   domain-feedback: make_actor fix (UserId/TenantId direct)")
    changes["edits"] += 1
    if not dry_run:
        fb_path.write_text(fb_text, encoding="utf-8")
        changes["files"] += 1

# Also fix make_create_cmd inner tenant_id shorthand mismatch
# L88-90: CreateFeedbackCommand { tenant_id, ... }  - tenant_id is TenantId, param tenant_id is TenantId, OK
# but the issue is the test code passes the local var `tenant_id` (Uuid) which we just changed to TenantId
# Already done by P1P2 fixer

# domain-integration make_test_actor fix
int_path = root / "crates/domain-integration/src/lib.rs"
int_text = int_path.read_text(encoding="utf-8")
int_old = "    fn make_test_actor(tenant_id: TenantId) -> ActorContext {\n        ActorContext::new(uuid::Uuid::new_v4(), *tenant_id.as_uuid())\n            .with_role(roles::PROJECT_ADMIN)\n            .with_project(*ProjectId::new().as_uuid())\n    }"
int_new = "    fn make_test_actor(tenant_id: TenantId) -> ActorContext {\n        ActorContext::new(UserId::new(), tenant_id)\n            .with_role(roles::PROJECT_ADMIN)\n            .with_project(ProjectId::new())\n    }"
if int_old in int_text:
    int_text = int_text.replace(int_old, int_new, 1)
    print("  OK   domain-integration: make_test_actor fix")
    changes["edits"] += 1
    if not dry_run:
        int_path.write_text(int_text, encoding="utf-8")
        changes["files"] += 1

print(f"\nTOTAL: {changes['files']} files, {changes['edits']} edits (dry_run={dry_run})")
