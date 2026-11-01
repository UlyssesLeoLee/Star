#!/usr/bin/env python3
# -*- coding: utf-8 -*-
# @cypher schema=1 source_sha256=ac66a4f442ed3b799fbff3f184f43da84edb4d4e9c8fdaf1d82b70d62037929f
# MERGE (self:File {path:"scripts/automation/test_star_k3s_context.py"})
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
# MERGE (stable:Symbol {id:"scripts/automation/test_star_k3s_context.py::StarK3sContextTests.test_output_is_stable", kind:"method"})
# MERGE (multipass:Symbol {id:"scripts/automation/test_star_k3s_context.py::StarK3sContextTests.test_build_invokes_multipass_not_host_docker", kind:"method"})
# MERGE (tamper:Symbol {id:"scripts/automation/test_star_k3s_context.py::StarK3sContextTests.test_build_rejects_context_tampering", kind:"method"})
# MERGE (unlisted:Symbol {id:"scripts/automation/test_star_k3s_context.py::StarK3sContextTests.test_build_rejects_unlisted_context_changes", kind:"method"})
# MERGE (self)-[:DEFINES]->(suite)
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
# MERGE (suite)-[:HAS_METHOD]->(stable)
# MERGE (suite)-[:HAS_METHOD]->(multipass)
# MERGE (suite)-[:HAS_METHOD]->(tamper)
# MERGE (suite)-[:HAS_METHOD]->(unlisted)
# MERGE (setup)-[:CALLS]->(write_fixture)
# MERGE (allowed)-[:CALLS]->(:Symbol {id:"scripts/automation/star_k3s_context.py::copy_context", kind:"function"})
# MERGE (allowed)-[:CALLS]->(:Symbol {id:"scripts/automation/star_k3s_context.py::select_packages", kind:"function"})
# MERGE (allowed)-[:CALLS]->(:Symbol {id:"scripts/automation/star_k3s_context.py::filter_lockfile", kind:"function"})
# MERGE (reject_game)-[:CALLS]->(:Symbol {id:"scripts/automation/star_k3s_context.py::select_packages", kind:"function"})
# MERGE (reject_missing)-[:CALLS]->(:Symbol {id:"scripts/automation/star_k3s_context.py::select_packages", kind:"function"})
# MERGE (reject_manifest)-[:CALLS]->(:Symbol {id:"scripts/automation/star_k3s_context.py::select_packages", kind:"function"})
# MERGE (reject_escape)-[:CALLS]->(:Symbol {id:"scripts/automation/star_k3s_context.py::select_packages", kind:"function"})
# MERGE (reject_symlink)-[:CALLS]->(:Symbol {id:"scripts/automation/star_k3s_context.py::copy_context", kind:"function"})
# MERGE (stable)-[:CALLS]->(:Symbol {id:"scripts/automation/star_k3s_context.py::copy_context", kind:"function"})
# MERGE (stable)-[:CALLS]->(tree_digest)
# MERGE (multipass)-[:CALLS]->(:Symbol {id:"scripts/automation/star_k3s_context.py::build_in_multipass", kind:"function"})
# MERGE (multipass)-[:CALLS]->(:Symbol {id:"scripts/automation/star_k3s_context.py::copy_context", kind:"function"})
# MERGE (tamper)-[:CALLS]->(:Symbol {id:"scripts/automation/star_k3s_context.py::build_in_multipass", kind:"function"})
# MERGE (unlisted)-[:CALLS]->(:Symbol {id:"scripts/automation/star_k3s_context.py::build_in_multipass", kind:"function"})
# @endcypher

"""Focused tests for the fail-closed local K3s image context."""

from __future__ import annotations

import hashlib
import json
import subprocess
import tempfile
import tomllib
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts.automation.star_k3s_context import ContextError, build_in_multipass, copy_context, filter_lockfile, select_packages


def write_fixture(root: Path, package_id_overrides: dict[str, str] | None = None) -> tuple[dict[str, object], str]:
    """Create a small workspace with two Star crates, a registry crate, and an excluded game crate."""
    package_id_overrides = package_id_overrides or {}
    members = ["crates/star-api-rest", "crates/domain-project", "crates/domain-tenant", "crates/star-game"]
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
        "[workspace.dependencies]",
        'serde = "1.0.219"',
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
        "star-api-rest": ('domain-project = { path = "../domain-project" }\nserde = { workspace = true }', 'star-game = { path = "../star-game" }'),
        "domain-project": ('domain-tenant = { path = "../domain-tenant" }', ""),
        "domain-tenant": ('serde = { workspace = true }', ""),
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
        (crate_root / "Cargo.toml").write_text(manifest, encoding="utf-8")

    registry_id = "registry+https://github.com/rust-lang/crates.io-index#serde@1.0.219"
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
    metadata_packages.append(
        {"id": registry_id, "name": "serde", "version": "1.0.219", "source": "registry+https://github.com/rust-lang/crates.io-index", "manifest_path": ""}
    )
    nodes = [
        {
            "id": package_ids["star-api-rest"],
            "deps": [
                {"pkg": package_ids["domain-project"], "dep_kinds": [{"kind": None, "target": None}]},
                {"pkg": package_ids["star-game"], "dep_kinds": [{"kind": "dev", "target": None}]},
                {"pkg": registry_id, "dep_kinds": [{"kind": None, "target": None}]},
            ],
        },
        {
            "id": package_ids["domain-project"],
            "deps": [{"pkg": package_ids["domain-tenant"], "dep_kinds": [{"kind": None, "target": None}]}],
        },
        {
            "id": package_ids["domain-tenant"],
            "deps": [{"pkg": registry_id, "dep_kinds": [{"kind": None, "target": None}]}],
        },
        {"id": package_ids["star-game"], "deps": []},
        {"id": registry_id, "deps": []},
    ]
    metadata = {"packages": metadata_packages, "resolve": {"nodes": nodes}}
    lock_blocks = [
        "version = 4\n\n",
        '[[package]]\nname = "star-api-rest"\nversion = "0.1.0"\ndependencies = ["domain-project 0.1.0", "serde 1.0.219", "star-game 0.1.0"]\n\n',
        '[[package]]\nname = "domain-project"\nversion = "0.1.0"\ndependencies = ["domain-tenant 0.1.0"]\n\n',
        '[[package]]\nname = "domain-tenant"\nversion = "0.1.0"\ndependencies = ["serde 1.0.219"]\n\n',
        '[[package]]\nname = "serde"\nversion = "1.0.219"\nsource = "registry+https://github.com/rust-lang/crates.io-index"\n\n',
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


class StarK3sContextTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory(prefix="star-k3s-context-test-")
        self.base = Path(self.temporary.name)
        self.repo = self.base / "repo"
        self.metadata, self.lock_text = write_fixture(self.repo)
        (self.repo / "Cargo.lock").write_text(self.lock_text, encoding="utf-8")

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def test_generates_only_the_resolved_star_closure(self) -> None:
        selected, local = select_packages(self.metadata, self.repo)
        self.assertEqual({package["name"] for package in local}, {"star-api-rest", "domain-project", "domain-tenant"})
        filtered = filter_lockfile(self.lock_text, [package for package in self.metadata["packages"] if package["id"] in selected])
        self.assertNotIn("star-game", filtered)
        output = self.base / "context"
        files, _digest = copy_context(self.repo, output, self.metadata)
        manifest = (output / "Cargo.toml").read_text(encoding="utf-8")
        api_manifest = (output / "crates" / "star-api-rest" / "Cargo.toml").read_text(encoding="utf-8")
        dockerfile = (output / "deploy" / "k3s-local" / "Dockerfile").read_text(encoding="utf-8")
        self.assertEqual(tomllib.loads(manifest)["workspace"]["members"], ["crates/domain-project", "crates/domain-tenant", "crates/star-api-rest"])
        self.assertIn("COPY crates/domain-project/ crates/domain-project/", dockerfile)
        self.assertIn("COPY crates/domain-tenant/ crates/domain-tenant/", dockerfile)
        self.assertNotIn("COPY crates ./crates", dockerfile)
        self.assertFalse(any("game" in path.casefold() or "legacy" in path.casefold() for path in files))
        self.assertFalse((output / "crates" / "star-game").exists())
        self.assertNotIn("star-game", api_manifest)

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
        with self.assertRaisesRegex(ContextError, "Symlink is not permitted"):
            copy_context(self.repo, self.base / "symlink-context", self.metadata)

    def test_output_is_stable(self) -> None:
        first = self.base / "context-a"
        second = self.base / "context-b"
        copy_context(self.repo, first, self.metadata)
        copy_context(self.repo, second, self.metadata)
        self.assertEqual(_tree_digest(first), _tree_digest(second))

    def test_build_invokes_multipass_not_host_docker(self) -> None:
        context = self.base / "build-context"
        _files, digest = copy_context(self.repo, context, self.metadata)
        with patch("scripts.automation.star_k3s_context.subprocess.run") as run:
            run.return_value = subprocess.CompletedProcess([], 0, stdout=b"", stderr=b"")
            build_in_multipass(context, "star-test", "w7-test", digest)
        commands = [call.args[0] for call in run.call_args_list]
        self.assertTrue(any(command[:3] == ["multipass", "exec", "star-test"] for command in commands))
        docker_build = next(command for command in commands if "docker" in command)
        self.assertEqual(docker_build[docker_build.index("docker") + 1], "build")
        self.assertIn("deploy/k3s-local/Dockerfile", docker_build)
        self.assertNotIn("docker", [command[0] for command in commands])

    def test_build_rejects_context_tampering(self) -> None:
        context = self.base / "tampered-context"
        _files, digest = copy_context(self.repo, context, self.metadata)
        (context / "crates" / "star-game").mkdir()
        with patch("scripts.automation.star_k3s_context.subprocess.run") as run:
            with self.assertRaisesRegex(ContextError, "(?i)game/legacy input"):
                build_in_multipass(context, "star-test", "w7-test", digest)
        run.assert_not_called()

    def test_build_rejects_unlisted_context_changes(self) -> None:
        context = self.base / "changed-context"
        _files, digest = copy_context(self.repo, context, self.metadata)
        (context / "README.private").write_text("unexpected input\n", encoding="utf-8")
        with patch("scripts.automation.star_k3s_context.subprocess.run") as run:
            with self.assertRaisesRegex(ContextError, "unexpected root entries|no longer matches"):
                build_in_multipass(context, "star-test", "w7-test", digest)
        run.assert_not_called()


if __name__ == "__main__":
    unittest.main()
