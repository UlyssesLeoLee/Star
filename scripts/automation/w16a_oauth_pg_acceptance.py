"""Run the W16-A OAuth transaction tests against a disposable local PostgreSQL 18 cluster."""

# @cypher schema=1 source_sha256=f12c38363d1c05ffeadba0cea38db9de2a7320ac51f9b664241340da93dbe8f9
# MERGE (self:File {path:"scripts/automation/w16a_oauth_pg_acceptance.py"})
# MERGE (root:Config {id:"scripts/automation/w16a_oauth_pg_acceptance.py::ROOT",kind:"path"})
# MERGE (runnerCache:Config {id:"scripts/automation/w16a_oauth_pg_acceptance.py::RUNNER_CACHE_ROOT",kind:"path"})
# MERGE (summaryArtifact:Config {id:".cache/w16a-oauth-pg-acceptance/runs/<run-id>/acceptance-summary.json",kind:"dynamic_output_path"})
# MERGE (databaseUrl:Config {id:"STAR_W16_DEVICE_AUTH_TEST_DATABASE_URL",kind:"environment_variable"})
# MERGE (expectedTests:Config {id:"scripts/automation/w16a_oauth_pg_acceptance.py::EXPECTED_PG_TESTS",kind:"test_set"})
# MERGE (main:Symbol {id:"scripts/automation/w16a_oauth_pg_acceptance.py::main",kind:"function"})
# MERGE (sanitize:Symbol {id:"scripts/automation/w16a_oauth_pg_acceptance.py::sanitized_environment",kind:"function"})
# MERGE (port:Symbol {id:"scripts/automation/w16a_oauth_pg_acceptance.py::select_loopback_port",kind:"function"})
# MERGE (inside:Symbol {id:"scripts/automation/w16a_oauth_pg_acceptance.py::is_inside",kind:"function"})
# MERGE (tail:Symbol {id:"scripts/automation/w16a_oauth_pg_acceptance.py::read_tail",kind:"function"})
# MERGE (step:Symbol {id:"scripts/automation/w16a_oauth_pg_acceptance.py::run_step",kind:"function"})
# MERGE (psqlCommand:Symbol {id:"scripts/automation/w16a_oauth_pg_acceptance.py::psql_command",kind:"function"})
# MERGE (sql:Symbol {id:"scripts/automation/w16a_oauth_pg_acceptance.py::run_sql",kind:"function"})
# MERGE (start:Symbol {id:"scripts/automation/w16a_oauth_pg_acceptance.py::start_cluster",kind:"function"})
# MERGE (apply:Symbol {id:"scripts/automation/w16a_oauth_pg_acceptance.py::apply_migrations",kind:"function"})
# MERGE (accepted:Symbol {id:"scripts/automation/w16a_oauth_pg_acceptance.py::accepted_test_output",kind:"function"})
# MERGE (cargoCommand:Symbol {id:"scripts/automation/w16a_oauth_pg_acceptance.py::cargo_test_command",kind:"function"})
# MERGE (testRun:Symbol {id:"scripts/automation/w16a_oauth_pg_acceptance.py::run_postgres_tests",kind:"function"})
# MERGE (stop:Symbol {id:"scripts/automation/w16a_oauth_pg_acceptance.py::stop_cluster",kind:"function"})
# MERGE (summary:Symbol {id:"scripts/automation/w16a_oauth_pg_acceptance.py::write_summary",kind:"function"})
# MERGE (self)-[:DEFINES]->(main),(self)-[:DEFINES]->(sanitize),(self)-[:DEFINES]->(port),(self)-[:DEFINES]->(inside),(self)-[:DEFINES]->(tail),(self)-[:DEFINES]->(step),(self)-[:DEFINES]->(psqlCommand),(self)-[:DEFINES]->(sql),(self)-[:DEFINES]->(start),(self)-[:DEFINES]->(apply),(self)-[:DEFINES]->(accepted),(self)-[:DEFINES]->(cargoCommand),(self)-[:DEFINES]->(testRun),(self)-[:DEFINES]->(stop),(self)-[:DEFINES]->(summary)
# MERGE (main)-[:CALLS]->(sanitize),(main)-[:CALLS]->(port),(main)-[:CALLS]->(inside),(main)-[:CALLS]->(start),(main)-[:CALLS]->(apply),(main)-[:CALLS]->(testRun),(main)-[:CALLS]->(stop),(main)-[:CALLS]->(summary)
# MERGE (main)-[:READS]->(root),(main)-[:READS]->(runnerCache)
# MERGE (sanitize)-[:CONFIGURES]->(databaseUrl)
# MERGE (step)-[:CALLS]->(inside),(step)-[:CALLS]->(tail)
# MERGE (psqlCommand)-[:READS]->(:ExternalService {name:"PostgreSQL 18 local cluster",kind:"database_server"})
# MERGE (sql)-[:CALLS]->(psqlCommand),(sql)-[:CALLS]->(step)
# MERGE (start)-[:CALLS]->(step),(start)-[:CALLS]->(psqlCommand)
# MERGE (apply)-[:CALLS]->(sql)
# MERGE (testRun)-[:CALLS]->(step),(testRun)-[:CALLS]->(accepted),(testRun)-[:CALLS]->(cargoCommand),(testRun)-[:READS]->(expectedTests),(testRun)-[:CONFIGURES]->(databaseUrl)
# MERGE (stop)-[:CALLS]->(step),(stop)-[:CALLS]->(inside)
# MERGE (summary)-[:WRITES]->(summaryArtifact)
# MERGE (testRun)-[:TESTS]->(:File {path:"crates/star-api-rest/tests/oauth_device_auth_pg.rs"})
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
from typing import Mapping

ROOT = Path(__file__).resolve().parents[2]
RUNNER_CACHE_ROOT = ROOT / ".cache" / "w16a-oauth-pg-acceptance"
DEFAULT_POSTGRES_BIN = Path("D:/PostgreSQL/18/bin")
DEFAULT_CARGO = shutil.which("cargo") or "cargo"
DEFAULT_RTK = shutil.which("rtk") or "rtk"
DATABASE_NAME = "w16a_oauth_acceptance"
EXPECTED_PG_TESTS = (
    "postgres_oauth_credentials_have_one_winner_when_test_database_is_configured",
    "postgres_auth_code_redemption_persists_tokens_with_consumption",
    "postgres_refresh_rotation_commits_one_successor_with_the_consume",
)
PG_TEST_BINARY = "oauth_device_auth_pg"
REQUIRED_POSTGRES_TOOLS = ("initdb.exe", "pg_ctl.exe", "postgres.exe", "psql.exe")
BASE_MIGRATION = ROOT / "db" / "migrations" / "2026-09-09-oauth2-server.sql"
W16_MIGRATION = ROOT / "db" / "migrations" / "2026-10-28-w16-device-auth.sql"
TEST_FILE = ROOT / "crates" / "star-api-rest" / "tests" / "oauth_device_auth_pg.rs"
MAX_LOG_TAIL_BYTES = 64 * 1024


def sanitized_environment(source: Mapping[str, str] | None = None) -> dict[str, str]:
    """Remove inherited PostgreSQL targets so this run can only use its private cluster."""
    environment = dict(os.environ if source is None else source)
    for key in list(environment):
        upper = key.upper()
        if upper.startswith("PG") or upper in {
            "DATABASE_URL",
            "STAR_DATABASE_URL",
            "STAR_W16_DEVICE_AUTH_TEST_DATABASE_URL",
            "SQLX_OFFLINE",
        }:
            environment.pop(key, None)
    environment["CARGO_TERM_COLOR"] = "never"
    return environment


def select_loopback_port() -> int:
    """Select a currently unused IPv4 loopback port for the disposable server."""
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as listener:
        listener.bind(("127.0.0.1", 0))
        return int(listener.getsockname()[1])


def is_inside(path: Path, root: Path) -> bool:
    """Return whether a resolved path is a strict descendant of the supplied root."""
    resolved_path = path.resolve()
    resolved_root = root.resolve()
    return resolved_path != resolved_root and resolved_path.is_relative_to(resolved_root)


def read_tail(path: Path, limit: int = MAX_LOG_TAIL_BYTES) -> str:
    """Read at most the final bounded number of bytes from a full evidence log."""
    if not path.is_file():
        return ""
    with path.open("rb") as log_file:
        log_file.seek(0, os.SEEK_END)
        size = log_file.tell()
        log_file.seek(max(0, size - limit))
        return log_file.read(limit).decode("utf-8", errors="replace")


def run_step(
    name: str,
    command: list[str],
    environment: Mapping[str, str],
    output_dir: Path,
    timeout: int,
) -> dict[str, object]:
    """Run one bounded command with file-backed logs and a bounded in-memory tail."""
    output_dir.mkdir(parents=True, exist_ok=True)
    stdout_path = output_dir / f"{name}.stdout.log"
    stderr_path = output_dir / f"{name}.stderr.log"
    print(f"COMMAND[{name}] {subprocess.list2cmdline(command)}", flush=True)
    try:
        with stdout_path.open("w", encoding="utf-8") as stdout_file, stderr_path.open(
            "w", encoding="utf-8"
        ) as stderr_file:
            completed = subprocess.run(
                command,
                cwd=ROOT,
                env=dict(environment),
                stdout=stdout_file,
                stderr=stderr_file,
                timeout=timeout,
                check=False,
            )
        result: dict[str, object] = {
            "name": name,
            "command": command,
            "exit_code": completed.returncode,
            "stdout_log": str(stdout_path.relative_to(ROOT)),
            "stderr_log": str(stderr_path.relative_to(ROOT)),
            "stdout_tail": read_tail(stdout_path),
            "stderr_tail": read_tail(stderr_path),
        }
    except FileNotFoundError as error:
        result = {
            "name": name,
            "command": command,
            "exit_code": 127,
            "stdout_log": str(stdout_path.relative_to(ROOT)),
            "stderr_log": str(stderr_path.relative_to(ROOT)),
            "stdout_tail": "",
            "stderr_tail": str(error),
            "error": "required executable was not found",
        }
        stdout_path.write_text("", encoding="utf-8")
        stderr_path.write_text(str(error), encoding="utf-8")
    except subprocess.TimeoutExpired as error:
        if error.stdout:
            stdout = error.stdout.decode(errors="replace") if isinstance(error.stdout, bytes) else error.stdout
            stdout_path.write_text(stdout, encoding="utf-8")
        if error.stderr:
            stderr = error.stderr.decode(errors="replace") if isinstance(error.stderr, bytes) else error.stderr
            stderr_path.write_text(stderr, encoding="utf-8")
        result = {
            "name": name,
            "command": command,
            "exit_code": 124,
            "stdout_log": str(stdout_path.relative_to(ROOT)),
            "stderr_log": str(stderr_path.relative_to(ROOT)),
            "error": f"timed out after {timeout} seconds",
        }
    result["stdout_tail"] = read_tail(stdout_path)
    result["stderr_tail"] = read_tail(stderr_path)
    print(f"EXIT[{name}] {result['exit_code']}", flush=True)
    if result["exit_code"]:
        for stream in ("stdout_tail", "stderr_tail"):
            tail = result.get(stream)
            if tail:
                print(f"{stream.upper()}[{name}]\n{tail}", flush=True)
    return result


def psql_command(
    postgres_bin: Path, port: int, database: str, sql: str | None = None, file: Path | None = None
) -> list[str]:
    command = [
        str(postgres_bin / "psql.exe"),
        "-X",
        "-h",
        "127.0.0.1",
        "-p",
        str(port),
        "-U",
        "postgres",
        "-d",
        database,
        "-v",
        "ON_ERROR_STOP=1",
    ]
    if sql is not None:
        command.extend(["-c", sql])
    elif file is not None:
        command.extend(["-f", str(file)])
    else:
        raise ValueError("psql command requires SQL or a file")
    return command


def run_sql(
    name: str,
    postgres_bin: Path,
    port: int,
    environment: Mapping[str, str],
    output_dir: Path,
    sql: str | None = None,
    file: Path | None = None,
) -> dict[str, object]:
    return run_step(
        name,
        psql_command(postgres_bin, port, DATABASE_NAME, sql=sql, file=file),
        environment,
        output_dir,
        timeout=180,
    )


def start_cluster(
    postgres_bin: Path,
    port: int,
    data_dir: Path,
    environment: Mapping[str, str],
    output_dir: Path,
) -> tuple[list[dict[str, object]], bool]:
    """Initialize a private trust-authenticated cluster and bind it only to loopback."""
    results: list[dict[str, object]] = []
    init = run_step(
        "postgres-init",
        [str(postgres_bin / "initdb.exe"), "-D", str(data_dir), "-U", "postgres", "--auth=trust", "--no-instructions"],
        environment,
        output_dir,
        timeout=300,
    )
    results.append(init)
    if init["exit_code"]:
        return results, False
    start_attempted = True
    start = run_step(
        "postgres-start",
        [
            str(postgres_bin / "pg_ctl.exe"),
            "-D",
            str(data_dir),
            "-l",
            str(data_dir / "server.log"),
            "-o",
            f"-h 127.0.0.1 -p {port}",
            "-w",
            "start",
        ],
        environment,
        output_dir,
        timeout=240,
    )
    results.append(start)
    if start["exit_code"]:
        return results, start_attempted
    ready = run_step(
        "postgres-ready",
        psql_command(postgres_bin, port, "postgres", sql="SELECT 1;"),
        environment,
        output_dir,
        timeout=30,
    )
    results.append(ready)
    return results, start_attempted


def apply_migrations(
    postgres_bin: Path, port: int, environment: Mapping[str, str], output_dir: Path
) -> list[dict[str, object]]:
    """Install test-only audit prerequisites and apply the exact OAuth/W16 migration pair."""
    results: list[dict[str, object]] = []
    create_database = run_step(
        "postgres-create-test-database",
        psql_command(postgres_bin, port, "postgres", sql=f"CREATE DATABASE {DATABASE_NAME};"),
        environment,
        output_dir,
        timeout=60,
    )
    results.append(create_database)
    if create_database["exit_code"]:
        return results

    prerequisites = """CREATE TABLE public.audit_audit_event (
      audit_event_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
      event_type text NOT NULL,
      resource_id uuid,
      tenant_id uuid,
      payload jsonb NOT NULL,
      occurred_at timestamptz NOT NULL
    );
    CREATE FUNCTION public.audit_audit_event() RETURNS trigger
      LANGUAGE plpgsql SECURITY DEFINER SET search_path=pg_catalog,public AS $$
      BEGIN
        INSERT INTO public.audit_audit_event(event_type,resource_id,tenant_id,payload,occurred_at)
        VALUES (TG_OP,NEW.id,NEW.tenant_id,to_jsonb(NEW),transaction_timestamp());
        RETURN NEW;
      END $$;
    CREATE FUNCTION public.scd_type2_close() RETURNS trigger
      LANGUAGE plpgsql AS $$ BEGIN RETURN NEW; END $$;
    -- The production Directory migration normally creates this referenced owner role.
    DO $w16_test_directory_role$
    BEGIN
      IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname='star_engineering_directory_owner') THEN
        CREATE ROLE star_engineering_directory_owner NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE
          NOREPLICATION NOBYPASSRLS NOINHERIT;
      END IF;
    END $w16_test_directory_role$;"""
    results.append(
        run_sql(
            "postgres-oauth-test-prerequisites",
            postgres_bin,
            port,
            environment,
            output_dir,
            sql=prerequisites,
        )
    )
    if results[-1]["exit_code"]:
        return results

    for name, migration in (
        ("postgres-oauth-base-migration", BASE_MIGRATION),
        ("postgres-w16-device-auth-migration", W16_MIGRATION),
    ):
        if not migration.is_file():
            results.append(
                {
                    "name": name,
                    "command": ["verify migration exists", str(migration)],
                    "exit_code": 2,
                    "error": "required migration file is missing",
                }
            )
            return results
        results.append(
            run_sql(name, postgres_bin, port, environment, output_dir, file=migration)
        )
        if results[-1]["exit_code"]:
            return results
    return results


def accepted_test_output(output: str) -> tuple[bool, list[str]]:
    """Require every named ignored PostgreSQL test and the exact clean libtest summary."""
    missing = [
        name
        for name in EXPECTED_PG_TESTS
        if re.search(rf"(?m)^test {re.escape(name)} \.\.\. ok$", output) is None
    ]
    expected_summary = (
        rf"(?m)^test result: ok\. {len(EXPECTED_PG_TESTS)} passed; 0 failed; 0 ignored;"
    )
    if re.search(expected_summary, output) is None:
        missing.append("clean test summary (all expected tests passed, none ignored)")
    return not missing, missing


def cargo_test_command(rtk: str, cargo: str) -> list[str]:
    """Build the one bounded OAuth test command through the repository RTK proxy."""
    return [
        rtk,
        "proxy",
        cargo,
        "+1.99.1",
        "test",
        "-j",
        "4",
        "--locked",
        "-p",
        "star-api-rest",
        "--test",
        PG_TEST_BINARY,
        "--",
        "--ignored",
        "--test-threads=1",
    ]


def run_postgres_tests(
    rtk: str,
    cargo: str,
    port: int,
    environment: Mapping[str, str],
    output_dir: Path,
) -> dict[str, object]:
    """Run the required ignored OAuth PostgreSQL tests on the private database."""
    test_environment = dict(environment)
    test_environment["STAR_W16_DEVICE_AUTH_TEST_DATABASE_URL"] = (
        f"postgres://postgres@127.0.0.1:{port}/{DATABASE_NAME}"
    )
    result = run_step(
        f"rust-{PG_TEST_BINARY}",
        cargo_test_command(rtk, cargo),
        test_environment,
        output_dir,
        timeout=3600,
    )
    combined = str(result.get("stdout_tail", "")) + "\n" + str(result.get("stderr_tail", ""))
    accepted, missing = accepted_test_output(combined)
    result["test_binary"] = PG_TEST_BINARY
    result["expected_tests"] = list(EXPECTED_PG_TESTS)
    result["required_tests_missing"] = missing
    result["acceptance_output_valid"] = accepted
    if result["exit_code"] == 0 and not accepted:
        result["exit_code"] = 1
        result["error"] = "Rust command exited successfully without proving every required ignored test passed"
    return result


def stop_cluster(
    postgres_bin: Path,
    data_dir: Path,
    environment: Mapping[str, str],
    output_dir: Path,
    start_attempted: bool,
) -> dict[str, object]:
    """Stop the invocation's private cluster while retaining its data and logs as evidence."""
    if not is_inside(data_dir, RUNNER_CACHE_ROOT):
        raise ValueError("refusing to manage PostgreSQL data outside this runner's cache")
    if start_attempted and data_dir.exists():
        stopped = run_step(
            "postgres-stop",
            [str(postgres_bin / "pg_ctl.exe"), "-D", str(data_dir), "-m", "fast", "-w", "stop"],
            environment,
            output_dir,
            timeout=180,
        )
        if stopped["exit_code"]:
            stopped["owned_data_dir_preserved"] = True
            stopped["owned_data_dir_removed"] = False
            return stopped
    else:
        stopped = {"name": "postgres-stop", "exit_code": 0, "reason": "cluster start was not attempted"}
    stopped["owned_data_dir_preserved"] = data_dir.exists()
    stopped["owned_data_dir_removed"] = False
    return stopped


def write_summary(summary_path: Path, summary: Mapping[str, object]) -> None:
    summary_path.parent.mkdir(parents=True, exist_ok=True)
    summary_path.write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--postgres-bin", type=Path, default=DEFAULT_POSTGRES_BIN)
    parser.add_argument("--cargo", default=DEFAULT_CARGO)
    parser.add_argument("--rtk", default=DEFAULT_RTK)
    options = parser.parse_args(argv)

    invocation_root = (RUNNER_CACHE_ROOT / "runs" / uuid.uuid4().hex).resolve()
    data_dir = invocation_root / "postgres-data"
    logs_dir = invocation_root / "logs"
    summary_path = invocation_root / "acceptance-summary.json"
    if not is_inside(invocation_root, RUNNER_CACHE_ROOT):
        raise RuntimeError("unique runner path escaped the private cache root")
    invocation_root.mkdir(parents=True, exist_ok=False)
    logs_dir.mkdir()
    environment = sanitized_environment()
    summary: dict[str, object] = {
        "phase": "W16-A OAuth PostgreSQL transaction acceptance",
        "status": "blocked",
        "runner_root": str(invocation_root.relative_to(ROOT)),
        "postgres_version": None,
        "test_binary": PG_TEST_BINARY,
        "expected_tests": list(EXPECTED_PG_TESTS),
        "steps": [],
        "database_scope": "fresh disposable PostgreSQL 18 cluster on IPv4 loopback",
        "database_role_note": "tests connect as the disposable cluster superuser; this run does not prove production role grants",
    }
    steps: list[dict[str, object]] = summary["steps"]  # type: ignore[assignment]
    start_attempted = False
    final_status = 1
    try:
        missing_tools = [name for name in REQUIRED_POSTGRES_TOOLS if not (options.postgres_bin / name).is_file()]
        if missing_tools:
            raise RuntimeError(f"PostgreSQL 18 tools are missing from --postgres-bin: {', '.join(missing_tools)}")
        cargo_path = Path(options.cargo)
        if not cargo_path.is_file() and shutil.which(options.cargo) is None:
            raise RuntimeError("Cargo executable was not found; provide --cargo or add cargo to PATH")
        rtk_path = Path(options.rtk)
        if not rtk_path.is_file() and shutil.which(options.rtk) is None:
            raise RuntimeError("RTK executable was not found; provide --rtk or add rtk to PATH")

        version = run_step(
            "postgres-version",
            [str(options.postgres_bin / "postgres.exe"), "--version"],
            environment,
            logs_dir,
            timeout=30,
        )
        steps.append(version)
        version_text = str(version.get("stdout_tail", "")) + str(version.get("stderr_tail", ""))
        version_match = re.search(r"PostgreSQL\)\s+(\d+)", version_text)
        summary["postgres_version"] = version_match.group(1) if version_match else None
        if version["exit_code"] or version_match is None or version_match.group(1) != "18":
            raise RuntimeError("W16-A acceptance requires PostgreSQL 18; version check did not confirm it")

        port = select_loopback_port()
        summary["loopback_port"] = port
        cluster_steps, start_attempted = start_cluster(
            options.postgres_bin, port, data_dir, environment, logs_dir
        )
        steps.extend(cluster_steps)
        if any(step["exit_code"] for step in cluster_steps):
            raise RuntimeError("disposable PostgreSQL cluster initialization/readiness failed")

        migration_steps = apply_migrations(options.postgres_bin, port, environment, logs_dir)
        steps.extend(migration_steps)
        if any(step["exit_code"] for step in migration_steps):
            raise RuntimeError("OAuth/W16 migration setup failed on the disposable database")

        test_result = run_postgres_tests(options.rtk, options.cargo, port, environment, logs_dir)
        steps.append(test_result)
        if test_result["exit_code"]:
            raise RuntimeError("one or more required real PostgreSQL integration tests failed or were not reported")
        summary["status"] = "accepted_local_postgresql_slice"
        final_status = 0
    except Exception as error:
        summary["failure"] = str(error)
        print(f"ACCEPTANCE_BLOCKED {error}", flush=True)
    except KeyboardInterrupt:
        summary["status"] = "interrupted"
        summary["failure"] = "received an interrupt before all selected PostgreSQL tests were proven"
        final_status = 130
        print("ACCEPTANCE_INTERRUPTED; preserving this invocation's logs and PostgreSQL data", flush=True)
    finally:
        cleanup = stop_cluster(
            options.postgres_bin,
            data_dir,
            environment,
            logs_dir,
            start_attempted,
        )
        steps.append(cleanup)
        if cleanup["exit_code"]:
            summary["status"] = "failed_cleanup"
            summary["failure"] = "could not stop the disposable PostgreSQL cluster cleanly"
            final_status = 1
        summary["completed_at_unix"] = time.time()
        summary["evidence_summary"] = str(summary_path.relative_to(ROOT))
        write_summary(summary_path, summary)
        print(f"SUMMARY {summary_path.relative_to(ROOT)}", flush=True)
        print(f"ACCEPTANCE_STATUS {summary['status']}", flush=True)
    return final_status


if __name__ == "__main__":
    raise SystemExit(main())
