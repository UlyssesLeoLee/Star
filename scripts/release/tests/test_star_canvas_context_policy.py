"""Focused fail-closed tests for the Star Canvas game/legacy context policy."""

# @cypher schema=1 source_sha256=4435f818ae636e316207576da5317c04c8567382aa780158d6acac7895f5a2a3
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
# MERGE (linksuite:Symbol {id:"scripts/release/tests/test_star_canvas_context_policy.py::LinkPolicyTests",kind:"class"})
# MERGE (linkcontrol:Symbol {id:"scripts/release/tests/test_star_canvas_context_policy.py::WorkflowExecutionTests.test_benign_named_link_would_smuggle_content_without_the_gate",kind:"method"})
# MERGE (linksmuggle:Symbol {id:"scripts/release/tests/test_star_canvas_context_policy.py::WorkflowExecutionTests.test_rejects_benign_named_link_that_would_smuggle_game_content",kind:"method"})
# MERGE (linkparentcontrol:Symbol {id:"scripts/release/tests/test_star_canvas_context_policy.py::WorkflowExecutionTests.test_internal_parent_link_would_smuggle_content_without_the_gate",kind:"method"})
# MERGE (linkparent:Symbol {id:"scripts/release/tests/test_star_canvas_context_policy.py::WorkflowExecutionTests.test_rejects_internal_parent_link_before_copying_payload",kind:"method"})
# MERGE (linkancestor:Symbol {id:"scripts/release/tests/test_star_canvas_context_policy.py::LinkPolicyTests.test_linked_ancestor_inside_the_checkout_fails_closed",kind:"method"})
# MERGE (linkcapwin:Symbol {id:"scripts/release/tests/test_star_canvas_context_policy.py::LinkPolicyTests.test_windows_without_junction_api_fails_closed",kind:"method"})
# MERGE (linkcapposix:Symbol {id:"scripts/release/tests/test_star_canvas_context_policy.py::LinkPolicyTests.test_non_windows_without_junction_api_still_accepts_ordinary_paths",kind:"method"})
# MERGE (linkcapm11:Symbol {id:"scripts/release/tests/test_star_canvas_context_policy.py::WorkflowExecutionTests.test_gate_fails_closed_without_junction_api_on_windows",kind:"method"})
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
# MERGE (self)-[:DEFINES]->(linksuite)
# MERGE (linksuite)-[:TESTS]->(policy)
# MERGE (linksuite)-[:DEFINES]->(linkancestor)
# MERGE (execution)-[:DEFINES]->(linkcontrol)
# MERGE (execution)-[:DEFINES]->(linksmuggle)
# MERGE (execution)-[:DEFINES]->(linkparentcontrol)
# MERGE (execution)-[:DEFINES]->(linkparent)
# MERGE (execution)-[:DEFINES]->(linkcapm11)
# MERGE (linksuite)-[:DEFINES]->(linkcapwin)
# MERGE (linksuite)-[:DEFINES]->(linkcapposix)
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
            "enforce(lambda: canvas_policy.ensure_link_free_sources(",
            "allowed_root=repo,",
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
            "ensure_link_free_sources",
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
rust-version = "1.99.1"

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


SMUGGLED_MARKER = "SMUGGLED-GAME-LEGACY-PAYLOAD"


def _create_directory_link(target: Path, link: Path) -> str | None:
    """Create a directory link and report its kind, or None if the platform refuses.

    Windows directory junctions need no privilege and are the harder case, because
    ``Path.is_symlink()`` reports them as ordinary directories while
    ``shutil.copytree`` still follows them. Real symlinks are used elsewhere; on
    Windows they require Developer Mode or elevation.
    """
    if sys.platform == "win32":
        try:
            import _winapi

            _winapi.CreateJunction(str(target), str(link))
            return "junction"
        except (ImportError, OSError):
            pass
    try:
        os.symlink(str(target), str(link), target_is_directory=True)
        return "symlink"
    except (OSError, NotImplementedError, TypeError):
        return None


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
        smuggle_link: str | None = None,
        crates_link_target: str | None = None,
        marker_files: tuple[str, ...] = (),
    ) -> dict[str, str]:
        """Build a workspace and report which links were actually created."""
        links: dict[str, str] = {}
        (root / "Cargo.toml").write_text(_FIXTURE_ROOT_CARGO, encoding="utf-8")
        (root / "Cargo.lock").write_text(
            _fixture_lock(root_dependencies, extra_packages), encoding="utf-8"
        )
        (root / "rust-toolchain.toml").write_text('[toolchain]\nchannel = "1.99.1"\n', encoding="utf-8")
        (root / "LICENSE").write_text("MIT License\n", encoding="utf-8")
        (root / ".dockerignore").write_text("target\n", encoding="utf-8")
        crate = root / "crates" / "canvas-engine"
        if crates_link_target is None:
            (crate / "src").mkdir(parents=True)
        else:
            # "crates" itself becomes a link to another directory that is still
            # inside the checkout, so realpath stays contained and os.walk sees
            # ordinary children. Only an ancestor check can catch this.
            real_parent = root / crates_link_target
            (real_parent / "canvas-engine" / "src").mkdir(parents=True)
            created = _create_directory_link(real_parent, root / "crates")
            if created:
                links["crates"] = created
        (crate / "Cargo.toml").write_text(_FIXTURE_CRATE_CARGO, encoding="utf-8")
        (crate / "Dockerfile").write_text("FROM scratch\n", encoding="utf-8")
        for name in src_files:
            (crate / "src" / name).write_text("// fixture source\n", encoding="utf-8")
        for name in marker_files:
            (crate / "src" / name).write_text(
                f"// {SMUGGLED_MARKER} retired payload, benign filename\n", encoding="utf-8"
            )
        if include_policy:
            policy_dir = root / "scripts" / "release"
            policy_dir.mkdir(parents=True, exist_ok=True)
            shutil.copy2(MODULE_PATH, policy_dir / MODULE_PATH.name)
        if smuggle_link is not None:
            # Retired payload lives outside the checkout, exactly like crates/star-game.
            outside = root.parent / "outside" / "star-game-src"
            outside.mkdir(parents=True)
            (outside / "lib.rs").write_text(
                f"// {SMUGGLED_MARKER} from star-game\n", encoding="utf-8"
            )
            created = _create_directory_link(outside, crate / "src" / smuggle_link)
            if created:
                links["smuggle"] = created
        return links

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

    def test_benign_named_link_would_smuggle_content_without_the_gate(self) -> None:
        """Control: prove the fixture models a real bypass, not a harmless tree.

        ``shutil.copytree`` is what the workflow uses, so copying the same fixture
        without the new gate must actually pull the retired payload in under the
        benign name. If this ever stops reproducing, the rejection test below
        would be proving nothing.
        """
        with tempfile.TemporaryDirectory() as temporary:
            workspace = Path(temporary) / "workspace"
            workspace.mkdir()
            links = self.build_fixture(workspace, smuggle_link="vendor")
            if "smuggle" not in links:
                self.skipTest("platform refuses link creation; no link fixture possible")
            destination = Path(temporary) / "copied-without-gate"
            shutil.copytree(workspace / "crates" / "canvas-engine" / "src", destination)
            copied = {
                path.relative_to(destination).as_posix(): path.read_text(encoding="utf-8")
                for path in destination.rglob("*")
                if path.is_file()
            }
        self.assertIn("vendor/lib.rs", copied)
        self.assertIn(SMUGGLED_MARKER, copied["vendor/lib.rs"])
        self.assertNotIn("game", "vendor/lib.rs")
        self.assertNotIn("legacy", "vendor/lib.rs")

    def test_rejects_benign_named_link_that_would_smuggle_game_content(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            workspace = Path(temporary) / "workspace"
            workspace.mkdir()
            links = self.build_fixture(workspace, smuggle_link="vendor")
            if "smuggle" not in links:
                self.skipTest("platform refuses link creation; no link fixture possible")
            result = self.run_gate(workspace)
            context = (
                workspace.parent / "runner-temp" / "star-canvas-oci" / "canvas-engine"
            )
            smuggled = [
                path.relative_to(context).as_posix()
                for path in context.rglob("*")
                if path.is_file()
                and SMUGGLED_MARKER in path.read_text(encoding="utf-8", errors="replace")
            ]
        self.assert_rejected(
            result, "image context sources are not link-free", "vendor", links["smuggle"]
        )
        self.assertEqual(smuggled, [], msg="retired payload reached the generated context")

    def test_internal_parent_link_would_smuggle_content_without_the_gate(self) -> None:
        """Control: a linked `crates` parent stays inside the checkout, so
        realpath containment and os.walk both see ordinary children."""
        with tempfile.TemporaryDirectory() as temporary:
            workspace = Path(temporary) / "workspace"
            workspace.mkdir()
            links = self.build_fixture(
                workspace,
                crates_link_target="legacy-source",
                src_files=(),
                marker_files=("payload.rs",),
            )
            if "crates" not in links:
                self.skipTest("platform refuses link creation; no link fixture possible")
            crate = workspace / "crates" / "canvas-engine"
            # The linked parent resolves inside the checkout: containment alone passes.
            self.assertTrue(
                Path(os.path.realpath(crate)).is_relative_to(Path(os.path.realpath(workspace)))
            )
            self.assertFalse((workspace / "crates").is_symlink())
            destination = Path(temporary) / "copied-without-gate"
            shutil.copytree(crate / "src", destination)
            copied = {
                path.relative_to(destination).as_posix(): path.read_text(encoding="utf-8")
                for path in destination.rglob("*")
                if path.is_file()
            }
        self.assertIn("payload.rs", copied)
        self.assertIn(SMUGGLED_MARKER, copied["payload.rs"])
        self.assertNotIn("game", "payload.rs")
        self.assertNotIn("legacy", "payload.rs")

    def test_rejects_internal_parent_link_before_copying_payload(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            workspace = Path(temporary) / "workspace"
            workspace.mkdir()
            links = self.build_fixture(
                workspace,
                crates_link_target="legacy-source",
                src_files=(),
                marker_files=("payload.rs",),
            )
            if "crates" not in links:
                self.skipTest("platform refuses link creation; no link fixture possible")
            result = self.run_gate(workspace)
            context = (
                workspace.parent / "runner-temp" / "star-canvas-oci" / "canvas-engine"
            )
            smuggled = [
                path.relative_to(context).as_posix()
                for path in context.rglob("*")
                if path.is_file()
                and SMUGGLED_MARKER in path.read_text(encoding="utf-8", errors="replace")
            ]
        self.assert_rejected(
            result, "image context sources are not link-free", "crates", links["crates"]
        )
        self.assertEqual(smuggled, [], msg="rejection must happen before any payload copy")

    def test_gate_fails_closed_without_junction_api_on_windows(self) -> None:
        """End-to-end M11: a pre-3.12 Windows interpreter cannot rule out junctions,
        so the real workflow gate must refuse to assemble the context at all."""
        with tempfile.TemporaryDirectory() as temporary:
            workspace = Path(temporary) / "workspace"
            workspace.mkdir()
            self.build_fixture(workspace)
            policy_copy = workspace / "scripts" / "release" / MODULE_PATH.name
            text = policy_copy.read_text(encoding="utf-8")
            patched = text.replace(
                '_IS_WINDOWS: bool = os.name == "nt"', "_IS_WINDOWS: bool = True"
            ).replace(
                "_JUNCTION_API_AVAILABLE: bool = hasattr(Path, \"is_junction\")",
                "_JUNCTION_API_AVAILABLE: bool = False",
            )
            # Prove the simulation actually applied; otherwise this test would pass
            # for the wrong reason on an interpreter that still has the API.
            self.assertNotEqual(patched, text, "capability simulation did not apply")
            self.assertIn("_IS_WINDOWS: bool = True", patched)
            self.assertIn("_JUNCTION_API_AVAILABLE: bool = False", patched)
            policy_copy.write_text(patched, encoding="utf-8")
            result = self.run_gate(workspace)
            context = (
                workspace.parent / "runner-temp" / "star-canvas-oci" / "canvas-engine"
            )
            copied = (
                [p.relative_to(context).as_posix() for p in context.rglob("crates/*")]
                if context.exists()
                else []
            )
        self.assert_rejected(result, "not link-free", "junction-check-unavailable")
        self.assertEqual(copied, [], msg="no crate content may be copied when links are unverifiable")


class LinkPolicyTests(unittest.TestCase):
    """Unit-level checks for the link/escape rule used before any copy."""

    def setUp(self) -> None:
        self._temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self._temporary.cleanup)
        self.root = Path(self._temporary.name)
        self.allowed = self.root / "checkout"
        (self.allowed / "crates" / "canvas-engine" / "src").mkdir(parents=True)
        (self.allowed / "crates" / "canvas-engine" / "src" / "lib.rs").write_text(
            "// benign\n", encoding="utf-8"
        )
        self.outside = self.root / "outside"
        self.outside.mkdir()

    def test_clean_tree_is_accepted(self) -> None:
        crate = self.allowed / "crates" / "canvas-engine"
        self.assertEqual(POLICY.find_link_violations(crate, allowed_root=self.allowed), ())
        checked = POLICY.ensure_link_free_sources([crate], allowed_root=self.allowed)
        self.assertEqual(len(checked), 1)

    def test_missing_path_fails_closed(self) -> None:
        missing = self.allowed / "crates" / "canvas-realtime"
        self.assertEqual(
            POLICY.find_link_violations(missing, allowed_root=self.allowed),
            ("canvas-realtime (missing)",),
        )

    def test_path_outside_the_checkout_fails_closed(self) -> None:
        violations = POLICY.find_link_violations(self.outside, allowed_root=self.allowed)
        self.assertEqual(len(violations), 1)
        self.assertIn("escapes the checkout", violations[0])
        with self.assertRaises(POLICY.CanvasContextError):
            POLICY.ensure_link_free_sources([self.outside], allowed_root=self.allowed)

    def test_link_inside_the_source_tree_fails_closed(self) -> None:
        src = self.allowed / "crates" / "canvas-engine" / "src"
        kind = _create_directory_link(self.outside, src / "vendor")
        if kind is None:
            self.skipTest("platform refuses link creation; no link fixture possible")
        violations = POLICY.find_link_violations(src.parent, allowed_root=self.allowed)
        self.assertEqual(violations, (f"src/vendor ({kind})",))
        with self.assertRaises(POLICY.CanvasContextError) as caught:
            POLICY.ensure_link_free_sources([src.parent], allowed_root=self.allowed)
        self.assertIn("not link-free", str(caught.exception))
        self.assertIn("src/vendor", str(caught.exception))

    def test_linked_ancestor_inside_the_checkout_fails_closed(self) -> None:
        """A linked parent keeps realpath contained, so only the ancestor chain catches it."""
        checkout = self.root / "checkout2"
        (checkout / "legacy-source" / "canvas-engine" / "src").mkdir(parents=True)
        (checkout / "legacy-source" / "canvas-engine" / "src" / "payload.rs").write_text(
            "// benign filename, retired payload inside\n", encoding="utf-8"
        )
        kind = _create_directory_link(checkout / "legacy-source", checkout / "crates")
        if kind is None:
            self.skipTest("platform refuses link creation; no link fixture possible")
        crate = checkout / "crates" / "canvas-engine"
        self.assertTrue(
            Path(os.path.realpath(crate)).is_relative_to(Path(os.path.realpath(checkout)))
        )
        self.assertFalse(crate.is_symlink())
        self.assertEqual(POLICY.find_link_violations(crate, allowed_root=checkout), (f"crates ({kind})",))
        with self.assertRaises(POLICY.CanvasContextError) as caught:
            POLICY.ensure_link_free_sources([crate], allowed_root=checkout)
        self.assertIn("crates", str(caught.exception))

    def test_windows_without_junction_api_fails_closed(self) -> None:
        """Python <=3.11 on Windows has no Path.is_junction, so a junction cannot
        be ruled out; the gate must reject rather than report the path as clean."""
        crate = self.allowed / "crates" / "canvas-engine"
        original = (POLICY._IS_WINDOWS, POLICY._JUNCTION_API_AVAILABLE)
        self.addCleanup(setattr, POLICY, "_IS_WINDOWS", original[0])
        self.addCleanup(setattr, POLICY, "_JUNCTION_API_AVAILABLE", original[1])
        POLICY._IS_WINDOWS = True
        POLICY._JUNCTION_API_AVAILABLE = False
        self.assertEqual(POLICY._link_kind(crate), "junction-check-unavailable")
        violations = POLICY.find_link_violations(crate, allowed_root=self.allowed)
        self.assertNotEqual(violations, ())
        self.assertTrue(all("junction-check-unavailable" in item for item in violations))
        with self.assertRaises(POLICY.CanvasContextError) as caught:
            POLICY.ensure_link_free_sources([crate], allowed_root=self.allowed)
        self.assertIn("not link-free", str(caught.exception))
        self.assertIn("junction-check-unavailable", str(caught.exception))

    def test_non_windows_without_junction_api_still_accepts_ordinary_paths(self) -> None:
        """Junctions do not exist off Windows, so the fail-closed branch must stay
        scoped to nt and must not break the Linux runners that publish images."""
        crate = self.allowed / "crates" / "canvas-engine"
        original = (POLICY._IS_WINDOWS, POLICY._JUNCTION_API_AVAILABLE)
        self.addCleanup(setattr, POLICY, "_IS_WINDOWS", original[0])
        self.addCleanup(setattr, POLICY, "_JUNCTION_API_AVAILABLE", original[1])
        POLICY._IS_WINDOWS = False
        POLICY._JUNCTION_API_AVAILABLE = False
        self.assertIsNone(POLICY._link_kind(crate))
        self.assertEqual(POLICY.find_link_violations(crate, allowed_root=self.allowed), ())
        self.assertEqual(len(POLICY.ensure_link_free_sources([crate], allowed_root=self.allowed)), 1)

    def test_junction_is_detected_even_though_is_symlink_is_false(self) -> None:
        """Windows junctions are the case a symlink-only check would miss."""
        if sys.platform != "win32":
            self.skipTest("junctions are a Windows reparse point")
        src = self.allowed / "crates" / "canvas-engine" / "src"
        kind = _create_directory_link(self.outside, src / "vendor")
        if kind is None:
            self.skipTest("platform refuses link creation; no link fixture possible")
        link = src / "vendor"
        self.assertFalse(link.is_symlink())
        self.assertEqual(POLICY._link_kind(link), "junction")
        self.assertNotEqual(POLICY.find_link_violations(src.parent, allowed_root=self.allowed), ())

    def test_link_violations_are_deterministic(self) -> None:
        src = self.allowed / "crates" / "canvas-engine" / "src"
        if _create_directory_link(self.outside, src / "zeta") is None:
            self.skipTest("platform refuses link creation; no link fixture possible")
        _create_directory_link(self.outside, src / "alpha")
        results = {
            POLICY.find_link_violations(src.parent, allowed_root=self.allowed) for _ in range(3)
        }
        self.assertEqual(len(results), 1)


if __name__ == "__main__":
    unittest.main()
