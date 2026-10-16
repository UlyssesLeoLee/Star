#!/usr/bin/env python3
"""Run isolated PostgreSQL 18 gates for BI Task Inbox, projection, and quarantine."""

# @cypher schema=1 source_sha256=762fc53a888dc0180b0b97bb4b92d5cf18cc003ef77da4118a27089fafb566e8
# MERGE (self:File {path:"scripts/automation/phase9f4e_bi_task_event_inbox.py"})
# MERGE (main:Symbol {id:"scripts/automation/phase9f4e_bi_task_event_inbox.py::main",kind:"function"})
# MERGE (race:Symbol {id:"scripts/automation/phase9f4e_bi_task_event_inbox.py::run_quarantine_race",kind:"function"})
# MERGE (worker:Symbol {id:"scripts/automation/phase9f4e_bi_task_event_inbox.py::record_quarantine_race",kind:"function"})
# MERGE (run:Symbol {id:"scripts/automation/phase9f4e_bi_task_event_inbox.py::run_command",kind:"function"})
# MERGE (port:Symbol {id:"scripts/automation/phase9f4e_bi_task_event_inbox.py::reserve_loopback_port",kind:"function"})
# MERGE (cleanup:Symbol {id:"scripts/automation/phase9f4e_bi_task_event_inbox.py::remove_owned_data_dir",kind:"function"})
# MERGE (migration:File {path:"db/migrations/2026-10-16-bi-task-event-inbox.sql"})
# MERGE (projection_migration:File {path:"db/migrations/2026-10-16-bi-task-event-projection.sql"})
# MERGE (test:File {path:"db/tests/bi-task-event-inbox.sql"})
# MERGE (seed:File {path:"db/tests/bi-task-event-raw-payload-seed.sql"})
# MERGE (projection_test:File {path:"db/tests/bi-task-event-projection.sql"})
# MERGE (quarantine_migration:File {path:"db/migrations/2026-10-17-bi-task-event-quarantine.sql"})
# MERGE (quarantine_test:File {path:"db/tests/bi-task-event-quarantine.sql"})
# MERGE (self)-[:DEFINES]->(main)
# MERGE (self)-[:DEFINES]->(run)
# MERGE (self)-[:DEFINES]->(port)
# MERGE (self)-[:DEFINES]->(cleanup)
# MERGE (main)-[:CALLS]->(run)
# MERGE (main)-[:CALLS]->(port)
# MERGE (main)-[:CALLS]->(cleanup)
# MERGE (main)-[:CALLS]->(race)
# MERGE (race)-[:CALLS]->(worker)
# MERGE (worker)-[:CALLS]->(run)
# MERGE (main)-[:READS]->(migration)
# MERGE (main)-[:READS]->(projection_migration)
# MERGE (main)-[:READS]->(test)
# MERGE (main)-[:READS]->(seed)
# MERGE (main)-[:READS]->(projection_test)
# MERGE (main)-[:READS]->(quarantine_migration)
# MERGE (main)-[:READS]->(quarantine_test)
# @endcypher

from __future__ import annotations

import argparse
from concurrent.futures import ThreadPoolExecutor
import shutil
import socket
import subprocess
import sys
import time
import uuid
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
MIGRATION = REPO_ROOT / "db" / "migrations" / "2026-10-16-bi-task-event-inbox.sql"
PROJECTION_MIGRATION = REPO_ROOT / "db" / "migrations" / "2026-10-16-bi-task-event-projection.sql"
TEST_SQL = REPO_ROOT / "db" / "tests" / "bi-task-event-inbox.sql"
RAW_SEED_SQL = REPO_ROOT / "db" / "tests" / "bi-task-event-raw-payload-seed.sql"
PROJECTION_TEST_SQL = REPO_ROOT / "db" / "tests" / "bi-task-event-projection.sql"
QUARANTINE_MIGRATION = REPO_ROOT / "db" / "migrations" / "2026-10-17-bi-task-event-quarantine.sql"
QUARANTINE_TEST_SQL = REPO_ROOT / "db" / "tests" / "bi-task-event-quarantine.sql"
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


def run_quarantine_race(
    psql_base: list[str],
    *,
    sequence: int,
    message_id: str,
    wire_digests: tuple[str, str],
    expected_outcomes: tuple[str, str],
    expect_same_receipt: bool,
) -> None:
    first_app = f"bi-quarantine-race-{sequence}-first"
    second_app = f"bi-quarantine-race-{sequence}-second"

    def race_sql(wire_digest: str, application_name: str, *, hold_transaction: bool) -> str:
        hold = "SELECT pg_sleep(3);" if hold_transaction else ""
        return f"""
SET application_name = '{application_name}';
BEGIN;
SET ROLE star_bi_runtime;
SET app.tenant_id = '11111111-1111-4111-8111-111111111111';
SELECT outcome || '|' || COALESCE(quarantine_id::TEXT, '')
  FROM bi.record_task_event_quarantine_v1(
      '11111111-1111-4111-8111-111111111111',
      '88888888-8888-4888-8888-888888888888',
      '77777777-7777-4777-8777-777777777777',
      'star-rust-log', 'v1', 'aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa', 1,
      decode(repeat('d4', 32), 'hex'), 'loopback-test', 'bi-task-events',
      'bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb', 'star-rust-log',
      'task-event-log', 'cccccccc-cccc-4ccc-8ccc-cccccccccccc', {sequence},
      '{message_id}', decode(repeat('{wire_digest}', 32), 'hex'), 1024,
      'malformed_envelope'
  );
{hold}
COMMIT;
"""

    def record_quarantine_race(wire_digest: str, application_name: str, *, hold_transaction: bool) -> tuple[str, str]:
        result = run_command(
            [*psql_base, "-qAt", "-c", race_sql(wire_digest, application_name, hold_transaction=hold_transaction)],
            timeout_seconds=20,
        )
        rows = [line for line in result.stdout.splitlines() if "|" in line]
        if len(rows) != 1:
            raise RuntimeError(f"quarantine race returned {len(rows)} result rows: {result.stdout.strip()}")
        outcome, receipt = rows[0].split("|", 1)
        return outcome, receipt

    def wait_for_activity(sql: str, description: str) -> None:
        deadline = time.monotonic() + 8
        while time.monotonic() < deadline:
            result = run_command([*psql_base, "-qAt", "-c", sql], timeout_seconds=5)
            if result.stdout.strip() == "t":
                return
            time.sleep(0.1)
        raise RuntimeError(f"timed out waiting for {description}")

    first_activity = (
        "SELECT EXISTS (SELECT 1 FROM pg_stat_activity "
        f"WHERE application_name = '{first_app}' AND state = 'active' "
        "AND position('pg_sleep' in query) > 0)"
    )
    second_lock_wait = (
        "SELECT EXISTS (SELECT 1 FROM pg_stat_activity "
        f"WHERE application_name = '{second_app}' AND state = 'active' "
        "AND wait_event_type = 'Lock')"
    )

    with ThreadPoolExecutor(max_workers=2) as executor:
        first = executor.submit(
            record_quarantine_race, wire_digests[0], first_app, hold_transaction=True
        )
        wait_for_activity(first_activity, "the first session holding an uncommitted quarantine insert")
        second = executor.submit(
            record_quarantine_race, wire_digests[1], second_app, hold_transaction=False
        )
        wait_for_activity(second_lock_wait, "the second session waiting on the unique-key conflict")
        results = [first.result(timeout=20), second.result(timeout=20)]

    outcomes = tuple(sorted(outcome for outcome, _ in results))
    if outcomes != tuple(sorted(expected_outcomes)):
        raise RuntimeError(f"quarantine race outcomes were {outcomes}, expected {expected_outcomes}")
    receipts = [receipt for _, receipt in results]
    if expect_same_receipt:
        if not receipts[0] or receipts[0] != receipts[1]:
            raise RuntimeError(f"exact-replay race did not return one stable receipt: {receipts}")
    elif sum(bool(receipt) for receipt in receipts) != 1:
        raise RuntimeError(f"conflicting race returned an unexpected receipt set: {receipts}")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--pg-bin", type=Path, default=Path(r"D:\PostgreSQL\18\bin"))
    args = parser.parse_args()
    bin_dir = args.pg_bin.resolve()
    initdb = bin_dir / "initdb.exe"
    pg_ctl = bin_dir / "pg_ctl.exe"
    psql = bin_dir / "psql.exe"
    for required in (
        initdb, pg_ctl, psql, MIGRATION, PROJECTION_MIGRATION,
        QUARANTINE_MIGRATION, TEST_SQL, RAW_SEED_SQL,
        PROJECTION_TEST_SQL, QUARANTINE_TEST_SQL,
    ):
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
        run_command([*psql_base, "-f", str(RAW_SEED_SQL)])
        run_command([*psql_base, "-f", str(PROJECTION_MIGRATION)])
        run_command([*psql_base, "-f", str(PROJECTION_MIGRATION)])
        run_command([*psql_base, "-f", str(TEST_SQL)])
        run_command([*psql_base, "-f", str(PROJECTION_MIGRATION)])
        run_command([*psql_base, "-f", str(PROJECTION_TEST_SQL)])
        run_command([*psql_base, "-f", str(QUARANTINE_MIGRATION)])
        run_command([*psql_base, "-f", str(QUARANTINE_MIGRATION)])
        run_command([*psql_base, "-f", str(QUARANTINE_TEST_SQL)])
        run_quarantine_race(
            psql_base,
            sequence=100,
            message_id="eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee",
            wire_digests=("a1", "a1"),
            expected_outcomes=("recorded", "exact_replay"),
            expect_same_receipt=True,
        )
        run_quarantine_race(
            psql_base,
            sequence=101,
            message_id="ffffffff-ffff-4fff-8fff-ffffffffffff",
            wire_digests=("a1", "b2"),
            expected_outcomes=("recorded", "conflict"),
            expect_same_receipt=False,
        )
        row_count = run_command([*psql_base, "-qAt", "-c", "SELECT count(*) FROM bi.task_event_quarantine"])
        if row_count.stdout.strip() != "5":
            raise RuntimeError(f"concurrent quarantine postcondition expected 5 rows, got {row_count.stdout.strip()}")
        print("9F4E/9F4F/9F4G BI Task Inbox, projection, and quarantine PostgreSQL gates passed (isolated loopback PostgreSQL).")
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
