"""Run authenticated Run-to-Worktree HTTP acceptance on an isolated local PostgreSQL cluster."""

# @cypher schema=1 source_sha256=a277dd8c2bb66429f683a8a35db60f380af893372614d2fff6f6f70ad7b97b0b
# MERGE (self:File {path:"scripts/automation/phase9f2_binding_http_acceptance.py"})
# MERGE (runnerRoot:Config {id:"scripts/automation/phase9f2_binding_http_acceptance.py::RUNNER_ROOT",kind:"path"})
# MERGE (protectedPorts:Config {id:"scripts/automation/phase9f2_binding_http_acceptance.py::PROTECTED_LOOPBACK_PORTS",kind:"port_set"})
# MERGE (main:Symbol {id:"scripts/automation/phase9f2_binding_http_acceptance.py::main",kind:"function"})
# MERGE (port:Symbol {id:"scripts/automation/phase9f2_binding_http_acceptance.py::select_loopback_port",kind:"function"})
# MERGE (environment:Symbol {id:"scripts/automation/phase9f2_binding_http_acceptance.py::sanitized_environment",kind:"function"})
# MERGE (step:Symbol {id:"scripts/automation/phase9f2_binding_http_acceptance.py::run_step",kind:"function"})
# MERGE (cluster:Symbol {id:"scripts/automation/phase9f2_binding_http_acceptance.py::start_cluster",kind:"function"})
# MERGE (psql:Symbol {id:"scripts/automation/phase9f2_binding_http_acceptance.py::psql_command",kind:"function"})
# MERGE (sql:Symbol {id:"scripts/automation/phase9f2_binding_http_acceptance.py::run_sql",kind:"function"})
# MERGE (migrations:Symbol {id:"scripts/automation/phase9f2_binding_http_acceptance.py::apply_migrations",kind:"function"})
# MERGE (fixture:Symbol {id:"scripts/automation/phase9f2_binding_http_acceptance.py::seed_fixture",kind:"function"})
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
# MERGE (self)-[:DEFINES]->(snapshot)
# MERGE (self)-[:DEFINES]->(assertion)
# MERGE (self)-[:DEFINES]->(test)
# MERGE (self)-[:DEFINES]->(fault)
# MERGE (self)-[:DEFINES]->(stop)
# MERGE (main)-[:CALLS]->(port)
# MERGE (main)-[:CALLS]->(environment)
# MERGE (main)-[:CALLS]->(step)
# MERGE (main)-[:CALLS]->(cluster)
# MERGE (main)-[:CALLS]->(migrations)
# MERGE (main)-[:CALLS]->(fixture)
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
TENANT_A = "10000000-0000-0000-0000-000000000001"
ACTOR_A = "30000000-0000-0000-0000-000000000001"
PROJECT_A = "20000000-0000-0000-0000-000000000001"
RUN_A = "60000000-0000-0000-0000-000000000001"
WORKTREE_A = "90000000-0000-0000-0000-000000000001"
PROJECT_BINDING = "50000000-0000-0000-0000-000000000001"


def sanitized_environment() -> dict[str, str]:
    """Keep unrelated user PostgreSQL settings from redirecting the fixture."""
    environment = os.environ.copy()
    for key in list(environment):
        if key.upper().startswith("PG") or key.upper() in {
            "DATABASE_URL", "STAR_DATABASE_URL", "STAR_BINDING_HTTP_TEST_DATABASE_URL",
            "STAR_BINDING_HTTP_TEST_PHASE", "SQLX_OFFLINE",
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
    """Apply and fingerprint the reviewed Run Directory migration chain in order."""
    sys.path.insert(0, str(ROOT / "scripts" / "automation"))
    import engineering_run_directory as directory

    results = [run_sql(
        "postgres-legacy-audit-prerequisite-fixture", bin_dir, port, environment, output_dir,
        sql="CREATE FUNCTION public.audit_trigger_func() RETURNS trigger LANGUAGE plpgsql "
            "AS $$ BEGIN RETURN COALESCE(NEW, OLD); END $$;",
    )]
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
    return results


def seed_fixture(bin_dir: Path, port: int, environment: dict[str, str],
                 output_dir: Path) -> list[dict]:
    """Seed one hostless Run, one Project-authorized Worktree, and a real runtime login."""
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
    results.append(run_sql(
        "postgres-runtime-role-grants-rls-acceptance", bin_dir, port, environment, output_dir,
        sql=directory.run_binding_role_acceptance_sql(), login=RUNTIME_ROLE,
    ))
    if results[-1]["exit_code"]:
        return results

    seed_sql = f"""BEGIN;
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
    command = ["rtk", "proxy", str(cargo), "+1.99.0", "test", "-j", "4", "--locked",
               "-p", "star-api-rest", "--test", "engineering_run_worktree_binding_routes",
               "real_postgres_authenticated_binding_http_acceptance", "--",
               "--ignored", "--exact", "--nocapture"]
    return run_step(f"rust-http-{phase}", command, phase_environment, output_dir, 3600)


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
