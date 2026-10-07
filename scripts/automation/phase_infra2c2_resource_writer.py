"""Verify the INFRA-2C2 Rust writer and schema in disposable PostgreSQL."""

# @cypher schema=1 source_sha256=aaff2266e1650bf7ff18b8b629d1e127aba171e79be3de68f21499f88da1328f
# MERGE (self:File {path:"scripts/automation/phase_infra2c2_resource_writer.py"})
# MERGE (main:Symbol {id:"scripts/automation/phase_infra2c2_resource_writer.py::main",kind:"function"})
# MERGE (contained:Symbol {id:"scripts/automation/phase_infra2c2_resource_writer.py::is_contained",kind:"function"})
# MERGE (reserve_port:Symbol {id:"scripts/automation/phase_infra2c2_resource_writer.py::reserve_port",kind:"function"})
# MERGE (run_step:Symbol {id:"scripts/automation/phase_infra2c2_resource_writer.py::run_step",kind:"function"})
# MERGE (postgres_status:Symbol {id:"scripts/automation/phase_infra2c2_resource_writer.py::postgres_status",kind:"function"})
# MERGE (run_writer_test:Symbol {id:"scripts/automation/phase_infra2c2_resource_writer.py::run_writer_test",kind:"function"})
# MERGE (write_sql:Symbol {id:"scripts/automation/phase_infra2c2_resource_writer.py::write_sql",kind:"function"})
# MERGE (run_sql:Symbol {id:"scripts/automation/phase_infra2c2_resource_writer.py::run_sql",kind:"function"})
# MERGE (migrations:Config {id:"db/migrations/2026-10-08-infrastructure-resource-manager-receipts.sql"})
# MERGE (base_schema:Config {id:"db/migrations/2026-10-07-infrastructure-capacity-owner-schema.sql"})
# MERGE (runtime_grants:Config {id:"db/migrations/2026-10-08-infrastructure-owner-runtime-grants.sql"})
# MERGE (provider_profile:Config {id:"db/migrations/2026-10-09-infrastructure-provider-profile-master.sql"})
# MERGE (capacity_ledger:Config {id:"db/migrations/2026-10-10-infrastructure-physical-capacity-ledger.sql"})
# MERGE (catalog_schema:Config {id:"db/migrations/2026-10-08-infrastructure-resource-master-catalog.sql"})
# MERGE (output:Config {id:"target/phase-infra2c2-resource-writer"})
# MERGE (data:Config {id:".cache/phase-infra2c2-resource-writer"})
# MERGE (postgres:ExternalService {id:"postgres.initdb_pg_ctl_psql",kind:"command"})
# MERGE (cargo:ExternalService {id:"cargo.test",kind:"command"})
# MERGE (process:ExternalService {id:"python.subprocess",kind:"module"})
# MERGE (socket:ExternalService {id:"python.socket",kind:"module"})
# MERGE (main)-[:CALLS]->(contained)
# MERGE (main)-[:CALLS]->(reserve_port)
# MERGE (main)-[:CALLS]->(run_step)
# MERGE (main)-[:CALLS]->(run_sql)
# MERGE (main)-[:READS]->(migrations)
# MERGE (main)-[:READS]->(base_schema)
# MERGE (main)-[:READS]->(runtime_grants)
# MERGE (main)-[:READS]->(provider_profile)
# MERGE (main)-[:READS]->(capacity_ledger)
# MERGE (main)-[:READS]->(catalog_schema)
# MERGE (main)-[:CONFIGURES]->(output)
# MERGE (main)-[:CONFIGURES]->(data)
# MERGE (main)-[:CONFIGURES]->(postgres)
# MERGE (main)-[:VERIFIES_STOPPED]->(postgres)
# MERGE (reserve_port)-[:CALLS]->(socket)
# MERGE (run_step)-[:CALLS]->(process)
# MERGE (run_writer_test)-[:CALLS]->(run_step)
# MERGE (run_writer_test)-[:CONFIGURES]->(cargo)
# MERGE (run_sql)-[:CALLS]->(write_sql)
# MERGE (run_sql)-[:CALLS]->(run_step)
# MERGE (main)-[:CALLS]->(postgres_status)
# MERGE (main)-[:CALLS]->(run_writer_test)
# MERGE (postgres_status)-[:CALLS]->(process)
# @endcypher

from __future__ import annotations

import argparse
import json
import os
import shutil
import socket
import subprocess
import sys
import uuid
from pathlib import Path
from typing import Sequence

ROOT = Path(__file__).resolve().parents[2]
OUTPUT_DIR = ROOT / "target" / "phase-infra2c2-resource-writer"
DATA_PARENT = ROOT / ".cache" / "phase-infra2c2-resource-writer"
MAX_STEP_SECONDS = 900
MIGRATION_NAMES = (
    "2026-10-07-infrastructure-capacity-owner-schema.sql",
    "2026-10-08-infrastructure-owner-runtime-grants.sql",
    "2026-10-09-infrastructure-provider-profile-master.sql",
    "2026-10-10-infrastructure-physical-capacity-ledger.sql",
    "2026-10-08-infrastructure-resource-master-catalog.sql",
    "2026-10-08-infrastructure-resource-manager-receipts.sql",
)

MISMATCHED_EVENT_SQL = r"""
SET ROLE star_infrastructure_owner;
INSERT INTO infrastructure.resource_catalog_event
    (resource_kind, resource_id, resource_generation, event_type, owner_state,
     source_command_id, correlation_id, actor_id, details_digest, manager_receipt_id)
VALUES
    ('host_environment', '54000000-0000-0000-0000-000000000001', 1,
     'state_changed', 'draining', '52000000-0000-0000-0000-000000000002',
     '55000000-0000-0000-0000-000000000001',
     '56000000-0000-0000-0000-000000000001', decode(repeat('ef', 32), 'hex'),
     '51000000-0000-0000-0000-000000000001');
"""

ORPHAN_RECEIPT_SQL = r"""
BEGIN;
SET LOCAL ROLE star_infrastructure_owner;
INSERT INTO infrastructure.resource_catalog_manager_receipt
    (receipt_id, operation_id, schema_version, manager_id, manager_key_id,
     resource_kind, resource_id, resource_generation, event_type, owner_state,
     correlation_id, actor_id, payload_digest, signature, observed_at)
VALUES
    ('51000000-0000-0000-0000-000000000002',
     '52000000-0000-0000-0000-000000000002', 1,
     '53000000-0000-0000-0000-000000000001', 'fixture-key-1',
     'host_environment', '54000000-0000-0000-0000-000000000002', 1,
     'registered', 'active', '55000000-0000-0000-0000-000000000001',
     '56000000-0000-0000-0000-000000000001', decode(repeat('ef', 32), 'hex'),
     decode(repeat('cd', 64), 'hex'), clock_timestamp());
COMMIT;
"""

SECURITY_ASSERTIONS_SQL = r"""
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace
         WHERE n.nspname = 'infrastructure'
           AND c.relname = 'resource_catalog_manager_receipt'
           AND c.relrowsecurity AND c.relforcerowsecurity
    ) THEN RAISE EXCEPTION 'Manager receipt table must use FORCE RLS'; END IF;
    IF NOT has_table_privilege('star_infrastructure_owner',
        'infrastructure.resource_catalog_manager_receipt', 'SELECT,INSERT')
       OR has_table_privilege('star_infrastructure_owner',
        'infrastructure.resource_catalog_manager_receipt', 'UPDATE,DELETE,TRUNCATE') THEN
        RAISE EXCEPTION 'Manager receipt grants exceed append-only owner scope';
    END IF;
    IF (SELECT count(*) FROM infrastructure.resource_catalog_manager_receipt) <> 5
       OR (SELECT count(*) FROM infrastructure.resource_catalog_event) <> 5
       OR (SELECT count(*) FROM infrastructure.host_environment_catalog) <> 1
       OR (SELECT count(*) FROM infrastructure.multipass_instance_catalog) <> 1
       OR (SELECT count(*) FROM infrastructure.k3s_cluster_catalog) <> 1
       OR (SELECT count(*) FROM infrastructure.k3s_cluster_instance_membership) <> 1
       OR (SELECT count(*) FROM infrastructure.kubernetes_node_catalog) <> 1 THEN
        RAISE EXCEPTION 'signed registration facts were not committed atomically';
    END IF;
END;
$$;
"""

RLS_NEGATIVE_SQL = r"""
CREATE ROLE resource_receipt_unprivileged NOLOGIN NOSUPERUSER NOBYPASSRLS;
SET ROLE resource_receipt_unprivileged;
SELECT count(*) FROM infrastructure.resource_catalog_manager_receipt;
"""


def is_contained(path: Path, parent: Path) -> bool:
    try:
        path.resolve().relative_to(parent.resolve())
        return True
    except ValueError:
        return False


def reserve_port() -> int:
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        return int(listener.getsockname()[1])


def write_sql(name: str, sql: str) -> Path:
    path = OUTPUT_DIR / f"{name}.sql"
    path.write_text(sql, encoding="utf-8")
    return path


def run_step(
    name: str,
    command: Sequence[str],
    cwd: Path,
    expect_success: bool = True,
    env: dict[str, str] | None = None,
) -> dict[str, object]:
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    stdout_path = OUTPUT_DIR / f"{name}.stdout.log"
    stderr_path = OUTPUT_DIR / f"{name}.stderr.log"
    try:
        with stdout_path.open("wb") as stdout, stderr_path.open("wb") as stderr:
            result = subprocess.run(
                list(command), cwd=cwd, stdin=subprocess.DEVNULL, stdout=stdout,
                stderr=stderr, check=False, timeout=MAX_STEP_SECONDS, env=env,
                creationflags=subprocess.CREATE_NO_WINDOW if sys.platform == "win32" else 0,
            )
        passed = result.returncode == 0
        return {"name": name, "status": "passed" if passed == expect_success else "failed", "exit_code": result.returncode}
    except Exception as error:
        stderr_path.write_text(f"{type(error).__name__}: {error}\n", encoding="utf-8")
        return {"name": name, "status": "failed", "error": f"{type(error).__name__}: {error}"}


def run_sql(name: str, psql: Sequence[str], sql: str, expect_success: bool = True) -> dict[str, object]:
    return run_step(name, [*psql, "-f", str(write_sql(name, sql))], ROOT, expect_success)


def postgres_status(pg_ctl: Path, data_dir: Path) -> dict[str, object]:
    try:
        result = subprocess.run(
            [str(pg_ctl), "-D", str(data_dir), "status"], cwd=ROOT,
            stdin=subprocess.DEVNULL, capture_output=True, text=True, check=False,
            timeout=30,
            creationflags=subprocess.CREATE_NO_WINDOW if sys.platform == "win32" else 0,
        )
        return {"exit_code": result.returncode}
    except Exception as error:
        return {"exit_code": None, "error": f"{type(error).__name__}: {error}"}


def run_writer_test(cargo_bin: Path, port: int) -> dict[str, object]:
    environment = os.environ.copy()
    environment["STAR_INFRASTRUCTURE_RESOURCE_WRITER_TEST_DATABASE_URL"] = (
        f"postgres://star_infra_test@127.0.0.1:{port}/postgres"
    )
    return run_step(
        "rust-writer-postgres-integration",
        [
            str(cargo_bin), "test", "--manifest-path", str(ROOT / "Cargo.toml"),
            "-p", "infrastructure", "--test", "infrastructure_resource_catalog_writer_pg",
            "--offline", "-j", "4", "--", "--ignored", "--nocapture",
        ],
        ROOT,
        env=environment,
    )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pg-bin", type=Path, required=True, help="PostgreSQL 18.6 bin directory")
    parser.add_argument("--cargo-bin", type=Path, default=None, help="Cargo executable (defaults to PATH)")
    args = parser.parse_args()
    pg_bin = args.pg_bin.resolve()
    cargo_bin = args.cargo_bin.resolve() if args.cargo_bin else Path(shutil.which("cargo") or "")
    executables = {name: pg_bin / f"{name}.exe" for name in ("initdb", "pg_ctl", "psql", "postgres")}
    missing = [str(path) for path in executables.values() if not path.is_file()]
    if not cargo_bin.is_file():
        missing.append(str(cargo_bin) if str(cargo_bin) else "cargo executable")
    migrations = [ROOT / "db" / "migrations" / name for name in MIGRATION_NAMES]
    missing.extend(str(path) for path in migrations if not path.is_file())
    if missing:
        print(json.dumps({"status": "blocked", "missing": missing}, indent=2))
        return 2
    version = subprocess.run([str(executables["postgres"]), "--version"], capture_output=True, text=True, check=False)
    if version.returncode != 0 or "18.6" not in version.stdout:
        print(json.dumps({"status": "blocked", "postgres_version": version.stdout.strip()}, indent=2))
        return 2

    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    DATA_PARENT.mkdir(parents=True, exist_ok=True)
    data_dir = DATA_PARENT / f"pgdata-{uuid.uuid4()}"
    if not is_contained(data_dir, DATA_PARENT):
        raise RuntimeError("temporary PostgreSQL data path escaped its owned parent")
    port = reserve_port()
    psql = [str(executables["psql"]), "-X", "-v", "ON_ERROR_STOP=1", "-h", "127.0.0.1", "-p", str(port), "-U", "star_infra_test", "-d", "postgres"]
    results: list[dict[str, object]] = []
    server_start_attempted = False
    stopped = True
    exit_code = 1
    try:
        results.append(run_step("initdb", [str(executables["initdb"]), "-D", str(data_dir), "-U", "star_infra_test", "--no-locale", "--encoding=UTF8", "--auth-local=trust", "--auth-host=trust"], ROOT))
        if results[-1]["status"] != "passed":
            return 1
        server_start_attempted = True
        stopped = False
        results.append(run_step("pg-start", [str(executables["pg_ctl"]), "-D", str(data_dir), "-l", str(OUTPUT_DIR / "postgres.log"), "-o", f"-h 127.0.0.1 -p {port}", "-w", "start"], ROOT))
        if results[-1]["status"] != "passed":
            return 1
        for migration in migrations:
            for attempt in (1, 2):
                results.append(run_sql(f"{migration.stem}-apply-{attempt}", psql, migration.read_text(encoding="utf-8")))
                if results[-1]["status"] != "passed":
                    break
            if results[-1]["status"] != "passed":
                break
        if all(item["status"] == "passed" for item in results):
            results.append(run_writer_test(cargo_bin, port))
        if results and all(item["status"] == "passed" for item in results):
            checks = (
                ("security-and-positive-facts", SECURITY_ASSERTIONS_SQL, True),
                ("reject-event-receipt-mismatch", MISMATCHED_EVENT_SQL, False),
                ("reject-orphan-manager-receipt", ORPHAN_RECEIPT_SQL, False),
                ("reject-receipt-update", "SET ROLE star_infrastructure_owner; UPDATE infrastructure.resource_catalog_manager_receipt SET recorded_at = clock_timestamp() WHERE receipt_id = '51000000-0000-0000-0000-000000000001';", False),
                ("reject-receipt-delete", "SET ROLE star_infrastructure_owner; DELETE FROM infrastructure.resource_catalog_manager_receipt WHERE receipt_id = '51000000-0000-0000-0000-000000000001';", False),
                ("reject-receipt-truncate", "SET ROLE star_infrastructure_owner; TRUNCATE infrastructure.resource_catalog_manager_receipt;", False),
                ("deny-unprivileged-receipt-read", RLS_NEGATIVE_SQL, False),
            )
            for name, sql, expected in checks:
                results.append(run_sql(name, psql, sql, expected))
                if results[-1]["status"] != "passed":
                    break
        if results and all(item["status"] == "passed" for item in results):
            results.append(run_sql("reapply-after-signed-facts", psql, migrations[-1].read_text(encoding="utf-8")))
            results.append(run_step("force-rls-count", [*psql, "-At", "-c", "SELECT count(*) FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname='infrastructure' AND c.relkind='r' AND c.relrowsecurity AND c.relforcerowsecurity;"], ROOT))
            count = (OUTPUT_DIR / "force-rls-count.stdout.log").read_text(encoding="utf-8").strip()
            results[-1]["expected_value"] = "17"
            results[-1]["actual_value"] = count
            if count != "17":
                results[-1]["status"] = "failed"
        exit_code = 0 if results and all(item["status"] == "passed" for item in results) else 1
    finally:
        if server_start_attempted:
            status = postgres_status(executables["pg_ctl"], data_dir)
            if status["exit_code"] == 0:
                stop = run_step("pg-stop", [str(executables["pg_ctl"]), "-D", str(data_dir), "-m", "fast", "-w", "stop"], ROOT)
                results.append(stop)
                if stop["status"] != "passed":
                    exit_code = 1
                status = postgres_status(executables["pg_ctl"], data_dir)
            stopped = status["exit_code"] == 3
            results.append({
                "name": "pg-confirm-stopped",
                "status": "passed" if stopped else "failed",
                **status,
            })
            if not stopped:
                exit_code = 1
        if stopped and is_contained(data_dir, DATA_PARENT) and data_dir.name.startswith("pgdata-") and data_dir.exists():
            shutil.rmtree(data_dir)
        (OUTPUT_DIR / "results.json").write_text(json.dumps(results, indent=2), encoding="utf-8")
        print(json.dumps(results, indent=2))
    return exit_code


if __name__ == "__main__":
    raise SystemExit(main())
