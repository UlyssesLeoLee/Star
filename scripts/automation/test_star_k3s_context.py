#!/usr/bin/env python3
# -*- coding: utf-8 -*-
# @cypher schema=1 source_sha256=69b2b1790f79ba77c012294cb5a876c2917ea8e94c91763eb22db401c751ad66
# MERGE (self:File {path:"scripts/automation/test_star_k3s_context.py"})
# MERGE (copy_fixture_context:Symbol {id:"scripts/automation/test_star_k3s_context.py::_copy_fixture_context", kind:"function"})
# MERGE (guest_deploy:Symbol {id:"scripts/automation/test_star_k3s_context.py::StarK3sContextTests.test_build_import_and_deploy_target_selected_guest", kind:"method"})
# MERGE (reject_instance:Symbol {id:"scripts/automation/test_star_k3s_context.py::StarK3sContextTests.test_rejects_missing_or_stopped_instance_before_guest_side_effects", kind:"method"})
# MERGE (reject_guest:Symbol {id:"scripts/automation/test_star_k3s_context.py::StarK3sContextTests.test_rejects_unready_guest_before_transfer", kind:"method"})
# MERGE (reject_k3s:Symbol {id:"scripts/automation/test_star_k3s_context.py::StarK3sContextTests.test_rejects_unready_k3s_before_transfer", kind:"method"})
# MERGE (manifest_tag:Symbol {id:"scripts/automation/test_star_k3s_context.py::StarK3sContextTests.test_manifest_pins_selected_local_tag", kind:"method"})
# MERGE (deploy_script:Symbol {id:"scripts/automation/test_star_k3s_context.py::StarK3sContextTests.test_deploy_script_requires_explicit_guest_and_has_no_host_docker_or_k3s", kind:"method"})
# MERGE (suite:Symbol {id:"scripts/automation/test_star_k3s_context.py::StarK3sContextTests", kind:"class"})
# MERGE (write_fixture:Symbol {id:"scripts/automation/test_star_k3s_context.py::write_fixture", kind:"function"})
# MERGE (tree_digest:Symbol {id:"scripts/automation/test_star_k3s_context.py::_tree_digest", kind:"function"})
# MERGE (setup:Symbol {id:"scripts/automation/test_star_k3s_context.py::StarK3sContextTests.setUp", kind:"method"})
# MERGE (teardown:Symbol {id:"scripts/automation/test_star_k3s_context.py::StarK3sContextTests.tearDown", kind:"method"})
# MERGE (allowed:Symbol {id:"scripts/automation/test_star_k3s_context.py::StarK3sContextTests.test_generates_only_the_resolved_star_closure", kind:"method"})
# MERGE (reject_game:Symbol {id:"scripts/automation/test_star_k3s_context.py::StarK3sContextTests.test_rejects_game_dependency", kind:"method"})
# MERGE (reject_missing:Symbol {id:"scripts/automation/test_star_k3s_context.py::StarK3sContextTests.test_rejects_missing_dependency_node", kind:"method"})
# MERGE (reject_manifest:Symbol {id:"scripts/automation/test_star_k3s_context.py::StarK3sContextTests.test_rejects_missing_local_manifest", kind:"method"})
# MERGE (reject_escape:Symbol {id:"scripts/automation/test_star_k3s_context.py::StarK3sContextTests.test_rejects_path_escape", kind:"method"})
# MERGE (reject_symlink:Symbol {id:"scripts/automation/test_star_k3s_context.py::StarK3sContextTests.test_rejects_symlinked_root_input", kind:"method"})
# MERGE (reject_junction:Symbol {id:"scripts/automation/test_star_k3s_context.py::StarK3sContextTests.test_rejects_junction_inside_selected_package", kind:"method"})
# MERGE (stable:Symbol {id:"scripts/automation/test_star_k3s_context.py::StarK3sContextTests.test_output_is_stable", kind:"method"})
# MERGE (tamper:Symbol {id:"scripts/automation/test_star_k3s_context.py::StarK3sContextTests.test_build_rejects_context_tampering", kind:"method"})
# MERGE (unlisted:Symbol {id:"scripts/automation/test_star_k3s_context.py::StarK3sContextTests.test_build_rejects_unlisted_context_changes", kind:"method"})
# MERGE (deploy_context:Symbol {id:"scripts/automation/test_star_k3s_context.py::StarK3sContextTests.test_deploy_script_uses_filtered_context_generator", kind:"method"})
# MERGE (lock_drift:Symbol {id:"scripts/automation/test_star_k3s_context.py::StarK3sContextTests.test_rejects_source_lock_identity_or_checksum_drift", kind:"method"})
# MERGE (self)-[:DEFINES]->(suite)
# MERGE (self)-[:DEFINES]->(copy_fixture_context)
# MERGE (self)-[:DEFINES]->(guest_deploy)
# MERGE (self)-[:DEFINES]->(reject_instance)
# MERGE (self)-[:DEFINES]->(reject_guest)
# MERGE (self)-[:DEFINES]->(reject_k3s)
# MERGE (self)-[:DEFINES]->(manifest_tag)
# MERGE (self)-[:DEFINES]->(deploy_script)
# MERGE (self)-[:DEFINES]->(write_fixture)
# MERGE (self)-[:DEFINES]->(tree_digest)
# MERGE (suite)-[:HAS_METHOD]->(setup)
# MERGE (suite)-[:HAS_METHOD]->(teardown)
# MERGE (suite)-[:HAS_METHOD]->(allowed)
# MERGE (suite)-[:HAS_METHOD]->(reject_game)
# MERGE (suite)-[:HAS_METHOD]->(reject_missing)
# MERGE (suite)-[:HAS_METHOD]->(reject_manifest)
# MERGE (suite)-[:HAS_METHOD]->(reject_escape)
# MERGE (suite)-[:HAS_METHOD]->(reject_symlink)
# MERGE (suite)-[:HAS_METHOD]->(reject_junction)
# MERGE (suite)-[:HAS_METHOD]->(stable)
# MERGE (suite)-[:HAS_METHOD]->(guest_deploy)
# MERGE (suite)-[:HAS_METHOD]->(reject_instance)
# MERGE (suite)-[:HAS_METHOD]->(reject_guest)
# MERGE (suite)-[:HAS_METHOD]->(reject_k3s)
# MERGE (suite)-[:HAS_METHOD]->(manifest_tag)
# MERGE (suite)-[:HAS_METHOD]->(deploy_script)
# MERGE (suite)-[:HAS_METHOD]->(tamper)
# MERGE (suite)-[:HAS_METHOD]->(unlisted)
# MERGE (suite)-[:HAS_METHOD]->(deploy_context)
# MERGE (suite)-[:HAS_METHOD]->(lock_drift)
# MERGE (setup)-[:CALLS]->(write_fixture)
# MERGE (copy_fixture_context)-[:CALLS]->(:Symbol {id:"scripts/automation/star_k3s_context.py::copy_context", kind:"function"})
# MERGE (allowed)-[:CALLS]->(:Symbol {id:"scripts/automation/star_k3s_context.py::copy_context", kind:"function"})
# MERGE (allowed)-[:CALLS]->(:Symbol {id:"scripts/automation/star_k3s_context.py::select_packages", kind:"function"})
# MERGE (reject_game)-[:CALLS]->(:Symbol {id:"scripts/automation/star_k3s_context.py::select_packages", kind:"function"})
# MERGE (reject_missing)-[:CALLS]->(:Symbol {id:"scripts/automation/star_k3s_context.py::select_packages", kind:"function"})
# MERGE (reject_manifest)-[:CALLS]->(:Symbol {id:"scripts/automation/star_k3s_context.py::select_packages", kind:"function"})
# MERGE (reject_escape)-[:CALLS]->(:Symbol {id:"scripts/automation/star_k3s_context.py::select_packages", kind:"function"})
# MERGE (reject_symlink)-[:CALLS]->(:Symbol {id:"scripts/automation/star_k3s_context.py::copy_context", kind:"function"})
# MERGE (reject_junction)-[:CALLS]->(copy_fixture_context)
# MERGE (reject_junction)-[:TESTS]->(:Symbol {id:"scripts/automation/star_k3s_context.py::_copy_local_package", kind:"function"})
# MERGE (stable)-[:CALLS]->(:Symbol {id:"scripts/automation/star_k3s_context.py::copy_context", kind:"function"})
# MERGE (stable)-[:CALLS]->(tree_digest)
# MERGE (guest_deploy)-[:CALLS]->(:Symbol {id:"scripts/automation/star_k3s_context.py::deploy_in_multipass", kind:"function"})
# MERGE (guest_deploy)-[:CALLS]->(:Symbol {id:"scripts/automation/star_k3s_context.py::_stream_image_to_k3s", kind:"function"})
# MERGE (reject_instance)-[:CALLS]->(:Symbol {id:"scripts/automation/star_k3s_context.py::_require_running_instance", kind:"function"})
# MERGE (reject_guest)-[:CALLS]->(:Symbol {id:"scripts/automation/star_k3s_context.py::_require_guest_prerequisites", kind:"function"})
# MERGE (reject_k3s)-[:CALLS]->(:Symbol {id:"scripts/automation/star_k3s_context.py::_require_guest_prerequisites", kind:"function"})
# MERGE (manifest_tag)-[:CALLS]->(:Symbol {id:"scripts/automation/star_k3s_context.py::_render_deployment_manifest", kind:"function"})
# MERGE (deploy_script)-[:TESTS]->(:File {path:"deploy/k3s-local/build-and-deploy.sh"})
# MERGE (tamper)-[:CALLS]->(:Symbol {id:"scripts/automation/star_k3s_context.py::deploy_in_multipass", kind:"function"})
# MERGE (unlisted)-[:CALLS]->(:Symbol {id:"scripts/automation/star_k3s_context.py::deploy_in_multipass", kind:"function"})
# MERGE (deploy_context)-[:TESTS]->(:File {path:"deploy/k3s-local/build-and-deploy.sh"})
# MERGE (lock_drift)-[:CALLS]->(:Symbol {id:"scripts/automation/star_k3s_context.py::_regenerate_context_lock", kind:"function"})
# @endcypher

"""Focused tests for the fail-closed local K3s image context."""

from __future__ import annotations

import hashlib
import io
import json
import os
import subprocess
import tempfile
import tomllib
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts.automation.star_k3s_context import (
    ContextError,
    _regenerate_context_lock,
    copy_context,
    deploy_in_multipass,
    _render_deployment_manifest,
    select_packages,
)


def write_fixture(root: Path, package_id_overrides: dict[str, str] | None = None) -> tuple[dict[str, object], str]:
    """Create a selected Star closure, an optional local crate, and an excluded game crate."""
    package_id_overrides = package_id_overrides or {}
    members = ["crates/star-api-rest", "crates/domain-project", "crates/domain-tenant", "crates/star-optional-proto", "crates/star-game"]
    workspace_lines = [
        "[workspace]",
        'resolver = "2"',
        "members = [",
        *[f'    {json.dumps(member)},' for member in members],
        "]",
        'exclude = ["target"]',
        "",
        "[workspace.package]",
        'edition = "2024"',
        'version = "1.0.0"',
        'license = "MIT"',
        "",
        "[workspace.lints.rust]",
        'unsafe_code = "forbid"',
        "",
        "[profile.release]",
        'lto = "thin"',
        "",
    ]
    root.mkdir(parents=True, exist_ok=True)
    (root / "Cargo.toml").write_text("\n".join(workspace_lines), encoding="utf-8")
    (root / "rust-toolchain.toml").write_text('[toolchain]\nchannel = "1.99.0"\n', encoding="utf-8")
    (root / "LICENSE").write_text("MIT\n", encoding="utf-8")
    (root / "deploy" / "k3s-local").mkdir(parents=True)
    dockerfile = "\n".join(
        [
            "FROM rust:1.99.0-slim-bookworm AS builder",
            "WORKDIR /build",
            "COPY Cargo.toml Cargo.lock rust-toolchain.toml LICENSE ./",
            "# @star-local-crates-begin",
            "# @star-local-crates-end",
            "RUN cargo build --release --locked -p star-api-rest",
            "",
        ]
    )
    (root / "deploy" / "k3s-local" / "Dockerfile").write_text(dockerfile, encoding="utf-8")

    crate_specs = {
        "star-api-rest": ('domain-project = { path = "../domain-project" }\nstar-optional-proto = { path = "../star-optional-proto", optional = true }', 'star-game = { path = "../star-game" }'),
        "domain-project": ('domain-tenant = { path = "../domain-tenant" }', ""),
        "domain-tenant": ("", ""),
        "star-optional-proto": ("", ""),
        "star-game": ("", ""),
    }
    package_ids = {}
    for name, (dependency, dev_dependency) in crate_specs.items():
        package_id = package_id_overrides.get(name, f"path+file:///fixture/crates/{name}#{name}@0.1.0")
        package_ids[name] = package_id
        crate_root = root / "crates" / name
        (crate_root / "src").mkdir(parents=True, exist_ok=True)
        (crate_root / "src" / "lib.rs").write_text("pub fn fixture() {}\n", encoding="utf-8")
        manifest = f'[package]\nname = "{name}"\nversion = "0.1.0"\nedition = "2024"\n'
        if dependency:
            manifest += f'\n[dependencies]\n{dependency}\n'
        if dev_dependency:
            manifest += f'\n[dev-dependencies]\n{dev_dependency}\n'
        if name == "star-api-rest":
            manifest += '\n[features]\nnative-proto = ["dep:star-optional-proto"]\n'
        (crate_root / "Cargo.toml").write_text(manifest, encoding="utf-8")

    metadata_packages = [
        {
            "id": package_ids[name],
            "name": name,
            "version": "0.1.0",
            "source": None,
            "manifest_path": str(root / "crates" / name / "Cargo.toml"),
        }
        for name in crate_specs
    ]
    nodes = [
        {
            "id": package_ids["star-api-rest"],
            "deps": [
                {"pkg": package_ids["domain-project"], "dep_kinds": [{"kind": None, "target": None}]},
                {"pkg": package_ids["star-game"], "dep_kinds": [{"kind": "dev", "target": None}]},
            ],
        },
        {
            "id": package_ids["domain-project"],
            "deps": [{"pkg": package_ids["domain-tenant"], "dep_kinds": [{"kind": None, "target": None}]}],
        },
        {"id": package_ids["domain-tenant"], "deps": []},
        {"id": package_ids["star-optional-proto"], "deps": []},
        {"id": package_ids["star-game"], "deps": []},
    ]
    metadata = {"packages": metadata_packages, "resolve": {"nodes": nodes}}
    lock_blocks = [
        "version = 4\n\n",
        '[[package]]\nname = "star-api-rest"\nversion = "0.1.0"\ndependencies = ["domain-project 0.1.0", "star-game 0.1.0", "star-optional-proto 0.1.0"]\n\n',
        '[[package]]\nname = "domain-project"\nversion = "0.1.0"\ndependencies = ["domain-tenant 0.1.0"]\n\n',
        '[[package]]\nname = "domain-tenant"\nversion = "0.1.0"\n\n',
        '[[package]]\nname = "star-optional-proto"\nversion = "0.1.0"\n\n',
        '[[package]]\nname = "star-game"\nversion = "0.1.0"\n\n',
    ]
    return metadata, "".join(lock_blocks)


def _tree_digest(directory: Path) -> str:
    value = hashlib.sha256()
    for file_path in sorted(path for path in directory.rglob("*") if path.is_file()):
        value.update(file_path.relative_to(directory).as_posix().encode("utf-8"))
        value.update(b"\0")
        value.update(file_path.read_bytes())
        value.update(b"\0")
    return value.hexdigest()


def _copy_fixture_context(repo: Path, output: Path, metadata: dict) -> tuple[list[str], str]:
    """Mock Cargo lock projection so filesystem policy tests need no native toolchain."""
    packages = [
        package
        for package in metadata["packages"]
        if package.get("source") is None and package.get("name") != "star-game"
    ]
    generated_lock = "version = 4\n\n" + "\n".join(
        f'[[package]]\nname = "{package["name"]}"\nversion = "{package["version"]}"\n'
        for package in packages
    )

    def project_lock(command: list[str], **kwargs: object) -> subprocess.CompletedProcess[bytes]:
        if command[:2] != ["cargo", "metadata"]:
            raise AssertionError(f"Unexpected subprocess in context fixture: {command}")
        Path(kwargs["cwd"]).joinpath("Cargo.lock").write_text(generated_lock, encoding="utf-8")
        return subprocess.CompletedProcess(command, 0, b"", b"")

    with patch("scripts.automation.star_k3s_context.subprocess.run", side_effect=project_lock):
        return copy_context(repo, output, metadata)


class StarK3sContextTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory(prefix="star-k3s-context-test-")
        self.base = Path(self.temporary.name)
        self.repo = self.base / "repo"
        self.metadata, self.lock_text = write_fixture(self.repo)
        (self.repo / "Cargo.lock").write_text(self.lock_text, encoding="utf-8")
        (self.repo / "deploy" / "k3s-local" / "star-api-rest-deploy.yaml").write_text(
            "apiVersion: apps/v1\nkind: Deployment\nspec:\n  template:\n    spec:\n      containers:\n        - name: star-api-rest\n          image: star-api-rest:local\n",
            encoding="utf-8",
        )

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def test_generates_only_the_resolved_star_closure(self) -> None:
        selected, local = select_packages(self.metadata, self.repo)
        self.assertEqual({package["name"] for package in local}, {"star-api-rest", "domain-project", "domain-tenant"})
        self.assertNotIn("star-game", {package["name"] for package in self.metadata["packages"] if package["id"] in selected})
        output = self.base / "context"
        files, _digest = _copy_fixture_context(self.repo, output, self.metadata)
        manifest = (output / "Cargo.toml").read_text(encoding="utf-8")
        api_manifest = (output / "crates" / "star-api-rest" / "Cargo.toml").read_text(encoding="utf-8")
        dockerfile = (output / "deploy" / "k3s-local" / "Dockerfile").read_text(encoding="utf-8")
        self.assertEqual(tomllib.loads(manifest)["workspace"]["members"], ["crates/domain-project", "crates/domain-tenant", "crates/star-api-rest"])
        self.assertIn("crates/star-optional-proto", tomllib.loads(manifest)["workspace"]["exclude"])
        self.assertIn("COPY crates/domain-project/ crates/domain-project/", dockerfile)
        self.assertIn("COPY crates/domain-tenant/ crates/domain-tenant/", dockerfile)
        self.assertIn("COPY crates/star-optional-proto/ crates/star-optional-proto/", dockerfile)
        self.assertNotIn("COPY crates ./crates", dockerfile)
        self.assertNotIn("star-game", (output / "Cargo.lock").read_text(encoding="utf-8"))
        self.assertFalse(any("game" in path.casefold() or "legacy" in path.casefold() for path in files))
        self.assertFalse((output / "crates" / "star-game").exists())
        self.assertTrue((output / "crates" / "star-optional-proto" / "Cargo.toml").is_file())
        self.assertNotIn("star-game", api_manifest)

    def test_rejects_source_lock_identity_or_checksum_drift(self) -> None:
        source_lock = (
            'version = 4\n\n[[package]]\nname = "serde"\nversion = "1.0.219"\n'
            'source = "registry+https://github.com/rust-lang/crates.io-index"\nchecksum = "source-checksum"\n'
        )
        generated_locks = [
            (
                'version = 4\n\n[[package]]\nname = "serde"\nversion = "1.0.219"\n'
                'source = "registry+https://github.com/rust-lang/crates.io-index"\nchecksum = "changed-checksum"\n',
                "checksum differs",
            ),
            (
                source_lock + '\n[[package]]\nname = "unlocked-extra"\nversion = "1.0.0"\n'
                'source = "registry+https://github.com/rust-lang/crates.io-index"\nchecksum = "extra-checksum"\n',
                "introduced a package absent from the source lock",
            ),
        ]
        for index, (generated_lock, error) in enumerate(generated_locks):
            with self.subTest(error=error):
                context = self.base / f"lock-drift-{index}"
                context.mkdir()

                def fake_cargo(_command: list[str], *, cwd: Path, **_kwargs: object) -> subprocess.CompletedProcess[bytes]:
                    (cwd / "Cargo.lock").write_text(generated_lock, encoding="utf-8")
                    return subprocess.CompletedProcess([], 0, stdout=b"{}", stderr=b"")

                with patch("scripts.automation.star_k3s_context.subprocess.run", side_effect=fake_cargo):
                    with self.assertRaisesRegex(ContextError, error):
                        _regenerate_context_lock(context, source_lock, [])

    def test_rejects_game_dependency(self) -> None:
        self.metadata["resolve"]["nodes"][0]["deps"].append(
            {"pkg": "path+file:///fixture/crates/star-game#star-game@0.1.0", "dep_kinds": [{"kind": None, "target": None}]}
        )
        with self.assertRaisesRegex(ContextError, "game/legacy|Forbidden package"):
            select_packages(self.metadata, self.repo)

    def test_rejects_missing_dependency_node(self) -> None:
        self.metadata["resolve"]["nodes"] = [node for node in self.metadata["resolve"]["nodes"] if not node["id"].endswith("domain-tenant@0.1.0")]
        with self.assertRaisesRegex(ContextError, "missing node"):
            select_packages(self.metadata, self.repo)

    def test_rejects_missing_local_manifest(self) -> None:
        package = next(package for package in self.metadata["packages"] if package["name"] == "domain-tenant")
        package["manifest_path"] = str(self.repo / "crates" / "missing-crate" / "Cargo.toml")
        with self.assertRaisesRegex(ContextError, "manifest is missing"):
            select_packages(self.metadata, self.repo)

    def test_rejects_path_escape(self) -> None:
        outside = self.base / "outside" / "Cargo.toml"
        outside.parent.mkdir()
        outside.write_text('[package]\nname = "outside-star"\nversion = "0.1.0"\n', encoding="utf-8")
        package_id = "path+file:///fixture/outside#outside-star@0.1.0"
        self.metadata["packages"].append({"id": package_id, "name": "outside-star", "version": "0.1.0", "source": None, "manifest_path": str(outside)})
        self.metadata["resolve"]["nodes"][0]["deps"].append({"pkg": package_id, "dep_kinds": [{"kind": None, "target": None}]})
        self.metadata["resolve"]["nodes"].append({"id": package_id, "deps": []})
        with self.assertRaisesRegex(ContextError, "escapes repository root"):
            select_packages(self.metadata, self.repo)

    def test_rejects_symlinked_root_input(self) -> None:
        outside = self.base / "outside-license"
        outside.write_text("not an approved repository input\n", encoding="utf-8")
        license_path = self.repo / "LICENSE"
        license_path.unlink()
        try:
            license_path.symlink_to(outside)
        except OSError as error:
            self.skipTest(f"symbolic links are unavailable in this environment: {error}")
        with self.assertRaisesRegex(ContextError, "Symlink or reparse point is not permitted"):
            _copy_fixture_context(self.repo, self.base / "symlink-context", self.metadata)

    @unittest.skipUnless(os.name == "nt", "Windows directory junctions are not available")
    def test_rejects_junction_inside_selected_package(self) -> None:
        outside = self.base / "outside-package-payload"
        outside.mkdir()
        (outside / "must-not-copy.txt").write_text("outside repository payload\n", encoding="utf-8")
        junction = self.repo / "crates" / "domain-project" / "external"
        junction_path = "'" + str(junction).replace("'", "''") + "'"
        outside_path = "'" + str(outside).replace("'", "''") + "'"
        command = (
            "$ErrorActionPreference = 'Stop'; "
            f"New-Item -ItemType Junction -Path {junction_path} -Target {outside_path} | Out-Null"
        )
        result = subprocess.run(
            ["powershell.exe", "-NoProfile", "-NonInteractive", "-Command", command],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(result.returncode, 0, f"Could not create Windows junction: {result.stdout}\n{result.stderr}")
        try:
            output = self.base / "junction-context"
            with self.assertRaisesRegex(ContextError, "Symlink or reparse point is not permitted"):
                _copy_fixture_context(self.repo, output, self.metadata)
            self.assertFalse((output / "crates" / "domain-project" / "external" / "must-not-copy.txt").exists())
        finally:
            junction.rmdir()

    def test_output_is_stable(self) -> None:
        first = self.base / "context-a"
        second = self.base / "context-b"
        _copy_fixture_context(self.repo, first, self.metadata)
        _copy_fixture_context(self.repo, second, self.metadata)
        self.assertEqual(_tree_digest(first), _tree_digest(second))

    def test_build_import_and_deploy_target_selected_guest(self) -> None:
        context = self.base / "build-context"
        _files, digest = _copy_fixture_context(self.repo, context, self.metadata)

        rendered_manifests: list[str] = []

        def run_command(command: list[str], **kwargs: object) -> subprocess.CompletedProcess[str]:
            if command == ["multipass", "list", "--format", "csv"]:
                return subprocess.CompletedProcess(command, 0, "Name,State\nstar-test,Running\n", "")
            if "kubectl" in command and "nodes" in command:
                return subprocess.CompletedProcess(
                    command,
                    0,
                    json.dumps({
                        "items": [
                            {
                                "metadata": {"name": "star-test"},
                                "status": {"conditions": [{"type": "Ready", "status": "True"}]},
                            }
                        ]
                    }),
                    "",
                )
            if command[:2] == ["multipass", "transfer"] and command[2].endswith(".yaml"):
                rendered_manifests.append(Path(command[2]).read_text(encoding="utf-8"))
            if "crictl" in command:
                return subprocess.CompletedProcess(
                    command,
                    0,
                    "IMAGE TAG IMAGE ID SIZE\ndocker.io/library/star-api-rest w7-test sha256:123 8MB\n",
                    "",
                )
            return subprocess.CompletedProcess(command, 0, "ready\n", "")

        processes: list[object] = []

        class FakeProcess:
            def __init__(self, command: list[str], **kwargs: object) -> None:
                self.args = command
                self.stdout = io.BytesIO() if kwargs.get("stdout") == subprocess.PIPE else None

            def wait(self) -> int:
                return 0

            def terminate(self) -> None:
                return None

        def popen(command: list[str], **kwargs: object) -> FakeProcess:
            process = FakeProcess(command, **kwargs)
            processes.append(process)
            return process

        with patch("scripts.automation.star_k3s_context.subprocess.run", side_effect=run_command) as run, patch(
            "scripts.automation.star_k3s_context.subprocess.Popen", side_effect=popen
        ):
            deploy_in_multipass(
                context,
                "star-test",
                "w7-test",
                digest,
                self.repo / "deploy" / "k3s-local" / "star-api-rest-deploy.yaml",
            )
        commands = [call.args[0] for call in run.call_args_list]
        self.assertIn(["multipass", "list", "--format", "csv"], commands)
        guest_commands = [command for command in commands if command[:2] == ["multipass", "exec"]]
        guest_commands.extend(process.args for process in processes)
        self.assertTrue(guest_commands)
        self.assertTrue(all(command[2] == "star-test" for command in guest_commands))
        build = next(command for command in guest_commands if "docker" in command and "build" in command)
        self.assertEqual(build[build.index("docker") + 1], "build")
        self.assertTrue(any(argument.endswith("deploy/k3s-local/Dockerfile") for argument in build))
        self.assertTrue(any("crictl" in command for command in guest_commands))
        kubectl_commands = [command for command in guest_commands if "kubectl" in command]
        self.assertTrue(any("apply" in command for command in kubectl_commands))
        self.assertTrue(any("rollout" in command for command in kubectl_commands))
        self.assertTrue(any("get" in command and "pods" in command for command in kubectl_commands))
        self.assertFalse(any(command[0] in {"docker", "kubectl", "sudo"} for command in commands))
        transfers = [command for command in commands if command[:2] == ["multipass", "transfer"]]
        self.assertEqual(len(transfers), 2)
        self.assertTrue(all(command[3].startswith("star-test:/tmp/star-k3s-context-") for command in transfers))
        rendered_manifest = rendered_manifests[0]
        self.assertIn("image: star-api-rest:w7-test", rendered_manifest)
        self.assertNotIn("image: star-api-rest:local", rendered_manifest)
        export_command = next(process.args for process in processes if "save" in process.args)
        import_command = next(process.args for process in processes if "ctr" in process.args)
        self.assertIn("docker", export_command)
        self.assertIn("k8s.io", import_command)
        self.assertEqual(import_command[-1], "-")

    def test_rejects_missing_or_stopped_instance_before_guest_side_effects(self) -> None:
        context = self.base / "reject-context"
        _files, digest = _copy_fixture_context(self.repo, context, self.metadata)
        for rows in ("Name,State\nother,Running\n", "Name,State\nstar-test,Stopped\n"):
            with self.subTest(rows=rows), patch(
                "scripts.automation.star_k3s_context.subprocess.run",
                return_value=subprocess.CompletedProcess(["multipass", "list"], 0, rows, ""),
            ) as run:
                with self.assertRaisesRegex(ContextError, "missing or ambiguous|not Running"):
                    deploy_in_multipass(
                        context,
                        "star-test",
                        "w7-test",
                        digest,
                        self.repo / "deploy" / "k3s-local" / "star-api-rest-deploy.yaml",
                    )
                self.assertEqual(run.call_count, 1)

    def test_rejects_unready_guest_before_transfer(self) -> None:
        context = self.base / "unready-context"
        _files, digest = _copy_fixture_context(self.repo, context, self.metadata)

        def run_command(command: list[str], **kwargs: object) -> subprocess.CompletedProcess[str]:
            if command == ["multipass", "list", "--format", "csv"]:
                return subprocess.CompletedProcess(command, 0, "Name,State\nstar-test,Running\n", "")
            return subprocess.CompletedProcess(command, 1, "", "docker unavailable")

        with patch("scripts.automation.star_k3s_context.subprocess.run", side_effect=run_command) as run, patch(
            "scripts.automation.star_k3s_context.subprocess.Popen"
        ) as popen:
            with self.assertRaisesRegex(ContextError, "no ready Docker build engine"):
                deploy_in_multipass(
                    context,
                    "star-test",
                    "w7-test",
                    digest,
                    self.repo / "deploy" / "k3s-local" / "star-api-rest-deploy.yaml",
                )
        self.assertEqual(run.call_count, 2)
        popen.assert_not_called()
        self.assertFalse(any(call.args[0][:2] == ["multipass", "transfer"] for call in run.call_args_list))

    def test_rejects_unready_k3s_before_transfer(self) -> None:
        context = self.base / "unready-k3s-context"
        _files, digest = _copy_fixture_context(self.repo, context, self.metadata)

        def run_command(command: list[str], **kwargs: object) -> subprocess.CompletedProcess[str]:
            if command == ["multipass", "list", "--format", "csv"]:
                return subprocess.CompletedProcess(command, 0, "Name,State\nstar-test,Running\n", "")
            if "docker" in command:
                return subprocess.CompletedProcess(command, 0, "28.0.0\n", "")
            return subprocess.CompletedProcess(
                command,
                0,
                json.dumps({
                    "items": [
                        {
                            "metadata": {"name": "star-test"},
                            "status": {"conditions": [{"type": "Ready", "status": "False"}]},
                        }
                    ]
                }),
                "",
            )

        with patch("scripts.automation.star_k3s_context.subprocess.run", side_effect=run_command) as run, patch(
            "scripts.automation.star_k3s_context.subprocess.Popen"
        ) as popen:
            with self.assertRaisesRegex(ContextError, "K3s nodes that are not Ready: star-test"):
                deploy_in_multipass(
                    context,
                    "star-test",
                    "w7-test",
                    digest,
                    self.repo / "deploy" / "k3s-local" / "star-api-rest-deploy.yaml",
                )
        self.assertEqual(run.call_count, 3)
        popen.assert_not_called()
        self.assertFalse(any(call.args[0][:2] == ["multipass", "transfer"] for call in run.call_args_list))

    def test_rejects_missing_or_invalid_node_readiness_before_transfer(self) -> None:
        context = self.base / "invalid-node-readiness-context"
        _files, digest = _copy_fixture_context(self.repo, context, self.metadata)
        cases = (
            ("malformed JSON", "{", "invalid K3s node readiness data"),
            ("empty node list", json.dumps({"items": []}), "no K3s nodes"),
            (
                "missing Ready condition",
                json.dumps({"items": [{"metadata": {"name": "star-test"}, "status": {"conditions": []}}]}),
                "K3s nodes that are not Ready: star-test",
            ),
        )

        for label, node_json, error in cases:
            with self.subTest(case=label):
                def run_command(command: list[str], **kwargs: object) -> subprocess.CompletedProcess[str]:
                    if command == ["multipass", "list", "--format", "csv"]:
                        return subprocess.CompletedProcess(command, 0, "Name,State\nstar-test,Running\n", "")
                    if "docker" in command:
                        return subprocess.CompletedProcess(command, 0, "28.0.0\n", "")
                    if "kubectl" in command and "nodes" in command:
                        return subprocess.CompletedProcess(command, 0, node_json, "")
                    return subprocess.CompletedProcess(command, 0, "ready\n", "")

                with patch("scripts.automation.star_k3s_context.subprocess.run", side_effect=run_command) as run, patch(
                    "scripts.automation.star_k3s_context.subprocess.Popen"
                ) as popen:
                    with self.assertRaisesRegex(ContextError, error):
                        deploy_in_multipass(
                            context,
                            "star-test",
                            "w7-test",
                            digest,
                            self.repo / "deploy" / "k3s-local" / "star-api-rest-deploy.yaml",
                        )
                popen.assert_not_called()
                self.assertFalse(any(call.args[0][:2] == ["multipass", "transfer"] for call in run.call_args_list))

    def test_manifest_pins_selected_local_tag(self) -> None:
        output = self.base / "deployment.yaml"
        _render_deployment_manifest(
            self.repo / "deploy" / "k3s-local" / "star-api-rest-deploy.yaml",
            output,
            "star-api-rest:stable-1",
        )
        self.assertIn("image: star-api-rest:stable-1", output.read_text(encoding="utf-8"))

    def test_deploy_script_requires_explicit_guest_and_has_no_host_docker_or_k3s(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        script = (repo_root / "deploy" / "k3s-local" / "build-and-deploy.sh").read_text(encoding="utf-8")
        self.assertIn('[[ $# -lt 1 || $# -gt 2 || -z "${1:-}" ]]', script)
        self.assertIn('--build-in-multipass "${INSTANCE}"', script)
        self.assertIn('--image-tag "${IMAGE_TAG}"', script)
        self.assertNotIn("docker build", script)
        self.assertNotIn("docker save", script)
        self.assertNotIn("/usr/local/bin/k3s", script)

    def test_deploy_script_uses_filtered_context_generator(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        script = (repo_root / "deploy" / "k3s-local" / "build-and-deploy.sh").read_text(encoding="utf-8")

        self.assertIn('star_k3s_context.py" \\', script)
        self.assertIn('--repo "${REPO_ROOT}" \\', script)
        self.assertIn('--build-in-multipass "${INSTANCE}"', script)
        self.assertIn('--image-tag "${IMAGE_TAG}"', script)
        self.assertNotIn('cp -r "${REPO_ROOT}/crates"', script)

    def test_build_rejects_context_tampering(self) -> None:
        context = self.base / "tampered-context"
        _files, digest = _copy_fixture_context(self.repo, context, self.metadata)
        (context / "crates" / "star-game").mkdir()
        with patch("scripts.automation.star_k3s_context.subprocess.run") as run:
            with self.assertRaisesRegex(ContextError, "(?i)game/legacy input"):
                deploy_in_multipass(
                    context,
                    "star-test",
                    "w7-test",
                    digest,
                    self.repo / "deploy" / "k3s-local" / "star-api-rest-deploy.yaml",
                )
        run.assert_not_called()

    def test_build_rejects_unlisted_context_changes(self) -> None:
        context = self.base / "changed-context"
        _files, digest = _copy_fixture_context(self.repo, context, self.metadata)
        (context / "README.private").write_text("unexpected input\n", encoding="utf-8")
        with patch("scripts.automation.star_k3s_context.subprocess.run") as run:
            with self.assertRaisesRegex(ContextError, "unexpected root entries|no longer matches"):
                deploy_in_multipass(
                    context,
                    "star-test",
                    "w7-test",
                    digest,
                    self.repo / "deploy" / "k3s-local" / "star-api-rest-deploy.yaml",
                )
        run.assert_not_called()


if __name__ == "__main__":
    unittest.main()
