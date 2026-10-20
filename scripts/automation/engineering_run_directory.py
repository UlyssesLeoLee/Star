"""
CYPHER STRUCTURE MANIFEST
CREATE (f:File {name:"engineering_run_directory.py",type:"file",language:"python"}),
 (cargo:Function {name:"find_cargo",type:"function"}), (run:Function {name:"run_step",type:"function"}),
 (main:Function {name:"main",type:"function"}), (root:Variable {name:"ROOT",type:"variable"}),
 (ddl:Function {name:"validate_postgres_ddl",type:"function"}), (guards:Function {name:"task_run_identity_guard_sql",type:"function"}), (uuid:Function {name:"uuid4",type:"function"}),
 (migrationPlan:Function {name:"directory_migration_plan",type:"function"}), (applyPlan:Function {name:"apply_migration_plan",type:"function"}),
 (bindingCatalog:Function {name:"run_binding_catalog_sql",type:"function"}), (fingerprint:Function {name:"migration_fingerprint_sql",type:"function"}),
 (remove:Function {name:"shutil.rmtree",type:"function"}), (relative:Function {name:"Path.is_relative_to",type:"function"}),
 (args:Class {name:"argparse.ArgumentParser",type:"class"}), (process:Function {name:"subprocess.run",type:"function"}),
 (which:Function {name:"shutil.which",type:"function"}), (home:Function {name:"Path.home",type:"function"}),
 (loads:Function {name:"tomllib.loads",type:"function"}), (read:Function {name:"Path.read_text",type:"function"}),
 (subn:Function {name:"re.subn",type:"function"}),
 (bytes:Function {name:"Path.read_bytes",type:"function"}), (write:Function {name:"Path.write_bytes",type:"function"}),
 (exists:Function {name:"Path.is_file",type:"function"}), (text:Function {name:"Path.write_text",type:"function"}),
 (mkdir:Function {name:"Path.mkdir",type:"function"}), (parse:Function {name:"ArgumentParser.parse_args",type:"function"}),
 (add:Function {name:"ArgumentParser.add_argument",type:"function"}), (dump:Function {name:"json.dumps",type:"function"}),
 (f)-[:CONTAINS]->(cargo),(f)-[:CONTAINS]->(run),(f)-[:CONTAINS]->(main),(f)-[:CONTAINS]->(root),
 (cargo)-[:CALLS]->(which),(cargo)-[:CALLS]->(home),(cargo)-[:CALLS]->(loads),(cargo)-[:CALLS]->(read),(cargo)-[:CALLS]->(exists),
 (run)-[:CALLS]->(process),(run)-[:CALLS]->(mkdir),(run)-[:CALLS]->(text),
 (main)-[:CALLS]->(cargo),(main)-[:CALLS]->(run),(main)-[:CALLS]->(bytes),(main)-[:CALLS]->(write),(main)-[:CALLS]->(exists),
 (main)-[:CALLS]->(parse),(main)-[:CALLS]->(add),(main)-[:CALLS]->(dump),(main)-[:CALLS]->(text),
 (main)-[:CALLS]->(subn),
 (f)-[:CONTAINS]->(ddl),(f)-[:CONTAINS]->(guards),(main)-[:CALLS]->(ddl),(ddl)-[:CALLS]->(guards),(ddl)-[:CALLS]->(run),(ddl)-[:CALLS]->(uuid),(ddl)-[:CALLS]->(remove),(ddl)-[:CALLS]->(relative),
 (f)-[:CONTAINS]->(migrationPlan),(f)-[:CONTAINS]->(applyPlan),(f)-[:CONTAINS]->(bindingCatalog),(f)-[:CONTAINS]->(fingerprint),
 (ddl)-[:CALLS]->(migrationPlan),(ddl)-[:CALLS]->(applyPlan),(ddl)-[:CALLS]->(bindingCatalog),(ddl)-[:CALLS]->(fingerprint),
 (cargo)-[:USES]->(root),(run)-[:USES]->(root),(main)-[:USES]->(root);

Compile the canonical Branch/Engineering Run directory slice. Runner tests run only with --runner-tests and Rust tests
only with --cli-tests; --run-binding-ddl applies the audited migrations once in a fresh disposable cluster. No production migrations,
credential inspection, remote writes or workspace-wide source formatting are run.
Temporary Cargo resolution is opt-in and the original lock bytes are restored.
See docs/automation-design.md sections 4.36 and 4.37. Optional Task Run DDL checks exercise
the new forward CLI identity gate and snapshot/FK constraints in the disposable cluster only.
"""

# @cypher schema=1 source_sha256=8bf7385ce343a4d7e99a0ccdebc24d354ef4ff3dc73b135c2ed1c1cab12e1e92
# MERGE (self:File {path:"scripts/automation/engineering_run_directory.py"})
# MERGE (find_cargo:Symbol {id:"scripts/automation/engineering_run_directory.py::find_cargo",kind:"function"})
# MERGE (run_step:Symbol {id:"scripts/automation/engineering_run_directory.py::run_step",kind:"function"})
# MERGE (task_guard:Symbol {id:"scripts/automation/engineering_run_directory.py::task_run_identity_guard_sql",kind:"function"})
# MERGE (validate_ddl:Symbol {id:"scripts/automation/engineering_run_directory.py::validate_postgres_ddl",kind:"function"})
# MERGE (migration_plan:Symbol {id:"scripts/automation/engineering_run_directory.py::directory_migration_plan",kind:"function"})
# MERGE (apply_plan:Symbol {id:"scripts/automation/engineering_run_directory.py::apply_migration_plan",kind:"function"})
# MERGE (binding_catalog:Symbol {id:"scripts/automation/engineering_run_directory.py::run_binding_catalog_sql",kind:"function"})
# MERGE (fingerprint:Symbol {id:"scripts/automation/engineering_run_directory.py::migration_fingerprint_sql",kind:"function"})
# MERGE (main:Symbol {id:"scripts/automation/engineering_run_directory.py::main",kind:"function"})
# MERGE (root:Config {id:"scripts/automation/engineering_run_directory.py::ROOT"})
# MERGE (callable:Type {id:"collections.abc.Callable"})
# MERGE (executable:Config {id:"python.sys.executable"})
# MERGE (process:ExternalService {id:"python.subprocess.run",name:"Python subprocess runner"})
# MERGE (self)-[:DEFINES]->(find_cargo)
# MERGE (self)-[:DEFINES]->(run_step)
# MERGE (self)-[:DEFINES]->(task_guard)
# MERGE (self)-[:DEFINES]->(validate_ddl)
# MERGE (self)-[:DEFINES]->(migration_plan)
# MERGE (self)-[:DEFINES]->(apply_plan)
# MERGE (self)-[:DEFINES]->(binding_catalog)
# MERGE (self)-[:DEFINES]->(fingerprint)
# MERGE (self)-[:DEFINES]->(main)
# MERGE (self)-[:DEFINES]->(root)
# MERGE (run_step)-[:CALLS]->(process)
# MERGE (main)-[:CALLS]->(find_cargo)
# MERGE (main)-[:CALLS]->(run_step)
# MERGE (main)-[:CALLS]->(validate_ddl)
# MERGE (validate_ddl)-[:CALLS]->(run_step)
# MERGE (validate_ddl)-[:CALLS]->(task_guard)
# MERGE (validate_ddl)-[:CALLS]->(migration_plan)
# MERGE (validate_ddl)-[:CALLS]->(apply_plan)
# MERGE (validate_ddl)-[:CALLS]->(binding_catalog)
# MERGE (validate_ddl)-[:CALLS]->(fingerprint)
# MERGE (apply_plan)-[:USES_TYPE]->(callable)
# MERGE (main)-[:READS]->(executable)
# MERGE (find_cargo)-[:USES_TYPE]->(root)
# MERGE (run_step)-[:USES_TYPE]->(root)
# MERGE (validate_ddl)-[:USES_TYPE]->(root)
# MERGE (main)-[:USES_TYPE]->(root)
# @endcypher

from __future__ import annotations

import argparse
from collections.abc import Callable
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tomllib
from uuid import uuid4

ROOT = Path(__file__).resolve().parents[2]


def find_cargo() -> Path:
    resolved = shutil.which("cargo")
    if resolved:
        return Path(resolved)
    channel = tomllib.loads((ROOT / "rust-toolchain.toml").read_text(encoding="utf-8"))["toolchain"]["channel"]
    candidate = Path.home() / ".rustup" / "toolchains" / f"{channel}-x86_64-pc-windows-msvc" / "bin" / "cargo.exe"
    if not candidate.is_file():
        raise FileNotFoundError("Pinned Cargo toolchain is unavailable")
    return candidate


def run_step(name: str, command: list[str], workdir: Path, environment: dict[str, str], output_dir: Path) -> dict:
    output_dir.mkdir(parents=True, exist_ok=True)
    log_path = output_dir / f"{name}.log"
    try:
        # A background postgres child can inherit pipe handles after pg_ctl exits.
        # Redirect to a file so waiting for the launcher never waits for the server.
        with log_path.open("w", encoding="utf-8") as stream:
            result = subprocess.run(command, cwd=workdir, env=environment, shell=False,
                                    stdout=stream, stderr=subprocess.STDOUT, timeout=600,
                                    creationflags=subprocess.CREATE_NO_WINDOW if os.name == "nt" else 0)
        output = log_path.read_text(encoding="utf-8", errors="replace")
        status = result.returncode
    except subprocess.TimeoutExpired:
        output, status = "Compile gate exceeded 600 seconds; no pass recorded.", 124
    except OSError as error:
        output, status = f"Compile process unavailable ({type(error).__name__}).", 127
    log_path.write_text(output, encoding="utf-8")
    print(f"{name}: exit={status}; log={log_path}")
    print("\n".join(output.splitlines()[-24:]))
    return {"name": name, "exit_code": status, "log": str(log_path)}


def directory_migration_plan(run_binding: bool = False) -> list[tuple[str, str]]:
    """Return the disposable directory chain; October 15 runs once, never as a replay probe."""
    migrations = [
        ("postgres-worktree-base", "2026-09-16-worktree-canvas-worktree.sql"),
        ("postgres-project-binding", "2026-09-29-worktree-group-phase-2b.sql"),
        ("postgres-directory-first", "2026-10-01-engineering-run-directory.sql"),
        ("postgres-directory-repeat", "2026-10-01-engineering-run-directory.sql"),
    ]
    if run_binding:
        migrations.extend([
            ("postgres-run-host-independent-directory", "2026-10-15-run-host-independent-directory.sql"),
            ("postgres-run-worktree-bindings", "2026-10-19-run-worktree-bindings.sql"),
        ])
    return migrations


def apply_migration_plan(
    migrations: list[tuple[str, str]],
    apply_step: Callable[[str, str], dict],
    fingerprint_step: Callable[[str, str], dict],
) -> list[dict]:
    """Apply ordered migrations and fingerprint each committed schema; stop on any failure."""
    results = []
    for name, migration in migrations:
        result = apply_step(name, migration)
        results.append(result)
        if result["exit_code"] != 0:
            break
        fingerprint = fingerprint_step(name, migration)
        results.append(fingerprint)
        if fingerprint["exit_code"] != 0:
            break
    return results


def migration_fingerprint_sql() -> str:
    """Fingerprint relevant columns, constraints, indexes, RLS flags and policies."""
    return """WITH signatures AS (
      SELECT format('column:%I.%I:%s:%s:%s:%s', table_schema,table_name,ordinal_position,
                    column_name,data_type,is_nullable) AS signature
        FROM information_schema.columns
       WHERE table_schema IN ('scm','permission','multica')
      UNION ALL
      SELECT format('constraint:%I.%I:%I:%s',n.nspname,r.relname,c.conname,pg_get_constraintdef(c.oid,true))
        FROM pg_constraint c JOIN pg_class r ON r.oid=c.conrelid
        JOIN pg_namespace n ON n.oid=r.relnamespace
       WHERE n.nspname IN ('scm','permission','multica')
      UNION ALL
      SELECT format('index:%I.%I:%I:%s',n.nspname,r.relname,i.relname,pg_get_indexdef(i.oid))
        FROM pg_index x JOIN pg_class r ON r.oid=x.indrelid
        JOIN pg_class i ON i.oid=x.indexrelid JOIN pg_namespace n ON n.oid=r.relnamespace
       WHERE n.nspname IN ('scm','permission','multica')
      UNION ALL
      SELECT format('rls:%I.%I:%s:%s',n.nspname,r.relname,r.relrowsecurity,r.relforcerowsecurity)
        FROM pg_class r JOIN pg_namespace n ON n.oid=r.relnamespace
       WHERE n.nspname IN ('scm','permission','multica') AND r.relkind IN ('r','p')
      UNION ALL
      SELECT format('policy:%I.%I:%I:%s:%s:%s',n.nspname,r.relname,p.polname,p.polcmd,
                    coalesce(pg_get_expr(p.polqual,p.polrelid),''),
                    coalesce(pg_get_expr(p.polwithcheck,p.polrelid),''))
        FROM pg_policy p JOIN pg_class r ON r.oid=p.polrelid
        JOIN pg_namespace n ON n.oid=r.relnamespace
       WHERE n.nspname IN ('scm','permission','multica')
    )
    SELECT md5(coalesce(string_agg(signature,E'\\n' ORDER BY signature),'')) AS schema_fingerprint
      FROM signatures;"""


def run_binding_catalog_sql() -> str:
    """Check the new Run binding tables and lifecycle Outbox allowlist as DDL evidence only."""
    return """DO $$ DECLARE total int; invalid_events int; event_check text; actual_events text[]; BEGIN
      SELECT count(*) INTO total FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace
      WHERE (n.nspname,c.relname) IN (
       ('scm','cloud_branch'),('scm','cloud_branch_revision'),('permission','cloud_branch_role_binding'),
       ('multica','engineering_run'),('multica','engineering_run_revision'),
       ('permission','engineering_run_role_binding'),('multica','engineering_run_worktree_binding'),
       ('multica','engineering_directory_event'),('multica','engineering_run_command_idempotency'),
       ('multica','engineering_run_outbox')) AND c.relrowsecurity AND c.relforcerowsecurity;
      IF total <> 10 THEN RAISE EXCEPTION 'Run binding FORCE RLS catalog incomplete'; END IF;
      SELECT count(*) INTO invalid_events FROM multica.engineering_run_outbox
       WHERE event_type NOT IN ('engineering_run.created','engineering_run.worktree_bound',
                                'engineering_run.worktree_reassigned','engineering_run.worktree_detached');
      IF invalid_events <> 0 THEN RAISE EXCEPTION 'Run Outbox contains values outside the lifecycle allowlist'; END IF;
      SELECT pg_get_constraintdef(oid,true) INTO event_check FROM pg_constraint
       WHERE conrelid='multica.engineering_run_outbox'::regclass
         AND conname='ck_engineering_run_outbox_event_type';
      IF event_check IS NULL THEN
        RAISE EXCEPTION 'Run Outbox lifecycle allowlist definition is missing or incomplete';
      END IF;
      SELECT array_agg(found_value[1] ORDER BY found_value[1]) INTO actual_events
        FROM regexp_matches(event_check,$re$(engineering_run[.][a-z_]+)$re$,'g') AS matches(found_value);
      IF actual_events IS DISTINCT FROM ARRAY[
          'engineering_run.created','engineering_run.worktree_bound',
          'engineering_run.worktree_detached','engineering_run.worktree_reassigned']::text[] THEN
        RAISE EXCEPTION 'Run Outbox lifecycle allowlist differs from the expected four event types: %', actual_events;
      END IF;
      RAISE NOTICE '10 directory tables have FORCE RLS and Run Outbox schema is valid; runtime grants, behavior and writes remain unverified';
    END $$;"""


def task_run_identity_guard_sql() -> str:
    """Validate forward CLI guards without supplying fake authoritative directory records."""
    return """DO $$
    DECLARE
      tenant uuid := gen_random_uuid(); project uuid := gen_random_uuid();
      task uuid := gen_random_uuid(); actor uuid := gen_random_uuid();
      repository uuid := gen_random_uuid(); branch uuid := gen_random_uuid();
      engineering_run uuid := gen_random_uuid(); worktree uuid := gen_random_uuid();
      project_binding uuid := gen_random_uuid(); run_binding uuid := gen_random_uuid();
      snapshot jsonb; failure_constraint text;
    BEGIN
      IF (SELECT count(*) FROM multica.task_execution_run
          WHERE task_snapshot->>'ddl_fixture' = 'legacy' AND engineering_run_id IS NULL) <> 1 THEN
        RAISE EXCEPTION 'historical unbound CLI row was not preserved';
      END IF;
      BEGIN
        INSERT INTO multica.task_execution_run
          (tenant_id,project_id,work_item_id,initiated_by,execution_channel,task_snapshot,correlation_id)
          VALUES(tenant,project,task,actor,'cli','{}',gen_random_uuid());
        RAISE EXCEPTION 'new unbound CLI row was accepted';
      EXCEPTION WHEN check_violation THEN
        RAISE NOTICE 'new CLI identity NULL gate rejects inserts';
      END;
      snapshot := jsonb_build_object('tenant_id',tenant,'project_id',project,'repository_id',repository,
        'branch_id',branch,'engineering_run_id',engineering_run,'worktree_id',worktree,'work_item_id',task,
        'worktree_project_binding_id',project_binding,'engineering_run_worktree_binding_id',run_binding);
      BEGIN
        INSERT INTO multica.task_execution_run
          (tenant_id,project_id,work_item_id,initiated_by,execution_channel,task_snapshot,correlation_id,
           repository_id,branch_id,engineering_run_id,worktree_id,runtime_id,worktree_project_binding_id,
           engineering_run_worktree_binding_id,engineering_run_snapshot)
          VALUES(tenant,project,task,actor,'cli','{}',gen_random_uuid(),repository,branch,engineering_run,
            worktree,gen_random_uuid(),project_binding,run_binding,snapshot);
        RAISE EXCEPTION 'tuple-only JSON without grant/revision evidence was accepted';
      EXCEPTION WHEN check_violation THEN
        GET STACKED DIAGNOSTICS failure_constraint = CONSTRAINT_NAME;
        IF failure_constraint <> 'ck_task_run_engineering_run_snapshot_shape' THEN RAISE; END IF;
        RAISE NOTICE 'tuple-only JSON without complete grant/revision evidence rejected';
      END;
      snapshot := snapshot || jsonb_build_object('branch_full_ref','refs/heads/dev',
        'worktree_project_binding_version',1,'project_role_binding_id',gen_random_uuid(),
        'project_role_binding_version',1,'project_role','developer','branch_revision_id',gen_random_uuid(),
        'branch_revision_version',1,'branch_role_binding_id',gen_random_uuid(),'branch_role_binding_version',1,
        'branch_role','developer','engineering_run_revision_id',gen_random_uuid(),
        'engineering_run_revision_version',1,'engineering_run_role_binding_id',gen_random_uuid(),
        'engineering_run_role_binding_version',1,'engineering_run_role','developer',
        'engineering_run_worktree_binding_version',1);
      BEGIN
        INSERT INTO multica.task_execution_run
          (tenant_id,project_id,work_item_id,initiated_by,execution_channel,task_snapshot,correlation_id,
           repository_id,branch_id,engineering_run_id,worktree_id,runtime_id,worktree_project_binding_id,
           engineering_run_worktree_binding_id,engineering_run_snapshot)
          VALUES(tenant,project,task,actor,'cli','{}',gen_random_uuid(),repository,branch,engineering_run,
            worktree,gen_random_uuid(),project_binding,run_binding,snapshot - 'project_id');
        RAISE EXCEPTION 'incomplete JSON scope was accepted';
      EXCEPTION WHEN check_violation THEN
        GET STACKED DIAGNOSTICS failure_constraint = CONSTRAINT_NAME;
        IF failure_constraint NOT IN ('ck_task_run_engineering_run_identity_complete',
                                      'ck_task_run_engineering_run_snapshot_shape') THEN RAISE; END IF;
        RAISE NOTICE 'missing JSON tuple field rejected';
      END;
      BEGIN
        INSERT INTO multica.task_execution_run
          (tenant_id,project_id,work_item_id,initiated_by,execution_channel,task_snapshot,correlation_id,
           repository_id,branch_id,engineering_run_id,worktree_id,runtime_id,worktree_project_binding_id,
           engineering_run_worktree_binding_id,engineering_run_snapshot)
          VALUES(tenant,project,task,actor,'cli','{}',gen_random_uuid(),repository,branch,engineering_run,
            worktree,gen_random_uuid(),project_binding,run_binding,
            snapshot || jsonb_build_object('oversized_fixture',repeat('x',8193)));
        RAISE EXCEPTION 'oversized JSON scope was accepted';
      EXCEPTION WHEN check_violation THEN
        GET STACKED DIAGNOSTICS failure_constraint = CONSTRAINT_NAME;
        IF failure_constraint NOT IN ('ck_task_run_engineering_run_identity_complete',
                                      'ck_task_run_engineering_run_snapshot_shape') THEN RAISE; END IF;
        RAISE NOTICE 'oversized JSON identity rejected';
      END;
      BEGIN
        INSERT INTO multica.task_execution_run
          (tenant_id,project_id,work_item_id,initiated_by,execution_channel,task_snapshot,correlation_id,
           repository_id,branch_id,engineering_run_id,worktree_id,runtime_id,worktree_project_binding_id,
           engineering_run_worktree_binding_id,engineering_run_snapshot)
          VALUES(tenant,project,task,actor,'cli','{}',gen_random_uuid(),repository,branch,engineering_run,
            worktree,gen_random_uuid(),project_binding,run_binding,snapshot);
        RAISE EXCEPTION 'missing canonical directory was accepted';
      EXCEPTION WHEN foreign_key_violation THEN
        RAISE NOTICE 'missing canonical directory/binding rejected by FK';
      END;
      RAISE NOTICE 'DDL guard scenarios passed; no real runtime-role/RLS or positive admission proven';
    END $$;"""


def validate_postgres_ddl(bin_dir: Path, port: int, environment: dict[str, str], output_dir: Path,
                          task_run: bool = False, run_binding: bool = False) -> list[dict]:
    """Create and remove an isolated loopback cluster; never accepts a production URL."""
    sandbox_root = (ROOT / ".cache" / "engineering-run-directory").resolve()
    data_dir = sandbox_root / f"postgres-{uuid4()}"
    if not data_dir.resolve().is_relative_to(sandbox_root) or data_dir.exists():
        raise ValueError("Isolated PostgreSQL path is invalid or already exists")
    steps = []
    started = False
    stopped = False
    try:
        steps.append(run_step("postgres-init", [str(bin_dir / "initdb.exe"), "-D", str(data_dir),
            "-U", "erun_validation", "--auth=trust", "--no-instructions"], ROOT, environment, output_dir))
        if steps[-1]["exit_code"]:
            return steps
        started = True  # Even a failed launcher may leave its server running.
        steps.append(run_step("postgres-start", [str(bin_dir / "pg_ctl.exe"), "-D", str(data_dir),
            "-l", str(data_dir / "server.log"), "-o", f"-h 127.0.0.1 -p {port}", "-w", "start"], ROOT, environment, output_dir))
        if steps[-1]["exit_code"]:
            return steps
        psql = [str(bin_dir / "psql.exe"), "-X", "-h", "127.0.0.1", "-p", str(port), "-U",
                "erun_validation", "-d", "postgres", "-v", "ON_ERROR_STOP=1"]
        # The legacy base migration requires an external audit_trigger_func not shipped
        # in this migration chain. This isolated fixture only permits DDL validation;
        # it provides no evidence about legacy audit behavior or production deployment.
        legacy_fixture = """CREATE FUNCTION public.audit_trigger_func() RETURNS trigger LANGUAGE plpgsql
            AS $$ BEGIN RETURN COALESCE(NEW, OLD); END $$;"""
        steps.append(run_step("postgres-legacy-prerequisite-fixture", psql + ["-c", legacy_fixture], ROOT, environment, output_dir))
        if steps[-1]["exit_code"]:
            return steps
        def apply_migration(name: str, migration: str) -> dict:
            return run_step(name, psql + ["-f", str(ROOT / "db" / "migrations" / migration)],
                            ROOT, environment, output_dir)

        def fingerprint_migration(name: str, _migration: str) -> dict:
            return run_step(f"{name}-schema-fingerprint", psql + ["-t", "-A", "-c", migration_fingerprint_sql()],
                            ROOT, environment, output_dir)

        steps.extend(apply_migration_plan(directory_migration_plan(run_binding), apply_migration,
                                         fingerprint_migration))
        if any(row["exit_code"] != 0 for row in steps):
            return steps
        catalog_sql = """DO $$ DECLARE total int; BEGIN
          SELECT count(*) INTO total FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace
          WHERE (n.nspname,c.relname) IN (
           ('scm','cloud_branch'),('scm','cloud_branch_revision'),('permission','cloud_branch_role_binding'),
           ('multica','engineering_run'),('multica','engineering_run_revision'),
           ('permission','engineering_run_role_binding'),('multica','engineering_run_worktree_binding'),
           ('multica','engineering_directory_event')) AND c.relrowsecurity AND c.relforcerowsecurity;
          IF total <> 8 THEN RAISE EXCEPTION 'directory FORCE RLS catalog incomplete'; END IF;
          RAISE NOTICE '8 directory tables have FORCE RLS; role behavior and write workflows remain unverified';
          END $$;"""
        steps.append(run_step("postgres-directory-catalog", psql + ["-c", catalog_sql], ROOT, environment, output_dir))
        if steps[-1]["exit_code"]:
            return steps
        if run_binding:
            steps.append(run_step("postgres-run-binding-catalog", psql + ["-c", run_binding_catalog_sql()],
                                  ROOT, environment, output_dir))
            if steps[-1]["exit_code"]:
                return steps
        if not task_run:
            return steps
        steps.append(run_step("postgres-task-metadata", psql + ["-f", str(ROOT / "db" / "migrations" /
            "2026-09-29-worktree-group-phase-2c.sql")], ROOT, environment, output_dir))
        if steps[-1]["exit_code"]:
            return steps
        steps.append(run_step("postgres-task-run-base", psql + ["-f", str(ROOT / "db" / "migrations" /
            "2026-09-30-worktree-task-execution-run.sql")], ROOT, environment, output_dir))
        if steps[-1]["exit_code"]:
            return steps
        legacy_sql = """INSERT INTO multica.task_execution_run
          (tenant_id,project_id,work_item_id,initiated_by,execution_channel,task_snapshot,correlation_id)
          VALUES(gen_random_uuid(),gen_random_uuid(),gen_random_uuid(),gen_random_uuid(),'cli',
                 '{"ddl_fixture":"legacy"}',gen_random_uuid());"""
        steps.append(run_step("postgres-task-run-legacy", psql + ["-c", legacy_sql], ROOT, environment, output_dir))
        if steps[-1]["exit_code"]:
            return steps
        for name in ["postgres-task-run-identity-first", "postgres-task-run-identity-repeat"]:
            steps.append(run_step(name, psql + ["-f", str(ROOT / "db" / "migrations" /
                "2026-10-02-task-run-engineering-run-binding.sql")], ROOT, environment, output_dir))
            if steps[-1]["exit_code"]:
                return steps
        steps.append(run_step("postgres-task-run-identity-guards", psql + ["-c", task_run_identity_guard_sql()],
                              ROOT, environment, output_dir))
        return steps
    finally:
        if started:
            stop = run_step("postgres-stop", [str(bin_dir / "pg_ctl.exe"), "-D", str(data_dir),
                            "-m", "fast", "-w", "stop"], ROOT, environment, output_dir)
            steps.append(stop)
            stopped = stop["exit_code"] == 0
        else:
            stopped = True
        if stopped and data_dir.is_dir() and data_dir.resolve().is_relative_to(sandbox_root):
            shutil.rmtree(data_dir)
        if not stopped:
            print(f"Isolated PostgreSQL stop failed; data preserved: {data_dir}")


def main() -> int:
    parser = argparse.ArgumentParser(description="Check Engineering Run directory; no production mutation")
    parser.add_argument("--rust", action="store_true")
    parser.add_argument("--frontend", action="store_true")
    parser.add_argument("--postgres-ddl", action="store_true", help="Apply only DDL in a fresh disposable loopback cluster")
    parser.add_argument("--run-binding-ddl", action="store_true",
                        help="Also apply October 15/19 Run binding migrations once in a fresh disposable cluster")
    parser.add_argument("--task-run-ddl", action="store_true", help="Also check Task Run identity forward guards in a disposable cluster")
    parser.add_argument("--postgres-bin", type=Path, default=Path("D:/PostgreSQL/18/bin"))
    parser.add_argument("--postgres-port", type=int, default=57432)
    parser.add_argument("--allow-lock-resolution", action="store_true",
                        help="Opt in to transient workspace dependency resolution; restores Cargo.lock")
    parser.add_argument("--backend-only", action="store_true",
                        help="Temporarily exclude the desktop member for targeted backend checks; restores manifest/lock")
    parser.add_argument("--offline", action="store_true", help="Use only cached Cargo dependencies")
    parser.add_argument("--runner-tests", action="store_true", help="Run only the focused Python runner tests")
    parser.add_argument("--cli-tests", action="store_true", help="Explicitly run only CLI and Runtime fence unit tests")
    parser.add_argument("--cargo-target-dir", type=Path, help="Independent Cargo build directory")
    parser.add_argument("--output-dir", type=Path, default=ROOT / ".cache" / "engineering-run-directory")
    options = parser.parse_args()
    if not (options.rust or options.frontend or options.postgres_ddl or options.task_run_ddl
            or options.run_binding_ddl or options.runner_tests):
        parser.error("select --rust, --frontend, --postgres-ddl, --run-binding-ddl, --task-run-ddl, or --runner-tests")
    if not 1024 <= options.postgres_port <= 65535:
        parser.error("invalid disposable PostgreSQL port")
    if options.allow_lock_resolution and not options.rust:
        parser.error("--allow-lock-resolution requires --rust")
    if options.backend_only and not (options.rust and options.allow_lock_resolution):
        parser.error("--backend-only requires --rust and --allow-lock-resolution")
    if (options.cli_tests or options.offline or options.cargo_target_dir) and not options.rust:
        parser.error("Cargo options require --rust")
    environment = os.environ.copy()
    results = []
    if options.runner_tests:
        results.append(run_step("runner-tests", [sys.executable, "-m", "unittest", "discover", "-s",
                                str(ROOT / "scripts" / "automation"), "-p", "test_engineering_run_directory.py"],
                                ROOT, environment, options.output_dir))
    if options.rust:
        cargo = find_cargo()
        environment["PATH"] = str(cargo.parent) + os.pathsep + environment.get("PATH", "")
        lock_path = ROOT / "Cargo.lock"
        original_lock = lock_path.read_bytes()
        manifest_path = ROOT / "Cargo.toml"
        original_manifest = manifest_path.read_bytes()
        cargo_options = ["-j", "4"]
        if not options.allow_lock_resolution:
            cargo_options.append("--locked")
        if options.offline:
            cargo_options.append("--offline")
        if options.cargo_target_dir:
            cargo_options.extend(["--target-dir", str(options.cargo_target_dir)])
        command = [str(cargo), "check", "-p", "star-api-rest", "--all-targets"] + cargo_options
        try:
            if options.backend_only:
                candidate, count = re.subn(rb'(?m)^    "crates/star-desktop/src-tauri",\r?\n', b"", original_manifest)
                if count != 1:
                    raise ValueError("Expected exactly one desktop workspace member")
                manifest_path.write_bytes(candidate)
            results.append(run_step("rust-check", command, ROOT, environment, options.output_dir))
            if options.cli_tests and results[-1]["exit_code"] == 0:
                for name, package, test_filter in [
                    ("runtime-fence-tests", "domain-local-runtime", "profile_bound_fence_tests"),
                    ("rest-cli-tests", "star-api-rest", "group_api::cli_sessions::tests"),
                ]:
                    results.append(run_step(name, [str(cargo), "test", "-p", package, "--lib", test_filter]
                                            + cargo_options, ROOT, environment, options.output_dir))
                    if results[-1]["exit_code"]:
                        break
        finally:
            if options.backend_only:
                manifest_path.write_bytes(original_manifest)
            if options.allow_lock_resolution and lock_path.read_bytes() != original_lock:
                lock_path.write_bytes(original_lock)
        for row in results:
            row["temporary_lock_resolution"] = options.allow_lock_resolution
            row["temporary_desktop_exclusion"] = options.backend_only
    if options.frontend:
        npm = shutil.which("npm.cmd" if os.name == "nt" else "npm")
        if not npm:
            raise FileNotFoundError("npm executable is unavailable")
        results.append(run_step("frontend-typecheck", [npm, "run", "typecheck"], ROOT / "frontend",
                                environment, options.output_dir))
    if options.postgres_ddl or options.task_run_ddl or options.run_binding_ddl:
        results.extend(validate_postgres_ddl(options.postgres_bin, options.postgres_port, environment,
                                            options.output_dir, options.task_run_ddl, options.run_binding_ddl))
    summary_path = options.output_dir / "summary.json"
    summary_path.write_text(json.dumps({"gates": results,
                                       "tests_run": any(row["name"].endswith("-tests") for row in results),
                                       "ddl_guard_scenarios_run": options.task_run_ddl,
                                       "runner_tests_requested": options.runner_tests,
                                       "run_binding_ddl_requested": options.run_binding_ddl,
                                       "task_run_ddl_requested": options.task_run_ddl,
                                       "production_migrations_run": False},
                                      ensure_ascii=False, indent=2), encoding="utf-8")
    return 0 if all(row["exit_code"] == 0 for row in results) else 1


if __name__ == "__main__":
    raise SystemExit(main())
