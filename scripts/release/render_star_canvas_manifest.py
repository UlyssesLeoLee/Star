#!/usr/bin/env python3
"""Render the active Canvas K3s template from resolver and digest records."""

# @cypher schema=1 source_sha256=8b3bc34be7bce38577916a4fb70cd0b0b1bbab82fc32f188e9a897434dc1088a
# MERGE (self:File {path:"scripts/release/render_star_canvas_manifest.py"})
# MERGE (error:Type {id:"scripts/release/render_star_canvas_manifest.py::RenderError"})
# MERGE (release:Symbol {id:"scripts/release/render_star_canvas_manifest.py::validate_release",kind:"function"})
# MERGE (record:Symbol {id:"scripts/release/render_star_canvas_manifest.py::validate_digest_records",kind:"function"})
# MERGE (render:Symbol {id:"scripts/release/render_star_canvas_manifest.py::render_manifest",kind:"function"})
# MERGE (read_json:Symbol {id:"scripts/release/render_star_canvas_manifest.py::_read_json",kind:"function"})
# MERGE (parser:Symbol {id:"scripts/release/render_star_canvas_manifest.py::build_parser",kind:"function"})
# MERGE (main:Symbol {id:"scripts/release/render_star_canvas_manifest.py::main",kind:"function"})
# MERGE (version_pattern:Config {id:"canonical-stable-tag-pattern"})
# MERGE (digest_pattern:Config {id:"immutable-oci-digest-pattern"})
# MERGE (template:Config {id:"star-canvas-k3s-release-template"})
# MERGE (self)-[:DEFINES]->(error)
# MERGE (self)-[:DEFINES]->(release)
# MERGE (self)-[:DEFINES]->(record)
# MERGE (self)-[:DEFINES]->(render)
# MERGE (self)-[:DEFINES]->(read_json)
# MERGE (self)-[:DEFINES]->(parser)
# MERGE (self)-[:DEFINES]->(main)
# MERGE (release)-[:READS]->(version_pattern)
# MERGE (record)-[:READS]->(digest_pattern)
# MERGE (render)-[:READS]->(template)
# MERGE (main)-[:CALLS]->(parser)
# MERGE (main)-[:CALLS]->(read_json)
# MERGE (main)-[:CALLS]->(release)
# MERGE (main)-[:CALLS]->(record)
# MERGE (main)-[:CALLS]->(render)
# @endcypher

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any


SERVICES = {
    "canvas-engine": "ghcr.io/ulyssesleolee/star-canvas-engine",
    "domain-canvas": "ghcr.io/ulyssesleolee/star-domain-canvas",
    "canvas-realtime": "ghcr.io/ulyssesleolee/star-canvas-realtime",
}
TAG_PATTERN = re.compile(r"v(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\Z")
OBJECT_ID_PATTERN = re.compile(r"(?:[0-9a-f]{40}|[0-9a-f]{64})\Z")
DIGEST_PATTERN = re.compile(r"sha256:([0-9a-f]{64})\Z")
VERSION_MARKER = "__STAR_RELEASE_TAG_REQUIRED__"
DIGEST_MARKERS = {
    "canvas-engine": "__CANVAS_ENGINE_DIGEST_REQUIRED__",
    "domain-canvas": "__DOMAIN_CANVAS_DIGEST_REQUIRED__",
    "canvas-realtime": "__CANVAS_REALTIME_DIGEST_REQUIRED__",
}
PLACEHOLDER_PATTERN = re.compile(r"__[A-Za-z0-9_]+__")
VERSION_LABEL_PATTERN = re.compile(r"(?m)^[ \t]*version:[ \t]*(\S+)[ \t]*$")


class RenderError(Exception):
    """A release input or Canvas deployment template is unsafe to render."""


def validate_release(value: Any) -> dict[str, str]:
    """Validate the exact JSON record emitted by the stable-tag resolver."""
    if not isinstance(value, dict) or set(value) != {"tag", "version", "source_sha"}:
        raise RenderError("resolver output must contain exactly tag, version, and source_sha")
    tag = value["tag"]
    version = value["version"]
    source_sha = value["source_sha"]
    if not isinstance(tag, str) or TAG_PATTERN.fullmatch(tag) is None:
        raise RenderError("resolver tag must be canonical vMAJOR.MINOR.PATCH")
    expected_version = tag[1:]
    if not isinstance(version, str) or version != expected_version:
        raise RenderError("resolver version must be the normalized version from its tag")
    if not isinstance(source_sha, str) or OBJECT_ID_PATTERN.fullmatch(source_sha) is None:
        raise RenderError("resolver source_sha must be a lowercase Git object id")
    return {"tag": tag, "version": version, "source_sha": source_sha}


def validate_digest_records(
    values: list[Any], release: dict[str, str]
) -> dict[str, str]:
    """Validate one immutable Buildx evidence record for each Canvas image."""
    if len(values) != len(SERVICES):
        raise RenderError("exactly three per-service image digest records are required")
    image_refs: dict[str, str] = {}
    expected_keys = {
        "service",
        "tag",
        "version",
        "source_sha",
        "version_ref",
        "source_ref",
        "image_ref",
        "digest",
    }
    for value in values:
        if not isinstance(value, dict) or set(value) != expected_keys:
            raise RenderError("each digest record must contain the complete release and image identity")
        service = value["service"]
        if not isinstance(service, str) or service not in SERVICES or service in image_refs:
            raise RenderError("digest records must identify each allowlisted service exactly once")
        if any(value[key] != release[key] for key in ("tag", "version", "source_sha")):
            raise RenderError(f"{service} digest record does not match the resolver output")

        repository = SERVICES[service]
        expected_version_ref = f"{repository}:{release['version']}"
        expected_source_ref = f"{repository}:{release['source_sha']}"
        digest = value["digest"]
        if not isinstance(digest, str) or DIGEST_PATTERN.fullmatch(digest) is None:
            raise RenderError(f"{service} must use an immutable sha256 digest")
        if value["version_ref"] != expected_version_ref:
            raise RenderError(f"{service} OCI version tag is not derived from the resolver output")
        if value["source_ref"] != expected_source_ref:
            raise RenderError(f"{service} OCI source tag is not derived from the resolver output")
        expected_image_ref = f"{repository}@{digest}"
        if value["image_ref"] != expected_image_ref:
            raise RenderError(f"{service} image reference must pin its exact repository digest")
        image_refs[service] = expected_image_ref

    if set(image_refs) != set(SERVICES):
        raise RenderError("digest records are missing one or more Canvas services")
    return image_refs


def render_manifest(
    template: str, release_value: Any, digest_values: list[Any]
) -> str:
    """Render a deterministic, versioned manifest with immutable image refs."""
    release = validate_release(release_value)
    image_refs = validate_digest_records(digest_values, release)
    version_labels = VERSION_LABEL_PATTERN.findall(template)
    if len(version_labels) != 6 or any(value != VERSION_MARKER for value in version_labels):
        raise RenderError("all six deployment version labels must use the release tag placeholder")

    rendered = template.replace(VERSION_MARKER, release["tag"])
    for service, marker in DIGEST_MARKERS.items():
        repository = SERVICES[service]
        template_image = f"image: {repository}@sha256:{marker}"
        if template.count(template_image) != 1:
            raise RenderError(f"template must contain exactly one pinned image marker for {service}")
        digest = image_refs[service].partition("@sha256:")[2]
        rendered = rendered.replace(f"sha256:{marker}", f"sha256:{digest}")

    if PLACEHOLDER_PATTERN.search(rendered):
        raise RenderError("rendered manifest still contains an unresolved release placeholder")
    if not rendered.endswith("\n"):
        rendered += "\n"
    for service, image_ref in image_refs.items():
        if rendered.count(f"image: {image_ref}") != 1:
            raise RenderError(f"rendered manifest does not carry exactly one digest pin for {service}")
    if rendered.count(f"version: {release['tag']}") != 6:
        raise RenderError("rendered deployment metadata does not match the resolver tag")
    return rendered


def _read_json(path: Path, description: str) -> Any:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise RenderError(f"could not read {description}: {error}") from error


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Render the active Star Canvas K3s manifest from stable release evidence."
    )
    parser.add_argument("--release-json", required=True, type=Path)
    parser.add_argument("--digest-record", required=True, action="append", type=Path)
    parser.add_argument("--template", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    try:
        input_paths = [args.release_json, args.template, *args.digest_record]
        output_path = args.output.resolve()
        if output_path in {path.resolve() for path in input_paths}:
            raise RenderError("output must not overwrite resolver, digest, or template input files")
        release_value = _read_json(args.release_json, "resolver output")
        digest_values = [
            _read_json(path, f"digest record {path}") for path in args.digest_record
        ]
        template = args.template.read_text(encoding="utf-8")
        result = render_manifest(template, release_value, digest_values)
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(result, encoding="utf-8", newline="\n")
    except (OSError, RenderError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 1
    print(f"Rendered {args.output} for {release_value['tag']} with three immutable Canvas image digests.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
