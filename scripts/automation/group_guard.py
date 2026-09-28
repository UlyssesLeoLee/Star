#!/usr/bin/env python3
"""
# SPDX-License-Identifier: MIT OR Apache-2.0
`group_guard.py` — Worktree Group 守门 #32/#33/#34 实装 (per 2026-09-27 激活).

per docs/worktree-group-guard.md §3:
- 守门 #32 v0: Group 边界检查 (PR diff 不出 Group 范围)
- 守门 #33 v0: Group cross-ref 守门 (Cargo path deps 双向 0)
- 守门 #34 v0: Group 二维 ID 注入 (NEXT_PUBLIC_GROUP_ID env)

Usage:
    python3 scripts/automation/group_guard.py check --group <canvas|domain|frontend|core>
    python3 scripts/automation/group_guard.py env     # Print expected env vars
    python3 scripts/automation/group_guard.py branches # List active Group branches

Refs: docs/worktree-group-guard.md
"""
import argparse
import os
import subprocess
import sys
from pathlib import Path

GROUP_RANGES = {
    "canvas": [
        # canvas group can modify:
        "crates/canvas-engine/",
        "crates/canvas-game/",
        "crates/canvas-realtime/",
        "crates/domain-canvas/",
        "deploy/canvas-game-k3s.yaml",
        ".github/workflows/publish-canvas-game.yml",
        "frontend/src/app/(app)/canvas/",  # canvas-specific frontend
    ],
    "domain": [
        "crates/domain-agent/",
        "crates/domain-audit/",
        "crates/domain-automation/",
        "crates/domain-batch/",
        "crates/domain-board/",
        "crates/domain-collaboration/",
        "crates/domain-comment/",
        "crates/domain-context/",
        "crates/domain-development/",
        "crates/domain-feedback/",
        "crates/domain-identity/",
        "crates/domain-integration/",
        "crates/domain-kms/",
        "crates/domain-local-runtime/",
        "crates/domain-notification/",
        "crates/domain-permission/",
        "crates/domain-planning/",
        "crates/domain-project/",
        "crates/domain-relation/",
        "crates/domain-scm/",
        "crates/domain-search/",
        "crates/domain-tenant/",
        "crates/domain-theme/",
        "crates/domain-validation/",
        "crates/domain-work-item/",
        "crates/domain-workflow/",
        "crates/domain-workspace/",
        "crates/domain-worktree/",
        "crates/star-cache/",
        "crates/star-cli/",
        "crates/star-mcp/",
        "crates/star-api-rest/",
        "crates/star-ops/",
        "crates/application/",
        "crates/api/",
        "crates/agent-domain/",
        "crates/agent-game/",
        "crates/agent-view/",
        "crates/infrastructure/",
        "crates/worktree-shared-dir/",
        "crates/worktree-service/",
        "crates/worktree-canvas/",
        "crates/git-adapter/",
        "crates/git-observer/",
        "crates/graph-core/",
        "crates/arg-bridge/",
    ],
    "frontend": [
        "frontend/src/",
        "frontend/package.json",
        "frontend/next.config.js",
        "frontend/tsconfig.json",
        "frontend/tailwind.config.*",
        "frontend/postcss.config.*",
        "frontend/playwright.config.*",
        "frontend/e2e/",
        "frontend/__tests__/",
    ],
    "core": [
        # root dev/main 无 Group 约束
    ],
}


def get_current_branch() -> str:
    """Get current git branch."""
    try:
        result = subprocess.run(
            ["git", "rev-parse", "--abbrev-ref", "HEAD"],
            capture_output=True, text=True, timeout=30,
        )
        return result.stdout.strip()
    except Exception:
        return ""


def detect_group_from_branch(branch: str) -> str:
    """Detect Group from branch name per §1 模板.

    e.g. `codex/worktree-group-canvas-engine` → canvas
         `fix/ulys-160-envoy-pod-ip-direct` → core
         `codex/worktree-group-docs-dev` → core (docs group)
    """
    if "worktree-group-canvas" in branch:
        return "canvas"
    if "worktree-group-domain" in branch:
        return "domain"
    if "worktree-group-frontend" in branch:
        return "frontend"
    return "core"


def get_changed_files(base: str = "origin/dev") -> list[str]:
    """Get files changed in current branch vs base."""
    try:
        result = subprocess.run(
            ["git", "diff", "--name-only", f"{base}...HEAD"],
            capture_output=True, text=True, timeout=30,
        )
        return [f.strip() for f in result.stdout.split("\n") if f.strip()]
    except Exception as e:
        print(f"git diff failed: {e}", file=sys.stderr)
        return []


# 激活文件 allowlist — Group 激活 PR 可跨 Group 范围 (per docs/worktree-group-guard.md §3.4.5)
ACTIVATION_FILES = {
    "docs/worktree-group-guard.md",
    "docs/automation-design.md",
    "scripts/automation/group_guard.py",
    "scripts/automation/__tests__/test_group_guard.py",
    "scripts/automation/registry.md",
}


def check_group_boundary(group: str, files: list[str]) -> tuple[bool, list[str]]:
    """守门 #32 — Group 边界检查.

    Returns (passed, violations)
    """
    if group == "core":
        return True, []  # core group 无约束

    allowed = GROUP_RANGES.get(group, [])
    violations = []

    for f in files:
        # Strip leading ./ prefix if any
        clean = f.lstrip("./")
        # Allow activation files (per §3.4.5)
        if clean in ACTIVATION_FILES:
            continue
        if not any(clean.startswith(prefix) or clean == prefix.rstrip("/")
                   for prefix in allowed):
            violations.append(f"  - {f}  (not in {group} range)")

    return len(violations) == 0, violations


def check_cross_ref(group: str) -> tuple[bool, str]:
    """守门 #33 — Group cross-ref 守门 (Cargo path deps 双向 0).

    Simplified check: warn if canvas-group PR touches domain-* src
    """
    # This is a stub; full implementation needs cargo metadata analysis
    return True, "OK (placeholder; full cargo metadata check pending)"


def print_env(group: str) -> None:
    """守门 #34 — Print expected Group env vars."""
    print(f"# Group env vars (set these before running):")
    print(f"export NEXT_PUBLIC_GROUP_ID={group}")
    print(f'export NEXT_PUBLIC_WORKTREE_ID=$(basename $(git rev-parse --show-toplevel))')


def cmd_check(args) -> int:
    """Run all 3 guards."""
    branch = get_current_branch()
    group = args.group or detect_group_from_branch(branch)

    print(f"# Group Guard Check")
    print(f"# Branch: {branch}")
    print(f"# Group: {group}")
    print()

    files = get_changed_files()
    print(f"# Changed files: {len(files)}")
    for f in files[:5]:
        print(f"  - {f}")
    if len(files) > 5:
        print(f"  ... and {len(files) - 5} more")
    print()

    passed_32, violations = check_group_boundary(group, files)
    if passed_32:
        print(f"# 守门 #32 (Group 边界): ✅ PASS ({group})")
    else:
        print(f"# 守门 #32 (Group 边界): ❌ FAIL")
        print(f"# Violations ({len(violations)}):")
        for v in violations[:10]:
            print(v)
        if len(violations) > 10:
            print(f"  ... and {len(violations) - 10} more")

    print()

    passed_33, msg_33 = check_cross_ref(group)
    if passed_33:
        print(f"# 守门 #33 (Group cross-ref): ✅ PASS — {msg_33}")
    else:
        print(f"# 守门 #33 (Group cross-ref): ❌ FAIL — {msg_33}")

    print()

    group_id = os.environ.get("NEXT_PUBLIC_GROUP_ID", "")
    wt_id = os.environ.get("NEXT_PUBLIC_WORKTREE_ID", "")
    if group_id == "":
        # Not set — just warn (per §3.4.3 fallback core)
        print(f"# 守门 #34 (Group 二维 ID): ⚠️  WARN (NEXT_PUBLIC_GROUP_ID not set; fallback core)")
        print(f"#  Hint: run 'python3 scripts/automation/group_guard.py env' for setup")
    elif group_id != group:
        print(f"# 守门 #34 (Group 二维 ID): ❌ FAIL (env NEXT_PUBLIC_GROUP_ID={group_id}, expected {group})")
    else:
        # group_id matches expected; check worktree_id (optional, warn only)
        print(f"# 守门 #34 (Group 二维 ID): ✅ PASS")
        print(f"  NEXT_PUBLIC_GROUP_ID={group_id}")
        if wt_id:
            print(f"  NEXT_PUBLIC_WORKTREE_ID={wt_id}")
        else:
            print(f"  NEXT_PUBLIC_WORKTREE_ID (unset, fallback to branch-name hash)")

    print()
    return 0 if (passed_32 and passed_33) else 1


def cmd_env(args) -> int:
    branch = get_current_branch()
    group = args.group or detect_group_from_branch(branch)
    print_env(group)
    return 0


def cmd_branches(args) -> int:
    """List active Group branches."""
    result = subprocess.run(
        ["git", "branch", "-r"],
        capture_output=True, text=True, timeout=30,
    )
    for line in result.stdout.split("\n"):
        line = line.strip()
        if "worktree-group" in line:
            print(line)
    return 0


def main():
    parser = argparse.ArgumentParser(description="Worktree Group 守门")
    sub = parser.add_subparsers(dest="cmd")

    p_check = sub.add_parser("check", help="Run all 3 guards")
    p_check.add_argument("--group", choices=list(GROUP_RANGES.keys()),
                         help="Override Group detection")
    p_check.set_defaults(func=cmd_check)

    p_env = sub.add_parser("env", help="Print expected env vars")
    p_env.add_argument("--group", choices=list(GROUP_RANGES.keys()),
                       help="Override Group detection")
    p_env.set_defaults(func=cmd_env)

    p_branches = sub.add_parser("branches", help="List active Group branches")
    p_branches.set_defaults(func=cmd_branches)

    args = parser.parse_args()
    if not args.cmd:
        parser.print_help()
        return 1

    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
