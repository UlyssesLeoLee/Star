"""Verify 9F4D Infrastructure owner schema, grants, profile Master, and admission in disposable PostgreSQL."""

# @cypher schema=1 source_sha256=bdbefd09153d22afd8af7587182f5f4b75eebbea5452c2ceb64177310f707a51
# MERGE (self:File {path:"scripts/automation/phase9f4d_infrastructure_owner_schema.py"})
# MERGE (main:Symbol {id:"scripts/automation/phase9f4d_infrastructure_owner_schema.py::main",kind:"function"})
# MERGE (grants_migration:Variable {id:"scripts/automation/phase9f4d_infrastructure_owner_schema.py::GRANTS_MIGRATION",kind:"constant"})
# MERGE (owner_probe_sql:Variable {id:"scripts/automation/phase9f4d_infrastructure_owner_schema.py::OWNER_RUNTIME_ROLE_SQL",kind:"constant"})
# MERGE (owner_privileges_sql:Variable {id:"scripts/automation/phase9f4d_infrastructure_owner_schema.py::OWNER_PRIVILEGE_ASSERTIONS_SQL",kind:"constant"})
# MERGE (physical_rls_sql:Variable {id:"scripts/automation/phase9f4d_infrastructure_owner_schema.py::PHYSICAL_RLS_ASSERTIONS_SQL",kind:"constant"})
# MERGE (physical_mutation_sql:Variable {id:"scripts/automation/phase9f4d_infrastructure_owner_schema.py::PHYSICAL_MUTATION_ASSERTIONS_SQL",kind:"constant"})
# MERGE (run_step:Symbol {id:"scripts/automation/phase9f4d_infrastructure_owner_schema.py::run_step",kind:"function"})
# MERGE (write_text:Symbol {id:"scripts/automation/phase9f4d_infrastructure_owner_schema.py::write_text",kind:"function"})
# MERGE (is_contained:Symbol {id:"scripts/automation/phase9f4d_infrastructure_owner_schema.py::is_contained",kind:"function"})
# MERGE (reserve_port:Symbol {id:"scripts/automation/phase9f4d_infrastructure_owner_schema.py::reserve_port",kind:"function"})
# MERGE (run_sql:Symbol {id:"scripts/automation/phase9f4d_infrastructure_owner_schema.py::run_sql",kind:"function"})
# MERGE (root:Variable {id:"scripts/automation/phase9f4d_infrastructure_owner_schema.py::ROOT",kind:"constant"})
# MERGE (migration:Variable {id:"scripts/automation/phase9f4d_infrastructure_owner_schema.py::MIGRATION",kind:"constant"})
# MERGE (profile_migration:Variable {id:"scripts/automation/phase9f4d_infrastructure_owner_schema.py::PROFILE_MIGRATION",kind:"constant"})
# MERGE (global_migration:Variable {id:"scripts/automation/phase9f4d_infrastructure_owner_schema.py::GLOBAL_MIGRATION",kind:"constant"})
# MERGE (cargo_test:Symbol {id:"scripts/automation/phase9f4d_infrastructure_owner_schema.py::owner_inbox_pg_test",kind:"step"})
# MERGE (integration_source:File {path:"crates/infrastructure/tests/infrastructure_owner_inbox_pg.rs"})
# MERGE (output:Variable {id:"scripts/automation/phase9f4d_infrastructure_owner_schema.py::OUTPUT_DIR",kind:"constant"})
# MERGE (pg:ExternalService {id:"postgres.initdb_pg_ctl_psql",kind:"command"})
# MERGE (subprocess:ExternalService {id:"python.subprocess.run",kind:"function"})
# MERGE (self)-[:DEFINES]->(main)
# MERGE (self)-[:DEFINES]->(grants_migration)
# MERGE (self)-[:DEFINES]->(owner_probe_sql)
# MERGE (self)-[:DEFINES]->(owner_privileges_sql)
# MERGE (self)-[:DEFINES]->(physical_rls_sql)
# MERGE (self)-[:DEFINES]->(physical_mutation_sql)
# MERGE (self)-[:DEFINES]->(run_step)
# MERGE (self)-[:DEFINES]->(write_text)
# MERGE (self)-[:DEFINES]->(is_contained)
# MERGE (self)-[:DEFINES]->(reserve_port)
# MERGE (self)-[:DEFINES]->(run_sql)
# MERGE (self)-[:DEFINES]->(root)
# MERGE (self)-[:DEFINES]->(migration)
# MERGE (self)-[:DEFINES]->(profile_migration)
# MERGE (self)-[:DEFINES]->(global_migration)
# MERGE (self)-[:DEFINES]->(output)
# MERGE (main)-[:CALLS]->(run_step)
# MERGE (main)-[:CALLS]->(write_text)
# MERGE (main)-[:CALLS]->(is_contained)
# MERGE (main)-[:CALLS]->(reserve_port)
# MERGE (main)-[:CALLS]->(run_sql)
# MERGE (main)-[:USES]->(root)
# MERGE (main)-[:USES]->(migration)
# MERGE (main)-[:USES]->(profile_migration)
# MERGE (main)-[:USES]->(global_migration)
# MERGE (main)-[:USES]->(grants_migration)
# MERGE (main)-[:USES]->(owner_probe_sql)
# MERGE (main)-[:USES]->(owner_privileges_sql)
# MERGE (main)-[:USES]->(physical_rls_sql)
# MERGE (main)-[:USES]->(physical_mutation_sql)
# MERGE (main)-[:CALLS]->(cargo_test)
# MERGE (cargo_test)-[:CONFIGURES]->(integration_source)
# MERGE (main)-[:USES]->(output)
# MERGE (main)-[:CONFIGURES]->(pg)
# MERGE (run_step)-[:CALLS]->(subprocess)
# MERGE (run_sql)-[:CALLS]->(run_step)
# MERGE (run_sql)-[:CONFIGURES]->(pg)
# @endcypher

from __future__ import annotations

import argparse
import json
import shutil
import socket
import subprocess
import sys
import uuid
from pathlib import Path
from typing import Sequence

ROOT = Path(__file__).resolve().parents[2]
MIGRATION = ROOT / "db" / "migrations" / "2026-10-07-infrastructure-capacity-owner-schema.sql"
GRANTS_MIGRATION = ROOT / "db" / "migrations" / "2026-10-08-infrastructure-owner-runtime-grants.sql"
PROFILE_MIGRATION = ROOT / "db" / "migrations" / "2026-10-09-infrastructure-provider-profile-master.sql"
GLOBAL_MIGRATION = ROOT / "db" / "migrations" / "2026-10-10-infrastructure-physical-capacity-ledger.sql"
OUTPUT_DIR = ROOT / "target" / "phase9f4d-infrastructure-owner-schema"
DATA_PARENT = ROOT / ".cache" / "phase9f4d-infrastructure-owner-schema"
MAX_STEP_SECONDS = 900

OWNER_RUNTIME_ROLE_SQL = """\
CREATE ROLE infrastructure_owner_runtime_probe NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOBYPASSRLS;
GRANT star_infrastructure_owner TO infrastructure_owner_runtime_probe;
"""

OWNER_PRIVILEGE_ASSERTIONS_SQL = """\
DO $$
DECLARE
    table_name TEXT;
    role_is_restricted BOOLEAN;
BEGIN
    SELECT NOT rolcanlogin AND NOT rolsuper AND NOT rolcreatedb AND NOT rolcreaterole
           AND NOT rolreplication AND NOT rolbypassrls AND NOT rolinherit
      INTO role_is_restricted
      FROM pg_roles
     WHERE rolname = 'star_infrastructure_owner';
    IF role_is_restricted IS DISTINCT FROM TRUE THEN
        RAISE EXCEPTION 'Infrastructure owner privilege group has unsafe role attributes';
    END IF;
    IF NOT has_schema_privilege('star_infrastructure_owner', 'infrastructure', 'USAGE') THEN
        RAISE EXCEPTION 'Infrastructure owner role cannot use its schema';
    END IF;
    IF has_schema_privilege('star_infrastructure_owner', 'infrastructure', 'CREATE') THEN
        RAISE EXCEPTION 'Infrastructure owner role can create schema objects';
    END IF;
    IF EXISTS (
        SELECT 1 FROM information_schema.columns AS c
         WHERE c.table_schema = 'infrastructure'
           AND c.table_name IN ('capacity_physical_reservation', 'capacity_physical_reservation_event')
           AND c.column_name = 'tenant_id'
    ) THEN
        RAISE EXCEPTION 'global physical capacity facts must not carry tenant identity';
    END IF;
    IF EXISTS (
        SELECT 1 FROM pg_class c
        JOIN pg_namespace n ON n.oid = c.relnamespace
         WHERE n.nspname = 'infrastructure'
           AND c.relname IN ('capacity_physical_reservation', 'capacity_physical_reservation_event')
           AND (NOT c.relrowsecurity OR NOT c.relforcerowsecurity)
    ) THEN
        RAISE EXCEPTION 'global physical capacity tables require ENABLE and FORCE RLS';
    END IF;
    IF (SELECT count(*) FROM pg_policies
         WHERE schemaname = 'infrastructure'
           AND tablename IN ('capacity_physical_reservation', 'capacity_physical_reservation_event')) <> 2
       OR EXISTS (
            SELECT 1 FROM pg_policies
             WHERE schemaname = 'infrastructure'
               AND tablename IN ('capacity_physical_reservation', 'capacity_physical_reservation_event')
               AND (roles <> ARRAY['star_infrastructure_owner']::name[]
                    OR coalesce(qual, '') <> 'true'
                    OR coalesce(with_check, '') <> 'true')
       ) THEN
        RAISE EXCEPTION 'global physical capacity RLS must expose only the Infrastructure owner';
    END IF;
    IF NOT EXISTS (
        SELECT 1 FROM pg_roles
         WHERE rolname = 'infrastructure_owner_runtime_probe'
           AND NOT rolsuper AND NOT rolbypassrls AND NOT rolcanlogin
    ) THEN
        RAISE EXCEPTION 'Infrastructure owner runtime probe has unsafe role attributes';
    END IF;
    FOREACH table_name IN ARRAY ARRAY[
        'capacity_demand_inbox', 'capacity_reservation_bundle', 'capacity_outbox_delivery',
        'capacity_physical_reservation'
    ] LOOP
        IF NOT has_table_privilege('star_infrastructure_owner', 'infrastructure.' || table_name, 'SELECT')
           OR NOT has_table_privilege('star_infrastructure_owner', 'infrastructure.' || table_name, 'INSERT')
           OR NOT has_table_privilege('star_infrastructure_owner', 'infrastructure.' || table_name, 'UPDATE')
           OR NOT has_table_privilege('star_infrastructure_owner', 'infrastructure.' || table_name, 'DELETE')
           OR has_table_privilege('star_infrastructure_owner', 'infrastructure.' || table_name, 'TRUNCATE')
           OR has_table_privilege('star_infrastructure_owner', 'infrastructure.' || table_name, 'REFERENCES')
           OR has_table_privilege('star_infrastructure_owner', 'infrastructure.' || table_name, 'TRIGGER') THEN
            RAISE EXCEPTION 'Infrastructure owner Work-table privileges are not least-privilege';
        END IF;
    END LOOP;
    FOREACH table_name IN ARRAY ARRAY[
        'capacity_demand_receipt', 'capacity_reservation_event', 'capacity_owner_outbox',
        'capacity_physical_reservation_event'
    ] LOOP
        IF NOT has_table_privilege('star_infrastructure_owner', 'infrastructure.' || table_name, 'SELECT')
           OR NOT has_table_privilege('star_infrastructure_owner', 'infrastructure.' || table_name, 'INSERT')
           OR has_table_privilege('star_infrastructure_owner', 'infrastructure.' || table_name, 'UPDATE')
           OR has_table_privilege('star_infrastructure_owner', 'infrastructure.' || table_name, 'DELETE')
           OR has_table_privilege('star_infrastructure_owner', 'infrastructure.' || table_name, 'TRUNCATE')
           OR has_table_privilege('star_infrastructure_owner', 'infrastructure.' || table_name, 'REFERENCES')
           OR has_table_privilege('star_infrastructure_owner', 'infrastructure.' || table_name, 'TRIGGER') THEN
            RAISE EXCEPTION 'Infrastructure owner Transaction-table privileges are not append-only';
        END IF;
    END LOOP;
    IF NOT has_table_privilege('star_infrastructure_owner', 'infrastructure.provider_profile_revision', 'SELECT')
       OR has_table_privilege('star_infrastructure_owner', 'infrastructure.provider_profile_revision', 'INSERT')
       OR has_table_privilege('star_infrastructure_owner', 'infrastructure.provider_profile_revision', 'UPDATE')
       OR has_table_privilege('star_infrastructure_owner', 'infrastructure.provider_profile_revision', 'DELETE')
       OR has_table_privilege('star_infrastructure_owner', 'infrastructure.provider_profile_revision', 'TRUNCATE') THEN
        RAISE EXCEPTION 'Infrastructure owner profile Master privileges are not read-only';
    END IF;
    IF NOT has_table_privilege('star_infrastructure_owner', 'infrastructure.provider_profile_revision_event', 'SELECT')
       OR has_table_privilege('star_infrastructure_owner', 'infrastructure.provider_profile_revision_event', 'INSERT')
       OR has_table_privilege('star_infrastructure_owner', 'infrastructure.provider_profile_revision_event', 'UPDATE')
       OR has_table_privilege('star_infrastructure_owner', 'infrastructure.provider_profile_revision_event', 'DELETE')
       OR has_table_privilege('star_infrastructure_owner', 'infrastructure.provider_profile_revision_event', 'TRUNCATE') THEN
        RAISE EXCEPTION 'Infrastructure owner profile Transaction privileges are not read-only';
    END IF;
END $$;
"""

RUNTIME_ROLE_SQL = """\
-- Deliberately overprivileged test-only probe to prove database WORM triggers also reject
-- mutations when SQL privileges would otherwise permit them. Never grant this role in production.
CREATE ROLE infrastructure_worm_guard_probe LOGIN NOSUPERUSER NOBYPASSRLS;
GRANT USAGE ON SCHEMA infrastructure TO infrastructure_worm_guard_probe;
GRANT SELECT, INSERT, UPDATE, DELETE, TRUNCATE ON ALL TABLES IN SCHEMA infrastructure
    TO infrastructure_worm_guard_probe;
"""

RUNTIME_FIXTURE_SQL = """\
SELECT set_config('app.tenant_id', '10000000-0000-4000-8000-000000000001', false);
SELECT set_config('app.actor_id', '12000000-0000-4000-8000-000000000001', false);
INSERT INTO infrastructure.provider_profile_revision (
    tenant_id, profile_id, revision, schema_version, provider_kind, placement_policy,
    host_cpu_millicores, host_memory_bytes, host_ephemeral_storage_bytes, host_gpu_units,
    cluster_cpu_millicores, cluster_memory_bytes, cluster_ephemeral_storage_bytes, cluster_gpu_units,
    profile_digest, changed_by
) VALUES (
    '10000000-0000-4000-8000-000000000001', '15000000-0000-4000-8000-000000000001', 1, 1,
    'multipass_k3s_cluster', 'shared_pool', 1000, 1048576, 0, 0, 1000, 1048576, 0, 0,
    decode(repeat('d', 64), 'hex'), '12000000-0000-4000-8000-000000000001'
);
INSERT INTO infrastructure.capacity_physical_reservation (
    reservation_bundle_id, host_capacity_reservation_id, host_environment_id, host_generation,
    host_cpu_millicores, host_memory_bytes, host_ephemeral_storage_bytes, host_gpu_units,
    cluster_capacity_reservation_id, k3s_cluster_id, k3s_cluster_generation,
    cluster_cpu_millicores, cluster_memory_bytes, cluster_ephemeral_storage_bytes, cluster_gpu_units,
    capacity_snapshot_digest, capacity_observed_at, lease_expires_at
) VALUES (
    'd0000000-0000-4000-8000-000000000001', 'e0000000-0000-4000-8000-000000000001',
    'a0000000-0000-4000-8000-000000000001', 1, 1000, 1048576, 0, 0,
    'f0000000-0000-4000-8000-000000000001', 'b0000000-0000-4000-8000-000000000001',
    1, 1000, 1048576, 0, 0, decode(repeat('c', 64), 'hex'),
    clock_timestamp(), clock_timestamp() + interval '1 hour'
);
INSERT INTO infrastructure.capacity_physical_reservation_event (
    reservation_bundle_id, event_type, host_capacity_reservation_id, host_environment_id,
    host_generation, host_cpu_millicores, host_memory_bytes, host_ephemeral_storage_bytes,
    host_gpu_units, cluster_capacity_reservation_id, k3s_cluster_id, k3s_cluster_generation,
    cluster_cpu_millicores, cluster_memory_bytes, cluster_ephemeral_storage_bytes, cluster_gpu_units,
    capacity_snapshot_digest, capacity_observed_at, lease_expires_at, actor_id
) VALUES (
    'd0000000-0000-4000-8000-000000000001', 'held', 'e0000000-0000-4000-8000-000000000001',
    'a0000000-0000-4000-8000-000000000001', 1, 1000, 1048576, 0, 0,
    'f0000000-0000-4000-8000-000000000001', 'b0000000-0000-4000-8000-000000000001',
    1, 1000, 1048576, 0, 0, decode(repeat('c', 64), 'hex'),
    clock_timestamp(), clock_timestamp() + interval '1 hour',
    '12000000-0000-4000-8000-000000000001'
);
BEGIN;
SET LOCAL ROLE infrastructure_owner_runtime_probe;
SELECT set_config('app.tenant_id', '10000000-0000-4000-8000-000000000001', true);
INSERT INTO infrastructure.capacity_demand_inbox (
    tenant_id, source_event_id, occurrence_id, project_id, branch_id,
    engineering_run_id, task_execution_run_id, schedule_rule_id, schedule_rule_revision,
    binding_id, binding_revision, binding_digest, host_environment_id, host_generation,
    k3s_cluster_id, k3s_cluster_generation, provider_kind, placement_policy,
    payload, payload_digest, deadline_at
) VALUES (
    '10000000-0000-4000-8000-000000000001', '20000000-0000-4000-8000-000000000001',
    '30000000-0000-4000-8000-000000000001', '40000000-0000-4000-8000-000000000001',
    '50000000-0000-4000-8000-000000000001', '60000000-0000-4000-8000-000000000001',
    '70000000-0000-4000-8000-000000000001', '80000000-0000-4000-8000-000000000001', 1,
    '90000000-0000-4000-8000-000000000001', 1, decode(repeat('a', 64), 'hex'),
    'a0000000-0000-4000-8000-000000000001', 1, 'b0000000-0000-4000-8000-000000000001',
    1, 'multipass_k3s_cluster', 'shared_pool', '{"schema_version":2}',
    decode(repeat('c', 64), 'hex'), clock_timestamp() + interval '1 hour'
);
INSERT INTO infrastructure.capacity_reservation_bundle (
    reservation_bundle_id, tenant_id, source_event_id, occurrence_id, task_execution_run_id,
    binding_id, binding_revision, binding_digest, host_capacity_reservation_id,
    host_environment_id, host_generation, host_cpu_millicores, host_memory_bytes,
    host_ephemeral_storage_bytes, host_gpu_units, cluster_capacity_reservation_id,
    k3s_cluster_id, k3s_cluster_generation, cluster_cpu_millicores, cluster_memory_bytes,
    cluster_ephemeral_storage_bytes, cluster_gpu_units, capacity_snapshot_digest,
    capacity_observed_at, lease_expires_at
) VALUES (
    'd0000000-0000-4000-8000-000000000001',
    '10000000-0000-4000-8000-000000000001', '20000000-0000-4000-8000-000000000001',
    '30000000-0000-4000-8000-000000000001', '70000000-0000-4000-8000-000000000001',
    '90000000-0000-4000-8000-000000000001', 1, decode(repeat('a', 64), 'hex'),
    'e0000000-0000-4000-8000-000000000001', 'a0000000-0000-4000-8000-000000000001',
    1, 1000, 1048576, 0, 0, 'f0000000-0000-4000-8000-000000000001',
    'b0000000-0000-4000-8000-000000000001', 1, 1000, 1048576, 0, 0,
    decode(repeat('c', 64), 'hex'), clock_timestamp(), clock_timestamp() + interval '1 hour'
);
INSERT INTO infrastructure.capacity_demand_receipt (
    tenant_id, source_event_id, occurrence_id, payload_digest, outcome,
    reservation_bundle_id, correlation_id
) VALUES (
    '10000000-0000-4000-8000-000000000001', '20000000-0000-4000-8000-000000000001',
    '30000000-0000-4000-8000-000000000001', decode(repeat('c', 64), 'hex'), 'accepted',
    'd0000000-0000-4000-8000-000000000001', '11000000-0000-4000-8000-000000000001'
);
INSERT INTO infrastructure.capacity_reservation_event (
    tenant_id, reservation_bundle_id, event_type, host_capacity_reservation_id,
    cluster_capacity_reservation_id, actor_id, correlation_id
) VALUES (
    '10000000-0000-4000-8000-000000000001', 'd0000000-0000-4000-8000-000000000001',
    'held', 'e0000000-0000-4000-8000-000000000001', 'f0000000-0000-4000-8000-000000000001',
    '12000000-0000-4000-8000-000000000001', '11000000-0000-4000-8000-000000000001'
);
INSERT INTO infrastructure.capacity_owner_outbox (
    event_id, tenant_id, source_event_id, occurrence_id, reservation_bundle_id, event_type,
    schema_version, idempotency_key, correlation_id, payload
) VALUES (
    '13000000-0000-4000-8000-000000000001',
    '10000000-0000-4000-8000-000000000001', '20000000-0000-4000-8000-000000000001',
    '30000000-0000-4000-8000-000000000001', 'd0000000-0000-4000-8000-000000000001',
    'capacity_accepted', 2, 'capacity-accepted:20000000-0000-4000-8000-000000000001',
    '11000000-0000-4000-8000-000000000001', '{"schema_version":2}'
);
INSERT INTO infrastructure.capacity_outbox_delivery (tenant_id, event_id)
VALUES (
    '10000000-0000-4000-8000-000000000001', '13000000-0000-4000-8000-000000000001'
);
COMMIT;
"""

RUNTIME_ASSERTIONS_SQL = """\
BEGIN;
SET LOCAL ROLE infrastructure_owner_runtime_probe;
SELECT set_config('app.tenant_id', '10000000-0000-4000-8000-000000000001', true);
DO $$
DECLARE row_count INTEGER;
BEGIN
    SELECT count(*) INTO row_count FROM infrastructure.capacity_demand_inbox;
    IF row_count <> 1 THEN RAISE EXCEPTION 'tenant A Inbox row is not visible'; END IF;
    SELECT count(*) INTO row_count FROM infrastructure.capacity_reservation_bundle;
    IF row_count <> 1 THEN RAISE EXCEPTION 'paired reservation row is not visible'; END IF;
    SELECT count(*) INTO row_count FROM infrastructure.capacity_demand_receipt;
    IF row_count <> 1 THEN RAISE EXCEPTION 'tenant A demand receipt is not visible'; END IF;
    SELECT count(*) INTO row_count FROM infrastructure.capacity_reservation_event;
    IF row_count <> 1 THEN RAISE EXCEPTION 'tenant A reservation event is not visible'; END IF;
    SELECT count(*) INTO row_count FROM infrastructure.capacity_owner_outbox;
    IF row_count <> 1 THEN RAISE EXCEPTION 'tenant A owner Outbox row is not visible'; END IF;
    SELECT count(*) INTO row_count FROM infrastructure.capacity_outbox_delivery;
    IF row_count <> 1 THEN RAISE EXCEPTION 'tenant A Outbox delivery row is not visible'; END IF;
    SELECT count(*) INTO row_count FROM infrastructure.provider_profile_revision;
    IF row_count <> 1 THEN RAISE EXCEPTION 'tenant A provider profile is not visible'; END IF;
    SELECT count(*) INTO row_count FROM infrastructure.provider_profile_revision_event;
    IF row_count <> 1 THEN RAISE EXCEPTION 'tenant A provider profile event is not visible'; END IF;
    SELECT count(*) INTO row_count FROM infrastructure.capacity_physical_reservation;
    IF row_count <> 1 THEN RAISE EXCEPTION 'Infrastructure owner cannot read the global physical reservation'; END IF;
    SELECT count(*) INTO row_count FROM infrastructure.capacity_physical_reservation_event;
    IF row_count <> 1 THEN RAISE EXCEPTION 'Infrastructure owner cannot read the global physical event'; END IF;
END $$;
SELECT set_config('app.tenant_id', '10000000-0000-4000-8000-000000000002', true);
DO $$
DECLARE row_count INTEGER;
BEGIN
    SELECT count(*) INTO row_count FROM infrastructure.capacity_demand_inbox;
    IF row_count <> 0 THEN RAISE EXCEPTION 'tenant B can read tenant A Inbox'; END IF;
    SELECT count(*) INTO row_count FROM infrastructure.capacity_owner_outbox;
    IF row_count <> 0 THEN RAISE EXCEPTION 'tenant B can read tenant A Outbox'; END IF;
    SELECT count(*) INTO row_count FROM infrastructure.capacity_reservation_bundle;
    IF row_count <> 0 THEN RAISE EXCEPTION 'tenant B can read tenant A reservation'; END IF;
    SELECT count(*) INTO row_count FROM infrastructure.capacity_demand_receipt;
    IF row_count <> 0 THEN RAISE EXCEPTION 'tenant B can read tenant A receipt'; END IF;
    SELECT count(*) INTO row_count FROM infrastructure.capacity_reservation_event;
    IF row_count <> 0 THEN RAISE EXCEPTION 'tenant B can read tenant A reservation event'; END IF;
    SELECT count(*) INTO row_count FROM infrastructure.capacity_outbox_delivery;
    IF row_count <> 0 THEN RAISE EXCEPTION 'tenant B can read tenant A delivery row'; END IF;
    SELECT count(*) INTO row_count FROM infrastructure.provider_profile_revision;
    IF row_count <> 0 THEN RAISE EXCEPTION 'tenant B can read tenant A provider profile'; END IF;
    SELECT count(*) INTO row_count FROM infrastructure.provider_profile_revision_event;
    IF row_count <> 0 THEN RAISE EXCEPTION 'tenant B can read tenant A provider profile event'; END IF;
    SELECT count(*) INTO row_count FROM infrastructure.capacity_physical_reservation;
    IF row_count <> 1 THEN RAISE EXCEPTION 'physical ledger unexpectedly depends on tenant scope'; END IF;
    BEGIN
        INSERT INTO infrastructure.capacity_demand_inbox (
            tenant_id, source_event_id, occurrence_id, project_id, branch_id,
            engineering_run_id, task_execution_run_id, schedule_rule_id, schedule_rule_revision,
            binding_id, binding_revision, binding_digest, host_environment_id, host_generation,
            k3s_cluster_id, k3s_cluster_generation, provider_kind, placement_policy,
            payload, payload_digest, deadline_at
        ) VALUES (
            '10000000-0000-4000-8000-000000000001', '21000000-0000-4000-8000-000000000001',
            '31000000-0000-4000-8000-000000000001', '41000000-0000-4000-8000-000000000001',
            '51000000-0000-4000-8000-000000000001', '61000000-0000-4000-8000-000000000001',
            '71000000-0000-4000-8000-000000000001', '81000000-0000-4000-8000-000000000001', 1,
            '91000000-0000-4000-8000-000000000001', 1, decode(repeat('a', 64), 'hex'),
            'a1000000-0000-4000-8000-000000000001', 1, 'b1000000-0000-4000-8000-000000000001',
            1, 'multipass_k3s_cluster', 'shared_pool', '{"schema_version":2}',
            decode(repeat('c', 64), 'hex'), clock_timestamp() + interval '1 hour'
        );
        RAISE EXCEPTION 'cross-tenant INSERT was not rejected';
    EXCEPTION WHEN insufficient_privilege THEN NULL;
    END;
END $$;
ROLLBACK;
"""

PHYSICAL_RLS_ASSERTIONS_SQL = """\
BEGIN;
SET LOCAL ROLE infrastructure_worm_guard_probe;
SELECT set_config('app.tenant_id', '10000000-0000-4000-8000-000000000001', true);
DO $$
DECLARE row_count INTEGER;
BEGIN
    SELECT count(*) INTO row_count FROM infrastructure.capacity_physical_reservation;
    IF row_count <> 0 THEN RAISE EXCEPTION 'tenant-scoped probe can read the global physical reservation'; END IF;
    SELECT count(*) INTO row_count FROM infrastructure.capacity_physical_reservation_event;
    IF row_count <> 0 THEN RAISE EXCEPTION 'tenant-scoped probe can read the global physical event'; END IF;
    BEGIN
        INSERT INTO infrastructure.capacity_physical_reservation (
            reservation_bundle_id, host_capacity_reservation_id, host_environment_id, host_generation,
            host_cpu_millicores, host_memory_bytes, host_ephemeral_storage_bytes, host_gpu_units,
            cluster_capacity_reservation_id, k3s_cluster_id, k3s_cluster_generation,
            cluster_cpu_millicores, cluster_memory_bytes, cluster_ephemeral_storage_bytes, cluster_gpu_units,
            capacity_snapshot_digest, capacity_observed_at, lease_expires_at
        ) VALUES (
            'd1000000-0000-4000-8000-000000000001', 'e1000000-0000-4000-8000-000000000001',
            'a0000000-0000-4000-8000-000000000001', 1, 1, 1, 0, 0,
            'f1000000-0000-4000-8000-000000000001', 'b0000000-0000-4000-8000-000000000001',
            1, 1, 1, 0, 0, decode(repeat('c', 64), 'hex'), clock_timestamp(),
            clock_timestamp() + interval '1 hour'
        );
        RAISE EXCEPTION 'tenant-scoped probe inserted into the global physical reservation';
    EXCEPTION WHEN insufficient_privilege THEN NULL;
    END;
END $$;
ROLLBACK;
"""

MUTATION_ASSERTIONS_SQL = """\
BEGIN;
SET LOCAL ROLE infrastructure_worm_guard_probe;
SELECT set_config('app.tenant_id', '10000000-0000-4000-8000-000000000001', true);
SELECT set_config('app.actor_id', '12000000-0000-4000-8000-000000000002', true);
DO $$
DECLARE denied BOOLEAN;
BEGIN
    denied := FALSE;
    BEGIN
        UPDATE infrastructure.capacity_demand_inbox
           SET payload = '{"schema_version":3}'::JSONB;
    EXCEPTION WHEN raise_exception THEN denied := TRUE;
    END;
    IF NOT denied THEN RAISE EXCEPTION 'Inbox payload rewrite was not blocked'; END IF;
    denied := FALSE;
    BEGIN
        UPDATE infrastructure.capacity_demand_inbox
           SET processing_state = 'rejected',
               terminal_at = clock_timestamp(),
               retention_expires_at = clock_timestamp() + interval '1 hour';
    EXCEPTION WHEN raise_exception THEN denied := TRUE;
    END;
    IF NOT denied THEN RAISE EXCEPTION 'Short Inbox terminal retention was not blocked'; END IF;
    denied := FALSE;
    BEGIN
        UPDATE infrastructure.capacity_demand_inbox
           SET processing_state = 'rejected',
               terminal_at = clock_timestamp(),
               retention_expires_at = clock_timestamp() + interval '30 days';
    EXCEPTION WHEN raise_exception THEN denied := TRUE;
    END;
    IF NOT denied THEN RAISE EXCEPTION 'Unbounded Inbox terminal retention was not blocked'; END IF;
    denied := FALSE;
    BEGIN
        UPDATE infrastructure.capacity_demand_inbox
           SET processing_state = 'rejected',
               terminal_at = clock_timestamp() - interval '2 days',
               retention_expires_at = clock_timestamp() - interval '1 day';
    EXCEPTION WHEN raise_exception THEN denied := TRUE;
    END;
    IF NOT denied THEN RAISE EXCEPTION 'Backdated Inbox retention was not blocked'; END IF;
    denied := FALSE;
    BEGIN
        UPDATE infrastructure.capacity_demand_receipt SET outcome = 'rejected';
    EXCEPTION WHEN raise_exception THEN denied := TRUE;
    END;
    IF NOT denied THEN RAISE EXCEPTION 'Transaction receipt UPDATE was not blocked'; END IF;
    denied := FALSE;
    BEGIN
        DELETE FROM infrastructure.capacity_owner_outbox;
    EXCEPTION WHEN raise_exception THEN denied := TRUE;
    END;
    IF NOT denied THEN RAISE EXCEPTION 'Outbox DELETE was not blocked'; END IF;
    denied := FALSE;
    BEGIN
        UPDATE infrastructure.capacity_reservation_event SET event_type = 'released';
    EXCEPTION WHEN raise_exception THEN denied := TRUE;
    END;
    IF NOT denied THEN RAISE EXCEPTION 'Reservation event UPDATE was not blocked'; END IF;
    denied := FALSE;
    BEGIN
        DELETE FROM infrastructure.capacity_demand_receipt;
    EXCEPTION WHEN raise_exception THEN denied := TRUE;
    END;
    IF NOT denied THEN RAISE EXCEPTION 'Demand receipt DELETE was not blocked'; END IF;
    denied := FALSE;
    BEGIN
        DELETE FROM infrastructure.capacity_reservation_event;
    EXCEPTION WHEN raise_exception THEN denied := TRUE;
    END;
    IF NOT denied THEN RAISE EXCEPTION 'Reservation event DELETE was not blocked'; END IF;
    denied := FALSE;
    BEGIN
        TRUNCATE TABLE infrastructure.capacity_demand_receipt;
    EXCEPTION WHEN raise_exception THEN denied := TRUE;
    END;
    IF NOT denied THEN RAISE EXCEPTION 'Demand receipt TRUNCATE was not blocked'; END IF;
    denied := FALSE;
    BEGIN
        TRUNCATE TABLE infrastructure.capacity_reservation_event;
    EXCEPTION WHEN raise_exception THEN denied := TRUE;
    END;
    IF NOT denied THEN RAISE EXCEPTION 'Reservation event TRUNCATE was not blocked'; END IF;
    denied := FALSE;
    BEGIN
        TRUNCATE TABLE infrastructure.capacity_owner_outbox CASCADE;
    EXCEPTION WHEN raise_exception THEN denied := TRUE;
    END;
    IF NOT denied THEN RAISE EXCEPTION 'Owner Outbox TRUNCATE was not blocked'; END IF;
    denied := FALSE;
    BEGIN
        UPDATE infrastructure.capacity_owner_outbox SET event_type = 'capacity_rejected';
    EXCEPTION WHEN raise_exception THEN denied := TRUE;
    END;
    IF NOT denied THEN RAISE EXCEPTION 'Owner Outbox UPDATE was not blocked'; END IF;
    denied := FALSE;
    BEGIN
        UPDATE infrastructure.capacity_reservation_bundle
           SET host_memory_bytes = host_memory_bytes + 1;
    EXCEPTION WHEN raise_exception THEN denied := TRUE;
    END;
    IF NOT denied THEN RAISE EXCEPTION 'Reservation capacity rewrite was not blocked'; END IF;
    denied := FALSE;
    BEGIN
        DELETE FROM infrastructure.capacity_reservation_bundle;
    EXCEPTION WHEN raise_exception THEN denied := TRUE;
    END;
    IF NOT denied THEN RAISE EXCEPTION 'Active reservation cleanup was not blocked'; END IF;
    denied := FALSE;
    BEGIN
        UPDATE infrastructure.capacity_reservation_bundle
           SET reservation_state = 'released',
               terminal_at = clock_timestamp(),
               retention_expires_at = clock_timestamp() + interval '30 days';
    EXCEPTION WHEN raise_exception THEN denied := TRUE;
    END;
    IF NOT denied THEN RAISE EXCEPTION 'Unbounded reservation retention was not blocked'; END IF;
    denied := FALSE;
    BEGIN
        UPDATE infrastructure.capacity_reservation_bundle
           SET reservation_state = 'released',
               terminal_at = clock_timestamp(),
               retention_expires_at = clock_timestamp() + interval '24 hours';
    EXCEPTION WHEN raise_exception THEN denied := TRUE;
    END;
    IF denied THEN RAISE EXCEPTION 'Valid reservation release transition was rejected'; END IF;
    denied := FALSE;
    BEGIN
        DELETE FROM infrastructure.capacity_reservation_bundle;
    EXCEPTION WHEN raise_exception THEN denied := TRUE;
    END;
    IF NOT denied THEN RAISE EXCEPTION 'Reservation cleanup before terminal retention was not blocked'; END IF;
    denied := FALSE;
    BEGIN
        UPDATE infrastructure.capacity_demand_inbox
           SET processing_state = 'rejected',
               terminal_at = clock_timestamp(),
               retention_expires_at = clock_timestamp() + interval '24 hours';
    EXCEPTION WHEN raise_exception THEN denied := TRUE;
    END;
    IF denied THEN RAISE EXCEPTION 'Valid Inbox rejection transition was rejected'; END IF;
    denied := FALSE;
    BEGIN
        DELETE FROM infrastructure.capacity_demand_inbox;
    EXCEPTION WHEN raise_exception THEN denied := TRUE;
    END;
    IF NOT denied THEN RAISE EXCEPTION 'Inbox cleanup before terminal retention was not blocked'; END IF;
    denied := FALSE;
    BEGIN
        UPDATE infrastructure.capacity_outbox_delivery
           SET delivery_state = 'leased',
               lease_owner_id = '14000000-0000-4000-8000-000000000001',
               lease_expires_at = clock_timestamp() + interval '1 hour';
        UPDATE infrastructure.capacity_outbox_delivery
           SET delivery_state = 'delivered',
               lease_owner_id = NULL,
               lease_expires_at = NULL,
               terminal_at = clock_timestamp(),
               retention_expires_at = clock_timestamp() + interval '24 hours';
    EXCEPTION WHEN raise_exception THEN denied := TRUE;
    END;
    IF denied THEN RAISE EXCEPTION 'Valid Outbox delivery transitions were rejected'; END IF;
    denied := FALSE;
    BEGIN
        DELETE FROM infrastructure.capacity_outbox_delivery;
    EXCEPTION WHEN raise_exception THEN denied := TRUE;
    END;
    IF NOT denied THEN RAISE EXCEPTION 'Outbox delivery cleanup before terminal retention was not blocked'; END IF;
    denied := FALSE;
    BEGIN
        UPDATE infrastructure.provider_profile_revision
           SET host_cpu_millicores = host_cpu_millicores + 1;
    EXCEPTION WHEN raise_exception THEN denied := TRUE;
    END;
    IF NOT denied THEN RAISE EXCEPTION 'Provider profile content rewrite was not blocked'; END IF;
    denied := FALSE;
    BEGIN
        DELETE FROM infrastructure.provider_profile_revision;
    EXCEPTION WHEN raise_exception THEN denied := TRUE;
    END;
    IF NOT denied THEN RAISE EXCEPTION 'Provider profile physical delete was not blocked'; END IF;
    denied := FALSE;
    BEGIN
        UPDATE infrastructure.provider_profile_revision_event SET event_type = 'superseded';
    EXCEPTION WHEN raise_exception THEN denied := TRUE;
    END;
    IF NOT denied THEN RAISE EXCEPTION 'Provider profile Transaction rewrite was not blocked'; END IF;
    denied := FALSE;
    BEGIN
        TRUNCATE TABLE infrastructure.provider_profile_revision_event;
    EXCEPTION WHEN raise_exception THEN denied := TRUE;
    END;
    IF NOT denied THEN RAISE EXCEPTION 'Provider profile Transaction truncate was not blocked'; END IF;
    UPDATE infrastructure.provider_profile_revision
       SET valid_to = transaction_timestamp()
     WHERE tenant_id = '10000000-0000-4000-8000-000000000001'
       AND profile_id = '15000000-0000-4000-8000-000000000001' AND revision = 1;
    denied := FALSE;
    BEGIN
        UPDATE infrastructure.provider_profile_revision SET valid_to = NULL;
    EXCEPTION WHEN raise_exception THEN denied := TRUE;
    END;
    IF NOT denied THEN RAISE EXCEPTION 'Closed provider profile revision was reopened'; END IF;
    IF (SELECT count(*) FROM infrastructure.provider_profile_revision_event
        WHERE tenant_id = '10000000-0000-4000-8000-000000000001'
          AND profile_id = '15000000-0000-4000-8000-000000000001') <> 2 THEN
        RAISE EXCEPTION 'Provider profile revision events do not record create and supersede';
    END IF;
END $$;
ROLLBACK;
"""

PHYSICAL_MUTATION_ASSERTIONS_SQL = """\
BEGIN;
SELECT set_config('app.actor_id', '12000000-0000-4000-8000-000000000002', true);
DO $$
DECLARE denied BOOLEAN;
BEGIN
    denied := FALSE;
    BEGIN
        UPDATE infrastructure.capacity_physical_reservation
           SET host_memory_bytes = host_memory_bytes + 1;
    EXCEPTION WHEN raise_exception THEN denied := TRUE;
    END;
    IF NOT denied THEN RAISE EXCEPTION 'physical reservation identity/resource rewrite was not blocked'; END IF;

    denied := FALSE;
    BEGIN
        UPDATE infrastructure.capacity_physical_reservation_event SET event_type = 'released';
    EXCEPTION WHEN raise_exception THEN denied := TRUE;
    END;
    IF NOT denied THEN RAISE EXCEPTION 'physical capacity Transaction rewrite was not blocked'; END IF;

    denied := FALSE;
    BEGIN
        DELETE FROM infrastructure.capacity_physical_reservation_event;
    EXCEPTION WHEN raise_exception THEN denied := TRUE;
    END;
    IF NOT denied THEN RAISE EXCEPTION 'physical capacity Transaction delete was not blocked'; END IF;

    denied := FALSE;
    BEGIN
        TRUNCATE TABLE infrastructure.capacity_physical_reservation_event;
    EXCEPTION WHEN raise_exception THEN denied := TRUE;
    END;
    IF NOT denied THEN RAISE EXCEPTION 'physical capacity Transaction truncate was not blocked'; END IF;

    denied := FALSE;
    BEGIN
        DELETE FROM infrastructure.capacity_physical_reservation;
    EXCEPTION WHEN raise_exception THEN denied := TRUE;
    END;
    IF NOT denied THEN RAISE EXCEPTION 'active physical capacity reservation cleanup was not blocked'; END IF;

    UPDATE infrastructure.capacity_physical_reservation
       SET reservation_state = 'released',
           terminal_at = clock_timestamp(),
           retention_expires_at = clock_timestamp() + interval '24 hours',
           updated_at = clock_timestamp();
    IF NOT FOUND THEN RAISE EXCEPTION 'valid physical reservation release transition was rejected'; END IF;
    IF (SELECT count(*) FROM infrastructure.capacity_physical_reservation_event
        WHERE reservation_bundle_id = 'd0000000-0000-4000-8000-000000000001') <> 2
       OR (SELECT count(*) FROM infrastructure.capacity_physical_reservation_event
           WHERE reservation_bundle_id = 'd0000000-0000-4000-8000-000000000001'
             AND event_type = 'released') <> 1 THEN
        RAISE EXCEPTION 'physical reservation transition was not appended atomically';
    END IF;
END $$;
ROLLBACK;
"""


def write_text(path: Path, value: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(value, encoding="utf-8")


def is_contained(path: Path, parent: Path) -> bool:
    try:
        path.resolve().relative_to(parent.resolve())
        return True
    except ValueError:
        return False


def reserve_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.bind(("127.0.0.1", 0))
        return int(sock.getsockname()[1])


def run_step(name: str, command: Sequence[str], cwd: Path, env: dict[str, str]) -> dict[str, object]:
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    stdout_path = OUTPUT_DIR / f"{name}.stdout.log"
    stderr_path = OUTPUT_DIR / f"{name}.stderr.log"
    try:
        with stdout_path.open("w", encoding="utf-8") as stdout, stderr_path.open(
            "w", encoding="utf-8"
        ) as stderr:
            result = subprocess.run(
                list(command),
                cwd=cwd,
                env=env,
                stdout=stdout,
                stderr=stderr,
                check=False,
                timeout=MAX_STEP_SECONDS,
                creationflags=subprocess.CREATE_NO_WINDOW if sys.platform == "win32" else 0,
            )
        return {
            "name": name,
            "status": "passed" if result.returncode == 0 else "failed",
            "exit_code": result.returncode,
            "command": list(command),
            "stdout_log": str(stdout_path),
            "stderr_log": str(stderr_path),
        }
    except subprocess.TimeoutExpired:
        return {"name": name, "status": "failed", "exit_code": 124, "reason": "timeout"}
    except OSError as error:
        return {"name": name, "status": "blocked", "exit_code": 127, "reason": type(error).__name__}


def run_sql(
    name: str,
    command: Sequence[str],
    sql: str | None,
    env: dict[str, str],
    expect_success: bool = True,
) -> dict[str, object]:
    if sql is not None:
        sql_path = OUTPUT_DIR / f"{name}.sql"
        write_text(sql_path, sql)
        command = [*command, "-f", str(sql_path)]
    result = run_step(name, command, ROOT, env)
    passed = result["status"] == "passed"
    result["asserted_status"] = "passed" if passed == expect_success else "failed"
    if passed != expect_success:
        result["status"] = "failed"
        result["reason"] = "command result did not match expected outcome"
    return result


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pg-bin", type=Path, required=True, help="PostgreSQL bin directory with initdb, pg_ctl, and psql")
    parser.add_argument("--cargo", default="cargo", help="Cargo executable or command name")
    parser.add_argument("--toolchain-bin", type=Path, help="Optional Rust toolchain bin directory")
    parser.add_argument("--target-dir", type=Path, help="Optional short Cargo target directory")
    parser.add_argument("--output-dir", type=Path, help="Optional report directory")
    arguments = parser.parse_args()

    global OUTPUT_DIR
    if arguments.output_dir:
        OUTPUT_DIR = arguments.output_dir.resolve()
    pg_bin = arguments.pg_bin.resolve()
    executable_suffix = ".exe" if sys.platform == "win32" else ""
    executables = {name: pg_bin / f"{name}{executable_suffix}" for name in ("initdb", "pg_ctl", "psql")}
    missing = [str(path) for path in executables.values() if not path.is_file()]
    if missing:
        print(json.dumps({"result": "blocked", "missing_tools": missing}))
        return 2
    missing_migrations = [
        str(path)
        for path in (MIGRATION, GRANTS_MIGRATION, PROFILE_MIGRATION, GLOBAL_MIGRATION)
        if not path.is_file()
    ]
    if missing_migrations:
        print(json.dumps({"result": "blocked", "missing_migrations": missing_migrations}))
        return 2

    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    DATA_PARENT.mkdir(parents=True, exist_ok=True)
    data_dir = DATA_PARENT / f"postgres-{uuid.uuid4()}"
    if not is_contained(data_dir, DATA_PARENT) or data_dir.exists():
        print(json.dumps({"result": "blocked", "reason": "unsafe or pre-existing data directory"}))
        return 2
    port = reserve_port()
    env = {}
    import os

    env.update(os.environ)
    if arguments.toolchain_bin:
        toolchain_bin = arguments.toolchain_bin.resolve()
        env["PATH"] = str(toolchain_bin) + os.pathsep + env.get("PATH", "")
        env["RUSTC"] = str(toolchain_bin / ("rustc.exe" if os.name == "nt" else "rustc"))
        cargo = str(toolchain_bin / ("cargo.exe" if os.name == "nt" else "cargo"))
    else:
        cargo = shutil.which(arguments.cargo) or arguments.cargo
    if arguments.target_dir:
        env["CARGO_TARGET_DIR"] = str(arguments.target_dir.resolve())
    psql_base = [
        str(executables["psql"]), "-X", "-h", "127.0.0.1", "-p", str(port),
        "-U", "star_infrastructure_admin", "-d", "postgres", "-v", "ON_ERROR_STOP=1",
    ]
    results: list[dict[str, object]] = []
    server_attempted = False
    started = False
    stop_confirmed = False
    exit_code = 1
    try:
        results.append(run_step(
            "postgres-init",
            [str(executables["initdb"]), "-D", str(data_dir), "-U", "star_infrastructure_admin", "--auth=trust", "--no-instructions"],
            ROOT,
            env,
        ))
        if results[-1]["status"] == "passed":
            server_attempted = True
            results.append(run_step(
                "postgres-start",
                [str(executables["pg_ctl"]), "-D", str(data_dir), "-l", str(data_dir / "server.log"),
                 "-o", f"-h 127.0.0.1 -p {port} -c listen_addresses=127.0.0.1", "-w", "start"],
                ROOT,
                env,
            ))
            started = results[-1]["status"] == "passed"
        if started:
            migration_sql = MIGRATION.read_text(encoding="utf-8")
            grants_migration_sql = GRANTS_MIGRATION.read_text(encoding="utf-8")
            profile_migration_sql = PROFILE_MIGRATION.read_text(encoding="utf-8")
            global_migration_sql = GLOBAL_MIGRATION.read_text(encoding="utf-8")
            for name, sql in (
                ("migration-apply-1", migration_sql),
                ("migration-apply-2", migration_sql),
                ("runtime-grants-migration-apply-1", grants_migration_sql),
                ("runtime-grants-migration-apply-2", grants_migration_sql),
                ("provider-profile-migration-apply-1", profile_migration_sql),
                ("provider-profile-migration-apply-2", profile_migration_sql),
                ("physical-capacity-migration-apply-1", global_migration_sql),
                ("physical-capacity-migration-apply-2", global_migration_sql),
                ("owner-runtime-probe-role", OWNER_RUNTIME_ROLE_SQL),
                ("owner-runtime-privilege-matrix", OWNER_PRIVILEGE_ASSERTIONS_SQL),
                ("runtime-role", RUNTIME_ROLE_SQL),
                ("runtime-fixture", RUNTIME_FIXTURE_SQL),
                ("runtime-tenant-isolation", RUNTIME_ASSERTIONS_SQL),
                ("physical-capacity-rls-isolation", PHYSICAL_RLS_ASSERTIONS_SQL),
                ("immutable-and-retention-guards", MUTATION_ASSERTIONS_SQL),
                ("physical-capacity-worm-guards", PHYSICAL_MUTATION_ASSERTIONS_SQL),
            ):
                results.append(run_sql(name, psql_base, sql, env))
                if results[-1]["status"] != "passed":
                    break
        if all(item["status"] == "passed" for item in results):
            results.append(run_sql(
                "force-rls-count",
                [*psql_base, "-At", "-c", "SELECT count(*) FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname='infrastructure' AND c.relkind='r' AND c.relrowsecurity AND c.relforcerowsecurity;"],
                None,
                env,
            ))
            log_path = OUTPUT_DIR / "force-rls-count.stdout.log"
            count = log_path.read_text(encoding="utf-8").strip() if log_path.exists() else ""
            results[-1]["expected_value"] = "10"
            results[-1]["actual_value"] = count
            if count != "10":
                results[-1]["status"] = "failed"
        if all(item["status"] == "passed" for item in results):
            test_env = env.copy()
            test_env["STAR_INFRA_OWNER_TEST_DATABASE_URL"] = (
                f"postgresql://star_infrastructure_admin@127.0.0.1:{port}/postgres"
            )
            results.append(run_step(
                "owner-inbox-postgres-integration",
                [cargo, "test", "--locked", "-p", "infrastructure", "--test", "infrastructure_owner_inbox_pg", "-j", "1", "--", "--ignored"],
                ROOT,
                test_env,
            ))
        if all(item["status"] == "passed" for item in results):
            exit_code = 0
    finally:
        if data_dir.exists() and is_contained(data_dir, DATA_PARENT):
            if server_attempted:
                stop = run_step(
                    "postgres-stop",
                    [str(executables["pg_ctl"]), "-D", str(data_dir), "-m", "fast", "-w", "stop"],
                    ROOT,
                    env,
                )
                stop_confirmed = stop["status"] == "passed"
                results.append(stop)
                if not stop_confirmed:
                    exit_code = 1
            else:
                stop_confirmed = True
            if stop_confirmed and is_contained(data_dir, DATA_PARENT):
                shutil.rmtree(data_dir)
            else:
                print(json.dumps({"preserved_postgres_data": str(data_dir), "reason": "server stop could not be confirmed"}))
        audit_path = OUTPUT_DIR / "audit.jsonl"
        write_text(audit_path, "".join(json.dumps(item, sort_keys=True) + "\n" for item in results))
        print(json.dumps({"result": "passed" if all(item["status"] == "passed" for item in results) else "failed", "steps": results}, sort_keys=True))
    return exit_code


if __name__ == "__main__":
    sys.exit(main())
