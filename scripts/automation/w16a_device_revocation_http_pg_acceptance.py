"""Run authenticated device-revocation HTTP acceptance on disposable PostgreSQL 18."""

# @cypher schema=1 source_sha256=31ac4ff986af7020cc76891deea60cbfa26d1716e280a07b52e6410744a46ce4
# MERGE (self:File {path:"scripts/automation/w16a_device_revocation_http_pg_acceptance.py"})
# MERGE (main:Symbol {id:"scripts/automation/w16a_device_revocation_http_pg_acceptance.py::main",kind:"function"})
# MERGE (sanitize:Symbol {id:"scripts/automation/w16a_device_revocation_http_pg_acceptance.py::sanitized_environment",kind:"function"})
# MERGE (port:Symbol {id:"scripts/automation/w16a_device_revocation_http_pg_acceptance.py::select_loopback_port",kind:"function"})
# MERGE (inside:Symbol {id:"scripts/automation/w16a_device_revocation_http_pg_acceptance.py::is_inside",kind:"function"})
# MERGE (step:Symbol {id:"scripts/automation/w16a_device_revocation_http_pg_acceptance.py::run_step",kind:"function"})
# MERGE (psql:Symbol {id:"scripts/automation/w16a_device_revocation_http_pg_acceptance.py::psql_command",kind:"function"})
# MERGE (migrate:Symbol {id:"scripts/automation/w16a_device_revocation_http_pg_acceptance.py::apply_migrations",kind:"function"})
# MERGE (parseToolchain:Symbol {id:"scripts/automation/w16a_device_revocation_http_pg_acceptance.py::parse_stable_toolchain",kind:"function"})
# MERGE (selectToolchain:Symbol {id:"scripts/automation/w16a_device_revocation_http_pg_acceptance.py::selected_toolchain",kind:"function"})
# MERGE (componentProbe:Symbol {id:"scripts/automation/w16a_device_revocation_http_pg_acceptance.py::toolchain_component_probe",kind:"function"})
# MERGE (rustProbe:Symbol {id:"scripts/automation/w16a_device_revocation_http_pg_acceptance.py::rust_toolchain_probe",kind:"function"})
# MERGE (cargoProbe:Symbol {id:"scripts/automation/w16a_device_revocation_http_pg_acceptance.py::cargo_toolchain_probe",kind:"function"})
# MERGE (prereqs:Symbol {id:"scripts/automation/w16a_device_revocation_http_pg_acceptance.py::required_tools_missing",kind:"function"})
# MERGE (output:Symbol {id:"scripts/automation/w16a_device_revocation_http_pg_acceptance.py::accepted_test_output",kind:"function"})
# MERGE (cargo:Symbol {id:"scripts/automation/w16a_device_revocation_http_pg_acceptance.py::cargo_test_command",kind:"function"})
# MERGE (stop:Symbol {id:"scripts/automation/w16a_device_revocation_http_pg_acceptance.py::stop_cluster",kind:"function"})
# MERGE (tail:Type {id:"scripts/automation/w16a_device_revocation_http_pg_acceptance.py::BoundedTail",kind:"class"})
# MERGE (capture:Symbol {id:"scripts/automation/w16a_device_revocation_http_pg_acceptance.py::capture_pipe",kind:"function"})
# MERGE (terminate:Symbol {id:"scripts/automation/w16a_device_revocation_http_pg_acceptance.py::terminate_process_tree",kind:"function"})
# MERGE (testBinary:File {path:"crates/star-api-rest/tests/device_revocation_http_pg.rs"})
# MERGE (db:Database {id:"PostgreSQL 18 disposable loopback cluster",kind:"database_server"})
# MERGE (self)-[:DEFINES]->(main),(self)-[:DEFINES]->(sanitize),(self)-[:DEFINES]->(port),(self)-[:DEFINES]->(inside),(self)-[:DEFINES]->(step),(self)-[:DEFINES]->(psql),(self)-[:DEFINES]->(migrate),(self)-[:DEFINES]->(parseToolchain),(self)-[:DEFINES]->(selectToolchain),(self)-[:DEFINES]->(componentProbe),(self)-[:DEFINES]->(rustProbe),(self)-[:DEFINES]->(cargoProbe),(self)-[:DEFINES]->(prereqs),(self)-[:DEFINES]->(output),(self)-[:DEFINES]->(cargo),(self)-[:DEFINES]->(stop),(self)-[:DEFINES]->(tail),(self)-[:DEFINES]->(capture),(self)-[:DEFINES]->(terminate)
# MERGE (main)-[:CALLS]->(sanitize),(main)-[:CALLS]->(port),(main)-[:CALLS]->(migrate),(main)-[:CALLS]->(step),(main)-[:CALLS]->(stop),(main)-[:CALLS]->(prereqs)
# MERGE (step)-[:CALLS]->(capture),(capture)-[:USES]->(tail),(step)-[:CALLS]->(terminate)
# MERGE (prereqs)-[:CALLS]->(rustProbe),(prereqs)-[:CALLS]->(cargoProbe)
# MERGE (rustProbe)-[:CALLS]->(componentProbe),(cargoProbe)-[:CALLS]->(componentProbe)
# MERGE (componentProbe)-[:CALLS]->(selectToolchain),(componentProbe)-[:CALLS]->(parseToolchain)
# MERGE (componentProbe)-[:CONFIGURES]->(:Config {id:"Requested stable Rust toolchain >=1.99.1 selected through RTK proxy; optional stable alias must resolve to exact request",kind:"toolchain"})
# MERGE (step)-[:CALLS]->(inside)
# MERGE (migrate)-[:CALLS]->(psql),(migrate)-[:CALLS]->(step)
# MERGE (psql)-[:CONFIGURES]->(db)
# MERGE (output)-[:TESTS]->(testBinary)
# MERGE (cargo)-[:TESTS]->(testBinary)
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
import threading
import time
import uuid
from pathlib import Path
from typing import BinaryIO, Mapping

ROOT = Path(__file__).resolve().parents[2]
CACHE = ROOT / ".cache" / "w16-device-revocation-http-pg"
PG_BIN_DEFAULT = Path("D:/PostgreSQL/18/bin")
DB = "w16a_device_revoke_http_acceptance"
APP_ROLE = "star_w16_device_revoke_app"
TEST_BIN = "device_revocation_http_pg"
MIN_RUST_TOOLCHAIN = (1, 99, 0)
DEFAULT_RUST_TOOLCHAIN = "1.99.1"
CARGO_DEFAULT = shutil.which("cargo") or "cargo"
RTK_DEFAULT = shutil.which("rtk") or "rtk"
BASE_MIGRATION = ROOT / "db/migrations/2026-09-09-oauth2-server.sql"
W16_MIGRATION = ROOT / "db/migrations/2026-10-28-w16-device-auth.sql"
TEST_FILE = ROOT / f"crates/star-api-rest/tests/{TEST_BIN}.rs"
PG_TOOLS = ("initdb.exe", "pg_ctl.exe", "postgres.exe", "psql.exe")
MAX_TAIL = 64 * 1024
MAX_LOG_BYTES = 1024 * 1024
EXPECTED_TESTS = (
    "unauthenticated_revoke_is_rejected",
    "authenticated_revoke_is_owner_scoped_and_audited",
    "concurrent_revoke_has_one_winner_and_one_audit_fact",
    "audit_failure_rolls_back_device_and_refresh_revocation",
)


def sanitized_environment(source: Mapping[str, str] | None = None) -> dict[str, str]:
    """Strip inherited PostgreSQL targets; this run sets only its own loopback URLs."""
    env = dict(os.environ if source is None else source)
    for key in tuple(env):
        upper = key.upper()
        if upper.startswith("PG") or upper in {
            "DATABASE_URL",
            "STAR_DATABASE_URL",
            "STAR_W16_DEVICE_AUTH_TEST_DATABASE_URL",
            "STAR_W16_DEVICE_AUTH_ADMIN_TEST_DATABASE_URL",
            "SQLX_OFFLINE",
        }:
            env.pop(key, None)
    env["CARGO_TERM_COLOR"] = "never"
    return env


def select_loopback_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as listener:
        listener.bind(("127.0.0.1", 0))
        return int(listener.getsockname()[1])


def database_urls(port: int) -> tuple[str, str]:
    authority = f"127.0.0.1:{port}/{DB}"
    return f"postgres://postgres@{authority}", f"postgres://{APP_ROLE}@{authority}"


def is_inside(path: Path, root: Path) -> bool:
    return path.resolve() != root.resolve() and path.resolve().is_relative_to(root.resolve())


def read_tail(path: Path, limit: int = MAX_TAIL) -> str:
    if not path.is_file():
        return ""
    with path.open("rb") as file:
        file.seek(0, os.SEEK_END)
        file.seek(max(0, file.tell() - limit))
        return file.read(limit).decode("utf-8", errors="replace")


class BoundedTail:
    """Keep only the newest `limit` bytes while a child process is running."""

    def __init__(self, limit: int = MAX_LOG_BYTES) -> None:
        if limit < 1:
            raise ValueError("tail limit must be positive")
        self.limit = limit
        self._buffer = bytearray()
        self._lock = threading.Lock()

    def append(self, chunk: bytes) -> None:
        with self._lock:
            if len(chunk) >= self.limit:
                self._buffer[:] = chunk[-self.limit :]
                return
            self._buffer.extend(chunk)
            overflow = len(self._buffer) - self.limit
            if overflow > 0:
                del self._buffer[:overflow]

    def value(self) -> bytes:
        with self._lock:
            return bytes(self._buffer)


def capture_pipe(stream: BinaryIO, tail: BoundedTail) -> None:
    """Drain a child pipe continuously while retaining only its bounded tail."""
    try:
        while chunk := stream.read(64 * 1024):
            tail.append(chunk)
    finally:
        stream.close()


def terminate_process_tree(process: subprocess.Popen[bytes]) -> None:
    """Stop the owned Windows process tree, then ensure the direct child is reaped."""
    if os.name == "nt" and process.poll() is None:
        try:
            subprocess.run(
                ["taskkill.exe", "/PID", str(process.pid), "/T", "/F"],
                stdin=subprocess.DEVNULL,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                timeout=15,
                check=False,
            )
        except (FileNotFoundError, subprocess.TimeoutExpired):
            pass
    if process.poll() is None:
        process.kill()
    process.wait()


def run_step(
    name: str,
    command: list[str],
    env: Mapping[str, str],
    logs: Path,
    timeout: int,
    *,
    capture_output: bool = True,
) -> dict[str, object]:
    logs.mkdir(parents=True, exist_ok=True)
    stdout_path, stderr_path = logs / f"{name}.stdout.log", logs / f"{name}.stderr.log"
    print(f"COMMAND[{name}] {subprocess.list2cmdline(command)}", flush=True)
    stdout_tail, stderr_tail = BoundedTail(), BoundedTail()
    readers: list[threading.Thread] = []
    process: subprocess.Popen[bytes] | None = None
    capture_error: str | None = None
    try:
        if capture_output:
            process = subprocess.Popen(command, cwd=ROOT, env=dict(env), stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            assert process.stdout is not None and process.stderr is not None
            readers = [
                threading.Thread(target=capture_pipe, args=(process.stdout, stdout_tail), daemon=True),
                threading.Thread(target=capture_pipe, args=(process.stderr, stderr_tail), daemon=True),
            ]
            for reader in readers:
                reader.start()
        else:
            process = subprocess.Popen(command, cwd=ROOT, env=dict(env), stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        try:
            code = process.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            terminate_process_tree(process)
            code = 124
    except FileNotFoundError as error:
        stderr_tail.append(str(error).encode("utf-8", errors="replace"))
        code = 127
    finally:
        for reader in readers:
            reader.join(timeout=10)
        if any(reader.is_alive() for reader in readers) and process is not None:
            terminate_process_tree(process)
            for reader in readers:
                reader.join(timeout=10)
        if any(reader.is_alive() for reader in readers):
            capture_error = "child output pipe did not close after process-tree termination"
            code = 125
        stdout_path.write_bytes(stdout_tail.value())
        stderr_path.write_bytes(stderr_tail.value())
    result: dict[str, object] = {
        "name": name,
        "command": command,
        "exit_code": code,
        "stdout_log": stdout_path.relative_to(ROOT).as_posix(),
        "stderr_log": stderr_path.relative_to(ROOT).as_posix(),
        "stdout_tail": read_tail(stdout_path),
        "stderr_tail": read_tail(stderr_path),
    }
    if capture_error:
        result["capture_error"] = capture_error
    if code:
        print(f"EXIT[{name}] {code}\n{result['stderr_tail']}", flush=True)
    return result


def psql(postgres_bin: Path, port: int, database: str, *, sql: str | None = None, file: Path | None = None) -> list[str]:
    command = [str(postgres_bin / "psql.exe"), "-X", "-h", "127.0.0.1", "-p", str(port), "-U", "postgres", "-d", database, "-v", "ON_ERROR_STOP=1"]
    if sql is not None:
        command.extend(["-c", sql])
    elif file is not None:
        command.extend(["-f", str(file)])
    else:
        raise ValueError("psql requires SQL or file")
    return command


def parse_stable_toolchain(value: str) -> tuple[int, int, int] | None:
    """Parse stable X.Y.Z toolchain names; aliases and prereleases are not accepted."""
    if re.fullmatch(r"(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)", value) is None:
        return None
    major, minor, patch = value.split(".")
    return int(major), int(minor), int(patch)


def selected_toolchain(requested: str, selector: str | None) -> tuple[str | None, str | None]:
    requested_version = parse_stable_toolchain(requested)
    if requested_version is None:
        return None, "Rust toolchain must be a stable X.Y.Z version >= 1.99.1"
    if requested_version < MIN_RUST_TOOLCHAIN:
        return None, f"Rust toolchain {requested} is below minimum 1.99.1"
    selected = selector or requested
    if selected not in {requested, "stable"}:
        return None, "Rust toolchain selector must be the requested version or the explicit 'stable' alias"
    return selected, None


def toolchain_component_probe(
    rtk: str,
    executable: str,
    component: str,
    requested_toolchain: str,
    selector: str | None,
) -> dict[str, object]:
    """Resolve a compiler/tool through RTK and require its version to match the request."""
    requested = parse_stable_toolchain(requested_toolchain)
    selected, selection_error = selected_toolchain(requested_toolchain, selector)
    info: dict[str, object] = {
        "requested": requested_toolchain,
        "selector": selected if selected is not None else (selector or requested_toolchain),
        "version": None,
        "probe_exit_code": None,
        "available": False,
        "error": selection_error,
    }
    if selection_error:
        return info
    if shutil.which(executable) is None:
        info["error"] = f"{component} executable not found on PATH: {executable}"
        return info
    assert selected is not None and requested is not None
    try:
        completed = subprocess.run(
            [rtk, "proxy", executable, f"+{selected}", "-V"],
            cwd=ROOT,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            timeout=30,
            check=False,
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        info["error"] = f"Rust {requested_toolchain} {component} preflight could not run: {type(error).__name__}: {error}"
        return info
    info["probe_exit_code"] = completed.returncode
    output_lines = ((completed.stdout or "") + "\n" + (completed.stderr or "")).strip().splitlines()
    diagnostic_lines = [line.strip() for line in output_lines if line.strip() and not line.lstrip().startswith("[rtk]")]
    version_line = next((line for line in diagnostic_lines if line.startswith(f"{component} ")), None)
    if completed.returncode != 0:
        detail = diagnostic_lines[-1][:300] if diagnostic_lines else "no diagnostic output"
        info["error"] = f"Rust {requested_toolchain} {component} unavailable (exit {completed.returncode}): {detail}"
        return info
    if version_line is None:
        info["error"] = f"Rust {requested_toolchain} {component} preflight returned no {component} version"
        return info
    info["version"] = version_line
    reported = version_line.split()[1]
    if parse_stable_toolchain(reported) != requested:
        info["error"] = f"requested Rust {requested_toolchain}, but {component} reported {reported}"
        return info
    info["available"] = True
    return info


def rust_toolchain_probe(rtk: str, rustc: str, toolchain: str, selector: str | None = None) -> dict[str, object]:
    result = toolchain_component_probe(rtk, rustc, "rustc", toolchain, selector)
    result["rustc_version"] = result.pop("version")
    return result


def cargo_toolchain_probe(rtk: str, cargo: str, toolchain: str, selector: str | None = None) -> dict[str, object]:
    result = toolchain_component_probe(rtk, cargo, "cargo", toolchain, selector)
    result["cargo_version"] = result.pop("version")
    return result


def required_tools_missing(
    pg_bin: Path,
    cargo: str,
    rtk: str,
    rust_toolchain: str = DEFAULT_RUST_TOOLCHAIN,
    rust_toolchain_selector: str | None = None,
    toolchain_info: dict[str, object] | None = None,
) -> list[str]:
    missing = [str(pg_bin / name) for name in PG_TOOLS if not (pg_bin / name).is_file()]
    missing.extend(name for name in (cargo, rtk) if shutil.which(name) is None)
    missing.extend(str(path) for path in (BASE_MIGRATION, W16_MIGRATION, TEST_FILE) if not path.is_file())
    rustc = shutil.which("rustc")
    if not shutil.which(rtk):
        info = {"requested": rust_toolchain, "selector": rust_toolchain_selector or rust_toolchain, "rustc_version": None, "cargo_version": None, "probe_exit_code": None, "available": False, "error": "RTK is unavailable; Rust toolchain preflight was not run"}
    else:
        rust_info = rust_toolchain_probe(rtk, rustc or "rustc", rust_toolchain, rust_toolchain_selector)
        cargo_info = cargo_toolchain_probe(rtk, shutil.which(cargo) or cargo, rust_toolchain, rust_toolchain_selector)
        info = {
            "requested": rust_toolchain,
            "selector": rust_info.get("selector"),
            "rustc_version": rust_info.get("rustc_version"),
            "rustc_probe_exit_code": rust_info.get("probe_exit_code"),
            "cargo_version": cargo_info.get("cargo_version"),
            "cargo_probe_exit_code": cargo_info.get("probe_exit_code"),
            "available": bool(rust_info.get("available")) and bool(cargo_info.get("available")),
            "error": "; ".join(str(item) for item in (rust_info.get("error"), cargo_info.get("error")) if item),
        }
    if toolchain_info is not None:
        toolchain_info.update(info)
    if info.get("error"):
        missing.append(str(info["error"]))
    return missing


def apply_migrations(pg_bin: Path, port: int, env: Mapping[str, str], logs: Path) -> list[dict[str, object]]:
    results = [run_step("postgres-create-database", psql(pg_bin, port, "postgres", sql=f"CREATE DATABASE {DB};"), env, logs, 60)]
    if results[-1]["exit_code"]:
        return results
    prerequisites = """
CREATE TABLE public.audit_audit_event (
  audit_event_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  event_type text NOT NULL, resource_id uuid, tenant_id uuid,
  payload jsonb NOT NULL, occurred_at timestamptz NOT NULL
);
CREATE FUNCTION public.audit_audit_event() RETURNS trigger
  LANGUAGE plpgsql SECURITY DEFINER SET search_path=pg_catalog,public AS $function$
  BEGIN
    INSERT INTO public.audit_audit_event(event_type,resource_id,tenant_id,payload,occurred_at)
    VALUES (TG_OP,NEW.id,NEW.tenant_id,to_jsonb(NEW),transaction_timestamp());
    RETURN NEW;
  END $function$;
CREATE FUNCTION public.scd_type2_close() RETURNS trigger
  LANGUAGE plpgsql AS $function$ BEGIN RETURN NEW; END $function$;
DO $directory_owner$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname='star_engineering_directory_owner') THEN
    CREATE ROLE star_engineering_directory_owner NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE
      NOREPLICATION NOBYPASSRLS NOINHERIT;
  END IF;
END $directory_owner$;
"""
    results.append(run_step("postgres-test-prerequisites", psql(pg_bin, port, DB, sql=prerequisites), env, logs, 60))
    if results[-1]["exit_code"]:
        return results
    for name, migration in (("postgres-oauth-base-migration", BASE_MIGRATION), ("postgres-w16-device-migration", W16_MIGRATION)):
        results.append(run_step(name, psql(pg_bin, port, DB, file=migration), env, logs, 180))
        if results[-1]["exit_code"]:
            return results
    roles = f"""
DO $app_login$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname='{APP_ROLE}') THEN
    CREATE ROLE {APP_ROLE} LOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE
      NOREPLICATION NOBYPASSRLS NOINHERIT;
  END IF;
END $app_login$;
ALTER ROLE {APP_ROLE} LOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE
  NOREPLICATION NOBYPASSRLS NOINHERIT;
REVOKE CONNECT ON DATABASE {DB} FROM PUBLIC;
GRANT CONNECT ON DATABASE {DB} TO postgres,{APP_ROLE};
GRANT star_oauth_credential_owner TO {APP_ROLE};
"""
    results.append(run_step("postgres-constrained-app-role", psql(pg_bin, port, DB, sql=roles), env, logs, 60))
    if results[-1]["exit_code"]:
        return results
    failure_trigger = """
CREATE OR REPLACE FUNCTION public.w16_test_reject_device_revoke_audit()
RETURNS trigger LANGUAGE plpgsql AS $w16_test_failure$
BEGIN
  IF NEW.event_kind='revoked' AND NEW.public_key_fingerprint=repeat('f',64) THEN
    RAISE EXCEPTION 'test-only injected device audit failure';
  END IF;
  RETURN NEW;
END $w16_test_failure$;
CREATE TRIGGER w16_test_reject_device_revoke_audit
BEFORE INSERT ON public.oauth_device_binding_audit
FOR EACH ROW EXECUTE FUNCTION public.w16_test_reject_device_revoke_audit();
"""
    results.append(run_step("postgres-audit-rollback-fixture", psql(pg_bin, port, DB, sql=failure_trigger), env, logs, 60))
    return results


def accepted_test_output(output: str) -> tuple[bool, list[str]]:
    missing = [name for name in EXPECTED_TESTS if re.search(rf"(?m)^test {re.escape(name)} \.\.\. ok$", output) is None]
    summary = rf"(?m)^test result: ok\. {len(EXPECTED_TESTS)} passed; 0 failed; 0 ignored;"
    if re.search(summary, output) is None:
        missing.append("clean result summary (all expected tests passed, zero ignored)")
    return not missing, missing


def cargo_test_command(
    rtk: str = RTK_DEFAULT,
    cargo: str = CARGO_DEFAULT,
    rust_toolchain: str = DEFAULT_RUST_TOOLCHAIN,
    rust_toolchain_selector: str | None = None,
) -> list[str]:
    selected = rust_toolchain_selector or rust_toolchain
    return [rtk, "proxy", cargo, f"+{selected}", "test", "-j", "4", "--locked", "-p", "star-api-rest", "--test", TEST_BIN, "--", "--ignored", "--test-threads=1"]


def run_postgres_tests(rtk: str, cargo: str, rust_toolchain: str, rust_toolchain_selector: str | None, port: int, env: Mapping[str, str], logs: Path) -> dict[str, object]:
    test_env = dict(env)
    test_env["STAR_W16_DEVICE_AUTH_ADMIN_TEST_DATABASE_URL"], test_env["STAR_W16_DEVICE_AUTH_TEST_DATABASE_URL"] = database_urls(port)
    result = run_step(f"rust-{TEST_BIN}", cargo_test_command(rtk, cargo, rust_toolchain, rust_toolchain_selector), test_env, logs, 3600)
    accepted, missing = accepted_test_output(str(result["stdout_tail"]) + "\n" + str(result["stderr_tail"]))
    result["expected_tests"] = list(EXPECTED_TESTS)
    result["acceptance_output_valid"] = accepted
    result["required_tests_missing"] = missing
    if result["exit_code"] == 0 and not accepted:
        result["exit_code"] = 1
        result["error"] = "test output did not prove every required case passed"
    return result


def stop_cluster(pg_bin: Path, data: Path, env: Mapping[str, str], logs: Path, attempted: bool) -> dict[str, object]:
    if not is_inside(data, CACHE):
        raise ValueError("refusing to manage PostgreSQL outside runner cache")
    if not data.exists():
        return {"exit_code": 0, "status": "already-absent"}
    if not attempted:
        result: dict[str, object] = {"exit_code": 0, "status": "not-started"}
        shutil.rmtree(data)
        result["data_removed"] = True
        return result
    status = run_step("postgres-status", [str(pg_bin / "pg_ctl.exe"), "-D", str(data), "status"], env, logs, 30)
    status_code = int(status.get("exit_code", 1))
    stop_result: dict[str, object] = {"status_check_exit_code": status_code}
    server_log = data / "postgres-server.log"
    if status_code == 3:
        if server_log.is_file():
            shutil.copyfile(server_log, logs / "postgres-server.log")
        shutil.rmtree(data)
        stop_result.update({"status": "not-running", "exit_code": 0, "data_removed": True})
        return stop_result
    if status_code != 0:
        if server_log.is_file():
            shutil.copyfile(server_log, logs / "postgres-server.log")
        stop_result.update({
            "status": "unknown",
            "exit_code": status_code,
            "data_retained": True,
            "reason": "pg_ctl status was inconclusive; refusing to remove the data directory",
        })
        return stop_result
    if status_code == 0:
        stopped = run_step("postgres-stop", [str(pg_bin / "pg_ctl.exe"), "-D", str(data), "-m", "fast", "-w", "stop"], env, logs, 120)
        stop_result["stop_exit_code"] = stopped["exit_code"]
        if stopped["exit_code"]:
            stop_result["exit_code"] = stopped["exit_code"]
            stop_result["data_retained"] = True
            return stop_result
    if server_log.is_file():
        shutil.copyfile(server_log, logs / "postgres-server.log")
    shutil.rmtree(data)
    stop_result["exit_code"] = 0
    stop_result["data_removed"] = True
    return stop_result


def write_summary(path: Path, value: dict[str, object]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--postgres-bin", type=Path, default=PG_BIN_DEFAULT)
    parser.add_argument("--cargo", default=CARGO_DEFAULT)
    parser.add_argument("--rtk", default=RTK_DEFAULT)
    parser.add_argument("--rust-toolchain", default=DEFAULT_RUST_TOOLCHAIN, help="Stable X.Y.Z compiler version, minimum 1.99.1")
    parser.add_argument("--rust-toolchain-selector", help="Optional rustup selector; only the requested version or explicit 'stable' alias is accepted")
    args = parser.parse_args()
    run_id = time.strftime("%Y%m%dT%H%M%S") + "-" + uuid.uuid4().hex[:10]
    run_dir = CACHE / "runs" / run_id
    logs = run_dir / "logs"
    data = run_dir / "postgres-data"
    if not is_inside(run_dir, CACHE):
        raise ValueError("run directory escaped runner cache")
    toolchain_info: dict[str, object] = {"requested": args.rust_toolchain, "selector": args.rust_toolchain_selector or args.rust_toolchain, "rustc_version": None, "cargo_version": None, "available": False, "error": None}
    missing = required_tools_missing(args.postgres_bin, args.cargo, args.rtk, args.rust_toolchain, args.rust_toolchain_selector, toolchain_info)
    summary: dict[str, object] = {"run_id": run_id, "status": "BLOCKED" if missing else "RUNNING", "database": DB, "bind": "127.0.0.1 only", "application_role": APP_ROLE, "rust_toolchain": toolchain_info, "expected_tests": list(EXPECTED_TESTS), "missing_prerequisites": missing, "steps": []}
    if missing:
        write_summary(run_dir / "acceptance-summary.json", summary)
        print(json.dumps(summary, ensure_ascii=False))
        return 2

    run_dir.mkdir(parents=True, exist_ok=True)
    env = sanitized_environment()
    port = select_loopback_port()
    summary["port"] = port
    steps: list[dict[str, object]] = []
    start_attempted = False
    cluster_ready = False
    try:
        data.mkdir()
        init = run_step("postgres-init", [str(args.postgres_bin / "initdb.exe"), "-D", str(data), "-U", "postgres", "--auth=trust", "--no-instructions"], env, logs, 300)
        steps.append(init)
        if init["exit_code"] == 0:
            start_attempted = True
            start = run_step("postgres-start", [str(args.postgres_bin / "pg_ctl.exe"), "-D", str(data), "-l", str(data / "postgres-server.log"), "-o", f"-h 127.0.0.1 -p {port}", "-w", "start"], env, logs, 240, capture_output=False)
            steps.append(start)
            if start["exit_code"] == 0:
                ready = run_step("postgres-ready", psql(args.postgres_bin, port, "postgres", sql="SELECT 1;"), env, logs, 30)
                steps.append(ready)
                cluster_ready = ready["exit_code"] == 0
                if cluster_ready:
                    steps.extend(apply_migrations(args.postgres_bin, port, env, logs))
                    if all(int(step.get("exit_code", 1)) == 0 for step in steps):
                        steps.append(run_postgres_tests(args.rtk, args.cargo, args.rust_toolchain, args.rust_toolchain_selector, port, env, logs))
    except Exception as error:
        summary["runner_error"] = f"{type(error).__name__}: {error}"
    finally:
        summary["cleanup"] = stop_cluster(args.postgres_bin, data, env, logs, start_attempted)
    tests = [step for step in steps if step.get("expected_tests") == list(EXPECTED_TESTS)]
    all_passed = bool(steps) and all(int(step.get("exit_code", 1)) == 0 for step in steps)
    summary["steps"] = steps
    summary["status"] = "PASS" if cluster_ready and all_passed and tests and tests[-1].get("acceptance_output_valid") else "FAIL"
    if int(summary["cleanup"].get("exit_code", 1)) != 0:
        summary["status"] = "FAIL"
    write_summary(run_dir / "acceptance-summary.json", summary)
    print(json.dumps(summary, ensure_ascii=False))
    return 0 if summary["status"] == "PASS" else 1


if __name__ == "__main__":
    sys.exit(main())
