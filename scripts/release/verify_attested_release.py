#!/usr/bin/env python3
"""Verify a downloaded Star release artifact against GitHub attestations."""

# @cypher schema=1 source_sha256=e0e5e3787b2227fec1744f604bf5b9abb93503812c665d44af9b1fc49e81f5d9
# MERGE (self:File {path:"scripts/release/verify_attested_release.py"})
# MERGE (error:Symbol {id:"scripts/release/verify_attested_release.py::VerificationError",kind:"class"})
# MERGE (validate_identity:Symbol {id:"scripts/release/verify_attested_release.py::_validate_release_identity",kind:"function"})
# MERGE (validate_subject:Symbol {id:"scripts/release/verify_attested_release.py::_prepare_subject",kind:"function"})
# MERGE (hash_file:Symbol {id:"scripts/release/verify_attested_release.py::_hash_file",kind:"function"})
# MERGE (parse_result:Symbol {id:"scripts/release/verify_attested_release.py::_validate_attestation_result",kind:"function"})
# MERGE (verify:Symbol {id:"scripts/release/verify_attested_release.py::verify_attested_release",kind:"function"})
# MERGE (build_parser:Symbol {id:"scripts/release/verify_attested_release.py::build_parser",kind:"function"})
# MERGE (main:Symbol {id:"scripts/release/verify_attested_release.py::main",kind:"function"})
# MERGE (workflow_map:Config {id:"approved-signer-workflows"})
# MERGE (timeout:Config {id:"gh-attestation-timeout-seconds"})
# MERGE (output_limit:Config {id:"gh-attestation-json-output-limit-bytes"})
# MERGE (predicate:Config {id:"slsa-provenance-v1-predicate"})
# MERGE (gh:ExternalService {id:"gh-attestation-verify"})
# MERGE (self)-[:DEFINES]->(error)
# MERGE (self)-[:DEFINES]->(validate_identity)
# MERGE (self)-[:DEFINES]->(validate_subject)
# MERGE (self)-[:DEFINES]->(hash_file)
# MERGE (self)-[:DEFINES]->(parse_result)
# MERGE (self)-[:DEFINES]->(verify)
# MERGE (self)-[:DEFINES]->(build_parser)
# MERGE (self)-[:DEFINES]->(main)
# MERGE (validate_identity)-[:READS]->(workflow_map)
# MERGE (validate_subject)-[:CALLS]->(hash_file)
# MERGE (hash_file)-[:READS]->(self)
# MERGE (parse_result)-[:READS]->(output_limit)
# MERGE (verify)-[:CALLS]->(validate_identity)
# MERGE (verify)-[:CALLS]->(validate_subject)
# MERGE (verify)-[:CALLS]->(parse_result)
# MERGE (verify)-[:CALLS]->(gh)
# MERGE (verify)-[:READS]->(timeout)
# MERGE (verify)-[:READS]->(predicate)
# MERGE (main)-[:CALLS]->(build_parser)
# MERGE (main)-[:CALLS]->(verify)
# @endcypher

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path


TAG_PATTERN = re.compile(r"v(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\Z")
REPOSITORY_PATTERN = re.compile(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+\Z")
COMMIT_PATTERN = re.compile(r"(?:[0-9a-f]{40}|[0-9a-f]{64})\Z")
DIGEST_PATTERN = re.compile(r"sha256:([0-9a-f]{64})\Z")
OCI_SUBJECT_PATTERN = re.compile(
    r"oci://(?P<registry>[a-z0-9][a-z0-9.-]*(?::[0-9]{1,5})?)/"
    r"(?P<repository>[a-z0-9][a-z0-9._/-]*)@sha256:(?P<digest>[0-9a-f]{64})\Z"
)
PREDICATE_TYPE = "https://slsa.dev/provenance/v1"
APPROVED_SIGNER_WORKFLOWS = {
    "desktop": ".github/workflows/star-stable-desktop.yml",
    "web": ".github/workflows/star-stable-web.yml",
    "canvas": ".github/workflows/publish-star-canvas-services.yml",
}
CANVAS_IMAGES = {"star-canvas-engine", "star-domain-canvas", "star-canvas-realtime"}
MAX_TIMEOUT_SECONDS = 600
MAX_RESULT_BYTES = 4 * 1024 * 1024


class VerificationError(Exception):
    """An artifact could not be verified against the required release identity."""


def _validate_release_identity(
    kind: str, repository: str, tag: str, source_commit: str, timeout: int
) -> tuple[str, str]:
    """Validate the release identity and return its canonical signer workflow."""
    if kind not in APPROVED_SIGNER_WORKFLOWS:
        raise VerificationError("artifact kind must be desktop, web, or canvas")
    if REPOSITORY_PATTERN.fullmatch(repository) is None:
        raise VerificationError("repository must have the form owner/name")
    if TAG_PATTERN.fullmatch(tag) is None:
        raise VerificationError("tag must be a canonical stable vMAJOR.MINOR.PATCH tag")
    if COMMIT_PATTERN.fullmatch(source_commit) is None:
        raise VerificationError("source commit must be a full lowercase 40- or 64-character Git object ID")
    if isinstance(timeout, bool) or not isinstance(timeout, int) or not 1 <= timeout <= MAX_TIMEOUT_SECONDS:
        raise VerificationError(f"timeout must be between 1 and {MAX_TIMEOUT_SECONDS} seconds")

    signer_workflow = f"github.com/{repository}/{APPROVED_SIGNER_WORKFLOWS[kind]}"
    return signer_workflow, f"refs/tags/{tag}"


def _hash_file(path: Path) -> str:
    """Hash a regular local artifact in bounded memory, rejecting symlink subjects."""
    try:
        if path.is_symlink():
            raise VerificationError("file subject must not be a symbolic link")
        resolved = path.resolve(strict=True)
        if not resolved.is_file():
            raise VerificationError("file subject must be an existing regular file")
        digest = hashlib.sha256()
        with resolved.open("rb") as artifact:
            for chunk in iter(lambda: artifact.read(1024 * 1024), b""):
                digest.update(chunk)
    except VerificationError:
        raise
    except OSError as error:
        raise VerificationError("could not read the local artifact file") from error
    return f"sha256:{digest.hexdigest()}"


def _prepare_subject(
    kind: str, subject: str, expected_digest: str, repository: str, tag: str, source_commit: str
) -> tuple[str, str, str]:
    """Validate the local-file or immutable Canvas OCI subject and its digest."""
    expected_match = DIGEST_PATTERN.fullmatch(expected_digest)
    if expected_match is None:
        raise VerificationError("expected digest must use sha256:<64 lowercase hex characters>")

    if kind == "canvas":
        match = OCI_SUBJECT_PATTERN.fullmatch(subject)
        if match is None:
            raise VerificationError("Canvas subject must be an immutable oci:// reference pinned by @sha256")
        owner = repository.split("/", maxsplit=1)[0].lower()
        image_path = match.group("repository")
        if match.group("registry") != "ghcr.io" or "/" not in image_path:
            raise VerificationError("Canvas subject must identify an approved Star GHCR image")
        image_owner, image_name = image_path.split("/", maxsplit=1)
        if image_owner != owner or image_name not in CANVAS_IMAGES:
            raise VerificationError("Canvas subject must identify an approved Star GHCR image")
        if match.group("digest") != expected_match.group(1):
            raise VerificationError("immutable OCI reference digest does not match the declared digest")
        safe_subject = subject
        actual_digest = f"sha256:{match.group('digest')}"
    else:
        file_path = Path(subject)
        actual_digest = _hash_file(file_path)
        if actual_digest != expected_digest:
            raise VerificationError("local artifact SHA-256 does not match the declared digest")
        if kind == "web" and not file_path.name.endswith(".tar.gz"):
            raise VerificationError("Web subject must be the stable .tar.gz evidence archive")
        safe_subject = file_path.name

    if kind == "web":
        version = ".".join(TAG_PATTERN.fullmatch(tag).groups())  # identity validation ran first
        expected_name = f"star-web-build-evidence-{tag}-{version}-{source_commit}.tar.gz"
        if safe_subject != expected_name:
            raise VerificationError("Web archive filename does not match the selected tag and source commit")

    return subject, actual_digest, safe_subject


def _validate_attestation_result(stdout: str) -> int:
    """Require a non-empty gh JSON result with the documented attestation records."""
    try:
        output_size = len(stdout.encode("utf-8"))
    except UnicodeError as error:
        raise VerificationError("gh attestation verify returned malformed JSON text") from error
    if output_size > MAX_RESULT_BYTES:
        raise VerificationError("gh attestation JSON result exceeds the configured size limit")
    try:
        result = json.loads(stdout)
    except (json.JSONDecodeError, UnicodeError) as error:
        raise VerificationError("gh attestation verify returned malformed JSON") from error
    if not isinstance(result, list) or not result:
        raise VerificationError("gh attestation verify returned no verified attestations")
    for record in result:
        if (
            not isinstance(record, dict)
            or not isinstance(record.get("attestation"), dict)
            or not isinstance(record.get("verificationResult"), dict)
        ):
            raise VerificationError("gh attestation JSON did not contain verified attestation records")
    return len(result)


def verify_attested_release(
    *,
    kind: str,
    repository: str,
    tag: str,
    source_commit: str,
    subject: str,
    expected_digest: str,
    timeout: int = 120,
    gh_executable: str = "gh",
) -> dict[str, str | int]:
    """Verify one selected release file or Canvas image using GitHub CLI constraints."""
    signer_workflow, source_ref = _validate_release_identity(
        kind, repository, tag, source_commit, timeout
    )
    if not gh_executable or "\x00" in gh_executable:
        raise VerificationError("gh executable must be a non-empty executable name or path")
    gh_subject, actual_digest, safe_subject = _prepare_subject(
        kind, subject, expected_digest, repository, tag, source_commit
    )
    command = [
        gh_executable,
        "attestation",
        "verify",
        gh_subject,
        "--repo",
        repository,
        "--source-digest",
        source_commit,
        "--source-ref",
        source_ref,
        "--signer-workflow",
        signer_workflow,
        "--predicate-type",
        PREDICATE_TYPE,
        "--format",
        "json",
    ]
    try:
        result = subprocess.run(
            command,
            check=False,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            text=True,
            encoding="utf-8",
            errors="replace",
            shell=False,
            timeout=timeout,
        )
    except subprocess.TimeoutExpired as error:
        raise VerificationError("gh attestation verify exceeded its timeout") from error
    except OSError as error:
        raise VerificationError("could not start GitHub CLI; install gh and confirm it is on PATH") from error

    if result.returncode != 0:
        raise VerificationError(
            f"gh attestation verify failed with exit status {result.returncode}; stderr was discarded"
        )
    if not isinstance(result.stdout, str):
        raise VerificationError("gh attestation verify returned no JSON output")
    record_count = _validate_attestation_result(result.stdout)
    return {
        "kind": kind,
        "subject": safe_subject,
        "digest": actual_digest,
        "repository": repository,
        "tag": tag,
        "source_commit": source_commit,
        "signer_workflow": signer_workflow,
        "attestation_records": record_count,
    }


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Verify a Star release artifact against repository, tag, source, signer, and SLSA constraints."
    )
    parser.add_argument("--kind", required=True, choices=sorted(APPROVED_SIGNER_WORKFLOWS))
    parser.add_argument("--repo", required=True, help="Star GitHub repository in owner/name form")
    parser.add_argument("--tag", required=True, help="canonical stable release tag, for example v1.0.6")
    parser.add_argument("--source-commit", required=True, help="full lowercase 40- or 64-character source commit ID")
    parser.add_argument("--subject", required=True, help="local file path or immutable Canvas oci:// reference")
    parser.add_argument("--expected-digest", required=True, help="expected digest in sha256:<64 lowercase hex> form")
    parser.add_argument("--timeout", type=int, default=120, help=f"gh timeout in seconds (1-{MAX_TIMEOUT_SECONDS})")
    parser.add_argument("--gh", default="gh", help="GitHub CLI executable name or path")
    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    try:
        result = verify_attested_release(
            kind=args.kind,
            repository=args.repo,
            tag=args.tag,
            source_commit=args.source_commit,
            subject=args.subject,
            expected_digest=args.expected_digest,
            timeout=args.timeout,
            gh_executable=args.gh,
        )
    except VerificationError as error:
        print(f"error: {error}", file=sys.stderr)
        return 1
    print(json.dumps(result, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
