"""Run authenticated Run-to-Worktree HTTP acceptance on an isolated local PostgreSQL cluster."""

# @cypher schema=1 source_sha256=4a9d3fa89127231d6dbe95ff502cfbe2b1a7af95a08ed4ecd4c519a834109c1c
# MERGE (self:File {path:"scripts/automation/phase9f2_binding_http_acceptance.py"})
# MERGE (runnerRoot:Config {id:"scripts/automation/phase9f2_binding_http_acceptance.py::RUNNER_ROOT",kind:"path"})
# MERGE (protectedPorts:Config {id:"scripts/automation/phase9f2_binding_http_acceptance.py::PROTECTED_LOOPBACK_PORTS",kind:"port_set"})
# MERGE (deviceTestUrl:Config {id:"STAR_W16_DEVICE_AUTH_TEST_DATABASE_URL",kind:"environment_variable"})
# MERGE (main:Symbol {id:"scripts/automation/phase9f2_binding_http_acceptance.py::main",kind:"function"})
# MERGE (port:Symbol {id:"scripts/automation/phase9f2_binding_http_acceptance.py::select_loopback_port",kind:"function"})
# MERGE (environment:Symbol {id:"scripts/automation/phase9f2_binding_http_acceptance.py::sanitized_environment",kind:"function"})
# MERGE (step:Symbol {id:"scripts/automation/phase9f2_binding_http_acceptance.py::run_step",kind:"function"})
# MERGE (cluster:Symbol {id:"scripts/automation/phase9f2_binding_http_acceptance.py::start_cluster",kind:"function"})
# MERGE (psql:Symbol {id:"scripts/automation/phase9f2_binding_http_acceptance.py::psql_command",kind:"function"})
# MERGE (sql:Symbol {id:"scripts/automation/phase9f2_binding_http_acceptance.py::run_sql",kind:"function"})
# MERGE (migrations:Symbol {id:"scripts/automation/phase9f2_binding_http_acceptance.py::apply_migrations",kind:"function"})
# MERGE (fixture:Symbol {id:"scripts/automation/phase9f2_binding_http_acceptance.py::seed_fixture",kind:"function"})
# MERGE (deviceChecks:Symbol {id:"scripts/automation/phase9f2_binding_http_acceptance.py::device_owner_catalog_checks",kind:"function"})
# MERGE (devicePgTest:Symbol {id:"scripts/automation/phase9f2_binding_http_acceptance.py::run_device_auth_pg_tests",kind:"function"})
# MERGE (snapshot:Symbol {id:"scripts/automation/phase9f2_binding_http_acceptance.py::read_snapshot",kind:"function"})
# MERGE (assertion:Symbol {id:"scripts/automation/phase9f2_binding_http_acceptance.py::assert_snapshot_delta",kind:"function"})
# MERGE (test:Symbol {id:"scripts/automation/phase9f2_binding_http_acceptance.py::run_rust_phase",kind:"function"})
# MERGE (fault:Symbol {id:"scripts/automation/phase9f2_binding_http_acceptance.py::set_detach_failure",kind:"function"})
# MERGE (stop:Symbol {id:"scripts/automation/phase9f2_binding_http_acceptance.py::stop_and_remove_cluster",kind:"function"})
# MERGE (self)-[:DEFINES]->(runnerRoot)
# MERGE (self)-[:DEFINES]->(protectedPorts)
# MERGE (self)-[:DEFINES]->(main)
# MERGE (self)-[:DEFINES]->(port)
# MERGE (self)-[:DEFINES]->(environment)
# MERGE (self)-[:DEFINES]->(step)
# MERGE (self)-[:DEFINES]->(cluster)
# MERGE (self)-[:DEFINES]->(psql)
# MERGE (self)-[:DEFINES]->(sql)
# MERGE (self)-[:DEFINES]->(migrations)
# MERGE (self)-[:DEFINES]->(fixture)
# MERGE (self)-[:DEFINES]->(deviceChecks)
# MERGE (self)-[:DEFINES]->(devicePgTest)
# MERGE (self)-[:DEFINES]->(snapshot)
# MERGE (self)-[:DEFINES]->(assertion)
# MERGE (self)-[:DEFINES]->(test)
# MERGE (self)-[:DEFINES]->(fault)
# MERGE (self)-[:DEFINES]->(stop)
# MERGE (main)-[:CALLS]->(port)
# MERGE (main)-[:CALLS]->(environment)
# MERGE (environment)-[:READS]->(deviceTestUrl)
# MERGE (main)-[:CALLS]->(step)
# MERGE (main)-[:CALLS]->(cluster)
# MERGE (main)-[:CALLS]->(migrations)
# MERGE (main)-[:CALLS]->(fixture)
# MERGE (main)-[:CALLS]->(devicePgTest)
# MERGE (main)-[:CALLS]->(snapshot)
# MERGE (main)-[:CALLS]->(assertion)
# MERGE (main)-[:CALLS]->(test)
# MERGE (main)-[:CALLS]->(fault)
# MERGE (main)-[:CALLS]->(stop)
# MERGE (main)-[:READS]->(runnerRoot)
# MERGE (stop)-[:READS]->(runnerRoot)
# MERGE (port)-[:READS]->(protectedPorts)
# MERGE (cluster)-[:CALLS]->(step)
# MERGE (psql)-[:CALLS]->(step)
# MERGE (sql)-[:CALLS]->(psql)
# MERGE (migrations)-[:CALLS]->(sql)
# MERGE (fixture)-[:CALLS]->(sql)
# MERGE (fixture)-[:CALLS]->(deviceChecks)
# MERGE (deviceChecks)-[:CALLS]->(sql)
# MERGE (devicePgTest)-[:CALLS]->(step)
# MERGE (devicePgTest)-[:CONFIGURES]->(deviceTestUrl)
# MERGE (devicePgTest)-[:TESTS]->(deviceAuthPgTests)
# MERGE (snapshot)-[:CALLS]->(sql)
# MERGE (assertion)-[:CALLS]->(snapshot)
# MERGE (test)-[:CALLS]->(step)
# MERGE (fault)-[:CALLS]->(sql)
# MERGE (stop)-[:CALLS]->(step)
# MERGE (test)-[:TESTS]->(:Endpoint {id:"PUT /api/v1/projects/{project_id}/engineering-runs/{run_id}/worktrees/{worktree_id}",kind:"http_route"})
# MERGE (test)-[:TESTS]->(:Endpoint {id:"DELETE /api/v1/projects/{project_id}/engineering-runs/{run_id}/worktrees/{worktree_id}",kind:"http_route"})
# MERGE (test)-[:TESTS]->(:Endpoint {id:"POST /api/v1/projects/{project_id}/engineering-runs",kind:"http_route"})
# MERGE (test)-[:TESTS]->(:Endpoint {id:"GET /api/v1/projects/{project_id}/engineering-runs",kind:"http_route"})
# MERGE (test)-[:TESTS]->(:Endpoint {id:"GET /api/v1/engineering-runs/{engineering_run_id}/context",kind:"http_route"})
# MERGE (snapshot)-[:READS]->(:Table {id:"multica.engineering_run_worktree_binding",kind:"table"})
# MERGE (snapshot)-[:READS]->(:Table {id:"multica.engineering_run_outbox",kind:"table"})
# MERGE (snapshot)-[:READS]->(:Table {id:"multica.engineering_directory_event",kind:"table"})
# MERGE (snapshot)-[:READS]->(:Table {id:"multica.engineering_run_command_idempotency",kind:"table"})
# MERGE (fixture)-[:READS]->(:Table {id:"permission.project_role_binding",kind:"table"})
# MERGE (fixture)-[:DEPENDS_ON]->(:File {path:"db/migrations/2026-10-23-engineering-directory-lock-only-policy.sql"})
# MERGE (fixture)-[:DEPENDS_ON]->(:File {path:"db/migrations/2026-10-23-engineering-run-create-owner-grants.sql"})
# MERGE (fixture)-[:TESTS_POLICY]->(:Policy {id:"permission.project_role_binding::engineering_directory_owner_no_project_grant_update",kind:"row_level_security"})
# MERGE (fixture)-[:READS_AND_LOCKS]->(:Table {id:"multica.worktree_project_binding",kind:"table"})
# MERGE (fixture)-[:READS_AND_LOCKS]->(:Table {id:"public.worktree_canvas_worktree",kind:"table"})
# MERGE (fixture)-[:TESTS_POLICY]->(:Policy {id:"multica.worktree_project_binding::engineering_directory_owner_no_worktree_binding_update",kind:"row_level_security"})
# MERGE (fixture)-[:TESTS_POLICY]->(:Policy {id:"public.worktree_canvas_worktree::engineering_directory_owner_no_worktree_projection_update",kind:"row_level_security"})
# MERGE (fixture)-[:READS_AND_LOCKS]->(:Table {id:"multica.engineering_run_revision",kind:"table"})
# MERGE (fixture)-[:READS_AND_LOCKS]->(:Table {id:"permission.engineering_run_role_binding",kind:"table"})
# MERGE (fixture)-[:TESTS_POLICY]->(:Policy {id:"multica.engineering_run_revision::engineering_directory_owner_no_run_revision_update",kind:"row_level_security"})
# MERGE (fixture)-[:TESTS_POLICY]->(:Policy {id:"permission.engineering_run_role_binding::engineering_directory_owner_no_run_grant_update",kind:"row_level_security"})
# MERGE (oauthBaseMigration:File {path:"db/migrations/2026-09-09-oauth2-server.sql"})
# MERGE (deviceMigration:File {path:"db/migrations/2026-10-28-w16-device-auth.sql"})
# MERGE (deviceBindings:Table {id:"public.oauth_device_bindings",kind:"table"})
# MERGE (deviceChallenges:Table {id:"public.oauth_device_enrollment_challenges",kind:"table"})
# MERGE (deviceAudit:Table {id:"public.oauth_device_binding_audit",kind:"table"})
# MERGE (refreshTokens:Table {id:"public.oauth_refresh_tokens",kind:"table"})
# MERGE (refreshDevicePair:Constraint {id:"public.oauth_refresh_tokens::oauth_refresh_tokens_device_generation_pair",kind:"check"})
# MERGE (oauthOwner:Role {name:"star_oauth_credential_owner",kind:"postgres_role"})
# MERGE (directoryOwner:Role {name:"star_engineering_directory_owner",kind:"postgres_role"})
# MERGE (deviceAuthPgTests:File {path:"crates/star-pg-adapter/tests/device_auth_pg.rs"})
# MERGE (auditEvents:Table {id:"public.audit_audit_event",kind:"table"})
# MERGE (migrations)-[:DEPENDS_ON]->(oauthBaseMigration)
# MERGE (fixture)-[:DEPENDS_ON]->(deviceMigration)
# MERGE (migrations)-[:CREATES]->(auditEvents)
# MERGE (fixture)-[:READS]->(deviceBindings)
# MERGE (fixture)-[:READS_AND_LOCKS]->(deviceBindings)
# MERGE (fixture)-[:READS]->(refreshTokens)
# MERGE (fixture)-[:WRITES]->(deviceBindings)
# MERGE (fixture)-[:WRITES]->(refreshTokens)
# MERGE (fixture)-[:TESTS_POLICY]->(:Policy {id:"public.oauth_device_bindings::engineering_directory_owner_no_oauth_device_binding_update",kind:"row_level_security"})
# MERGE (fixture)-[:TESTS]->(refreshDevicePair)
# MERGE (deviceChecks)-[:TESTS]->(deviceBindings)
# MERGE (deviceChecks)-[:TESTS]->(deviceChallenges)
# MERGE (deviceChecks)-[:TESTS]->(deviceAudit)
# MERGE (deviceChecks)-[:TESTS]->(refreshTokens)
# MERGE (deviceChecks)-[:TESTS]->(oauthOwner)
# MERGE (deviceChecks)-[:TESTS]->(directoryOwner)
# @endcypher

from __future__ import annotations

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
# Give each invocation its own root so a retry never overwrites or removes a
# previous acceptance run's evidence or PostgreSQL files.
RUNNER_ROOT = (
    ROOT / ".cache" / "wld0-binding-http-positive-20261023" / "runs" / uuid.uuid4().hex
)
PROTECTED_LOOPBACK_PORTS = frozenset({5432, 5555})
RUNTIME_ROLE = "erun_directory_api_test"
OAUTH_OWNER_ROLE = "star_oauth_credential_owner"
DIRECTORY_OWNER_ROLE = "star_engineering_directory_owner"
TENANT_A = "10000000-0000-0000-0000-000000000001"
TENANT_C = "10000000-0000-0000-0000-000000000002"
ACTOR_A = "30000000-0000-0000-0000-000000000001"
ACTOR_B = "30000000-0000-0000-0000-000000000002"
ACTOR_C = "30000000-0000-0000-0000-000000000003"
PROJECT_A = "20000000-0000-0000-0000-000000000001"
RUN_A = "60000000-0000-0000-0000-000000000001"
WORKTREE_A = "90000000-0000-0000-0000-000000000001"
PROJECT_BINDING = "50000000-0000-0000-0000-000000000001"
DEVICE_A = "a0000000-0000-0000-0000-000000000001"
DEVICE_B = "a0000000-0000-0000-0000-000000000002"
DEVICE_C = "a0000000-0000-0000-0000-000000000003"
DEVICE_D = "a0000000-0000-0000-0000-000000000004"


def sanitized_environment() -> dict[str, str]:
    """Keep unrelated user PostgreSQL settings from redirecting the fixture."""
    environment = os.environ.copy()
    for key in list(environment):
        if key.upper().startswith("PG") or key.upper() in {
            "DATABASE_URL", "STAR_DATABASE_URL", "STAR_BINDING_HTTP_TEST_DATABASE_URL",
            "STAR_BINDING_HTTP_TEST_PHASE", "STAR_W16_DEVICE_AUTH_TEST_DATABASE_URL", "SQLX_OFFLINE",
        }:
            environment.pop(key, None)
    environment["CARGO_TERM_COLOR"] = "never"
    return environment


def select_loopback_port() -> int:
    """Reserve a loopback port, excluding both system and protected listeners."""
    while True:
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as listener:
            listener.bind(("127.0.0.1", 0))
            port = int(listener.getsockname()[1])
        if port not in PROTECTED_LOOPBACK_PORTS:
            return port


def run_step(name: str, command: list[str], environment: dict[str, str],
             output_dir: Path, timeout: int = 1800, file_handles: bool = False) -> dict:
    """Run one bounded command and retain full stdout/stderr in the private ignored cache."""
    output_dir.mkdir(parents=True, exist_ok=True)
    stdout_path = output_dir / f"{name}.stdout.log"
    stderr_path = output_dir / f"{name}.stderr.log"
    displayed_command = subprocess.list2cmdline(command)
    if len(displayed_command) > 320:
        displayed_command = f"{command[0]} [full command retained in {stdout_path.parent / 'acceptance-summary.json'}]"
    print(f"COMMAND[{name}] {displayed_command}", flush=True)
    try:
        if file_handles:
            # pg_ctl may leave descendant handles open on Windows; file-backed output
            # prevents subprocess.run from waiting for the postmaster's inherited pipe.
            with stdout_path.open("w", encoding="utf-8") as stdout_file, \
                    stderr_path.open("w", encoding="utf-8") as stderr_file:
                completed = subprocess.run(
                    command, cwd=ROOT, env=environment, text=True, encoding="utf-8", errors="replace",
                    stdout=stdout_file, stderr=stderr_file, timeout=timeout, check=False,
                )
            stdout = stdout_path.read_text(encoding="utf-8")
            stderr = stderr_path.read_text(encoding="utf-8")
        else:
            completed = subprocess.run(
                command, cwd=ROOT, env=environment, text=True, encoding="utf-8", errors="replace",
                stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=timeout, check=False,
            )
            stdout = completed.stdout
            stderr = completed.stderr
            stdout_path.write_text(stdout, encoding="utf-8")
            stderr_path.write_text(stderr, encoding="utf-8")
        result = {
            "name": name, "command": command, "exit_code": completed.returncode,
            "stdout_log": str(stdout_path.relative_to(ROOT)),
            "stderr_log": str(stderr_path.relative_to(ROOT)),
            "stdout_tail": stdout[-5000:], "stderr_tail": stderr[-5000:],
        }
    except subprocess.TimeoutExpired as error:
        stdout = error.stdout.decode(errors="replace") if isinstance(error.stdout, bytes) else (error.stdout or "")
        stderr = error.stderr.decode(errors="replace") if isinstance(error.stderr, bytes) else (error.stderr or "")
        stdout_path.write_text(stdout, encoding="utf-8")
        stderr_path.write_text(stderr, encoding="utf-8")
        result = {
            "name": name, "command": command, "exit_code": 124,
            "stdout_log": str(stdout_path.relative_to(ROOT)),
            "stderr_log": str(stderr_path.relative_to(ROOT)),
            "stdout_tail": stdout[-5000:], "stderr_tail": stderr[-5000:],
            "error": f"timed out after {timeout} seconds",
        }
    print(f"EXIT[{name}] {result['exit_code']}", flush=True)
    if result["exit_code"] and result["stdout_tail"]:
        print(f"STDOUT_TAIL[{name}]\n{result['stdout_tail'][-2500:]}", flush=True)
    if result["exit_code"] and result["stderr_tail"]:
        print(f"STDERR_TAIL[{name}]\n{result['stderr_tail'][-2500:]}", flush=True)
    return result


def psql_command(bin_dir: Path, port: int, sql: str | None = None,
                 file: Path | None = None, login: str = "erun_validation") -> list[str]:
    command = [str(bin_dir / "psql.exe"), "-X", "-h", "127.0.0.1", "-p", str(port),
               "-U", login, "-d", "postgres", "-v", "ON_ERROR_STOP=1"]
    if sql is not None:
        command.extend(["-c", sql])
    elif file is not None:
        command.extend(["-f", str(file)])
    else:
        raise ValueError("psql command requires SQL or a file")
    return command


def run_sql(name: str, bin_dir: Path, port: int, environment: dict[str, str],
            output_dir: Path, sql: str | None = None, file: Path | None = None,
            login: str = "erun_validation") -> dict:
    return run_step(name, psql_command(bin_dir, port, sql, file, login), environment, output_dir, 180)


def start_cluster(bin_dir: Path, port: int, data_dir: Path, environment: dict[str, str],
                  output_dir: Path) -> tuple[list[dict], bool, bool]:
    """Initialize and start one new PostgreSQL 18 data directory bound to IPv4 loopback."""
    results = []
    initdb = run_step("postgres-init", [str(bin_dir / "initdb.exe"), "-D", str(data_dir),
                         "-U", "erun_validation", "--auth=trust", "--no-instructions"],
                      environment, output_dir, 300)
    results.append(initdb)
    if initdb["exit_code"]:
        return results, False, False
    cleanup_required = True  # A failed pg_ctl command may still have launched a postmaster.
    start = run_step("postgres-start", [str(bin_dir / "pg_ctl.exe"), "-D", str(data_dir),
                         "-l", str(data_dir / "server.log"), "-o",
                         f"-h 127.0.0.1 -p {port}", "start"],
                     environment, output_dir, 180, file_handles=True)
    results.append(start)
    if start["exit_code"]:
        return results, cleanup_required, False
    ready = wait_for_postgres(bin_dir, port, environment, output_dir)
    results.append(ready)
    return results, cleanup_required, ready["exit_code"] == 0


def wait_for_postgres(bin_dir: Path, port: int, environment: dict[str, str],
                      output_dir: Path, timeout: float = 45.0) -> dict:
    """Poll the private loopback endpoint using psql until PostgreSQL accepts connections."""
    command = psql_command(bin_dir, port, "SELECT current_setting('port');")
    started_at = time.monotonic()
    attempt = 0
    last_stdout = ""
    last_stderr = ""
    last_returncode = 1
    while time.monotonic() - started_at < timeout:
        attempt += 1
        try:
            result = subprocess.run(
                command, cwd=ROOT, env=environment, text=True, encoding="utf-8", errors="replace",
                stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=3, check=False,
            )
        except subprocess.TimeoutExpired:
            time.sleep(0.5)
            continue
        last_stdout, last_stderr = result.stdout, result.stderr
        last_returncode = result.returncode
        if result.returncode == 0:
            break
        time.sleep(0.5)
    output_dir.mkdir(parents=True, exist_ok=True)
    stdout_path = output_dir / "postgres-ready.stdout.log"
    stderr_path = output_dir / "postgres-ready.stderr.log"
    stdout_path.write_text(last_stdout, encoding="utf-8")
    stderr_path.write_text(last_stderr, encoding="utf-8")
    exit_code = 0 if last_returncode == 0 else 1
    outcome = {
        "name": "postgres-ready", "command": command, "exit_code": exit_code,
        "attempts": attempt,
        "stdout_log": str(stdout_path.relative_to(ROOT)),
        "stderr_log": str(stderr_path.relative_to(ROOT)),
        "stdout_tail": last_stdout[-2000:], "stderr_tail": last_stderr[-2000:],
    }
    print(f"POSTGRES_READY port={port} attempts={attempt} exit={exit_code}", flush=True)
    return outcome


def apply_migrations(bin_dir: Path, port: int, environment: dict[str, str],
                     output_dir: Path) -> list[dict]:
    """Apply OAuth and Run Directory migrations before owner-role setup."""
    sys.path.insert(0, str(ROOT / "scripts" / "automation"))
    import engineering_run_directory as directory

    results = [run_sql(
        "postgres-oauth-audit-prerequisite-fixture", bin_dir, port, environment, output_dir,
        sql="""-- Test-only dependencies normally supplied by the platform audit schema.
        CREATE TABLE public.audit_audit_event (
          audit_event_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
          event_type text NOT NULL,
          resource_id uuid,
          tenant_id uuid,
          payload jsonb NOT NULL,
          occurred_at timestamptz NOT NULL
        );
        CREATE FUNCTION public.audit_audit_event() RETURNS trigger LANGUAGE plpgsql SECURITY DEFINER SET search_path=pg_catalog,public AS $$
        BEGIN
          INSERT INTO public.audit_audit_event(event_type,resource_id,tenant_id,payload,occurred_at)
          VALUES (TG_OP,NEW.id,NEW.tenant_id,to_jsonb(NEW),transaction_timestamp());
          RETURN NEW;
        END $$;
        -- The OAuth migration installs this trigger but this harness does not exercise SCD2 writes.
        CREATE FUNCTION public.scd_type2_close() RETURNS trigger LANGUAGE plpgsql AS $$
        BEGIN RETURN NEW; END $$;
        -- Legacy prerequisite for the existing Run Directory migration chain.
        CREATE FUNCTION public.audit_trigger_func() RETURNS trigger LANGUAGE plpgsql AS $$
        BEGIN RETURN COALESCE(NEW, OLD); END $$;""",
    )]
    if results[-1]["exit_code"]:
        return results
    oauth_base = ROOT / "db" / "migrations" / "2026-09-09-oauth2-server.sql"
    results.append(run_sql(
        "postgres-oauth-base-migration", bin_dir, port, environment, output_dir,
        file=oauth_base,
    ))
    if results[-1]["exit_code"]:
        return results
    for name, migration in directory.directory_migration_plan(run_binding=True):
        path = ROOT / "db" / "migrations" / migration
        results.append(run_sql(name, bin_dir, port, environment, output_dir, file=path))
        if results[-1]["exit_code"]:
            return results
        results.append(run_sql(
            f"{name}-schema-fingerprint", bin_dir, port, environment, output_dir,
            sql=directory.migration_fingerprint_sql(),
        ))
        if results[-1]["exit_code"]:
            return results
    # Model an installation that already has the original weak W16 CHECK.
    # The current migration must replace it with the NULL-safe constraint.
    results.append(run_sql(
        "postgres-w16-legacy-weak-refresh-device-generation-pair",
        bin_dir, port, environment, output_dir,
        sql="""ALTER TABLE public.oauth_refresh_tokens ADD COLUMN device_id UUID;
        ALTER TABLE public.oauth_refresh_tokens ADD COLUMN device_generation BIGINT;
        ALTER TABLE public.oauth_refresh_tokens
          ADD CONSTRAINT oauth_refresh_tokens_device_generation_pair
          CHECK ((device_id IS NULL AND device_generation IS NULL)
                 OR (device_id IS NOT NULL AND device_generation > 0));""",
    ))
    if results[-1]["exit_code"]:
        return results

    return results


def device_owner_catalog_checks(bin_dir: Path, port: int, environment: dict[str, str],
                                output_dir: Path) -> list[dict]:
    """Grant only role membership, then verify explicit OAuth and Directory ACL contracts."""
    results = [run_sql(
        "postgres-runtime-oauth-owner-role-membership-grant", bin_dir, port,
        environment, output_dir,
        sql=f"GRANT {OAUTH_OWNER_ROLE} TO {RUNTIME_ROLE};",
    )]
    if results[-1]["exit_code"]:
        return results

    membership_check = f"""DO $verify$
    DECLARE runtime_is_safe boolean;
            oauth_owner_is_safe boolean;
    BEGIN
      SELECT rolcanlogin AND NOT rolsuper AND NOT rolinherit AND NOT rolbypassrls
        INTO runtime_is_safe FROM pg_roles WHERE rolname='{RUNTIME_ROLE}';
      IF runtime_is_safe IS DISTINCT FROM TRUE THEN
        RAISE EXCEPTION 'runtime login must be login, non-superuser, NOINHERIT, and NOBYPASSRLS';
      END IF;
      SELECT NOT rolcanlogin AND NOT rolsuper AND NOT rolinherit AND NOT rolbypassrls
        INTO oauth_owner_is_safe FROM pg_roles WHERE rolname='{OAUTH_OWNER_ROLE}';
      IF oauth_owner_is_safe IS DISTINCT FROM TRUE THEN
        RAISE EXCEPTION 'OAuth owner must be NOLOGIN, NOSUPERUSER, NOINHERIT, and NOBYPASSRLS';
      END IF;
      IF NOT pg_has_role('{RUNTIME_ROLE}','{OAUTH_OWNER_ROLE}','MEMBER')
         OR NOT pg_has_role('{RUNTIME_ROLE}','{OAUTH_OWNER_ROLE}','SET') THEN
        RAISE EXCEPTION 'runtime login must be able to SET ROLE to the OAuth owner';
      END IF;
      IF NOT pg_has_role('{RUNTIME_ROLE}','{DIRECTORY_OWNER_ROLE}','MEMBER')
         OR NOT pg_has_role('{RUNTIME_ROLE}','{DIRECTORY_OWNER_ROLE}','SET') THEN
        RAISE EXCEPTION 'runtime login must be able to SET ROLE to the Directory lock owner';
      END IF;
    END
    $verify$;"""
    results.append(run_sql(
        "postgres-runtime-oauth-and-directory-role-membership-check", bin_dir, port,
        environment, output_dir, sql=membership_check,
    ))
    if results[-1]["exit_code"]:
        return results

    no_direct_acl_check = f"""DO $verify$
    BEGIN
      IF EXISTS (
        WITH protected(relid) AS (VALUES
          ('public.oauth_device_enrollment_challenges'::regclass::oid),
          ('public.oauth_device_bindings'::regclass::oid),
          ('public.oauth_device_binding_audit'::regclass::oid),
          ('public.oauth_refresh_tokens'::regclass::oid)
        )
        SELECT 1 FROM protected AS p JOIN pg_class AS c ON c.oid=p.relid
        CROSS JOIN LATERAL aclexplode(CASE WHEN cardinality(c.relacl)>0 THEN c.relacl END) AS acl
        WHERE acl.grantee IN (0::oid,'{RUNTIME_ROLE}'::regrole::oid)
      ) OR EXISTS (
        WITH protected(relid) AS (VALUES
          ('public.oauth_device_enrollment_challenges'::regclass::oid),
          ('public.oauth_device_bindings'::regclass::oid),
          ('public.oauth_device_binding_audit'::regclass::oid),
          ('public.oauth_refresh_tokens'::regclass::oid)
        )
        SELECT 1 FROM protected AS p JOIN pg_attribute AS a ON a.attrelid=p.relid
        CROSS JOIN LATERAL aclexplode(CASE WHEN cardinality(a.attacl)>0 THEN a.attacl END) AS acl
        WHERE a.attnum>0 AND NOT a.attisdropped
          AND acl.grantee IN (0::oid,'{RUNTIME_ROLE}'::regrole::oid)
      ) THEN
        RAISE EXCEPTION 'runtime login or PUBLIC has a direct OAuth device/refresh-token table or column ACL';
      END IF;
    END
    $verify$;"""
    results.append(run_sql(
        "postgres-runtime-no-direct-oauth-table-or-column-acl-check", bin_dir, port,
        environment, output_dir, sql=no_direct_acl_check,
    ))
    if results[-1]["exit_code"]:
        return results

    owner_grant_check = f"""DO $verify$
    DECLARE relation_contract record;
            privilege_name text;
            column_contract record;
            actual_table_acl text[];
    BEGIN
      FOR relation_contract IN
        SELECT relid, allowed_table_privileges, allowed_update_columns
        FROM (VALUES
          ('public.oauth_device_enrollment_challenges'::regclass::oid,
           ARRAY['SELECT','INSERT','DELETE']::text[], ARRAY['consumed_at']::text[]),
          ('public.oauth_device_bindings'::regclass::oid,
           ARRAY['SELECT','INSERT']::text[], ARRAY['generation','status','revoked_at']::text[]),
          ('public.oauth_device_binding_audit'::regclass::oid,
           ARRAY['INSERT']::text[], ARRAY[]::text[]),
          ('public.oauth_refresh_tokens'::regclass::oid,
           ARRAY['SELECT']::text[], ARRAY['revoked_at']::text[])
        ) AS expected(relid, allowed_table_privileges, allowed_update_columns)
      LOOP
        IF (SELECT relowner FROM pg_class WHERE oid=relation_contract.relid)
             = '{OAUTH_OWNER_ROLE}'::regrole::oid THEN
          RAISE EXCEPTION 'OAuth owner must not own %', relation_contract.relid::regclass;
        END IF;

        FOREACH privilege_name IN ARRAY
          ARRAY['SELECT','INSERT','UPDATE','DELETE','TRUNCATE','REFERENCES','TRIGGER']::text[]
        LOOP
          IF has_table_privilege('{OAUTH_OWNER_ROLE}', relation_contract.relid, privilege_name)
             IS DISTINCT FROM (privilege_name = ANY(relation_contract.allowed_table_privileges)) THEN
            RAISE EXCEPTION 'OAuth owner table privilege % does not match allowlist for %',
              privilege_name, relation_contract.relid::regclass;
          END IF;
        END LOOP;

        SELECT coalesce(array_agg(DISTINCT acl.privilege_type ORDER BY acl.privilege_type), ARRAY[]::text[])
          INTO actual_table_acl
        FROM pg_class AS c
        CROSS JOIN LATERAL aclexplode(CASE WHEN cardinality(c.relacl)>0 THEN c.relacl END) AS acl
        WHERE c.oid=relation_contract.relid
          AND acl.grantee='{OAUTH_OWNER_ROLE}'::regrole::oid
          AND NOT acl.is_grantable;
        IF actual_table_acl IS DISTINCT FROM
           (SELECT coalesce(array_agg(allowed.privilege_name ORDER BY allowed.privilege_name), ARRAY[]::text[])
            FROM unnest(relation_contract.allowed_table_privileges) AS allowed(privilege_name)) THEN
          RAISE EXCEPTION 'OAuth owner direct table ACL does not match allowlist for %',
            relation_contract.relid::regclass;
        END IF;
        IF EXISTS (
          SELECT 1
          FROM pg_class AS c
          CROSS JOIN LATERAL aclexplode(CASE WHEN cardinality(c.relacl)>0 THEN c.relacl END) AS acl
          WHERE c.oid=relation_contract.relid
            AND acl.grantee='{OAUTH_OWNER_ROLE}'::regrole::oid
            AND acl.is_grantable
        ) THEN
          RAISE EXCEPTION 'OAuth owner has table grant options on %', relation_contract.relid::regclass;
        END IF;

        FOR column_contract IN
          SELECT a.attname
          FROM pg_attribute AS a
          WHERE a.attrelid=relation_contract.relid AND a.attnum>0 AND NOT a.attisdropped
        LOOP
          IF has_column_privilege('{OAUTH_OWNER_ROLE}', relation_contract.relid,
                                  column_contract.attname, 'UPDATE')
             IS DISTINCT FROM (column_contract.attname = ANY(relation_contract.allowed_update_columns)) THEN
            RAISE EXCEPTION 'OAuth owner UPDATE column % does not match allowlist for %',
              column_contract.attname, relation_contract.relid::regclass;
          END IF;
        END LOOP;

        IF EXISTS (
          SELECT 1
          FROM pg_attribute AS a
          CROSS JOIN LATERAL aclexplode(CASE WHEN cardinality(a.attacl)>0 THEN a.attacl END) AS acl
          WHERE a.attrelid=relation_contract.relid AND a.attnum>0 AND NOT a.attisdropped
            AND acl.grantee='{OAUTH_OWNER_ROLE}'::regrole::oid
            AND (acl.privilege_type <> 'UPDATE'
                 OR NOT (a.attname = ANY(relation_contract.allowed_update_columns))
                 OR acl.is_grantable)
        ) THEN
          RAISE EXCEPTION 'OAuth owner has a non-allowlisted or grantable column ACL on %',
            relation_contract.relid::regclass;
        END IF;
        IF EXISTS (
          SELECT 1
          FROM unnest(relation_contract.allowed_update_columns) AS expected_column(attname)
          WHERE NOT EXISTS (
            SELECT 1
            FROM pg_attribute AS a
            CROSS JOIN LATERAL aclexplode(CASE WHEN cardinality(a.attacl)>0 THEN a.attacl END) AS acl
            WHERE a.attrelid=relation_contract.relid
              AND a.attnum>0 AND NOT a.attisdropped
              AND a.attname=expected_column.attname
              AND acl.grantee='{OAUTH_OWNER_ROLE}'::regrole::oid
              AND acl.privilege_type='UPDATE'
              AND NOT acl.is_grantable
          )
        ) THEN
          RAISE EXCEPTION 'OAuth owner is missing a direct allowlisted column UPDATE ACL on %',
            relation_contract.relid::regclass;
        END IF;
      END LOOP;
    END
    $verify$;"""
    results.append(run_sql(
        "postgres-oauth-owner-table-and-column-grant-check", bin_dir, port,
        environment, output_dir, sql=owner_grant_check,
    ))
    if results[-1]["exit_code"]:
        return results

    directory_read_check = f"""DO $verify$
    DECLARE privilege_name text;
            column_contract record;
            matched_policies integer;
    BEGIN
      FOREACH privilege_name IN ARRAY
        ARRAY['SELECT','INSERT','UPDATE','DELETE','TRUNCATE','REFERENCES','TRIGGER']::text[]
      LOOP
        IF has_table_privilege('{DIRECTORY_OWNER_ROLE}',
                               'public.oauth_device_bindings'::regclass::oid,
                               privilege_name)
           IS DISTINCT FROM (privilege_name = 'SELECT') THEN
          RAISE EXCEPTION 'Directory owner table privilege % violates the table ACL allowlist',
            privilege_name;
        END IF;
      END LOOP;
      FOR column_contract IN
        SELECT a.attname
        FROM pg_attribute AS a
        WHERE a.attrelid='public.oauth_device_bindings'::regclass::oid
          AND a.attnum>0 AND NOT a.attisdropped
      LOOP
        IF has_column_privilege('{DIRECTORY_OWNER_ROLE}',
                                'public.oauth_device_bindings'::regclass::oid,
                                column_contract.attname, 'UPDATE')
           IS DISTINCT FROM (column_contract.attname = 'generation') THEN
          RAISE EXCEPTION 'Directory owner UPDATE column % violates the generation-only allowlist',
            column_contract.attname;
        END IF;
      END LOOP;
      IF (SELECT relowner FROM pg_class
          WHERE oid='public.oauth_device_bindings'::regclass::oid)
           = '{DIRECTORY_OWNER_ROLE}'::regrole::oid
         OR NOT EXISTS (
          SELECT 1
          FROM pg_class AS c
          CROSS JOIN LATERAL aclexplode(CASE WHEN cardinality(c.relacl)>0 THEN c.relacl END) AS acl
          WHERE c.oid='public.oauth_device_bindings'::regclass::oid
            AND acl.grantee='{DIRECTORY_OWNER_ROLE}'::regrole::oid
            AND acl.privilege_type='SELECT' AND NOT acl.is_grantable
         )
         OR EXISTS (
          SELECT 1
          FROM pg_class AS c
          CROSS JOIN LATERAL aclexplode(CASE WHEN cardinality(c.relacl)>0 THEN c.relacl END) AS acl
          WHERE c.oid='public.oauth_device_bindings'::regclass::oid
            AND acl.grantee='{DIRECTORY_OWNER_ROLE}'::regrole::oid
            AND (acl.privilege_type <> 'SELECT' OR acl.is_grantable)
         ) THEN
        RAISE EXCEPTION 'Directory owner table-level ACL must be SELECT only; UPDATE is allowed only on generation';
      END IF;
      IF EXISTS (
        SELECT 1
        FROM pg_attribute AS a
        CROSS JOIN LATERAL aclexplode(CASE WHEN cardinality(a.attacl)>0 THEN a.attacl END) AS acl
        WHERE a.attrelid='public.oauth_device_bindings'::regclass::oid
          AND a.attnum>0 AND NOT a.attisdropped
          AND acl.grantee='{DIRECTORY_OWNER_ROLE}'::regrole::oid
          AND (acl.privilege_type <> 'UPDATE' OR a.attname <> 'generation' OR acl.is_grantable)
      ) THEN
        RAISE EXCEPTION 'Directory owner direct column ACLs must be non-grantable UPDATE(generation) only';
      END IF;
      IF NOT EXISTS (
        SELECT 1
        FROM pg_attribute AS a
        CROSS JOIN LATERAL aclexplode(CASE WHEN cardinality(a.attacl)>0 THEN a.attacl END) AS acl
        WHERE a.attrelid='public.oauth_device_bindings'::regclass::oid
          AND a.attname='generation' AND a.attnum>0 AND NOT a.attisdropped
          AND acl.grantee='{DIRECTORY_OWNER_ROLE}'::regrole::oid
          AND acl.privilege_type='UPDATE' AND NOT acl.is_grantable
      ) THEN
        RAISE EXCEPTION 'Directory owner is missing direct UPDATE(generation) for FOR SHARE locking';
      END IF;
      SELECT count(*) INTO matched_policies
      FROM pg_policies AS p
      WHERE p.schemaname='public' AND p.tablename='oauth_device_bindings'
        AND p.policyname='engineering_directory_owner_no_oauth_device_binding_update'
        AND p.permissive='RESTRICTIVE' AND p.cmd='UPDATE'
        AND p.roles @> ARRAY['{DIRECTORY_OWNER_ROLE}']::name[]
        AND lower(coalesce(p.qual,'')) LIKE '%tenant_id%'
        AND lower(coalesce(p.qual,'')) LIKE '%app.tenant_id%'
        AND lower(coalesce(p.qual,'')) LIKE '%actor_id%'
        AND lower(coalesce(p.qual,'')) LIKE '%app.actor_id%'
        AND lower(coalesce(p.qual,'')) LIKE '% and %'
        AND lower(coalesce(p.qual,'')) NOT LIKE '% or %'
        AND p.with_check='false';
      IF matched_policies <> 1 THEN
        RAISE EXCEPTION 'expected the named Directory owner restrictive tenant+actor USING and false WITH CHECK policy; found %',
          matched_policies;
      END IF;
      IF NOT has_table_privilege('{DIRECTORY_OWNER_ROLE}',
                                 'public.oauth_device_bindings'::regclass::oid,
                                 'SELECT') THEN
        RAISE EXCEPTION 'Directory owner is missing read access for Run/Worktree lock paths';
      END IF;
    END
    $verify$;"""
    results.append(run_sql(
        "postgres-directory-owner-device-binding-read-only-check", bin_dir, port,
        environment, output_dir, sql=directory_read_check,
    ))
    if results[-1]["exit_code"]:
        return results

    rls_check = f"""DO $verify$
    DECLARE refresh_policies integer;
            refresh_select_policies integer;
    BEGIN
      IF EXISTS (
        SELECT 1 FROM pg_class AS c
        WHERE c.oid IN (
          'public.oauth_device_enrollment_challenges'::regclass,
          'public.oauth_device_bindings'::regclass,
          'public.oauth_device_binding_audit'::regclass,
          'public.oauth_refresh_tokens'::regclass
        ) AND (NOT c.relrowsecurity OR NOT c.relforcerowsecurity)
      ) THEN
        RAISE EXCEPTION 'device and refresh-token tables must ENABLE and FORCE row-level security';
      END IF;
      IF NOT EXISTS (
        SELECT 1 FROM pg_policies AS p
        WHERE p.schemaname='public' AND p.tablename='oauth_refresh_tokens'
          AND p.permissive='PERMISSIVE' AND p.cmd IN ('UPDATE','ALL')
          AND p.roles @> ARRAY['{OAUTH_OWNER_ROLE}']::name[]
      ) THEN
        RAISE EXCEPTION 'OAuth owner-targeted permissive refresh-token policy is missing';
      END IF;
      SELECT count(*) INTO refresh_policies FROM pg_policies AS p
      WHERE p.schemaname='public' AND p.tablename='oauth_refresh_tokens'
        AND p.permissive='RESTRICTIVE' AND p.cmd IN ('UPDATE','ALL')
        AND (p.roles @> ARRAY['{OAUTH_OWNER_ROLE}']::name[] OR p.roles @> ARRAY['public']::name[])
        AND lower(coalesce(p.qual,'')) LIKE '%tenant_id%'
        AND lower(coalesce(p.qual,'')) LIKE '%app.tenant_id%'
        AND lower(coalesce(p.qual,'')) LIKE '%user_id%'
        AND lower(coalesce(p.qual,'')) LIKE '%app.actor_id%'
        AND lower(coalesce(p.qual,'')) LIKE '% and %'
        AND lower(coalesce(p.qual,'')) NOT LIKE '% or %'
        AND lower(coalesce(p.with_check,'')) LIKE '%tenant_id%'
        AND lower(coalesce(p.with_check,'')) LIKE '%app.tenant_id%'
        AND lower(coalesce(p.with_check,'')) LIKE '%user_id%'
        AND lower(coalesce(p.with_check,'')) LIKE '%app.actor_id%'
        AND lower(coalesce(p.with_check,'')) LIKE '% and %'
        AND lower(coalesce(p.with_check,'')) NOT LIKE '% or %';
      IF refresh_policies <> 1 THEN
        RAISE EXCEPTION 'expected one restrictive tenant+actor refresh-token policy; found %', refresh_policies;
      END IF;
      SELECT count(*) INTO refresh_select_policies FROM pg_policies AS p
      WHERE p.schemaname='public' AND p.tablename='oauth_refresh_tokens'
        AND p.permissive='RESTRICTIVE' AND p.cmd IN ('SELECT','ALL')
        AND p.roles @> ARRAY['{OAUTH_OWNER_ROLE}']::name[]
        AND lower(coalesce(p.qual,'')) LIKE '%tenant_id%'
        AND lower(coalesce(p.qual,'')) LIKE '%app.tenant_id%'
        AND lower(coalesce(p.qual,'')) LIKE '%user_id%'
        AND lower(coalesce(p.qual,'')) LIKE '%app.actor_id%'
        AND lower(coalesce(p.qual,'')) LIKE '% and %'
        AND lower(coalesce(p.qual,'')) NOT LIKE '% or %';
      IF refresh_select_policies <> 1 THEN
        RAISE EXCEPTION 'expected one owner-targeted restrictive tenant+actor refresh-token SELECT policy; found %', refresh_select_policies;
      END IF;
    END
    $verify$;"""
    results.append(run_sql(
        "postgres-device-refresh-token-rls-policy-catalog-check", bin_dir, port,
        environment, output_dir, sql=rls_check,
    ))
    return results


def seed_fixture(bin_dir: Path, port: int, environment: dict[str, str],
                 output_dir: Path) -> list[dict]:
    """Seed scoped device identities, one hostless Run, and a real runtime login."""
    sys.path.insert(0, str(ROOT / "scripts" / "automation"))
    import engineering_run_directory as directory

    results = [run_sql(
        "postgres-owner-role-drift", bin_dir, port, environment, output_dir,
        sql=directory.run_binding_role_drift_sql(),
    )]
    if results[-1]["exit_code"]:
        return results
    results.append(run_sql(
        "postgres-owner-role-normalize", bin_dir, port, environment, output_dir,
        file=ROOT / "db" / "migrations" / "2026-10-20-engineering-directory-owner-role.sql",
    ))
    if results[-1]["exit_code"]:
        return results
    results.append(run_sql(
        "postgres-engineering-directory-lock-only-policy-migration", bin_dir, port,
        environment, output_dir,
        file=ROOT / "db" / "migrations" / "2026-10-23-engineering-directory-lock-only-policy.sql",
    ))
    if results[-1]["exit_code"]:
        return results
    results.append(run_sql(
        "postgres-engineering-run-create-owner-grants", bin_dir, port, environment,
        output_dir,
        file=ROOT / "db" / "migrations" / "2026-10-23-engineering-run-create-owner-grants.sql",
    ))
    if results[-1]["exit_code"]:
        return results
    # This legacy owner-role migration revokes all public-schema table ACLs.
    # Apply W16 afterward so its device-binding grants describe the final ACL.
    results.append(run_sql(
        "postgres-w16-device-auth-migration", bin_dir, port, environment, output_dir,
        file=ROOT / "db" / "migrations" / "2026-10-28-w16-device-auth.sql",
    ))
    if results[-1]["exit_code"]:
        return results
    policy_catalog_check = """DO $verify$
    DECLARE matched_policies integer;
    BEGIN
      WITH expected(schemaname, tablename, policyname) AS (
        VALUES
          ('permission','project_role_binding','engineering_directory_owner_no_project_grant_update'),
          ('multica','worktree_project_binding','engineering_directory_owner_no_worktree_binding_update'),
          ('public','worktree_canvas_worktree','engineering_directory_owner_no_worktree_projection_update'),
          ('multica','engineering_run_revision','engineering_directory_owner_no_run_revision_update'),
          ('permission','engineering_run_role_binding','engineering_directory_owner_no_run_grant_update')
      )
      SELECT count(*) INTO matched_policies
        FROM expected e
        JOIN pg_policies p USING (schemaname, tablename, policyname)
       WHERE p.permissive='RESTRICTIVE' AND p.cmd='UPDATE'
         AND p.qual='true' AND p.with_check='false'
         AND p.roles @> ARRAY['star_engineering_directory_owner']::name[];
      IF matched_policies <> 5 THEN
        RAISE EXCEPTION 'expected five restrictive lock-only owner policies; found %', matched_policies;
      END IF;
    END
    $verify$;"""
    results.append(run_sql(
        "postgres-engineering-directory-lock-only-policy-catalog-check", bin_dir,
        port, environment, output_dir, sql=policy_catalog_check,
    ))
    if results[-1]["exit_code"]:
        return results
    results.append(run_sql(
        "postgres-runtime-role-project-worktree-fixture", bin_dir, port, environment, output_dir,
        sql=directory.run_binding_role_setup_sql(),
    ))
    if results[-1]["exit_code"]:
        return results
    results.extend(device_owner_catalog_checks(bin_dir, port, environment, output_dir))
    if any(result["exit_code"] for result in results):
        return results
    results.append(run_sql(
        "postgres-runtime-role-grants-rls-acceptance", bin_dir, port, environment, output_dir,
        sql=directory.run_binding_role_acceptance_sql(), login=RUNTIME_ROLE,
    ))
    if results[-1]["exit_code"]:
        return results
    results.append(run_sql(
        "postgres-test-only-refresh-fixture-helper", bin_dir, port, environment, output_dir,
        sql=f"""CREATE OR REPLACE FUNCTION public.w16_test_insert_device_bound_refresh_fixture(
          p_tenant_id uuid, p_actor_id uuid, p_device_id uuid
        ) RETURNS void LANGUAGE plpgsql SECURITY DEFINER SET search_path=pg_catalog,public AS $fixture$
        BEGIN
          PERFORM set_config('app.tenant_id',p_tenant_id::text,true);
          PERFORM set_config('app.actor_id',p_actor_id::text,true);
          INSERT INTO public.oauth_refresh_tokens
            (tenant_id,token_hash,client_id,user_id,access_token_id,scope,expires_at,device_id,device_generation)
          VALUES
            (p_tenant_id,'w16-refresh-fixture-'||replace(gen_random_uuid()::text,'-',''),
             'w16-pg-fixture',p_actor_id,gen_random_uuid(),'canvas:read',
             transaction_timestamp()+interval '1 day',p_device_id,1);
        END $fixture$;
        REVOKE ALL ON FUNCTION public.w16_test_insert_device_bound_refresh_fixture(uuid,uuid,uuid) FROM PUBLIC;
        GRANT EXECUTE ON FUNCTION public.w16_test_insert_device_bound_refresh_fixture(uuid,uuid,uuid) TO {RUNTIME_ROLE};""",
    ))
    if results[-1]["exit_code"]:
        return results

    refresh_rls_hashes = [f"w16-device-rls-{uuid.uuid4().hex}" for _ in range(3)]
    refresh_rls_access_ids = [str(uuid.uuid4()) for _ in range(3)]
    seed_sql = f"""BEGIN;
    SELECT set_config('app.tenant_id','{TENANT_A}',true),
           set_config('app.actor_id','{ACTOR_A}',true),
           set_config('app.correlation_id','10000000-0000-0000-0000-000000000099',true);
    INSERT INTO public.oauth_device_bindings
      (tenant_id,actor_id,device_id,public_key_fingerprint,generation,status)
    VALUES
      ('{TENANT_A}','{ACTOR_A}','{DEVICE_A}','aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',1,'active'),
      ('{TENANT_A}','{ACTOR_B}','{DEVICE_B}','bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb',1,'active'),
      ('{TENANT_C}','{ACTOR_C}','{DEVICE_C}','cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc',1,'active'),
      ('{TENANT_C}','{ACTOR_A}','{DEVICE_D}','dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd',1,'active');
    INSERT INTO public.oauth_refresh_tokens
      (tenant_id,token_hash,client_id,user_id,access_token_id,scope,expires_at)
    VALUES
      ('{TENANT_A}','{refresh_rls_hashes[0]}','w16-device-rls-fixture','{ACTOR_A}','{refresh_rls_access_ids[0]}','canvas:read',now()+interval '1 day'),
      ('{TENANT_A}','{refresh_rls_hashes[1]}','w16-device-rls-fixture','{ACTOR_B}','{refresh_rls_access_ids[1]}','canvas:read',now()+interval '1 day'),
      ('{TENANT_C}','{refresh_rls_hashes[2]}','w16-device-rls-fixture','{ACTOR_C}','{refresh_rls_access_ids[2]}','canvas:read',now()+interval '1 day');
    -- The OAuth locator trigger scopes each inserted token; restore the directory
    -- writer context after this deliberately multi-tenant fixture batch.
    SELECT set_config('app.tenant_id','{TENANT_A}',true),
           set_config('app.actor_id','{ACTOR_A}',true),
           set_config('app.correlation_id','10000000-0000-0000-0000-000000000099',true);
    INSERT INTO multica.worktree_project_binding
      (binding_id,tenant_id,project_id,worktree_id,assigned_by,valid_from,version)
    VALUES ('{PROJECT_BINDING}','{TENANT_A}','{PROJECT_A}','{WORKTREE_A}','{ACTOR_A}',now()-interval '1 second',1);
    INSERT INTO multica.engineering_run
      (engineering_run_id,tenant_id,project_id,repository_id,branch_id)
    VALUES ('{RUN_A}','{TENANT_A}','{PROJECT_A}',NULL,NULL);
    INSERT INTO multica.engineering_run_revision
      (tenant_id,engineering_run_id,title,state,owner_user_id,version)
    VALUES ('{TENANT_A}','{RUN_A}','WLD0 hostless binding acceptance','active','{ACTOR_A}',1);
    INSERT INTO permission.engineering_run_role_binding
      (tenant_id,engineering_run_id,user_id,role,granted_by,version)
    VALUES ('{TENANT_A}','{RUN_A}','{ACTOR_A}','project_admin','{ACTOR_A}',1);
    INSERT INTO permission.project_role_binding
      (id,tenant_id,project_id,user_id,role,granted_by,valid_from,version)
    VALUES ('40000000-0000-0000-0000-000000000002','{TENANT_A}','{PROJECT_A}',
            '30000000-0000-0000-0000-000000000002','developer','{ACTOR_A}',now()-interval '1 second',1);
    COMMIT;"""
    results.append(run_sql("postgres-hostless-http-fixture", bin_dir, port,
                           environment, output_dir, sql=seed_sql))
    if results[-1]["exit_code"]:
        return results
    directory_device_lock_probe = f"""BEGIN;
    SET LOCAL ROLE {DIRECTORY_OWNER_ROLE};
    SELECT set_config('app.tenant_id','{TENANT_A}',true),
           set_config('app.actor_id','{ACTOR_A}',true);
    DO $verify$
    DECLARE locked_generation bigint;
            out_of_scope_generation bigint;
            cross_tenant_generation bigint;
            update_sqlstate text := NULL;
            observed_generation bigint;
    BEGIN
      SELECT generation INTO locked_generation
      FROM public.oauth_device_bindings
      WHERE tenant_id='{TENANT_A}' AND device_id='{DEVICE_A}'
      FOR SHARE;
      IF locked_generation IS DISTINCT FROM 1 THEN
        RAISE EXCEPTION 'Directory owner FOR SHARE must lock its scoped device generation; found %',
          locked_generation;
      END IF;
      SELECT generation INTO out_of_scope_generation
      FROM public.oauth_device_bindings
      WHERE device_id='{DEVICE_B}'
      FOR SHARE;
      IF out_of_scope_generation IS NOT NULL THEN
        RAISE EXCEPTION 'Directory owner FOR SHARE exposed another actor device generation %',
          out_of_scope_generation;
      END IF;
      SELECT generation INTO cross_tenant_generation
      FROM public.oauth_device_bindings
      WHERE device_id='{DEVICE_D}'
      FOR SHARE;
      IF cross_tenant_generation IS NOT NULL THEN
        RAISE EXCEPTION 'Directory owner FOR SHARE exposed the same actor device from another tenant: %',
          cross_tenant_generation;
      END IF;
      BEGIN
        UPDATE public.oauth_device_bindings
           SET generation=generation+1
         WHERE tenant_id='{TENANT_A}' AND device_id='{DEVICE_A}';
      EXCEPTION WHEN SQLSTATE '42501' THEN
        GET STACKED DIAGNOSTICS update_sqlstate = RETURNED_SQLSTATE;
        RAISE NOTICE 'Directory owner UPDATE rejected with SQLSTATE %', update_sqlstate;
      END;
      IF update_sqlstate IS DISTINCT FROM '42501' THEN
        RAISE EXCEPTION 'Directory owner direct UPDATE must fail closed with SQLSTATE 42501; got %',
          update_sqlstate;
      END IF;
      SELECT generation INTO observed_generation
      FROM public.oauth_device_bindings
      WHERE tenant_id='{TENANT_A}' AND device_id='{DEVICE_A}';
      IF observed_generation IS DISTINCT FROM locked_generation THEN
        RAISE EXCEPTION 'denied Directory owner UPDATE changed generation from % to %',
          locked_generation, observed_generation;
      END IF;
    END
    $verify$;
    ROLLBACK;"""
    results.append(run_sql(
        "postgres-directory-owner-device-generation-for-share-and-denied-update-check",
        bin_dir, port, environment, output_dir,
        sql=directory_device_lock_probe, login=RUNTIME_ROLE,
    ))
    if results[-1]["exit_code"]:
        return results
    refresh_pair_constraint_catalog_check = """DO $verify$
    BEGIN
      IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint AS c
        WHERE c.conrelid='public.oauth_refresh_tokens'::regclass
          AND c.conname='oauth_refresh_tokens_device_generation_pair'
          AND c.convalidated
          AND pg_get_constraintdef(c.oid) ~* 'device_generation\\s+IS\\s+NOT\\s+NULL'
      ) THEN
        RAISE EXCEPTION 'refresh-token device pair constraint must explicitly require non-null generation';
      END IF;
    END
    $verify$;"""
    results.append(run_sql(
        "postgres-refresh-token-device-generation-pair-constraint-catalog-check",
        bin_dir, port, environment, output_dir,
        sql=refresh_pair_constraint_catalog_check,
    ))
    if results[-1]["exit_code"]:
        return results
    refresh_pair_constraint_behavior_check = f"""BEGIN;
    DO $verify$
    BEGIN
      BEGIN
        INSERT INTO public.oauth_refresh_tokens
          (tenant_id,token_hash,client_id,user_id,access_token_id,scope,expires_at,
           device_id,device_generation)
        VALUES
          ('{TENANT_A}','w16-invalid-device-generation-{uuid.uuid4().hex}',
           'w16-device-pair-constraint-check','{ACTOR_A}',
           '50000000-0000-0000-0000-000000000099','canvas:read',now()+interval '1 day',
           '{DEVICE_A}',NULL);
        RAISE EXCEPTION 'device_id without device_generation unexpectedly passed the pair constraint';
      EXCEPTION WHEN check_violation THEN
        RAISE NOTICE 'device_id with NULL device_generation rejected by check constraint';
      END;
    END
    $verify$;
    ROLLBACK;"""
    results.append(run_sql(
        "postgres-refresh-token-invalid-device-generation-pair-admin-rollback-check",
        bin_dir, port, environment, output_dir,
        sql=refresh_pair_constraint_behavior_check, login="erun_validation",
    ))
    if results[-1]["exit_code"]:
        return results
    refresh_rls_probe = f"""BEGIN;
    SET LOCAL ROLE {OAUTH_OWNER_ROLE};
    SELECT set_config('app.tenant_id','{TENANT_A}',true),
           set_config('app.actor_id','{ACTOR_A}',true);
    DO $verify$
    DECLARE visible_rows integer;
            updated_rows integer;
    BEGIN
      SELECT count(*) INTO visible_rows FROM public.oauth_refresh_tokens
       WHERE token_hash IN ('{refresh_rls_hashes[0]}','{refresh_rls_hashes[1]}','{refresh_rls_hashes[2]}');
      IF visible_rows <> 1 THEN
        RAISE EXCEPTION 'refresh-token tenant+actor scope expected 1 visible row; found %', visible_rows;
      END IF;
      UPDATE public.oauth_refresh_tokens SET revoked_at=clock_timestamp()
       WHERE token_hash IN ('{refresh_rls_hashes[0]}','{refresh_rls_hashes[1]}','{refresh_rls_hashes[2]}');
      GET DIAGNOSTICS updated_rows = ROW_COUNT;
      IF updated_rows <> 1 THEN
        RAISE EXCEPTION 'refresh-token tenant+actor update expected 1 row; updated %', updated_rows;
      END IF;
    END
    $verify$;
    ROLLBACK;"""
    results.append(run_sql(
        "postgres-device-refresh-token-tenant-actor-rls-behavior-check", bin_dir,
        port, environment, output_dir, sql=refresh_rls_probe, login=RUNTIME_ROLE,
    ))
    if results[-1]["exit_code"]:
        return results
    grant_lock_probe = f"""BEGIN;
    SET LOCAL ROLE star_engineering_directory_owner;
    SELECT set_config('app.tenant_id','{TENANT_A}',true);
    SELECT 'plain-select' AS probe,id,role FROM permission.project_role_binding
      WHERE tenant_id='{TENANT_A}' AND project_id='{PROJECT_A}' AND user_id='{ACTOR_A}'
        AND valid_from<=now() AND valid_to IS NULL;
    SELECT 'select-for-share' AS probe,id,role FROM permission.project_role_binding
      WHERE tenant_id='{TENANT_A}' AND project_id='{PROJECT_A}' AND user_id='{ACTOR_A}'
        AND valid_from<=now() AND valid_to IS NULL FOR SHARE;
    ROLLBACK;"""
    results.append(run_sql(
        "postgres-project-grant-for-share-probe", bin_dir, port, environment, output_dir,
        sql=grant_lock_probe, login=RUNTIME_ROLE,
    ))
    if results[-1]["exit_code"]:
        return results
    project_grant_lock_policy_probe = f"""BEGIN;
    SET LOCAL ROLE star_engineering_directory_owner;
    SELECT set_config('app.tenant_id','{TENANT_A}',true),
           set_config('app.actor_id','{ACTOR_A}',true),
           set_config('app.correlation_id','10000000-0000-0000-0000-000000000091',true);
    DO $verify$
    DECLARE grant_id uuid; affected integer := 0; active_grants integer := 0;
            update_sqlstate text := NULL;
    BEGIN
      IF NOT has_column_privilege(current_user,'permission.project_role_binding','valid_to','UPDATE') THEN
        RAISE EXCEPTION 'narrow Project grant UPDATE(valid_to) grant is missing';
      END IF;
      SELECT id INTO grant_id FROM permission.project_role_binding
       WHERE tenant_id='{TENANT_A}' AND project_id='{PROJECT_A}' AND user_id='{ACTOR_A}'
         AND valid_from<=now() AND valid_to IS NULL FOR SHARE;
      IF grant_id IS DISTINCT FROM '40000000-0000-0000-0000-000000000001'::uuid THEN
        RAISE EXCEPTION 'migrated lock-only policy did not expose the exact Project grant to FOR SHARE';
      END IF;
      RAISE NOTICE 'FOR SHARE accepted exact Project grant %', grant_id;
      BEGIN
        UPDATE permission.project_role_binding SET valid_to=now()
         WHERE id=grant_id AND valid_to IS NULL;
        GET DIAGNOSTICS affected = ROW_COUNT;
      EXCEPTION WHEN SQLSTATE '42501' THEN
        GET STACKED DIAGNOSTICS update_sqlstate = RETURNED_SQLSTATE;
        RAISE NOTICE 'UPDATE rejected with SQLSTATE %', update_sqlstate;
      END;
      IF update_sqlstate IS DISTINCT FROM '42501' THEN
        RAISE EXCEPTION 'expected UPDATE rejection SQLSTATE 42501 on the locked Project grant; got %, rows=%', update_sqlstate, affected;
      END IF;
      SELECT count(*) INTO active_grants FROM permission.project_role_binding
       WHERE id='40000000-0000-0000-0000-000000000001' AND valid_to IS NULL;
      IF active_grants <> 1 THEN
        RAISE EXCEPTION 'migrated lock-only policy changed or hid the source Project grant; active rows=%', active_grants;
      END IF;
      RAISE NOTICE 'source Project grant remains active';
    END
    $verify$;
    ROLLBACK;"""
    results.append(run_sql(
        "postgres-project-grant-lock-only-policy-proof", bin_dir, port, environment,
        output_dir, sql=project_grant_lock_policy_probe, login=RUNTIME_ROLE,
    ))
    if results[-1]["exit_code"]:
        return results
    worktree_project_binding_policy_probe = f"""BEGIN;
    SET LOCAL ROLE star_engineering_directory_owner;
    SELECT set_config('app.tenant_id','{TENANT_A}',true),
           set_config('app.actor_id','{ACTOR_A}',true),
           set_config('app.correlation_id','10000000-0000-0000-0000-000000000092',true);
    DO $verify$
    DECLARE locked_binding_id uuid; affected integer := 0; active_bindings integer := 0;
            update_sqlstate text := NULL;
    BEGIN
      IF NOT has_column_privilege(current_user,'multica.worktree_project_binding','valid_to','UPDATE') THEN
        RAISE EXCEPTION 'narrow Project-Worktree binding UPDATE(valid_to) grant is missing';
      END IF;
      SELECT p.binding_id INTO locked_binding_id FROM multica.worktree_project_binding p
       WHERE p.tenant_id='{TENANT_A}' AND p.project_id='{PROJECT_A}' AND p.worktree_id='{WORKTREE_A}'
         AND p.valid_from<=now() AND p.valid_to IS NULL FOR SHARE;
      IF locked_binding_id IS DISTINCT FROM '{PROJECT_BINDING}'::uuid THEN
        RAISE EXCEPTION 'migrated lock-only policy did not expose exact Project-Worktree binding to FOR SHARE';
      END IF;
      RAISE NOTICE 'FOR SHARE accepted exact Project-Worktree binding %', locked_binding_id;
      BEGIN
        UPDATE multica.worktree_project_binding AS target SET valid_to=now()
         WHERE target.binding_id=locked_binding_id AND target.valid_to IS NULL;
        GET DIAGNOSTICS affected = ROW_COUNT;
      EXCEPTION WHEN SQLSTATE '42501' THEN
        GET STACKED DIAGNOSTICS update_sqlstate = RETURNED_SQLSTATE;
        RAISE NOTICE 'UPDATE rejected with SQLSTATE %', update_sqlstate;
      END;
      IF update_sqlstate IS DISTINCT FROM '42501' THEN
        RAISE EXCEPTION 'expected UPDATE rejection SQLSTATE 42501 on the locked Project-Worktree binding; got %, rows=%', update_sqlstate, affected;
      END IF;
      SELECT count(*) INTO active_bindings FROM multica.worktree_project_binding
       WHERE multica.worktree_project_binding.binding_id='{PROJECT_BINDING}'
         AND multica.worktree_project_binding.valid_to IS NULL;
      IF active_bindings <> 1 THEN
        RAISE EXCEPTION 'migrated lock-only policy changed or hid Project-Worktree fact; active rows=%', active_bindings;
      END IF;
      RAISE NOTICE 'source Project-Worktree binding remains active';
    END
    $verify$;
    ROLLBACK;"""
    results.append(run_sql(
        "postgres-worktree-project-binding-lock-only-policy-proof", bin_dir, port,
        environment, output_dir, sql=worktree_project_binding_policy_probe, login=RUNTIME_ROLE,
    ))
    if results[-1]["exit_code"]:
        return results
    worktree_projection_policy_probe = f"""BEGIN;
    SET LOCAL ROLE star_engineering_directory_owner;
    SELECT set_config('app.tenant_id','{TENANT_A}',true),
           set_config('app.actor_id','{ACTOR_A}',true),
           set_config('app.correlation_id','10000000-0000-0000-0000-000000000093',true);
    DO $verify$
    DECLARE worktree_id uuid; affected integer := 0; remaining_worktrees integer := 0;
            update_sqlstate text := NULL;
    BEGIN
      IF NOT has_column_privilege(current_user,'public.worktree_canvas_worktree','id','UPDATE') THEN
        RAISE EXCEPTION 'narrow Worktree projection UPDATE(id) grant is missing';
      END IF;
      SELECT w.id INTO worktree_id FROM public.worktree_canvas_worktree w
       WHERE w.tenant_id='{TENANT_A}' AND w.project_id='{PROJECT_A}' AND w.id='{WORKTREE_A}'
       FOR SHARE;
      IF worktree_id IS DISTINCT FROM '{WORKTREE_A}'::uuid THEN
        RAISE EXCEPTION 'migrated lock-only policy did not expose exact Worktree projection to FOR SHARE';
      END IF;
      RAISE NOTICE 'FOR SHARE accepted exact Worktree projection %', worktree_id;
      BEGIN
        UPDATE public.worktree_canvas_worktree SET id=id
         WHERE id=worktree_id;
        GET DIAGNOSTICS affected = ROW_COUNT;
      EXCEPTION WHEN SQLSTATE '42501' THEN
        GET STACKED DIAGNOSTICS update_sqlstate = RETURNED_SQLSTATE;
        RAISE NOTICE 'UPDATE rejected with SQLSTATE %', update_sqlstate;
      END;
      IF update_sqlstate IS DISTINCT FROM '42501' THEN
        RAISE EXCEPTION 'expected UPDATE rejection SQLSTATE 42501 on the locked Worktree projection; got %, rows=%', update_sqlstate, affected;
      END IF;
      SELECT count(*) INTO remaining_worktrees FROM public.worktree_canvas_worktree
       WHERE tenant_id='{TENANT_A}' AND project_id='{PROJECT_A}' AND id='{WORKTREE_A}';
      IF remaining_worktrees <> 1 THEN
        RAISE EXCEPTION 'migrated lock-only policy changed or hid Worktree projection; rows=%', remaining_worktrees;
      END IF;
      RAISE NOTICE 'source Worktree projection remains unchanged';
    END
    $verify$;
    ROLLBACK;"""
    results.append(run_sql(
        "postgres-worktree-projection-lock-only-policy-proof", bin_dir, port,
        environment, output_dir, sql=worktree_projection_policy_probe, login=RUNTIME_ROLE,
    ))
    if results[-1]["exit_code"]:
        return results
    project_worktree_lock_probe = f"""BEGIN;
    SET LOCAL ROLE star_engineering_directory_owner;
    SELECT set_config('app.tenant_id','{TENANT_A}',true),
           set_config('app.actor_id','{ACTOR_A}',true);
    SELECT 'project-binding-plain' AS probe,p.binding_id,p.version
      FROM multica.worktree_project_binding p
      WHERE p.tenant_id='{TENANT_A}' AND p.project_id='{PROJECT_A}'
        AND p.worktree_id='{WORKTREE_A}' AND p.valid_from<=now() AND p.valid_to IS NULL;
    SELECT 'project-binding-for-share' AS probe,p.binding_id,p.version
      FROM multica.worktree_project_binding p
      WHERE p.tenant_id='{TENANT_A}' AND p.project_id='{PROJECT_A}'
        AND p.worktree_id='{WORKTREE_A}' AND p.valid_from<=now() AND p.valid_to IS NULL
      FOR SHARE;
    SELECT 'worktree-plain' AS probe,w.id,w.project_id,w.repo_id
      FROM public.worktree_canvas_worktree w
      WHERE w.tenant_id='{TENANT_A}' AND w.project_id='{PROJECT_A}' AND w.id='{WORKTREE_A}';
    SELECT 'worktree-for-share' AS probe,w.id,w.project_id,w.repo_id
      FROM public.worktree_canvas_worktree w
      WHERE w.tenant_id='{TENANT_A}' AND w.project_id='{PROJECT_A}' AND w.id='{WORKTREE_A}'
      FOR SHARE;
    SELECT 'exact-load-project-worktree-for-share' AS probe,
           p.binding_id,p.version,w.repo_id,w.branch,w.archived
      FROM multica.worktree_project_binding p
      JOIN public.worktree_canvas_worktree w ON w.tenant_id=p.tenant_id AND w.id=p.worktree_id
      WHERE p.tenant_id='{TENANT_A}' AND p.project_id='{PROJECT_A}' AND p.worktree_id='{WORKTREE_A}'
        AND p.valid_from<=now() AND p.valid_to IS NULL AND w.project_id=p.project_id
      FOR SHARE OF p,w;
    ROLLBACK;"""
    results.append(run_sql(
        "postgres-project-worktree-lock-probe", bin_dir, port, environment, output_dir,
        sql=project_worktree_lock_probe, login=RUNTIME_ROLE,
    ))
    if results[-1]["exit_code"]:
        return results
    run_authority_lock_probe = f"""BEGIN;
    SET LOCAL ROLE star_engineering_directory_owner;
    SELECT set_config('app.tenant_id','{TENANT_A}',true),
           set_config('app.actor_id','{ACTOR_A}',true);
    SELECT 'run-revision-plain' AS probe,v.tenant_id,v.engineering_run_id,v.state
      FROM multica.engineering_run r
      JOIN multica.engineering_run_revision v
        ON v.tenant_id=r.tenant_id AND v.engineering_run_id=r.engineering_run_id
      WHERE r.tenant_id='{TENANT_A}' AND r.engineering_run_id='{RUN_A}'
        AND v.valid_from<=now() AND v.valid_to IS NULL AND v.state<>'archived';
    SELECT 'run-revision-for-share' AS probe,v.tenant_id,v.engineering_run_id,v.state
      FROM multica.engineering_run r
      JOIN multica.engineering_run_revision v
        ON v.tenant_id=r.tenant_id AND v.engineering_run_id=r.engineering_run_id
      WHERE r.tenant_id='{TENANT_A}' AND r.engineering_run_id='{RUN_A}'
        AND v.valid_from<=now() AND v.valid_to IS NULL AND v.state<>'archived'
      FOR SHARE OF v;
    SELECT 'run-grant-plain' AS probe,engineering_run_id,role
      FROM permission.engineering_run_role_binding
      WHERE tenant_id='{TENANT_A}' AND engineering_run_id='{RUN_A}'
        AND user_id='{ACTOR_A}' AND valid_from<=now() AND valid_to IS NULL;
    SELECT 'run-grant-for-share' AS probe,engineering_run_id,role
      FROM permission.engineering_run_role_binding
      WHERE tenant_id='{TENANT_A}' AND engineering_run_id='{RUN_A}'
        AND user_id='{ACTOR_A}' AND valid_from<=now() AND valid_to IS NULL
      FOR SHARE;
    ROLLBACK;"""
    results.append(run_sql(
        "postgres-run-authority-lock-probe", bin_dir, port, environment, output_dir,
        sql=run_authority_lock_probe, login=RUNTIME_ROLE,
    ))
    if results[-1]["exit_code"]:
        return results
    run_revision_policy_probe = f"""BEGIN;
    SET LOCAL ROLE star_engineering_directory_owner;
    SELECT set_config('app.tenant_id','{TENANT_A}',true),
           set_config('app.actor_id','{ACTOR_A}',true),
           set_config('app.correlation_id','10000000-0000-0000-0000-000000000098',true);
    DO $verify$
    DECLARE visible_run uuid; affected integer := 0; active_revisions integer := 0;
            update_sqlstate text := NULL;
    BEGIN
      IF NOT has_column_privilege(current_user,'multica.engineering_run_revision','valid_to','UPDATE') THEN
        RAISE EXCEPTION 'narrow Engineering Run revision UPDATE(valid_to) grant is missing';
      END IF;
      SELECT v.engineering_run_id INTO visible_run
        FROM multica.engineering_run r
        JOIN multica.engineering_run_revision v
          ON v.tenant_id=r.tenant_id AND v.engineering_run_id=r.engineering_run_id
       WHERE r.tenant_id='{TENANT_A}' AND r.engineering_run_id='{RUN_A}'
         AND v.valid_from<=now() AND v.valid_to IS NULL AND v.state<>'archived'
       FOR SHARE OF v;
      IF visible_run IS DISTINCT FROM '{RUN_A}'::uuid THEN
        RAISE EXCEPTION 'migrated lock-only policy did not expose exact active Run revision to FOR SHARE';
      END IF;
      RAISE NOTICE 'FOR SHARE accepted exact active Run revision %', visible_run;
      BEGIN
        UPDATE multica.engineering_run_revision SET valid_to=now()
         WHERE tenant_id='{TENANT_A}' AND engineering_run_id=visible_run
           AND valid_from<=now() AND valid_to IS NULL AND state<>'archived';
        GET DIAGNOSTICS affected = ROW_COUNT;
      EXCEPTION WHEN SQLSTATE '42501' THEN
        GET STACKED DIAGNOSTICS update_sqlstate = RETURNED_SQLSTATE;
        RAISE NOTICE 'UPDATE rejected with SQLSTATE %', update_sqlstate;
      END;
      IF update_sqlstate IS DISTINCT FROM '42501' THEN
        RAISE EXCEPTION 'expected UPDATE rejection SQLSTATE 42501 on the locked Run revision; got %, rows=%', update_sqlstate, affected;
      END IF;
      SELECT count(*) INTO active_revisions FROM multica.engineering_run_revision
       WHERE tenant_id='{TENANT_A}' AND engineering_run_id='{RUN_A}' AND valid_to IS NULL;
      IF active_revisions <> 1 THEN
        RAISE EXCEPTION 'migrated lock-only policy changed or hid Run revision; active rows=%', active_revisions;
      END IF;
      RAISE NOTICE 'source Run revision remains active';
    END
    $verify$;
    ROLLBACK;"""
    results.append(run_sql(
        "postgres-run-revision-lock-only-policy-proof", bin_dir, port,
        environment, output_dir, sql=run_revision_policy_probe, login=RUNTIME_ROLE,
    ))
    if results[-1]["exit_code"]:
        return results
    run_grant_policy_probe = f"""BEGIN;
    SET LOCAL ROLE star_engineering_directory_owner;
    SELECT set_config('app.tenant_id','{TENANT_A}',true),
           set_config('app.actor_id','{ACTOR_A}',true),
           set_config('app.correlation_id','10000000-0000-0000-0000-000000000097',true);
    DO $verify$
    DECLARE grant_id uuid; affected integer := 0; active_grants integer := 0;
            update_sqlstate text := NULL;
    BEGIN
      IF NOT has_column_privilege(current_user,'permission.engineering_run_role_binding','valid_to','UPDATE') THEN
        RAISE EXCEPTION 'narrow Engineering Run grant UPDATE(valid_to) grant is missing';
      END IF;
      SELECT binding_id INTO grant_id FROM permission.engineering_run_role_binding
       WHERE tenant_id='{TENANT_A}' AND engineering_run_id='{RUN_A}'
         AND user_id='{ACTOR_A}' AND valid_from<=now() AND valid_to IS NULL
       FOR SHARE;
      IF grant_id IS NULL THEN
        RAISE EXCEPTION 'migrated lock-only policy did not expose exact active Run grant to FOR SHARE';
      END IF;
      RAISE NOTICE 'FOR SHARE accepted active Run grant %', grant_id;
      BEGIN
        UPDATE permission.engineering_run_role_binding SET valid_to=now()
         WHERE binding_id=grant_id AND valid_to IS NULL;
        GET DIAGNOSTICS affected = ROW_COUNT;
      EXCEPTION WHEN SQLSTATE '42501' THEN
        GET STACKED DIAGNOSTICS update_sqlstate = RETURNED_SQLSTATE;
        RAISE NOTICE 'UPDATE rejected with SQLSTATE %', update_sqlstate;
      END;
      IF update_sqlstate IS DISTINCT FROM '42501' THEN
        RAISE EXCEPTION 'expected UPDATE rejection SQLSTATE 42501 on the locked Run grant; got %, rows=%', update_sqlstate, affected;
      END IF;
      SELECT count(*) INTO active_grants FROM permission.engineering_run_role_binding
       WHERE tenant_id='{TENANT_A}' AND engineering_run_id='{RUN_A}'
         AND user_id='{ACTOR_A}' AND valid_to IS NULL;
      IF active_grants <> 1 THEN
        RAISE EXCEPTION 'migrated lock-only policy changed or hid Run grant; active rows=%', active_grants;
      END IF;
      RAISE NOTICE 'source Run grant remains active';
    END
    $verify$;
    ROLLBACK;"""
    results.append(run_sql(
        "postgres-run-grant-lock-only-policy-proof", bin_dir, port,
        environment, output_dir, sql=run_grant_policy_probe, login=RUNTIME_ROLE,
    ))
    return results


def read_snapshot(bin_dir: Path, port: int, environment: dict[str, str],
                  output_dir: Path, index: int) -> tuple[dict | None, dict]:
    """Read only the fixture Run's durable binding, audit, Outbox, and idempotency facts."""
    sql = f"""SELECT json_build_object(
      'binding_total',(SELECT count(*) FROM multica.engineering_run_worktree_binding WHERE tenant_id='{TENANT_A}' AND worktree_id='{WORKTREE_A}'),
      'binding_active',(SELECT count(*) FROM multica.engineering_run_worktree_binding WHERE tenant_id='{TENANT_A}' AND worktree_id='{WORKTREE_A}' AND valid_to IS NULL),
      'binding_closed',(SELECT count(*) FROM multica.engineering_run_worktree_binding WHERE tenant_id='{TENANT_A}' AND worktree_id='{WORKTREE_A}' AND valid_to IS NOT NULL),
      'outbox_bound',(SELECT count(*) FROM multica.engineering_run_outbox WHERE tenant_id='{TENANT_A}' AND project_id='{PROJECT_A}' AND engineering_run_id='{RUN_A}' AND event_type='engineering_run.worktree_bound'),
      'outbox_detached',(SELECT count(*) FROM multica.engineering_run_outbox WHERE tenant_id='{TENANT_A}' AND project_id='{PROJECT_A}' AND engineering_run_id='{RUN_A}' AND event_type='engineering_run.worktree_detached'),
      'audit_created',(SELECT count(*) FROM multica.engineering_directory_event WHERE tenant_id='{TENANT_A}' AND engineering_run_id='{RUN_A}' AND source_table='multica.engineering_run_worktree_binding' AND action='created'),
      'audit_closed',(SELECT count(*) FROM multica.engineering_directory_event WHERE tenant_id='{TENANT_A}' AND engineering_run_id='{RUN_A}' AND source_table='multica.engineering_run_worktree_binding' AND action='closed'),
      'idempotency',(SELECT count(*) FROM multica.engineering_run_command_idempotency WHERE tenant_id='{TENANT_A}' AND actor_id='{ACTOR_A}')
    )::text;"""
    result = run_sql(f"snapshot-{index}", bin_dir, port, environment, output_dir, sql=sql)
    if result["exit_code"]:
        return None, result
    try:
        lines = [line.strip() for line in result["stdout_tail"].splitlines() if line.strip()]
        text = next(line for line in reversed(lines) if line.startswith("{"))
        return json.loads(text), result
    except (StopIteration, json.JSONDecodeError) as error:
        result["parse_error"] = str(error)
        return None, result


def assert_snapshot_delta(before: dict, after: dict, expected: dict, label: str) -> None:
    for key, delta in expected.items():
        actual = int(after[key]) - int(before[key])
        if actual != delta:
            raise AssertionError(f"{label} {key} delta: expected {delta}, observed {actual}; snapshot={after}")


def run_rust_phase(cargo: Path, phase: str, port: int, environment: dict[str, str],
                   output_dir: Path) -> dict:
    """Run one exact ignored route test phase using the repository Rust baseline."""
    phase_environment = environment.copy()
    phase_environment["STAR_BINDING_HTTP_TEST_DATABASE_URL"] = (
        f"postgres://{RUNTIME_ROLE}@127.0.0.1:{port}/postgres"
    )
    phase_environment["STAR_BINDING_HTTP_TEST_PHASE"] = phase
    command = ["rtk", "proxy", str(cargo), "+1.99.1", "test", "-j", "4", "--locked",
               "-p", "star-api-rest", "--test", "engineering_run_worktree_binding_routes",
               "real_postgres_authenticated_binding_http_acceptance", "--",
               "--ignored", "--exact", "--nocapture"]
    return run_step(f"rust-http-{phase}", command, phase_environment, output_dir, 3600)


def run_device_auth_pg_tests(cargo: Path, port: int, environment: dict[str, str],
                             output_dir: Path) -> dict:
    """Run the ignored W16 adapter PostgreSQL tests against this disposable cluster."""
    test_environment = environment.copy()
    test_environment["STAR_W16_DEVICE_AUTH_TEST_DATABASE_URL"] = (
        f"postgres://{RUNTIME_ROLE}@127.0.0.1:{port}/postgres"
    )
    command = ["rtk", "proxy", str(cargo), "+1.99.1", "test", "-j", "4", "--locked",
               "-p", "star-pg-adapter", "--test", "device_auth_pg", "--",
               "--ignored", "--test-threads=1", "--nocapture"]
    return run_step("rust-device-auth-pg", command, test_environment, output_dir, 3600)


def set_detach_failure(bin_dir: Path, port: int, environment: dict[str, str],
                       output_dir: Path, enabled: bool) -> dict:
    if enabled:
        sql = """CREATE OR REPLACE FUNCTION public.wld0_fail_detach_outbox()
                 RETURNS trigger LANGUAGE plpgsql AS $$
                 BEGIN
                   IF NEW.event_type='engineering_run.worktree_detached' THEN
                     RAISE EXCEPTION 'wld0 injected Outbox failure';
                   END IF;
                   RETURN NEW;
                 END $$;
                 CREATE TRIGGER wld0_fail_detach_outbox BEFORE INSERT
                   ON multica.engineering_run_outbox FOR EACH ROW
                   EXECUTE FUNCTION public.wld0_fail_detach_outbox();"""
    else:
        sql = "DROP TRIGGER IF EXISTS wld0_fail_detach_outbox ON multica.engineering_run_outbox; DROP FUNCTION IF EXISTS public.wld0_fail_detach_outbox();"
    return run_sql("outbox-failure-trigger-on" if enabled else "outbox-failure-trigger-off",
                   bin_dir, port, environment, output_dir, sql=sql)


def stop_and_remove_cluster(bin_dir: Path, data_dir: Path, environment: dict[str, str],
                            output_dir: Path, cleanup_required: bool,
                            preserve_data: bool = False) -> dict:
    """Stop the owned cluster and remove its data only after successful acceptance."""
    sandbox_root = (RUNNER_ROOT / "postgres").resolve()
    resolved_data = data_dir.resolve()
    if not resolved_data.is_relative_to(sandbox_root) or resolved_data == sandbox_root:
        raise ValueError("refusing to remove a PostgreSQL path outside the private runner root")
    if cleanup_required:
        stopped = run_step("postgres-stop", [str(bin_dir / "pg_ctl.exe"), "-D", str(data_dir),
                              "-m", "fast", "-w", "stop"], environment, output_dir, 180,
                            file_handles=True)
        if stopped["exit_code"]:
            print(f"preserving isolated PostgreSQL data because shutdown failed: {data_dir}", flush=True)
            stopped["owned_data_dir_preserved"] = data_dir.exists()
            stopped["owned_data_dir_removed"] = False
            return stopped
    else:
        stopped = {"name": "postgres-stop", "exit_code": 0, "reason": "startup was never attempted"}
    if data_dir.exists() and preserve_data:
        print(f"preserving isolated PostgreSQL data because acceptance failed: {data_dir}", flush=True)
        stopped["owned_data_dir_preserved"] = True
        stopped["owned_data_dir_removed"] = False
    elif data_dir.exists():
        shutil.rmtree(resolved_data)
        stopped["owned_data_dir_preserved"] = False
        stopped["owned_data_dir_removed"] = True
    else:
        stopped["owned_data_dir_preserved"] = False
        stopped["owned_data_dir_removed"] = False
    return stopped


def main() -> int:
    import argparse

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--postgres-bin", type=Path, default=Path("D:/PostgreSQL/18/bin"))
    parser.add_argument("--cargo", type=Path, default=Path("E:/DevCache/cargo/bin/cargo.exe"))
    parser.add_argument("--run-unit-tests", action="store_true")
    parser.add_argument("--python", type=str, default=sys.executable)
    options = parser.parse_args()

    RUNNER_ROOT.mkdir(parents=True, exist_ok=True)
    log_dir = RUNNER_ROOT / "logs" / uuid.uuid4().hex
    data_dir = RUNNER_ROOT / "postgres" / f"data-{uuid.uuid4().hex}"
    data_dir.parent.mkdir(parents=True, exist_ok=True)
    if data_dir.exists() or not data_dir.resolve().is_relative_to((RUNNER_ROOT / "postgres").resolve()):
        raise RuntimeError("unique disposable PostgreSQL data path check failed")
    port = select_loopback_port()
    environment = sanitized_environment()
    results: list[dict] = []
    cleanup_required = False
    cluster_ready = False
    status = 1
    try:
        if options.run_unit_tests:
            results.append(run_step(
                "runner-unit-tests", [options.python, "-m", "unittest", "-v",
                                      "scripts.automation.test_phase9f2_binding_http_acceptance"],
                environment, log_dir, 180,
            ))
            if results[-1]["exit_code"]:
                return 1
        migration_results, cleanup_required, cluster_ready = start_cluster(
            options.postgres_bin, port, data_dir, environment, log_dir,
        )
        results.extend(migration_results)
        if not cluster_ready or any(row["exit_code"] for row in results):
            return 1
        results.extend(apply_migrations(options.postgres_bin, port, environment, log_dir))
        if any(row["exit_code"] for row in results):
            return 1
        results.extend(seed_fixture(options.postgres_bin, port, environment, log_dir))
        if any(row["exit_code"] for row in results):
            return 1
        results.append(run_device_auth_pg_tests(options.cargo, port, environment, log_dir))
        if results[-1]["exit_code"]:
            return 1
        results.append(run_rust_phase(options.cargo, "hostless-run", port, environment, log_dir))
        if results[-1]["exit_code"]:
            return 1

        # Hostless Run creation has its own Outbox/idempotency facts; take the
        # binding-race baseline after that independent HTTP acceptance phase.
        baseline, snapshot_result = read_snapshot(options.postgres_bin, port, environment, log_dir, 0)
        results.append(snapshot_result)
        if baseline is None:
            return 1

        results.append(run_rust_phase(options.cargo, "bind-race", port, environment, log_dir))
        if results[-1]["exit_code"]:
            return 1
        bound, snapshot_result = read_snapshot(options.postgres_bin, port, environment, log_dir, 1)
        results.append(snapshot_result)
        if bound is None:
            return 1
        assert_snapshot_delta(baseline, bound, {
            "binding_total": 1, "binding_active": 1, "binding_closed": 0,
            "outbox_bound": 1, "outbox_detached": 0,
            "audit_created": 1, "audit_closed": 0, "idempotency": 1,
        }, "authenticated bind and concurrent CAS")

        results.append(set_detach_failure(options.postgres_bin, port, environment, log_dir, True))
        if results[-1]["exit_code"]:
            return 1
        results.append(run_rust_phase(options.cargo, "rollback-detach", port, environment, log_dir))
        if results[-1]["exit_code"]:
            return 1
        rolled_back, snapshot_result = read_snapshot(options.postgres_bin, port, environment, log_dir, 2)
        results.append(snapshot_result)
        if rolled_back is None:
            return 1
        assert_snapshot_delta(bound, rolled_back, {
            "binding_total": 0, "binding_active": 0, "binding_closed": 0,
            "outbox_bound": 0, "outbox_detached": 0,
            "audit_created": 0, "audit_closed": 0, "idempotency": 0,
        }, "failed detach transaction rollback")

        results.append(set_detach_failure(options.postgres_bin, port, environment, log_dir, False))
        if results[-1]["exit_code"]:
            return 1
        results.append(run_rust_phase(options.cargo, "detach", port, environment, log_dir))
        if results[-1]["exit_code"]:
            return 1
        detached, snapshot_result = read_snapshot(options.postgres_bin, port, environment, log_dir, 3)
        results.append(snapshot_result)
        if detached is None:
            return 1
        assert_snapshot_delta(rolled_back, detached, {
            "binding_total": 0, "binding_active": -1, "binding_closed": 1,
            "outbox_bound": 0, "outbox_detached": 1,
            "audit_created": 0, "audit_closed": 1, "idempotency": 1,
        }, "authenticated detach and exact replay")
        status = 0
        return status
    except Exception as error:
        results.append({"name": "acceptance-assertion", "exit_code": 1,
                        "error": f"{type(error).__name__}: {error}"})
        print(f"ACCEPTANCE_ASSERTION_FAILURE {type(error).__name__}: {error}", flush=True)
        return 1
    finally:
        if cleanup_required:
            stop = stop_and_remove_cluster(options.postgres_bin, data_dir, environment,
                                           log_dir, cleanup_required=True,
                                           preserve_data=status != 0)
            results.append(stop)
            if stop.get("exit_code") != 0:
                status = 1
        log_dir.mkdir(parents=True, exist_ok=True)
        summary = {
            "phase": "WLD0_BINDING_HTTP_POSITIVE_20261023",
            "isolated_loopback_port": port,
            "postgres_data_dir": str(data_dir),
            "postgres_bin": str(options.postgres_bin),
            "runtime_role": RUNTIME_ROLE,
            "target_database_touched": False,
            "multipass_or_docker_used": False,
            "results": results,
            "final_status": status,
        }
        summary_path = log_dir / "acceptance-summary.json"
        summary_path.write_text(json.dumps(summary, ensure_ascii=False, indent=2), encoding="utf-8")
        print(f"ACCEPTANCE_SUMMARY {summary_path}", flush=True)
        print(f"ACCEPTANCE_FINAL_STATUS {status}", flush=True)


if __name__ == "__main__":
    raise SystemExit(main())
