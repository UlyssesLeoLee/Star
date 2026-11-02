#!/usr/bin/env python3
"""Fail-closed game/legacy payload filters for Star Canvas image contexts.

The Canvas publish workflow allowlists three service images and assembles one
minimal Cargo context per image. This module owns the single deterministic rule
that keeps retired game and legacy payload names out of that context: every
generated source path and every resolved Cargo package identity is rejected
case-insensitively as soon as it contains a forbidden term.

The rule is intentionally narrow and fail closed. It never widens an allowlist,
never normalizes a rejected value into an accepted one, and reports every
violation it finds in sorted order so a failed gate is reproducible.
"""

# @cypher schema=1 source_sha256=3efc645ceabf3b0cc59264a97e2d293188c08055949193a96004ccfbdbdaa261
# MERGE (self:File {path:"scripts/release/star_canvas_context_policy.py"})
# MERGE (terms:Config {id:"scripts/release/star_canvas_context_policy.py::FORBIDDEN_TERMS"})
# MERGE (prefixes:Config {id:"scripts/release/star_canvas_context_policy.py::FORBIDDEN_PATH_PREFIXES"})
# MERGE (error:Type {id:"scripts/release/star_canvas_context_policy.py::CanvasContextError"})
# MERGE (matches:Symbol {id:"scripts/release/star_canvas_context_policy.py::forbidden_matches",kind:"function"})
# MERGE (image:Symbol {id:"scripts/release/star_canvas_context_policy.py::ensure_allowed_image",kind:"function"})
# MERGE (names:Symbol {id:"scripts/release/star_canvas_context_policy.py::ensure_clean_package_names",kind:"function"})
# MERGE (identities:Symbol {id:"scripts/release/star_canvas_context_policy.py::ensure_clean_package_identities",kind:"function"})
# MERGE (paths:Symbol {id:"scripts/release/star_canvas_context_policy.py::ensure_clean_context_paths",kind:"function"})
# MERGE (self)-[:DEFINES]->(terms)
# MERGE (self)-[:DEFINES]->(prefixes)
# MERGE (self)-[:DEFINES]->(error)
# MERGE (self)-[:DEFINES]->(matches)
# MERGE (self)-[:DEFINES]->(image)
# MERGE (self)-[:DEFINES]->(names)
# MERGE (self)-[:DEFINES]->(identities)
# MERGE (self)-[:DEFINES]->(paths)
# MERGE (matches)-[:READS]->(terms)
# MERGE (image)-[:CALLS]->(matches)
# MERGE (names)-[:CALLS]->(matches)
# MERGE (identities)-[:CALLS]->(matches)
# MERGE (paths)-[:CALLS]->(matches)
# MERGE (paths)-[:READS]->(prefixes)
# @endcypher

from __future__ import annotations

from collections.abc import Iterable, Sequence
from typing import Any


FORBIDDEN_TERMS: tuple[str, ...] = ("game", "legacy")
FORBIDDEN_PATH_PREFIXES: tuple[str, ...] = (".git/", "target/")


class CanvasContextError(RuntimeError):
    """Raised when a build input cannot be proven free of forbidden payloads."""


def _sort_key(value: Any) -> tuple[str, ...]:
    """Return a total order for identities whose source component may be None."""
    if isinstance(value, str):
        return (value,)
    if isinstance(value, (tuple, list)):
        return tuple("" if part is None else str(part) for part in value)
    return (repr(value),)


def _describe(value: Any) -> str:
    """Render a value for a rejection message without raising on odd inputs."""
    if isinstance(value, str):
        return repr(value)
    if isinstance(value, (tuple, list)):
        return repr(tuple("" if part is None else str(part) for part in value))
    return repr(value)


def forbidden_matches(value: Any) -> tuple[str, ...]:
    """Return every forbidden term contained in a name, path, or identity part.

    Matching is case-insensitive over the whole value, which subsumes matching
    each path segment, so a term is caught wherever it appears. ``None`` is an
    absent optional component (a workspace-local Cargo package has no source) and
    matches nothing. Every other non-string value, and any empty string, is
    reported as a violation because its payload cannot be proven.
    """
    if value is None:
        return ()
    if isinstance(value, str):
        if not value.strip():
            return ("<empty-identity>",)
        folded = value.casefold()
        return tuple(sorted(term for term in FORBIDDEN_TERMS if term in folded))
    if isinstance(value, (tuple, list)):
        return tuple(sorted({term for part in value for term in forbidden_matches(part)}))
    return ("<unprintable-identity>",)


def ensure_allowed_image(image: Any, allowed: Iterable[str]) -> str:
    """Require an exactly allowlisted image name that carries no forbidden term."""
    approved = frozenset(allowed)
    if not isinstance(image, str) or image not in approved:
        raise CanvasContextError(
            f"image {image!r} is outside the approved Canvas allowlist {sorted(approved)}"
        )
    matches = forbidden_matches(image)
    if matches:
        raise CanvasContextError(
            f"allowlisted image {image!r} contains forbidden terms: {list(matches)}"
        )
    return image


def ensure_clean_package_names(names: Iterable[Any]) -> tuple[str, ...]:
    """Reject any Cargo package name containing a forbidden game or legacy term."""
    rejected = []
    for name in names:
        if not isinstance(name, str):
            rejected.append(f"{_describe(name)} (<invalid-package-name>)")
            continue
        matches = forbidden_matches(name)
        if matches:
            rejected.append(f"{_describe(name)} ({', '.join(matches)})")
    if rejected:
        raise CanvasContextError(
            f"Cargo package names contain forbidden game/legacy terms: {sorted(rejected)}"
        )
    return tuple(sorted(names, key=_sort_key))


def ensure_clean_package_identities(identities: Iterable[Any]) -> tuple[tuple[str, ...], ...]:
    """Reject any resolved Cargo package identity carrying a forbidden term.

    Each identity is checked across all of its components (name, version, and
    source when present) so a forbidden term cannot enter the closure through a
    renamed package, a vendored path, or a non-crates.io source reference.
    """
    rejected = []
    normalized = []
    for identity in identities:
        if isinstance(identity, str):
            components: Sequence[Any] = (identity,)
        elif isinstance(identity, Sequence):
            components = identity
        else:
            components = (identity,)
        matches = sorted({term for part in components for term in forbidden_matches(part)})
        if matches:
            rejected.append(f"{_describe(components)} ({', '.join(matches)})")
        normalized.append(tuple("" if part is None else str(part) for part in components))
    if rejected:
        raise CanvasContextError(
            f"Cargo package identities contain forbidden game/legacy terms: {sorted(rejected)}"
        )
    return tuple(sorted(normalized, key=_sort_key))


def ensure_clean_context_paths(paths: Iterable[Any]) -> tuple[str, ...]:
    """Reject forbidden terms and VCS/build output in generated context paths."""
    rejected = []
    for path in paths:
        if not isinstance(path, str) or not path.strip():
            rejected.append(f"{_describe(path)} (<invalid-path>)")
            continue
        matches = forbidden_matches(path)
        if matches:
            rejected.append(f"{_describe(path)} ({', '.join(matches)})")
            continue
        lowered = path.casefold().lstrip("/")
        prefixes = [
            prefix
            for prefix in FORBIDDEN_PATH_PREFIXES
            if lowered == prefix.rstrip("/") or lowered.startswith(prefix)
        ]
        if prefixes:
            rejected.append(f"{_describe(path)} ({', '.join(sorted(prefixes))})")
    if rejected:
        raise CanvasContextError(
            f"generated image context paths are not Star-only: {sorted(rejected)}"
        )
    return tuple(sorted(paths, key=_sort_key))
