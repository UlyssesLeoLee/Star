#!/usr/bin/env python3
"""Aggregate required-check gate.

Reads the `needs` context as JSON (env NEEDS_JSON) and exits non-zero unless
every needed job finished `success` or `skipped`.

Why this exists: branch rulesets need ONE stable check name. Jobs that are
legitimately skipped (path filtering) must not block a merge, while a job that
failed or was cancelled must. `needs` + `if: always()` + this script gives
exactly that, and an empty/unparseable context fails closed.
"""

# @cypher schema=1
# MERGE (self:File {path:"scripts/ci/gate.py"})
# MERGE (ok:Config {id:"scripts/ci/gate.py::OK",name:"success|skipped are passing results"})
# MERGE (needs_json:Config {id:"env.NEEDS_JSON",name:"toJSON(needs) from the aggregating job"})
# MERGE (summary:Config {id:"github-step-summary-file"})
# MERGE (evaluate:Symbol {id:"scripts/ci/gate.py::evaluate",kind:"function"})
# MERGE (main:Symbol {id:"scripts/ci/gate.py::main",kind:"function"})
# MERGE (self)-[:DEFINES]->(evaluate)
# MERGE (self)-[:DEFINES]->(main)
# MERGE (evaluate)-[:READS]->(ok)
# MERGE (main)-[:READS]->(needs_json)
# MERGE (main)-[:WRITES]->(summary)
# MERGE (main)-[:CALLS]->(evaluate)
# @endcypher

from __future__ import annotations

import json
import os
import sys

OK = {"success", "skipped"}


def evaluate(needs: dict[str, dict[str, str]]) -> tuple[bool, list[str]]:
    """Return (passed, report lines)."""
    if not needs:
        return False, ["no needed jobs reported; refusing to pass an empty gate"]
    lines: list[str] = []
    passed = True
    for name in sorted(needs):
        result = (needs[name] or {}).get("result", "<missing>")
        good = result in OK
        passed = passed and good
        lines.append(f"{'ok  ' if good else 'FAIL'} {name}: {result}")
    return passed, lines


def main() -> int:
    raw = os.environ.get("NEEDS_JSON", "")
    try:
        needs = json.loads(raw)
    except json.JSONDecodeError as error:
        print(f"::error::NEEDS_JSON is not valid JSON: {error}")
        return 2
    if not isinstance(needs, dict):
        print("::error::NEEDS_JSON must be an object")
        return 2
    passed, lines = evaluate(needs)
    print("\n".join(lines))
    summary = os.environ.get("GITHUB_STEP_SUMMARY")
    if summary:
        with open(summary, "a", encoding="utf-8") as handle:
            handle.write("### CI gate\n\n```\n" + "\n".join(lines) + "\n```\n")
    if not passed:
        print("::error::one or more required jobs did not succeed")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
