#!/usr/bin/env python3
"""Resolve a stable Star release version from an exact Git tag."""

# @cypher schema=1 source_sha256=a350eec137baf112e6672ba09de418fbcf16d12431093606cd9e0635aa2100ca
# MERGE (self:File {path:"scripts/release/resolve_star_version.py"})
# MERGE (pattern:Config {id:"canonical-stable-tag-pattern"})
# MERGE (output:Config {id:"github-output-file"})
# MERGE (validate:Symbol {id:"scripts/release/resolve_star_version.py::validate_tag",kind:"function"})
# MERGE (run_git:Symbol {id:"scripts/release/resolve_star_version.py::_run_git",kind:"function"})
# MERGE (resolve:Symbol {id:"scripts/release/resolve_star_version.py::resolve_release",kind:"function"})
# MERGE (append_output:Symbol {id:"scripts/release/resolve_star_version.py::_append_github_output",kind:"function"})
# MERGE (build_parser:Symbol {id:"scripts/release/resolve_star_version.py::build_parser",kind:"function"})
# MERGE (main:Symbol {id:"scripts/release/resolve_star_version.py::main",kind:"function"})
# MERGE (git_cli:ExternalService {id:"git-cli"})
# MERGE (self)-[:DEFINES]->(validate)
# MERGE (self)-[:DEFINES]->(run_git)
# MERGE (self)-[:DEFINES]->(resolve)
# MERGE (self)-[:DEFINES]->(append_output)
# MERGE (self)-[:DEFINES]->(build_parser)
# MERGE (self)-[:DEFINES]->(main)
# MERGE (validate)-[:READS]->(pattern)
# MERGE (run_git)-[:CALLS]->(git_cli)
# MERGE (resolve)-[:CALLS]->(validate)
# MERGE (resolve)-[:CALLS]->(run_git)
# MERGE (append_output)-[:WRITES]->(output)
# MERGE (main)-[:CALLS]->(build_parser)
# MERGE (main)-[:CALLS]->(resolve)
# MERGE (main)-[:CALLS]->(append_output)
# @endcypher

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path


TAG_PATTERN = re.compile(r"v(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\Z")
OBJECT_ID_PATTERN = re.compile(r"(?:[0-9a-f]{40}|[0-9a-f]{64})\Z")


class ResolverError(Exception):
    """A release tag cannot be resolved safely from the current checkout."""


def validate_tag(tag: str) -> str:
    """Return the normalized version for a canonical stable tag."""
    match = TAG_PATTERN.fullmatch(tag)
    if match is None:
        raise ResolverError(
            f"invalid release tag {tag!r}; expected canonical vMAJOR.MINOR.PATCH "
            "with no leading zeroes, prerelease, or build metadata"
        )
    return ".".join(match.groups())


def _run_git(repository: Path, *arguments: str) -> subprocess.CompletedProcess[str]:
    """Run Git with an argument list and return its captured result."""
    try:
        return subprocess.run(
            ["git", "-C", str(repository), *arguments],
            check=False,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            shell=False,
        )
    except OSError as error:
        raise ResolverError(
            "could not start Git; install Git and confirm it is available on PATH"
        ) from error


def resolve_release(repository: Path, tag: str) -> dict[str, str]:
    """Resolve the exact stable tag and require it to identify checked-out HEAD."""
    version = validate_tag(tag)
    ref = f"refs/tags/{tag}"

    ref_result = _run_git(repository, "show-ref", "--verify", "--quiet", "--", ref)
    if ref_result.returncode == 1:
        raise ResolverError(
            f"tag {tag!r} does not exist as an exact tag ref in {repository}"
        )
    if ref_result.returncode != 0:
        raise ResolverError(
            f"could not verify tag refs in {repository}; confirm --repo is a Git checkout"
        )

    peeled_result = _run_git(
        repository,
        "rev-parse",
        "--verify",
        "--end-of-options",
        f"{ref}^{{}}",
    )
    if peeled_result.returncode != 0:
        raise ResolverError(
            f"tag {tag!r} exists but its target could not be resolved; check the tag object"
        )
    source_sha = peeled_result.stdout.strip()
    if OBJECT_ID_PATTERN.fullmatch(source_sha) is None:
        raise ResolverError(f"Git returned an invalid peeled object id for tag {tag!r}")

    object_type_result = _run_git(repository, "cat-file", "-t", source_sha)
    if object_type_result.returncode != 0:
        raise ResolverError(f"could not inspect the peeled target for tag {tag!r}")
    object_type = object_type_result.stdout.strip()
    if object_type != "commit":
        raise ResolverError(
            f"tag {tag!r} peels to a {object_type or 'non-commit object'}, not a commit"
        )

    head_result = _run_git(
        repository, "rev-parse", "--verify", "--end-of-options", "HEAD^{commit}"
    )
    if head_result.returncode != 0:
        raise ResolverError(f"could not resolve HEAD in {repository} to a commit")
    head_sha = head_result.stdout.strip()
    if head_sha != source_sha:
        raise ResolverError(
            f"tag {tag!r} resolves to {source_sha}, but checked-out HEAD is {head_sha}; "
            "check out the tagged commit before resolving the release"
        )

    return {"tag": tag, "version": version, "source_sha": source_sha}


def _append_github_output(output_path: Path, release: dict[str, str]) -> None:
    """Append validated release values to an explicitly requested output file."""
    with output_path.open("a", encoding="utf-8", newline="\n") as output_file:
        output_file.write(f"STAR_RELEASE_TAG={release['tag']}\n")
        output_file.write(f"STAR_RELEASE_VERSION={release['version']}\n")
        output_file.write(f"STAR_RELEASE_SOURCE_SHA={release['source_sha']}\n")


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Resolve a stable Star release version from an exact Git tag."
    )
    parser.add_argument("--tag", required=True, help="canonical stable tag, for example v1.0.6")
    parser.add_argument(
        "--repo",
        type=Path,
        default=Path.cwd(),
        help="Git checkout to validate (defaults to the current directory)",
    )
    parser.add_argument(
        "--output",
        type=Path,
        help="append STAR_RELEASE_* values to this GitHub Actions output file after validation",
    )
    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    repository = args.repo.resolve()
    if not repository.is_dir():
        print(f"error: repository path is not a directory: {repository}", file=sys.stderr)
        return 1

    try:
        release = resolve_release(repository, args.tag)
        if args.output is not None:
            _append_github_output(args.output, release)
    except ResolverError as error:
        print(f"error: {error}", file=sys.stderr)
        return 1
    except OSError as error:
        print(f"error: could not append GitHub output file: {error}", file=sys.stderr)
        return 1

    print(json.dumps(release, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
