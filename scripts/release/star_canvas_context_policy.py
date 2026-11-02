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

# @cypher schema=1 source_sha256=44a4f33957e0d953164a258b7fb49cf4731fb81ff15adaeb4890af6895b39ec4
# MERGE (self:File {path:"scripts/release/star_canvas_context_policy.py"})
# MERGE (terms:Config {id:"scripts/release/star_canvas_context_policy.py::FORBIDDEN_TERMS"})
# MERGE (prefixes:Config {id:"scripts/release/star_canvas_context_policy.py::FORBIDDEN_PATH_PREFIXES"})
# MERGE (error:Type {id:"scripts/release/star_canvas_context_policy.py::CanvasContextError"})
# MERGE (matches:Symbol {id:"scripts/release/star_canvas_context_policy.py::forbidden_matches",kind:"function"})
# MERGE (image:Symbol {id:"scripts/release/star_canvas_context_policy.py::ensure_allowed_image",kind:"function"})
# MERGE (names:Symbol {id:"scripts/release/star_canvas_context_policy.py::ensure_clean_package_names",kind:"function"})
# MERGE (identities:Symbol {id:"scripts/release/star_canvas_context_policy.py::ensure_clean_package_identities",kind:"function"})
# MERGE (paths:Symbol {id:"scripts/release/star_canvas_context_policy.py::ensure_clean_context_paths",kind:"function"})
# MERGE (linkkind:Symbol {id:"scripts/release/star_canvas_context_policy.py::_link_kind",kind:"function"})
# MERGE (chain:Symbol {id:"scripts/release/star_canvas_context_policy.py::_descendant_chain",kind:"function"})
# MERGE (violations:Symbol {id:"scripts/release/star_canvas_context_policy.py::find_link_violations",kind:"function"})
# MERGE (linkfree:Symbol {id:"scripts/release/star_canvas_context_policy.py::ensure_link_free_sources",kind:"function"})
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
# MERGE (self)-[:DEFINES]->(linkkind)
# MERGE (self)-[:DEFINES]->(chain)
# MERGE (self)-[:DEFINES]->(violations)
# MERGE (self)-[:DEFINES]->(linkfree)
# MERGE (violations)-[:CALLS]->(linkkind)
# MERGE (violations)-[:CALLS]->(chain)
# MERGE (linkfree)-[:CALLS]->(violations)
# @endcypher

from __future__ import annotations

import os
from collections.abc import Iterable, Sequence
from pathlib import Path
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


def _realpath(path: Any) -> str:
    """Return a normalized, case-folded real path so containment checks are stable."""
    try:
        return os.path.normcase(os.path.realpath(os.fspath(path)))
    except TypeError:
        return os.path.normcase(os.path.realpath(str(path)))


def _within(child: Any, parent: Any) -> bool:
    """Report whether a resolved child path stays inside a resolved parent path."""
    resolved_child = _realpath(child)
    resolved_parent = _realpath(parent)
    if resolved_child == resolved_parent:
        return True
    return Path(resolved_child).is_relative_to(Path(resolved_parent))


def _link_kind(path: Path) -> str | None:
    """Classify a filesystem link, or return None for an ordinary path.

    Windows directory junctions are reparse points that ``is_symlink()`` reports
    as ordinary directories while ``shutil.copytree`` still follows them, so they
    are detected separately. Anything that cannot be classified fails closed.
    """
    try:
        if path.is_symlink():
            return "symlink"
        is_junction = getattr(Path, "is_junction", None)
        if is_junction is not None and is_junction(path):
            return "junction"
    except OSError:
        return "unresolved-link"
    return None


def _descendant_chain(source: Path, allowed_root: Path) -> tuple[Path, ...]:
    """Return every lexical path component between allowed_root and source.

    The chain is built from the *unresolved* path on purpose: resolving first
    would collapse the very link that has to be inspected. ``allowed_root`` itself
    is excluded so a runner whose workspace sits behind a system symlink is not
    rejected; containment of the resolved paths is checked separately.
    """
    try:
        relative = source.relative_to(allowed_root)
    except ValueError:
        return ()
    chain = []
    current = allowed_root
    for part in relative.parts:
        current = current / part
        chain.append(current)
    return tuple(chain)


def find_link_violations(root: Any, *, allowed_root: Any) -> tuple[str, ...]:
    """Return every reason a source path is unsafe to copy into the image context.

    A path is unsafe when it, or any lexical ancestor between the allowed root and
    it, is a symlink or junction, or when its resolved real path escapes the
    allowed root. Checking ancestors matters: ``repo/crates`` linked to
    ``repo/legacy-source`` still resolves inside the checkout and exposes ordinary
    children to ``os.walk``, so retired payload could be copied under an accepted
    ``crates/<image>`` name. Directory entries that are links or that escape are
    pruned rather than descended into, so a link aimed at a large tree cannot turn
    the gate into an unbounded scan.
    """
    source = Path(os.fspath(root)) if not isinstance(root, Path) else root
    allowed = Path(os.fspath(allowed_root)) if not isinstance(allowed_root, Path) else allowed_root
    label = source.name or str(source)
    if not source.exists():
        return (f"{label} (missing)",)

    linked = []
    for ancestor in _descendant_chain(source, allowed):
        ancestor_kind = _link_kind(ancestor)
        if ancestor_kind:
            linked.append(f"{ancestor.relative_to(allowed).as_posix()} ({ancestor_kind})")
    if linked:
        return tuple(sorted(linked))

    allowed_real = _realpath(allowed_root)
    source_real = _realpath(source)
    if not Path(source_real).is_relative_to(Path(allowed_real)):
        return (f"{label} (escapes the checkout at {allowed_root})",)

    if not source.is_dir():
        return ()

    violations = []
    for directory, dirnames, filenames in os.walk(source, followlinks=False):
        current = Path(directory)
        for name in list(dirnames):
            entry = current / name
            relative = entry.relative_to(source).as_posix()
            entry_kind = _link_kind(entry)
            if entry_kind:
                violations.append(f"{relative} ({entry_kind})")
                dirnames.remove(name)
                continue
            if not _within(entry, source_real):
                violations.append(f"{relative} (escapes the source crate)")
                dirnames.remove(name)
        for name in filenames:
            entry = current / name
            relative = entry.relative_to(source).as_posix()
            entry_kind = _link_kind(entry)
            if entry_kind:
                violations.append(f"{relative} ({entry_kind})")
            elif not _within(entry, source_real):
                violations.append(f"{relative} (escapes the source crate)")
    return tuple(sorted(violations))


def ensure_link_free_sources(paths: Iterable[Any], *, allowed_root: Any) -> tuple[str, ...]:
    """Require every copied source path to be an ordinary file inside the checkout.

    ``shutil.copytree`` and ``shutil.copy2`` follow links by default, so a
    benign-named link inside a Canvas crate can copy retired game or legacy
    content into the generated context under a name that every other gate
    accepts. This check runs before any copy and rejects the link itself rather
    than relying on the copied path's name.
    """
    violations = []
    checked = []
    for path in paths:
        found = find_link_violations(path, allowed_root=allowed_root)
        if found:
            violations.extend(found)
        else:
            checked.append(_realpath(path))
    if violations:
        raise CanvasContextError(
            f"image context sources are not link-free: {sorted(set(violations))}"
        )
    return tuple(sorted(set(checked)))
