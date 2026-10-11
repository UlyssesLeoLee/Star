#!/usr/bin/env python3
# @cypher schema=1 source_sha256=dde1fd8c3028cc7d73347e1984e44f99f969a1dc851d7721de76eb7b2a73461c
# MERGE (self:File {path:"scripts/automation/wtg_canvas_stream.py"})
# MERGE (main:Symbol {id:"scripts/automation/wtg_canvas_stream.py::main",kind:"function"})
# MERGE (contract:Symbol {id:"scripts/automation/wtg_canvas_stream.py::source_contract",kind:"function"})
# MERGE (runner:Symbol {id:"scripts/automation/wtg_canvas_stream.py::run_step",kind:"function"})
# MERGE (record:Symbol {id:"scripts/automation/wtg_canvas_stream.py::record",kind:"function"})
# MERGE (api:File {path:"crates/star-api-rest/src/group_api/canvas.rs"})
# MERGE (migration:File {path:"db/migrations/2026-10-05-worktree-group-canvas-stream-offset.sql"})
# MERGE (frontend:File {path:"frontend/src/lib/group/worktreeGroupApi.ts"})
# MERGE (tests:File {path:"frontend/src/lib/group/__tests__/worktreeGroupApi.test.ts"})
# MERGE (audit:File {path:"docs/reports/PHASE-WTG-3C-CANVAS-STREAM.log"})
# MERGE (self)-[:DEFINES]->(main)
# MERGE (self)-[:DEFINES]->(contract)
# MERGE (self)-[:DEFINES]->(runner)
# MERGE (self)-[:DEFINES]->(record)
# MERGE (main)-[:CALLS]->(contract)
# MERGE (main)-[:CALLS]->(runner)
# MERGE (contract)-[:READS]->(api)
# MERGE (contract)-[:READS]->(migration)
# MERGE (contract)-[:READS]->(frontend)
# MERGE (contract)-[:READS]->(tests)
# MERGE (runner)-[:CALLS]->(record)
# MERGE (record)-[:WRITES]->(audit)
# @endcypher
"""Run the bounded Worktree Canvas stream source and focused regression gates."""
from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path
from typing import Sequence


PHASE = "WTG-3C-CANVAS-STREAM"
ROOT = Path(__file__).resolve().parents[2]
TIMEOUT_SECONDS = 900


def record(log_path: Path, action: str, inputs: object, output: object, error: str = "") -> None:
    log_path.parent.mkdir(parents=True, exist_ok=True)
    row = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "phase": PHASE,
        "action": action,
        "input": inputs,
        "output": output,
        "error": error,
    }
    with log_path.open("a", encoding="utf-8") as stream:
        stream.write(json.dumps(row, ensure_ascii=False) + "\n")


def process_output(value: str | bytes | None) -> str:
    if isinstance(value, bytes):
        return value.decode("utf-8", errors="replace")
    return value or ""


def run_step(log_path: Path, name: str, command: Sequence[str], cwd: Path) -> bool:
    started = time.monotonic()
    suffix = Path(command[0]).suffix.lower()
    is_windows_batch = os.name == "nt" and suffix in {".cmd", ".bat"}
    actual_command: Sequence[str] | str = command
    if os.name == "nt" and suffix == ".ps1":
        powershell = shutil.which("pwsh") or shutil.which("powershell")
        if not powershell:
            record(log_path, name, {"command": list(command)}, {}, "PowerShell executable not found")
            return False
        actual_command = [powershell, "-NoProfile", "-File", *command]
    elif is_windows_batch:
        actual_command = subprocess.list2cmdline(list(command))
    try:
        result = subprocess.run(
            actual_command,
            cwd=cwd,
            text=True,
            encoding="utf-8",
            errors="replace",
            capture_output=True,
            check=False,
            shell=is_windows_batch,
            timeout=TIMEOUT_SECONDS,
        )
    except subprocess.TimeoutExpired as exc:
        output = (process_output(exc.stdout) + process_output(exc.stderr))[-2000:]
        record(log_path, name, {"command": list(command), "cwd": str(cwd)}, {}, f"timeout after {TIMEOUT_SECONDS}s: {output}")
        print(f"FAIL {name}: timeout after {TIMEOUT_SECONDS}s", file=sys.stderr)
        return False
    except OSError as exc:
        record(log_path, name, {"command": list(command), "cwd": str(cwd)}, {}, str(exc))
        print(f"FAIL {name}: {exc}", file=sys.stderr)
        return False
    duration_ms = round((time.monotonic() - started) * 1000)
    output = (process_output(result.stdout) + process_output(result.stderr)).strip()
    record(
        log_path,
        name,
        {"command": list(command), "cwd": str(cwd)},
        {"exit_code": result.returncode, "duration_ms": duration_ms, "tail": output[-2000:]},
        "" if result.returncode == 0 else output[-2000:],
    )
    print(f"{'PASS' if result.returncode == 0 else 'FAIL'} {name} ({duration_ms} ms)")
    if result.returncode != 0:
        print(output[-2000:], file=sys.stderr)
    return result.returncode == 0


def source_contract() -> list[str]:
    api = (ROOT / "crates/star-api-rest/src/group_api/canvas.rs").read_text(encoding="utf-8")
    migration = (ROOT / "db/migrations/2026-10-05-worktree-group-canvas-stream-offset.sql").read_text(encoding="utf-8")
    frontend = (ROOT / "frontend/src/lib/group/worktreeGroupApi.ts").read_text(encoding="utf-8")
    tests = (ROOT / "frontend/src/lib/group/__tests__/worktreeGroupApi.test.ts").read_text(encoding="utf-8")
    checks = {
        "worktree events route": '"/api/v1/worktrees/{worktree_id}/canvas-events"' in api,
        "scope authorization": "authorize_group_scope(&state.resolver.pool, &actor, worktree_id, false)" in api,
        "transaction advisory lock": "pg_advisory_xact_lock(hashtextextended($1, 0))" in api,
        "writer acquires stream lock": "lock_worktree_stream(tx, actor.tenant_id, worktree_id).await?" in api,
        "bounded DB deadlines": "set_config('statement_timeout', '5000', true)" in api and "set_config('lock_timeout', '1000', true)" in api,
        "identity offset and unique scope index": "GENERATED ALWAYS AS IDENTITY" in migration and "(tenant_id, worktree_id, stream_offset)" in migration,
        "opaque decimal cursor": "function isValidStreamOffset" in frontend and "after_offset: string" in frontend,
        "focused cursor regressions": "opaque 64-bit decimal offset" in tests and "malformed empty Worktree Canvas pages" in tests,
    }
    return [name for name, passed in checks.items() if not passed]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--audit-log", required=True, type=Path, help="append JSONL audit events to this file")
    parser.add_argument("--cargo", help="explicit Cargo executable when it is not on PATH")
    parser.add_argument("--skip-cargo", action="store_true", help="skip the focused Rust unit test")
    parser.add_argument("--skip-frontend", action="store_true", help="skip focused Vitest and typecheck")
    args = parser.parse_args()
    log_path = args.audit_log if args.audit_log.is_absolute() else ROOT / args.audit_log
    failures = source_contract()
    record(log_path, "source_contract", {"root": str(ROOT)}, {"failed_checks": failures}, "; ".join(failures))
    if failures:
        print("FAIL source_contract: " + "; ".join(failures), file=sys.stderr)
        return 1
    print("PASS source_contract")

    if not args.skip_cargo:
        cargo = args.cargo or shutil.which("cargo")
        if not cargo:
            record(log_path, "cargo_test", {}, {}, "cargo executable not found")
            print("BLOCKED cargo_test: pass --cargo or put cargo on PATH", file=sys.stderr)
            return 2
        if not run_step(log_path, "cargo_test", [cargo, "test", "-p", "star-api-rest", "--lib", "-j", "4", "canvas_event"], ROOT):
            return 1

    if not args.skip_frontend:
        pnpm = shutil.which("pnpm")
        if not pnpm:
            record(log_path, "frontend_tests", {}, {}, "pnpm executable not found")
            print("BLOCKED frontend_tests: pnpm executable not found", file=sys.stderr)
            return 2
        frontend = ROOT / "frontend"
        if not run_step(log_path, "vitest", [pnpm, "exec", "vitest", "run", "src/lib/group/__tests__/worktreeGroupApi.test.ts"], frontend):
            return 1
        if not run_step(log_path, "typecheck", [pnpm, "typecheck"], frontend):
            return 1

    if not run_step(log_path, "diff_check", ["git", "diff", "--check"], ROOT):
        return 1
    if not run_step(log_path, "registry_check", [sys.executable, str(ROOT / "scripts/automation/registry_check.py")], ROOT):
        return 1
    print(f"Audit log: {log_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
