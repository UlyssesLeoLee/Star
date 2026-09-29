#!/usr/bin/env python3
"""Tests for group_guard.py — 守门 #32/#33/#34 unit tests."""
import os
import sys
from pathlib import Path
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import group_guard


def _cargo_metadata(edges):
    root = Path(group_guard.__file__).resolve().parents[2]
    packages = [
        {"id": "canvas-id", "name": "canvas-engine", "manifest_path": str(root / "crates/canvas-engine/Cargo.toml")},
        {"id": "domain-id", "name": "domain-work-item", "manifest_path": str(root / "crates/domain-work-item/Cargo.toml")},
        {"id": "domain-canvas-id", "name": "domain-canvas", "manifest_path": str(root / "crates/domain-canvas/Cargo.toml")},
        {"id": "api-id", "name": "api", "manifest_path": str(root / "crates/api/Cargo.toml")},
    ]
    deps = {package["id"]: [] for package in packages}
    for source, target in edges:
        deps[source].append({"pkg": target})
    return {
        "packages": packages,
        "workspace_members": list(deps),
        "resolve": {"nodes": [{"id": package_id, "deps": package_deps} for package_id, package_deps in deps.items()]},
    }


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


def test_check_cross_ref_allows_zero_cross_group_edges():
    passed, message = group_guard.check_cross_ref("canvas", metadata=_cargo_metadata([]))
    assert passed
    assert "0 cross-group" in message


def test_check_cross_ref_rejects_canvas_to_domain_edge():
    passed, message = group_guard.check_cross_ref(
        "canvas", metadata=_cargo_metadata([("canvas-id", "domain-id")])
    )
    assert not passed
    assert "canvas:canvas-engine -> domain:domain-work-item" in message


def test_check_cross_ref_rejects_domain_to_canvas_edge_and_assigns_domain_canvas_to_canvas():
    passed, message = group_guard.check_cross_ref(
        "domain", metadata=_cargo_metadata([("domain-id", "domain-canvas-id")])
    )
    assert not passed
    assert "domain:domain-work-item -> canvas:domain-canvas" in message


def test_check_cross_ref_ignores_shared_unclassified_package_edges():
    metadata = _cargo_metadata([("api-id", "canvas-id"), ("canvas-id", "api-id")])
    passed, message = group_guard.check_cross_ref("canvas", metadata=metadata)
    assert passed
    assert "0 cross-group" in message


def test_check_cross_ref_fails_closed_when_resolution_is_missing():
    passed, message = group_guard.check_cross_ref("canvas", metadata={"packages": [], "workspace_members": []})
    assert not passed
    assert "resolved dependency graph" in message


def test_check_cross_ref_fails_closed_when_workspace_node_is_missing():
    metadata = _cargo_metadata([])
    metadata["resolve"]["nodes"] = metadata["resolve"]["nodes"][:-1]
    passed, message = group_guard.check_cross_ref("canvas", metadata=metadata)
    assert not passed
    assert "missing a workspace member node" in message


def test_check_cross_ref_fails_closed_when_dependency_target_is_missing():
    metadata = _cargo_metadata([])
    metadata["resolve"]["nodes"][0]["deps"] = [{}]
    passed, message = group_guard.check_cross_ref("canvas", metadata=metadata)
    assert not passed
    assert "malformed dependency edge" in message


def test_check_cross_ref_is_not_applicable_to_frontend_group():
    passed, message = group_guard.check_cross_ref("frontend")
    assert passed
    assert "N/A" in message


def test_cmd_check_fails_when_group_id_does_not_match(monkeypatch):
    from types import SimpleNamespace

    monkeypatch.setattr(group_guard, "get_current_branch", lambda: "codex/worktree-group-canvas-dev")
    monkeypatch.setattr(group_guard, "get_changed_files", lambda: [])
    monkeypatch.setattr(group_guard, "check_cross_ref", lambda _group: (True, "0 cross-group edges"))
    monkeypatch.setenv("NEXT_PUBLIC_GROUP_ID", "domain")
    assert group_guard.cmd_check(SimpleNamespace(group="canvas")) == 1

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
    test_check_cross_ref_allows_zero_cross_group_edges()
    test_check_cross_ref_rejects_canvas_to_domain_edge()
    test_check_cross_ref_rejects_domain_to_canvas_edge_and_assigns_domain_canvas_to_canvas()
    test_check_cross_ref_ignores_shared_unclassified_package_edges()
    test_check_cross_ref_fails_closed_when_resolution_is_missing()
    test_check_cross_ref_fails_closed_when_workspace_node_is_missing()
    test_check_cross_ref_fails_closed_when_dependency_target_is_missing()
    test_check_cross_ref_is_not_applicable_to_frontend_group()
    print("✅ All 19 direct group_guard checks PASS")
