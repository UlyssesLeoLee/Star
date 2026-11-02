#!/usr/bin/env python3
# -*- coding: utf-8 -*-
# @cypher schema=1 source_sha256=12fe51e60543871c0b0b25e0ce0a5d929d5c8b993d6b8c5b38dea19f62525496
# MERGE (self:File {path:"scripts/automation/star_k3s_context.py"})
# MERGE (running_instance:Symbol {id:"scripts/automation/star_k3s_context.py::_require_running_instance", kind:"function"})
# MERGE (guest_command:Symbol {id:"scripts/automation/star_k3s_context.py::_guest_command", kind:"function"})
# MERGE (guest_prerequisites:Symbol {id:"scripts/automation/star_k3s_context.py::_require_guest_prerequisites", kind:"function"})
# MERGE (deployment_manifest:Symbol {id:"scripts/automation/star_k3s_context.py::_render_deployment_manifest", kind:"function"})
# MERGE (image_import:Symbol {id:"scripts/automation/star_k3s_context.py::_stream_image_to_k3s", kind:"function"})
# MERGE (deploy:Symbol {id:"scripts/automation/star_k3s_context.py::deploy_in_multipass", kind:"function"})
# MERGE (forbidden_check:Symbol {id:"scripts/automation/star_k3s_context.py::_is_forbidden", kind:"function"})
# MERGE (identity:Symbol {id:"scripts/automation/star_k3s_context.py::_identity", kind:"function"})
# MERGE (relative:Symbol {id:"scripts/automation/star_k3s_context.py::_repo_relative", kind:"function"})
# MERGE (repo_file:Symbol {id:"scripts/automation/star_k3s_context.py::_repo_file", kind:"function"})
# MERGE (dependency_entries:Symbol {id:"scripts/automation/star_k3s_context.py::_dependency_entries", kind:"function"})
# MERGE (local_dependency:Symbol {id:"scripts/automation/star_k3s_context.py::_local_dependency_manifest_path", kind:"function"})
# MERGE (validate_dependencies:Symbol {id:"scripts/automation/star_k3s_context.py::_validate_local_dependencies", kind:"function"})
# MERGE (toml_key:Symbol {id:"scripts/automation/star_k3s_context.py::_toml_key", kind:"function"})
# MERGE (toml_value:Symbol {id:"scripts/automation/star_k3s_context.py::_toml_value", kind:"function"})
# MERGE (render_table:Symbol {id:"scripts/automation/star_k3s_context.py::_render_table", kind:"function"})
# MERGE (render_nested:Symbol {id:"scripts/automation/star_k3s_context.py::_render_nested_tables", kind:"function"})
# MERGE (strip_dev:Symbol {id:"scripts/automation/star_k3s_context.py::_without_dev_dependencies", kind:"function"})
# MERGE (load:Symbol {id:"scripts/automation/star_k3s_context.py::load_metadata", kind:"function"})
# MERGE (select:Symbol {id:"scripts/automation/star_k3s_context.py::select_packages", kind:"function"})
# MERGE (parse_lock:Symbol {id:"scripts/automation/star_k3s_context.py::_parse_lockfile", kind:"function"})
# MERGE (regenerate_lock:Symbol {id:"scripts/automation/star_k3s_context.py::_regenerate_context_lock", kind:"function"})
# MERGE (manifest:Symbol {id:"scripts/automation/star_k3s_context.py::filtered_workspace_manifest", kind:"function"})
# MERGE (copy_package:Symbol {id:"scripts/automation/star_k3s_context.py::_copy_local_package", kind:"function"})
# MERGE (validate_context:Symbol {id:"scripts/automation/star_k3s_context.py::_validate_context", kind:"function"})
# MERGE (copy:Symbol {id:"scripts/automation/star_k3s_context.py::copy_context", kind:"function"})
# MERGE (context_digest:Symbol {id:"scripts/automation/star_k3s_context.py::_context_digest", kind:"function"})
# MERGE (main:Symbol {id:"scripts/automation/star_k3s_context.py::main", kind:"function"})
# MERGE (self)-[:DEFINES]->(running_instance)
# MERGE (self)-[:DEFINES]->(guest_command)
# MERGE (self)-[:DEFINES]->(guest_prerequisites)
# MERGE (self)-[:DEFINES]->(deployment_manifest)
# MERGE (self)-[:DEFINES]->(image_import)
# MERGE (self)-[:DEFINES]->(deploy)
# MERGE (service:Symbol {id:"scripts/automation/star_k3s_context.py::SERVICE_PACKAGE", kind:"constant"})
# MERGE (forbidden:Symbol {id:"scripts/automation/star_k3s_context.py::FORBIDDEN_TERMS", kind:"constant"})
# MERGE (self)-[:DEFINES]->(load)
# MERGE (self)-[:DEFINES]->(forbidden_check)
# MERGE (self)-[:DEFINES]->(identity)
# MERGE (self)-[:DEFINES]->(relative)
# MERGE (self)-[:DEFINES]->(repo_file)
# MERGE (self)-[:DEFINES]->(dependency_entries)
# MERGE (self)-[:DEFINES]->(local_dependency)
# MERGE (self)-[:DEFINES]->(validate_dependencies)
# MERGE (self)-[:DEFINES]->(toml_key)
# MERGE (self)-[:DEFINES]->(toml_value)
# MERGE (self)-[:DEFINES]->(render_table)
# MERGE (self)-[:DEFINES]->(render_nested)
# MERGE (self)-[:DEFINES]->(strip_dev)
# MERGE (self)-[:DEFINES]->(select)
# MERGE (self)-[:DEFINES]->(parse_lock)
# MERGE (self)-[:DEFINES]->(regenerate_lock)
# MERGE (self)-[:DEFINES]->(manifest)
# MERGE (self)-[:DEFINES]->(copy_package)
# MERGE (self)-[:DEFINES]->(validate_context)
# MERGE (self)-[:DEFINES]->(copy)
# MERGE (self)-[:DEFINES]->(context_digest)
# MERGE (self)-[:DEFINES]->(main)
# MERGE (running_instance)-[:READS]->(:ExternalService {name:"Multipass instance inventory"})
# MERGE (guest_command)-[:USES]->(:ExternalService {name:"Multipass exec"})
# MERGE (guest_prerequisites)-[:CALLS]->(guest_command)
# MERGE (guest_prerequisites)-[:USES]->(:ExternalService {name:"Guest Docker engine and K3s API"})
# MERGE (deployment_manifest)-[:READS]->(:Config {path:"deploy/k3s-local/star-api-rest-deploy.yaml"})
# MERGE (deployment_manifest)-[:USES]->(service)
# MERGE (image_import)-[:CALLS]->(guest_command)
# MERGE (deploy)-[:CALLS]->(running_instance)
# MERGE (deploy)-[:CALLS]->(guest_prerequisites)
# MERGE (deploy)-[:CALLS]->(deployment_manifest)
# MERGE (deploy)-[:CALLS]->(guest_command)
# MERGE (deploy)-[:USES]->(service)
# MERGE (deploy)-[:CALLS]->(image_import)
# MERGE (deploy)-[:CALLS]->(validate_context)
# MERGE (deploy)-[:CALLS]->(context_digest)
# MERGE (deploy)-[:USES]->(:ExternalService {name:"Multipass transfer"})
# MERGE (deploy)-[:USES]->(:ExternalService {name:"K3s kubectl"})
# MERGE (self)-[:DEFINES]->(service)
# MERGE (self)-[:DEFINES]->(forbidden)
# MERGE (forbidden_check)-[:USES]->(forbidden)
# MERGE (relative)-[:CALLS]->(forbidden_check)
# MERGE (repo_file)-[:CALLS]->(relative)
# MERGE (local_dependency)-[:CALLS]->(relative)
# MERGE (select)-[:USES]->(service)
# MERGE (select)-[:CALLS]->(relative)
# MERGE (select)-[:CALLS]->(forbidden_check)
# MERGE (regenerate_lock)-[:CALLS]->(parse_lock)
# MERGE (regenerate_lock)-[:CALLS]->(identity)
# MERGE (regenerate_lock)-[:CALLS]->(forbidden_check)
# MERGE (manifest)-[:USES]->(service)
# MERGE (manifest)-[:CALLS]->(relative)
# MERGE (manifest)-[:CALLS]->(forbidden_check)
# MERGE (manifest)-[:CALLS]->(dependency_entries)
# MERGE (manifest)-[:CALLS]->(local_dependency)
# MERGE (manifest)-[:CALLS]->(validate_dependencies)
# MERGE (manifest)-[:CALLS]->(toml_value)
# MERGE (manifest)-[:CALLS]->(render_table)
# MERGE (manifest)-[:CALLS]->(render_nested)
# MERGE (manifest)-[:CALLS]->(toml_key)
# MERGE (manifest)-[:CALLS]->(forbidden_check)
# MERGE (validate_dependencies)-[:CALLS]->(dependency_entries)
# MERGE (validate_dependencies)-[:CALLS]->(local_dependency)
# MERGE (validate_dependencies)-[:CALLS]->(relative)
# MERGE (toml_value)-[:CALLS]->(toml_key)
# MERGE (render_table)-[:CALLS]->(toml_key)
# MERGE (render_table)-[:CALLS]->(toml_value)
# MERGE (render_nested)-[:CALLS]->(render_table)
# MERGE (render_nested)-[:CALLS]->(render_nested)
# MERGE (copy_package)-[:CALLS]->(relative)
# MERGE (copy_package)-[:CALLS]->(forbidden_check)
# MERGE (copy_package)-[:CALLS]->(strip_dev)
# MERGE (validate_context)-[:CALLS]->(forbidden_check)
# MERGE (validate_context)-[:CALLS]->(relative)
# MERGE (copy)-[:CALLS]->(select)
# MERGE (copy)-[:CALLS]->(repo_file)
# MERGE (copy)-[:CALLS]->(manifest)
# MERGE (copy)-[:CALLS]->(copy_package)
# MERGE (copy)-[:CALLS]->(regenerate_lock)
# MERGE (copy)-[:CALLS]->(validate_context)
# MERGE (copy)-[:CALLS]->(relative)
# MERGE (copy)-[:CALLS]->(forbidden_check)
# MERGE (copy)-[:CALLS]->(toml_value)
# MERGE (copy)-[:CALLS]->(context_digest)
# MERGE (main)-[:CALLS]->(running_instance)
# MERGE (main)-[:CALLS]->(guest_prerequisites)
# MERGE (main)-[:CALLS]->(load)
# MERGE (main)-[:CALLS]->(copy)
# MERGE (main)-[:CALLS]->(deploy)
# @endcypher

"""Create a verified Star-only context and optionally deploy it through Multipass."""

from __future__ import annotations

import argparse
import csv
import hashlib
import io
import json
import os
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile
import tomllib
import uuid
from pathlib import Path, PurePosixPath
from typing import Any


SERVICE_PACKAGE = "star-api-rest"
FORBIDDEN_TERMS = ("game", "legacy")
_COPY_MARKERS = ("# @star-local-crates-begin", "# @star-local-crates-end")
_IGNORED_DIRS = {".git", "target", "node_modules"}
_DOCKERIGNORE = ".git\n**/.git\ntarget\n**/target\nnode_modules\n**/node_modules\n"


class ContextError(RuntimeError):
    """Raised when a build input cannot be proven to be inside the Star closure."""


def _is_forbidden(value: str) -> bool:
    """Reject game/legacy names in packages and repository-relative paths."""
    parts = re.split(r"[/\\]+", value.casefold())
    return any(term in part for part in parts for term in FORBIDDEN_TERMS)


def _identity(package: dict[str, Any]) -> tuple[str, str, str | None]:
    return (package["name"], package["version"], package.get("source"))


def _repo_relative(root: Path, value: str) -> Path:
    root_path = root.resolve()
    candidate = Path(value)
    if not candidate.is_absolute():
        candidate = root_path / candidate
    candidate = Path(os.path.abspath(candidate))
    try:
        relative = candidate.relative_to(root_path)
    except ValueError as error:
        raise ContextError(f"Path escapes repository root: {value}") from error
    current = root_path
    for part in relative.parts:
        current = current / part
        if current.is_symlink():
            raise ContextError(f"Symlink is not permitted in a repository input path: {relative}")
    path = candidate.resolve()
    try:
        relative = path.relative_to(root_path)
    except ValueError as error:
        raise ContextError(f"Path escapes repository root: {value}") from error
    if _is_forbidden(relative.as_posix()):
        raise ContextError(f"Game/legacy path is not permitted in a Star context: {relative}")
    return relative


def _repo_file(root: Path, relative: str) -> Path:
    """Return a required repository file after rejecting symlinks and escapes."""
    path = root / relative
    normalized = _repo_relative(root, str(path))
    source = root.resolve() / normalized
    if not source.is_file():
        raise ContextError(f"Required Star build input is missing: {relative}")
    return source


def load_metadata(repo_root: Path, target_triple: str) -> dict[str, Any]:
    """Resolve the release dependency graph without changing the checked-in lock."""
    command = [
        "cargo",
        "metadata",
        "--format-version",
        "1",
        "--locked",
        "--offline",
        "--manifest-path",
        "crates/star-api-rest/Cargo.toml",
    ]
    command.extend(["--filter-platform", target_triple])
    result = subprocess.run(command, cwd=repo_root, capture_output=True, check=False)
    if result.returncode != 0:
        detail = result.stderr.decode("utf-8", errors="replace")[-1200:]
        raise ContextError(f"Cargo metadata failed closed (exit {result.returncode}): {detail}")
    try:
        metadata = json.loads(result.stdout.decode("utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise ContextError("Cargo metadata returned invalid UTF-8 JSON") from error
    return metadata


def select_packages(metadata: dict[str, Any], repo_root: Path) -> tuple[set[str], list[dict[str, Any]]]:
    """Walk star-api-rest's active non-dev dependency edges and validate local packages."""
    packages = metadata.get("packages")
    resolve = metadata.get("resolve")
    if not isinstance(packages, list) or not isinstance(resolve, dict) or not isinstance(resolve.get("nodes"), list):
        raise ContextError("Cargo metadata is missing its resolved package graph")

    package_by_id = {package.get("id"): package for package in packages if isinstance(package, dict)}
    expected_manifest = (repo_root / "crates" / SERVICE_PACKAGE / "Cargo.toml").resolve()
    roots = [
        package
        for package in packages
        if package.get("name") == SERVICE_PACKAGE
        and Path(package.get("manifest_path", "")).resolve() == expected_manifest
    ]
    if len(roots) != 1:
        raise ContextError(f"Expected one {SERVICE_PACKAGE} package, found {len(roots)}")

    nodes = {node.get("id"): node for node in resolve["nodes"] if isinstance(node, dict)}
    pending = [roots[0]["id"]]
    selected: set[str] = set()
    while pending:
        package_id = pending.pop()
        if package_id in selected:
            continue
        node = nodes.get(package_id)
        if node is None:
            raise ContextError(f"Resolved package graph is missing node {package_id}")
        if package_id not in package_by_id:
            raise ContextError(f"Resolved package graph references unknown package {package_id}")
        selected.add(package_id)
        for dependency in node.get("deps", []):
            dep_kinds = dependency.get("dep_kinds")
            if not isinstance(dep_kinds, list) or not dep_kinds:
                raise ContextError(f"Dependency kind is missing for {package_id} -> {dependency.get('pkg')}")
            if any(kind.get("kind") in (None, "build") for kind in dep_kinds):
                pending.append(dependency.get("pkg"))

    selected_packages = [package_by_id[package_id] for package_id in selected]
    local_packages: list[dict[str, Any]] = []
    for package in selected_packages:
        if _is_forbidden(package.get("name", "")):
            raise ContextError(f"Forbidden package in selected Cargo closure: {package.get('name')}")
        if package.get("source") is not None:
            continue
        relative = _repo_relative(repo_root, package.get("manifest_path", ""))
        if relative.parts[:1] != ("crates",) or relative.name != "Cargo.toml":
            raise ContextError(f"Local package is outside crates/: {relative}")
        manifest_path = repo_root / relative
        if not manifest_path.is_file():
            raise ContextError(f"Local package manifest is missing: {relative}")
        local_packages.append(package)

    local_packages.sort(key=lambda package: _repo_relative(repo_root, package["manifest_path"]).as_posix())
    if not any(package["name"] == SERVICE_PACKAGE for package in local_packages):
        raise ContextError(f"The selected closure does not contain {SERVICE_PACKAGE}")
    return selected, local_packages


def _parse_lockfile(lock_text: str) -> tuple[str, list[tuple[dict[str, Any], str]]]:
    marker = re.compile(r"(?m)^\[\[package\]\]\s*$")
    matches = list(marker.finditer(lock_text))
    if not matches:
        raise ContextError("Cargo.lock contains no package records")
    prefix = lock_text[: matches[0].start()]
    blocks: list[tuple[dict[str, Any], str]] = []
    for index, match in enumerate(matches):
        end = matches[index + 1].start() if index + 1 < len(matches) else len(lock_text)
        block = lock_text[match.start() : end]
        try:
            package = tomllib.loads(block)["package"][0]
        except (KeyError, tomllib.TOMLDecodeError) as error:
            raise ContextError("Cargo.lock contains an invalid package record") from error
        blocks.append((package, block))
    return prefix, blocks


def _regenerate_context_lock(
    context: Path,
    source_lock_text: str,
    expected_local_packages: list[dict[str, Any]],
) -> None:
    """Ask Cargo to project the lockfile, then prove versions/checksums came from the source lock."""
    _prefix, source_blocks = _parse_lockfile(source_lock_text)
    source_by_identity: dict[tuple[str, str, str | None], dict[str, Any]] = {}
    for package, _block in source_blocks:
        identity = _identity(package)
        if identity in source_by_identity:
            raise ContextError(f"Source Cargo.lock contains a duplicate package identity: {identity}")
        source_by_identity[identity] = package

    expected_local = {_identity(package) for package in expected_local_packages}
    lock_path = context / "Cargo.lock"
    lock_path.write_text(source_lock_text, encoding="utf-8", newline="\n")
    result = subprocess.run(
        ["cargo", "metadata", "--format-version", "1", "--offline", "--manifest-path", "Cargo.toml"],
        cwd=context,
        capture_output=True,
        check=False,
    )
    if result.returncode != 0:
        detail = result.stderr.decode("utf-8", errors="replace")[-1600:]
        raise ContextError(f"Cargo could not regenerate the filtered lockfile offline: {detail}")

    try:
        _prefix, generated_blocks = _parse_lockfile(lock_path.read_text(encoding="utf-8"))
    except OSError as error:
        raise ContextError("Cargo did not produce a context Cargo.lock") from error
    generated_identities: set[tuple[str, str, str | None]] = set()
    for package, _block in generated_blocks:
        identity = _identity(package)
        if identity in generated_identities:
            raise ContextError(f"Generated Cargo.lock contains a duplicate package identity: {identity}")
        generated_identities.add(identity)
        if _is_forbidden(identity[0]):
            raise ContextError(f"Generated Cargo.lock contains a forbidden package: {identity[0]}")
        if identity[2] is None:
            if identity not in expected_local:
                raise ContextError(f"Generated Cargo.lock contains an unapproved local package: {identity}")
            continue
        source_package = source_by_identity.get(identity)
        if source_package is None:
            raise ContextError(f"Generated Cargo.lock introduced a package absent from the source lock: {identity}")
        if source_package.get("checksum") != package.get("checksum"):
            raise ContextError(f"Generated Cargo.lock checksum differs from the source lock: {identity}")


def _dependency_entries(manifest: dict[str, Any]) -> list[tuple[str, Any]]:
    """Return only normal and build dependency declarations, including target tables."""
    entries: list[tuple[str, Any]] = []
    for group in ("dependencies", "build-dependencies"):
        entries.extend(manifest.get(group, {}).items())
    for target in manifest.get("target", {}).values():
        if not isinstance(target, dict):
            raise ContextError("Cargo target dependency table is malformed")
        for group in ("dependencies", "build-dependencies"):
            entries.extend(target.get(group, {}).items())
    return entries


def _local_dependency_manifest_path(base: Path, dependency: dict[str, Any], repo_root: Path) -> Path:
    value = dependency.get("path")
    if not isinstance(value, str) or not value:
        raise ContextError("Cargo local path dependency has an invalid path")
    path = base / value
    manifest_path = path / "Cargo.toml" if path.is_dir() else path
    relative = _repo_relative(repo_root, str(manifest_path))
    resolved_manifest = repo_root.resolve() / relative
    if not resolved_manifest.is_file():
        raise ContextError(f"Local dependency manifest is missing: {manifest_path}")
    return resolved_manifest


def _validate_local_dependencies(
    local_packages: list[dict[str, Any]],
    selected_ids: set[str],
    package_by_id: dict[str, dict[str, Any]],
    root_manifest: dict[str, Any],
    repo_root: Path,
) -> tuple[set[str], list[dict[str, Any]]]:
    """Collect workspace aliases and local manifests Cargo must parse for the selected closure."""
    selected_manifests = {
        Path(package["manifest_path"]).resolve()
        for package_id, package in package_by_id.items()
        if package_id in selected_ids and package.get("source") is None
    }
    inherited_names: set[str] = set()
    supplemental_by_manifest: dict[Path, dict[str, Any]] = {}
    workspace_dependencies = root_manifest.get("workspace", {}).get("dependencies", {})
    pending = list(local_packages)
    visited_manifests: set[Path] = set()
    while pending:
        package = pending.pop()
        manifest_path = Path(package["manifest_path"]).resolve()
        if manifest_path in visited_manifests:
            continue
        visited_manifests.add(manifest_path)
        try:
            manifest = tomllib.loads(manifest_path.read_text(encoding="utf-8"))
        except (OSError, tomllib.TOMLDecodeError) as error:
            raise ContextError(f"Local package manifest is unreadable: {manifest_path}") from error
        for name, dependency in _dependency_entries(manifest):
            if not isinstance(dependency, dict):
                continue
            if dependency.get("workspace") is True:
                inherited_names.add(name)
                definition = workspace_dependencies.get(name)
                if definition is None:
                    raise ContextError(f"Workspace dependency is missing: {name}")
                dependency_definition = definition
            elif "path" in dependency:
                dependency_definition = dependency
            else:
                continue
            if not isinstance(dependency_definition, dict) or "path" not in dependency_definition:
                continue
            target_manifest = _local_dependency_manifest_path(
                repo_root if dependency.get("workspace") is True else manifest_path.parent,
                dependency_definition,
                repo_root,
            )
            if target_manifest in selected_manifests or target_manifest in visited_manifests:
                continue
            relative = _repo_relative(repo_root, str(target_manifest))
            if relative.parts[:1] != ("crates",) or relative.name != "Cargo.toml":
                raise ContextError(f"Manifest-required local package is outside crates/: {relative}")
            try:
                target_package = tomllib.loads(target_manifest.read_text(encoding="utf-8")).get("package", {})
            except (OSError, tomllib.TOMLDecodeError) as error:
                raise ContextError(f"Manifest-required local package is unreadable: {relative}") from error
            target_name = target_package.get("name")
            if not isinstance(target_name, str) or _is_forbidden(target_name):
                raise ContextError(f"Manifest-required local package has an invalid or forbidden name: {target_name}")
            target_version = target_package.get("version")
            if not isinstance(target_version, str) or not target_version:
                raise ContextError(f"Manifest-required local package has no version: {relative}")
            supplemental = {
                "id": f"manifest-only:{relative.as_posix()}",
                "name": target_name,
                "version": target_version,
                "manifest_path": str(target_manifest),
                "source": None,
            }
            supplemental_by_manifest[target_manifest] = supplemental
            pending.append(supplemental)
    supplemental_packages = sorted(
        supplemental_by_manifest.values(),
        key=lambda package: _repo_relative(repo_root, package["manifest_path"]).as_posix(),
    )
    return inherited_names, supplemental_packages


def _toml_key(value: str) -> str:
    return value if re.fullmatch(r"[A-Za-z0-9_-]+", value) else json.dumps(value, ensure_ascii=False)


def _toml_value(value: Any) -> str:
    if isinstance(value, str):
        return json.dumps(value, ensure_ascii=False)
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, int):
        return str(value)
    if isinstance(value, float):
        return repr(value)
    if isinstance(value, list):
        return "[" + ", ".join(_toml_value(item) for item in value) + "]"
    if isinstance(value, dict):
        return "{ " + ", ".join(f"{_toml_key(key)} = {_toml_value(item)}" for key, item in sorted(value.items())) + " }"
    raise ContextError(f"Unsupported TOML value type in build manifest: {type(value).__name__}")


def _render_table(name: str, values: dict[str, Any]) -> str:
    if not values:
        return ""
    rows = [f"[{name}]"]
    rows.extend(f"{_toml_key(key)} = {_toml_value(value)}" for key, value in sorted(values.items()))
    return "\n".join(rows) + "\n\n"


def _render_nested_tables(name: str, values: dict[str, Any]) -> str:
    scalar_values = {key: value for key, value in values.items() if not isinstance(value, dict)}
    output = _render_table(name, scalar_values)
    for key, value in sorted(values.items()):
        if isinstance(value, dict):
            output += _render_nested_tables(f"{name}.{_toml_key(key)}", value)
    return output


def _without_dev_dependencies(manifest_text: str) -> str:
    """Remove development-only Cargo dependency tables from the generated build context."""
    lines = manifest_text.splitlines(keepends=True)
    kept: list[str] = []
    omit = False
    for line in lines:
        header = re.match(r"^\s*\[([^\]]+)\]\s*(?:#.*)?$", line.rstrip("\r\n"))
        if header:
            section = header.group(1)
            omit = section == "dev-dependencies" or section.endswith(".dev-dependencies")
        if not omit:
            kept.append(line)
    return "".join(kept)


def filtered_workspace_manifest(
    root_manifest_text: str,
    local_packages: list[dict[str, Any]],
    selected_ids: set[str],
    package_by_id: dict[str, dict[str, Any]],
    repo_root: Path,
) -> tuple[str, list[dict[str, Any]]]:
    """Render only selected workspace members and inherited dependencies."""
    try:
        root_manifest = tomllib.loads(root_manifest_text)
    except tomllib.TOMLDecodeError as error:
        raise ContextError("Root Cargo.toml is invalid TOML") from error
    if set(root_manifest) - {"workspace", "profile"}:
        raise ContextError(f"Unsupported root Cargo.toml sections: {sorted(set(root_manifest) - {'workspace', 'profile'})}")
    workspace = root_manifest.get("workspace")
    if not isinstance(workspace, dict):
        raise ContextError("Root Cargo.toml has no workspace table")
    supported_workspace_keys = {"resolver", "members", "exclude", "package", "dependencies", "lints"}
    if set(workspace) - supported_workspace_keys:
        raise ContextError(f"Unsupported workspace settings: {sorted(set(workspace) - supported_workspace_keys)}")
    member_paths = sorted(
        {
            _repo_relative(repo_root, package["manifest_path"]).parent.as_posix()
            for package in local_packages
        }
    )
    inherited_names, supplemental_packages = _validate_local_dependencies(
        local_packages, selected_ids, package_by_id, root_manifest, repo_root
    )
    all_workspace_dependencies = workspace.get("dependencies", {})
    missing = inherited_names - all_workspace_dependencies.keys()
    if missing:
        raise ContextError(f"Workspace dependencies are missing: {sorted(missing)}")
    workspace_dependencies = {name: all_workspace_dependencies[name] for name in sorted(inherited_names)}
    if any(_is_forbidden(name) for name in workspace_dependencies):
        raise ContextError("Workspace dependency alias contains a forbidden game/legacy name")

    rows = ["[workspace]"]
    if "resolver" in workspace:
        rows.append(f"resolver = {_toml_value(workspace['resolver'])}")
    rows.append("members = [")
    rows.extend(f"    {_toml_value(path)}," for path in member_paths)
    rows.append("]")
    exclusion_set = {path for path in workspace.get("exclude", []) if not _is_forbidden(path)}
    for package in supplemental_packages:
        path = _repo_relative(repo_root, package["manifest_path"]).parent.as_posix()
        if path not in member_paths:
            exclusion_set.add(path)
    exclusions = sorted(exclusion_set)
    if exclusions:
        rows.append(f"exclude = {_toml_value(exclusions)}")
    output = "\n".join(rows) + "\n\n"
    output += _render_table("workspace.package", workspace.get("package", {}))
    output += _render_table("workspace.dependencies", workspace_dependencies)
    if workspace.get("lints"):
        output += _render_nested_tables("workspace.lints", workspace["lints"])
    for profile_name, settings in sorted(root_manifest.get("profile", {}).items()):
        if not isinstance(settings, dict):
            raise ContextError(f"Profile settings must be tables: {profile_name}")
        output += _render_nested_tables(f"profile.{_toml_key(profile_name)}", settings)
    try:
        parsed = tomllib.loads(output)
    except tomllib.TOMLDecodeError as error:
        raise ContextError("Filtered Cargo.toml is invalid TOML") from error
    if parsed.get("workspace", {}).get("members") != member_paths:
        raise ContextError("Filtered Cargo.toml members do not match the selected local packages")
    if any(_is_forbidden(path) for path in parsed["workspace"]["members"]):
        raise ContextError("Filtered Cargo.toml retained a game/legacy workspace path")
    return output, supplemental_packages


def _copy_local_package(repo_root: Path, package: dict[str, Any], output: Path) -> None:
    relative_manifest = _repo_relative(repo_root, package["manifest_path"])
    source_root = (repo_root / relative_manifest).parent
    destination = output / source_root.relative_to(repo_root)
    if destination.exists():
        raise ContextError(f"Duplicate local package directory in selected closure: {source_root}")
    destination.mkdir(parents=True)
    for current, directory_names, file_names in os.walk(source_root, followlinks=False):
        current_path = Path(current)
        relative_dir = current_path.relative_to(source_root)
        if _is_forbidden(relative_dir.as_posix()) and relative_dir.parts:
            raise ContextError(f"Game/legacy source directory found: {relative_manifest.parent / relative_dir}")
        kept_dirs = []
        for name in sorted(directory_names):
            child = current_path / name
            if name in _IGNORED_DIRS:
                continue
            if child.is_symlink():
                raise ContextError(f"Symlink is not permitted in the build context: {child}")
            if _is_forbidden(name):
                raise ContextError(f"Game/legacy source directory found: {child}")
            kept_dirs.append(name)
        directory_names[:] = kept_dirs
        for name in sorted(file_names):
            source = current_path / name
            relative_file = source.relative_to(source_root)
            if _is_forbidden(relative_file.as_posix()):
                raise ContextError(f"Game/legacy source file found: {source}")
            if source.is_symlink():
                raise ContextError(f"Symlink is not permitted in the build context: {source}")
            target = destination / relative_file
            target.parent.mkdir(parents=True, exist_ok=True)
            if relative_file.as_posix() == "Cargo.toml":
                target.write_text(_without_dev_dependencies(source.read_text(encoding="utf-8")), encoding="utf-8", newline="\n")
            else:
                shutil.copy2(source, target)


def copy_context(
    repo_root: Path,
    output: Path,
    metadata: dict[str, Any],
) -> tuple[list[str], str]:
    """Materialize a deterministic source context limited to the resolved Star closure."""
    repo_root = repo_root.resolve()
    output = output.resolve()
    try:
        output.relative_to(repo_root)
    except ValueError:
        pass
    else:
        raise ContextError("Build context output must be outside the source repository")
    if output.exists():
        raise ContextError(f"Refusing to overwrite existing build-context path: {output}")

    selected_ids, local_packages = select_packages(metadata, repo_root)
    package_by_id = {package["id"]: package for package in metadata["packages"]}
    root_manifest_path = _repo_file(repo_root, "Cargo.toml")
    lock_path = _repo_file(repo_root, "Cargo.lock")
    rust_toolchain_path = _repo_file(repo_root, "rust-toolchain.toml")
    license_path = _repo_file(repo_root, "LICENSE")

    root_manifest, supplemental_packages = filtered_workspace_manifest(
        root_manifest_path.read_text(encoding="utf-8"),
        local_packages,
        selected_ids,
        package_by_id,
        repo_root,
    )
    lock_text = lock_path.read_text(encoding="utf-8")
    expected_local_packages = [
        package_by_id[package_id]
        for package_id in selected_ids
        if package_by_id[package_id].get("source") is None
    ] + supplemental_packages
    dockerfile_source = _repo_file(repo_root, "deploy/k3s-local/Dockerfile")
    dockerfile = dockerfile_source.read_text(encoding="utf-8")
    if dockerfile.count(_COPY_MARKERS[0]) != 1 or dockerfile.count(_COPY_MARKERS[1]) != 1:
        raise ContextError("K3s Dockerfile must contain one pair of local-crate copy markers")
    context_packages = local_packages + supplemental_packages
    copy_lines = "".join(
        f"COPY {path.as_posix()}/ {path.as_posix()}/\n"
        for path in sorted(
            {Path(_repo_relative(repo_root, package["manifest_path"]).parent) for package in context_packages},
            key=lambda path: path.as_posix(),
        )
    )
    dockerfile = dockerfile.replace(_COPY_MARKERS[0] + "\n" + _COPY_MARKERS[1], copy_lines.rstrip("\n"))
    if "COPY crates ./crates" in dockerfile or "star-game" in dockerfile.casefold():
        raise ContextError("Generated K3s Dockerfile still includes broad or forbidden workspace inputs")

    output.mkdir(parents=True)
    (output / "Cargo.toml").write_text(root_manifest, encoding="utf-8", newline="\n")
    (output / "Cargo.lock").write_text(lock_text, encoding="utf-8", newline="\n")
    shutil.copy2(rust_toolchain_path, output / "rust-toolchain.toml")
    shutil.copy2(license_path, output / "LICENSE")
    dockerfile_output = output / "deploy" / "k3s-local" / "Dockerfile"
    dockerfile_output.parent.mkdir(parents=True)
    dockerfile_output.write_text(dockerfile, encoding="utf-8", newline="\n")
    (output / ".dockerignore").write_text(_DOCKERIGNORE, encoding="utf-8", newline="\n")
    for package in context_packages:
        _copy_local_package(repo_root, package, output)

    _regenerate_context_lock(output, lock_text, expected_local_packages)
    files = _validate_context(output)
    digest = _context_digest(output, files)
    return files, digest


def _context_digest(context: Path, files: list[str]) -> str:
    """Hash a sorted file inventory without loading the whole build context into memory."""
    digest = hashlib.sha256()
    for relative in files:
        path = context / relative
        if path.is_symlink() or not path.is_file():
            raise ContextError(f"Build context file changed during verification: {relative}")
        digest.update(relative.encode("utf-8"))
        digest.update(b"\0")
        with path.open("rb") as source:
            for chunk in iter(lambda: source.read(1024 * 1024), b""):
                digest.update(chunk)
        digest.update(b"\0")
    return digest.hexdigest()


def _validate_context(context: Path) -> list[str]:
    """Revalidate generated files immediately before use or Multipass transfer."""
    if not context.is_dir():
        raise ContextError(f"Build context directory is missing: {context}")
    files: list[str] = []
    for path in sorted(context.rglob("*")):
        relative = path.relative_to(context).as_posix()
        if path.is_symlink():
            raise ContextError(f"Symlinks are not permitted in build context: {relative}")
        if _is_forbidden(relative):
            raise ContextError(f"Game/legacy input found in build context: {relative}")
        if path.is_file():
            files.append(relative)
    files.sort()
    required = {"Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "LICENSE", ".dockerignore", "deploy/k3s-local/Dockerfile"}
    if not required.issubset(files):
        raise ContextError(f"Build context is missing required files: {sorted(required - set(files))}")
    try:
        manifest = tomllib.loads((context / "Cargo.toml").read_text(encoding="utf-8"))
        lock = tomllib.loads((context / "Cargo.lock").read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError) as error:
        raise ContextError("Generated Cargo manifest or lockfile is invalid") from error
    members = manifest.get("workspace", {}).get("members")
    if not isinstance(members, list) or not members:
        raise ContextError("Generated workspace has no explicit member allowlist")
    expected_crates = set(members)
    excludes = manifest.get("workspace", {}).get("exclude", [])
    if not isinstance(excludes, list) or any(not isinstance(path, str) or _is_forbidden(path) for path in excludes):
        raise ContextError("Generated workspace has invalid or forbidden exclude paths")
    excluded_crates = {path for path in excludes if path.startswith("crates/")}
    allowed_crates = expected_crates | excluded_crates
    allowed_root_entries = required | {"crates", "deploy"}
    unexpected_root_entries = {path.name for path in context.iterdir()} - allowed_root_entries
    if unexpected_root_entries:
        raise ContextError(f"Build context contains unexpected root entries: {sorted(unexpected_root_entries)}")
    for relative in files:
        parts = Path(relative).parts
        if parts[0] == "crates":
            if len(parts) < 3 or Path(*parts[:2]).as_posix() not in allowed_crates:
                raise ContextError(f"Build context contains a file outside selected or manifest-required crates: {relative}")
        elif parts[0] == "deploy":
            if relative != "deploy/k3s-local/Dockerfile":
                raise ContextError(f"Build context contains an unexpected deployment input: {relative}")
        elif parts[0] not in required:
            raise ContextError(f"Build context contains an unexpected file: {relative}")
    for member in members:
        if not isinstance(member, str) or _is_forbidden(member) or not member.startswith("crates/") or ".." in Path(member).parts:
            raise ContextError(f"Generated workspace member is not an approved crate path: {member}")
        member_manifest = context / member / "Cargo.toml"
        if not member_manifest.is_file():
            raise ContextError(f"Generated workspace member manifest is missing: {member}")
        try:
            package = tomllib.loads(member_manifest.read_text(encoding="utf-8")).get("package", {})
        except (OSError, tomllib.TOMLDecodeError) as error:
            raise ContextError(f"Generated package manifest is invalid: {member}") from error
        if _is_forbidden(package.get("name", "")):
            raise ContextError(f"Generated workspace contains a forbidden package: {package.get('name')}")
    crate_manifests = {
        path.parent.relative_to(context).as_posix()
        for path in context.glob("crates/**/Cargo.toml")
        if path.is_file()
    }
    foreign_manifests = {
        path for path in crate_manifests
        if not any(path == member or path.startswith(f"{member}/") for member in allowed_crates)
    }
    if foreign_manifests:
        raise ContextError(f"Build context contains crate manifests outside the workspace allowlist: {sorted(foreign_manifests)}")
    if any(_is_forbidden(package.get("name", "")) for package in lock.get("package", [])):
        raise ContextError("Generated Cargo.lock includes a game/legacy package")
    inherited = manifest.get("workspace", {}).get("dependencies", {})
    for alias, definition in inherited.items():
        if _is_forbidden(alias) or (isinstance(definition, dict) and any(_is_forbidden(str(definition.get(key, ""))) for key in ("package", "path"))):
            raise ContextError(f"Generated workspace dependency is forbidden: {alias}")
    for member in members:
        member_manifest = tomllib.loads((context / member / "Cargo.toml").read_text(encoding="utf-8"))
        for name, definition in _dependency_entries(member_manifest):
            if _is_forbidden(name):
                raise ContextError(f"Generated package dependency is forbidden: {name}")
            if isinstance(definition, dict) and any(_is_forbidden(str(definition.get(key, ""))) for key in ("package", "path")):
                raise ContextError(f"Generated package dependency is forbidden: {name}")
    dockerfile = (context / "deploy" / "k3s-local" / "Dockerfile").read_text(encoding="utf-8")
    if any(marker in dockerfile for marker in _COPY_MARKERS) or re.search(r"(?m)^\s*COPY\s+crates/?\s+\./?crates/?\s*$", dockerfile):
        raise ContextError("Build Dockerfile is not expanded to an explicit crate allowlist")
    return files


def _guest_command(instance: str, *arguments: str) -> list[str]:
    """Build one fixed-argument command for an explicitly selected guest."""
    return ["multipass", "exec", instance, "--", *arguments]


def _require_running_instance(instance: str) -> None:
    """Fail closed unless Multipass inventory has exactly one matching Running instance."""
    if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._-]{0,63}", instance):
        raise ContextError("Multipass instance name has an invalid format")
    result = subprocess.run(
        ["multipass", "list", "--format", "csv"],
        check=True,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    reader = csv.DictReader(io.StringIO(result.stdout))
    if not reader.fieldnames or not {"Name", "State"}.issubset(reader.fieldnames):
        raise ContextError("Multipass CSV inventory is missing the Name or State column")
    matches = [row for row in reader if row.get("Name") == instance]
    if len(matches) != 1:
        raise ContextError(f"Multipass instance is missing or ambiguous: {instance}")
    if matches[0].get("State") != "Running":
        raise ContextError(f"Multipass instance is not Running: {instance}")


def _require_guest_prerequisites(instance: str) -> None:
    """Check existing guest build and K3s readiness without installing or starting tools."""
    docker = subprocess.run(
        _guest_command(instance, "docker", "info", "--format", "{{.ServerVersion}}"),
        check=False,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if docker.returncode != 0 or not docker.stdout.strip():
        raise ContextError("Selected Multipass guest has no ready Docker build engine; install/start nothing automatically")
    k3s = subprocess.run(
        _guest_command(
            instance,
            "sudo",
            "-n",
            "/usr/local/bin/k3s",
            "kubectl",
            "get",
            "nodes",
            "-o",
            "json",
            "--request-timeout=10s",
        ),
        check=False,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if k3s.returncode != 0:
        raise ContextError("Selected Multipass guest has no ready K3s API; the helper will not start or repair it")
    try:
        node_list = json.loads(k3s.stdout)
    except json.JSONDecodeError as error:
        raise ContextError("Selected Multipass guest returned invalid K3s node readiness data") from error
    nodes = node_list.get("items") if isinstance(node_list, dict) else None
    if not isinstance(nodes, list) or not nodes:
        raise ContextError("Selected Multipass guest has no K3s nodes")
    unready_nodes = []
    for node in nodes:
        metadata = node.get("metadata") if isinstance(node, dict) else None
        name = metadata.get("name", "<unknown>") if isinstance(metadata, dict) else "<unknown>"
        status = node.get("status") if isinstance(node, dict) else None
        conditions = status.get("conditions", []) if isinstance(status, dict) else []
        ready = any(
            isinstance(condition, dict)
            and condition.get("type") == "Ready"
            and condition.get("status") == "True"
            for condition in conditions
        )
        if not ready:
            unready_nodes.append(name)
    if unready_nodes:
        raise ContextError(f"Selected Multipass guest has K3s nodes that are not Ready: {', '.join(unready_nodes)}")


def _render_deployment_manifest(source: Path, destination: Path, image: str) -> None:
    """Pin the local deployment manifest to the exact image built for this run."""
    if source.is_symlink() or not source.is_file():
        raise ContextError(f"Deployment manifest is missing or is a symlink: {source}")
    original = source.read_text(encoding="utf-8")
    placeholder = f"image: {SERVICE_PACKAGE}:local"
    if original.count(placeholder) != 1:
        raise ContextError("Deployment manifest must contain exactly one Star local-image placeholder")
    destination.write_text(original.replace(placeholder, f"image: {image}"), encoding="utf-8", newline="\n")


def _stream_image_to_k3s(instance: str, image: str) -> None:
    """Stream image bytes between fixed guest commands without buffering in host memory."""
    exporter = subprocess.Popen(
        _guest_command(instance, "docker", "save", image),
        stdout=subprocess.PIPE,
    )
    if exporter.stdout is None:
        exporter.wait()
        raise ContextError("Multipass image export did not provide a stream")
    try:
        importer = subprocess.Popen(
            _guest_command(
                instance,
                "sudo",
                "-n",
                "/usr/local/bin/k3s",
                "ctr",
                "-n",
                "k8s.io",
                "images",
                "import",
                "-",
            ),
            stdin=exporter.stdout,
        )
    except BaseException:
        exporter.stdout.close()
        exporter.terminate()
        exporter.wait()
        raise
    exporter.stdout.close()
    import_status = importer.wait()
    export_status = exporter.wait()
    if export_status != 0 or import_status != 0:
        raise ContextError("Guest image export/import failed; the image was not accepted by K3s")


def deploy_in_multipass(context: Path, instance: str, image_tag: str, expected_digest: str, manifest_path: Path) -> None:
    """Transfer the filtered context, build/import the image, and deploy inside one existing guest."""
    files = _validate_context(context)
    if not re.fullmatch(r"[0-9a-f]{64}", expected_digest) or _context_digest(context, files) != expected_digest:
        raise ContextError("Build context no longer matches the digest produced during generation")
    _require_running_instance(instance)
    if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._-]{0,63}", image_tag):
        raise ContextError("Image tag must be a local tag token, without a registry or repository")
    _require_guest_prerequisites(instance)
    token = uuid.uuid4().hex
    guest_dir = f"/tmp/star-k3s-context-{token}"
    guest_archive = f"{guest_dir}.tar.gz"
    guest_manifest = f"{guest_dir}/star-api-rest-deploy.yaml"
    image = f"{SERVICE_PACKAGE}:{image_tag}"
    with tempfile.TemporaryDirectory(prefix="star-k3s-context-") as temporary:
        archive_path = Path(temporary) / "context.tar.gz"
        manifest_copy = Path(temporary) / "star-api-rest-deploy.yaml"
        _render_deployment_manifest(manifest_path, manifest_copy, image)
        with tarfile.open(archive_path, "w:gz") as archive:
            for relative in files:
                path = context / relative
                if path.is_symlink() or not path.is_file():
                    raise ContextError(f"Build context changed before transfer: {relative}")
                archive.add(path, arcname=relative, recursive=False)
        with tarfile.open(archive_path, "r:gz") as archive:
            members = sorted((member for member in archive.getmembers() if member.isfile()), key=lambda member: member.name)
            member_names = [member.name for member in members]
            if member_names != files:
                raise ContextError("Multipass archive inventory differs from the verified context")
            archive_digest = hashlib.sha256()
            for member in members:
                archive_digest.update(member.name.encode("utf-8"))
                archive_digest.update(b"\0")
                source = archive.extractfile(member)
                if source is None:
                    raise ContextError(f"Multipass archive member is unreadable: {member.name}")
                with source:
                    for chunk in iter(lambda: source.read(1024 * 1024), b""):
                        archive_digest.update(chunk)
                archive_digest.update(b"\0")
            if archive_digest.hexdigest() != expected_digest:
                raise ContextError("Multipass archive content differs from the verified context")
        try:
            _require_running_instance(instance)
            _require_guest_prerequisites(instance)
            subprocess.run(_guest_command(instance, "mkdir", "-p", guest_dir), check=True)
            subprocess.run(["multipass", "transfer", str(archive_path), f"{instance}:{guest_archive}"], check=True)
            subprocess.run(["multipass", "transfer", str(manifest_copy), f"{instance}:{guest_manifest}"], check=True)
            subprocess.run(_guest_command(instance, "tar", "-xzf", guest_archive, "-C", guest_dir), check=True)
            subprocess.run(
                _guest_command(
                    instance,
                    "docker",
                    "build",
                    "--tag",
                    image,
                    "--file",
                    f"{guest_dir}/deploy/k3s-local/Dockerfile",
                    guest_dir,
                ),
                check=True,
            )
            _stream_image_to_k3s(instance, image)
            runtime_images = subprocess.run(
                _guest_command(instance, "sudo", "-n", "/usr/local/bin/k3s", "crictl", "images"),
                check=True,
                capture_output=True,
                text=True,
                encoding="utf-8",
                errors="replace",
            ).stdout
            if not any(
                fields[:2] == [f"docker.io/library/{SERVICE_PACKAGE}", image_tag]
                for fields in (line.split() for line in runtime_images.splitlines())
            ):
                raise ContextError(f"K3s CRI cannot see {image}; deployment is stopped")
            subprocess.run(
                _guest_command(instance, "sudo", "-n", "/usr/local/bin/k3s", "kubectl", "apply", "-f", guest_manifest),
                check=True,
            )
            subprocess.run(
                _guest_command(
                    instance,
                    "sudo",
                    "-n",
                    "/usr/local/bin/k3s",
                    "kubectl",
                    "-n",
                    "star-system",
                    "rollout",
                    "status",
                    "deployment/star-api-rest",
                    "--timeout=120s",
                ),
                check=True,
            )
            subprocess.run(
                _guest_command(
                    instance,
                    "sudo",
                    "-n",
                    "/usr/local/bin/k3s",
                    "kubectl",
                    "-n",
                    "star-system",
                    "get",
                    "pods",
                    "-o",
                    "wide",
                    "-l",
                    "app=star-api-rest",
                ),
                check=True,
            )
        finally:
            try:
                cleanup = subprocess.run(
                    _guest_command(instance, "rm", "-rf", "--", guest_dir, guest_archive),
                    check=False,
                    capture_output=True,
                )
                if cleanup.returncode != 0:
                    detail = cleanup.stderr.decode("utf-8", errors="replace")[-500:]
                    print(f"warning: temporary Multipass context cleanup failed: {detail}", file=sys.stderr)
            except OSError as error:
                print(f"warning: temporary Multipass context cleanup could not run: {error}", file=sys.stderr)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, default=Path.cwd(), help="Star repository root")
    parser.add_argument("--target-triple", default="x86_64-unknown-linux-gnu")
    parser.add_argument("--output", type=Path, help="New output directory; must not already exist")
    parser.add_argument("--build-in-multipass", metavar="INSTANCE", help="Build and deploy in this existing, running Multipass guest")
    parser.add_argument("--image-tag", help="Local tag token required with --build-in-multipass")
    args = parser.parse_args(argv)
    try:
        if bool(args.build_in_multipass) != bool(args.image_tag):
            raise ContextError("--build-in-multipass and --image-tag must be supplied together")
        if args.build_in_multipass:
            if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._-]{0,63}", args.image_tag):
                raise ContextError("Image tag must be a local tag token, without a registry or repository")
            _require_running_instance(args.build_in_multipass)
            _require_guest_prerequisites(args.build_in_multipass)
        if args.output is None and args.build_in_multipass is None:
            raise ContextError("Provide --output to generate a context or --build-in-multipass to build locally in a guest")
        metadata = load_metadata(args.repo, args.target_triple)
        if args.output is not None:
            context = args.output.resolve()
            files, digest = copy_context(args.repo, context, metadata)
            print(f"Verified Star K3s context: {len(files)} files; sha256={digest}; no game/legacy inputs.")
            if args.build_in_multipass:
                deploy_in_multipass(
                    context,
                    args.build_in_multipass,
                    args.image_tag,
                    digest,
                    args.repo / "deploy" / "k3s-local" / "star-api-rest-deploy.yaml",
                )
        else:
            with tempfile.TemporaryDirectory(prefix="star-k3s-context-") as temporary:
                context = Path(temporary) / "context"
                files, digest = copy_context(args.repo, context, metadata)
                print(f"Verified Star K3s context: {len(files)} files; sha256={digest}; no game/legacy inputs.")
                deploy_in_multipass(
                    context,
                    args.build_in_multipass,
                    args.image_tag,
                    digest,
                    args.repo / "deploy" / "k3s-local" / "star-api-rest-deploy.yaml",
                )
        return 0
    except (ContextError, OSError, subprocess.SubprocessError, tomllib.TOMLDecodeError) as error:
        print(f"star-k3s-context: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
