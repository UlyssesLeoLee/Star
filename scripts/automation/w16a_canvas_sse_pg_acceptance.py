#!/usr/bin/env python3
"""Run one Canvas SSE revocation test on an owned disposable PostgreSQL 18 cluster."""

# @cypher schema=1 source_sha256=96303fe00e63e064f99689e4336a2067e28259e8c98774b031e5db088f46ed25
# MERGE (self:File {path:"scripts/automation/w16a_canvas_sse_pg_acceptance.py"})
# MERGE (root:Config {id:"scripts/automation/w16a_canvas_sse_pg_acceptance.py::ROOT",kind:"path"})
# MERGE (runsRoot:Config {id:"scripts/automation/w16a_canvas_sse_pg_acceptance.py::RUNS_ROOT",kind:"external_temp_path"})
# MERGE (databaseUrl:Config {id:"STAR_W16_DEVICE_AUTH_TEST_DATABASE_URL",kind:"environment_variable"})
# MERGE (expectedTest:Config {id:"scripts/automation/w16a_canvas_sse_pg_acceptance.py::EXPECTED_TEST",kind:"test_case"})
# MERGE (summaryArtifact:Config {id:"<temp>/star-w16a-canvas-sse-pg-acceptance/runs/<run-id>/acceptance-summary.json",kind:"dynamic_output_path"})
# MERGE (attestationLimit:Config {id:"scripts/automation/w16a_canvas_sse_pg_acceptance.py::MAX_BUILD_ATTESTATION_BYTES",kind:"byte_limit"})
# MERGE (main:Symbol {id:"scripts/automation/w16a_canvas_sse_pg_acceptance.py::main",kind:"function"})
# MERGE (sanitize:Symbol {id:"scripts/automation/w16a_canvas_sse_pg_acceptance.py::sanitized_environment",kind:"function"})
# MERGE (inside:Symbol {id:"scripts/automation/w16a_canvas_sse_pg_acceptance.py::is_inside",kind:"function"})
# MERGE (port:Symbol {id:"scripts/automation/w16a_canvas_sse_pg_acceptance.py::reserve_loopback_port",kind:"function"})
# MERGE (step:Symbol {id:"scripts/automation/w16a_canvas_sse_pg_acceptance.py::run_step",kind:"function"})
# MERGE (redact:Symbol {id:"scripts/automation/w16a_canvas_sse_pg_acceptance.py::redact_log_file",kind:"function"})
# MERGE (redactStream:Symbol {id:"scripts/automation/w16a_canvas_sse_pg_acceptance.py::redact_stream",kind:"function"})
# MERGE (streamRedactor:Symbol {id:"scripts/automation/w16a_canvas_sse_pg_acceptance.py::DatabaseUrlRedactor",kind:"class"})
# MERGE (supervise:Symbol {id:"scripts/automation/w16a_canvas_sse_pg_acceptance.py::supervise_subprocess",kind:"function"})
# MERGE (psql:Symbol {id:"scripts/automation/w16a_canvas_sse_pg_acceptance.py::psql_command",kind:"function"})
# MERGE (migrations:Symbol {id:"scripts/automation/w16a_canvas_sse_pg_acceptance.py::migration_plan",kind:"function"})
# MERGE (apply:Symbol {id:"scripts/automation/w16a_canvas_sse_pg_acceptance.py::apply_migrations",kind:"function"})
# MERGE (binary:Symbol {id:"scripts/automation/w16a_canvas_sse_pg_acceptance.py::find_test_binary",kind:"function"})
# MERGE (validateBinary:Symbol {id:"scripts/automation/w16a_canvas_sse_pg_acceptance.py::validate_test_binary",kind:"function"})
# MERGE (attestation:Symbol {id:"scripts/automation/w16a_canvas_sse_pg_acceptance.py::load_build_attestation",kind:"function"})
# MERGE (validateAttestation:Symbol {id:"scripts/automation/w16a_canvas_sse_pg_acceptance.py::validate_build_attestation",kind:"function"})
# MERGE (hashFile:Symbol {id:"scripts/automation/w16a_canvas_sse_pg_acceptance.py::sha256_file",kind:"function"})
# MERGE (repoHead:Symbol {id:"scripts/automation/w16a_canvas_sse_pg_acceptance.py::current_repository_head",kind:"function"})
# MERGE (testCommand:Symbol {id:"scripts/automation/w16a_canvas_sse_pg_acceptance.py::test_binary_command",kind:"function"})
# MERGE (summaryProof:Symbol {id:"scripts/automation/w16a_canvas_sse_pg_acceptance.py::accepted_test_output",kind:"function"})
# MERGE (cleanup:Symbol {id:"scripts/automation/w16a_canvas_sse_pg_acceptance.py::stop_and_cleanup",kind:"function"})
# MERGE (validateCleanup:Symbol {id:"scripts/automation/w16a_canvas_sse_pg_acceptance.py::validate_owned_cleanup_paths",kind:"function"})
# MERGE (reparse:Symbol {id:"scripts/automation/w16a_canvas_sse_pg_acceptance.py::has_reparse_point",kind:"function"})
# MERGE (decision:Symbol {id:"scripts/automation/w16a_canvas_sse_pg_acceptance.py::should_remove_data",kind:"function"})
# MERGE (writeSummary:Symbol {id:"scripts/automation/w16a_canvas_sse_pg_acceptance.py::write_summary",kind:"function"})
# MERGE (self)-[:DEFINES]->(main),(self)-[:DEFINES]->(sanitize),(self)-[:DEFINES]->(inside),(self)-[:DEFINES]->(port),(self)-[:DEFINES]->(step),(self)-[:DEFINES]->(redact),(self)-[:DEFINES]->(redactStream),(self)-[:DEFINES]->(streamRedactor),(self)-[:DEFINES]->(supervise),(self)-[:DEFINES]->(psql),(self)-[:DEFINES]->(migrations),(self)-[:DEFINES]->(apply),(self)-[:DEFINES]->(binary),(self)-[:DEFINES]->(validateBinary),(self)-[:DEFINES]->(attestation),(self)-[:DEFINES]->(validateAttestation),(self)-[:DEFINES]->(hashFile),(self)-[:DEFINES]->(repoHead),(self)-[:DEFINES]->(testCommand),(self)-[:DEFINES]->(summaryProof),(self)-[:DEFINES]->(cleanup),(self)-[:DEFINES]->(validateCleanup),(self)-[:DEFINES]->(reparse),(self)-[:DEFINES]->(decision),(self)-[:DEFINES]->(writeSummary)
# MERGE (self)-[:DEFINES]->(attestationLimit)
# MERGE (main)-[:CALLS]->(sanitize),(main)-[:CALLS]->(inside),(main)-[:CALLS]->(port),(main)-[:CALLS]->(step),(main)-[:CALLS]->(migrations),(main)-[:CALLS]->(apply),(main)-[:CALLS]->(binary),(main)-[:CALLS]->(validateBinary),(main)-[:CALLS]->(attestation),(main)-[:CALLS]->(testCommand),(main)-[:CALLS]->(summaryProof),(main)-[:CALLS]->(cleanup),(main)-[:CALLS]->(decision),(main)-[:CALLS]->(writeSummary)
# MERGE (main)-[:READS]->(root),(main)-[:READS]->(runsRoot),(main)-[:CONFIGURES]->(databaseUrl)
# MERGE (step)-[:CALLS]->(supervise), (step)-[:CALLS]->(:Symbol {id:"scripts/automation/w16a_canvas_sse_pg_acceptance.py::read_tail",kind:"function"})
# MERGE (redact)-[:CALLS]->(redactStream)
# MERGE (redactStream)-[:CREATES]->(streamRedactor)
# MERGE (supervise)-[:CREATES]->(:ExternalService {name:"bounded subprocess output reader thread",kind:"thread"}),(supervise)-[:CREATES]->(streamRedactor)
# MERGE (psql)-[:CONFIGURES]->(:ExternalService {name:"owned PostgreSQL 18 cluster",kind:"loopback_database"})
# MERGE (apply)-[:CALLS]->(psql),(apply)-[:CALLS]->(step),(apply)-[:CALLS]->(migrations)
# MERGE (migrations)-[:CALLS]->(:Symbol {id:"scripts/automation/engineering_run_directory.py::directory_migration_plan",kind:"function"})
# MERGE (summaryProof)-[:READS]->(expectedTest)
# MERGE (cleanup)-[:CALLS]->(step),(cleanup)-[:CALLS]->(inside),(cleanup)-[:CALLS]->(decision)
# MERGE (attestation)-[:CALLS]->(validateBinary),(attestation)-[:CALLS]->(validateAttestation),(attestation)-[:CALLS]->(hashFile),(attestation)-[:CALLS]->(repoHead)
# MERGE (attestation)-[:READS]->(attestationLimit)
# MERGE (validateAttestation)-[:CALLS]->(hashFile)
# MERGE (cleanup)-[:CALLS]->(validateCleanup)
# MERGE (validateCleanup)-[:CALLS]->(inside),(validateCleanup)-[:CALLS]->(reparse)
# MERGE (writeSummary)-[:WRITES]->(summaryArtifact)
# MERGE (main)-[:TESTS]->(:File {path:"crates/star-api-rest/tests/canvas_device_auth_pg.rs"})
# @endcypher

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
import queue
import re
import shutil
import socket
import stat
import subprocess
import sys
import tempfile
import threading
import time
import uuid
from pathlib import Path
from typing import BinaryIO, Mapping

ROOT = Path(__file__).resolve().parents[2]
RUNNER_CACHE_ROOT = Path(tempfile.gettempdir()) / "star-w16a-canvas-sse-pg-acceptance"
RUNS_ROOT = RUNNER_CACHE_ROOT / "runs"
DEFAULT_POSTGRES_BIN = Path("D:/PostgreSQL/18/bin")
DATABASE_NAME = "w16a_canvas_sse_acceptance"
DATABASE_URL_ENV = "STAR_W16_DEVICE_AUTH_TEST_DATABASE_URL"
EXPECTED_TEST = "active_canvas_sse_closes_after_device_binding_revocation"
PG_TEST_BINARY = "canvas_device_auth_pg"
CANVAS_TEST_SOURCE = ROOT / "crates" / "star-api-rest" / "tests" / "canvas_device_auth_pg.rs"
EXPECTED_RUSTC_VERSION = "1.99.0 (b940084d7 2026-09-28)"
EXPECTED_BUILD_COMMAND = (
    "rtk proxy cargo +1.99.0 test -p star-api-rest --test canvas_device_auth_pg "
    "--no-run --locked --offline -j 1"
)
REQUIRED_POSTGRES_TOOLS = ("initdb.exe", "pg_ctl.exe", "postgres.exe", "psql.exe")
BASE_MIGRATION = ROOT / "db" / "migrations" / "2026-09-09-oauth2-server.sql"
WORKTREE_PHASE_2D = ROOT / "db" / "migrations" / "2026-09-29-worktree-group-phase-2d.sql"
CANVAS_PHASE_3 = ROOT / "db" / "migrations" / "2026-09-29-worktree-group-phase-3-canvas.sql"
W16_MIGRATION = ROOT / "db" / "migrations" / "2026-10-28-w16-device-auth.sql"
CANVAS_WAKE_MIGRATION = ROOT / "db" / "migrations" / "2026-10-28-canvas-outbox-wakeup.sql"
MAX_LOG_TAIL_BYTES = 64 * 1024
REDACTION_CHUNK_BYTES = 64 * 1024
OUTPUT_QUEUE_CHUNKS = 4
PROCESS_KILL_GRACE_SECONDS = 5.0
PROCESS_QUEUE_POLL_SECONDS = 0.1
MAX_BUILD_ATTESTATION_BYTES = 64 * 1024
DATABASE_URL_PATTERN = re.compile(r"(?i)postgres(?:ql)?://[^\s\"'<>]+")
DATABASE_URL_BYTES_PATTERN = re.compile(rb"(?i)postgres(?:ql)?://[^\s\"'<>]+")
DATABASE_URL_DELIMITER_PATTERN = re.compile(rb"[\s\"'<>]")
DATABASE_URL_PREFIXES = (b"postgres://", b"postgresql://")
SAFE_REDACTION_FAILURE_LOG = b"Log redaction failed; output suppressed.\n"


class AcceptanceError(RuntimeError):
    """A fail-closed runner precondition or acceptance proof failed."""


def sanitized_environment(source: Mapping[str, str] | None = None) -> dict[str, str]:
    """Remove inherited database targets; caller adds only its generated loopback URL."""
    environment = dict(os.environ if source is None else source)
    for key in list(environment):
        upper = key.upper()
        if (
            upper.startswith("PG")
            or "DATABASE_URL" in upper
            or upper in {"DB_URL", "SQLX_OFFLINE"}
        ):
            environment.pop(key, None)
    environment["CARGO_TERM_COLOR"] = "never"
    environment["NO_COLOR"] = "1"
    return environment


def generated_database_url(port: int) -> str:
    if not 1 <= port <= 65535:
        raise ValueError("loopback port is outside the valid TCP range")
    return f"postgres://postgres@127.0.0.1:{port}/{DATABASE_NAME}"


def test_environment(source: Mapping[str, str], port: int) -> dict[str, str]:
    """Build the test environment without accepting a caller-supplied database URL."""
    environment = sanitized_environment(source)
    environment[DATABASE_URL_ENV] = generated_database_url(port)
    return environment


def is_inside(path: Path, root: Path) -> bool:
    """Return true only for a strict descendant after resolving symlinks and traversal."""
    resolved_path = path.resolve()
    resolved_root = root.resolve()
    return resolved_path != resolved_root and resolved_path.is_relative_to(resolved_root)


def paths_are_disjoint(left: Path, right: Path) -> bool:
    """Ensure the external runner evidence root and Git worktree cannot contain each other."""
    resolved_left = left.resolve()
    resolved_right = right.resolve()
    return (
        resolved_left != resolved_right
        and not resolved_left.is_relative_to(resolved_right)
        and not resolved_right.is_relative_to(resolved_left)
    )


def has_reparse_point(path: Path, *, recursive: bool = False) -> bool:
    """Detect symlinks, junctions, and Windows reparse points without following them."""
    try:
        metadata = path.lstat()
    except FileNotFoundError:
        return False
    reparse_flag = getattr(stat, "FILE_ATTRIBUTE_REPARSE_POINT", 0x400)
    if stat.S_ISLNK(metadata.st_mode) or getattr(metadata, "st_file_attributes", 0) & reparse_flag:
        return True
    if recursive and stat.S_ISDIR(metadata.st_mode):
        with os.scandir(path) as entries:
            for entry in entries:
                child = Path(entry.path)
                if has_reparse_point(child, recursive=False):
                    return True
                if entry.is_dir(follow_symlinks=False) and has_reparse_point(child, recursive=True):
                    return True
    return False


def validate_owned_cleanup_paths(
    run_dir: Path,
    data_dir: Path,
    *,
    evidence_root: Path = RUNS_ROOT,
    runner_root: Path = RUNNER_CACHE_ROOT,
    worktree_root: Path = ROOT,
) -> Path:
    """Revalidate exact run ownership and reject reparse points before deleting cluster data."""
    if not paths_are_disjoint(runner_root, worktree_root):
        raise AcceptanceError("external evidence root no longer remains separate from the Git worktree")
    if has_reparse_point(runner_root) or has_reparse_point(evidence_root):
        raise AcceptanceError("external evidence root contains a symlink, junction, or reparse point")
    try:
        resolved_evidence = evidence_root.resolve(strict=True)
        resolved_run = run_dir.resolve(strict=True)
        resolved_data = data_dir.resolve(strict=True)
    except OSError as error:
        raise AcceptanceError("owned run or PostgreSQL data path no longer resolves") from error
    if re.fullmatch(r"[0-9a-f]{32}", run_dir.name) is None:
        raise AcceptanceError("owned run directory name is not a generated run id")
    expected_run = resolved_evidence / run_dir.name
    if resolved_run != expected_run or not is_inside(run_dir, evidence_root):
        raise AcceptanceError("owned run directory is not a direct child of the evidence runs root")
    expected_data = resolved_run / "postgres-data"
    if resolved_data != expected_data or data_dir.name != "postgres-data":
        raise AcceptanceError("PostgreSQL data path is not the canonical <run>/postgres-data directory")
    if not resolved_run.is_dir() or not resolved_data.is_dir():
        raise AcceptanceError("owned run and PostgreSQL data paths must both remain directories")
    if has_reparse_point(run_dir, recursive=True):
        raise AcceptanceError("owned run tree contains a symlink, junction, or reparse point")
    return resolved_data


def run_paths(run_id: str, runs_root: Path = RUNS_ROOT) -> tuple[Path, Path, Path]:
    if re.fullmatch(r"[0-9a-f]{32}", run_id) is None:
        raise ValueError("run id must be a 32-character lowercase UUID hex value")
    run_dir = runs_root / run_id
    if not is_inside(run_dir, runs_root):
        raise ValueError("run directory escapes the runner's private runs root")
    if run_dir.exists():
        raise FileExistsError("unique run directory already exists")
    return run_dir, run_dir / "postgres-data", run_dir / "logs"


def create_run_paths(run_id: str, runs_root: Path = RUNS_ROOT) -> tuple[Path, Path, Path]:
    run_dir, data_dir, logs_dir = run_paths(run_id, runs_root)
    run_dir.mkdir(parents=True, exist_ok=False)
    if not is_inside(run_dir, runs_root):
        raise ValueError("created run directory escaped the runner's private runs root")
    data_dir.mkdir()
    logs_dir.mkdir()
    return run_dir, data_dir, logs_dir


def reserve_loopback_port() -> tuple[int, socket.socket]:
    """Hold an ephemeral IPv4 loopback port until PostgreSQL is ready to start."""
    reservation = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    try:
        reservation.bind(("127.0.0.1", 0))
        return int(reservation.getsockname()[1]), reservation
    except BaseException:
        reservation.close()
        raise


def read_tail(path: Path, limit: int = MAX_LOG_TAIL_BYTES) -> str:
    if limit <= 0:
        raise ValueError("log tail limit must be positive")
    if not path.is_file():
        return ""
    with path.open("rb") as stream:
        stream.seek(0, os.SEEK_END)
        size = stream.tell()
        stream.seek(max(0, size - limit))
        raw_tail = stream.read(limit)
    decoded = raw_tail.decode("utf-8", errors="replace")
    return DATABASE_URL_PATTERN.sub("<redacted-database-url>", decoded)


def redact_stream(source: BinaryIO, destination: BinaryIO, chunk_size: int = REDACTION_CHUNK_BYTES) -> None:
    """Redact arbitrarily long URL tokens with bounded memory and chunked I/O."""
    if chunk_size <= 0:
        raise ValueError("redaction chunk size must be positive")
    redactor = DatabaseUrlRedactor()
    while chunk := source.read(chunk_size):
        destination.write(redactor.feed(chunk))
    destination.write(redactor.finish())


class DatabaseUrlRedactor:
    """Transform output chunks while withholding any unfinished PostgreSQL URL token."""

    def __init__(self) -> None:
        self.carry = b""
        self.redacting_url = False
        self.max_prefix_length = max(map(len, DATABASE_URL_PREFIXES))

    def feed(self, chunk: bytes) -> bytes:
        safe = bytearray()
        if self.redacting_url:
            delimiter = DATABASE_URL_DELIMITER_PATTERN.search(chunk)
            if delimiter is None:
                return b""
            safe.extend(chunk[delimiter.start() : delimiter.end()])
            chunk = chunk[delimiter.end() :]
            self.redacting_url = False

        combined = self.carry + chunk
        self.carry = b""
        cursor = 0
        for match in DATABASE_URL_BYTES_PATTERN.finditer(combined):
            safe.extend(combined[cursor : match.start()])
            safe.extend(b"<redacted-database-url>")
            cursor = match.end()
            if match.end() == len(combined):
                self.redacting_url = True
                cursor = len(combined)
                break
        if self.redacting_url:
            return bytes(safe)

        remaining = combined[cursor:]
        keep = 0
        for suffix_length in range(min(len(remaining), self.max_prefix_length), 0, -1):
            suffix = remaining[-suffix_length:].lower()
            if any(prefix.startswith(suffix) for prefix in DATABASE_URL_PREFIXES):
                keep = suffix_length
                break
        safe.extend(remaining[:-keep] if keep else remaining)
        if keep:
            self.carry = remaining[-keep:]
        return bytes(safe)

    def finish(self) -> bytes:
        if self.redacting_url:
            self.carry = b""
            return b""
        safe = self.carry
        self.carry = b""
        return safe


def redact_log_file(path: Path) -> None:
    """Sanitize a log in place without renaming/replacing its Windows file entry."""
    if path.is_symlink():
        raise AcceptanceError("refusing to redact a log path that is a symlink")
    if not path.is_file():
        return
    redacted_path = path.with_name(f".{path.name}.{uuid.uuid4().hex}.redacted.tmp")
    temp_created = False
    try:
        with path.open("rb") as source, redacted_path.open("xb") as destination:
            temp_created = True
            redact_stream(source, destination)
            destination.flush()
            os.fsync(destination.fileno())
        with redacted_path.open("rb") as source, path.open("r+b") as destination:
            shutil.copyfileobj(source, destination, length=REDACTION_CHUNK_BYTES)
            destination.truncate()
            destination.flush()
            os.fsync(destination.fileno())
    except Exception:
        try:
            with path.open("wb") as failure_log:
                failure_log.write(SAFE_REDACTION_FAILURE_LOG)
                failure_log.flush()
                os.fsync(failure_log.fileno())
        except OSError:
            pass
        raise AcceptanceError("log redaction failed; raw output tail suppressed") from None
    finally:
        if temp_created:
            try:
                redacted_path.unlink(missing_ok=True)
            except OSError:
                try:
                    with path.open("wb") as failure_log:
                        failure_log.write(SAFE_REDACTION_FAILURE_LOG)
                except OSError:
                    pass
                raise AcceptanceError("redaction temporary cleanup failed; raw output tail suppressed") from None


def run_step(
    name: str,
    command: list[str],
    environment: Mapping[str, str],
    logs_dir: Path,
    timeout_seconds: int,
    *,
    capture_output: bool = True,
) -> dict[str, object]:
    """Capture redacted output by default; suppress streams for detached service startup."""
    if re.fullmatch(r"[A-Za-z0-9_-]+", name) is None:
        raise ValueError("step name contains unsafe path characters")
    if timeout_seconds <= 0:
        raise ValueError("step timeout must be positive")
    logs_dir.mkdir(parents=True, exist_ok=True)
    log_path = logs_dir / f"{name}.log"
    result: dict[str, object]
    try:
        with log_path.open("xb") as output:
            try:
                process = subprocess.Popen(
                    command,
                    cwd=ROOT,
                    env=dict(environment),
                    stdout=subprocess.PIPE if capture_output else subprocess.DEVNULL,
                    stderr=subprocess.STDOUT,
                    bufsize=0,
                    shell=False,
                    creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
                )
            except FileNotFoundError:
                output.write(b"FileNotFoundError: required executable was not found\n")
                return_code, timed_out, capture_failed = 127, False, False
                result = {
                    "name": name,
                    "command": command,
                    "exit_code": return_code,
                    "error": "required executable was not found",
                }
            except OSError as error:
                output.write(f"{type(error).__name__}: subprocess could not start\n".encode("utf-8"))
                return_code, timed_out, capture_failed = 127, False, False
                result = {
                    "name": name,
                    "command": command,
                    "exit_code": return_code,
                    "error": f"subprocess could not start ({type(error).__name__})",
                }
            else:
                try:
                    if capture_output:
                        return_code, timed_out, capture_failed = supervise_subprocess(
                            process,
                            output,
                            timeout_seconds,
                        )
                    else:
                        output.write(
                            b"Subprocess stdout/stderr redirected to the null device; "
                            b"service readiness is checked by a separate step.\n"
                        )
                        try:
                            return_code = process.wait(timeout=timeout_seconds)
                            timed_out = False
                        except subprocess.TimeoutExpired:
                            timed_out = True
                            try:
                                process.kill()
                            except OSError:
                                pass
                            try:
                                process.wait(timeout=PROCESS_KILL_GRACE_SECONDS)
                            except (OSError, subprocess.TimeoutExpired):
                                pass
                            return_code = 124
                        except BaseException:
                            try:
                                process.kill()
                            except BaseException:
                                pass
                            try:
                                process.wait(timeout=PROCESS_KILL_GRACE_SECONDS)
                            except BaseException:
                                pass
                            raise
                        capture_failed = False
                        if return_code is None:
                            return_code = 124 if timed_out else 125
                    if capture_failed and not timed_out:
                        return_code = 125
                        output.seek(0)
                        output.truncate()
                        output.write(SAFE_REDACTION_FAILURE_LOG)
                    result = {"name": name, "command": command, "exit_code": return_code}
                    if timed_out:
                        result["error"] = f"timed out after {timeout_seconds} seconds"
                    elif capture_failed:
                        result["error"] = "subprocess output capture failed; acceptance cannot be proven"
                except Exception:
                    output.seek(0)
                    output.truncate()
                    output.write(SAFE_REDACTION_FAILURE_LOG)
                    output.flush()
                    raise AcceptanceError("subprocess output redaction failed; raw output tail suppressed") from None
            output.flush()
            os.fsync(output.fileno())
    except FileExistsError:
        raise
    result["log"] = str(log_path)
    result["output_tail"] = read_tail(log_path)
    print(f"STEP[{name}] exit={result['exit_code']} log={log_path}", flush=True)
    if result["exit_code"]:
        tail = result["output_tail"]
        if tail:
            print(f"OUTPUT_TAIL[{name}]\n{tail}", flush=True)
    return result


def supervise_subprocess(
    process: subprocess.Popen[bytes],
    output: BinaryIO,
    timeout_seconds: int,
) -> tuple[int, bool, bool]:
    """Pump raw pipe bytes through a bounded queue and redact before writing the log."""
    if process.stdout is None:
        raise AcceptanceError("subprocess output pipe was not created")
    output_queue: queue.Queue[bytes | object] = queue.Queue(maxsize=OUTPUT_QUEUE_CHUNKS)
    eof_token = object()
    reader_failed = threading.Event()

    def read_pipe() -> None:
        try:
            while chunk := process.stdout.read(REDACTION_CHUNK_BYTES):
                output_queue.put(chunk)
        except BaseException:
            reader_failed.set()
        finally:
            output_queue.put(eof_token)

    reader = threading.Thread(target=read_pipe, name="w16a-redacted-output-reader", daemon=True)
    reader.start()
    redactor = DatabaseUrlRedactor()
    deadline = time.monotonic() + timeout_seconds
    eof_seen = False
    timed_out = False
    post_kill_deadline: float | None = None

    def drain_until(until: float) -> bool:
        nonlocal eof_seen
        while not eof_seen:
            remaining = until - time.monotonic()
            if remaining <= 0:
                return False
            try:
                item = output_queue.get(timeout=min(PROCESS_QUEUE_POLL_SECONDS, remaining))
            except queue.Empty:
                continue
            if item is eof_token:
                eof_seen = True
                break
            safe_chunk = redactor.feed(item)  # type: ignore[arg-type]
            if safe_chunk:
                output.write(safe_chunk)
            output.flush()
        return True

    try:
        eof_seen = drain_until(deadline)
        if eof_seen:
            try:
                remaining = max(0.001, deadline - time.monotonic())
                return_code = process.wait(timeout=remaining)
            except subprocess.TimeoutExpired:
                timed_out = True
                return_code = process.poll()
        else:
            timed_out = True
            return_code = process.poll()

        if timed_out:
            try:
                process.kill()
            except OSError:
                pass
            drain_deadline = time.monotonic() + PROCESS_KILL_GRACE_SECONDS
            post_kill_deadline = drain_deadline
            if not eof_seen:
                eof_seen = drain_until(drain_deadline)
            try:
                return_code = process.wait(timeout=max(0.001, drain_deadline - time.monotonic()))
            except subprocess.TimeoutExpired:
                return_code = process.poll()

        if eof_seen:
            final_chunk = redactor.finish()
            if final_chunk:
                output.write(final_chunk)
        else:
            try:
                process.stdout.close()
            except OSError:
                pass
            output.write(b"\nOutput capture ended before pipe EOF; trailing output omitted.\n")
            reader_failed.set()

        reader_join_budget = (
            PROCESS_KILL_GRACE_SECONDS
            if post_kill_deadline is None
            else max(0.0, post_kill_deadline - time.monotonic())
        )
        reader.join(timeout=reader_join_budget)
        if reader.is_alive():
            try:
                process.stdout.close()
            except OSError:
                pass
            reader_failed.set()
        output.flush()
        os.fsync(output.fileno())
        if return_code is None:
            return_code = process.poll()
        if return_code is None:
            return_code = 124 if timed_out else 125
            reader_failed.set()
        capture_failed = reader_failed.is_set()
        if timed_out:
            return 124, True, capture_failed
        if capture_failed:
            return 125, False, True
        return return_code, False, False
    except BaseException:
        try:
            process.kill()
        except OSError:
            pass
        cleanup_deadline = time.monotonic() + PROCESS_KILL_GRACE_SECONDS
        try:
            process.wait(timeout=max(0.001, cleanup_deadline - time.monotonic()))
        except (OSError, subprocess.TimeoutExpired):
            pass
        while not eof_seen:
            remaining = cleanup_deadline - time.monotonic()
            if remaining <= 0:
                break
            try:
                item = output_queue.get(timeout=min(PROCESS_QUEUE_POLL_SECONDS, remaining))
            except queue.Empty:
                continue
            if item is eof_token:
                eof_seen = True
        if not eof_seen:
            try:
                process.stdout.close()
            except OSError:
                pass
        try:
            output.flush()
            os.fsync(output.fileno())
        except OSError:
            pass
        reader.join(timeout=max(0.0, cleanup_deadline - time.monotonic()))
        raise


def psql_command(
    postgres_bin: Path,
    port: int,
    database: str,
    *,
    sql: str | None = None,
    file: Path | None = None,
    login: str = "postgres",
) -> list[str]:
    if not 1 <= port <= 65535:
        raise ValueError("psql port is outside the valid TCP range")
    if not re.fullmatch(r"[a-z][a-z0-9_]{0,62}", database):
        raise ValueError("database name is not an accepted fixed identifier")
    if not re.fullmatch(r"[a-z][a-z0-9_]{0,62}", login):
        raise ValueError("PostgreSQL login is not an accepted fixed identifier")
    if (sql is None) == (file is None):
        raise ValueError("psql command requires exactly one SQL string or SQL file")
    command = [
        str(postgres_bin / "psql.exe"),
        "-X",
        "-w",
        "-h",
        "127.0.0.1",
        "-p",
        str(port),
        "-U",
        login,
        "-d",
        database,
        "-v",
        "ON_ERROR_STOP=1",
        "-q",
        "-A",
        "-t",
    ]
    if sql is not None:
        command.extend(["-c", sql])
    else:
        command.extend(["-f", str(file)])
    return command


def run_sql(
    name: str,
    postgres_bin: Path,
    port: int,
    environment: Mapping[str, str],
    logs_dir: Path,
    *,
    sql: str | None = None,
    file: Path | None = None,
    database: str = DATABASE_NAME,
    login: str = "postgres",
) -> dict[str, object]:
    return run_step(
        name,
        psql_command(postgres_bin, port, database, sql=sql, file=file, login=login),
        environment,
        logs_dir,
        timeout_seconds=240,
    )


def directory_helper():
    """Import the repository's audited migration-plan and role-check helpers."""
    root_text = str(ROOT)
    inserted = root_text not in sys.path
    if inserted:
        sys.path.insert(0, root_text)
    try:
        from scripts.automation import engineering_run_directory

        return engineering_run_directory
    finally:
        if inserted:
            try:
                sys.path.remove(root_text)
            except ValueError:
                pass


def migration_plan() -> list[tuple[str, Path]]:
    """Return the reviewed OAuth, Directory, Worktree Group, W16 and Canvas chain."""
    directory = directory_helper()
    migrations: list[tuple[str, Path]] = [("postgres-oauth-base", BASE_MIGRATION)]
    migrations.extend(
        (name, ROOT / "db" / "migrations" / filename)
        for name, filename in directory.directory_migration_plan(run_binding=True)
    )
    migrations.extend(
        [
            ("postgres-worktree-group-phase-2d", WORKTREE_PHASE_2D),
            ("postgres-worktree-group-phase-3-canvas", CANVAS_PHASE_3),
            ("postgres-w16-device-auth", W16_MIGRATION),
            ("postgres-canvas-outbox-wakeup", CANVAS_WAKE_MIGRATION),
        ]
    )
    return migrations


def validate_migration_plan(plan: list[tuple[str, Path]]) -> None:
    migration_root = (ROOT / "db" / "migrations").resolve()
    if not plan:
        raise AcceptanceError("required migration plan is empty")
    if plan[0] != ("postgres-oauth-base", BASE_MIGRATION):
        raise AcceptanceError("OAuth base migration must be the first migration step")
    names: set[str] = set()
    missing: list[str] = []
    for name, path in plan:
        if name in names:
            raise AcceptanceError(f"duplicate migration step name: {name}")
        names.add(name)
        if not is_inside(path, migration_root):
            raise AcceptanceError(f"migration path is outside db/migrations: {path.name}")
        if not path.is_file():
            missing.append(path.name)
    if missing:
        raise AcceptanceError("required migration files are missing: " + ", ".join(missing))
    filenames = [path.name for _, path in plan]
    directory = directory_helper()
    helper_files = [filename for _, filename in directory.directory_migration_plan(run_binding=True)]
    if filenames[1 : 1 + len(helper_files)] != helper_files:
        raise AcceptanceError("Directory migration helper order changed unexpectedly")
    index_2d = filenames.index(WORKTREE_PHASE_2D.name)
    index_phase_3 = filenames.index(CANVAS_PHASE_3.name)
    index_w16 = filenames.index(W16_MIGRATION.name)
    index_canvas_wake = filenames.index(CANVAS_WAKE_MIGRATION.name)
    if not (index_2d > len(helper_files) and index_2d < index_phase_3 < index_w16 < index_canvas_wake):
        raise AcceptanceError("2D, Canvas phase-3, W16 and Canvas wake migrations are out of order")
    if "2026-10-05-worktree-group-canvas-stream-offset.sql" in filenames:
        raise AcceptanceError("Canvas stream-offset migration is outside this test's required schema")


def test_only_prerequisites_sql() -> str:
    """Stand in only for platform audit/SCD2 dependencies on this disposable schema."""
    return """-- TEST-ONLY placeholders for platform-owned audit/SCD2 dependencies.
    CREATE TABLE public.audit_audit_event (
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
    CREATE FUNCTION public.audit_trigger_func() RETURNS trigger
      LANGUAGE plpgsql AS $$ BEGIN RETURN COALESCE(NEW,OLD); END $$;"""


def legacy_refresh_pair_sql() -> str:
    return """-- TEST-ONLY legacy shape; W16 migration must replace this weak CHECK.
    ALTER TABLE public.oauth_refresh_tokens ADD COLUMN IF NOT EXISTS device_id uuid;
    ALTER TABLE public.oauth_refresh_tokens ADD COLUMN IF NOT EXISTS device_generation bigint;
    DO $legacy_pair$
    BEGIN
      IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conrelid='public.oauth_refresh_tokens'::regclass
          AND conname='oauth_refresh_tokens_device_generation_pair'
      ) THEN
        ALTER TABLE public.oauth_refresh_tokens
          ADD CONSTRAINT oauth_refresh_tokens_device_generation_pair
          CHECK ((device_id IS NULL AND device_generation IS NULL)
              OR (device_id IS NOT NULL AND device_generation > 0));
      END IF;
    END
    $legacy_pair$;"""


def verify_w16_catalog_sql() -> str:
    return """DO $verify$
    DECLARE role_contract boolean;
    BEGIN
      IF to_regclass('public.oauth_device_bindings') IS NULL
         OR to_regclass('canvas.canvas_group_outbox') IS NULL THEN
        RAISE EXCEPTION 'required W16 device or Canvas Outbox relation is missing';
      END IF;
      IF NOT EXISTS (
        SELECT 1 FROM pg_trigger
        WHERE tgrelid='public.oauth_device_bindings'::regclass
          AND tgname='w16_oauth_device_binding_generation_wake'
          AND NOT tgisinternal AND tgenabled IN ('O','A')
      ) THEN RAISE EXCEPTION 'W16 device-binding wake trigger is missing'; END IF;
      IF NOT EXISTS (
        SELECT 1 FROM pg_trigger
        WHERE tgrelid='canvas.canvas_group_outbox'::regclass
          AND tgname='canvas_group_outbox_wake'
          AND NOT tgisinternal AND tgenabled IN ('O','A')
      ) THEN RAISE EXCEPTION 'Canvas Outbox wake trigger is missing'; END IF;
      SELECT NOT rolcanlogin AND NOT rolsuper AND NOT rolinherit AND NOT rolbypassrls
        INTO role_contract FROM pg_roles WHERE rolname='star_oauth_credential_owner';
      IF role_contract IS DISTINCT FROM TRUE THEN
        RAISE EXCEPTION 'W16 OAuth credential owner role attributes are unsafe or missing';
      END IF;
      IF NOT has_table_privilege('star_oauth_credential_owner','public.oauth_device_bindings','SELECT')
         OR NOT has_table_privilege('star_oauth_credential_owner','public.oauth_device_bindings','INSERT')
         OR NOT has_column_privilege('star_oauth_credential_owner','public.oauth_device_bindings','generation','UPDATE')
         OR NOT has_column_privilege('star_oauth_credential_owner','public.oauth_device_bindings','status','UPDATE')
         OR NOT has_table_privilege('star_oauth_credential_owner','public.oauth_device_binding_audit','INSERT') THEN
        RAISE EXCEPTION 'W16 OAuth owner table/column grants are incomplete';
      END IF;
      IF EXISTS (
        WITH protected(relid) AS (VALUES
          ('public.oauth_device_enrollment_challenges'::regclass::oid),
          ('public.oauth_device_bindings'::regclass::oid),
          ('public.oauth_device_binding_audit'::regclass::oid)
        )
        SELECT 1 FROM protected p JOIN pg_class c ON c.oid=p.relid
        CROSS JOIN LATERAL aclexplode(CASE WHEN cardinality(c.relacl)>0 THEN c.relacl END) acl
        WHERE acl.grantee IN (0::oid,'erun_directory_api_test'::regrole::oid)
      ) THEN RAISE EXCEPTION 'PUBLIC or runtime login has a direct OAuth device table grant'; END IF;
    END
    $verify$;"""


def apply_migrations(
    postgres_bin: Path,
    port: int,
    environment: Mapping[str, str],
    logs_dir: Path,
    plan: list[tuple[str, Path]],
) -> list[dict[str, object]]:
    """Apply only the reviewed fixture chain and stop at the first failed step."""
    directory = directory_helper()
    results: list[dict[str, object]] = []

    def apply_sql_step(
        name: str,
        *,
        sql: str | None = None,
        file: Path | None = None,
        database: str = DATABASE_NAME,
        login: str = "postgres",
    ) -> bool:
        result = run_sql(
            name,
            postgres_bin,
            port,
            environment,
            logs_dir,
            sql=sql,
            file=file,
            database=database,
            login=login,
        )
        results.append(result)
        return result["exit_code"] == 0

    if not apply_sql_step("postgres-create-test-database", database="postgres", sql=f"CREATE DATABASE {DATABASE_NAME};"):
        return results
    if not apply_sql_step("postgres-test-only-audit-prerequisites", sql=test_only_prerequisites_sql()):
        return results

    directory_names = {name for name, _ in directory.directory_migration_plan(run_binding=True)}
    for name, path in plan:
        if name == "postgres-w16-device-auth":
            if not apply_sql_step("postgres-test-only-legacy-refresh-pair", sql=legacy_refresh_pair_sql()):
                return results
            setup_sql = directory.run_binding_role_setup_sql()
            if not apply_sql_step("postgres-directory-owner-role-fixture", sql=setup_sql):
                return results
            role_sql = directory.run_binding_role_acceptance_sql()
            if not apply_sql_step(
                "postgres-directory-owner-grant-acceptance",
                sql=role_sql,
                login="erun_directory_api_test",
            ):
                return results
        if not apply_sql_step(name, file=path):
            return results
        if name in directory_names:
            fingerprint = run_sql(
                f"{name}-schema-fingerprint",
                postgres_bin,
                port,
                environment,
                logs_dir,
                sql=directory.migration_fingerprint_sql(),
            )
            results.append(fingerprint)
            if fingerprint["exit_code"]:
                return results
        if name == "postgres-canvas-outbox-wakeup":
            if not apply_sql_step("postgres-w16-and-canvas-trigger-catalog-check", sql=verify_w16_catalog_sql()):
                return results
    return results


def postgres_tool_paths(postgres_bin: Path) -> dict[str, Path]:
    bin_dir = postgres_bin.resolve()
    if not bin_dir.is_dir():
        raise AcceptanceError("--postgres-bin must name an existing local PostgreSQL bin directory")
    tools: dict[str, Path] = {}
    for name in REQUIRED_POSTGRES_TOOLS:
        tool = bin_dir / name
        if not tool.is_file() or tool.resolve().parent != bin_dir:
            raise AcceptanceError(f"required PostgreSQL 18 tool is missing or outside --postgres-bin: {name}")
        tools[name] = tool.resolve()
    return tools


def find_test_binary(target_root: Path = ROOT / "target") -> Path:
    if target_root.is_symlink():
        raise AcceptanceError("--target-dir must not be a symlink")
    resolved_target = target_root.resolve()
    deps_dir = resolved_target / "debug" / "deps"
    if deps_dir.parent.is_symlink() or deps_dir.is_symlink() or not is_inside(deps_dir, resolved_target):
        raise AcceptanceError("compiled test dependency directory must be a real target/debug/deps directory")
    filename_pattern = re.compile(rf"^{re.escape(PG_TEST_BINARY)}-[0-9a-f]{{16}}\.exe$", re.IGNORECASE)
    candidates = sorted(
        path
        for path in deps_dir.glob(f"{PG_TEST_BINARY}-*.exe")
        if filename_pattern.fullmatch(path.name) and path.is_file() and not path.is_symlink()
    ) if deps_dir.is_dir() else []
    if len(candidates) != 1:
        raise AcceptanceError(
            f"expected exactly one compiled {PG_TEST_BINARY} test binary under --target-dir/debug/deps; found {len(candidates)}"
        )
    binary = candidates[0].resolve()
    if not is_inside(binary, deps_dir):
        raise AcceptanceError("compiled Canvas test binary is outside --target-dir/debug/deps")
    return binary


def validate_test_binary(test_binary: Path, target_root: Path | None = None) -> tuple[Path, Path]:
    """Validate an explicitly supplied exact Cargo test artifact and its target root."""
    if test_binary.is_symlink():
        raise AcceptanceError("--test-binary must not be a symlink")
    binary = test_binary.resolve()
    if not binary.is_file() or binary.suffix.lower() != ".exe":
        raise AcceptanceError("--test-binary must name an existing compiled .exe file")
    filename_pattern = re.compile(rf"^{re.escape(PG_TEST_BINARY)}-[0-9a-f]{{16}}\.exe$", re.IGNORECASE)
    if filename_pattern.fullmatch(binary.name) is None:
        raise AcceptanceError(f"--test-binary must be the exact {PG_TEST_BINARY}-<16-hex-hash>.exe target")
    deps_dir = binary.parent
    debug_dir = deps_dir.parent
    inferred_target = debug_dir.parent
    if deps_dir.name.lower() != "deps" or debug_dir.name.lower() != "debug":
        raise AcceptanceError("--test-binary must use the standard target/debug/deps layout")
    if deps_dir.is_symlink() or debug_dir.is_symlink() or inferred_target.is_symlink():
        raise AcceptanceError("--test-binary target/debug/deps path must not contain symlink directories")
    if not is_inside(binary, deps_dir) or binary.parent != deps_dir:
        raise AcceptanceError("--test-binary must be a direct file in target/debug/deps")
    if target_root is not None:
        if target_root.is_symlink():
            raise AcceptanceError("--target-dir must not be a symlink")
        resolved_target = target_root.resolve()
        if inferred_target != resolved_target:
            raise AcceptanceError("--test-binary is not under the supplied --target-dir")
    return binary, inferred_target


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def current_repository_head() -> str:
    completed = subprocess.run(
        ["git", "-C", str(ROOT), "rev-parse", "--verify", "HEAD"],
        cwd=ROOT,
        env=sanitized_environment(),
        capture_output=True,
        text=True,
        timeout=15,
        check=False,
    )
    head = completed.stdout.strip()
    if completed.returncode != 0 or re.fullmatch(r"[0-9a-f]{40}", head) is None:
        raise AcceptanceError("could not verify the current full Git HEAD for build attestation")
    return head


def validate_build_attestation(
    record: Mapping[str, object],
    test_binary: Path,
    current_head: str,
    source_path: Path = CANVAS_TEST_SOURCE,
) -> dict[str, str | int | float]:
    """Compare an unsigned operator attestation with current source, HEAD, and binary bytes."""
    required_fields = {
        "head",
        "canvas_test_source_sha256",
        "test_binary_sha256",
        "rustc_version",
        "build_command",
        "build_exit_code",
        "build_elapsed_seconds",
    }
    if set(record) != required_fields:
        raise AcceptanceError("build attestation must contain exactly the seven required evidence fields")
    values = {
        key: record.get(key)
        for key in required_fields
        if key not in {"build_exit_code", "build_elapsed_seconds"}
    }
    if not all(isinstance(value, str) for value in values.values()):
        raise AcceptanceError("build attestation text evidence fields must all be strings")
    if type(record.get("build_exit_code")) is not int or record["build_exit_code"] != 0:
        raise AcceptanceError("build attestation must record a successful build exit code of zero")
    build_elapsed = record.get("build_elapsed_seconds")
    if (
        isinstance(build_elapsed, bool)
        or not isinstance(build_elapsed, (int, float))
        or not math.isfinite(build_elapsed)
        or build_elapsed < 0
    ):
        raise AcceptanceError("build attestation elapsed time must be a finite non-negative number")
    asserted_head = str(values["head"]).lower()
    if re.fullmatch(r"[0-9a-f]{40}", current_head) is None or asserted_head != current_head.lower():
        raise AcceptanceError("build attestation HEAD does not match the current repository HEAD")
    if values["rustc_version"] != EXPECTED_RUSTC_VERSION:
        raise AcceptanceError("build attestation does not name the required Rust 1.99.0 toolchain")
    if values["build_command"] != EXPECTED_BUILD_COMMAND:
        raise AcceptanceError("build attestation command does not match the fixed locked/offline --no-run command")
    if source_path.is_symlink() or not source_path.is_file():
        raise AcceptanceError("Canvas SSE acceptance source is missing or is a symlink")
    actual_source_hash = sha256_file(source_path)
    actual_binary_hash = sha256_file(test_binary)
    if str(values["canvas_test_source_sha256"]).lower() != actual_source_hash:
        raise AcceptanceError("build attestation source SHA-256 does not match canvas_device_auth_pg.rs")
    if str(values["test_binary_sha256"]).lower() != actual_binary_hash:
        raise AcceptanceError("build attestation binary SHA-256 does not match the selected test executable")
    return {
        "head": current_head.lower(),
        "canvas_test_source_sha256": actual_source_hash,
        "test_binary_sha256": actual_binary_hash,
        "rustc_version": EXPECTED_RUSTC_VERSION,
        "build_command": EXPECTED_BUILD_COMMAND,
        "build_exit_code": 0,
        "build_elapsed_seconds": build_elapsed,
    }


def load_build_attestation(attestation_path: Path, test_binary: Path) -> dict[str, str | int | float]:
    if attestation_path.is_symlink() or not attestation_path.is_file():
        raise AcceptanceError("--build-attestation must name an existing regular JSON file")
    if not paths_are_disjoint(attestation_path, ROOT):
        raise AcceptanceError("--build-attestation must be stored outside the Git worktree")
    test_binary, _ = validate_test_binary(test_binary)
    try:
        with attestation_path.open("rb") as attestation_file:
            raw_attestation = attestation_file.read(MAX_BUILD_ATTESTATION_BYTES + 1)
        if len(raw_attestation) > MAX_BUILD_ATTESTATION_BYTES:
            raise AcceptanceError("--build-attestation exceeds the 64 KiB size limit")
        record = json.loads(raw_attestation.decode("utf-8"))
    except AcceptanceError:
        raise
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        raise AcceptanceError("--build-attestation is not readable valid JSON") from error
    if not isinstance(record, dict):
        raise AcceptanceError("--build-attestation JSON must be an object")
    verified = validate_build_attestation(record, test_binary, current_repository_head())
    verified["attestation_file"] = str(attestation_path.resolve())
    return verified


def test_binary_command(test_binary: Path) -> list[str]:
    return [str(test_binary), EXPECTED_TEST, "--ignored", "--exact", "--test-threads=1"]


def accepted_test_output(output: str) -> tuple[bool, list[str]]:
    """Require exactly the named test and one clean libtest pass summary."""
    issues: list[str] = []
    if re.search(r"(?m)^running 1 test$", output) is None:
        issues.append("test binary did not report exactly one running test")
    expected_line = rf"(?m)^test {re.escape(EXPECTED_TEST)} \.\.\. ok$"
    if re.search(expected_line, output) is None:
        issues.append(f"expected test did not pass: {EXPECTED_TEST}")
    test_lines = re.findall(r"(?m)^test .+ \.\.\. (?:ok|ignored|FAILED)$", output)
    if test_lines != [f"test {EXPECTED_TEST} ... ok"]:
        issues.append("output included a different test selection or more than one test result")
    summary = r"(?m)^test result: ok\. 1 passed; 0 failed; 0 ignored; 0 measured; [0-9]+ filtered out; finished in [0-9.]+s$"
    if re.search(summary, output) is None:
        issues.append("missing one-pass, zero-failure, zero-ignored summary")
    return not issues, issues


def should_remove_data(test_passed: bool, shutdown_confirmed: bool, path_is_owned: bool) -> bool:
    """Never remove data unless it belongs to this run and shutdown is confirmed."""
    return test_passed and shutdown_confirmed and path_is_owned


def stop_and_cleanup(
    postgres_bin: Path,
    data_dir: Path,
    run_dir: Path,
    environment: Mapping[str, str],
    logs_dir: Path,
    *,
    start_attempted: bool,
    test_passed: bool,
    evidence_root: Path = RUNS_ROOT,
    runner_root: Path = RUNNER_CACHE_ROOT,
    worktree_root: Path = ROOT,
) -> dict[str, object]:
    try:
        canonical_data_dir = validate_owned_cleanup_paths(
            run_dir,
            data_dir,
            evidence_root=evidence_root,
            runner_root=runner_root,
            worktree_root=worktree_root,
        )
    except (AcceptanceError, OSError) as error:
        return {
            "name": "postgres-stop",
            "exit_code": 1,
            "shutdown_confirmed": False,
            "owned_data_dir_preserved": data_dir.exists(),
            "error": f"refusing cluster management because owned path validation failed ({type(error).__name__})",
        }
    if not start_attempted:
        return {
            "name": "postgres-stop",
            "exit_code": 0,
            "shutdown_confirmed": False,
            "owned_data_dir_preserved": data_dir.exists(),
            "reason": "cluster start was not attempted",
        }
    stopped = run_step(
        "postgres-stop",
        [
            str(postgres_bin / "pg_ctl.exe"),
            "-D",
            str(canonical_data_dir),
            "-m",
            "fast",
            "-w",
            "stop",
        ],
        environment,
        logs_dir,
        timeout_seconds=180,
    )
    shutdown_confirmed = stopped["exit_code"] == 0
    stopped["shutdown_confirmed"] = shutdown_confirmed
    remove_data = should_remove_data(test_passed, shutdown_confirmed, True)
    if remove_data:
        try:
            deletion_target = validate_owned_cleanup_paths(
                run_dir,
                data_dir,
                evidence_root=evidence_root,
                runner_root=runner_root,
                worktree_root=worktree_root,
            )
        except (AcceptanceError, OSError) as error:
            stopped["exit_code"] = 1
            stopped["owned_data_dir_removed"] = False
            stopped["owned_data_dir_preserved"] = data_dir.exists()
            stopped["error"] = f"refusing data removal after ownership recheck failed ({type(error).__name__})"
            return stopped
        shutil.rmtree(deletion_target)
        stopped["owned_data_dir_removed"] = True
        stopped["owned_data_dir_preserved"] = False
    else:
        stopped["owned_data_dir_removed"] = False
        stopped["owned_data_dir_preserved"] = data_dir.exists()
    return stopped


def write_summary(path: Path, summary: Mapping[str, object]) -> None:
    if not path.parent.is_dir():
        raise AcceptanceError("summary parent directory is not present")
    path.write_text(json.dumps(summary, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--postgres-bin", type=Path, default=DEFAULT_POSTGRES_BIN)
    parser.add_argument("--test-binary", type=Path)
    parser.add_argument("--target-dir", type=Path)
    parser.add_argument("--build-attestation", type=Path, required=True)
    options = parser.parse_args(argv)

    run_id = uuid.uuid4().hex
    run_dir, data_dir, logs_dir = run_paths(run_id)
    if not paths_are_disjoint(RUNNER_CACHE_ROOT, ROOT):
        raise AcceptanceError("external runner evidence root overlaps the Git worktree")
    run_dir, data_dir, logs_dir = create_run_paths(run_id)
    canonical_data_dir = data_dir
    summary_path = run_dir / "acceptance-summary.json"
    summary: dict[str, object] = {
        "phase": "W16-A Canvas SSE device-revocation PostgreSQL acceptance",
        "run_id": run_id,
        "status": "blocked",
        "evidence_path": str(run_dir),
        "summary_path": str(summary_path),
        "test_binary": PG_TEST_BINARY,
        "expected_test": EXPECTED_TEST,
        "database_scope": "fresh PostgreSQL 18 cluster bound only to IPv4 loopback",
        "test_database_role": "postgres superuser in this disposable cluster",
        "authorization_proof_boundary": (
            "The test exercises persisted HTTP authorization and SSE revocation through the production route/hub. "
            "Its PostgreSQL superuser pool does not prove runtime NOBYPASSRLS or production grants."
        ),
        "build_attestation_kind": "unsigned_coordinator_operator_assertion",
        "build_attestation_signature_verified": False,
        "build_attestation_ci_proof_verified": False,
        "build_attestation_boundary": (
            "The runner recomputes current HEAD and source/binary hashes and compares them with the attestation. "
            "The fact that Cargo ran with the stated command/toolchain is a coordinator/operator assertion, "
            "not a cryptographically signed record or independently verified CI proof."
        ),
        "path_cleanup_boundary": (
            "Canonical ownership and reparse checks run before initdb/pg_ctl use of the data path and are repeated "
            "immediately before data deletion. Path-based checks cannot eliminate a same-user TOCTOU race between "
            "validation and the following filesystem/process call; this local utility provides no handle-relative "
            "or descriptor-anchored protection."
        ),
        "test_only_placeholders": {
            "audit": (
                "The disposable schema has a simplified audit table/function; the generic audit_trigger_func is a no-op."
            ),
            "scd2": "The test-only scd_type2_close trigger function is a no-op that returns NEW.",
            "boundary": (
                "These placeholders satisfy migration prerequisites only; they do not prove production audit, SCD2, "
                "RLS, or grant behavior."
            ),
        },
        "replay_offset_migration_applied": False,
        "database_url_recorded": False,
        "steps": [],
    }
    steps: list[dict[str, object]] = summary["steps"]  # type: ignore[assignment]
    environment = sanitized_environment()
    port: int | None = None
    reservation: socket.socket | None = None
    start_attempted = False
    test_passed = False
    final_status = 1
    try:
        if not paths_are_disjoint(RUNNER_CACHE_ROOT, ROOT) or not is_inside(run_dir, RUNS_ROOT):
            raise AcceptanceError("external run paths failed the ownership check")
        tools = postgres_tool_paths(options.postgres_bin)
        plan = migration_plan()
        validate_migration_plan(plan)
        if options.test_binary is not None:
            binary, resolved_target = validate_test_binary(options.test_binary, options.target_dir)
        else:
            resolved_target = options.target_dir or ROOT / "target"
            binary = find_test_binary(resolved_target)
        summary["test_binary_path"] = str(binary)
        summary["cargo_target_dir"] = str(resolved_target.resolve())
        build_attestation = load_build_attestation(options.build_attestation, binary)
        summary["build_attestation"] = build_attestation

        version = run_step(
            "postgres-version",
            [str(tools["postgres.exe"]), "--version"],
            environment,
            logs_dir,
            timeout_seconds=30,
        )
        steps.append(version)
        version_output = str(version.get("output_tail", ""))
        version_match = re.search(r"PostgreSQL\)\s+(\d+)", version_output)
        summary["postgres_major_version"] = version_match.group(1) if version_match else None
        if version["exit_code"] != 0 or version_match is None or version_match.group(1) != "18":
            raise AcceptanceError("PostgreSQL 18 was not confirmed by the trusted local binaries")

        port, reservation = reserve_loopback_port()
        summary["loopback_port"] = port
        canonical_data_dir = validate_owned_cleanup_paths(run_dir, data_dir)
        summary["postgres_data_directory"] = str(canonical_data_dir)
        init = run_step(
            "postgres-init",
            [
                str(tools["initdb.exe"]),
                "-D",
                str(canonical_data_dir),
                "-U",
                "postgres",
                "--auth=trust",
                "--no-instructions",
            ],
            environment,
            logs_dir,
            timeout_seconds=300,
        )
        steps.append(init)
        if init["exit_code"] != 0:
            raise AcceptanceError("owned PostgreSQL cluster initialization failed")

        reservation.close()
        reservation = None
        canonical_data_dir = validate_owned_cleanup_paths(run_dir, data_dir)
        summary["postgres_data_directory"] = str(canonical_data_dir)
        start_attempted = True
        start = run_step(
            "postgres-start",
            [
                str(tools["pg_ctl.exe"]),
                "-D",
                str(canonical_data_dir),
                "-l",
                str(canonical_data_dir / "server.log"),
                "-o",
                f"-h 127.0.0.1 -p {port}",
                "-w",
                "start",
            ],
            environment,
            logs_dir,
            timeout_seconds=240,
            capture_output=False,
        )
        steps.append(start)
        if start["exit_code"] != 0:
            raise AcceptanceError("owned PostgreSQL cluster failed to start on IPv4 loopback")

        ready = run_step(
            "postgres-ready-and-version",
            psql_command(tools["psql.exe"].parent, port, "postgres", sql="SHOW server_version_num;"),
            environment,
            logs_dir,
            timeout_seconds=30,
        )
        steps.append(ready)
        version_number = str(ready.get("output_tail", "")).strip()
        if ready["exit_code"] != 0 or not version_number.startswith("18"):
            raise AcceptanceError("loopback server readiness or PostgreSQL 18 check failed")

        migration_results = apply_migrations(tools["psql.exe"].parent, port, environment, logs_dir, plan)
        steps.extend(migration_results)
        failed_migration = next((step for step in migration_results if step["exit_code"] != 0), None)
        if failed_migration is not None:
            raise AcceptanceError(f"migration/setup step failed closed: {failed_migration['name']}")

        verified_before_execution = load_build_attestation(options.build_attestation, binary)
        if verified_before_execution != build_attestation:
            raise AcceptanceError("build attestation or artifacts changed after runner setup")
        summary["build_attestation"] = verified_before_execution
        test_env = test_environment(environment, port)
        test_result = run_step(
            "canvas-device-auth-pg-exact-test",
            test_binary_command(binary),
            test_env,
            logs_dir,
            timeout_seconds=180,
        )
        steps.append(test_result)
        test_output = str(test_result.get("output_tail", ""))
        accepted, proof_issues = accepted_test_output(test_output)
        test_result["acceptance_output_valid"] = accepted
        test_result["proof_issues"] = proof_issues
        if test_result["exit_code"] != 0 or not accepted:
            raise AcceptanceError("exact Canvas SSE PostgreSQL case failed or its one-test summary was not proven")
        test_passed = True
        summary["status"] = "accepted_local_postgresql_case"
        final_status = 0
    except KeyboardInterrupt:
        summary["status"] = "interrupted"
        summary["failure"] = "runner interrupted before the selected PostgreSQL case was proven"
        final_status = 130
        print("ACCEPTANCE_INTERRUPTED; stopping only this run's cluster", flush=True)
    except Exception as error:
        summary["failure"] = f"{type(error).__name__}: {error}"
        print(f"ACCEPTANCE_BLOCKED {type(error).__name__}: {error}", flush=True)
    finally:
        if reservation is not None:
            reservation.close()
        try:
            if port is not None:
                cleanup = stop_and_cleanup(
                    options.postgres_bin,
                    canonical_data_dir,
                    run_dir,
                    environment,
                    logs_dir,
                    start_attempted=start_attempted,
                    test_passed=test_passed,
                )
            else:
                cleanup = {
                    "name": "postgres-stop",
                    "exit_code": 0,
                    "shutdown_confirmed": False,
                    "owned_data_dir_preserved": data_dir.exists(),
                    "reason": "cluster start was never attempted",
                }
        except Exception as error:
            cleanup = {
                "name": "postgres-stop",
                "exit_code": 1,
                "shutdown_confirmed": False,
                "owned_data_dir_preserved": data_dir.exists(),
                "error": f"cleanup failed ({type(error).__name__})",
            }
        steps.append(cleanup)
        if cleanup["exit_code"] != 0:
            summary["status"] = "failed_cleanup"
            summary["failure"] = "could not confirm shutdown or safely manage this run's data directory"
            final_status = 1
        summary["test_passed"] = test_passed
        summary["final_status"] = final_status
        summary["completed_at_unix"] = time.time()
        try:
            write_summary(summary_path, summary)
            print(f"SUMMARY {summary_path}", flush=True)
            print(f"ACCEPTANCE_STATUS {summary['status']}", flush=True)
        except OSError as error:
            print(f"SUMMARY_WRITE_FAILED {type(error).__name__}", flush=True)
            final_status = 1
    return final_status


if __name__ == "__main__":
    raise SystemExit(main())
