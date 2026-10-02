"""Run Schedule 9F3 Rust checks and isolated-PostgreSQL invariants."""

# @cypher schema=1 source_sha256=93ab5567fc9d26141edc8fb93e442f2e5377c150f08f6b52bdac4202f1d1fe20
# MERGE (self:File {path:"scripts/automation/phase9f3_schedule.py"})
# MERGE (main:Symbol {id:"scripts/automation/phase9f3_schedule.py::main",kind:"function"})
# MERGE (run_step:Symbol {id:"scripts/automation/phase9f3_schedule.py::run_step",kind:"function"})
# MERGE (postgres:Symbol {id:"scripts/automation/phase9f3_schedule.py::run_isolated_postgres",kind:"function"})
# MERGE (cluster:Symbol {id:"scripts/automation/phase9f3_schedule.py::DisposablePostgres",kind:"class"})
# MERGE (scope:Symbol {id:"scripts/automation/phase9f3_schedule.py::is_contained",kind:"function"})
# MERGE (repo:ExternalService {id:"git.repository",kind:"directory"})
# MERGE (postgres_service:ExternalService {id:"postgres.initdb_pg_ctl_psql",kind:"command"})
# MERGE (cargo:ExternalService {id:"cargo.test_clippy_check",kind:"command"})
# MERGE (subprocess:ExternalService {id:"python.subprocess.run",kind:"function"})
# MERGE (self)-[:DEFINES]->(main)
# MERGE (self)-[:DEFINES]->(run_step)
# MERGE (self)-[:DEFINES]->(postgres)
# MERGE (self)-[:DEFINES]->(cluster)
# MERGE (self)-[:DEFINES]->(scope)
# MERGE (main)-[:CALLS]->(run_step)
# MERGE (main)-[:CALLS]->(postgres)
# MERGE (run_step)-[:CALLS]->(subprocess)
# MERGE (postgres)-[:CALLS]->(run_step)
# MERGE (postgres)-[:CONFIGURES]->(postgres_service)
# MERGE (main)-[:USES]->(repo)
# MERGE (postgres)-[:CALLS]->(scope)
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

ROOT = Path(__file__).resolve().parents[2]
OUTPUT_DIR = ROOT / ".cache" / "phase9f3-schedule"
MAX_STEP_SECONDS = 900
MIGRATION = ROOT / "db" / "migrations" / "2026-10-02-automation-schedule-occurrence.sql"
RUST_FILES = (
    "crates/domain-automation/src/lib.rs",
    "crates/domain-automation/src/schedule.rs",
    "crates/star-pg-adapter/src/repository/mod.rs",
    "crates/star-pg-adapter/src/repository/automation_schedule.rs",
    "crates/star-pg-adapter/tests/schedule_postgres.rs",
)


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
GRANT USAGE ON SCHEMA automation TO schedule_runtime;
GRANT SELECT ON automation.schedule_rule_revision, automation.schedule_rule_audit,
  automation.occurrence, automation.occurrence_dispatch, automation.occurrence_event
  TO schedule_runtime;
GRANT INSERT ON automation.schedule_rule_audit, automation.occurrence,
  automation.occurrence_dispatch, automation.occurrence_event TO schedule_runtime;
GRANT UPDATE, DELETE ON automation.occurrence_dispatch TO schedule_runtime;"""
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


def run_isolated_postgres(bin_dir: Path) -> list[dict[str, object]]:
    """Apply the Schedule migration twice and run real role/RLS/concurrency adapter tests."""
    cluster = DisposablePostgres(bin_dir)
    steps: list[dict[str, object]] = []
    try:
        steps.extend(cluster.create_and_start())
        if not cluster.started:
            return steps
        for name in ("postgres-schedule-migration-first", "postgres-schedule-migration-repeat"):
            steps.append(
                run_step(
                    name,
                    [*cluster.psql("schedule_admin"), "-f", str(MIGRATION)],
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
                      'occurrence_dispatch','occurrence_event')
    AND c.relrowsecurity AND c.relforcerowsecurity;
  IF forced_count <> 5 THEN RAISE EXCEPTION 'expected five FORCE RLS Schedule tables'; END IF;
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

        runtime_env = os.environ.copy()
        runtime_env["STAR_SCHEDULE_ADMIN_DATABASE_URL"] = (
            f"postgresql://schedule_admin@127.0.0.1:{cluster.port}/postgres"
        )
        runtime_env["STAR_SCHEDULE_TEST_DATABASE_URL"] = (
            f"postgresql://schedule_runtime@127.0.0.1:{cluster.port}/postgres"
        )
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
        description="Run real Schedule 9F3 Rust and isolated PostgreSQL gates."
    )
    parser.add_argument("--cargo", required=True, help="Cargo executable path")
    parser.add_argument("--rustfmt", required=True, help="rustfmt executable path")
    parser.add_argument("--postgres-bin-dir", required=True, help="directory containing initdb/pg_ctl/psql")
    args = parser.parse_args()

    cargo = Path(args.cargo).resolve()
    rustfmt = Path(args.rustfmt).resolve()
    postgres_bin = Path(args.postgres_bin_dir).resolve()
    results: list[dict[str, object]] = []
    if not cargo.is_file() or not rustfmt.is_file() or not MIGRATION.is_file():
        print(json.dumps({"status": "blocked", "reason": "required Rust tool or Schedule migration is unavailable"}, indent=2))
        return 2

    rustfmt_command = [str(rustfmt), "--check", "--edition", "2024", *RUST_FILES]
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
        for name, command in cargo_commands:
            results.append(run_step(name, command))
            if results[-1]["status"] != "passed":
                break
    if results and all(result["status"] == "passed" for result in results):
        if not all((postgres_bin / f"{name}{'.exe' if os.name == 'nt' else ''}").is_file() for name in ("initdb", "pg_ctl", "psql")):
            results.append({"name": "postgres-tools", "status": "blocked", "reason": "initdb/pg_ctl/psql not found in supplied directory"})
        else:
            results.extend(run_isolated_postgres(postgres_bin))

    print(json.dumps({"status": "passed" if results and all(result["status"] == "passed" for result in results) else "failed", "results": results}, indent=2, ensure_ascii=False))
    return 0 if results and all(result["status"] == "passed" for result in results) else 1


if __name__ == "__main__":
    raise SystemExit(main())
