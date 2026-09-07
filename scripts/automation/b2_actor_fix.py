#!/usr/bin/env python3
"""
B.2 修法: 批量替换 `ActorContext::new(Uuid::nil(), X)` 为 `ActorContext::nil_actor_with_tenant(X)` 或 `ActorContext::new(Uuid::new_v4(), X)`

per Phase B T1.7 B.2 (per OPT-WORKER-03 brief, 2026-09-07)
- 触发原因: `ActorContext::new` 走 INV-ACT-01 守门, user_id 不能为 nil
- 修法: (1) handler 简化用 `nil_actor_with_tenant` (新 helper, 仅 cross-tenant 拒绝用)
        (2) 普通测试用 `Uuid::new_v4()` 替代 `Uuid::nil()`
"""
import re
import sys
from pathlib import Path

WORKTREE = Path("D:/Star/.worktrees/wt-opt-phase-b")
FILES = [
    "crates/star-mcp/src/handlers/identity.rs",
    "crates/star-mcp/src/handlers/permission.rs",
    "crates/star-mcp/src/handlers/project.rs",
    "crates/star-mcp/src/handlers/tenant.rs",
    "crates/star-mcp/src/handlers/workspace.rs",
    "crates/star-mcp/src/handlers/work_item.rs",
    "crates/star-mcp/src/handlers/worktree.rs",
    "crates/star-mcp/src/tools/get_issue.rs",
    "crates/star-mcp/src/tools/get_current_task.rs",
    "crates/star-mcp/src/tools/get_workspace.rs",
    "crates/star-mcp/src/tools/get_worktree.rs",
    "crates/star-mcp/src/transport.rs",
]


def fix_file(rel_path: str) -> tuple[int, int]:
    """Fix one file, return (nil_replaced, new4_replaced)."""
    full = WORKTREE / rel_path
    text = full.read_text(encoding="utf-8")
    original = text

    # Pattern 1: ActorContext::new(uuid::Uuid::nil(), X) -> ActorContext::nil_actor_with_tenant(X)
    # 用于 handler production code (跨 tenant 拒绝用)
    p1 = re.compile(
        r"ActorContext::new\(\s*uuid::Uuid::nil\(\)\s*,\s*([^)]+?)\)(\.with_role\([^)]+\))?"
    )
    nil_count = 0
    for m in p1.finditer(text):
        nil_count += 1
    text = p1.sub(r"ActorContext::nil_actor_with_tenant(\1)\2", text)

    # Pattern 2: ActorContext::new(Uuid::nil(), X) -> ActorContext::nil_actor_with_tenant(X)
    p2 = re.compile(
        r"ActorContext::new\(\s*Uuid::nil\(\)\s*,\s*([^)]+?)\)(\.with_role\([^)]+\))?"
    )
    text = p2.sub(r"ActorContext::nil_actor_with_tenant(\1)\2", text)

    if text != original:
        full.write_text(text, encoding="utf-8")
        return (nil_count, 0)
    return (0, 0)


def main():
    total_nil = 0
    for f in FILES:
        nil_count, new4_count = fix_file(f)
        if nil_count > 0:
            print(f"  {f}: replaced {nil_count} nil-actor patterns")
            total_nil += nil_count
    print(f"\nTotal: {total_nil} nil-actor patterns replaced across {len(FILES)} files")


if __name__ == "__main__":
    main()
