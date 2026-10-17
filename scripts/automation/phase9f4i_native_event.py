#!/usr/bin/env python3
# @cypher schema=1 source_sha256=e01524357fd89fdd700a585f8984b6eb2efe60020a5bdabd591d119f15a10cef
# MERGE (self:File {path:"scripts/automation/phase9f4i_native_event.py"})
# MERGE (main:Symbol {id:"scripts/automation/phase9f4i_native_event.py::main",kind:"function"})
# MERGE (run:Symbol {id:"scripts/automation/phase9f4e_bi_task_event_inbox.py::run_command",kind:"function"})
# MERGE (port:Symbol {id:"scripts/automation/phase9f4e_bi_task_event_inbox.py::reserve_loopback_port",kind:"function"})
# MERGE (migration:File {path:"db/migrations/2026-10-18-platform-native-event-kind.sql"})
# MERGE (contract:File {path:"db/tests/platform-native-event-kind.sql"})
# MERGE (self)-[:DEFINES]->(main)
# MERGE (main)-[:CALLS]->(run)
# MERGE (main)-[:CALLS]->(port)
# MERGE (main)-[:READS]->(migration)
# MERGE (main)-[:READS]->(contract)
# MERGE (dependency0:File {path:"db/migrations/2026-10-12-platform-transport-profile-master.sql"})
# MERGE (main)-[:READS]->(dependency0)
# MERGE (dependency1:File {path:"db/migrations/2026-10-12-platform-native-transport-log.sql"})
# MERGE (main)-[:READS]->(dependency1)
# MERGE (dependency2:File {path:"db/migrations/2026-10-12-platform-transport-z-native-server-auth.sql"})
# MERGE (main)-[:READS]->(dependency2)
# MERGE (dependency3:File {path:"db/migrations/2026-10-12-platform-native-origin-verification.sql"})
# MERGE (main)-[:READS]->(dependency3)
# MERGE (dependency4:File {path:"db/migrations/2026-10-16-platform-transport-profile-event-kind.sql"})
# MERGE (main)-[:READS]->(dependency4)
# @endcypher
"""Verify native event authorization in a unique loopback-only disposable PostgreSQL instance."""

from __future__ import annotations

import argparse
import shutil
import time
import uuid
from pathlib import Path

from phase9f4e_bi_task_event_inbox import REPO_ROOT, reserve_loopback_port, run_command


def main() -> int:
    """Apply the real native migration chain twice and exercise event and legacy contracts."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pg-bin", type=Path, default=Path(r"D:\PostgreSQL\18\bin"))
    args = parser.parse_args()
    tools = {name: args.pg_bin.resolve() / f"{name}.exe" for name in ("initdb", "pg_ctl", "psql")}
    migrations = [
        "2026-10-12-platform-transport-profile-master.sql",
        "2026-10-12-platform-native-transport-log.sql",
        "2026-10-12-platform-transport-z-native-server-auth.sql",
        "2026-10-12-platform-native-origin-verification.sql",
        "2026-10-16-platform-transport-profile-event-kind.sql",
        "2026-10-18-platform-native-event-kind.sql",
    ]
    contracts = ["platform-native-transport-server-auth.sql", "platform-native-origin-verification.sql", "platform-native-event-kind.sql"]
    inputs = [*(REPO_ROOT / "db/migrations" / name for name in migrations), *(REPO_ROOT / "db/tests" / name for name in contracts)]
    for required in [*tools.values(), *inputs]:
        if not required.is_file():
            raise RuntimeError(f"missing input: {required}")
    target = (REPO_ROOT / "target").resolve()
    target.mkdir(exist_ok=True)
    data = target / f"phase9f4i-native-event-{uuid.uuid4().hex}"
    log = data.with_suffix(".log")
    port = reserve_loopback_port()
    passed = False
    started = False
    stopped = False
    steps = 0
    try:
        run_command([str(tools["initdb"]), "-D", str(data), "-U", "postgres", "--auth-local=trust", "--auth-host=trust", "--no-sync"])
        start = run_command([str(tools["pg_ctl"]), "-D", str(data), "-l", str(log), "-o", f"-h 127.0.0.1 -p {port} -c listen_addresses=127.0.0.1 -c unix_socket_directories=", "-W", "start"], check=False, capture_output=False)
        status = run_command([str(tools["pg_ctl"]), "-D", str(data), "status"], check=False)
        started = start.returncode == 0 or status.returncode == 0
        if not started:
            raise RuntimeError(f"PostgreSQL startup failed; preserved log: {log}")
        psql = [str(tools["psql"]), "-X", "-v", "ON_ERROR_STOP=1", "-h", "127.0.0.1", "-p", str(port), "-U", "postgres", "-d", "postgres"]
        for _ in range(40):
            if run_command([*psql, "-At", "-c", "SELECT 1"], check=False, timeout_seconds=3).returncode == 0:
                break
            time.sleep(0.25)
        else:
            raise RuntimeError("PostgreSQL loopback readiness timed out")
        for name in migrations:
            for attempt in (1, 2):
                run_command([*psql, "-f", str(REPO_ROOT / "db/migrations" / name)])
                steps += 1
                print(f"PASS migration {name} apply {attempt}", flush=True)
            if name == "2026-10-12-platform-transport-z-native-server-auth.sql":
                run_command([*psql, "-f", str(REPO_ROOT / "db/tests/platform-native-transport-server-auth.sql")])
                steps += 1
                print("PASS pre-attestation server-auth contract", flush=True)
        for name in contracts[1:]:
            run_command([*psql, "-f", str(REPO_ROOT / "db/tests" / name)])
            steps += 1
            print(f"PASS contract {name}", flush=True)
        passed = True
        print(f"PASS {steps} migration/contract steps (not production deployment or BI delivery acceptance)", flush=True)
    finally:
        if started:
            result = run_command([str(tools["pg_ctl"]), "-D", str(data), "-m", "fast", "-w", "stop"], check=False)
            stopped = result.returncode == 0
        if passed and stopped:
            resolved = data.resolve()
            if resolved.parent != target or not resolved.name.startswith("phase9f4i-native-event-"):
                raise RuntimeError("refusing cleanup outside this runner's unique target directory")
            shutil.rmtree(resolved)
        else:
            print(f"Preserved diagnostic data: {data}; log: {log}", flush=True)
        if started and not stopped:
            raise RuntimeError(f"PostgreSQL stop failed; inspect owned instance: {data}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
