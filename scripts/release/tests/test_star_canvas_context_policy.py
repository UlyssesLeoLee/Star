"""Focused fail-closed tests for the Star Canvas game/legacy context policy."""

# @cypher schema=1 source_sha256=c5565a6f45bb9a62282c2a76d5b82f74ebbedc6802e28bd2152d33b2b0862f84
# MERGE (self:File {path:"scripts/release/tests/test_star_canvas_context_policy.py"})
# MERGE (policy:File {path:"scripts/release/star_canvas_context_policy.py"})
# MERGE (workflow:Config {id:"publish-star-canvas-services-workflow"})
# MERGE (suite:Symbol {id:"scripts/release/tests/test_star_canvas_context_policy.py::PolicyRuleTests",kind:"class"})
# MERGE (wiring:Symbol {id:"scripts/release/tests/test_star_canvas_context_policy.py::WorkflowWiringTests",kind:"class"})
# MERGE (execution:Symbol {id:"scripts/release/tests/test_star_canvas_context_policy.py::WorkflowExecutionTests",kind:"class"})
# MERGE (matches:Symbol {id:"scripts/release/tests/test_star_canvas_context_policy.py::PolicyRuleTests.test_forbidden_matches_are_case_insensitive",kind:"method"})
# MERGE (accepted:Symbol {id:"scripts/release/tests/test_star_canvas_context_policy.py::PolicyRuleTests.test_accepts_star_canvas_identities",kind:"method"})
# MERGE (images:Symbol {id:"scripts/release/tests/test_star_canvas_context_policy.py::PolicyRuleTests.test_image_allowlist_is_never_broadened",kind:"method"})
# MERGE (pkgnames:Symbol {id:"scripts/release/tests/test_star_canvas_context_policy.py::PolicyRuleTests.test_rejects_forbidden_package_names",kind:"method"})
# MERGE (identities:Symbol {id:"scripts/release/tests/test_star_canvas_context_policy.py::PolicyRuleTests.test_rejects_forbidden_package_identities",kind:"method"})
# MERGE (paths:Symbol {id:"scripts/release/tests/test_star_canvas_context_policy.py::PolicyRuleTests.test_rejects_forbidden_context_paths",kind:"method"})
# MERGE (determinism:Symbol {id:"scripts/release/tests/test_star_canvas_context_policy.py::PolicyRuleTests.test_rejections_are_deterministic",kind:"method"})
# MERGE (uses:Symbol {id:"scripts/release/tests/test_star_canvas_context_policy.py::WorkflowWiringTests.test_workflow_uses_the_shared_policy_and_keeps_existing_gates",kind:"method"})
# MERGE (self)-[:DEFINES]->(suite)
# MERGE (self)-[:DEFINES]->(wiring)
# MERGE (suite)-[:DEFINES]->(matches)
# MERGE (suite)-[:DEFINES]->(accepted)
# MERGE (suite)-[:DEFINES]->(images)
# MERGE (suite)-[:DEFINES]->(pkgnames)
# MERGE (suite)-[:DEFINES]->(identities)
# MERGE (suite)-[:DEFINES]->(paths)
# MERGE (suite)-[:DEFINES]->(determinism)
# MERGE (wiring)-[:DEFINES]->(uses)
# MERGE (suite)-[:TESTS]->(policy)
# MERGE (wiring)-[:TESTS]->(workflow)
# MERGE (wiring)-[:TESTS]->(policy)
# MERGE (self)-[:DEFINES]->(execution)
# MERGE (execution)-[:TESTS]->(workflow)
# MERGE (execution)-[:TESTS]->(policy)
# @endcypher

from __future__ import annotations

import importlib.util
import os
import re
import shutil
import subprocess
import sys
import tempfile
import textwrap
import unittest
from pathlib import Path


REPOSITORY = Path(__file__).resolve().parents[3]
MODULE_PATH = REPOSITORY / "scripts" / "release" / "star_canvas_context_policy.py"
WORKFLOW_PATH = REPOSITORY / ".github" / "workflows" / "publish-star-canvas-services.yml"
SPEC = importlib.util.spec_from_file_location("star_canvas_context_policy", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
POLICY = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(POLICY)

ALLOWED_IMAGES = ("canvas-engine", "domain-canvas", "canvas-realtime")
REGISTRY = "registry+https://github.com/rust-lang/crates.io-index"


class PolicyRuleTests(unittest.TestCase):
    def test_forbidden_matches_are_case_insensitive(self) -> None:
        for value, expected in (
            ("star-game", ("game",)),
            ("STAR-GAME", ("game",)),
            ("StarGame", ("game",)),
            ("game-legacy", ("game", "legacy")),
            ("LegacyAuth", ("legacy",)),
            ("LEGACY_COMPAT", ("legacy",)),
            ("crates/legacy-auth/src/lib.rs", ("legacy",)),
            ("crates/GamePad/src/lib.rs", ("game",)),
        ):
            with self.subTest(value=value):
                self.assertEqual(POLICY.forbidden_matches(value), expected)

    def test_unprovable_values_fail_closed(self) -> None:
        self.assertEqual(POLICY.forbidden_matches(None), ())
        self.assertEqual(POLICY.forbidden_matches(""), ("<empty-identity>",))
        self.assertEqual(POLICY.forbidden_matches("   "), ("<empty-identity>",))
        self.assertEqual(POLICY.forbidden_matches(123), ("<unprintable-identity>",))
        with self.assertRaises(POLICY.CanvasContextError):
            POLICY.ensure_clean_package_names([None])
        with self.assertRaises(POLICY.CanvasContextError):
            POLICY.ensure_clean_context_paths([None])

    def test_accepts_star_canvas_identities(self) -> None:
        for name in ALLOWED_IMAGES:
            with self.subTest(name=name):
                self.assertEqual(POLICY.forbidden_matches(name), ())
        accepted_names = [*ALLOWED_IMAGES, "serde", "tokio", "axum", "star-canvas-shared"]
        self.assertEqual(
            POLICY.ensure_clean_package_names(accepted_names), tuple(sorted(accepted_names))
        )
        self.assertEqual(
            POLICY.ensure_clean_package_identities(
                [
                    ("canvas-engine", "1.0.6", None),
                    ("serde", "1.0.219", REGISTRY),
                    ("domain-canvas", "1.0.6", None),
                ]
            ),
            (
                ("canvas-engine", "1.0.6", ""),
                ("domain-canvas", "1.0.6", ""),
                ("serde", "1.0.219", REGISTRY),
            ),
        )
        self.assertEqual(
            POLICY.ensure_clean_context_paths(
                [
                    "Cargo.toml",
                    "Cargo.lock",
                    "Dockerfile",
                    "LICENSE",
                    ".dockerignore",
                    "rust-toolchain.toml",
                    "crates/canvas-engine/Cargo.toml",
                    "crates/canvas-engine/src/lib.rs",
                ]
            ),
            (
                ".dockerignore",
                "Cargo.lock",
                "Cargo.toml",
                "Dockerfile",
                "LICENSE",
                "crates/canvas-engine/Cargo.toml",
                "crates/canvas-engine/src/lib.rs",
                "rust-toolchain.toml",
            ),
        )

    def test_image_allowlist_is_never_broadened(self) -> None:
        for image in ALLOWED_IMAGES:
            with self.subTest(image=image):
                self.assertEqual(POLICY.ensure_allowed_image(image, ALLOWED_IMAGES), image)
        for image in ("star-game", "game-legacy", "canvas-engine ", "CANVAS-ENGINE", "", None, 7):
            with self.subTest(image=image):
                with self.assertRaises(POLICY.CanvasContextError):
                    POLICY.ensure_allowed_image(image, ALLOWED_IMAGES)
        with self.assertRaises(POLICY.CanvasContextError):
            POLICY.ensure_allowed_image("star-game", [*ALLOWED_IMAGES, "star-game"])

    def test_rejects_forbidden_package_names(self) -> None:
        for name in ("star-game", "legacy-auth", "LegacyAuth", "GAME_UTILS", "game-legacy-core"):
            with self.subTest(name=name):
                with self.assertRaises(POLICY.CanvasContextError):
                    POLICY.ensure_clean_package_names(["serde", name])

    def test_rejects_forbidden_package_identities(self) -> None:
        cases = (
            [("star-game", "0.1.0", None)],
            [("legacy-auth", "1.0.0", REGISTRY)],
            [("GAME_UTILS", "1.0.0", REGISTRY)],
            [("serde", "1.0.0-legacy", REGISTRY)],
            [("serde", "1.0.0", "git+https://example.invalid/legacy-fork#abc123")],
        )
        for identities in cases:
            with self.subTest(identities=identities):
                with self.assertRaises(POLICY.CanvasContextError):
                    POLICY.ensure_clean_package_identities(identities)

    def test_rejects_forbidden_context_paths(self) -> None:
        cases = (
            ["crates/canvas-engine/src/legacy_compat.rs"],
            ["crates/canvas-engine/src/LegacyCompat.rs"],
            ["crates/canvas-engine/src/game_pad.rs"],
            ["crates/star-game/Cargo.toml"],
            ["crates/canvas-engine/src/../star-game/src/lib.rs"],
            [".git/config"],
            ["target/release/canvas-engine"],
        )
        for paths in cases:
            with self.subTest(paths=paths):
                with self.assertRaises(POLICY.CanvasContextError):
                    POLICY.ensure_clean_context_paths(["Cargo.toml", *paths])

    def test_rejections_are_deterministic(self) -> None:
        dirty = ["crates/legacy-b/src/lib.rs", "crates/a-game/src/lib.rs", ".git/HEAD"]
        messages = []
        for _ in range(3):
            with self.assertRaises(POLICY.CanvasContextError) as caught:
                POLICY.ensure_clean_context_paths(list(reversed(dirty)))
            messages.append(str(caught.exception))
        self.assertEqual(len(set(messages)), 1)
        self.assertIn("a-game", messages[0])
        self.assertIn("legacy-b", messages[0])
        self.assertIn(".git/HEAD", messages[0])


class WorkflowWiringTests(unittest.TestCase):
    def setUp(self) -> None:
        self.workflow = WORKFLOW_PATH.read_text(encoding="utf-8")

    def test_workflow_uses_the_shared_policy_and_keeps_existing_gates(self) -> None:
        for required in (
            # shared validation is imported from the checkout and fails closed
            "import star_canvas_context_policy as canvas_policy",
            "Star Canvas context policy helper is unavailable",
            "canvas_policy.CanvasContextError",
            # every gate calls the shared policy before the image build
            "enforce(lambda: canvas_policy.ensure_allowed_image(image, allowed))",
            "enforce(lambda: canvas_policy.ensure_clean_package_identities(included))",
            "enforce(lambda: canvas_policy.ensure_clean_package_names(filtered_names))",
            "enforce(lambda: canvas_policy.ensure_clean_context_paths(files))",
            # pre-existing allowlist and lock/closure gates are untouched
            'allowed = {"canvas-engine", "domain-canvas", "canvas-realtime"}',
            "Refusing unapproved Canvas image",
            'fromJSON(\'["canvas-engine","domain-canvas","canvas-realtime"]\')',
            "Local path dependency is not allowed in isolated image context",
            "Workspace dependency resolves to a local product crate",
            "Canvas package closure reaches non-Canvas workspace members",
            "Canvas package closure unexpectedly includes star-game.",
            "Filtered Cargo.lock failed the isolated package closure check.",
            'if "star-game" in filtered_names',
            "Unexpected files in image context",
            "Image context contains a crate outside the allowlisted service.",
            're.fullmatch(r"(?:[0-9a-f]{40}|[0-9a-f]{64})", release_source_sha)',
            "Filtered Cargo.lock root does not match the resolved release version.",
        ):
            with self.subTest(required=required):
                self.assertIn(required, self.workflow)

    def test_policy_gate_runs_before_the_image_build(self) -> None:
        assembly = self.workflow.index("Assemble and verify a minimal per-service Cargo context")
        login = self.workflow.index("Log in to GHCR")
        build = self.workflow.index("Build and push")
        self.assertLess(assembly, login)
        self.assertLess(login, build)
        for call in (
            "ensure_allowed_image",
            "ensure_clean_package_identities",
            "ensure_clean_package_names",
            "ensure_clean_context_paths",
        ):
            with self.subTest(call=call):
                position = self.workflow.index(call)
                self.assertGreater(position, assembly)
                self.assertLess(position, login)

    def test_allowlist_is_not_broadened_in_the_workflow(self) -> None:
        allowlist_lines = [
            line.strip()
            for line in self.workflow.splitlines()
            if line.strip().startswith("allowed = {")
        ]
        self.assertEqual(
            allowlist_lines,
            ['allowed = {"canvas-engine", "domain-canvas", "canvas-realtime"}'] * 2,
        )
        matrix_lines = [
            line.strip() for line in self.workflow.splitlines() if "fromJSON(" in line
        ]
        self.assertEqual(
            matrix_lines,
            ['image: ${{ fromJSON(\'["canvas-engine","domain-canvas","canvas-realtime"]\') }}'],
        )


_FIXTURE_ROOT_CARGO = """\
[workspace]
resolver = "2"
members = ["crates/canvas-engine"]

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT"
rust-version = "1.99.0"

[workspace.dependencies]
serde = { version = "1.0.219", features = ["derive"] }

[workspace.lints.rust]
unsafe_code = "forbid"
"""

_FIXTURE_CRATE_CARGO = """\
[package]
name = "canvas-engine"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
serde = { workspace = true }
"""

_FIXTURE_CHECKSUM = "0" * 64
_HERE_STRING = re.compile(r"@'\n(.*?)\n[ ]*'*@", re.S)


def _registry_package(name: str, version: str = "1.0.0") -> str:
    return (
        f'[[package]]\nname = "{name}"\nversion = "{version}"\n'
        f'source = "{REGISTRY}"\nchecksum = "{_FIXTURE_CHECKSUM}"\n'
    )


def _local_package(name: str, version: str = "0.1.0") -> str:
    return f'[[package]]\nname = "{name}"\nversion = "{version}"\n'


def _fixture_lock(root_dependencies: tuple[str, ...], extra_packages: tuple[str, ...]) -> str:
    dependencies = "".join(f' "{name}",\n' for name in root_dependencies)
    blocks = [
        f'[[package]]\nname = "canvas-engine"\nversion = "0.1.0"\n'
        f"dependencies = [\n{dependencies}]\n",
        f'[[package]]\nname = "serde"\nversion = "1.0.219"\n'
        f'source = "{REGISTRY}"\nchecksum = "{_FIXTURE_CHECKSUM}"\n',
        *extra_packages,
    ]
    return (
        "# This file is automatically @generated by Cargo.\n"
        "# It is not intended for manual editing.\n"
        "version = 3\n\n" + "\n".join(blocks)
    )


class WorkflowExecutionTests(unittest.TestCase):
    """Run the workflow's real context gate so mutations prove the rule bites."""

    @classmethod
    def setUpClass(cls) -> None:
        workflow = WORKFLOW_PATH.read_text(encoding="utf-8")
        blocks = _HERE_STRING.findall(workflow)
        assert len(blocks) == 1, f"expected one PowerShell here-string, found {len(blocks)}"
        cls.script = textwrap.dedent(blocks[0])
        assert "star_canvas_context_policy" in cls.script

    def build_fixture(
        self,
        root: Path,
        *,
        root_dependencies: tuple[str, ...] = ("serde",),
        extra_packages: tuple[str, ...] = (),
        src_files: tuple[str, ...] = ("lib.rs",),
        include_policy: bool = True,
    ) -> None:
        (root / "Cargo.toml").write_text(_FIXTURE_ROOT_CARGO, encoding="utf-8")
        (root / "Cargo.lock").write_text(
            _fixture_lock(root_dependencies, extra_packages), encoding="utf-8"
        )
        (root / "rust-toolchain.toml").write_text('[toolchain]\nchannel = "1.99.0"\n', encoding="utf-8")
        (root / "LICENSE").write_text("MIT License\n", encoding="utf-8")
        (root / ".dockerignore").write_text("target\n", encoding="utf-8")
        crate = root / "crates" / "canvas-engine"
        (crate / "src").mkdir(parents=True)
        (crate / "Cargo.toml").write_text(_FIXTURE_CRATE_CARGO, encoding="utf-8")
        (crate / "Dockerfile").write_text("FROM scratch\n", encoding="utf-8")
        for name in src_files:
            (crate / "src" / name).write_text("// fixture source\n", encoding="utf-8")
        if include_policy:
            policy_dir = root / "scripts" / "release"
            policy_dir.mkdir(parents=True, exist_ok=True)
            shutil.copy2(MODULE_PATH, policy_dir / MODULE_PATH.name)

    def run_gate(self, workspace: Path, image: str = "canvas-engine") -> subprocess.CompletedProcess[str]:
        runner_temp = workspace.parent / "runner-temp"
        runner_temp.mkdir(parents=True, exist_ok=True)
        output_file = runner_temp / "github_output.txt"
        output_file.write_text("", encoding="utf-8")
        env = dict(os.environ)
        env.update(
            CANVAS_IMAGE=image,
            STAR_RELEASE_TAG="v1.0.6",
            STAR_RELEASE_VERSION="1.0.6",
            STAR_RELEASE_SOURCE_SHA="a" * 40,
            GITHUB_WORKSPACE=str(workspace),
            RUNNER_TEMP=str(runner_temp),
            GITHUB_OUTPUT=str(output_file),
        )
        result = subprocess.run(
            [sys.executable, "-"],
            input=self.script,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            env=env,
            cwd=str(workspace),
        )
        result.github_output = output_file.read_text(encoding="utf-8")  # type: ignore[attr-defined]
        return result

    def assert_rejected(self, result: subprocess.CompletedProcess[str], *fragments: str) -> None:
        self.assertNotEqual(result.returncode, 0, msg=result.stdout + result.stderr)
        self.assertEqual(result.github_output, "")  # type: ignore[attr-defined]
        for fragment in fragments:
            with self.subTest(fragment=fragment):
                self.assertIn(fragment, result.stderr)

    def test_control_clean_canvas_fixture_is_accepted(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            workspace = Path(temporary) / "workspace"
            workspace.mkdir()
            self.build_fixture(workspace)
            result = self.run_gate(workspace)
        self.assertEqual(result.returncode, 0, msg=result.stdout + result.stderr)
        self.assertIn("Verified canvas-engine: 2 locked packages; 8 files", result.stdout)
        self.assertIn("no game/legacy or foreign workspace sources", result.stdout)
        self.assertTrue(result.github_output.startswith("directory="))  # type: ignore[attr-defined]

    def test_rejects_legacy_registry_package_the_old_rule_missed(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            workspace = Path(temporary) / "workspace"
            workspace.mkdir()
            self.build_fixture(
                workspace,
                root_dependencies=("serde", "legacy-auth-shim"),
                extra_packages=(_registry_package("legacy-auth-shim"),),
            )
            result = self.run_gate(workspace)
        self.assert_rejected(result, "legacy-auth-shim", "legacy")

    def test_rejects_forbidden_identity_regardless_of_case(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            workspace = Path(temporary) / "workspace"
            workspace.mkdir()
            self.build_fixture(
                workspace,
                root_dependencies=("serde", "GAME_UTILS"),
                extra_packages=(_registry_package("GAME_UTILS"),),
            )
            result = self.run_gate(workspace)
        self.assert_rejected(result, "GAME_UTILS", "game")

    def test_rejects_star_game_local_package(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            workspace = Path(temporary) / "workspace"
            workspace.mkdir()
            self.build_fixture(
                workspace,
                root_dependencies=("serde", "star-game"),
                extra_packages=(_local_package("star-game"),),
            )
            result = self.run_gate(workspace)
        self.assert_rejected(result, "star-game", "game")

    def test_rejects_legacy_source_file_the_old_rule_missed(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            workspace = Path(temporary) / "workspace"
            workspace.mkdir()
            self.build_fixture(workspace, src_files=("lib.rs", "legacy_compat.rs"))
            result = self.run_gate(workspace)
        self.assert_rejected(result, "legacy_compat.rs", "legacy")

    def test_rejects_uppercase_legacy_source_file(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            workspace = Path(temporary) / "workspace"
            workspace.mkdir()
            self.build_fixture(workspace, src_files=("lib.rs", "LegacyCompat.rs"))
            result = self.run_gate(workspace)
        self.assert_rejected(
            result,
            "Star Canvas context policy rejected this build",
            "LegacyCompat.rs",
            "legacy",
        )

    def test_retained_game_gate_still_rejects_uppercase_game_file(self) -> None:
        """The pre-existing game check is retained and still fires first."""
        with tempfile.TemporaryDirectory() as temporary:
            workspace = Path(temporary) / "workspace"
            workspace.mkdir()
            self.build_fixture(workspace, src_files=("lib.rs", "GamePad.rs"))
            result = self.run_gate(workspace)
        self.assert_rejected(result, "Unexpected files in image context", "GamePad.rs")

    def test_rejects_unallowlisted_image(self) -> None:
        for image in ("star-game", "legacy-canvas", "canvas-engine-legacy"):
            with self.subTest(image=image), tempfile.TemporaryDirectory() as temporary:
                workspace = Path(temporary) / "workspace"
                workspace.mkdir()
                self.build_fixture(workspace)
                result = self.run_gate(workspace, image=image)
                self.assert_rejected(result, "Refusing unapproved Canvas image")

    def test_fails_closed_when_the_shared_policy_is_unavailable(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            workspace = Path(temporary) / "workspace"
            workspace.mkdir()
            self.build_fixture(workspace, include_policy=False)
            result = self.run_gate(workspace)
        self.assert_rejected(result, "Star Canvas context policy helper is unavailable")

    def test_mutated_payloads_are_invisible_to_the_previous_rule(self) -> None:
        """Prove the mutations above are new coverage, not already-caught cases."""
        legacy_path = "crates/canvas-engine/src/legacy_compat.rs"
        self.assertNotIn("game", legacy_path.lower())
        self.assertNotIn("legacy", "crates/canvas-engine/src/GamePad.rs".lower())
        for name in ("legacy-auth-shim", "GAME_UTILS"):
            with self.subTest(name=name):
                self.assertNotEqual(name, "star-game")
                self.assertEqual(POLICY.forbidden_matches(name), POLICY.forbidden_matches(name.lower()))
                self.assertNotEqual(POLICY.forbidden_matches(name), ())


if __name__ == "__main__":
    unittest.main()
