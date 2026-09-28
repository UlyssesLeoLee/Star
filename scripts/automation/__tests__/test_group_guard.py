#!/usr/bin/env python3
"""Tests for group_guard.py — 守门 #32/#33/#34 unit tests."""
import os
import sys
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import group_guard


def test_check_group_boundary_canvas():
    """canvas-group 改 canvas-* OK, 改 domain-* FAIL."""
    files_canvas = [
        "crates/canvas-engine/src/lib.rs",
        "deploy/canvas-game-k3s.yaml",
    ]
    passed, violations = group_guard.check_group_boundary("canvas", files_canvas)
    assert passed, f"canvas-group canvas files should pass: {violations}"
    assert len(violations) == 0


def test_check_group_boundary_canvas_violation():
    """canvas-group 改 frontend FAIL."""
    files_violation = [
        "crates/canvas-engine/src/lib.rs",
        "frontend/src/components/AppHeader.tsx",  # not canvas
    ]
    passed, violations = group_guard.check_group_boundary("canvas", files_violation)
    assert not passed
    assert len(violations) == 1
    assert "frontend" in violations[0]


def test_check_group_boundary_frontend():
    """frontend-group 改 frontend OK, 改 crates/ FAIL."""
    files_fe = [
        "frontend/src/components/AppHeader.tsx",
        "frontend/package.json",
    ]
    passed, violations = group_guard.check_group_boundary("frontend", files_fe)
    assert passed, f"frontend-group fe files should pass: {violations}"


def test_check_group_boundary_frontend_violation():
    files_fe_violation = [
        "frontend/src/App.tsx",
        "crates/canvas-engine/src/lib.rs",  # not frontend
    ]
    passed, violations = group_guard.check_group_boundary("frontend", files_fe_violation)
    assert not passed
    assert any("crates/canvas-engine" in v for v in violations)


def test_check_group_boundary_core():
    """core-group 无约束, 任何文件都过."""
    files = [
        "crates/canvas-engine/src/lib.rs",
        "frontend/src/App.tsx",
        "AGENTS.md",
    ]
    passed, violations = group_guard.check_group_boundary("core", files)
    assert passed
    assert len(violations) == 0


def test_detect_group_canvas():
    branch = "codex/worktree-group-canvas-engine-gameplay"
    assert group_guard.detect_group_from_branch(branch) == "canvas"


def test_detect_group_domain():
    branch = "codex/worktree-group-domain-worktree-v2"
    assert group_guard.detect_group_from_branch(branch) == "domain"


def test_detect_group_frontend():
    branch = "codex/worktree-group-frontend-app-header"
    assert group_guard.detect_group_from_branch(branch) == "frontend"


def test_detect_group_core():
    branch = "fix/ulys-160-envoy-pod-ip-direct"
    assert group_guard.detect_group_from_branch(branch) == "core"



def test_check_group_boundary_activation_allowed():
    """Activation files (docs/scripts/automation) allowed in any group."""
    files = [
        "docs/worktree-group-guard.md",
        "scripts/automation/group_guard.py",
    ]
    passed, violations = group_guard.check_group_boundary("canvas", files)
    assert passed, f"Activation files should be allowed in canvas: {violations}"


def test_check_group_boundary_frontend_with_canvas_violation():
    """frontend-group PR 改 canvas-engine still fails (no cross-group)."""
    files = ["frontend/src/App.tsx", "crates/canvas-engine/src/lib.rs"]
    passed, violations = group_guard.check_group_boundary("frontend", files)
    assert not passed
    assert any("canvas-engine" in v for v in violations)

if __name__ == "__main__":
    test_check_group_boundary_canvas()
    test_check_group_boundary_canvas_violation()
    test_check_group_boundary_frontend()
    test_check_group_boundary_frontend_violation()
    test_check_group_boundary_core()
    test_detect_group_canvas()
    test_detect_group_domain()
    test_detect_group_frontend()
    test_detect_group_core()
    test_check_group_boundary_activation_allowed()
    test_check_group_boundary_frontend_with_canvas_violation()
    print("✅ All 11 group_guard tests PASS")
