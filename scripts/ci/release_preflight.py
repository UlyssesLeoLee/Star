#!/usr/bin/env python3
"""Release preflight helpers used by .github/workflows/release.yml.

Two checks that are easy to get subtly wrong inline in YAML:

  * `check-runs`: read the GitHub `commits/<sha>/check-runs` payload on stdin
    and require every named check to have its newest run concluded `success`.
    A missing check is a failure (fail closed): a release must not be cut from
    a commit on which a required gate never ran.
  * `tag`: validate a stable release tag name before it is used in any ref.

Usage:
  release_preflight.py tag v1.2.3
  gh api repos/OWNER/REPO/commits/SHA/check-runs --paginate | release_preflight.py check-runs
"""

# @cypher schema=1
# MERGE (self:File {path:"scripts/ci/release_preflight.py"})
# MERGE (required:Config {id:"scripts/ci/release_preflight.py::REQUIRED_CHECKS",name:"CI gate (required), License (cargo-deny)"})
# MERGE (tag_pattern:Config {id:"scripts/ci/release_preflight.py::TAG_PATTERN",name:"canonical stable vMAJOR.MINOR.PATCH"})
# MERGE (validate_tag:Symbol {id:"scripts/ci/release_preflight.py::validate_tag",kind:"function"})
# MERGE (latest:Symbol {id:"scripts/ci/release_preflight.py::_latest_by_name",kind:"function"})
# MERGE (evaluate:Symbol {id:"scripts/ci/release_preflight.py::evaluate_check_runs",kind:"function"})
# MERGE (main:Symbol {id:"scripts/ci/release_preflight.py::main",kind:"function"})
# MERGE (resolver:File {path:"scripts/release/resolve_star_version.py"})
# MERGE (self)-[:DEFINES]->(validate_tag)
# MERGE (self)-[:DEFINES]->(latest)
# MERGE (self)-[:DEFINES]->(evaluate)
# MERGE (self)-[:DEFINES]->(main)
# MERGE (validate_tag)-[:READS]->(tag_pattern)
# MERGE (evaluate)-[:READS]->(required)
# MERGE (evaluate)-[:CALLS]->(latest)
# MERGE (main)-[:CALLS]->(validate_tag)
# MERGE (main)-[:CALLS]->(evaluate)
# MERGE (validate_tag)-[:CONFIGURES]->(resolver)
# @endcypher

from __future__ import annotations

import json
import re
import sys

# Same grammar as scripts/release/resolve_star_version.py (which remains the
# authority inside the build workflows); duplicated here only so the
# orchestrator can reject a bad tag before it is placed in a checkout ref.
TAG_PATTERN = re.compile(r"v(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\Z")

# Check-run names as GitHub reports them (the job `name:`, not the workflow name).
REQUIRED_CHECKS = ("CI gate (required)", "License (cargo-deny)")


def validate_tag(tag: str) -> bool:
    """True for a canonical stable tag such as v1.2.3."""
    return TAG_PATTERN.fullmatch(tag) is not None


def _latest_by_name(runs: list[dict]) -> dict[str, dict]:
    """Newest run per check name (highest id wins; ids grow monotonically)."""
    latest: dict[str, dict] = {}
    for run in runs:
        name = run.get("name")
        if not name:
            continue
        if name not in latest or run.get("id", 0) > latest[name].get("id", 0):
            latest[name] = run
    return latest


def evaluate_check_runs(
    payload: dict | list, required: tuple[str, ...] = REQUIRED_CHECKS
) -> tuple[bool, list[str]]:
    """Return (all required checks succeeded, report lines).

    Accepts either one `check-runs` page ({"check_runs": [...]}) or the list of
    pages that `gh api --paginate --slurp` produces.
    """
    pages = payload if isinstance(payload, list) else [payload]
    runs: list[dict] = []
    for page in pages:
        runs.extend((page or {}).get("check_runs", []))
    latest = _latest_by_name(runs)

    ok = True
    lines: list[str] = []
    for name in required:
        run = latest.get(name)
        if run is None:
            ok = False
            lines.append(f"FAIL {name}: no such check run on this commit")
            continue
        conclusion = run.get("conclusion")
        good = conclusion == "success"
        ok = ok and good
        lines.append(f"{'ok  ' if good else 'FAIL'} {name}: {conclusion or run.get('status')}")
    return ok, lines


def main(argv: list[str]) -> int:
    if len(argv) >= 2 and argv[0] == "tag":
        if validate_tag(argv[1]):
            print(f"ok   tag {argv[1]}")
            return 0
        print(
            f"::error::invalid release tag {argv[1]!r}; expected canonical "
            "vMAJOR.MINOR.PATCH (no leading zeroes, prerelease or build metadata)"
        )
        return 1
    if argv == ["check-runs"]:
        try:
            payload = json.load(sys.stdin)
        except json.JSONDecodeError as error:
            print(f"::error::check-runs payload is not valid JSON: {error}")
            return 2
        ok, lines = evaluate_check_runs(payload)
        print("\n".join(lines))
        if not ok:
            print("::error::required checks are not all green on the tagged commit")
        return 0 if ok else 1
    print(__doc__)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
