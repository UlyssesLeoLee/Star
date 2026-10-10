"""Run the disposable PostgreSQL + NATS integration gate for the Run-owner consumer."""

# @cypher schema=1 source_sha256=73e027ef3134dbc1c8bf96cf2ae1ccf8c8eebaf62156fc465985f0e72cc85c76
# MERGE (self:File {path:"scripts/automation/phase9f4d_run_owner_consumer.py"})
# MERGE (main:Symbol {id:"scripts/automation/phase9f4d_run_owner_consumer.py::main",kind:"function"})
# MERGE (run_step:Symbol {id:"scripts/automation/phase9f4d_run_owner_consumer.py::run_step",kind:"function"})
# MERGE (run_sql:Symbol {id:"scripts/automation/phase9f4d_run_owner_consumer.py::run_sql",kind:"function"})
# MERGE (write_text:Symbol {id:"scripts/automation/phase9f4d_run_owner_consumer.py::write_text",kind:"function"})
# MERGE (is_contained:Symbol {id:"scripts/automation/phase9f4d_run_owner_consumer.py::is_contained",kind:"function"})
# MERGE (reserve_port:Symbol {id:"scripts/automation/phase9f4d_run_owner_consumer.py::reserve_port",kind:"function"})
# MERGE (start_nats:Symbol {id:"scripts/automation/phase9f4d_run_owner_consumer.py::start_nats_server",kind:"function"})
# MERGE (stop_nats:Symbol {id:"scripts/automation/phase9f4d_run_owner_consumer.py::stop_nats_server",kind:"function"})
# MERGE (redact_urls:Symbol {id:"scripts/automation/phase9f4d_run_owner_consumer.py::redact_urls",kind:"function"})
# MERGE (rls_fixture:Symbol {id:"scripts/automation/phase9f4d_run_owner_consumer.py::rls_fixture_sql",kind:"function"})
# MERGE (migration:Config {id:"db/migrations/2026-10-07-run-owner-infrastructure-capacity-result.sql"})
# MERGE (consumer_test:Config {id:"crates/run-owner-runtime/tests/capacity_result_jetstream_pg.rs"})
# MERGE (quarantine_migration:Config {id:"db/migrations/2026-10-08-platform-poison-quarantine.sql"})
# MERGE (quarantine_schema_test:Config {id:"db/tests/platform-poison-quarantine-schema.sql"})
# MERGE (admission_lifecycle:Config {id:"db/migrations/2026-10-09-task-run-admission-attempt-lifecycle.sql"})
# MERGE (admission_lifecycle_test:Config {id:"db/tests/task-run-admission-attempt-lifecycle.sql"})
# MERGE (run_binding:Config {id:"db/migrations/2026-10-10-multica-run-infrastructure-binding-master.sql"})
# MERGE (run_binding_test:Config {id:"db/tests/run-infrastructure-binding-master.sql"})
# MERGE (binding_sqlx_test:Config {id:"crates/run-owner-runtime/tests/run_infrastructure_binding_pg.rs"})
# MERGE (admission_migration:Config {id:"db/migrations/2026-10-10-run-owner-schedule-admission.sql"})
# MERGE (admission_test:Config {id:"crates/run-owner-runtime/tests/schedule_admission_pg.rs"})
# MERGE (currentness:Config {id:"db/migrations/2026-10-10-run-owner-admission-currentness.sql"})
# MERGE (acl_lock:Config {id:"db/migrations/2026-10-05-schedule-run-as-acl-lock.sql"})
# MERGE (legacy_audit_fixture:Config {id:"disposable-test-only::public.audit_trigger_func"})
# MERGE (postgres:ExternalService {id:"postgres.initdb_pg_ctl_psql",kind:"command"})
# MERGE (nats:ExternalService {id:"nats-server.jetstream",kind:"command"})
# MERGE (cargo:ExternalService {id:"cargo.test",kind:"command"})
# MERGE (subprocess:ExternalService {id:"python.subprocess",kind:"module"})
# MERGE (socket:ExternalService {id:"python.socket",kind:"module"})
# MERGE (time:ExternalService {id:"python.time",kind:"module"})
# MERGE (uuid:ExternalService {id:"python.uuid",kind:"module"})
# MERGE (os:ExternalService {id:"python.os",kind:"module"})
# MERGE (shutil:ExternalService {id:"python.shutil",kind:"module"})
# MERGE (argparse:ExternalService {id:"python.argparse",kind:"module"})
# MERGE (json:ExternalService {id:"python.json",kind:"module"})
# MERGE (re:ExternalService {id:"python.re",kind:"module"})
# MERGE (sys:ExternalService {id:"python.sys",kind:"module"})
# MERGE (pathlib:ExternalService {id:"python.pathlib",kind:"module"})
# MERGE (typing:ExternalService {id:"python.typing",kind:"module"})
# MERGE (self)-[:DEFINES]->(main)
# MERGE (self)-[:DEFINES]->(run_step)
# MERGE (self)-[:DEFINES]->(run_sql)
# MERGE (self)-[:DEFINES]->(write_text)
# MERGE (self)-[:DEFINES]->(is_contained)
# MERGE (self)-[:DEFINES]->(reserve_port)
# MERGE (self)-[:DEFINES]->(start_nats)
# MERGE (self)-[:DEFINES]->(stop_nats)
# MERGE (main)-[:CALLS]->(run_step)
# MERGE (main)-[:CALLS]->(run_sql)
# MERGE (main)-[:CALLS]->(write_text)
# MERGE (main)-[:CALLS]->(is_contained)
# MERGE (main)-[:CALLS]->(reserve_port)
# MERGE (main)-[:CALLS]->(start_nats)
# MERGE (main)-[:CALLS]->(stop_nats)
# MERGE (main)-[:CALLS]->(rls_fixture)
# MERGE (main)-[:CALLS]->(redact_urls)
# MERGE (main)-[:CONFIGURES]->(postgres)
# MERGE (main)-[:CONFIGURES]->(nats)
# MERGE (main)-[:CONFIGURES]->(cargo)
# MERGE (main)-[:READS]->(migration)
# MERGE (main)-[:READS]->(quarantine_migration)
# MERGE (main)-[:READS]->(quarantine_schema_test)
# MERGE (main)-[:READS]->(admission_lifecycle)
# MERGE (main)-[:TESTS]->(admission_lifecycle_test)
# MERGE (main)-[:READS]->(run_binding)
# MERGE (main)-[:TESTS]->(run_binding_test)
# MERGE (main)-[:TESTS]->(binding_sqlx_test)
# MERGE (main)-[:READS]->(admission_migration)
# MERGE (main)-[:TESTS]->(admission_test)
# MERGE (main)-[:READS]->(currentness)
# MERGE (main)-[:READS]->(catalog_currentness:Config {id:"db/migrations/2026-10-10-run-owner-catalog-currentness.sql"})
# MERGE (main)-[:READS]->(policy_currentness:Config {id:"db/migrations/2026-10-10-run-owner-policy-currentness.sql"})
# MERGE (main)-[:READS]->(task_currentness:Config {id:"db/migrations/2026-10-10-run-owner-task-currentness.sql"})
# MERGE (main)-[:READS]->(acl_lock)
# MERGE (main)-[:USES_TEST_FIXTURE]->(legacy_audit_fixture)
# MERGE (main)-[:TESTS]->(consumer_test)
# MERGE (run_sql)-[:CALLS]->(run_step)
# MERGE (run_sql)-[:CALLS]->(write_text)
# MERGE (start_nats)-[:CALLS]->(subprocess)
# MERGE (start_nats)-[:CALLS]->(socket)
# MERGE (start_nats)-[:CALLS]->(time)
# MERGE (start_nats)-[:CALLS]->(json)
# MERGE (start_nats)-[:CALLS]->(redact_urls)
# MERGE (stop_nats)-[:CALLS]->(subprocess)
# MERGE (run_step)-[:CALLS]->(subprocess)
# MERGE (run_step)-[:CALLS]->(redact_urls)
# MERGE (reserve_port)-[:CALLS]->(socket)
# MERGE (main)-[:USES]->(uuid)
# MERGE (main)-[:USES]->(os)
# MERGE (main)-[:USES]->(shutil)
# MERGE (main)-[:USES]->(argparse)
# MERGE (main)-[:USES]->(json)
# MERGE (self)-[:IMPORTS]->(subprocess)
# MERGE (self)-[:IMPORTS]->(socket)
# MERGE (self)-[:IMPORTS]->(time)
# MERGE (self)-[:IMPORTS]->(uuid)
# MERGE (self)-[:IMPORTS]->(os)
# MERGE (self)-[:IMPORTS]->(shutil)
# MERGE (self)-[:IMPORTS]->(argparse)
# MERGE (self)-[:IMPORTS]->(json)
# MERGE (self)-[:IMPORTS]->(re)
# MERGE (self)-[:IMPORTS]->(sys)
# MERGE (self)-[:IMPORTS]->(pathlib)
# MERGE (self)-[:IMPORTS]->(typing)
# MERGE (main)-[:READS]->(capacity_migration:Config {id:"db/migrations/2026-10-10-run-owner-project-capacity.sql"})
# MERGE (main)-[:READS]->(preparation_migration:Config {id:"db/migrations/2026-10-10-run-owner-schedule-preparation.sql"})
# @endcypher

from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import socket
import subprocess
import sys
import time
import uuid
from pathlib import Path
from typing import Sequence

ROOT = Path(__file__).resolve().parents[2]
RUN_MIGRATION = ROOT / "db" / "migrations" / "2026-10-07-run-owner-infrastructure-capacity-result.sql"
ADMISSION_LIFECYCLE_MIGRATION = ROOT / "db" / "migrations" / "2026-10-09-task-run-admission-attempt-lifecycle.sql"
RUN_BINDING_MIGRATION = ROOT / "db" / "migrations" / "2026-10-10-multica-run-infrastructure-binding-master.sql"
SCHEDULE_ADMISSION_MIGRATION = ROOT / "db" / "migrations" / "2026-10-10-run-owner-schedule-admission.sql"
SCHEDULE_PREPARATION_MIGRATION = ROOT / "db" / "migrations" / "2026-10-10-run-owner-schedule-preparation.sql"
PROJECT_CAPACITY_MIGRATION = ROOT / "db" / "migrations" / "2026-10-10-run-owner-project-capacity.sql"
CATALOG_CURRENTNESS_MIGRATION = ROOT / "db" / "migrations" / "2026-10-10-run-owner-catalog-currentness.sql"
POLICY_CURRENTNESS_MIGRATION = ROOT / "db" / "migrations" / "2026-10-10-run-owner-policy-currentness.sql"
TASK_CURRENTNESS_MIGRATION = ROOT / "db" / "migrations" / "2026-10-10-run-owner-task-currentness.sql"
ADMISSION_CURRENTNESS_MIGRATION = ROOT / "db" / "migrations" / "2026-10-10-run-owner-admission-currentness.sql"
QUARANTINE_SCHEMA_TEST = ROOT / "db" / "tests" / "platform-poison-quarantine-schema.sql"
ADMISSION_LIFECYCLE_TEST = ROOT / "db" / "tests" / "task-run-admission-attempt-lifecycle.sql"
RUN_BINDING_TEST = ROOT / "db" / "tests" / "run-infrastructure-binding-master.sql"
PREREQUISITE_MIGRATIONS = (
    ROOT / "db" / "migrations" / "2026-09-16-worktree-canvas-worktree.sql",
    ROOT / "db" / "migrations" / "2026-09-29-worktree-group-phase-2b.sql",
    ROOT / "db" / "migrations" / "2026-09-29-worktree-group-phase-2c.sql",
    ROOT / "db" / "migrations" / "2026-10-01-engineering-run-directory.sql",
    ROOT / "db" / "migrations" / "2026-10-02-task-run-owner.sql",
    ROOT / "db" / "migrations" / "2026-09-30-multica-hook-policy.sql",
    ROOT / "db" / "migrations" / "2026-10-01-multica-agent-execution-profile.sql",
    ROOT / "db" / "migrations" / "2026-10-01-multica-agent-execution-catalog.sql",
    ROOT / "db" / "migrations" / "2026-10-05-schedule-run-as-acl-lock.sql",
    ROOT / "db" / "migrations" / "2026-09-30-worktree-task-execution-run.sql",
    ROOT / "db" / "migrations" / "2026-10-01-multica-run-resource-reservations.sql",
    ROOT / "db" / "migrations" / "2026-10-02-task-run-engineering-run-binding.sql",
    ROOT / "db" / "migrations" / "2026-10-07-infrastructure-capacity-owner-schema.sql",
    ROOT / "db" / "migrations" / "2026-10-08-infrastructure-owner-runtime-grants.sql",
    ROOT / "db" / "migrations" / "2026-10-08-platform-poison-quarantine.sql",
    ROOT / "db" / "migrations" / "2026-10-09-infrastructure-provider-profile-master.sql",
    ROOT / "db" / "migrations" / "2026-10-10-infrastructure-physical-capacity-ledger.sql",
)
LEGACY_AUDIT_PREREQUISITE_SQL = """\
CREATE OR REPLACE FUNCTION public.audit_trigger_func()
RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RETURN COALESCE(NEW, OLD); END $$;"""
OUTPUT_ROOT = ROOT / "target" / "phase9f4d-run-owner-consumer"
OUTPUT_DIR = OUTPUT_ROOT
DATA_PARENT = ROOT / ".cache" / "phase9f4d-run-owner-consumer"
MAX_STEP_SECONDS = 900

SCHEMA_ASSERTIONS_SQL = """\
DO $assertions$
DECLARE
    table_name TEXT;
    role_name TEXT;
    role_is_restricted BOOLEAN;
    protected_tables TEXT[] := ARRAY[
        'multica.infrastructure_capacity_demand_outbox',
        'multica.infrastructure_capacity_result_inbox',
        'multica.schedule_admission_inbox',
        'multica.schedule_admission_result_outbox'
    ];
BEGIN
    SELECT NOT rolcanlogin AND NOT rolsuper AND NOT rolcreatedb AND NOT rolcreaterole
           AND NOT rolreplication AND NOT rolbypassrls AND NOT rolinherit
      INTO role_is_restricted
      FROM pg_roles
     WHERE rolname = 'star_multica_run_owner';
    IF role_is_restricted IS DISTINCT FROM TRUE THEN
        RAISE EXCEPTION 'Run owner role is missing or has unsafe role attributes';
    END IF;
    IF NOT has_schema_privilege('star_multica_run_owner', 'multica', 'USAGE')
       OR has_schema_privilege('star_multica_run_owner', 'multica', 'CREATE') THEN
        RAISE EXCEPTION 'Run owner schema privileges are not least-privilege';
    END IF;
    IF NOT has_function_privilege('star_multica_run_owner',
        'multica.verify_schedule_run_as_current(uuid,uuid,uuid,uuid,uuid,uuid)', 'EXECUTE')
       OR NOT EXISTS (SELECT 1 FROM pg_proc WHERE oid =
        'multica.verify_schedule_run_as_current(uuid,uuid,uuid,uuid,uuid,uuid)'::regprocedure
        AND prosecdef AND proconfig @> ARRAY['search_path=pg_catalog'])
       OR EXISTS (SELECT 1 FROM pg_proc p, LATERAL aclexplode(p.proacl) a
        WHERE p.oid='multica.verify_schedule_run_as_current(uuid,uuid,uuid,uuid,uuid,uuid)'::regprocedure
          AND a.grantee=0 AND a.privilege_type='EXECUTE') THEN
        RAISE EXCEPTION 'Native currentness function must be fixed-path owner-only execution';
    END IF;
    FOREACH table_name IN ARRAY ARRAY[
      'permission.project_role_binding','permission.cloud_branch_role_binding',
      'permission.engineering_run_role_binding','multica.engineering_run_revision',
      'scm.cloud_branch_revision','public.worktree_canvas_worktree',
      'multica.worktree_project_binding','multica.engineering_run_worktree_binding',
      'multica.task_metadata','multica.work_item_worktree',
      'multica.agent_execution_profile','multica.hook_policy_set',
      'multica.agent_execution_catalog_revision','multica.agent_execution_provider_catalog',
      'multica.agent_execution_provider_capability','multica.agent_execution_skill_catalog',
      'multica.agent_execution_skill_capability','multica.agent_execution_grant_set',
      'multica.agent_execution_grant_capability'
    ] LOOP
        IF has_table_privilege('star_multica_run_owner',table_name,'INSERT')
           OR has_any_column_privilege('star_multica_run_owner',table_name,'UPDATE')
           OR has_table_privilege('star_multica_run_owner',table_name,'DELETE') THEN
            RAISE EXCEPTION 'Native reader must not grant directory/ACL mutation to runtime';
        END IF;
    END LOOP;

    IF NOT has_function_privilege('star_multica_run_owner',
        'multica.verify_schedule_task_current(uuid,uuid,uuid,uuid,uuid,uuid,uuid,uuid)', 'EXECUTE')
       OR NOT EXISTS (SELECT 1 FROM pg_proc WHERE oid =
        'multica.verify_schedule_task_current(uuid,uuid,uuid,uuid,uuid,uuid,uuid,uuid)'::regprocedure
        AND prosecdef AND proconfig @> ARRAY['search_path=pg_catalog'])
       OR EXISTS (SELECT 1 FROM pg_proc p, LATERAL aclexplode(p.proacl) a
        WHERE p.oid='multica.verify_schedule_task_current(uuid,uuid,uuid,uuid,uuid,uuid,uuid,uuid)'::regprocedure
          AND a.grantee=0 AND a.privilege_type='EXECUTE') THEN
        RAISE EXCEPTION 'Task currentness must be fixed-path owner-only execution';
    END IF;

    IF NOT has_function_privilege('star_multica_run_owner',
       'multica.read_schedule_policies_current(uuid,uuid,uuid,uuid,uuid,uuid,uuid,uuid,uuid)','EXECUTE')
       OR NOT EXISTS (SELECT 1 FROM pg_proc WHERE oid =
        'multica.read_schedule_policies_current(uuid,uuid,uuid,uuid,uuid,uuid,uuid,uuid,uuid)'::regprocedure
        AND prosecdef AND proconfig @> ARRAY['search_path=pg_catalog'])
       OR EXISTS (SELECT 1 FROM pg_proc p, LATERAL aclexplode(p.proacl) a
        WHERE p.oid='multica.read_schedule_policies_current(uuid,uuid,uuid,uuid,uuid,uuid,uuid,uuid,uuid)'::regprocedure
        AND a.grantee=0 AND a.privilege_type='EXECUTE')
       OR NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgrelid='multica.hook_policy_set'::regclass
        AND tgname='hook_policy_00_publication_lock' AND tgenabled='O') THEN
        RAISE EXCEPTION 'Policy reader/publication serialization capability invalid';
    END IF;

    IF NOT has_function_privilege('star_multica_run_owner',
      'multica.read_schedule_catalog_current(uuid,uuid,uuid,uuid,uuid,uuid,uuid,uuid,text[],text[])','EXECUTE')
      OR NOT EXISTS(SELECT 1 FROM pg_proc WHERE oid=
        'multica.read_schedule_catalog_current(uuid,uuid,uuid,uuid,uuid,uuid,uuid,uuid,text[],text[])'::regprocedure
        AND prosecdef AND proconfig @> ARRAY['search_path=pg_catalog'])
      OR EXISTS(SELECT 1 FROM pg_proc p,LATERAL aclexplode(p.proacl) a WHERE p.oid=
        'multica.read_schedule_catalog_current(uuid,uuid,uuid,uuid,uuid,uuid,uuid,uuid,text[],text[])'::regprocedure
        AND a.grantee=0 AND a.privilege_type='EXECUTE')
      OR (SELECT count(*) FROM pg_trigger WHERE tgname='agent_execution_00_publication_lock' AND tgenabled='O'
        AND tgrelid IN ('multica.agent_execution_provider_catalog'::regclass,'multica.agent_execution_skill_catalog'::regclass,'multica.agent_execution_grant_set'::regclass))<>3
      OR position('pg_advisory_xact_lock_shared' IN pg_get_functiondef('multica.verify_schedule_run_as_current(uuid,uuid,uuid,uuid,uuid,uuid)'::regprocedure))=0 THEN
       RAISE EXCEPTION 'Catalog/parallel authorization capabilities invalid';
    END IF;

    IF NOT has_function_privilege('star_multica_run_owner',
      'multica.read_schedule_project_capacity_current(uuid,uuid,uuid,uuid,uuid,uuid,uuid,uuid)','EXECUTE')
      OR NOT EXISTS(SELECT 1 FROM pg_proc WHERE oid=
        'multica.read_schedule_project_capacity_current(uuid,uuid,uuid,uuid,uuid,uuid,uuid,uuid)'::regprocedure
        AND prosecdef AND proconfig @> ARRAY['search_path=pg_catalog'])
      OR EXISTS(SELECT 1 FROM pg_proc p,LATERAL aclexplode(p.proacl) a WHERE p.oid=
        'multica.read_schedule_project_capacity_current(uuid,uuid,uuid,uuid,uuid,uuid,uuid,uuid)'::regprocedure
        AND a.grantee=0 AND a.privilege_type='EXECUTE') THEN
       RAISE EXCEPTION 'Project capacity native capability invalid';
    END IF;
    FOREACH table_name IN ARRAY ARRAY['project_execution_resource_admission_lock',
      'project_execution_resource_quota','project_execution_resource_reservation'] LOOP
      IF has_table_privilege('star_multica_run_owner','multica.' || table_name,'INSERT')
        OR has_any_column_privilege('star_multica_run_owner','multica.' || table_name,'UPDATE')
        OR has_table_privilege('star_multica_run_owner','multica.' || table_name,'DELETE') THEN
         RAISE EXCEPTION 'Runtime must not directly mutate Project capacity %',table_name;
      END IF;
    END LOOP;

    IF NOT has_function_privilege('star_multica_run_owner','multica.prepare_schedule_execution_run(bytea,text,uuid,uuid,uuid)','EXECUTE')
      OR NOT EXISTS(SELECT 1 FROM pg_proc WHERE oid='multica.prepare_schedule_execution_run(bytea,text,uuid,uuid,uuid)'::regprocedure
        AND prosecdef AND proconfig @> ARRAY['search_path=pg_catalog'])
      OR EXISTS(SELECT 1 FROM pg_proc p,LATERAL aclexplode(p.proacl) a WHERE p.oid='multica.prepare_schedule_execution_run(bytea,text,uuid,uuid,uuid)'::regprocedure
        AND a.grantee=0 AND a.privilege_type='EXECUTE') THEN RAISE EXCEPTION 'Preparation native capability invalid'; END IF;
    FOREACH table_name IN ARRAY ARRAY['task_contract','task_lifecycle_current'] LOOP
      IF has_table_privilege('star_multica_run_owner','multica.' || table_name,'INSERT')
      OR has_any_column_privilege('star_multica_run_owner','multica.' || table_name,'UPDATE')
      OR has_table_privilege('star_multica_run_owner','multica.' || table_name,'DELETE') THEN
       RAISE EXCEPTION 'Preparation must not grant Task owner mutation %',table_name;
      END IF;
    END LOOP;

    FOREACH table_name IN ARRAY protected_tables LOOP
        IF NOT has_table_privilege('star_multica_run_owner', table_name, 'SELECT')
           OR NOT has_table_privilege('star_multica_run_owner', table_name, 'INSERT')
           OR has_table_privilege('star_multica_run_owner', table_name, 'UPDATE')
           OR has_table_privilege('star_multica_run_owner', table_name, 'DELETE')
           OR has_table_privilege('star_multica_run_owner', table_name, 'TRUNCATE')
           OR has_table_privilege('star_multica_run_owner', table_name, 'REFERENCES')
           OR has_table_privilege('star_multica_run_owner', table_name, 'TRIGGER') THEN
            RAISE EXCEPTION 'Run owner Transaction table privileges are not append-only';
        END IF;
    END LOOP;

    IF NOT has_column_privilege('star_multica_run_owner', 'multica.task_execution_run', 'tenant_id', 'SELECT')
       OR NOT has_column_privilege('star_multica_run_owner', 'multica.task_execution_run', 'project_id', 'SELECT')
       OR NOT has_column_privilege('star_multica_run_owner', 'multica.task_execution_run', 'work_item_id', 'SELECT')
       OR NOT has_column_privilege('star_multica_run_owner', 'multica.task_execution_run', 'run_id', 'SELECT')
       OR NOT has_column_privilege('star_multica_run_owner', 'multica.task_execution_run', 'automation_occurrence_id', 'SELECT')
        OR has_column_privilege('star_multica_run_owner', 'multica.task_execution_run', 'task_snapshot', 'SELECT')
       OR NOT has_column_privilege('star_multica_run_owner', 'multica.task_execution_run', 'automation_admission_attempt', 'SELECT')
       OR NOT has_column_privilege('star_multica_run_owner', 'multica.task_execution_run', 'started_at', 'SELECT')
       OR NOT has_column_privilege('star_multica_run_owner', 'multica.task_execution_run', 'initiated_by', 'SELECT')
       OR NOT has_column_privilege('star_multica_run_owner', 'multica.task_execution_run', 'worktree_id', 'SELECT')
       OR has_table_privilege('star_multica_run_owner', 'multica.task_execution_run', 'INSERT')
       OR has_table_privilege('star_multica_run_owner', 'multica.task_execution_run', 'UPDATE')
       OR has_table_privilege('star_multica_run_owner', 'multica.task_execution_run', 'DELETE') THEN
        RAISE EXCEPTION 'Run owner TaskExecutionRun access is not limited to identity lookup columns';
    END IF;
    IF has_table_privilege('star_multica_run_owner', 'multica.task_execution_run_event', 'SELECT')
       OR has_table_privilege('star_multica_run_owner', 'multica.task_execution_run_event', 'INSERT')
       OR has_table_privilege('star_multica_run_owner', 'multica.task_execution_run_idempotency', 'SELECT')
       OR has_table_privilege('star_multica_run_owner', 'multica.task_execution_run_idempotency', 'INSERT') THEN
        RAISE EXCEPTION 'Run owner role received unrelated Multica table privileges';
    END IF;
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
         WHERE conrelid = 'multica.task_execution_run_event'::regclass
           AND conname = 'ck_task_execution_run_event_type_v3'
           AND contype = 'c'
    ) THEN
        RAISE EXCEPTION 'Run event vocabulary must include explicit admission confirmation and compensation';
    END IF;
    IF (SELECT count(*) FROM pg_class AS c JOIN pg_namespace AS n ON n.oid = c.relnamespace
        WHERE n.nspname = 'multica' AND c.relname IN ('schedule_admission_inbox','schedule_admission_result_outbox')
          AND c.relrowsecurity AND c.relforcerowsecurity) <> 2
       OR (SELECT count(*) FROM pg_policies WHERE schemaname = 'multica'
        AND tablename IN ('schedule_admission_inbox','schedule_admission_result_outbox')
        AND qual ILIKE '%app.tenant_id%' AND with_check ILIKE '%app.tenant_id%') <> 2 THEN
        RAISE EXCEPTION 'Admission facts require FORCE tenant RLS';
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint
        WHERE conrelid = 'multica.schedule_admission_inbox'::regclass
          AND conname = 'schedule_admission_paired_result' AND condeferrable AND condeferred)
       OR (SELECT count(*) FROM pg_trigger WHERE tgname = 'schedule_admission_worm'
        AND tgrelid IN ('multica.schedule_admission_inbox'::regclass,'multica.schedule_admission_result_outbox'::regclass)
        AND NOT tgisinternal) <> 2 THEN
        RAISE EXCEPTION 'Admission pair commit/append-only guards are incomplete';
    END IF;

    IF (SELECT count(*) FROM pg_class AS c
          JOIN pg_namespace AS n ON n.oid = c.relnamespace
         WHERE n.nspname = 'multica'
           AND c.relname IN ('infrastructure_capacity_demand_outbox', 'infrastructure_capacity_result_inbox')
           AND c.relkind = 'r' AND c.relrowsecurity AND c.relforcerowsecurity) <> 2 THEN
        RAISE EXCEPTION 'Both Run owner fact tables require ENABLE and FORCE ROW LEVEL SECURITY';
    END IF;
    IF (SELECT count(*) FROM pg_policies
         WHERE schemaname = 'multica'
           AND tablename IN ('infrastructure_capacity_demand_outbox', 'infrastructure_capacity_result_inbox')
           AND qual ILIKE '%app.tenant_id%'
           AND with_check ILIKE '%app.tenant_id%') <> 2 THEN
        RAISE EXCEPTION 'Both Run owner fact tables require tenant-scoped RLS policies';
    END IF;
    IF (SELECT count(*) FROM pg_trigger AS t
          JOIN pg_class AS c ON c.oid = t.tgrelid
          JOIN pg_namespace AS n ON n.oid = c.relnamespace
         WHERE n.nspname = 'multica' AND NOT t.tgisinternal
           AND t.tgname IN (
               'run_capacity_demand_outbox_worm', 'run_capacity_demand_outbox_truncate_guard',
               'run_capacity_result_inbox_worm', 'run_capacity_result_inbox_truncate_guard'
           )) <> 4 THEN
        RAISE EXCEPTION 'Append-only row and truncate guards are incomplete';
    END IF;
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
         WHERE conrelid = 'multica.infrastructure_capacity_demand_outbox'::regclass
           AND contype = 'f' AND confrelid = 'multica.task_execution_run'::regclass
    ) OR NOT EXISTS (
        SELECT 1 FROM pg_constraint
         WHERE conrelid = 'multica.infrastructure_capacity_result_inbox'::regclass
           AND contype = 'f' AND confrelid = 'multica.infrastructure_capacity_demand_outbox'::regclass
    ) THEN
        RAISE EXCEPTION 'TaskExecutionRun and source-demand composite foreign keys are required';
    END IF;
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
         WHERE conrelid = 'multica.infrastructure_capacity_result_inbox'::regclass
           AND conname = 'uq_run_capacity_result_inbox_event_id'
           AND contype = 'u'
    ) THEN
        RAISE EXCEPTION 'Infrastructure result event_id must be globally unique across tenants';
    END IF;

    FOREACH role_name IN ARRAY ARRAY['star_infrastructure_owner', 'star_automation_owner'] LOOP
        IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = role_name) THEN
            FOREACH table_name IN ARRAY protected_tables LOOP
                IF has_table_privilege(role_name, table_name, 'SELECT')
                   OR has_table_privilege(role_name, table_name, 'INSERT')
                   OR has_table_privilege(role_name, table_name, 'UPDATE')
                   OR has_table_privilege(role_name, table_name, 'DELETE') THEN
                    RAISE EXCEPTION 'Infrastructure/Automation owner received Run table privileges';
                END IF;
            END LOOP;
        END IF;
    END LOOP;
END
$assertions$;
"""


def rls_fixture_sql() -> tuple[str, str, str, dict[str, str]]:
    def new_id() -> str:
        return str(uuid.uuid4())

    scopes: dict[str, dict[str, str]] = {}
    for label in ("a", "b"):
        scopes[label] = {
            "tenant_id": new_id(),
            "project_id": new_id(),
            "work_item_id": new_id(),
            "branch_id": new_id(),
            "engineering_run_id": new_id(),
            "task_execution_run_id": new_id(),
            "occurrence_id": new_id(),
            "correlation_id": new_id(),
            "initiated_by": new_id(),
            "source_event_id": new_id(),
        }
    scopes["a"]["result_event_id"] = new_id()
    scopes["b"]["result_event_id"] = new_id()

    setup: list[str] = ["BEGIN;"]
    for label in ("a", "b"):
        scope = scopes[label]
        setup.append(
            "INSERT INTO multica.task_execution_run (run_id, tenant_id, project_id, work_item_id, "
            "initiated_by, execution_channel, task_snapshot, automation_occurrence_id, correlation_id, started_at) VALUES "
            f"('{scope['task_execution_run_id']}', '{scope['tenant_id']}', '{scope['project_id']}', "
            f"'{scope['work_item_id']}', '{scope['initiated_by']}', 'agent', '{{}}'::jsonb, "
            f"'{scope['occurrence_id']}', '{scope['correlation_id']}', clock_timestamp());"
        )
        setup.append(
            "INSERT INTO multica.infrastructure_capacity_demand_outbox (event_id, tenant_id, project_id, "
            "work_item_id, branch_id, engineering_run_id, task_execution_run_id, occurrence_id, "
            "schema_version, idempotency_key, correlation_id, payload) VALUES "
            f"('{scope['source_event_id']}', '{scope['tenant_id']}', '{scope['project_id']}', "
            f"'{scope['work_item_id']}', '{scope['branch_id']}', '{scope['engineering_run_id']}', "
            f"'{scope['task_execution_run_id']}', '{scope['occurrence_id']}', 2, "
            f"'runner:{label}:{scope['source_event_id']}', '{scope['correlation_id']}', '{{}}'::jsonb);"
        )

    scope_a = scopes["a"]
    setup.extend([
        f"SET LOCAL app.tenant_id = '{scope_a['tenant_id']}';",
        "SET LOCAL ROLE star_multica_run_owner;",
        "INSERT INTO multica.infrastructure_capacity_result_inbox (tenant_id, event_id, source_event_id, "
        "project_id, work_item_id, branch_id, engineering_run_id, task_execution_run_id, occurrence_id, "
        "event_type, schema_version, correlation_id, idempotency_key, payload_digest, payload) VALUES "
        f"('{scope_a['tenant_id']}', '{scope_a['result_event_id']}', '{scope_a['source_event_id']}', "
        f"'{scope_a['project_id']}', '{scope_a['work_item_id']}', '{scope_a['branch_id']}', "
        f"'{scope_a['engineering_run_id']}', '{scope_a['task_execution_run_id']}', '{scope_a['occurrence_id']}', "
        f"'capacity_accepted', 2, '{scope_a['correlation_id']}', 'runner:result:{scope_a['result_event_id']}', "
        "decode(repeat('a', 64), 'hex'), '{\"fixture\":true}'::jsonb);",
        "DO $visible$ BEGIN IF (SELECT count(*) FROM multica.infrastructure_capacity_result_inbox) <> 1 "
        "THEN RAISE EXCEPTION 'Run owner cannot read its tenant result Inbox row'; END IF; END $visible$;",
        "COMMIT;",
    ])

    scope_b = scopes["b"]
    visibility = "\n".join([
        "BEGIN;",
        f"SET LOCAL app.tenant_id = '{scope_b['tenant_id']}';",
        "SET LOCAL ROLE star_multica_run_owner;",
        "DO $hidden$ BEGIN IF EXISTS (SELECT 1 FROM multica.infrastructure_capacity_result_inbox "
        f"WHERE event_id = '{scope_a['result_event_id']}') THEN "
        "RAISE EXCEPTION 'Run owner tenant RLS exposed another tenant result row'; END IF; END $hidden$;",
        "COMMIT;",
    ])

    collision = "\n".join([
        "BEGIN;",
        f"SET LOCAL app.tenant_id = '{scope_b['tenant_id']}';",
        "SET LOCAL ROLE star_multica_run_owner;",
        "INSERT INTO multica.infrastructure_capacity_result_inbox (tenant_id, event_id, source_event_id, "
        "project_id, work_item_id, branch_id, engineering_run_id, task_execution_run_id, occurrence_id, "
        "event_type, schema_version, correlation_id, idempotency_key, payload_digest, payload) VALUES "
        f"('{scope_b['tenant_id']}', '{scope_a['result_event_id']}', '{scope_b['source_event_id']}', "
        f"'{scope_b['project_id']}', '{scope_b['work_item_id']}', '{scope_b['branch_id']}', "
        f"'{scope_b['engineering_run_id']}', '{scope_b['task_execution_run_id']}', '{scope_b['occurrence_id']}', "
        f"'capacity_accepted', 2, '{scope_b['correlation_id']}', 'runner:collision:{scope_a['result_event_id']}', "
        "decode(repeat('b', 64), 'hex'), '{\"fixture\":true}'::jsonb);",
        "COMMIT;",
    ])
    fixture_ids = {
        "tenant_a": scope_a["tenant_id"],
        "tenant_b": scope_b["tenant_id"],
        "event_id": scope_a["result_event_id"],
    }
    return "\n".join(setup), visibility, collision, fixture_ids


def write_text(path: Path, value: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(value, encoding="utf-8")


def is_contained(path: Path, parent: Path) -> bool:
    try:
        path.resolve().relative_to(parent.resolve())
        return True
    except ValueError:
        return False


def redact_urls(path: Path) -> None:
    if not path.is_file():
        return
    content = path.read_text(encoding="utf-8", errors="replace")
    content = re.sub(r"(?i)\b(?:postgres(?:ql)?|nats)://[^\s'\"<>]+", "<redacted-url>", content)
    path.write_text(content, encoding="utf-8")


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
        redact_urls(stdout_path)
        redact_urls(stderr_path)
        return {
            "name": name,
            "status": "passed" if result.returncode == 0 else "failed",
            "exit_code": result.returncode,
            "command": list(command),
            "stdout_log": str(stdout_path),
            "stderr_log": str(stderr_path),
        }
    except subprocess.TimeoutExpired:
        redact_urls(stdout_path)
        redact_urls(stderr_path)
        return {"name": name, "status": "failed", "exit_code": 124, "reason": "timeout"}
    except OSError as error:
        redact_urls(stdout_path)
        redact_urls(stderr_path)
        return {"name": name, "status": "blocked", "exit_code": 127, "reason": type(error).__name__}


def run_sql(
    name: str,
    command: Sequence[str],
    sql: str,
    env: dict[str, str],
    expect_success: bool = True,
) -> dict[str, object]:
    sql_path = OUTPUT_DIR / f"{name}.sql"
    write_text(sql_path, sql)
    result = run_step(name, [*command, "-f", str(sql_path)], ROOT, env)
    passed = result["status"] == "passed"
    expectation_met = passed == expect_success
    result["asserted_status"] = "passed" if expectation_met else "failed"
    result["status"] = "passed" if expectation_met else "failed"
    if not expectation_met:
        result["reason"] = "command result did not match expected outcome"
    return result


def start_nats_server(
    binary: Path, store_dir: Path, port: int, server_name: str, env: dict[str, str]
) -> tuple[subprocess.Popen[bytes] | None, dict[str, object]]:
    command = [
        str(binary), "-js", "-a", "127.0.0.1", "-p", str(port),
        "--server_name", server_name, "-sd", str(store_dir),
    ]
    log_path = OUTPUT_DIR / "nats-server.log"
    try:
        with log_path.open("wb") as log_file:
            process = subprocess.Popen(
                command,
                cwd=ROOT,
                env=env,
                stdout=log_file,
                stderr=subprocess.STDOUT,
                creationflags=subprocess.CREATE_NO_WINDOW if sys.platform == "win32" else 0,
            )
    except OSError as error:
        return None, {"name": "nats-start", "status": "blocked", "reason": type(error).__name__}
    deadline = time.monotonic() + 60
    while time.monotonic() < deadline:
        return_code = process.poll()
        if return_code is not None:
            redact_urls(log_path)
            return process, {"name": "nats-start", "status": "failed", "exit_code": return_code, "log": str(log_path)}
        try:
            with socket.create_connection(("127.0.0.1", port), timeout=0.25) as connection:
                connection.settimeout(0.5)
                greeting = connection.recv(4096)
                if greeting.startswith(b"INFO "):
                    info = json.loads(greeting[5:].splitlines()[0])
                    if (
                        info.get("jetstream") is True
                        and info.get("server_name") == server_name
                        and process.poll() is None
                    ):
                        return process, {
                            "name": "nats-start",
                            "status": "passed",
                            "pid": process.pid,
                            "server_version": info.get("version"),
                            "server_name": server_name,
                            "jetstream": True,
                            "log": str(log_path),
                        }
        except OSError:
            time.sleep(0.1)
        except (IndexError, json.JSONDecodeError):
            time.sleep(0.1)
    redact_urls(log_path)
    log_tail = log_path.read_text(encoding="utf-8", errors="replace").splitlines()[-12:]
    return process, {
        "name": "nats-start",
        "status": "failed",
        "reason": "loopback listener not ready",
        "pid": process.pid,
        "port": port,
        "exit_code": process.poll(),
        "log_tail": log_tail,
        "log": str(log_path),
    }


def stop_nats_server(process: subprocess.Popen[bytes]) -> dict[str, object]:
    if process.poll() is not None:
        return {"name": "nats-stop", "status": "passed", "already_exited": True, "exit_code": process.returncode}
    try:
        process.terminate()
        process.wait(timeout=10)
        return {"name": "nats-stop", "status": "passed", "termination_requested": True}
    except subprocess.TimeoutExpired:
        process.kill()
        try:
            process.wait(timeout=10)
            return {"name": "nats-stop", "status": "passed", "termination_requested": True, "forced_after_timeout": True}
        except subprocess.TimeoutExpired:
            return {"name": "nats-stop", "status": "failed", "reason": "owned process could not be stopped"}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pg-bin", type=Path, required=True, help="PostgreSQL bin directory with initdb, pg_ctl, and psql")
    parser.add_argument("--cargo", default="cargo", help="Cargo executable or command name")
    parser.add_argument("--toolchain-bin", type=Path, help="Optional Rust toolchain bin directory")
    parser.add_argument("--target-dir", type=Path, help="Optional Cargo target directory")
    parser.add_argument("--output-dir", type=Path, help="Optional report directory under the repository target directory")
    parser.add_argument("--nats-server", type=Path, help="NATS Server binary with JetStream support; not needed with --schema-only")
    parser.add_argument("--schema-only", action="store_true", help="Run PostgreSQL migration, privilege, RLS, and dedupe checks without the Rust/NATS integration test")
    parser.add_argument("--binding-tests", action="store_true", help="Also run Rust SQLx binding roundtrip/CAS/rollback tests; combine with --schema-only to avoid NATS")
    parser.add_argument("--admission-tests", action="store_true", help="Also run Rust SQLx owner-local admission receipt/result tests; --schema-only avoids NATS")
    arguments = parser.parse_args()

    global OUTPUT_DIR
    if arguments.output_dir:
        OUTPUT_DIR = arguments.output_dir.resolve()
    else:
        OUTPUT_DIR = OUTPUT_ROOT / str(uuid.uuid4())
    target_root = (ROOT / "target").resolve()
    if not is_contained(target_root, ROOT) or not is_contained(OUTPUT_DIR, target_root):
        print(json.dumps({"result": "blocked", "reason": "output directory must stay under the repository target directory"}))
        return 2
    if OUTPUT_DIR.exists():
        print(json.dumps({"result": "blocked", "reason": "output directory already exists; use a new path to preserve run evidence"}))
        return 2

    pg_bin = arguments.pg_bin.resolve()
    suffix = ".exe" if sys.platform == "win32" else ""
    executables = {name: pg_bin / f"{name}{suffix}" for name in ("initdb", "pg_ctl", "psql")}
    missing_tools = [str(path) for path in executables.values() if not path.is_file()]
    if missing_tools:
        print(json.dumps({"result": "blocked", "missing_tools": missing_tools}))
        return 2
    missing_migrations = [
        str(path)
        for path in (
            *PREREQUISITE_MIGRATIONS,
            ADMISSION_LIFECYCLE_MIGRATION,
            ADMISSION_LIFECYCLE_TEST,
            RUN_BINDING_MIGRATION,
            SCHEDULE_ADMISSION_MIGRATION,
            ADMISSION_CURRENTNESS_MIGRATION,
            TASK_CURRENTNESS_MIGRATION,
            POLICY_CURRENTNESS_MIGRATION,
            CATALOG_CURRENTNESS_MIGRATION,
            PROJECT_CAPACITY_MIGRATION,
            SCHEDULE_PREPARATION_MIGRATION,
            RUN_BINDING_TEST,
            RUN_MIGRATION,
            QUARANTINE_SCHEMA_TEST,
        )
        if not path.is_file()
    ]
    if missing_migrations:
        print(json.dumps({"result": "blocked", "missing_migrations": missing_migrations}))
        return 2
    if not arguments.schema_only and (
        arguments.nats_server is None or not arguments.nats_server.is_file()
    ):
        print(json.dumps({
            "result": "blocked",
            "missing_nats_server": str(arguments.nats_server) if arguments.nats_server else None,
        }))
        return 2

    cache_root = (ROOT / ".cache").resolve()
    data_parent = DATA_PARENT.resolve()
    if not is_contained(cache_root, ROOT) or not is_contained(data_parent, cache_root):
        print(json.dumps({"result": "blocked", "reason": "temporary data directory escapes repository cache"}))
        return 2
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    data_parent.mkdir(parents=True, exist_ok=True)
    pg_data = data_parent / f"postgres-{uuid.uuid4()}"
    nats_data = data_parent / f"nats-{uuid.uuid4()}"
    if not is_contained(pg_data, data_parent) or pg_data.exists() or not is_contained(nats_data, data_parent) or nats_data.exists():
        print(json.dumps({"result": "blocked", "reason": "unsafe or pre-existing temporary directory"}))
        return 2

    env = os.environ.copy()
    version_steps: list[dict[str, object]] = []
    if arguments.toolchain_bin:
        toolchain_bin = arguments.toolchain_bin.resolve()
        cargo_name = "cargo.exe" if os.name == "nt" else "cargo"
        rustc_name = "rustc.exe" if os.name == "nt" else "rustc"
        rustdoc_name = "rustdoc.exe" if os.name == "nt" else "rustdoc"
        cargo_path = toolchain_bin / cargo_name
        if not cargo_path.is_file() or not (toolchain_bin / rustc_name).is_file() or not (toolchain_bin / rustdoc_name).is_file():
            print(json.dumps({"result": "blocked", "reason": "toolchain bin directory is incomplete"}))
            return 2
        env["PATH"] = str(toolchain_bin) + os.pathsep + env.get("PATH", "")
        env["RUSTC"] = str(toolchain_bin / rustc_name)
        env["RUSTDOC"] = str(toolchain_bin / rustdoc_name)
        cargo = str(cargo_path)
        version_steps.append(run_step("rustc-version", [str(toolchain_bin / rustc_name), "--version"], ROOT, env))
    else:
        cargo = shutil.which(arguments.cargo) or arguments.cargo
        version_steps.append(run_step("cargo-version", [cargo, "--version"], ROOT, env))
    if not arguments.schema_only:
        assert arguments.nats_server is not None
        version_steps.append(run_step("nats-version", [str(arguments.nats_server.resolve()), "--version"], ROOT, env))
    if any(item["status"] != "passed" for item in version_steps):
        print(json.dumps({"result": "blocked", "reason": "a required tool version check failed", "steps": version_steps}, sort_keys=True))
        return 2
    if arguments.target_dir:
        env["CARGO_TARGET_DIR"] = str(arguments.target_dir.resolve())

    pg_port = reserve_port()
    psql_base = [
        str(executables["psql"]), "-X", "-h", "127.0.0.1", "-p", str(pg_port),
        "-U", "star_run_owner_test_admin", "-d", "postgres", "-v", "ON_ERROR_STOP=1",
    ]
    results: list[dict[str, object]] = version_steps
    nats_process: subprocess.Popen[bytes] | None = None
    pg_data_owned = False
    nats_data_owned = False
    nats_cleanup_confirmed = True
    pg_start_attempted = False
    pg_started = False
    exit_code = 1
    try:
        pg_data.mkdir(parents=False)
        pg_data_owned = True
        results.append(run_step(
            "postgres-init",
            [str(executables["initdb"]), "-D", str(pg_data), "-U", "star_run_owner_test_admin", "--auth=trust", "--no-instructions"],
            ROOT,
            env,
        ))
        if results[-1]["status"] == "passed":
            pg_start_attempted = True
            results.append(run_step(
                "postgres-start",
                [str(executables["pg_ctl"]), "-D", str(pg_data), "-l", str(pg_data / "server.log"),
                 "-o", f"-h 127.0.0.1 -p {pg_port} -c listen_addresses=127.0.0.1", "-w", "start"],
                ROOT,
                env,
            ))
            pg_started = results[-1]["status"] == "passed"

        if pg_started:
            version_step = run_step(
                "postgres-version",
                [*psql_base, "-t", "-A", "-c", "SHOW server_version"],
                ROOT,
                env,
            )
            if version_step["status"] == "passed":
                version_step["server_version"] = (OUTPUT_DIR / "postgres-version.stdout.log").read_text(
                    encoding="utf-8", errors="replace"
                ).strip()
            results.append(version_step)

        if pg_started and all(item["status"] == "passed" for item in results):
            results.append(
                run_sql(
                    "disposable-legacy-audit-prerequisite-fixture",
                    psql_base,
                    LEGACY_AUDIT_PREREQUISITE_SQL,
                    env,
                )
            )
        if pg_started and all(item["status"] == "passed" for item in results):
            for migration in PREREQUISITE_MIGRATIONS:
                results.append(run_sql(f"apply-{migration.stem}", psql_base, migration.read_text(encoding="utf-8"), env))
                if results[-1]["status"] != "passed":
                    break
        if pg_started and all(item["status"] == "passed" for item in results):
            lifecycle_sql = ADMISSION_LIFECYCLE_MIGRATION.read_text(encoding="utf-8")
            for attempt in (1, 2):
                results.append(run_sql(f"admission-lifecycle-migration-apply-{attempt}", psql_base, lifecycle_sql, env))
                if results[-1]["status"] != "passed":
                    break
        if pg_started and all(item["status"] == "passed" for item in results):
            results.append(
                run_sql(
                    "admission-attempt-lifecycle-invariants",
                    psql_base,
                    ADMISSION_LIFECYCLE_TEST.read_text(encoding="utf-8"),
                    env,
                )
            )
        if pg_started and all(item["status"] == "passed" for item in results):
            migration_sql = RUN_MIGRATION.read_text(encoding="utf-8")
            for attempt in (1, 2):
                results.append(run_sql(f"run-owner-migration-apply-{attempt}", psql_base, migration_sql, env))
                if results[-1]["status"] != "passed":
                    break
        if pg_started and all(item["status"] == "passed" for item in results):
            binding_sql = RUN_BINDING_MIGRATION.read_text(encoding="utf-8")
            for attempt in (1, 2):
                results.append(run_sql(f"run-binding-migration-apply-{attempt}", psql_base, binding_sql, env))
                if results[-1]["status"] != "passed":
                    break
        if pg_started and all(item["status"] == "passed" for item in results):
            for attempt in (1, 2):
                results.append(run_sql(f"schedule-admission-migration-apply-{attempt}", psql_base, SCHEDULE_ADMISSION_MIGRATION.read_text(encoding="utf-8"), env))
                if results[-1]["status"] != "passed":
                    break
        if pg_started and all(item["status"] == "passed" for item in results):
            for attempt in (1, 2):
                results.append(run_sql(f"admission-currentness-migration-apply-{attempt}", psql_base, ADMISSION_CURRENTNESS_MIGRATION.read_text(encoding="utf-8"), env))
                if results[-1]["status"] != "passed":
                    break
                results.append(run_sql(f"task-currentness-migration-apply-{attempt}", psql_base, TASK_CURRENTNESS_MIGRATION.read_text(encoding="utf-8"), env))
                if results[-1]["status"] != "passed":
                    break
                results.append(run_sql(f"policy-currentness-migration-apply-{attempt}", psql_base, POLICY_CURRENTNESS_MIGRATION.read_text(encoding="utf-8"), env))
                if results[-1]["status"] != "passed":
                    break
                results.append(run_sql(f"catalog-currentness-migration-apply-{attempt}", psql_base, CATALOG_CURRENTNESS_MIGRATION.read_text(encoding="utf-8"), env))
                if results[-1]["status"] != "passed":
                    break
        if pg_started and all(item["status"] == "passed" for item in results):
            for attempt in (1, 2):
                results.append(run_sql(f"project-capacity-migration-apply-{attempt}", psql_base, PROJECT_CAPACITY_MIGRATION.read_text(encoding="utf-8"), env))
                if results[-1]["status"] != "passed":
                    break
        if pg_started and all(item["status"] == "passed" for item in results):
            for attempt in (1, 2):
                results.append(run_sql(f"schedule-preparation-migration-apply-{attempt}", psql_base, SCHEDULE_PREPARATION_MIGRATION.read_text(encoding="utf-8"), env))
                if results[-1]["status"] != "passed":
                    break
        if pg_started and all(item["status"] == "passed" for item in results):
            results.append(run_sql("run-owner-schema-assertions", psql_base, SCHEMA_ASSERTIONS_SQL, env))
        if pg_started and all(item["status"] == "passed" for item in results):
            results.append(run_sql("run-binding-master-schema-and-lifecycle", psql_base, RUN_BINDING_TEST.read_text(encoding="utf-8"), env))
        if pg_started and all(item["status"] == "passed" for item in results):
            results.append(
                run_sql(
                    "platform-poison-quarantine-schema",
                    psql_base,
                    QUARANTINE_SCHEMA_TEST.read_text(encoding="utf-8"),
                    env,
                )
            )
        if pg_started and all(item["status"] == "passed" for item in results):
            fixture_setup, tenant_b_visibility, cross_tenant_collision, fixture_ids = rls_fixture_sql()
            results.append(run_sql("run-owner-rls-fixture-and-tenant-a-write", psql_base, fixture_setup, env))
            if results[-1]["status"] == "passed":
                results.append(run_sql("run-owner-rls-tenant-b-hides-tenant-a", psql_base, tenant_b_visibility, env))
            if results[-1]["status"] == "passed":
                collision_step = run_sql(
                    "run-owner-rls-cross-tenant-event-id-collision",
                    psql_base,
                    cross_tenant_collision,
                    env,
                    expect_success=False,
                )
                if collision_step["status"] == "passed":
                    collision_log = "\n".join(
                        (OUTPUT_DIR / f"{name}.{stream}.log").read_text(encoding="utf-8", errors="replace")
                        for name in ["run-owner-rls-cross-tenant-event-id-collision"]
                        for stream in ("stdout", "stderr")
                    )
                    collision_step["expected_constraint"] = "uq_run_capacity_result_inbox_event_id"
                    if (
                        "uq_run_capacity_result_inbox_event_id" not in collision_log
                        or any(field in collision_log.lower() for field in ("tenant_id", "project_id", "work_item_id"))
                        or fixture_ids["tenant_a"] in collision_log
                        or fixture_ids["tenant_b"] in collision_log
                    ):
                        collision_step["status"] = "failed"
                        collision_step["reason"] = "collision did not fail only on global event_id without tenant-scoped detail"
                results.append(collision_step)

        if pg_started and all(item["status"] == "passed" for item in results) and arguments.binding_tests:
            binding_env = env.copy()
            binding_env["STAR_RUN_OWNER_TEST_DATABASE_URL"] = (
                f"postgresql://star_run_owner_test_admin@127.0.0.1:{pg_port}/postgres"
            )
            results.append(run_step(
                "run-binding-postgres-sqlx-integration",
                [cargo, "test", "--offline", "--locked", "-p", "run-owner-runtime", "--test", "run_infrastructure_binding_pg", "-j", "1", "--", "--ignored", "--test-threads=1"],
                ROOT,
                binding_env,
            ))

        if pg_started and all(item["status"] == "passed" for item in results) and arguments.admission_tests:
            admission_env = env.copy()
            admission_env["STAR_RUN_OWNER_TEST_DATABASE_URL"] = (
                f"postgresql://star_run_owner_test_admin@127.0.0.1:{pg_port}/postgres"
            )
            results.append(run_step(
                "schedule-admission-postgres-sqlx-integration",
                [cargo, "test", "--offline", "--locked", "-p", "run-owner-runtime", "--test", "schedule_admission_pg", "-j", "1", "--", "--ignored", "--test-threads=1"],
                ROOT,
                admission_env,
            ))

        if pg_started and all(item["status"] == "passed" for item in results) and not arguments.schema_only:
            nats_data.mkdir(parents=False)
            nats_data_owned = True
            nats_port = reserve_port()
            nats_name = f"star-run-owner-test-{uuid.uuid4().hex}"
            nats_process, nats_start = start_nats_server(
                arguments.nats_server.resolve(), nats_data, nats_port, nats_name, env
            )
            nats_version = next((item for item in results if item["name"] == "nats-version"), {})
            if nats_start["status"] == "passed" and nats_version.get("stdout_log"):
                nats_start["reported_version"] = Path(str(nats_version["stdout_log"])).read_text(
                    encoding="utf-8", errors="replace"
                ).strip()
            results.append(nats_start)
            if nats_start["status"] == "passed":
                test_env = env.copy()
                test_env["STAR_RUN_OWNER_TEST_DATABASE_URL"] = (
                    f"postgresql://star_run_owner_test_admin@127.0.0.1:{pg_port}/postgres"
                )
                test_env["STAR_RUN_OWNER_TEST_NATS_URL"] = f"nats://127.0.0.1:{nats_port}"
                results.append(run_step(
                    "run-owner-capacity-result-postgres-jetstream-integration",
                    [cargo, "test", "--offline", "--locked", "-p", "run-owner-runtime", "--test", "capacity_result_jetstream_pg", "-j", "1", "--", "--ignored", "--test-threads=1"],
                    ROOT,
                    test_env,
                ))
        if all(item["status"] == "passed" for item in results):
            exit_code = 0
    except OSError as error:
        results.append({"name": "runner", "status": "failed", "reason": type(error).__name__})
    finally:
        if nats_process is not None:
            nats_stop = stop_nats_server(nats_process)
            nats_cleanup_confirmed = nats_stop["status"] == "passed"
            results.append(nats_stop)
            redact_urls(OUTPUT_DIR / "nats-server.log")
        if nats_data_owned and nats_data.exists() and is_contained(nats_data, data_parent):
            if nats_cleanup_confirmed:
                shutil.rmtree(nats_data)
            else:
                print(json.dumps({"preserved_nats_data": str(nats_data), "reason": "owned server stop could not be confirmed"}))
                exit_code = 1

        pg_stop_confirmed = not pg_start_attempted
        if pg_data_owned and pg_data.exists() and is_contained(pg_data, data_parent):
            if pg_start_attempted:
                pg_stop = run_step(
                    "postgres-stop",
                    [str(executables["pg_ctl"]), "-D", str(pg_data), "-m", "fast", "-w", "stop"],
                    ROOT,
                    env,
                )
                pg_stop_confirmed = pg_stop["status"] == "passed"
                results.append(pg_stop)
            if pg_stop_confirmed:
                shutil.rmtree(pg_data)
            else:
                print(json.dumps({"preserved_postgres_data": str(pg_data), "reason": "owned server stop could not be confirmed"}))
                exit_code = 1

        write_text(OUTPUT_DIR / "audit.jsonl", "".join(json.dumps(item, sort_keys=True) + "\n" for item in results))
        print(json.dumps({
            "result": "passed" if exit_code == 0 and all(item["status"] == "passed" for item in results) else "failed",
            "mode": ("postgres_admission_rust_integration" if arguments.admission_tests else "postgres_binding_rust_integration" if arguments.binding_tests else "schema_only") if arguments.schema_only else "postgres_nats_rust_integration",
            "steps": results,
            "audit_path": str(OUTPUT_DIR / "audit.jsonl"),
            "multipass_instances_created": 0,
            "multipass_inventory_modified": False,
        }, sort_keys=True))
    return exit_code


if __name__ == "__main__":
    sys.exit(main())
