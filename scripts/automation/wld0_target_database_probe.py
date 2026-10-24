# @cypher schema=1 source_sha256=7b888a6c9d09f5a676ed562f7cb7a204fd109797a6f204bac274b4e5fc09d330
# MERGE (self:File {path:"scripts/automation/wld0_target_database_probe.py"})
# MERGE (validate:Symbol {id:"scripts/automation/wld0_target_database_probe.py::validate_inputs",kind:"function"})
# MERGE (command:Symbol {id:"scripts/automation/wld0_target_database_probe.py::build_psql_command",kind:"function"})
# MERGE (parse:Symbol {id:"scripts/automation/wld0_target_database_probe.py::parse_psql_output",kind:"function"})
# MERGE (inspect:Symbol {id:"scripts/automation/wld0_target_database_probe.py::inspect_snapshot",kind:"function"})
# MERGE (probe:Symbol {id:"scripts/automation/wld0_target_database_probe.py::run_probe",kind:"function"})
# MERGE (main:Symbol {id:"scripts/automation/wld0_target_database_probe.py::main",kind:"function"})
# MERGE (sql:Symbol {id:"scripts/automation/wld0_target_database_probe.py::PSQL_SCRIPT",kind:"constant"})
# MERGE (objects:Symbol {id:"scripts/automation/wld0_target_database_probe.py::EXPECTED_OBJECTS",kind:"constant"})
# MERGE (locks:Symbol {id:"scripts/automation/wld0_target_database_probe.py::LOCK_GRANTS",kind:"constant"})
# MERGE (privileges:Symbol {id:"scripts/automation/wld0_target_database_probe.py::OWNER_TABLE_PRIVILEGES",kind:"constant"})
# MERGE (rowsLimit:Symbol {id:"scripts/automation/wld0_target_database_probe.py::MAX_OUTPUT_ROWS",kind:"constant"})
# MERGE (self)-[:DEFINES]->(validate)
# MERGE (self)-[:DEFINES]->(command)
# MERGE (self)-[:DEFINES]->(parse)
# MERGE (self)-[:DEFINES]->(inspect)
# MERGE (self)-[:DEFINES]->(probe)
# MERGE (self)-[:DEFINES]->(main)
# MERGE (self)-[:DEFINES]->(sql)
# MERGE (self)-[:DEFINES]->(objects)
# MERGE (self)-[:DEFINES]->(locks)
# MERGE (self)-[:DEFINES]->(privileges)
# MERGE (self)-[:DEFINES]->(rowsLimit)
# MERGE (command)-[:READS]->(sql)
# MERGE (command)-[:CALLS]->(validate)
# MERGE (jsonLoads:ExternalService {name:"Python stdlib json.loads"})
# MERGE (shutilWhich:ExternalService {name:"Python stdlib shutil.which"})
# MERGE (subprocessRun:ExternalService {name:"Python stdlib subprocess.run"})
# MERGE (parseArgs:ExternalService {name:"Python stdlib argparse.ArgumentParser.parse_args"})
# MERGE (jsonDumps:ExternalService {name:"Python stdlib json.dumps"})
# MERGE (parse)-[:CALLS]->(jsonLoads)
# MERGE (inspect)-[:CALLS]->(parse)
# MERGE (inspect)-[:READS]->(objects)
# MERGE (inspect)-[:READS]->(privileges)
# MERGE (probe)-[:CALLS]->(validate)
# MERGE (probe)-[:CALLS]->(command)
# MERGE (probe)-[:CALLS]->(shutilWhich)
# MERGE (probe)-[:CALLS]->(subprocessRun)
# MERGE (probe)-[:CALLS]->(parse)
# MERGE (probe)-[:CALLS]->(inspect)
# MERGE (main)-[:CALLS]->(parseArgs)
# MERGE (main)-[:CALLS]->(probe)
# MERGE (main)-[:CALLS]->(jsonDumps)
# @endcypher
"""Read-only evidence probe for the configured WLD-0 PostgreSQL target."""
from __future__ import annotations

import argparse
import ipaddress
import json
import re
import shutil
import subprocess
import sys
from typing import Any


EXPECTED_OBJECTS = (
    ("permission", "project_role_binding"),
    ("scm", "cloud_branch"),
    ("scm", "cloud_branch_revision"),
    ("permission", "cloud_branch_role_binding"),
    ("multica", "engineering_run"),
    ("multica", "engineering_run_revision"),
    ("permission", "engineering_run_role_binding"),
    ("multica", "worktree_project_binding"),
    ("public", "worktree_canvas_worktree"),
    ("multica", "task_metadata"),
    ("multica", "work_item_worktree"),
    ("multica", "engineering_run_worktree_binding"),
    ("multica", "engineering_directory_event"),
    ("multica", "engineering_run_command_idempotency"),
    ("multica", "engineering_run_outbox"),
)
LOCK_GRANTS = (
    ("permission", "project_role_binding", "valid_to", "true"),
    ("scm", "cloud_branch_revision", "valid_to", "false"),
    ("permission", "cloud_branch_role_binding", "valid_to", "false"),
    ("permission", "engineering_run_role_binding", "valid_to", "true"),
    ("multica", "engineering_run_revision", "valid_to", "true"),
    ("multica", "worktree_project_binding", "valid_to", "true"),
    ("public", "worktree_canvas_worktree", "id", "true"),
    ("multica", "task_metadata", "valid_to", "false"),
    ("multica", "work_item_worktree", "valid_to", "false"),
)
OWNER_TABLE_PRIVILEGES = {
    ("permission", "project_role_binding"): (True, False, False, False, False, False, False),
    ("scm", "cloud_branch"): (True, False, False, False, False, False, False),
    ("scm", "cloud_branch_revision"): (True, False, False, False, False, False, False),
    ("permission", "cloud_branch_role_binding"): (True, False, False, False, False, False, False),
    ("multica", "engineering_run"): (True, False, False, False, False, False, False),
    ("multica", "engineering_run_revision"): (True, False, False, False, False, False, False),
    ("permission", "engineering_run_role_binding"): (True, False, False, False, False, False, False),
    ("multica", "worktree_project_binding"): (True, False, False, False, False, False, False),
    ("public", "worktree_canvas_worktree"): (True, False, False, False, False, False, False),
    ("multica", "task_metadata"): (True, False, False, False, False, False, False),
    ("multica", "work_item_worktree"): (True, False, False, False, False, False, False),
    ("multica", "engineering_run_worktree_binding"): (True, True, False, False, False, False, False),
    ("multica", "engineering_directory_event"): (False, True, False, False, False, False, False),
    ("multica", "engineering_run_command_idempotency"): (True, True, False, True, False, False, False),
    ("multica", "engineering_run_outbox"): (False, True, False, False, False, False, False),
}
MAX_LEDGER_ROWS = 256
MAX_OUTPUT_BYTES = 128 * 1024
MAX_OUTPUT_ROWS = MAX_LEDGER_ROWS + len(EXPECTED_OBJECTS) + len(LOCK_GRANTS) + 8
PSQL_TIMEOUT_SECONDS = 8
SERVICE_RE = re.compile(r"[A-Za-z0-9_.-]{1,63}\Z")
IDENTITY_RE = re.compile(r"[A-Za-z_][A-Za-z0-9_$-]{0,62}\Z")


PSQL_SCRIPT = r"""\set ON_ERROR_STOP on
BEGIN ISOLATION LEVEL REPEATABLE READ READ ONLY;
SET LOCAL statement_timeout = '4000ms';
SET LOCAL lock_timeout = '1000ms';
SET LOCAL idle_in_transaction_session_timeout = '6000ms';
SET LOCAL search_path = pg_catalog;
SELECT json_build_object(
  'kind','identity','database',current_database(),'user',session_user,
  'address',inet_server_addr()::text,'port',inet_server_port()
)::text;
SELECT json_build_object(
  'kind','login_role','name',r.rolname,'can_login',r.rolcanlogin,
  'superuser',r.rolsuper,'bypass_rls',r.rolbypassrls,'inherit',r.rolinherit,
  'createdb',r.rolcreatedb,'createrole',r.rolcreaterole,'replication',r.rolreplication,
  'owner_member',coalesce(pg_catalog.pg_has_role(session_user::regrole::oid,
    (SELECT oid FROM pg_catalog.pg_roles WHERE rolname = 'star_engineering_directory_owner'),
    'MEMBER'),false)
)::text
FROM pg_catalog.pg_roles r WHERE r.rolname = session_user;
SELECT json_build_object(
  'kind','owner_role','present',count(*) = 1,
  'can_login',coalesce(bool_or(rolcanlogin),false),
  'superuser',coalesce(bool_or(rolsuper),false),
  'bypass_rls',coalesce(bool_or(rolbypassrls),false),
  'inherit',coalesce(bool_or(rolinherit),false),
  'createdb',coalesce(bool_or(rolcreatedb),false),
  'createrole',coalesce(bool_or(rolcreaterole),false),
  'replication',coalesce(bool_or(rolreplication),false),
  'has_memberships',EXISTS (SELECT 1 FROM pg_catalog.pg_auth_members m
    WHERE m.member=(SELECT oid FROM pg_catalog.pg_roles
      WHERE rolname='star_engineering_directory_owner')),
  'owns_objects',EXISTS (SELECT 1 FROM pg_catalog.pg_shdepend d
    WHERE d.refclassid='pg_catalog.pg_authid'::pg_catalog.regclass
      AND d.refobjid=(SELECT oid FROM pg_catalog.pg_roles
        WHERE rolname='star_engineering_directory_owner') AND d.deptype='o'),
  'direct_schema_create',EXISTS (
    SELECT 1 FROM pg_catalog.pg_namespace n
    CROSS JOIN LATERAL pg_catalog.aclexplode(coalesce(n.nspacl,
      pg_catalog.acldefault('n',n.nspowner))) a
    WHERE n.nspname IN ('public','scm','permission','multica')
      AND a.grantee=(SELECT oid FROM pg_catalog.pg_roles
        WHERE rolname='star_engineering_directory_owner')
      AND a.privilege_type='CREATE'),
  'direct_database_privileges',EXISTS (
    SELECT 1 FROM pg_catalog.pg_database d
    CROSS JOIN LATERAL pg_catalog.aclexplode(coalesce(d.datacl,
      pg_catalog.acldefault('d',d.datdba))) a
    WHERE d.datname=current_database()
      AND a.grantee=(SELECT oid FROM pg_catalog.pg_roles
        WHERE rolname='star_engineering_directory_owner'))
)::text
FROM pg_catalog.pg_roles WHERE rolname = 'star_engineering_directory_owner';
SELECT
  (SELECT count(*) FROM pg_catalog.pg_class c
    WHERE c.relname IN ('schema_migrations','_sqlx_migrations') AND c.relkind IN ('r','p'))
      AS ledger_candidate_count,
  (SELECT pg_catalog.to_regclass('public.schema_migrations') IS NOT NULL)
      AS star_ledger_present,
  (
    (SELECT count(*) FROM pg_catalog.pg_class c
      WHERE c.relname IN ('schema_migrations','_sqlx_migrations') AND c.relkind IN ('r','p')) = 1
    AND pg_catalog.to_regclass('public.schema_migrations') IS NOT NULL
    AND (SELECT count(*) FROM pg_catalog.pg_attribute a
      WHERE a.attrelid = pg_catalog.to_regclass('public.schema_migrations')
        AND a.attnum > 0 AND NOT a.attisdropped) = 2
    AND EXISTS (SELECT 1 FROM pg_catalog.pg_attribute a
      WHERE a.attrelid = pg_catalog.to_regclass('public.schema_migrations')
        AND a.attname = 'version' AND a.attnum > 0 AND NOT a.attisdropped
        AND pg_catalog.format_type(a.atttypid,a.atttypmod) = 'text' AND a.attnotnull)
    AND EXISTS (SELECT 1 FROM pg_catalog.pg_attribute a
      WHERE a.attrelid = pg_catalog.to_regclass('public.schema_migrations')
        AND a.attname = 'applied_at' AND a.attnum > 0 AND NOT a.attisdropped
        AND pg_catalog.format_type(a.atttypid,a.atttypmod) = 'timestamp with time zone'
        AND a.attnotnull)
    AND EXISTS (SELECT 1 FROM pg_catalog.pg_constraint k
      JOIN pg_catalog.pg_attribute a ON a.attrelid = k.conrelid AND a.attname = 'version'
        AND a.attnum > 0 AND NOT a.attisdropped
      WHERE k.conrelid = pg_catalog.to_regclass('public.schema_migrations') AND k.contype = 'p'
        AND k.conkey = ARRAY[a.attnum]::smallint[])
  ) AS ledger_v1_known
\gset
SELECT json_build_object(
  'kind','ledger_meta','candidate_count',:ledger_candidate_count,
  'star_ledger_present',:'star_ledger_present',
  'recognized_star_v1',:'ledger_v1_known'
)::text;
\if :ledger_v1_known
SELECT json_build_object(
  'kind','ledger_entry','version',left(version,64),
  'version_truncated',length(version) > 64,'applied_at',applied_at::text
)::text
FROM public.schema_migrations LIMIT 257;
\else
SELECT json_build_object('kind','ledger_unknown')::text;
\endif
WITH lock_targets(schema_name,table_name,column_name,expected_using) AS (VALUES
  ('permission','project_role_binding','valid_to','true'),
  ('scm','cloud_branch_revision','valid_to','false'),
  ('permission','cloud_branch_role_binding','valid_to','false'),
  ('permission','engineering_run_role_binding','valid_to','true'),
  ('multica','engineering_run_revision','valid_to','true'),
  ('multica','worktree_project_binding','valid_to','true'),
  ('public','worktree_canvas_worktree','id','true'),
  ('multica','task_metadata','valid_to','false'),
  ('multica','work_item_worktree','valid_to','false')
)
SELECT json_build_object(
  'kind','lock_grant','schema',t.schema_name,'name',t.table_name,'column',t.column_name,
  'expected_using',t.expected_using,
  'owner_select',coalesce(pg_catalog.has_table_privilege(
    (SELECT oid FROM pg_catalog.pg_roles WHERE rolname='star_engineering_directory_owner'),c.oid,'SELECT'),false),
  'owner_column_update',coalesce(pg_catalog.has_column_privilege(
    (SELECT oid FROM pg_catalog.pg_roles WHERE rolname='star_engineering_directory_owner'),
    format('%I.%I',t.schema_name,t.table_name),t.column_name,'UPDATE'),false),
  'restrictive_update_denial',EXISTS (
    SELECT 1 FROM pg_catalog.pg_policy p
    WHERE p.polrelid=c.oid AND p.polcmd='w' AND NOT p.polpermissive
      AND (SELECT oid FROM pg_catalog.pg_roles WHERE rolname='star_engineering_directory_owner') = ANY(p.polroles)
      AND pg_catalog.pg_get_expr(p.polqual,p.polrelid)=t.expected_using
      AND pg_catalog.pg_get_expr(p.polwithcheck,p.polrelid)='false'
  )
)::text
FROM lock_targets t
LEFT JOIN pg_catalog.pg_namespace n ON n.nspname=t.schema_name
LEFT JOIN pg_catalog.pg_class c ON c.relnamespace=n.oid AND c.relname=t.table_name
ORDER BY t.schema_name,t.table_name;
WITH expected(schema_name,table_name) AS (VALUES
  ('permission','project_role_binding'),
  ('scm','cloud_branch'),
  ('scm','cloud_branch_revision'),
  ('permission','cloud_branch_role_binding'),
  ('multica','engineering_run'),
  ('multica','engineering_run_revision'),
  ('permission','engineering_run_role_binding'),
  ('multica','worktree_project_binding'),
  ('public','worktree_canvas_worktree'),
  ('multica','task_metadata'),
  ('multica','work_item_worktree'),
  ('multica','engineering_run_worktree_binding'),
  ('multica','engineering_directory_event'),
  ('multica','engineering_run_command_idempotency'),
  ('multica','engineering_run_outbox')
)
SELECT json_build_object(
  'kind','object','schema',e.schema_name,'name',e.table_name,
  'present',c.oid IS NOT NULL,'row_security',c.relrowsecurity,
  'force_row_security',c.relforcerowsecurity,
  'owner',pg_catalog.pg_get_userbyid(c.relowner),
  'policy_count',(SELECT count(*) FROM pg_catalog.pg_policy p WHERE p.polrelid = c.oid),
  'login_effective_privileges',CASE WHEN c.oid IS NULL THEN NULL ELSE json_build_object(
    'select',pg_catalog.has_table_privilege(session_user,c.oid,'SELECT'),
    'insert',pg_catalog.has_table_privilege(session_user,c.oid,'INSERT'),
    'update',pg_catalog.has_table_privilege(session_user,c.oid,'UPDATE'),
    'delete',pg_catalog.has_table_privilege(session_user,c.oid,'DELETE'),
    'truncate',pg_catalog.has_table_privilege(session_user,c.oid,'TRUNCATE'),
    'references',pg_catalog.has_table_privilege(session_user,c.oid,'REFERENCES'),
    'trigger',pg_catalog.has_table_privilege(session_user,c.oid,'TRIGGER')
  ) END,
  'owner_effective_privileges',CASE WHEN c.oid IS NULL THEN NULL ELSE json_build_object(
    'select',coalesce(pg_catalog.has_table_privilege(
      (SELECT oid FROM pg_catalog.pg_roles WHERE rolname='star_engineering_directory_owner'),c.oid,'SELECT'),false),
    'insert',coalesce(pg_catalog.has_table_privilege(
      (SELECT oid FROM pg_catalog.pg_roles WHERE rolname='star_engineering_directory_owner'),c.oid,'INSERT'),false),
    'update',coalesce(pg_catalog.has_table_privilege(
      (SELECT oid FROM pg_catalog.pg_roles WHERE rolname='star_engineering_directory_owner'),c.oid,'UPDATE'),false),
    'delete',coalesce(pg_catalog.has_table_privilege(
      (SELECT oid FROM pg_catalog.pg_roles WHERE rolname='star_engineering_directory_owner'),c.oid,'DELETE'),false),
    'truncate',coalesce(pg_catalog.has_table_privilege(
      (SELECT oid FROM pg_catalog.pg_roles WHERE rolname='star_engineering_directory_owner'),c.oid,'TRUNCATE'),false),
    'references',coalesce(pg_catalog.has_table_privilege(
      (SELECT oid FROM pg_catalog.pg_roles WHERE rolname='star_engineering_directory_owner'),c.oid,'REFERENCES'),false),
    'trigger',coalesce(pg_catalog.has_table_privilege(
      (SELECT oid FROM pg_catalog.pg_roles WHERE rolname='star_engineering_directory_owner'),c.oid,'TRIGGER'),false)
  ) END,
  'owner_column_valid_to_update',CASE WHEN c.oid IS NULL THEN false
    WHEN e.schema_name='multica' AND e.table_name='engineering_run_worktree_binding'
      THEN EXISTS (SELECT 1 FROM pg_catalog.pg_attribute a
        WHERE a.attrelid=c.oid AND a.attname='valid_to' AND a.attnum>0 AND NOT a.attisdropped)
        AND pg_catalog.has_column_privilege(
          (SELECT oid FROM pg_catalog.pg_roles WHERE rolname='star_engineering_directory_owner'),
          format('%I.%I',e.schema_name,e.table_name),'valid_to','UPDATE')
    ELSE false END,
  'owner_column_expires_at_update',CASE WHEN c.oid IS NULL THEN false
    WHEN e.schema_name='multica' AND e.table_name='engineering_run_command_idempotency'
      THEN EXISTS (SELECT 1 FROM pg_catalog.pg_attribute a
        WHERE a.attrelid=c.oid AND a.attname='expires_at' AND a.attnum>0 AND NOT a.attisdropped)
        AND pg_catalog.has_column_privilege(
          (SELECT oid FROM pg_catalog.pg_roles WHERE rolname='star_engineering_directory_owner'),
          format('%I.%I',e.schema_name,e.table_name),'expires_at','UPDATE')
    ELSE false END
)::text
FROM expected e
LEFT JOIN pg_catalog.pg_namespace n ON n.nspname = e.schema_name
LEFT JOIN pg_catalog.pg_class c ON c.relnamespace = n.oid AND c.relname = e.table_name
ORDER BY e.schema_name,e.table_name;
SELECT coalesce(pg_catalog.pg_has_role(session_user::regrole::oid,
  (SELECT oid FROM pg_catalog.pg_roles WHERE rolname='star_engineering_directory_owner'),
  'MEMBER'),false) AS owner_member
\gset
\if :owner_member
SET LOCAL ROLE star_engineering_directory_owner;
SELECT json_build_object('kind','role_switch','active',current_user='star_engineering_directory_owner',
  'login',session_user)::text;
\else
SELECT json_build_object('kind','role_switch','active',false,'login',session_user)::text;
\endif
COMMIT;
"""


def validate_inputs(args: argparse.Namespace) -> tuple[dict[str, Any] | None, str | None]:
    """Validate the libpq service selector and all required target identity pins."""
    service = args.service
    if not isinstance(service, str) or not SERVICE_RE.fullmatch(service):
        return None, "missing_or_invalid_service_name"

    database = args.expected_database
    login = args.expected_login
    address = args.expected_address
    port = args.expected_port
    if not all(isinstance(value, str) and value for value in (database, login, address, port)):
        return None, "missing_target_identity_pin"
    if not IDENTITY_RE.fullmatch(database) or not IDENTITY_RE.fullmatch(login):
        return None, "invalid_target_identity_pin"
    try:
        normalized_address = str(ipaddress.ip_address(address))
        normalized_port = int(port, 10)
    except (ValueError, TypeError):
        return None, "invalid_target_identity_pin"
    if not 1 <= normalized_port <= 65535:
        return None, "invalid_target_identity_pin"
    return {
        "service": service,
        "database": database,
        "login": login,
        "address": normalized_address,
        "port": normalized_port,
    }, None


def build_psql_command(psql_path: str, service: str) -> list[str]:
    """Build the fixed psql argv; the only connection selector is a validated service name."""
    if not SERVICE_RE.fullmatch(service):
        raise ValueError("invalid service name")
    return [
        psql_path,
        "-X",
        "-q",
        "-A",
        "-t",
        "-w",
        "--set=ON_ERROR_STOP=1",
        "--file=-",
        f"--dbname=service={service}",
    ]


def parse_psql_output(stdout: str) -> tuple[list[dict[str, Any]] | None, str | None]:
    """Parse a bounded stream of JSON catalog rows and reject unexpected output."""
    if not isinstance(stdout, str) or len(stdout.encode("utf-8", errors="replace")) > MAX_OUTPUT_BYTES:
        return None, "probe_output_limit_exceeded"
    rows: list[dict[str, Any]] = []
    for line in stdout.splitlines():
        if not line:
            continue
        try:
            row = json.loads(line)
        except (json.JSONDecodeError, TypeError):
            return None, "probe_output_unrecognized"
        if not isinstance(row, dict) or not isinstance(row.get("kind"), str):
            return None, "probe_output_unrecognized"
        rows.append(row)
        if len(rows) > MAX_OUTPUT_ROWS:
            return None, "probe_output_limit_exceeded"
    return rows, None


def inspect_snapshot(
    rows: list[dict[str, Any]], expected: dict[str, Any]
) -> dict[str, Any]:
    """Assess identity, ledger shape, role attributes, objects, and RLS without inferring deployment."""
    by_kind: dict[str, list[dict[str, Any]]] = {}
    for row in rows:
        by_kind.setdefault(row["kind"], []).append(row)

    required_singletons = ("identity", "login_role", "owner_role", "ledger_meta", "role_switch")
    if any(len(by_kind.get(kind, [])) != 1 for kind in required_singletons):
        return {"status": "UNKNOWN", "reason": "incomplete_catalog_snapshot"}
    identity = by_kind["identity"][0]
    observed_address = identity.get("address")
    try:
        normalized_observed_address = str(ipaddress.ip_address(observed_address))
    except (ValueError, TypeError):
        return {"status": "UNKNOWN", "reason": "target_identity_unavailable"}
    if (
        identity.get("database") != expected["database"]
        or identity.get("user") != expected["login"]
        or normalized_observed_address != expected["address"]
        or identity.get("port") != expected["port"]
    ):
        return {"status": "BLOCKED", "reason": "target_identity_mismatch"}

    object_rows = by_kind.get("object", [])
    expected_objects = set(EXPECTED_OBJECTS)
    observed_objects = {(row.get("schema"), row.get("name")) for row in object_rows}
    if len(object_rows) != len(EXPECTED_OBJECTS) or observed_objects != expected_objects:
        return {"status": "UNKNOWN", "reason": "incomplete_wld_object_snapshot"}

    ledger_meta = by_kind["ledger_meta"][0]
    ledger_entries = by_kind.get("ledger_entry", [])
    ledger_known = ledger_meta.get("recognized_star_v1") == "t"
    if not ledger_known or by_kind.get("ledger_unknown"):
        return {
            "status": "UNKNOWN",
            "reason": "missing_or_unrecognized_migration_ledger",
            "identity": identity,
            "login_role": by_kind["login_role"][0],
            "owner_role": by_kind["owner_role"][0],
            "objects": object_rows,
            "ledger": {"recognized": False, "candidate_count": ledger_meta.get("candidate_count")},
        }
    if len(ledger_entries) > MAX_LEDGER_ROWS or any(
        row.get("version_truncated") for row in ledger_entries
    ):
        return {"status": "UNKNOWN", "reason": "migration_ledger_snapshot_truncated"}
    if any(not isinstance(row.get("version"), str) or not isinstance(row.get("applied_at"), str)
           for row in ledger_entries):
        return {"status": "UNKNOWN", "reason": "migration_ledger_entry_unrecognized"}

    login_role = by_kind["login_role"][0]
    owner_role = by_kind["owner_role"][0]
    if login_role.get("name") != expected["login"]:
        return {"status": "UNKNOWN", "reason": "login_role_snapshot_mismatch"}
    if (
        not login_role.get("can_login")
        or login_role.get("superuser")
        or login_role.get("bypass_rls")
        or login_role.get("inherit")
        or login_role.get("createdb")
        or login_role.get("createrole")
        or login_role.get("replication")
    ):
        return {"status": "BLOCKED", "reason": "login_role_not_safe_for_runtime_evidence"}
    if not owner_role.get("present"):
        return {"status": "BLOCKED", "reason": "engineering_directory_owner_role_missing"}
    if (
        owner_role.get("can_login")
        or owner_role.get("superuser")
        or owner_role.get("bypass_rls")
        or owner_role.get("inherit")
        or owner_role.get("createdb")
        or owner_role.get("createrole")
        or owner_role.get("replication")
        or owner_role.get("has_memberships") is not False
        or owner_role.get("owns_objects") is not False
        or owner_role.get("direct_schema_create") is not False
        or owner_role.get("direct_database_privileges") is not False
    ):
        return {"status": "BLOCKED", "reason": "engineering_directory_owner_role_attributes"}
    role_switch = by_kind["role_switch"][0]
    if (
        login_role.get("owner_member") is not True
        or role_switch.get("active") is not True
        or role_switch.get("login") != expected["login"]
    ):
        return {"status": "BLOCKED", "reason": "authenticated_owner_role_switch_failed"}

    lock_rows = by_kind.get("lock_grant", [])
    expected_lock_targets = set(LOCK_GRANTS)
    observed_lock_targets = {
        (row.get("schema"), row.get("name"), row.get("column"), row.get("expected_using"))
        for row in lock_rows
    }
    if len(lock_rows) != len(LOCK_GRANTS):
        return {"status": "UNKNOWN", "reason": "incomplete_lock_grant_snapshot"}
    if observed_lock_targets != expected_lock_targets or any(
        row.get("owner_select") is not True
        or row.get("owner_column_update") is not True
        or row.get("restrictive_update_denial") is not True
        for row in lock_rows
    ):
        return {"status": "BLOCKED", "reason": "required_lock_only_grants_or_restrictive_policy_missing"}

    for row in object_rows:
        if row.get("present") is not True:
            return {"status": "BLOCKED", "reason": "required_wld_object_missing"}
        if not isinstance(row.get("row_security"), bool) or not isinstance(
            row.get("force_row_security"), bool
        ):
            return {"status": "UNKNOWN", "reason": "wld_rls_catalog_state_unavailable"}
        if not row["row_security"] or not row["force_row_security"]:
            return {"status": "BLOCKED", "reason": "wld_rls_or_force_rls_disabled"}
        expected_privileges = OWNER_TABLE_PRIVILEGES[(row["schema"], row["name"])]
        privilege_record = row.get("owner_effective_privileges")
        privilege_names = (
            "select", "insert", "update", "delete", "truncate", "references", "trigger"
        )
        if not isinstance(privilege_record, dict) or any(
            not isinstance(privilege_record.get(name), bool) for name in privilege_names
        ):
            return {"status": "UNKNOWN", "reason": "owner_table_privilege_snapshot_unavailable"}
        if tuple(privilege_record[name] for name in privilege_names) != expected_privileges:
            return {"status": "BLOCKED", "reason": "owner_table_privilege_contract_mismatch"}
    binding = next(
        row for row in object_rows
        if (row["schema"], row["name"]) == ("multica", "engineering_run_worktree_binding")
    )
    idempotency = next(
        row for row in object_rows
        if (row["schema"], row["name"]) == ("multica", "engineering_run_command_idempotency")
    )
    if binding.get("owner_column_valid_to_update") is not True or idempotency.get(
        "owner_column_expires_at_update"
    ) is not True:
        return {"status": "BLOCKED", "reason": "owner_mutation_column_grant_missing"}

    return {
        "status": "OBSERVED_READ_ONLY",
        "reason": "bounded_target_catalog_snapshot_collected",
        "identity": identity,
        "login_role": login_role,
        "owner_role": owner_role,
        "role_switch": role_switch,
        "ledger": {
            "recognized": True,
            "format": "public.schema_migrations(version text, applied_at timestamptz)",
            "row_count": len(ledger_entries),
            "entries": ledger_entries,
            "stores_source_checksum": False,
        },
        "objects": object_rows,
        "lock_grants": lock_rows,
        "migration_reconciliation": {
            "status": "BLOCKED",
            "reason": "probe_has_no_pinned_checksum_or_complete_catalog_fingerprint_manifest",
        },
        "authenticated_binding_checks": "NOT_PERFORMED_READ_ONLY_PROBE",
    }


def run_probe(
    args: argparse.Namespace,
    *,
    which: Any = None,
    runner: Any = None,
) -> tuple[dict[str, Any], int]:
    """Run one bounded psql process; never expose command, environment, or stderr."""
    expected, validation_error = validate_inputs(args)
    if validation_error:
        return {"status": "BLOCKED", "reason": validation_error}, 2
    which = which or shutil.which
    runner = runner or subprocess.run
    psql_path = which("psql")
    if not psql_path:
        return {"status": "BLOCKED", "reason": "psql_not_found"}, 2

    try:
        completed = runner(
            build_psql_command(psql_path, expected["service"]),
            input=PSQL_SCRIPT,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            text=True,
            encoding="utf-8",
            errors="replace",
            timeout=PSQL_TIMEOUT_SECONDS,
            check=False,
            shell=False,
        )
    except subprocess.TimeoutExpired:
        return {"status": "BLOCKED", "reason": "probe_timeout"}, 2
    except OSError:
        return {"status": "BLOCKED", "reason": "psql_execution_failed"}, 2
    if completed.returncode != 0:
        return {"status": "BLOCKED", "reason": "database_probe_failed"}, 2

    rows, parse_error = parse_psql_output(completed.stdout)
    if parse_error:
        return {"status": "UNKNOWN", "reason": parse_error}, 3
    result = inspect_snapshot(rows, expected)
    if result["status"] == "UNKNOWN":
        return result, 3
    if result["status"] == "BLOCKED":
        return result, 2
    return result, 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Collect a bounded, read-only catalog snapshot from a pinned WLD-0 PostgreSQL target."
    )
    parser.add_argument("--service", help="Explicit libpq service name; never a URL or connection string.")
    parser.add_argument("--expected-database")
    parser.add_argument("--expected-login")
    parser.add_argument("--expected-address", help="Expected PostgreSQL server IP address.")
    parser.add_argument("--expected-port", help="Expected PostgreSQL server port.")
    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    result, exit_code = run_probe(args)
    print(json.dumps(result, sort_keys=True, separators=(",", ":")))
    return exit_code


if __name__ == "__main__":
    sys.exit(main())
