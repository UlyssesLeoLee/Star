"""Run Schedule recurrence, immutable run-as, and admission-persistence gates."""

# @cypher schema=1 source_sha256=2ef674bf71280e68ab69aba3bd6142d351db305eb8d6c3b2a075ef69cf40e870
# MERGE (self:File {path:"scripts/automation/phase9f3_schedule.py"})
# MERGE (main:Symbol {id:"scripts/automation/phase9f3_schedule.py::main",kind:"function"})
# MERGE (run_step:Symbol {id:"scripts/automation/phase9f3_schedule.py::run_step",kind:"function"})
# MERGE (postgres:Symbol {id:"scripts/automation/phase9f3_schedule.py::run_isolated_postgres",kind:"function"})
# MERGE (cluster:Symbol {id:"scripts/automation/phase9f3_schedule.py::DisposablePostgres",kind:"class"})
# MERGE (docker_cluster:Symbol {id:"scripts/automation/phase9f3_schedule.py::DisposableDockerPostgres",kind:"class"})
# MERGE (scope:Symbol {id:"scripts/automation/phase9f3_schedule.py::is_contained",kind:"function"})
# MERGE (repo:ExternalService {id:"git.repository",kind:"directory"})
# MERGE (postgres_service:ExternalService {id:"postgres.initdb_pg_ctl_psql",kind:"command"})
# MERGE (docker_service:ExternalService {id:"docker.run_exec_cp_rm",kind:"command"})
# MERGE (cargo:ExternalService {id:"cargo.test_clippy_check",kind:"command"})
# MERGE (subprocess:ExternalService {id:"python.subprocess.run",kind:"function"})
# MERGE (migrations:Config {id:"scripts/automation/phase9f3_schedule.py::MIGRATIONS"})
# MERGE (task_run_schema:Config {id:"scripts/automation/phase9f3_schedule.py::TASK_RUN_SCHEMA_FIXTURE_SQL"})
# MERGE (admission_fixture:Config {id:"scripts/automation/phase9f3_schedule.py::SCHEDULE_ADMISSION_FIXTURE_SQL"})
# MERGE (legacy_fixture:Config {id:"scripts/automation/phase9f3_schedule.py::LEGACY_BACKFILL_FIXTURE_SQL"})
# MERGE (legacy_assertion:Config {id:"scripts/automation/phase9f3_schedule.py::LEGACY_BACKFILL_ASSERTION_SQL"})
# MERGE (directory_migrations:Config {id:"scripts/automation/phase9f3_schedule.py::DIRECTORY_MIGRATIONS"})
# MERGE (directory_prerequisite:Config {id:"scripts/automation/phase9f3_schedule.py::DIRECTORY_AUDIT_PREREQUISITE_SQL"})
# MERGE (directory_acl_fixture:Config {id:"scripts/automation/phase9f3_schedule.py::DIRECTORY_AUTH_FIXTURE_SQL"})
# MERGE (acl_rust_test:Symbol {id:"crates/star-api-rest/src/group_api/schedule_rules.rs::tests::schedule_run_as_authorization_rechecks_canonical_directory_grants",kind:"test"})
# MERGE (self)-[:DEFINES]->(main)
# MERGE (self)-[:DEFINES]->(run_step)
# MERGE (self)-[:DEFINES]->(postgres)
# MERGE (self)-[:DEFINES]->(cluster)
# MERGE (self)-[:DEFINES]->(docker_cluster)
# MERGE (self)-[:DEFINES]->(scope)
# MERGE (main)-[:CALLS]->(run_step)
# MERGE (main)-[:CALLS]->(postgres)
# MERGE (run_step)-[:CALLS]->(subprocess)
# MERGE (postgres)-[:CALLS]->(run_step)
# MERGE (postgres)-[:CONFIGURES]->(postgres_service)
# MERGE (postgres)-[:CONFIGURES]->(docker_service)
# MERGE (main)-[:USES]->(repo)
# MERGE (postgres)-[:CALLS]->(scope)
# MERGE (postgres)-[:READS]->(legacy_fixture)
# MERGE (postgres)-[:READS]->(legacy_assertion)
# MERGE (postgres)-[:READS]->(migrations)
# MERGE (postgres)-[:READS]->(task_run_schema)
# MERGE (postgres)-[:READS]->(admission_fixture)
# MERGE (postgres)-[:READS]->(directory_migrations)
# MERGE (postgres)-[:READS]->(directory_prerequisite)
# MERGE (postgres)-[:READS]->(directory_acl_fixture)
# MERGE (postgres)-[:TESTS]->(acl_rust_test)
# @endcypher

from __future__ import annotations

import argparse
import json
import os
import shutil
import socket
import subprocess
import sys
import time
import uuid
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OUTPUT_DIR = ROOT / ".cache" / "phase9f3-schedule"
MAX_STEP_SECONDS = 900
MIGRATIONS = (
    ROOT / "db" / "migrations" / "2026-10-02-automation-schedule-occurrence.sql",
    ROOT / "db" / "migrations" / "2026-10-03-automation-schedule-rule-api.sql",
    ROOT / "db" / "migrations" / "2026-10-04-schedule-run-as-actor.sql",
    ROOT / "db" / "migrations" / "2026-10-04-schedule-run-admission.sql",
)
DIRECTORY_MIGRATIONS = (
    ROOT / "db" / "migrations" / "2026-09-16-worktree-canvas-worktree.sql",
    ROOT / "db" / "migrations" / "2026-09-29-worktree-group-phase-2b.sql",
    ROOT / "db" / "migrations" / "2026-10-01-engineering-run-directory.sql",
)
DIRECTORY_AUDIT_PREREQUISITE_SQL = """\
CREATE OR REPLACE FUNCTION public.audit_trigger_func()
RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RETURN COALESCE(NEW, OLD); END $$;"""
TASK_RUN_SCHEMA_FIXTURE_SQL = """\
CREATE SCHEMA IF NOT EXISTS multica;
CREATE TABLE IF NOT EXISTS multica.task_execution_run (
    run_id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    work_item_id UUID NOT NULL,
    engineering_run_id UUID NOT NULL,
    initiated_by UUID NOT NULL,
    execution_channel VARCHAR(24) NOT NULL,
    run_origin VARCHAR(16) NOT NULL,
    automation_rule_id UUID,
    automation_rule_version BIGINT,
    automation_occurrence_id UUID,
    UNIQUE (tenant_id, run_id),
    UNIQUE (tenant_id, project_id, work_item_id, run_id)
);"""
SCHEDULE_ADMISSION_FIXTURE_SQL = """\
INSERT INTO automation.occurrence_dispatch (
    tenant_id, occurrence_id, dispatch_state, attempt_count, next_attempt_at,
    fencing_generation, lease_owner_id, lease_expires_at, occurrence_deadline_at
) VALUES (
    '10000000-0000-4000-8000-000000000001',
    'c0000000-0000-4000-8000-000000000001', 'leased', 1, now(), 1,
    'd0000000-0000-4000-8000-000000000001', now() + interval '5 minutes',
    now() + interval '1 hour'
);
INSERT INTO multica.task_execution_run (
    run_id, tenant_id, project_id, work_item_id, engineering_run_id, initiated_by,
    execution_channel, run_origin, automation_rule_id, automation_rule_version,
    automation_occurrence_id
) VALUES (
    'e0000000-0000-4000-8000-000000000001',
    '10000000-0000-4000-8000-000000000001',
    '20000000-0000-4000-8000-000000000001',
    '80000000-0000-4000-8000-000000000001',
    '50000000-0000-4000-8000-000000000001',
    'a0000000-0000-4000-8000-000000000001', 'agent', 'schedule',
    '30000000-0000-4000-8000-000000000001', 2,
    'c0000000-0000-4000-8000-000000000001'
);
UPDATE automation.occurrence_dispatch
   SET dispatch_state = 'admitted', admitted_run_id = 'e0000000-0000-4000-8000-000000000001',
       lease_owner_id = NULL, lease_expires_at = NULL, updated_at = now()
 WHERE tenant_id = '10000000-0000-4000-8000-000000000001'
   AND occurrence_id = 'c0000000-0000-4000-8000-000000000001';
DO $$
BEGIN
  BEGIN
    INSERT INTO automation.schedule_run_outbox (
        tenant_id, project_id, engineering_run_id, rule_id, rule_version, occurrence_id,
        work_item_id, run_id, run_as_actor_id, event_type, correlation_id, payload
    ) VALUES (
        '10000000-0000-4000-8000-000000000001',
        '20000000-0000-4000-8000-000000000001',
        '50000000-0000-4000-8000-000000000001',
        '30000000-0000-4000-8000-000000000001', 2,
        'c0000000-0000-4000-8000-000000000001',
        '80000000-0000-4000-8000-000000000001',
        'e0000000-0000-4000-8000-000000000001',
        'b0000000-0000-4000-8000-000000000002', 'schedule_run.admitted',
        'f0000000-0000-4000-8000-000000000002', '{"schema_version":1}'::jsonb
    );
    RAISE EXCEPTION 'mismatched run-as Outbox insert unexpectedly succeeded';
  EXCEPTION WHEN OTHERS THEN
    IF SQLERRM <> 'Schedule Run Outbox does not match the admitted occurrence and Run' THEN
      RAISE;
    END IF;
  END;
END $$;
INSERT INTO automation.schedule_run_outbox (
    tenant_id, project_id, engineering_run_id, rule_id, rule_version, occurrence_id,
    work_item_id, run_id, run_as_actor_id, event_type, correlation_id, payload
) VALUES (
    '10000000-0000-4000-8000-000000000001',
    '20000000-0000-4000-8000-000000000001',
    '50000000-0000-4000-8000-000000000001',
    '30000000-0000-4000-8000-000000000001', 2,
    'c0000000-0000-4000-8000-000000000001',
    '80000000-0000-4000-8000-000000000001',
    'e0000000-0000-4000-8000-000000000001',
    'a0000000-0000-4000-8000-000000000001', 'schedule_run.admitted',
    'f0000000-0000-4000-8000-000000000001', '{"schema_version":1}'::jsonb
);
DO $$
BEGIN
  BEGIN
    UPDATE automation.schedule_run_outbox
       SET payload = '{"mutated":true}'::jsonb
     WHERE event_id = (SELECT min(event_id) FROM automation.schedule_run_outbox);
    RAISE EXCEPTION 'Schedule Run Outbox mutation unexpectedly succeeded';
  EXCEPTION WHEN OTHERS THEN
    IF SQLERRM = 'Schedule Run Outbox mutation unexpectedly succeeded' THEN RAISE; END IF;
  END;
  BEGIN
    UPDATE automation.occurrence_dispatch
       SET admitted_run_id = 'e0000000-0000-4000-8000-000000000002'
     WHERE tenant_id = '10000000-0000-4000-8000-000000000001'
       AND occurrence_id = 'c0000000-0000-4000-8000-000000000001';
    RAISE EXCEPTION 'admitted Run identity mutation unexpectedly succeeded';
  EXCEPTION WHEN OTHERS THEN
    IF SQLERRM = 'admitted Run identity mutation unexpectedly succeeded' THEN RAISE; END IF;
  END;
END $$;"""
LEGACY_BACKFILL_FIXTURE_SQL = """\
INSERT INTO automation.schedule_rule_revision (
    tenant_id, project_id, rule_id, rule_version, cron_expression, time_zone,
    parser_version, tzdb_version, dst_gap_policy, dst_fold_policy, overlap_policy,
    misfire_policy, pause_policy, retry_max_attempts, retry_initial_backoff_seconds,
    retry_max_backoff_seconds, deadline_seconds, branch_id, engineering_run_id,
    repository_id, worktree_id, work_item_id, execution_profile_id,
    execution_profile_version, execution_profile_digest, hook_set_version,
    hook_set_digest, changed_by, valid_from, valid_to
) VALUES
    ('10000000-0000-4000-8000-000000000001', '20000000-0000-4000-8000-000000000001',
     '30000000-0000-4000-8000-000000000001', 1, '0 0 * * *', 'UTC', 'legacy-parser',
     'legacy-tzdb', 'skip', 'earlier_instant', 'queue_one', 'skip', 'skip_elapsed',
     1, 1, 1, 60, '40000000-0000-4000-8000-000000000001',
     '50000000-0000-4000-8000-000000000001', '60000000-0000-4000-8000-000000000001',
     '70000000-0000-4000-8000-000000000001', '80000000-0000-4000-8000-000000000001',
     '90000000-0000-4000-8000-000000000001', 1, repeat('a', 64), 1, repeat('b', 64),
     'a0000000-0000-4000-8000-000000000001', now() - interval '2 days', now() - interval '1 day'),
    ('10000000-0000-4000-8000-000000000001', '20000000-0000-4000-8000-000000000001',
     '30000000-0000-4000-8000-000000000001', 2, '0 0 * * *', 'UTC', 'legacy-parser',
     'legacy-tzdb', 'skip', 'earlier_instant', 'queue_one', 'skip', 'skip_elapsed',
     1, 1, 1, 60, '40000000-0000-4000-8000-000000000001',
     '50000000-0000-4000-8000-000000000001', '60000000-0000-4000-8000-000000000001',
     '70000000-0000-4000-8000-000000000001', '80000000-0000-4000-8000-000000000001',
     '90000000-0000-4000-8000-000000000001', 1, repeat('a', 64), 1, repeat('b', 64),
     'b0000000-0000-4000-8000-000000000002', now() - interval '1 day', NULL);

INSERT INTO automation.occurrence (
    occurrence_id, tenant_id, project_id, rule_id, rule_version, scheduled_for_utc,
    scheduled_local_label, utc_offset_seconds, parser_version, tzdb_version,
    target_snapshot, target_snapshot_digest, materialized_at
) VALUES (
    'c0000000-0000-4000-8000-000000000001',
    '10000000-0000-4000-8000-000000000001',
    '20000000-0000-4000-8000-000000000001',
    '30000000-0000-4000-8000-000000000001', 2, now() + interval '1 day',
    to_char(now() + interval '1 day', 'YYYY-MM-DD"T"HH24:MI:SS'), 0,
    'legacy-parser', 'legacy-tzdb', '{}'::jsonb, repeat('c', 64), now()
);"""
LEGACY_BACKFILL_ASSERTION_SQL = """\
DO $$
DECLARE rule_rows integer;
DECLARE wrong_rule_actors integer;
DECLARE occurrence_actor uuid;
BEGIN
    SELECT count(*), count(*) FILTER (
        WHERE run_as_actor_id IS DISTINCT FROM 'a0000000-0000-4000-8000-000000000001'::uuid
    ) INTO rule_rows, wrong_rule_actors
    FROM automation.schedule_rule_revision
    WHERE tenant_id = '10000000-0000-4000-8000-000000000001'::uuid
      AND rule_id = '30000000-0000-4000-8000-000000000001'::uuid;
    IF rule_rows <> 2 OR wrong_rule_actors <> 0 THEN
        RAISE EXCEPTION 'legacy Rule revisions did not inherit the original creator';
    END IF;
    SELECT run_as_actor_id INTO occurrence_actor
    FROM automation.occurrence
    WHERE occurrence_id = 'c0000000-0000-4000-8000-000000000001'::uuid;
    IF occurrence_actor IS DISTINCT FROM 'a0000000-0000-4000-8000-000000000001'::uuid THEN
        RAISE EXCEPTION 'legacy occurrence did not inherit its exact Rule creator';
    END IF;
END $$;"""
DIRECTORY_AUTH_FIXTURE_SQL = """\
BEGIN;
SELECT set_config('app.tenant_id', '21000000-0000-4000-8000-000000000001', true);
SELECT set_config('app.actor_id', '22000000-0000-4000-8000-000000000002', true);
SELECT set_config('app.correlation_id', '22000000-0000-4000-8000-000000000003', true);

INSERT INTO permission.project_role_binding
    (tenant_id, project_id, user_id, role, granted_by, valid_from, version)
VALUES
    ('21000000-0000-4000-8000-000000000001', '23000000-0000-4000-8000-000000000001', '22000000-0000-4000-8000-000000000002', 'project_admin', '22000000-0000-4000-8000-000000000002', now() - interval '1 minute', 1),
    ('21000000-0000-4000-8000-000000000001', '23000000-0000-4000-8000-000000000001', '22000000-0000-4000-8000-000000000001', 'developer', '22000000-0000-4000-8000-000000000002', now() - interval '1 minute', 1),
    ('21000000-0000-4000-8000-000000000001', '23000000-0000-4000-8000-000000000002', '22000000-0000-4000-8000-000000000002', 'project_admin', '22000000-0000-4000-8000-000000000002', now() - interval '1 minute', 1),
    ('21000000-0000-4000-8000-000000000001', '23000000-0000-4000-8000-000000000002', '22000000-0000-4000-8000-000000000001', 'developer', '22000000-0000-4000-8000-000000000002', now() - interval '1 minute', 1),
    ('21000000-0000-4000-8000-000000000001', '23000000-0000-4000-8000-000000000003', '22000000-0000-4000-8000-000000000002', 'project_admin', '22000000-0000-4000-8000-000000000002', now() - interval '1 minute', 1),
    ('21000000-0000-4000-8000-000000000001', '23000000-0000-4000-8000-000000000003', '22000000-0000-4000-8000-000000000001', 'developer', '22000000-0000-4000-8000-000000000002', now() - interval '1 minute', 1),
    ('21000000-0000-4000-8000-000000000001', '23000000-0000-4000-8000-000000000004', '22000000-0000-4000-8000-000000000002', 'project_admin', '22000000-0000-4000-8000-000000000002', now() - interval '1 minute', 1),
    ('21000000-0000-4000-8000-000000000001', '23000000-0000-4000-8000-000000000004', '22000000-0000-4000-8000-000000000001', 'developer', '22000000-0000-4000-8000-000000000002', now() - interval '1 minute', 1),
    ('21000000-0000-4000-8000-000000000001', '23000000-0000-4000-8000-000000000005', '22000000-0000-4000-8000-000000000002', 'project_admin', '22000000-0000-4000-8000-000000000002', now() - interval '1 minute', 1),
    ('21000000-0000-4000-8000-000000000001', '23000000-0000-4000-8000-000000000005', '22000000-0000-4000-8000-000000000001', 'developer', '22000000-0000-4000-8000-000000000002', now() - interval '1 minute', 1);

INSERT INTO scm.cloud_branch
    (branch_id, tenant_id, project_id, repository_id, remote_identity)
VALUES
    ('24000000-0000-4000-8000-000000000001', '21000000-0000-4000-8000-000000000001', '23000000-0000-4000-8000-000000000001', '26000000-0000-4000-8000-000000000001', 'schedule-acl-fixture:branch-1'),
    ('24000000-0000-4000-8000-000000000002', '21000000-0000-4000-8000-000000000001', '23000000-0000-4000-8000-000000000002', '26000000-0000-4000-8000-000000000001', 'schedule-acl-fixture:branch-2'),
    ('24000000-0000-4000-8000-000000000003', '21000000-0000-4000-8000-000000000001', '23000000-0000-4000-8000-000000000003', '26000000-0000-4000-8000-000000000001', 'schedule-acl-fixture:branch-3'),
    ('24000000-0000-4000-8000-000000000004', '21000000-0000-4000-8000-000000000001', '23000000-0000-4000-8000-000000000004', '26000000-0000-4000-8000-000000000001', 'schedule-acl-fixture:branch-4'),
    ('24000000-0000-4000-8000-000000000005', '21000000-0000-4000-8000-000000000001', '23000000-0000-4000-8000-000000000005', '26000000-0000-4000-8000-000000000001', 'schedule-acl-fixture:branch-5');

INSERT INTO scm.cloud_branch_revision
    (tenant_id, branch_id, name, full_ref, state, changed_by, valid_from, version)
VALUES
    ('21000000-0000-4000-8000-000000000001', '24000000-0000-4000-8000-000000000001', 'auth-1', 'refs/heads/auth-1', 'active', '22000000-0000-4000-8000-000000000002', now() - interval '1 minute', 1),
    ('21000000-0000-4000-8000-000000000001', '24000000-0000-4000-8000-000000000002', 'auth-2', 'refs/heads/auth-2', 'active', '22000000-0000-4000-8000-000000000002', now() - interval '1 minute', 1),
    ('21000000-0000-4000-8000-000000000001', '24000000-0000-4000-8000-000000000003', 'auth-3', 'refs/heads/auth-3', 'active', '22000000-0000-4000-8000-000000000002', now() - interval '1 minute', 1),
    ('21000000-0000-4000-8000-000000000001', '24000000-0000-4000-8000-000000000004', 'auth-4', 'refs/heads/auth-4', 'active', '22000000-0000-4000-8000-000000000002', now() - interval '1 minute', 1),
    ('21000000-0000-4000-8000-000000000001', '24000000-0000-4000-8000-000000000005', 'auth-5', 'refs/heads/auth-5', 'active', '22000000-0000-4000-8000-000000000002', now() - interval '1 minute', 1);

INSERT INTO permission.cloud_branch_role_binding
    (tenant_id, branch_id, user_id, role, granted_by, valid_from, version)
VALUES
    ('21000000-0000-4000-8000-000000000001', '24000000-0000-4000-8000-000000000001', '22000000-0000-4000-8000-000000000001', 'developer', '22000000-0000-4000-8000-000000000002', now() - interval '1 minute', 1),
    ('21000000-0000-4000-8000-000000000001', '24000000-0000-4000-8000-000000000002', '22000000-0000-4000-8000-000000000001', 'developer', '22000000-0000-4000-8000-000000000002', now() - interval '1 minute', 1),
    ('21000000-0000-4000-8000-000000000001', '24000000-0000-4000-8000-000000000003', '22000000-0000-4000-8000-000000000001', 'developer', '22000000-0000-4000-8000-000000000002', now() - interval '1 minute', 1),
    ('21000000-0000-4000-8000-000000000001', '24000000-0000-4000-8000-000000000004', '22000000-0000-4000-8000-000000000001', 'developer', '22000000-0000-4000-8000-000000000002', now() - interval '1 minute', 1),
    ('21000000-0000-4000-8000-000000000001', '24000000-0000-4000-8000-000000000005', '22000000-0000-4000-8000-000000000001', 'developer', '22000000-0000-4000-8000-000000000002', now() - interval '1 minute', 1);

INSERT INTO multica.engineering_run
    (engineering_run_id, tenant_id, project_id, repository_id, branch_id)
VALUES
    ('25000000-0000-4000-8000-000000000001', '21000000-0000-4000-8000-000000000001', '23000000-0000-4000-8000-000000000001', '26000000-0000-4000-8000-000000000001', '24000000-0000-4000-8000-000000000001'),
    ('25000000-0000-4000-8000-000000000002', '21000000-0000-4000-8000-000000000001', '23000000-0000-4000-8000-000000000002', '26000000-0000-4000-8000-000000000001', '24000000-0000-4000-8000-000000000002'),
    ('25000000-0000-4000-8000-000000000003', '21000000-0000-4000-8000-000000000001', '23000000-0000-4000-8000-000000000003', '26000000-0000-4000-8000-000000000001', '24000000-0000-4000-8000-000000000003'),
    ('25000000-0000-4000-8000-000000000004', '21000000-0000-4000-8000-000000000001', '23000000-0000-4000-8000-000000000004', '26000000-0000-4000-8000-000000000001', '24000000-0000-4000-8000-000000000004'),
    ('25000000-0000-4000-8000-000000000005', '21000000-0000-4000-8000-000000000001', '23000000-0000-4000-8000-000000000005', '26000000-0000-4000-8000-000000000001', '24000000-0000-4000-8000-000000000005');

INSERT INTO multica.engineering_run_revision
    (tenant_id, engineering_run_id, title, state, owner_user_id, changed_by, valid_from, version)
VALUES
    ('21000000-0000-4000-8000-000000000001', '25000000-0000-4000-8000-000000000001', 'Schedule ACL active 1', 'active', '22000000-0000-4000-8000-000000000001', '22000000-0000-4000-8000-000000000002', now() - interval '1 minute', 1),
    ('21000000-0000-4000-8000-000000000001', '25000000-0000-4000-8000-000000000002', 'Schedule ACL revoked project', 'active', '22000000-0000-4000-8000-000000000001', '22000000-0000-4000-8000-000000000002', now() - interval '1 minute', 1),
    ('21000000-0000-4000-8000-000000000001', '25000000-0000-4000-8000-000000000003', 'Schedule ACL revoked branch', 'active', '22000000-0000-4000-8000-000000000001', '22000000-0000-4000-8000-000000000002', now() - interval '1 minute', 1),
    ('21000000-0000-4000-8000-000000000001', '25000000-0000-4000-8000-000000000004', 'Schedule ACL revoked run', 'active', '22000000-0000-4000-8000-000000000001', '22000000-0000-4000-8000-000000000002', now() - interval '1 minute', 1),
    ('21000000-0000-4000-8000-000000000001', '25000000-0000-4000-8000-000000000005', 'Schedule ACL paused run', 'paused', '22000000-0000-4000-8000-000000000001', '22000000-0000-4000-8000-000000000002', now() - interval '1 minute', 1);

INSERT INTO permission.engineering_run_role_binding
    (tenant_id, engineering_run_id, user_id, role, granted_by, valid_from, version)
VALUES
    ('21000000-0000-4000-8000-000000000001', '25000000-0000-4000-8000-000000000001', '22000000-0000-4000-8000-000000000001', 'developer', '22000000-0000-4000-8000-000000000002', now() - interval '1 minute', 1),
    ('21000000-0000-4000-8000-000000000001', '25000000-0000-4000-8000-000000000002', '22000000-0000-4000-8000-000000000001', 'developer', '22000000-0000-4000-8000-000000000002', now() - interval '1 minute', 1),
    ('21000000-0000-4000-8000-000000000001', '25000000-0000-4000-8000-000000000003', '22000000-0000-4000-8000-000000000001', 'developer', '22000000-0000-4000-8000-000000000002', now() - interval '1 minute', 1),
    ('21000000-0000-4000-8000-000000000001', '25000000-0000-4000-8000-000000000004', '22000000-0000-4000-8000-000000000001', 'developer', '22000000-0000-4000-8000-000000000002', now() - interval '1 minute', 1),
    ('21000000-0000-4000-8000-000000000001', '25000000-0000-4000-8000-000000000005', '22000000-0000-4000-8000-000000000001', 'developer', '22000000-0000-4000-8000-000000000002', now() - interval '1 minute', 1);

DO $$
DECLARE closed_rows integer;
BEGIN
    UPDATE permission.project_role_binding SET valid_to = statement_timestamp()
     WHERE tenant_id = '21000000-0000-4000-8000-000000000001'
       AND project_id = '23000000-0000-4000-8000-000000000002'
       AND user_id = '22000000-0000-4000-8000-000000000001' AND valid_to IS NULL;
    GET DIAGNOSTICS closed_rows = ROW_COUNT;
    IF closed_rows <> 1 THEN RAISE EXCEPTION 'Project revoke fixture did not close one grant'; END IF;

    UPDATE permission.cloud_branch_role_binding SET valid_to = statement_timestamp()
     WHERE tenant_id = '21000000-0000-4000-8000-000000000001'
       AND branch_id = '24000000-0000-4000-8000-000000000003'
       AND user_id = '22000000-0000-4000-8000-000000000001' AND valid_to IS NULL;
    GET DIAGNOSTICS closed_rows = ROW_COUNT;
    IF closed_rows <> 1 THEN RAISE EXCEPTION 'Branch revoke fixture did not close one grant'; END IF;

    UPDATE permission.engineering_run_role_binding SET valid_to = statement_timestamp()
     WHERE tenant_id = '21000000-0000-4000-8000-000000000001'
       AND engineering_run_id = '25000000-0000-4000-8000-000000000004'
       AND user_id = '22000000-0000-4000-8000-000000000001' AND valid_to IS NULL;
    GET DIAGNOSTICS closed_rows = ROW_COUNT;
    IF closed_rows <> 1 THEN RAISE EXCEPTION 'Run revoke fixture did not close one grant'; END IF;
END $$;
COMMIT;"""
RUST_FILES = (
    "crates/domain-automation/src/lib.rs",
    "crates/domain-automation/src/schedule.rs",
    "crates/star-pg-adapter/src/repository/mod.rs",
    "crates/star-pg-adapter/src/repository/automation_schedule.rs",
    "crates/star-pg-adapter/tests/schedule_postgres.rs",
)
API_RUST_FILE = "crates/star-api-rest/src/group_api/schedule_rules.rs"


def is_contained(path: Path, parent: Path) -> bool:
    """Return whether a resolved path remains within its expected workspace parent."""
    try:
        path.resolve().relative_to(parent.resolve())
        return True
    except ValueError:
        return False


def schedule_test_binary() -> Path | None:
    """Locate the test harness just compiled by the preceding Cargo no-run gate."""
    target_dir = Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target")).resolve()
    suffix = ".exe" if os.name == "nt" else ""
    candidates = [
        path.resolve()
        for path in (target_dir / "debug" / "deps").glob(f"schedule_postgres-*{suffix}")
        if path.is_file()
    ]
    return max(candidates, key=lambda path: path.stat().st_mtime_ns) if candidates else None


def run_step(name: str, command: list[str], cwd: Path = ROOT, env: dict[str, str] | None = None) -> dict[str, object]:
    """Run a bounded step with file-backed logs; never print its environment."""
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
                env=env,
                stdout=stdout,
                stderr=stderr,
                check=False,
                timeout=MAX_STEP_SECONDS,
                creationflags=subprocess.CREATE_NO_WINDOW if os.name == "nt" else 0,
            )
        return {
            "name": name,
            "status": "passed" if result.returncode == 0 else "failed",
            "exit_code": result.returncode,
            "command": command,
            "stdout_log": str(stdout_path),
            "stderr_log": str(stderr_path),
        }
    except subprocess.TimeoutExpired:
        return {
            "name": name,
            "status": "failed",
            "exit_code": 124,
            "reason": f"exceeded {MAX_STEP_SECONDS}s",
            "command": command,
        }
    except OSError as error:
        return {
            "name": name,
            "status": "blocked",
            "exit_code": 127,
            "reason": type(error).__name__,
            "command": command,
        }


class DisposablePostgres:
    """Create a unique loopback-only trust-auth cluster and preserve it if shutdown fails."""

    def __init__(self, bin_dir: Path):
        self.bin_dir = bin_dir.resolve()
        self.parent = (ROOT / ".cache" / "phase9f3-schedule").resolve()
        self.data_dir = self.parent / f"postgres-{uuid.uuid4()}"
        self.port = self._reserve_port()
        self.server_attempted = False
        self.started = False
        self.stopped = False

    @staticmethod
    def _reserve_port() -> int:
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
            sock.bind(("127.0.0.1", 0))
            return int(sock.getsockname()[1])

    def executable(self, name: str) -> Path:
        suffix = ".exe" if os.name == "nt" else ""
        path = self.bin_dir / f"{name}{suffix}"
        if not path.is_file():
            raise FileNotFoundError(name)
        return path

    def psql(self, user: str) -> list[str]:
        return [
            str(self.executable("psql")),
            "-X",
            "-h",
            "127.0.0.1",
            "-p",
            str(self.port),
            "-U",
            user,
            "-d",
            "postgres",
            "-v",
            "ON_ERROR_STOP=1",
        ]

    def connection_url(self, user: str) -> str:
        return f"postgresql://{user}@127.0.0.1:{self.port}/postgres"

    def validate_paths(self) -> None:
        self.parent.mkdir(parents=True, exist_ok=True)
        if not is_contained(self.data_dir, self.parent) or self.data_dir.exists():
            raise ValueError("disposable PostgreSQL data path is unsafe or already exists")

    def create_and_start(self) -> list[dict[str, object]]:
        self.validate_paths()
        steps: list[dict[str, object]] = []
        steps.append(
            run_step(
                "postgres-init",
                [
                    str(self.executable("initdb")),
                    "-D",
                    str(self.data_dir),
                    "-U",
                    "schedule_admin",
                    "--auth=trust",
                    "--no-instructions",
                ],
            )
        )
        if steps[-1]["status"] != "passed":
            return steps
        self.server_attempted = True
        steps.append(
            run_step(
                "postgres-start",
                [
                    str(self.executable("pg_ctl")),
                    "-D",
                    str(self.data_dir),
                    "-l",
                    str(self.data_dir / "server.log"),
                    "-o",
                    f"-h 127.0.0.1 -p {self.port}",
                    "-w",
                    "start",
                ],
            )
        )
        self.started = steps[-1]["status"] == "passed"
        return steps

    def install_runtime_role(self) -> dict[str, object]:
        role_sql = """CREATE ROLE schedule_runtime LOGIN NOSUPERUSER NOBYPASSRLS;
GRANT USAGE ON SCHEMA automation, permission, scm, multica TO schedule_runtime;
GRANT SELECT ON automation.schedule_rule_revision, automation.schedule_rule_audit,
  automation.occurrence, automation.occurrence_dispatch, automation.occurrence_event,
  automation.schedule_run_outbox
  TO schedule_runtime;
GRANT INSERT ON automation.schedule_rule_audit, automation.occurrence,
  automation.occurrence_dispatch, automation.occurrence_event,
  automation.schedule_run_outbox TO schedule_runtime;
GRANT UPDATE, DELETE ON automation.occurrence_dispatch TO schedule_runtime;
GRANT SELECT ON permission.project_role_binding, permission.cloud_branch_role_binding,
  permission.engineering_run_role_binding, scm.cloud_branch, scm.cloud_branch_revision,
  multica.engineering_run, multica.engineering_run_revision TO schedule_runtime;"""
        fixture = OUTPUT_DIR / "runtime-role.sql"
        fixture.write_text(role_sql, encoding="utf-8")
        return run_step("postgres-runtime-role", [*self.psql("schedule_admin"), "-f", str(fixture)])

    def cleanup(self) -> dict[str, object] | None:
        if not self.data_dir.exists() or not is_contained(self.data_dir, self.parent):
            self.stopped = True
            return None
        if self.server_attempted:
            stop = run_step(
                "postgres-stop",
                [
                    str(self.executable("pg_ctl")),
                    "-D",
                    str(self.data_dir),
                    "-m",
                    "fast",
                    "-w",
                    "stop",
                ],
            )
            if stop["status"] == "passed":
                self.stopped = True
            else:
                status = run_step(
                    "postgres-status-after-stop",
                    [str(self.executable("pg_ctl")), "-D", str(self.data_dir), "status"],
                )
                if status["status"] == "failed" and status.get("exit_code") == 3:
                    self.stopped = True
                else:
                    print(f"PostgreSQL stop could not be confirmed; cluster preserved at {self.data_dir}")
                    return stop
        else:
            self.stopped = True
        if self.stopped and is_contained(self.data_dir, self.parent):
            shutil.rmtree(self.data_dir)
        return None


class DisposableDockerPostgres:
    """Run a disposable PostgreSQL image with a loopback-only published port."""

    def __init__(self, docker_executable: Path, image: str):
        self.docker_executable = docker_executable.resolve()
        self.image = image
        self.name = f"star-schedule-{uuid.uuid4().hex}"
        self.port = DisposablePostgres._reserve_port()
        self.created = False
        self.started = False
        self.stopped = False

    def psql(self, user: str) -> list[str]:
        return [
            str(self.docker_executable),
            "exec",
            self.name,
            "psql",
            "-X",
            "-h",
            "127.0.0.1",
            "-U",
            user,
            "-d",
            "postgres",
            "-v",
            "ON_ERROR_STOP=1",
        ]

    def connection_url(self, user: str) -> str:
        return f"postgresql://{user}@127.0.0.1:{self.port}/postgres"

    def copy_file(self, source: Path, destination: str, step_name: str) -> dict[str, object]:
        return run_step(
            step_name,
            [str(self.docker_executable), "cp", str(source), f"{self.name}:{destination}"],
        )

    def create_and_start(self) -> list[dict[str, object]]:
        steps = [
            run_step(
                "postgres-docker-start",
                [
                    str(self.docker_executable),
                    "run",
                    "--detach",
                    "--rm",
                    "--name",
                    self.name,
                    "--env",
                    "POSTGRES_USER=schedule_admin",
                    "--env",
                    "POSTGRES_HOST_AUTH_METHOD=trust",
                    "--publish",
                    f"127.0.0.1:{self.port}:5432",
                    self.image,
                ],
            )
        ]
        self.created = steps[-1]["status"] == "passed"
        if not self.created:
            return steps
        ready: dict[str, object] | None = None
        for _ in range(90):
            ready = run_step(
                "postgres-docker-ready",
                [
                    str(self.docker_executable),
                    "exec",
                    self.name,
                    "pg_isready",
                    "-h",
                    "127.0.0.1",
                    "-U",
                    "schedule_admin",
                    "-d",
                    "postgres",
                ],
            )
            if ready["status"] == "passed":
                break
            time.sleep(1)
        if ready is None or ready["status"] != "passed":
            steps.append(
                {
                    "name": "postgres-docker-ready",
                    "status": "failed",
                    "reason": "PostgreSQL container was not ready within 90 seconds",
                }
            )
        else:
            self.started = True
            steps.append(ready)
        return steps

    def install_runtime_role(self) -> dict[str, object]:
        role_sql = """CREATE ROLE schedule_runtime LOGIN NOSUPERUSER NOBYPASSRLS;
GRANT USAGE ON SCHEMA automation, permission, scm, multica TO schedule_runtime;
GRANT SELECT ON automation.schedule_rule_revision, automation.schedule_rule_audit,
  automation.occurrence, automation.occurrence_dispatch, automation.occurrence_event,
  automation.schedule_run_outbox
  TO schedule_runtime;
GRANT INSERT ON automation.schedule_rule_audit, automation.occurrence,
  automation.occurrence_dispatch, automation.occurrence_event,
  automation.schedule_run_outbox TO schedule_runtime;
GRANT UPDATE, DELETE ON automation.occurrence_dispatch TO schedule_runtime;
GRANT SELECT ON permission.project_role_binding, permission.cloud_branch_role_binding,
  permission.engineering_run_role_binding, scm.cloud_branch, scm.cloud_branch_revision,
  multica.engineering_run, multica.engineering_run_revision TO schedule_runtime;"""
        fixture = OUTPUT_DIR / "runtime-role.sql"
        fixture.write_text(role_sql, encoding="utf-8")
        copied = self.copy_file(fixture, "/tmp/runtime-role.sql", "postgres-docker-copy-runtime-role")
        if copied["status"] != "passed":
            return copied
        return run_step(
            "postgres-runtime-role",
            [*self.psql("schedule_admin"), "-f", "/tmp/runtime-role.sql"],
        )

    def cleanup(self) -> dict[str, object] | None:
        if not self.created or self.stopped:
            self.stopped = True
            return None
        result = run_step(
            "postgres-docker-remove",
            [str(self.docker_executable), "rm", "--force", self.name],
        )
        if result["status"] == "passed":
            self.stopped = True
            return None
        print(f"PostgreSQL container cleanup could not be confirmed; container name is {self.name}")
        return result


def run_isolated_postgres(
    cargo: Path,
    bin_dir: Path | None = None,
    docker_executable: Path | None = None,
    docker_image: str | None = None,
) -> list[dict[str, object]]:
    """Apply canonical directory and Schedule migrations, then run ACL/RLS/adapter tests."""
    if bin_dir is not None:
        cluster: DisposablePostgres | DisposableDockerPostgres = DisposablePostgres(bin_dir)
    elif docker_executable is not None and docker_image is not None:
        cluster = DisposableDockerPostgres(docker_executable, docker_image)
    else:
        return [{"name": "postgres-tools", "status": "blocked", "reason": "select a local PostgreSQL bin directory or a Docker image"}]
    steps: list[dict[str, object]] = []
    try:
        steps.extend(cluster.create_and_start())
        if not cluster.started:
            return steps
        steps.append(
            run_step(
                "postgres-directory-audit-prerequisite",
                [*cluster.psql("schedule_admin"), "-c", DIRECTORY_AUDIT_PREREQUISITE_SQL],
            )
        )
        if steps[-1]["status"] != "passed":
            return steps
        for index, migration in enumerate(DIRECTORY_MIGRATIONS, start=1):
            if isinstance(cluster, DisposableDockerPostgres):
                copied = cluster.copy_file(
                    migration,
                    f"/tmp/{migration.name}",
                    f"postgres-docker-copy-directory-{index}",
                )
                steps.append(copied)
                if copied["status"] != "passed":
                    return steps
                migration_path = f"/tmp/{migration.name}"
            else:
                migration_path = str(migration)
            steps.append(
                run_step(
                    f"postgres-directory-migration-{index}",
                    [*cluster.psql("schedule_admin"), "-f", migration_path],
                )
            )
            if steps[-1]["status"] != "passed":
                return steps
        steps.append(
            run_step(
                "postgres-schedule-task-run-schema-fixture",
                [*cluster.psql("schedule_admin"), "-c", TASK_RUN_SCHEMA_FIXTURE_SQL],
            )
        )
        if steps[-1]["status"] != "passed":
            return steps
        for pass_name in ("first", "repeat"):
            if pass_name == "repeat":
                steps.append(
                    run_step(
                        "postgres-schedule-run-admission-link-fixture",
                        [*cluster.psql("schedule_admin"), "-c", SCHEDULE_ADMISSION_FIXTURE_SQL],
                    )
                )
                if steps[-1]["status"] != "passed":
                    return steps
            for index, migration in enumerate(MIGRATIONS, start=1):
                if pass_name == "first" and index == 3:
                    steps.append(
                        run_step(
                            "postgres-schedule-run-as-legacy-fixture",
                            [*cluster.psql("schedule_admin"), "-c", LEGACY_BACKFILL_FIXTURE_SQL],
                        )
                    )
                    if steps[-1]["status"] != "passed":
                        return steps
                if isinstance(cluster, DisposableDockerPostgres):
                    copied = cluster.copy_file(
                        migration,
                        f"/tmp/{migration.name}",
                        f"postgres-docker-copy-{pass_name}-{index}",
                    )
                    steps.append(copied)
                    if copied["status"] != "passed":
                        return steps
                    migration_path = f"/tmp/{migration.name}"
                else:
                    migration_path = str(migration)
                steps.append(
                    run_step(
                        f"postgres-schedule-migration-{pass_name}-{index}",
                        [*cluster.psql("schedule_admin"), "-f", migration_path],
                    )
                )
                if steps[-1]["status"] != "passed":
                    return steps
                if pass_name == "first" and index == 3:
                    steps.append(
                        run_step(
                            "postgres-schedule-run-as-legacy-backfill",
                            [*cluster.psql("schedule_admin"), "-c", LEGACY_BACKFILL_ASSERTION_SQL],
                        )
                    )
                    if steps[-1]["status"] != "passed":
                        return steps

        catalog_sql = """DO $$
DECLARE forced_count integer;
BEGIN
  SELECT count(*) INTO forced_count FROM pg_class c
  JOIN pg_namespace n ON n.oid = c.relnamespace
  WHERE n.nspname = 'automation'
    AND c.relname IN ('schedule_rule_revision','schedule_rule_audit','occurrence',
                      'occurrence_dispatch','occurrence_event','schedule_rule_outbox',
                      'schedule_rule_command_idempotency','schedule_run_outbox')
    AND c.relrowsecurity AND c.relforcerowsecurity;
  IF forced_count <> 8 THEN RAISE EXCEPTION 'expected eight FORCE RLS Schedule tables'; END IF;
END $$;"""
        steps.append(
            run_step(
                "postgres-schedule-force-rls-catalog",
                [*cluster.psql("schedule_admin"), "-c", catalog_sql],
            )
        )
        if steps[-1]["status"] != "passed":
            return steps
        steps.append(cluster.install_runtime_role())
        if steps[-1]["status"] != "passed":
            return steps

        steps.append(
            run_step(
                "postgres-schedule-run-as-directory-acl-fixture",
                [*cluster.psql("schedule_admin"), "-c", DIRECTORY_AUTH_FIXTURE_SQL],
            )
        )
        if steps[-1]["status"] != "passed":
            return steps

        runtime_env = os.environ.copy()
        runtime_env["STAR_SCHEDULE_ADMIN_DATABASE_URL"] = cluster.connection_url("schedule_admin")
        runtime_env["STAR_SCHEDULE_TEST_DATABASE_URL"] = cluster.connection_url("schedule_runtime")
        runtime_env["STAR_SCHEDULE_ACL_DATABASE_URL"] = cluster.connection_url("schedule_runtime")
        steps.append(
            run_step(
                "postgres-schedule-run-as-directory-acl-rust-test",
                [
                    str(cargo),
                    "test",
                    "--locked",
                    "-p",
                    "star-api-rest",
                    "--lib",
                    "schedule_run_as_authorization_rechecks_canonical_directory_grants",
                    "-j",
                    "1",
                    "--",
                    "--ignored",
                    "--nocapture",
                ],
                env=runtime_env,
            )
        )
        if steps[-1]["status"] != "passed":
            return steps

        outbox_rls_sql = """DO $$
DECLARE own_rows integer;
DECLARE foreign_rows integer;
BEGIN
  PERFORM set_config('app.tenant_id', '10000000-0000-4000-8000-000000000001', true);
  SELECT count(*) INTO own_rows FROM automation.schedule_run_outbox;
  PERFORM set_config('app.tenant_id', '11000000-0000-4000-8000-000000000002', true);
  SELECT count(*) INTO foreign_rows FROM automation.schedule_run_outbox;
  IF own_rows <> 1 OR foreign_rows <> 0 THEN
    RAISE EXCEPTION 'Schedule Run Outbox tenant RLS isolation failed';
  END IF;
END $$;"""
        steps.append(
            run_step(
                "postgres-schedule-run-outbox-tenant-rls",
                [*cluster.psql("schedule_runtime"), "-c", outbox_rls_sql],
            )
        )
        if steps[-1]["status"] != "passed":
            return steps

        test_binary = schedule_test_binary()
        if test_binary is None:
            steps.append(
                {
                    "name": "postgres-schedule-integration-tests",
                    "status": "blocked",
                    "reason": "schedule_postgres harness was not produced by the no-run gate",
                }
            )
        else:
            steps.append(
                run_step(
                    "postgres-schedule-integration-tests",
                    [str(test_binary), "--ignored", "--test-threads=1", "--nocapture"],
                    env=runtime_env,
                )
            )
        return steps
    finally:
        cleanup = cluster.cleanup()
        if cleanup is not None:
            steps.append(cleanup)


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Run Schedule 9F3 Rust, canonical ACL, and isolated PostgreSQL gates."
    )
    parser.add_argument("--cargo", required=True, help="Cargo executable path")
    parser.add_argument("--rustfmt", required=True, help="rustfmt executable path")
    parser.add_argument("--postgres-bin-dir", help="directory containing initdb/pg_ctl/psql")
    parser.add_argument("--postgres-docker-image", help="optional disposable PostgreSQL image; binds only to loopback")
    parser.add_argument("--docker", default="docker", help="Docker CLI used with --postgres-docker-image")
    args = parser.parse_args()

    cargo = Path(args.cargo).resolve()
    rustfmt = Path(args.rustfmt).resolve()
    postgres_bin = Path(args.postgres_bin_dir).resolve() if args.postgres_bin_dir else None
    results: list[dict[str, object]] = []
    if not cargo.is_file() or not rustfmt.is_file() or not all(
        path.is_file() for path in (*DIRECTORY_MIGRATIONS, *MIGRATIONS)
    ):
        print(json.dumps({"status": "blocked", "reason": "required Rust tool or directory/Schedule migration is unavailable"}, indent=2))
        return 2

    rustfmt_command = [str(rustfmt), "--check", "--edition", "2024", *RUST_FILES]
    api_rustfmt_command = [str(rustfmt), "--check", "--edition", "2021", API_RUST_FILE]
    cargo_commands = (
        (
            "schedule-domain-tests",
            [str(cargo), "test", "--locked", "-p", "domain-automation", "--lib", "-j", "4"],
        ),
        (
            "schedule-adapter-check",
            [str(cargo), "check", "--locked", "-p", "star-pg-adapter", "--all-targets", "-j", "4"],
        ),
        (
            "schedule-rule-api-check",
            [str(cargo), "check", "--locked", "-p", "star-api-rest", "--all-targets", "-j", "4"],
        ),
        (
            "schedule-rule-api-run-as-test",
            [str(cargo), "test", "--locked", "-p", "star-api-rest", "--lib", "schedule_rule_request_cannot_select_run_as_actor", "-j", "1"],
        ),
        (
            "schedule-rule-api-authorization-role-test",
            [str(cargo), "test", "--locked", "-p", "star-api-rest", "--lib", "task_execution_rules_do_not_grant_agent_role_schedule_authority", "-j", "1"],
        ),
        (
            "schedule-integration-test-compile",
            [str(cargo), "test", "--locked", "-p", "star-pg-adapter", "--test", "schedule_postgres", "--no-run", "-j", "4"],
        ),
        (
            "schedule-domain-clippy",
            [str(cargo), "clippy", "--locked", "-p", "domain-automation", "--all-targets", "-j", "4", "--", "-D", "warnings", "-A", "clippy::derivable_impls", "-A", "clippy::unnecessary_sort_by", "-A", "clippy::bool_assert_comparison", "--no-deps"],
        ),
        (
            "schedule-adapter-clippy",
            [
                str(cargo),
                "clippy",
                "--locked",
                "-p",
                "star-pg-adapter",
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
                "-A",
                "clippy::empty_line_after_outer_attr",
                "-A",
                "clippy::empty_line_after_doc_comments",
                "-A",
                "clippy::needless_borrows_for_generic_args",
                "-A",
                "clippy::let_unit_value",
                "--no-deps",
            ],
        ),
    )
    results.append(run_step("schedule-rustfmt", rustfmt_command))
    if results[-1]["status"] == "passed":
        results.append(run_step("schedule-rule-api-rustfmt", api_rustfmt_command))
    if results and all(result["status"] == "passed" for result in results):
        for name, command in cargo_commands:
            results.append(run_step(name, command))
            if results[-1]["status"] != "passed":
                break
    if results and all(result["status"] == "passed" for result in results):
        if postgres_bin is not None:
            if all((postgres_bin / f"{name}{'.exe' if os.name == 'nt' else ''}").is_file() for name in ("initdb", "pg_ctl", "psql")):
                results.extend(run_isolated_postgres(cargo=cargo, bin_dir=postgres_bin))
            else:
                results.append({"name": "postgres-tools", "status": "blocked", "reason": "initdb/pg_ctl/psql not found in supplied directory"})
        elif args.postgres_docker_image:
            docker_executable = shutil.which(args.docker)
            if docker_executable is None:
                results.append({"name": "postgres-docker-tools", "status": "blocked", "reason": "Docker CLI was not found"})
            else:
                results.extend(
                    run_isolated_postgres(
                        cargo=cargo,
                        docker_executable=Path(docker_executable),
                        docker_image=args.postgres_docker_image,
                    )
                )
        else:
            results.append({"name": "postgres-tools", "status": "blocked", "reason": "supply --postgres-bin-dir or --postgres-docker-image"})

    print(json.dumps({"status": "passed" if results and all(result["status"] == "passed" for result in results) else "failed", "results": results}, indent=2, ensure_ascii=False))
    return 0 if results and all(result["status"] == "passed" for result in results) else 1


if __name__ == "__main__":
    raise SystemExit(main())
