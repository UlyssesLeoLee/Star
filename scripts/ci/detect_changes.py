#!/usr/bin/env python3
"""Decide which CI areas a change touches, so docs-only pushes skip heavy jobs.

Writes `rust`, `bff`, `frontend`, `desktop`, `docs`, `workflows` = true|false to
$GITHUB_OUTPUT (or stdout when run locally).

Fail-safe: anything that prevents an exact answer (no base commit, unreachable
base after a force push, git error, workflow_dispatch, schedule) turns every
area on. A skipped job is only ever the result of a positive "this change
cannot affect it" decision.

Usage (CI):   BASE_SHA=<sha> HEAD_SHA=<sha> EVENT_NAME=<event> detect_changes.py
Usage (test): detect_changes.py --files path1 path2 ...
"""

# @cypher schema=1
# MERGE (self:File {path:"scripts/ci/detect_changes.py"})
# MERGE (areas:Config {id:"scripts/ci/detect_changes.py::AREAS",name:"rust/bff/frontend/desktop/docs/workflows"})
# MERGE (shared:Config {id:"scripts/ci/detect_changes.py::_SHARED_RUST",name:"files that affect every Rust build"})
# MERGE (patterns:Config {id:"scripts/ci/detect_changes.py::PATTERNS",name:"fnmatch globs per area"})
# MERGE (classify:Symbol {id:"scripts/ci/detect_changes.py::classify",kind:"function"})
# MERGE (git_changed:Symbol {id:"scripts/ci/detect_changes.py::_git_changed_files",kind:"function"})
# MERGE (emit:Symbol {id:"scripts/ci/detect_changes.py::_emit",kind:"function"})
# MERGE (main:Symbol {id:"scripts/ci/detect_changes.py::main",kind:"function"})
# MERGE (git_cli:ExternalService {id:"git-cli"})
# MERGE (github_output:Config {id:"github-output-file"})
# MERGE (self)-[:DEFINES]->(classify)
# MERGE (self)-[:DEFINES]->(git_changed)
# MERGE (self)-[:DEFINES]->(emit)
# MERGE (self)-[:DEFINES]->(main)
# MERGE (patterns)-[:READS]->(shared)
# MERGE (classify)-[:READS]->(patterns)
# MERGE (emit)-[:READS]->(areas)
# MERGE (emit)-[:WRITES]->(github_output)
# MERGE (git_changed)-[:CALLS]->(git_cli)
# MERGE (main)-[:CALLS]->(classify)
# MERGE (main)-[:CALLS]->(git_changed)
# MERGE (main)-[:CALLS]->(emit)
# @endcypher

from __future__ import annotations

import argparse
import fnmatch
import os
import subprocess
import sys

AREAS = ("rust", "bff", "frontend", "desktop", "docs", "workflows")

# Files whose change can affect the CI definition itself or every Rust build.
_SHARED_RUST = (
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
    "deny.toml",
    ".github/workflows/ci.yml",
    "scripts/ci/*",
)

# Glob patterns (fnmatch; `*` also crosses `/`) per area.
PATTERNS: dict[str, tuple[str, ...]] = {
    "rust": _SHARED_RUST
    + (
        "crates/*",
        "tests/*",
        "bench/*",
        "db/*",
        "data/*",
        "tools/*",
        "bff/*",
    ),
    "bff": _SHARED_RUST + ("bff/*", "crates/*"),
    "frontend": (
        "frontend/*",
        ".github/workflows/ci.yml",
        "scripts/ci/*",
    ),
    "desktop": (
        "crates/star-desktop/*",
        "Cargo.toml",
        "Cargo.lock",
        "rust-toolchain.toml",
        ".github/workflows/ci.yml",
        "scripts/ci/*",
    ),
    "docs": (
        "*.md",
        ".markdownlint*",
        ".github/workflows/ci.yml",
        "scripts/ci/*",
    ),
    "workflows": (".github/*", "scripts/ci/*"),
}


def classify(files: list[str]) -> dict[str, bool]:
    """Return the per-area decision for a list of changed paths."""
    result = {area: False for area in AREAS}
    for raw in files:
        path = raw.strip().replace("\\", "/")
        if not path:
            continue
        for area, patterns in PATTERNS.items():
            if not result[area] and any(fnmatch.fnmatch(path, p) for p in patterns):
                result[area] = True
    return result


def _git_changed_files(base: str, head: str) -> list[str] | None:
    """Changed paths between two commits, or None when it cannot be determined."""
    try:
        # Shallow clones: make sure both commits' trees are present.
        for sha in (base, head):
            probe = subprocess.run(
                ["git", "cat-file", "-e", f"{sha}^{{commit}}"],
                capture_output=True,
                check=False,
            )
            if probe.returncode != 0:
                fetched = subprocess.run(
                    ["git", "fetch", "--no-tags", "--depth=1", "origin", sha],
                    capture_output=True,
                    check=False,
                )
                if fetched.returncode != 0:
                    return None
        diff = subprocess.run(
            ["git", "diff", "--name-only", "--no-renames", base, head],
            capture_output=True,
            text=True,
            encoding="utf-8",
            check=False,
        )
        if diff.returncode != 0:
            return None
        return [line for line in diff.stdout.splitlines() if line]
    except OSError:
        return None


def _emit(decision: dict[str, bool]) -> None:
    lines = [f"{area}={'true' if decision[area] else 'false'}" for area in AREAS]
    target = os.environ.get("GITHUB_OUTPUT")
    if target:
        with open(target, "a", encoding="utf-8") as handle:
            handle.write("\n".join(lines) + "\n")
    print("change detection:", ", ".join(lines))


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--files", nargs="*", help="classify these paths and exit")
    args = parser.parse_args(argv)

    if args.files is not None:
        _emit(classify(args.files))
        return 0

    event = os.environ.get("EVENT_NAME", "")
    base = os.environ.get("BASE_SHA", "")
    head = os.environ.get("HEAD_SHA", "")
    all_on = {area: True for area in AREAS}

    if event not in ("push", "pull_request"):
        print(f"event '{event}' -> run everything")
        _emit(all_on)
        return 0
    if not base or set(base) == {"0"} or not head:
        print("no usable base commit (new branch?) -> run everything")
        _emit(all_on)
        return 0

    files = _git_changed_files(base, head)
    if files is None:
        print("could not diff base..head (force push / shallow?) -> run everything")
        _emit(all_on)
        return 0
    print(f"{len(files)} changed file(s)")
    _emit(classify(files))
    return 0


if __name__ == "__main__":
    sys.exit(main())
