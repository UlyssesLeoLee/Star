"""Run bounded checks for the Phase 9F2 schedule occurrence contract."""

# @cypher schema=1 source_sha256=e98e013a2a66319632f1156fb94ad15a93c38c72784c1be828d20eeb8ae04fc1
# MERGE (self:File {path:"scripts/automation/phase9f2_schedule_occurrence.py"})
# MERGE (main:Symbol {id:"scripts/automation/phase9f2_schedule_occurrence.py::main",kind:"function"})
# MERGE (run_step:Symbol {id:"scripts/automation/phase9f2_schedule_occurrence.py::run_step",kind:"function"})
# MERGE (check_contract:Symbol {id:"scripts/automation/phase9f2_schedule_occurrence.py::check_migration_contract",kind:"function"})
# MERGE (root:Symbol {id:"scripts/automation/phase9f2_schedule_occurrence.py::ROOT",kind:"constant"})
# MERGE (output_dir:Symbol {id:"scripts/automation/phase9f2_schedule_occurrence.py::OUTPUT_DIR",kind:"constant"})
# MERGE (migration:Symbol {id:"scripts/automation/phase9f2_schedule_occurrence.py::MIGRATION",kind:"constant"})
# MERGE (lockfile:Symbol {id:"scripts/automation/phase9f2_schedule_occurrence.py::LOCKFILE",kind:"constant"})
# MERGE (contract_fragments:Symbol {id:"scripts/automation/phase9f2_schedule_occurrence.py::CONTRACT_FRAGMENTS",kind:"constant"})
# MERGE (timeout:Symbol {id:"scripts/automation/phase9f2_schedule_occurrence.py::TIMEOUT_SECONDS",kind:"constant"})
# MERGE (subprocess:ExternalService {id:"python.subprocess.run",kind:"function"})
# MERGE (pathlib:ExternalService {id:"python.Path.read_text",kind:"method"})
# MERGE (pathlib_write:ExternalService {id:"python.Path.write_bytes",kind:"method"})
# MERGE (pathlib_read_bytes:ExternalService {id:"python.Path.read_bytes",kind:"method"})
# MERGE (pathlib_open:ExternalService {id:"python.Path.open",kind:"method"})
# MERGE (pathlib_mkdir:ExternalService {id:"python.Path.mkdir",kind:"method"})
# MERGE (git:ExternalService {id:"git.diff",kind:"command"})
# MERGE (argparse:ExternalService {id:"python.argparse.ArgumentParser",kind:"function"})
# MERGE (json:ExternalService {id:"python.json.dumps",kind:"function"})
# MERGE (shutil:ExternalService {id:"python.shutil.which",kind:"function"})
# MERGE (system_exit:ExternalService {id:"python.sys.exit",kind:"function"})
# MERGE (system:ExternalService {id:"python.SystemExit",kind:"class"})
# MERGE (self)-[:DEFINES]->(main)
# MERGE (self)-[:DEFINES]->(run_step)
# MERGE (self)-[:DEFINES]->(check_contract)
# MERGE (self)-[:DEFINES]->(root)
# MERGE (self)-[:DEFINES]->(output_dir)
# MERGE (self)-[:DEFINES]->(migration)
# MERGE (self)-[:DEFINES]->(lockfile)
# MERGE (self)-[:DEFINES]->(contract_fragments)
# MERGE (self)-[:DEFINES]->(timeout)
# MERGE (main)-[:CALLS]->(argparse)
# MERGE (main)-[:CALLS]->(subprocess)
# MERGE (main)-[:CALLS]->(git)
# MERGE (main)-[:CALLS]->(shutil)
# MERGE (main)-[:CALLS]->(json)
# MERGE (main)-[:CALLS]->(run_step)
# MERGE (main)-[:CALLS]->(check_contract)
# MERGE (main)-[:USES]->(lockfile)
# MERGE (run_step)-[:CALLS]->(subprocess)
# MERGE (run_step)-[:CALLS]->(pathlib_open)
# MERGE (run_step)-[:CALLS]->(pathlib_mkdir)
# MERGE (run_step)-[:USES]->(output_dir)
# MERGE (run_step)-[:USES]->(timeout)
# MERGE (check_contract)-[:CALLS]->(pathlib)
# MERGE (check_contract)-[:READS]->(migration)
# MERGE (check_contract)-[:USES]->(contract_fragments)
# MERGE (main)-[:CALLS]->(pathlib_write)
# MERGE (main)-[:CALLS]->(pathlib_read_bytes)
# MERGE (self)-[:CALLS]->(system_exit)
# MERGE (self)-[:CALLS]->(main)
# MERGE (self)-[:USES_TYPE]->(system)
# @endcypher

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OUTPUT_DIR = ROOT / ".cache" / "phase9f2-schedule-occurrence"
MIGRATION = ROOT / "db" / "migrations" / "2026-10-02-automation-schedule-occurrence.sql"
TIMEOUT_SECONDS = 900
LOCKFILE = ROOT / "Cargo.lock"
CONTRACT_FRAGMENTS = (
    "CREATE TABLE IF NOT EXISTS automation.schedule_rule_revision",
    "CREATE TABLE IF NOT EXISTS automation.schedule_rule_audit",
    "CREATE TABLE IF NOT EXISTS automation.occurrence (",
    "CREATE TABLE IF NOT EXISTS automation.occurrence_dispatch",
    "CREATE TABLE IF NOT EXISTS automation.occurrence_event",
    "UNIQUE (tenant_id, rule_id, rule_version, scheduled_for_utc)",
    "UNIQUE (tenant_id, project_id, occurrence_id)",
    "fencing_generation BIGINT NOT NULL",
    "automation.guard_occurrence_dispatch",
    "terminal_at + retention_period",
    "FORCE ROW LEVEL SECURITY",
    "retention_period INTERVAL NOT NULL",
)


def check_migration_contract() -> dict[str, object]:
    """Check only that the authored migration retains its declared substrate."""
    sql = MIGRATION.read_text(encoding="utf-8")
    missing = [fragment for fragment in CONTRACT_FRAGMENTS if fragment not in sql]
    return {
        "name": "migration-contract-text",
        "status": "failed" if missing else "passed",
        "missing_fragments": missing,
        "database_executed": False,
    }


def run_step(name: str, command: list[str], cwd: Path) -> dict[str, object]:
    """Run one gate with file-backed logs and a bounded wall-clock limit."""
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    stdout_path = OUTPUT_DIR / f"{name}.stdout.log"
    stderr_path = OUTPUT_DIR / f"{name}.stderr.log"
    try:
        with stdout_path.open("w", encoding="utf-8") as stdout, stderr_path.open(
            "w", encoding="utf-8"
        ) as stderr:
            result = subprocess.run(
                command,
                cwd=cwd,
                stdout=stdout,
                stderr=stderr,
                check=False,
                timeout=TIMEOUT_SECONDS,
            )
        return {
            "name": name,
            "status": "passed" if result.returncode == 0 else "failed",
            "exit_code": result.returncode,
            "command": command,
        }
    except subprocess.TimeoutExpired:
        return {
            "name": name,
            "status": "failed",
            "exit_code": 124,
            "reason": f"exceeded {TIMEOUT_SECONDS}s",
            "command": command,
        }
    except OSError as error:
        return {
            "name": name,
            "status": "blocked",
            "exit_code": 127,
            "reason": str(error),
            "command": command,
        }


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Run bounded Rust and migration-source checks for Schedule 9F2."
    )
    parser.add_argument("--cargo", help="Cargo executable; defaults to PATH lookup.")
    parser.add_argument("--rustfmt", help="rustfmt executable; defaults to PATH lookup.")
    parser.add_argument(
        "--online",
        action="store_true",
        help="allow Cargo dependency resolution/downloads; Cargo.lock is restored afterwards",
    )
    args = parser.parse_args()

    cargo = args.cargo or shutil.which("cargo")
    rustfmt = args.rustfmt or shutil.which("rustfmt")
    results: list[dict[str, object]] = [check_migration_contract()]
    cargo_network_args = [] if args.online else ["--offline"]
    commands = (
        (
            "rustfmt",
            rustfmt,
            [
                "--check",
                "--edition",
                "2024",
                "crates/domain-automation/src/lib.rs",
                "crates/domain-automation/src/schedule.rs",
            ],
        ),
        (
            "domain-tests",
            cargo,
            [*cargo_network_args, "test", "-p", "domain-automation", "--lib", "-j", "4"],
        ),
        (
            "domain-clippy",
            cargo,
            [
                "clippy",
                *cargo_network_args,
                "-p",
                "domain-automation",
                "--all-targets",
                "-j",
                "4",
                "--",
                "-D",
                "warnings",
                "-A",
                "clippy::derivable_impls",
                "-A",
                "clippy::unnecessary_sort_by",
                "-A",
                "clippy::bool_assert_comparison",
                "--no-deps",
            ],
        ),
    )
    lock_snapshot = LOCKFILE.read_bytes()
    lock_status = subprocess.run(
        ["git", "diff", "--quiet", "HEAD", "--", "Cargo.lock"],
        cwd=ROOT,
        check=False,
    )
    try:
        for name, executable, arguments in commands:
            if executable is None:
                results.append(
                    {
                        "name": name,
                        "status": "blocked",
                        "reason": "tool is not on PATH; pass its executable path explicitly",
                    }
                )
            elif name.startswith("domain-") and lock_status.returncode != 0:
                results.append(
                    {
                        "name": name,
                        "status": "blocked",
                        "reason": "Cargo.lock differs from HEAD; skipping resolver to preserve existing changes",
                    }
                )
            else:
                results.append(run_step(name, [str(executable), *arguments], ROOT))
    finally:
        if lock_status.returncode == 0 and LOCKFILE.read_bytes() != lock_snapshot:
            LOCKFILE.write_bytes(lock_snapshot)

    print(
        json.dumps(
            {
                "cargo_mode": "online-transient-resolution" if args.online else "offline",
                "results": results,
                "cargo_lock_restored": LOCKFILE.read_bytes() == lock_snapshot,
            },
            indent=2,
            ensure_ascii=False,
        )
    )
    return 0 if all(result["status"] == "passed" for result in results) else 1


if __name__ == "__main__":
    raise SystemExit(main())
