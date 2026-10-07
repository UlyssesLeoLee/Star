"""Verify the INFRA-2C1 resource catalog migration in a disposable loopback PostgreSQL cluster."""

# @cypher schema=1 source_sha256=be14207028f96c5626570526522879073e36877328ce6ed021a2eaeb311e9be0
# MERGE (self:File {path:"scripts/automation/phase_infra2c_resource_catalog.py"})
# MERGE (main:Symbol {id:"scripts/automation/phase_infra2c_resource_catalog.py::main",kind:"function"})
# MERGE (run_step:Symbol {id:"scripts/automation/phase_infra2c_resource_catalog.py::run_step",kind:"function"})
# MERGE (run_sql:Symbol {id:"scripts/automation/phase_infra2c_resource_catalog.py::run_sql",kind:"function"})
# MERGE (contained:Symbol {id:"scripts/automation/phase_infra2c_resource_catalog.py::is_contained",kind:"function"})
# MERGE (reserve_port:Symbol {id:"scripts/automation/phase_infra2c_resource_catalog.py::reserve_port",kind:"function"})
# MERGE (orphan_master_sql:Config {id:"scripts/automation/phase_infra2c_resource_catalog.py::MASTER_WITHOUT_REGISTRATION_FAILURE_SQL"})
# MERGE (migration:Config {id:"db/migrations/2026-10-08-infrastructure-resource-master-catalog.sql"})
# MERGE (base_migration:Config {id:"db/migrations/2026-10-07-infrastructure-capacity-owner-schema.sql"})
# MERGE (grants_migration:Config {id:"db/migrations/2026-10-08-infrastructure-owner-runtime-grants.sql"})
# MERGE (output:Config {id:"target/phase-infra2c-resource-catalog"})
# MERGE (data:Config {id:".cache/phase-infra2c-resource-catalog"})
# MERGE (pg:ExternalService {id:"postgres.initdb_pg_ctl_psql",kind:"command"})
# MERGE (process:ExternalService {id:"python.subprocess",kind:"module"})
# MERGE (socket:ExternalService {id:"python.socket",kind:"module"})
# MERGE (main)-[:CALLS]->(run_step)
# MERGE (main)-[:CALLS]->(run_sql)
# MERGE (main)-[:CALLS]->(contained)
# MERGE (main)-[:CALLS]->(reserve_port)
# MERGE (main)-[:READS]->(orphan_master_sql)
# MERGE (main)-[:USES]->(migration)
# MERGE (main)-[:USES]->(base_migration)
# MERGE (main)-[:USES]->(grants_migration)
# MERGE (main)-[:CONFIGURES]->(output)
# MERGE (main)-[:CONFIGURES]->(data)
# MERGE (main)-[:CONFIGURES]->(pg)
# MERGE (run_step)-[:CALLS]->(process)
# MERGE (run_sql)-[:CALLS]->(run_step)
# MERGE (reserve_port)-[:CALLS]->(socket)
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
BASE_MIGRATION = ROOT / "db" / "migrations" / "2026-10-07-infrastructure-capacity-owner-schema.sql"
GRANTS_MIGRATION = ROOT / "db" / "migrations" / "2026-10-08-infrastructure-owner-runtime-grants.sql"
PROFILE_MIGRATION = ROOT / "db" / "migrations" / "2026-10-09-infrastructure-provider-profile-master.sql"
PHYSICAL_LEDGER_MIGRATION = ROOT / "db" / "migrations" / "2026-10-10-infrastructure-physical-capacity-ledger.sql"
CATALOG_MIGRATION = ROOT / "db" / "migrations" / "2026-10-08-infrastructure-resource-master-catalog.sql"
OUTPUT_DIR = ROOT / "target" / "phase-infra2c-resource-catalog"
DATA_PARENT = ROOT / ".cache" / "phase-infra2c-resource-catalog"
MAX_STEP_SECONDS = 900

FIXTURE_SQL = r"""
BEGIN;
SET LOCAL ROLE star_infrastructure_owner;
INSERT INTO infrastructure.host_environment_catalog
    (host_environment_id, generation, provider_kind, host_os, host_arch, actor_id)
VALUES
    ('10000000-0000-0000-0000-000000000001', 1, 'multipass_local', 'windows', 'x86_64',
     '90000000-0000-0000-0000-000000000001');
INSERT INTO infrastructure.multipass_instance_catalog
    (multipass_instance_id, generation, host_environment_id, host_generation, provider_instance_name, actor_id)
VALUES
    ('10000000-0000-0000-0000-000000000002', 1,
     '10000000-0000-0000-0000-000000000001', 1, 'star-host-01',
     '90000000-0000-0000-0000-000000000001');
INSERT INTO infrastructure.k3s_cluster_catalog
    (k3s_cluster_id, generation, host_environment_id, host_generation, provider_kind,
     cluster_identity_digest, k3s_version, actor_id)
VALUES
    ('10000000-0000-0000-0000-000000000003', 1,
     '10000000-0000-0000-0000-000000000001', 1, 'multipass_local',
     decode(repeat('ab', 32), 'hex'), 'v1.36.5+k3s1',
     '90000000-0000-0000-0000-000000000001');
INSERT INTO infrastructure.k3s_cluster_instance_membership
    (membership_id, generation, k3s_cluster_id, k3s_cluster_generation,
     multipass_instance_id, multipass_instance_generation, host_environment_id,
     host_generation, node_role, actor_id)
VALUES
    ('10000000-0000-0000-0000-000000000004', 1,
     '10000000-0000-0000-0000-000000000003', 1,
     '10000000-0000-0000-0000-000000000002', 1,
     '10000000-0000-0000-0000-000000000001', 1,
     'server', '90000000-0000-0000-0000-000000000001');
INSERT INTO infrastructure.kubernetes_node_catalog
    (kubernetes_node_id, generation, membership_id, membership_generation,
     k3s_cluster_id, k3s_cluster_generation, multipass_instance_id,
     multipass_instance_generation, host_environment_id, host_generation,
     kubernetes_node_uid, kubernetes_node_name, node_role, actor_id)
VALUES
    ('10000000-0000-0000-0000-000000000005', 1,
     '10000000-0000-0000-0000-000000000004', 1,
     '10000000-0000-0000-0000-000000000003', 1,
     '10000000-0000-0000-0000-000000000002', 1,
     '10000000-0000-0000-0000-000000000001', 1,
     'node-uid-0001', 'star-node-01', 'server',
     '90000000-0000-0000-0000-000000000001');
INSERT INTO infrastructure.resource_catalog_event
    (resource_kind, resource_id, resource_generation, event_type, owner_state,
     source_command_id, correlation_id, actor_id)
VALUES
    ('host_environment','10000000-0000-0000-0000-000000000001',1,'registered','active',
     '80000000-0000-0000-0000-000000000001','70000000-0000-0000-0000-000000000001','90000000-0000-0000-0000-000000000001'),
    ('multipass_instance','10000000-0000-0000-0000-000000000002',1,'registered','active',
     '80000000-0000-0000-0000-000000000002','70000000-0000-0000-0000-000000000001','90000000-0000-0000-0000-000000000001'),
    ('k3s_cluster','10000000-0000-0000-0000-000000000003',1,'registered','active',
     '80000000-0000-0000-0000-000000000003','70000000-0000-0000-0000-000000000001','90000000-0000-0000-0000-000000000001'),
    ('cluster_instance_membership','10000000-0000-0000-0000-000000000004',1,'registered','active',
     '80000000-0000-0000-0000-000000000004','70000000-0000-0000-0000-000000000001','90000000-0000-0000-0000-000000000001'),
    ('kubernetes_node','10000000-0000-0000-0000-000000000005',1,'registered','active',
     '80000000-0000-0000-0000-000000000005','70000000-0000-0000-0000-000000000001','90000000-0000-0000-0000-000000000001');
COMMIT;
"""

SECURITY_ASSERTIONS_SQL = r"""
DO $$
DECLARE
    table_name TEXT;
    master_tables TEXT[] := ARRAY[
        'host_environment_catalog','multipass_instance_catalog','k3s_cluster_catalog',
        'k3s_cluster_instance_membership','kubernetes_node_catalog'
    ];
    catalog_tables TEXT[] := ARRAY[
        'host_environment_catalog','multipass_instance_catalog','k3s_cluster_catalog',
        'k3s_cluster_instance_membership','kubernetes_node_catalog','resource_catalog_event'
    ];
    all_tables TEXT[] := ARRAY[
        'capacity_demand_inbox','capacity_reservation_bundle','capacity_outbox_delivery',
        'capacity_demand_receipt','capacity_reservation_event','capacity_owner_outbox',
        'provider_profile_revision','provider_profile_revision_event',
        'capacity_physical_reservation','capacity_physical_reservation_event',
        'host_environment_catalog','multipass_instance_catalog','k3s_cluster_catalog',
        'k3s_cluster_instance_membership','kubernetes_node_catalog','resource_catalog_event'
    ];
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns AS catalog_column
         WHERE catalog_column.table_schema = 'infrastructure'
           AND catalog_column.table_name = ANY(catalog_tables)
           AND catalog_column.column_name = 'tenant_id'
    ) THEN
        RAISE EXCEPTION 'global resource catalog must not carry tenant identity';
    END IF;
    IF (SELECT count(*) FROM pg_policies
         WHERE schemaname = 'infrastructure'
           AND tablename = ANY(catalog_tables)) <> 6
       OR EXISTS (
            SELECT 1 FROM pg_policies
             WHERE schemaname = 'infrastructure'
               AND tablename = ANY(catalog_tables)
               AND (roles <> ARRAY['star_infrastructure_owner']::name[]
                    OR coalesce(qual, '') <> 'true'
                    OR coalesce(with_check, '') <> 'true')
       ) THEN
        RAISE EXCEPTION 'resource catalog RLS must expose all six tables only to the Infrastructure owner';
    END IF;
    FOREACH table_name IN ARRAY master_tables LOOP
        IF NOT has_table_privilege('star_infrastructure_owner', 'infrastructure.' || table_name, 'SELECT')
           OR NOT has_table_privilege('star_infrastructure_owner', 'infrastructure.' || table_name, 'INSERT')
           OR NOT has_table_privilege('star_infrastructure_owner', 'infrastructure.' || table_name, 'UPDATE')
           OR has_table_privilege('star_infrastructure_owner', 'infrastructure.' || table_name, 'DELETE')
           OR has_table_privilege('star_infrastructure_owner', 'infrastructure.' || table_name, 'TRUNCATE')
           OR has_table_privilege('star_infrastructure_owner', 'infrastructure.' || table_name, 'REFERENCES')
           OR has_table_privilege('star_infrastructure_owner', 'infrastructure.' || table_name, 'TRIGGER') THEN
            RAISE EXCEPTION 'Master table privileges are not least privilege: %', table_name;
        END IF;
    END LOOP;
    IF NOT has_table_privilege('star_infrastructure_owner', 'infrastructure.resource_catalog_event', 'SELECT')
       OR NOT has_table_privilege('star_infrastructure_owner', 'infrastructure.resource_catalog_event', 'INSERT')
       OR has_table_privilege('star_infrastructure_owner', 'infrastructure.resource_catalog_event', 'UPDATE')
       OR has_table_privilege('star_infrastructure_owner', 'infrastructure.resource_catalog_event', 'DELETE')
       OR has_table_privilege('star_infrastructure_owner', 'infrastructure.resource_catalog_event', 'TRUNCATE')
       OR has_table_privilege('star_infrastructure_owner', 'infrastructure.resource_catalog_event', 'REFERENCES')
       OR has_table_privilege('star_infrastructure_owner', 'infrastructure.resource_catalog_event', 'TRIGGER') THEN
        RAISE EXCEPTION 'Transaction event privileges are not append-only';
    END IF;
    IF EXISTS (
        SELECT 1 FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace
         WHERE n.nspname = 'infrastructure' AND c.relname = ANY(all_tables)
           AND (NOT c.relrowsecurity OR NOT c.relforcerowsecurity)
    ) THEN
        RAISE EXCEPTION 'all resource catalog tables require ENABLE and FORCE RLS';
    END IF;
END;
$$;
"""

STATE_MACHINE_FAILURE_SQL = r"""
SET ROLE star_infrastructure_owner;
INSERT INTO infrastructure.resource_catalog_event
    (resource_kind, resource_id, resource_generation, event_type, owner_state,
     source_command_id, correlation_id, actor_id)
VALUES
    ('host_environment','10000000-0000-0000-0000-000000000001',1,'registered','active',
     '80000000-0000-0000-0000-000000000099','70000000-0000-0000-0000-000000000001','90000000-0000-0000-0000-000000000001');
"""

MASTER_UPDATE_FAILURE_SQL = r"""
SET ROLE star_infrastructure_owner;
UPDATE infrastructure.host_environment_catalog
   SET host_os = 'linux'
 WHERE host_environment_id = '10000000-0000-0000-0000-000000000001' AND generation = 1;
"""

MASTER_DELETE_FAILURE_SQL = r"""
DELETE FROM infrastructure.host_environment_catalog
 WHERE host_environment_id = '10000000-0000-0000-0000-000000000001' AND generation = 1;
"""

MASTER_WITHOUT_REGISTRATION_FAILURE_SQL = r"""
SET ROLE star_infrastructure_owner;
INSERT INTO infrastructure.host_environment_catalog
    (host_environment_id, generation, provider_kind, host_os, host_arch, actor_id)
VALUES
    ('10000000-0000-0000-0000-000000000099', 1, 'multipass_local', 'windows', 'x86_64',
     '90000000-0000-0000-0000-000000000001');
"""

EVENT_UPDATE_FAILURE_SQL = r"""
UPDATE infrastructure.resource_catalog_event
   SET owner_state = 'draining'
 WHERE resource_kind = 'host_environment'
   AND resource_id = '10000000-0000-0000-0000-000000000001'
   AND resource_generation = 1;
"""

EVENT_TRUNCATE_FAILURE_SQL = "TRUNCATE infrastructure.resource_catalog_event;"

EVENT_UNKNOWN_RESOURCE_FAILURE_SQL = r"""
INSERT INTO infrastructure.resource_catalog_event
    (resource_kind, resource_id, resource_generation, event_type, owner_state,
     source_command_id, correlation_id, actor_id)
VALUES
    ('k3s_cluster','10000000-0000-0000-0000-000000000099',1,'registered','active',
     '80000000-0000-0000-0000-000000000099','70000000-0000-0000-0000-000000000001','90000000-0000-0000-0000-000000000001');
"""

MISMATCHED_HOST_FAILURE_SQL = r"""
INSERT INTO infrastructure.k3s_cluster_instance_membership
    (membership_id, generation, k3s_cluster_id, k3s_cluster_generation,
     multipass_instance_id, multipass_instance_generation, host_environment_id,
     host_generation, node_role, actor_id)
VALUES
    ('10000000-0000-0000-0000-000000000009', 1,
     '10000000-0000-0000-0000-000000000003', 1,
     '10000000-0000-0000-0000-000000000002', 1,
     '10000000-0000-0000-0000-000000000099', 1,
     'agent', '90000000-0000-0000-0000-000000000001');
"""

MULTIPLE_CLUSTER_MEMBERSHIP_FAILURE_SQL = r"""
BEGIN;
SET LOCAL ROLE star_infrastructure_owner;
INSERT INTO infrastructure.k3s_cluster_catalog
    (k3s_cluster_id, generation, host_environment_id, host_generation, provider_kind,
     cluster_identity_digest, k3s_version, actor_id)
VALUES
    ('10000000-0000-0000-0000-000000000006', 1,
     '10000000-0000-0000-0000-000000000001', 1, 'multipass_local',
     decode(repeat('cd', 32), 'hex'), 'v1.36.5+k3s1',
     '90000000-0000-0000-0000-000000000001');
INSERT INTO infrastructure.resource_catalog_event
    (resource_kind, resource_id, resource_generation, event_type, owner_state,
     source_command_id, correlation_id, actor_id)
VALUES
    ('k3s_cluster','10000000-0000-0000-0000-000000000006',1,'registered','active',
     '80000000-0000-0000-0000-000000000008','70000000-0000-0000-0000-000000000001','90000000-0000-0000-0000-000000000001');
INSERT INTO infrastructure.k3s_cluster_instance_membership
    (membership_id, generation, k3s_cluster_id, k3s_cluster_generation,
     multipass_instance_id, multipass_instance_generation, host_environment_id,
     host_generation, node_role, actor_id)
VALUES
    ('10000000-0000-0000-0000-000000000009', 1,
     '10000000-0000-0000-0000-000000000006', 1,
     '10000000-0000-0000-0000-000000000002', 1,
     '10000000-0000-0000-0000-000000000001', 1,
     'agent', '90000000-0000-0000-0000-000000000001');
ROLLBACK;
"""

DUPLICATE_CURRENT_NODE_NAME_FAILURE_SQL = r"""
SET ROLE star_infrastructure_owner;
INSERT INTO infrastructure.kubernetes_node_catalog
    (kubernetes_node_id, generation, membership_id, membership_generation,
     k3s_cluster_id, k3s_cluster_generation, multipass_instance_id,
     multipass_instance_generation, host_environment_id, host_generation,
     kubernetes_node_uid, kubernetes_node_name, node_role, actor_id)
VALUES
    ('10000000-0000-0000-0000-000000000006', 1,
     '10000000-0000-0000-0000-000000000004', 1,
     '10000000-0000-0000-0000-000000000003', 1,
     '10000000-0000-0000-0000-000000000002', 1,
     '10000000-0000-0000-0000-000000000001', 1,
     'node-uid-0002', 'star-node-01', 'server',
     '90000000-0000-0000-0000-000000000001');
"""

NODE_SCD2_SQL = r"""
BEGIN;
SET LOCAL ROLE star_infrastructure_owner;
INSERT INTO infrastructure.resource_catalog_event
    (resource_kind, resource_id, resource_generation, event_type, owner_state,
     source_command_id, correlation_id, actor_id)
VALUES
    ('kubernetes_node','10000000-0000-0000-0000-000000000005',1,'retired','retired',
     '80000000-0000-0000-0000-000000000006','70000000-0000-0000-0000-000000000001','90000000-0000-0000-0000-000000000001');
UPDATE infrastructure.kubernetes_node_catalog
   SET valid_to = clock_timestamp()
 WHERE kubernetes_node_id = '10000000-0000-0000-0000-000000000005' AND generation = 1;
INSERT INTO infrastructure.kubernetes_node_catalog
    (kubernetes_node_id, generation, membership_id, membership_generation,
     k3s_cluster_id, k3s_cluster_generation, multipass_instance_id,
     multipass_instance_generation, host_environment_id, host_generation,
     kubernetes_node_uid, kubernetes_node_name, node_role, actor_id)
VALUES
    ('10000000-0000-0000-0000-000000000005', 2,
     '10000000-0000-0000-0000-000000000004', 1,
     '10000000-0000-0000-0000-000000000003', 1,
     '10000000-0000-0000-0000-000000000002', 1,
     '10000000-0000-0000-0000-000000000001', 1,
     'node-uid-0002', 'star-node-02', 'server',
     '90000000-0000-0000-0000-000000000001');
INSERT INTO infrastructure.resource_catalog_event
    (resource_kind, resource_id, resource_generation, event_type, owner_state,
     source_command_id, correlation_id, actor_id)
VALUES
    ('kubernetes_node','10000000-0000-0000-0000-000000000005',2,'registered','active',
     '80000000-0000-0000-0000-000000000007','70000000-0000-0000-0000-000000000001','90000000-0000-0000-0000-000000000001');
COMMIT;
"""

RLS_NEGATIVE_SQL = r"""
CREATE ROLE resource_catalog_unprivileged NOLOGIN NOSUPERUSER NOBYPASSRLS;
SET ROLE resource_catalog_unprivileged;
SELECT count(*) FROM infrastructure.host_environment_catalog;
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

def run_step(name: str, command: Sequence[str], cwd: Path, env: dict[str, str], sql: str | None = None,
             expect_success: bool = True) -> dict[str, object]:
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    stdout_path = OUTPUT_DIR / f"{name}.stdout.log"
    stderr_path = OUTPUT_DIR / f"{name}.stderr.log"
    if sql is not None:
        command = [*command, "-f", str(write_sql(name, sql))]
    try:
        with stdout_path.open("wb") as stdout, stderr_path.open("wb") as stderr:
            result = subprocess.run(
                list(command),
                cwd=cwd,
                env=env,
                stdin=subprocess.DEVNULL,
                stdout=stdout,
                stderr=stderr,
                check=False,
                timeout=MAX_STEP_SECONDS,
                creationflags=subprocess.CREATE_NO_WINDOW if sys.platform == "win32" else 0,
            )
        passed = result.returncode == 0
        expected = passed == expect_success
        return {
            "name": name,
            "status": "passed" if expected else "failed",
            "exit_code": result.returncode,
            "expected_success": expect_success,
            "stdout_log": str(stdout_path),
            "stderr_log": str(stderr_path),
        }
    except subprocess.TimeoutExpired:
        return {"name": name, "status": "failed", "exit_code": 124, "reason": "timeout"}
    except OSError as error:
        return {"name": name, "status": "blocked", "exit_code": 127, "reason": type(error).__name__}

def write_sql(name: str, sql: str) -> Path:
    path = OUTPUT_DIR / f"{name}.sql"
    path.write_text(sql, encoding="utf-8")
    return path

def psql_command(psql: Path, port: int) -> list[str]:
    return [
        str(psql), "-X", "-h", "127.0.0.1", "-p", str(port), "-U",
        "star_infrastructure_admin", "-d", "postgres", "-v", "ON_ERROR_STOP=1",
    ]

def run_sql(name: str, command: Sequence[str], sql: str, env: dict[str, str],
            expect_success: bool = True) -> dict[str, object]:
    return run_step(name, command, ROOT, env, sql=sql, expect_success=expect_success)

def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pg-bin", type=Path, required=True, help="PostgreSQL bin directory")
    parser.add_argument("--output-dir", type=Path, help="Optional audit/log directory")
    arguments = parser.parse_args()

    global OUTPUT_DIR
    if arguments.output_dir:
        OUTPUT_DIR = arguments.output_dir.resolve()
    pg_bin = arguments.pg_bin.resolve()
    suffix = ".exe" if os.name == "nt" else ""
    tools = {name: pg_bin / f"{name}{suffix}" for name in ("initdb", "pg_ctl", "psql")}
    missing = [str(path) for path in tools.values() if not path.is_file()]
    if missing:
        print(json.dumps({"result": "blocked", "missing_tools": missing}))
        return 2
    migrations = (
        BASE_MIGRATION, GRANTS_MIGRATION, PROFILE_MIGRATION,
        PHYSICAL_LEDGER_MIGRATION, CATALOG_MIGRATION,
    )
    missing_migrations = [str(path) for path in migrations if not path.is_file()]
    if missing_migrations:
        print(json.dumps({"result": "blocked", "missing_migrations": missing_migrations}))
        return 2

    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    DATA_PARENT.mkdir(parents=True, exist_ok=True)
    data_dir = DATA_PARENT / f"postgres-{uuid.uuid4()}"
    if not is_contained(data_dir, DATA_PARENT) or data_dir.exists():
        print(json.dumps({"result": "blocked", "reason": "unsafe or pre-existing data directory"}))
        return 2

    env = os.environ.copy()
    port = reserve_port()
    psql = psql_command(tools["psql"], port)
    results: list[dict[str, object]] = []
    attempted_start = False
    started = False
    stopped = False
    try:
        results.append(run_step(
            "postgres-init",
            [str(tools["initdb"]), "-D", str(data_dir), "-U", "star_infrastructure_admin",
             "--auth=trust", "--no-instructions"],
            ROOT, env,
        ))
        if results[-1]["status"] == "passed":
            attempted_start = True
            results.append(run_step(
                "postgres-start",
                [str(tools["pg_ctl"]), "-D", str(data_dir), "-l", str(data_dir / "server.log"),
                 "-o", f"-h 127.0.0.1 -p {port} -c listen_addresses=127.0.0.1", "-w", "start"],
                ROOT, env,
            ))
            started = results[-1]["status"] == "passed"

        if started:
            for migration in migrations:
                for attempt in (1, 2):
                    results.append(run_sql(
                        f"{migration.stem}-apply-{attempt}",
                        psql, migration.read_text(encoding="utf-8"), env,
                    ))
                    if results[-1]["status"] != "passed":
                        break
                if results[-1]["status"] != "passed":
                    break

        if started and all(item["status"] == "passed" for item in results):
            checks = (
                ("catalog-security-assertions", SECURITY_ASSERTIONS_SQL, True),
                ("catalog-positive-fixture", FIXTURE_SQL, True),
                ("catalog-state-machine-rejects-duplicate-register", STATE_MACHINE_FAILURE_SQL, False),
                ("catalog-rejects-unregistered-event", EVENT_UNKNOWN_RESOURCE_FAILURE_SQL, False),
                ("catalog-rejects-cross-host-membership", MISMATCHED_HOST_FAILURE_SQL, False),
                ("catalog-rejects-instance-in-multiple-clusters", MULTIPLE_CLUSTER_MEMBERSHIP_FAILURE_SQL, False),
                ("catalog-rejects-duplicate-current-node-name", DUPLICATE_CURRENT_NODE_NAME_FAILURE_SQL, False),
                ("catalog-rejects-master-mutation", MASTER_UPDATE_FAILURE_SQL, False),
                ("catalog-rejects-master-delete", MASTER_DELETE_FAILURE_SQL, False),
                ("catalog-rejects-master-without-registration", MASTER_WITHOUT_REGISTRATION_FAILURE_SQL, False),
                ("catalog-rejects-event-mutation", EVENT_UPDATE_FAILURE_SQL, False),
                ("catalog-rejects-event-truncate", EVENT_TRUNCATE_FAILURE_SQL, False),
                ("catalog-scd2-generation-replacement", NODE_SCD2_SQL, True),
                ("catalog-denies-unprivileged-read", RLS_NEGATIVE_SQL, False),
            )
            for name, sql, expected in checks:
                results.append(run_sql(name, psql, sql, env, expect_success=expected))
                if results[-1]["status"] != "passed":
                    break

        if started and all(item["status"] == "passed" for item in results):
            count_query = (
                "SELECT count(*) FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace "
                "WHERE n.nspname='infrastructure' AND c.relkind='r' "
                "AND c.relrowsecurity AND c.relforcerowsecurity;"
            )
            results.append(run_step("force-rls-count", [*psql, "-At", "-c", count_query], ROOT, env))
            count = (OUTPUT_DIR / "force-rls-count.stdout.log").read_text(encoding="utf-8").strip()
            results[-1]["expected_value"] = "16"
            results[-1]["actual_value"] = count
        if count != "16":
                results[-1]["status"] = "failed"

        if started and all(item["status"] == "passed" for item in results):
            results.append(run_sql(
                "catalog-generation-audit",
                psql,
                """DO $$ BEGIN
                    IF (SELECT count(*) FROM infrastructure.resource_catalog_event) <> 7 THEN
                        RAISE EXCEPTION 'unexpected catalog event count';
                    END IF;
                    IF NOT EXISTS (
                        SELECT 1 FROM infrastructure.kubernetes_node_catalog
                         WHERE kubernetes_node_id='10000000-0000-0000-0000-000000000005'
                           AND generation=2 AND valid_to IS NULL
                    ) THEN
                        RAISE EXCEPTION 'expected current node generation is missing';
                    END IF;
                END $$;""",
                env,
            ))
        if all(item["status"] == "passed" for item in results):
            exit_code = 0
        else:
            exit_code = 1
    except Exception as error:
        results.append({"name": "runner", "status": "failed", "reason": type(error).__name__})
        exit_code = 1
    finally:
        if data_dir.exists() and is_contained(data_dir, DATA_PARENT):
            if attempted_start:
                stop = run_step(
                    "postgres-stop",
                    [str(tools["pg_ctl"]), "-D", str(data_dir), "-m", "fast", "-w", "stop"],
                    ROOT, env,
                )
                stopped = stop["status"] == "passed"
                results.append(stop)
            else:
                stopped = True
            if stopped and is_contained(data_dir, DATA_PARENT):
                shutil.rmtree(data_dir)
            else:
                print(json.dumps({"preserved_postgres_data": str(data_dir), "reason": "server stop could not be confirmed"}))
                exit_code = 1
        audit = OUTPUT_DIR / "audit.jsonl"
        audit.write_text("".join(json.dumps(item, sort_keys=True) + "\n" for item in results), encoding="utf-8")
        print(json.dumps({"result": "passed" if all(item["status"] == "passed" for item in results) else "failed",
                          "steps": results}, sort_keys=True))
    return exit_code

if __name__ == "__main__":
    sys.exit(main())
