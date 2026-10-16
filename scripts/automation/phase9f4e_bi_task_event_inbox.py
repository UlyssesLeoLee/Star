#!/usr/bin/env python3
"""Run the isolated PostgreSQL 18 BI Task event Inbox gate."""

# @cypher schema=1 source_sha256=a240bfbc60c511ca05cb3ff9469c0c179e334992dbc079fd33a3e1ea747659ae
# MERGE (self:File {path:"scripts/automation/phase9f4e_bi_task_event_inbox.py"})
# MERGE (main:Symbol {id:"scripts/automation/phase9f4e_bi_task_event_inbox.py::main",kind:"function"})
# MERGE (run:Symbol {id:"scripts/automation/phase9f4e_bi_task_event_inbox.py::run_command",kind:"function"})
# MERGE (port:Symbol {id:"scripts/automation/phase9f4e_bi_task_event_inbox.py::reserve_loopback_port",kind:"function"})
# MERGE (cleanup:Symbol {id:"scripts/automation/phase9f4e_bi_task_event_inbox.py::remove_owned_data_dir",kind:"function"})
# MERGE (migration:File {path:"db/migrations/2026-10-16-bi-task-event-inbox.sql"})
# MERGE (test:File {path:"db/tests/bi-task-event-inbox.sql"})
# MERGE (self)-[:DEFINES]->(main)
# MERGE (self)-[:DEFINES]->(run)
# MERGE (self)-[:DEFINES]->(port)
# MERGE (self)-[:DEFINES]->(cleanup)
# MERGE (main)-[:CALLS]->(run)
# MERGE (main)-[:CALLS]->(port)
# MERGE (main)-[:CALLS]->(cleanup)
# MERGE (main)-[:READS]->(migration)
# MERGE (main)-[:READS]->(test)
# @endcypher

from __future__ import annotations

import argparse
import shutil
import socket
import subprocess
import sys
import time
import uuid
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
MIGRATION = REPO_ROOT / "db" / "migrations" / "2026-10-16-bi-task-event-inbox.sql"
TEST_SQL = REPO_ROOT / "db" / "tests" / "bi-task-event-inbox.sql"
TARGET_ROOT = (REPO_ROOT / "target").resolve()


def run_command(
    args: list[str],
    *,
    check: bool = True,
    timeout_seconds: int = 30,
    capture_output: bool = True,
) -> subprocess.CompletedProcess[str]:
    if capture_output:
        result = subprocess.run(
            args,
            text=True,
            encoding="utf-8",
            errors="replace",
            capture_output=True,
            check=False,
            timeout=timeout_seconds,
        )
    else:
        result = subprocess.run(
            args,
            text=True,
            encoding="utf-8",
            errors="replace",
            stdin=subprocess.DEVNULL,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            check=False,
            timeout=timeout_seconds,
        )
    if check and result.returncode != 0:
        message = result.stderr.strip() or result.stdout.strip()
        raise RuntimeError(f"command failed ({result.returncode}): {Path(args[0]).name}: {message}")
    return result


def reserve_loopback_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as listener:
        listener.bind(("127.0.0.1", 0))
        return int(listener.getsockname()[1])


def remove_owned_data_dir(data_dir: Path) -> None:
    resolved = data_dir.resolve()
    if resolved.parent != TARGET_ROOT or not resolved.name.startswith("phase9f4e-bi-"):
        raise RuntimeError("refusing cleanup outside this runner's unique target directory")
    shutil.rmtree(resolved)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--pg-bin", type=Path, default=Path(r"D:\PostgreSQL\18\bin"))
    args = parser.parse_args()
    bin_dir = args.pg_bin.resolve()
    initdb = bin_dir / "initdb.exe"
    pg_ctl = bin_dir / "pg_ctl.exe"
    psql = bin_dir / "psql.exe"
    for required in (initdb, pg_ctl, psql, MIGRATION, TEST_SQL):
        if not required.is_file():
            raise RuntimeError(f"required file not found: {required}")

    TARGET_ROOT.mkdir(parents=True, exist_ok=True)
    data_dir = TARGET_ROOT / f"phase9f4e-bi-{uuid.uuid4().hex}"
    log_path = data_dir.with_suffix(".log")
    data_dir.mkdir()
    started = False
    passed = False
    port = reserve_loopback_port()
    try:
        run_command([
            str(initdb), "-D", str(data_dir), "-U", "postgres",
            "--auth-local=trust", "--auth-host=trust", "--no-sync",
        ])
        start = run_command([
            str(pg_ctl), "-D", str(data_dir), "-l", str(log_path),
            "-o", f"-h 127.0.0.1 -p {port} -c listen_addresses=127.0.0.1 -c unix_socket_directories=",
            "-W", "start",
        ], check=False, capture_output=False)
        psql_base = [
            str(psql), "-X", "-v", "ON_ERROR_STOP=1", "-h", "127.0.0.1",
            "-p", str(port), "-U", "postgres", "-d", "postgres",
        ]
        status = run_command([str(pg_ctl), "-D", str(data_dir), "status"], check=False)
        started = start.returncode == 0 or status.returncode == 0
        if not started:
            log_tail = log_path.read_text(encoding="utf-8", errors="replace")[-3000:] if log_path.exists() else ""
            raise RuntimeError(f"PostgreSQL failed to start ({start.returncode})\n{log_tail}")
        for _ in range(60):
            probe = run_command(
                [*psql_base, "-At", "-c", "SELECT 1"], check=False, timeout_seconds=5
            )
            if probe.returncode == 0:
                break
            time.sleep(0.5)
        else:
            raise RuntimeError("PostgreSQL loopback readiness check timed out")
        run_command([*psql_base, "-f", str(MIGRATION)])
        run_command([*psql_base, "-f", str(MIGRATION)])
        run_command([*psql_base, "-f", str(TEST_SQL)])
        print("9F4E BI Task Inbox PostgreSQL gates passed (isolated loopback PostgreSQL).")
        passed = True
    finally:
        pid_file = data_dir / "postmaster.pid"
        stopped = not pid_file.exists()
        if not stopped:
            stop = run_command([str(pg_ctl), "-D", str(data_dir), "-m", "fast", "-w", "-t", "30", "stop"], check=False)
            stopped = stop.returncode == 0 and not pid_file.exists()
        if stopped:
            remove_owned_data_dir(data_dir)
            if log_path.exists():
                log_path.unlink()
        else:
            raise RuntimeError(f"PostgreSQL did not stop; preserving its owned data directory: {data_dir}")
    return 0 if passed else 1


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as exc:
        print(f"9F4E BI Task Inbox gate failed: {exc}", file=sys.stderr)
        raise SystemExit(1)
