"""
CYPHER STRUCTURE MANIFEST
CREATE (f:File {name:"engineering_run_directory.py",type:"file",language:"python"}),
 (cargo:Function {name:"find_cargo",type:"function"}), (run:Function {name:"run_step",type:"function"}),
 (main:Function {name:"main",type:"function"}), (root:Variable {name:"ROOT",type:"variable"}),
 (ddl:Function {name:"validate_postgres_ddl",type:"function"}), (uuid:Function {name:"uuid4",type:"function"}),
 (remove:Function {name:"shutil.rmtree",type:"function"}), (relative:Function {name:"Path.is_relative_to",type:"function"}),
 (args:Class {name:"argparse.ArgumentParser",type:"class"}), (process:Function {name:"subprocess.run",type:"function"}),
 (which:Function {name:"shutil.which",type:"function"}), (home:Function {name:"Path.home",type:"function"}),
 (loads:Function {name:"tomllib.loads",type:"function"}), (read:Function {name:"Path.read_text",type:"function"}),
 (bytes:Function {name:"Path.read_bytes",type:"function"}), (write:Function {name:"Path.write_bytes",type:"function"}),
 (exists:Function {name:"Path.is_file",type:"function"}), (text:Function {name:"Path.write_text",type:"function"}),
 (mkdir:Function {name:"Path.mkdir",type:"function"}), (parse:Function {name:"ArgumentParser.parse_args",type:"function"}),
 (add:Function {name:"ArgumentParser.add_argument",type:"function"}), (dump:Function {name:"json.dumps",type:"function"}),
 (f)-[:CONTAINS]->(cargo),(f)-[:CONTAINS]->(run),(f)-[:CONTAINS]->(main),(f)-[:CONTAINS]->(root),
 (cargo)-[:CALLS]->(which),(cargo)-[:CALLS]->(home),(cargo)-[:CALLS]->(loads),(cargo)-[:CALLS]->(read),(cargo)-[:CALLS]->(exists),
 (run)-[:CALLS]->(process),(run)-[:CALLS]->(mkdir),(run)-[:CALLS]->(text),
 (main)-[:CALLS]->(cargo),(main)-[:CALLS]->(run),(main)-[:CALLS]->(bytes),(main)-[:CALLS]->(write),(main)-[:CALLS]->(exists),
 (main)-[:CALLS]->(parse),(main)-[:CALLS]->(add),(main)-[:CALLS]->(dump),(main)-[:CALLS]->(text),
 (f)-[:CONTAINS]->(ddl),(main)-[:CALLS]->(ddl),(ddl)-[:CALLS]->(run),(ddl)-[:CALLS]->(uuid),(ddl)-[:CALLS]->(remove),(ddl)-[:CALLS]->(relative),
 (cargo)-[:USES]->(root),(run)-[:USES]->(root),(main)-[:USES]->(root);

Compile the canonical Branch/Engineering Run directory slice. No tests, production migrations,
credential inspection, remote writes or workspace-wide source formatting are run.
Temporary Cargo resolution is opt-in and the original lock bytes are restored.
See docs/automation-design.md section 4.36.
"""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
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


def validate_postgres_ddl(bin_dir: Path, port: int, environment: dict[str, str], output_dir: Path) -> list[dict]:
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
        for name, migration in [
            ("postgres-worktree-base", "2026-09-16-worktree-canvas-worktree.sql"),
            ("postgres-project-binding", "2026-09-29-worktree-group-phase-2b.sql"),
            ("postgres-directory-first", "2026-10-01-engineering-run-directory.sql"),
            ("postgres-directory-repeat", "2026-10-01-engineering-run-directory.sql"),
        ]:
            steps.append(run_step(name, psql + ["-f", str(ROOT / "db" / "migrations" / migration)], ROOT, environment, output_dir))
            if steps[-1]["exit_code"]:
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
    parser.add_argument("--postgres-bin", type=Path, default=Path("D:/PostgreSQL/18/bin"))
    parser.add_argument("--postgres-port", type=int, default=57432)
    parser.add_argument("--allow-lock-resolution", action="store_true",
                        help="Opt in to transient workspace dependency resolution; restores Cargo.lock")
    parser.add_argument("--output-dir", type=Path, default=ROOT / ".cache" / "engineering-run-directory")
    options = parser.parse_args()
    if not (options.rust or options.frontend or options.postgres_ddl):
        parser.error("select --rust, --frontend and/or --postgres-ddl")
    if not 1024 <= options.postgres_port <= 65535:
        parser.error("invalid disposable PostgreSQL port")
    if options.allow_lock_resolution and not options.rust:
        parser.error("--allow-lock-resolution requires --rust")
    environment = os.environ.copy()
    results = []
    if options.rust:
        cargo = find_cargo()
        environment["PATH"] = str(cargo.parent) + os.pathsep + environment.get("PATH", "")
        lock_path = ROOT / "Cargo.lock"
        original_lock = lock_path.read_bytes()
        command = [str(cargo), "check", "-p", "star-api-rest", "--all-targets", "-j", "4"]
        if not options.allow_lock_resolution:
            command.append("--locked")
        try:
            results.append(run_step("rust-check", command, ROOT, environment, options.output_dir))
        finally:
            if options.allow_lock_resolution and lock_path.read_bytes() != original_lock:
                lock_path.write_bytes(original_lock)
        results[-1]["temporary_lock_resolution"] = options.allow_lock_resolution
    if options.frontend:
        npm = shutil.which("npm.cmd" if os.name == "nt" else "npm")
        if not npm:
            raise FileNotFoundError("npm executable is unavailable")
        results.append(run_step("frontend-typecheck", [npm, "run", "typecheck"], ROOT / "frontend",
                                environment, options.output_dir))
    if options.postgres_ddl:
        results.extend(validate_postgres_ddl(options.postgres_bin, options.postgres_port, environment, options.output_dir))
    summary_path = options.output_dir / "summary.json"
    summary_path.write_text(json.dumps({"gates": results, "tests_run": False, "production_migrations_run": False},
                                      ensure_ascii=False, indent=2), encoding="utf-8")
    return 0 if all(row["exit_code"] == 0 for row in results) else 1


if __name__ == "__main__":
    raise SystemExit(main())
