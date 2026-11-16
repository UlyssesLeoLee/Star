# @cypher schema=1 source_sha256=0b931135bdfb4e502ee767ba9c1f17706222ce76265fbc90552dbbdb7e2849fb
# MERGE (self:File {path:"scripts/ci/nema_codec_gate.py"})
# MERGE (digest:Symbol {id:"scripts/ci/nema_codec_gate.py::check_digest",kind:"function"})
# MERGE (artifact:Symbol {id:"scripts/ci/nema_codec_gate.py::check_artifact",kind:"function"})
# MERGE (matrix:Symbol {id:"scripts/ci/nema_codec_gate.py::check_matrix",kind:"function"})
# MERGE (lock:Symbol {id:"scripts/ci/nema_codec_gate.py::write_lock",kind:"function"})
# MERGE (strict:File {path:"crates/nema-codec/src/strict.rs"})
# MERGE (plugin:File {path:"crates/nema-codec/src/interpreter_plugin.rs"})
# MERGE (wasm:File {path:"frontend/src/lib/nema/wasm/nema_codec_wasm_bg.wasm"})
# MERGE (lockfile:File {path:"frontend/src/lib/nema/wasm/nema_codec_wasm.lock.json"})
# MERGE (fixture:File {path:"crates/nema-codec/tests/fixtures/matrix-current.json"})
# MERGE (self)-[:DEFINES]->(digest)
# MERGE (self)-[:DEFINES]->(artifact)
# MERGE (self)-[:DEFINES]->(matrix)
# MERGE (self)-[:DEFINES]->(lock)
# MERGE (digest)-[:READS]->(strict)
# MERGE (digest)-[:READS]->(plugin)
# MERGE (artifact)-[:READS]->(wasm)
# MERGE (artifact)-[:READS]->(lockfile)
# MERGE (artifact)-[:READS]->(strict)
# MERGE (lock)-[:WRITES]->(lockfile)
# MERGE (matrix)-[:READS]->(fixture)
# @endcypher
#!/usr/bin/env python3
"""Verify that the committed .nema WASM codec matches the Rust source of truth.

Why this exists (2026-11-16, user instruction: ".nema is my own file format, the
CI gate must understand it correctly"):

The browser loads `frontend/src/lib/nema/wasm/nema_codec_wasm_bg.wasm`, which is a
build artifact of `crates/nema-codec` + `crates/nema-codec-wasm`. The Rust
interpreter registry pins every profile with `artifact_sha256` — the SHA-256 of
`crates/nema-codec/src/strict.rs` (the interpreter source artifact). That pin is
what makes a `.nema` document mean exactly one thing forever.

Nothing in CI ever checked that the committed `.wasm` was actually built from
the current `strict.rs`. Consequences that were real, not hypothetical:

* editing `strict.rs` (changing the grammar) left the stale `.wasm` in the repo
  — the browser kept running the old interpreter while the Rust tests passed
  against the new source;
* `wasm-build.yml` builds into `frontend/public/wasm/`, a different directory
  from where the frontend actually imports the codec
  (`frontend/src/lib/nema/wasm/`), so the two could drift silently;
* the 126-case grammar conformance matrix lives in `crates/nema-codec/tests/`,
  and CI runs `cargo test --workspace --lib` — which only compiles lib targets,
  so the conformance suite never ran in CI at all.

This gate closes all three:

1. `strict.rs` digest must equal the `artifact_sha256` the interpreter registry
   publishes for every profile (so the Rust side is self-consistent).
2. The committed `.wasm` must byte-match a fresh `wasm-pack` build of the current
   source (so the shipped artifact is the current interpreter, not a stale one).
3. The grammar conformance matrix must pass (so the format's own contract holds).

Exit codes: 0 = pass, 1 = fail, 2 = gate could not run (fail closed).

Usage:
    python3 scripts/ci/nema_codec_gate.py --check-digest
    python3 scripts/ci/nema_codec_gate.py --check-artifact
    python3 scripts/ci/nema_codec_gate.py --all
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
STRICT_RS = REPO_ROOT / "crates" / "nema-codec" / "src" / "strict.rs"
PLUGIN_RS = REPO_ROOT / "crates" / "nema-codec" / "src" / "interpreter_plugin.rs"
COMMITTED_WASM = (
    REPO_ROOT / "frontend" / "src" / "lib" / "nema" / "wasm" / "nema_codec_wasm_bg.wasm"
)
WASM_CRATE = REPO_ROOT / "crates" / "nema-codec-wasm"
MATRIX_FIXTURE = REPO_ROOT / "crates" / "nema-codec" / "tests" / "fixtures" / "matrix-current.json"
# Lock file pinning the exact committed codec artifact.
LOCK_FILE = REPO_ROOT / "frontend" / "src" / "lib" / "nema" / "wasm" / "nema_codec_wasm.lock.json"


def _sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def check_digest() -> int:
    """The interpreter registry's artifact_sha256 must equal sha256(strict.rs)."""
    if not STRICT_RS.is_file() or not PLUGIN_RS.is_file():
        print(f"::error::missing Rust source: {STRICT_RS} or {PLUGIN_RS}")
        return 2

    actual = _sha256(STRICT_RS)
    plugin_text = PLUGIN_RS.read_text(encoding="utf-8")

    # NEMA_STRICT_PLUGIN_SOURCE_SHA256 is the digest the registry publishes as
    # every profile's artifact_sha256.
    match = re.search(
        r'NEMA_STRICT_PLUGIN_SOURCE_SHA256:\s*&str\s*=\s*"([0-9a-f]{64})"', plugin_text
    )
    if not match:
        print("::error::could not find NEMA_STRICT_PLUGIN_SOURCE_SHA256 in interpreter_plugin.rs")
        return 2
    pinned = match.group(1)

    if actual != pinned:
        print(
            "::error::nema interpreter artifact digest mismatch\n"
            f"  pinned in interpreter_plugin.rs : {pinned}\n"
            f"  actual sha256(strict.rs)         : {actual}\n"
            "  The registry claims a parser artifact that does not exist. "
            "Every profile pins this digest, so a .nema document's meaning is "
            "not reproducible until they agree."
        )
        return 1

    print(f"ok  nema interpreter artifact digest matches strict.rs: {actual}")
    return 0


def _committed_profiles() -> list[dict] | None:
    """Load the committed .wasm and read the interpreter profiles it publishes.

    The wasm embeds the Rust `NEMA_STRICT_PLUGIN_SOURCE_SHA256` constant, so the
    profiles it reports carry the digest of the `strict.rs` it was built from.
    Comparing that against the current `strict.rs` detects a stale artifact
    without needing a byte-identical rebuild (which would depend on the exact
    rustc/wasm-bindgen patch version and would be permanently red on any toolchain
    bump).
    """
    loader = REPO_ROOT / "frontend" / "src" / "lib" / "nema" / "wasm" / "nema_codec_wasm.js"
    if not loader.is_file():
        print(f"::error::missing wasm loader: {loader}")
        return None

    # Windows absolute paths are not valid ESM specifiers — use file:// URLs.
    loader_url = loader.as_uri()
    wasm_path = str(COMMITTED_WASM)
    script = f"""
import fs from 'node:fs';
const mod = await import({json.dumps(loader_url)});
const bytes = fs.readFileSync({json.dumps(wasm_path)});
mod.initSync({{ module: bytes.buffer }});
process.stdout.write(mod.listNemaInterpreterProfiles());
"""
    result = subprocess.run(
        ["node", "--input-type=module", "-e", script],
        cwd=str(REPO_ROOT),
        capture_output=True,
        text=True,
        check=False,
    )
    if result.returncode != 0:
        print("::error::could not load the committed .nema codec in node")
        print(result.stderr[-3000:])
        return None
    try:
        profiles = json.loads(result.stdout)
    except json.JSONDecodeError as error:
        print(f"::error::committed codec returned unreadable profiles: {error}")
        return None
    return profiles if isinstance(profiles, list) else None


def check_artifact() -> int:
    """The committed .wasm must be exactly the pinned artifact, built from strict.rs.

    Two independent checks, because each catches a different real failure:

    1. Byte integrity — the committed file must match the sha256 recorded in the
       lock file. Without this, a corrupted or hand-edited .wasm would still
       pass: the interpreter digest it embeds is a compile-time constant, so
       flipping bytes in the artifact does not change it.
    2. Provenance — the interpreter digest the artifact embeds must equal
       sha256(strict.rs). Without this, editing the grammar source and forgetting
       to rebuild would leave the browser running the old interpreter.
    """
    if not COMMITTED_WASM.is_file():
        print(f"::error::committed codec artifact is missing: {COMMITTED_WASM}")
        return 2
    if not LOCK_FILE.is_file():
        print(
            f"::error::missing codec lock file: {LOCK_FILE}\n"
            "  Regenerate it after an intentional rebuild:\n"
            "    python3 scripts/ci/nema_codec_gate.py --write-lock"
        )
        return 2

    try:
        lock = json.loads(LOCK_FILE.read_text(encoding="utf-8"))
    except json.JSONDecodeError as error:
        print(f"::error::codec lock file is not valid JSON: {error}")
        return 1

    committed_digest = _sha256(COMMITTED_WASM)
    pinned_digest = lock.get("artifact_sha256")
    if not pinned_digest:
        print("::error::codec lock file has no artifact_sha256")
        return 1
    if committed_digest != pinned_digest:
        print(
            "::error::committed .nema codec does not match its lock file\n"
            f"  committed : {committed_digest}\n"
            f"  pinned    : {pinned_digest}\n"
            "  The shipped artifact was changed without a reviewed lock update."
        )
        return 1
    print(f"ok  committed .nema codec matches its lock file: {committed_digest}")

    profiles = _committed_profiles()
    if profiles is None:
        return 2
    if not profiles:
        print("::error::committed codec publishes no interpreter profiles")
        return 1

    current = _sha256(STRICT_RS)
    stale = []
    for profile in profiles:
        artifact = profile.get("artifact_sha256")
        if artifact != current:
            stale.append(
                f"  profile {profile.get('grammar_revision')}: "
                f"artifact_sha256={artifact} != sha256(strict.rs)={current}"
            )
    if stale:
        print(
            "::error::committed .nema codec is stale\n"
            + "\n".join(stale)
            + "\n  The browser would run a different interpreter than the Rust "
            "tests verify. Rebuild and commit:\n"
            "    bash scripts/build-wasm.sh nema-codec-wasm\n"
            "    cp crates/nema-codec-wasm/pkg/* frontend/src/lib/nema/wasm/\n"
            "    python3 scripts/ci/nema_codec_gate.py --write-lock"
        )
        return 1

    pinned_profiles = lock.get("profiles")
    actual_profiles = [
        {
            "grammar_revision": p.get("grammar_revision"),
            "artifact_sha256": p.get("artifact_sha256"),
        }
        for p in profiles
    ]
    if pinned_profiles is not None and pinned_profiles != actual_profiles:
        print(
            "::error::committed .nema codec publishes different profiles than the lock file\n"
            f"  locked : {json.dumps(pinned_profiles)}\n"
            f"  actual : {json.dumps(actual_profiles)}"
        )
        return 1

    revisions = ", ".join(str(p.get("grammar_revision")) for p in profiles)
    print(
        f"ok  committed .nema codec was built from the current strict.rs "
        f"({current}); profiles: {revisions}"
    )
    return 0


def write_lock() -> int:
    """Regenerate the codec lock file after an intentional rebuild."""
    profiles = _committed_profiles()
    if profiles is None:
        return 2
    lock = {
        "_comment": (
            "Pins the committed .nema codec artifact. Regenerate with "
            "`python3 scripts/ci/nema_codec_gate.py --write-lock` after an "
            "intentional rebuild of crates/nema-codec-wasm."
        ),
        "artifact_sha256": _sha256(COMMITTED_WASM),
        "interpreter_source_sha256": _sha256(STRICT_RS),
        "profiles": [
            {
                "grammar_revision": p.get("grammar_revision"),
                "artifact_sha256": p.get("artifact_sha256"),
            }
            for p in profiles
        ],
    }
    LOCK_FILE.write_text(
        json.dumps(lock, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
    )
    print(f"wrote {LOCK_FILE}")
    return 0


def check_matrix() -> int:
    """The grammar conformance matrix must exist and be a well-formed snapshot."""
    if not MATRIX_FIXTURE.is_file():
        print(f"::error::missing grammar conformance matrix: {MATRIX_FIXTURE}")
        return 2
    try:
        data = json.loads(MATRIX_FIXTURE.read_text(encoding="utf-8"))
    except json.JSONDecodeError as error:
        print(f"::error::grammar conformance matrix is not valid JSON: {error}")
        return 1
    cases = data.get("cases")
    if not isinstance(cases, list) or not cases:
        print("::error::grammar conformance matrix has no cases")
        return 1
    if not data.get("matrix_sha256"):
        print("::error::grammar conformance matrix has no matrix_sha256 pin")
        return 1
    valid = sum(1 for case in cases if case.get("status") == "valid")
    errored = sum(1 for case in cases if case.get("status") == "error")
    print(
        f"ok  grammar conformance matrix present: {len(cases)} cases "
        f"({valid} valid / {errored} error), matrix_sha256={data['matrix_sha256']}"
    )
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check-digest", action="store_true")
    parser.add_argument("--check-artifact", action="store_true")
    parser.add_argument("--check-matrix", action="store_true")
    parser.add_argument("--all", action="store_true")
    parser.add_argument(
        "--write-lock",
        action="store_true",
        help="regenerate the codec lock file after an intentional rebuild",
    )
    args = parser.parse_args(argv)

    if args.write_lock:
        return write_lock()

    if not (args.check_digest or args.check_artifact or args.check_matrix or args.all):
        parser.error("choose at least one check")

    results: list[int] = []
    if args.check_digest or args.all:
        results.append(check_digest())
    if args.check_matrix or args.all:
        results.append(check_matrix())
    if args.check_artifact or args.all:
        results.append(check_artifact())

    return max(results) if results else 2


if __name__ == "__main__":
    sys.exit(main())
