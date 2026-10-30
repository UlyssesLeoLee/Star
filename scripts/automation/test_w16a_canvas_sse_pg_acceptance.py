"""Unit checks for the Canvas SSE runner's fail-closed guards."""

# @cypher schema=1 source_sha256=7d13373fc862c6644f89bca19fa1b2dd15a68c5f0fde212bb06bdae892321690
# MERGE (self:File {path:"scripts/automation/test_w16a_canvas_sse_pg_acceptance.py"})
# MERGE (runner:File {path:"scripts/automation/w16a_canvas_sse_pg_acceptance.py"})
# MERGE (environmentTest:Symbol {id:"scripts/automation/test_w16a_canvas_sse_pg_acceptance.py::test_environment_sanitization_removes_inherited_database_targets",kind:"test"})
# MERGE (generatedUrlTest:Symbol {id:"scripts/automation/test_w16a_canvas_sse_pg_acceptance.py::test_test_environment_sets_only_generated_loopback_url",kind:"test"})
# MERGE (pathTest:Symbol {id:"scripts/automation/test_w16a_canvas_sse_pg_acceptance.py::test_run_paths_are_private_descendants",kind:"test"})
# MERGE (portTest:Symbol {id:"scripts/automation/test_w16a_canvas_sse_pg_acceptance.py::test_port_reservation_is_ipv4_loopback",kind:"test"})
# MERGE (migrationTest:Symbol {id:"scripts/automation/test_w16a_canvas_sse_pg_acceptance.py::test_migration_plan_keeps_required_canvas_order",kind:"test"})
# MERGE (oauthBaseTest:Symbol {id:"scripts/automation/test_w16a_canvas_sse_pg_acceptance.py::test_migration_plan_requires_oauth_base_at_position_zero",kind:"test"})
# MERGE (binaryTest:Symbol {id:"scripts/automation/test_w16a_canvas_sse_pg_acceptance.py::test_binary_command_selects_only_the_exact_ignored_case",kind:"test"})
# MERGE (binaryPathTest:Symbol {id:"scripts/automation/test_w16a_canvas_sse_pg_acceptance.py::test_binary_discovery_rejects_ambiguous_artifacts",kind:"test"})
# MERGE (externalBinaryTest:Symbol {id:"scripts/automation/test_w16a_canvas_sse_pg_acceptance.py::test_external_target_directory_and_binary_are_accepted",kind:"test"})
# MERGE (binaryLayoutTest:Symbol {id:"scripts/automation/test_w16a_canvas_sse_pg_acceptance.py::test_explicit_binary_requires_exact_name_layout_and_target",kind:"test"})
# MERGE (attestationTest:Symbol {id:"scripts/automation/test_w16a_canvas_sse_pg_acceptance.py::test_build_attestation_rejects_head_and_hash_mismatches",kind:"test"})
# MERGE (attestationSizeTest:Symbol {id:"scripts/automation/test_w16a_canvas_sse_pg_acceptance.py::test_oversized_build_attestation_is_rejected_before_json_parse",kind:"test"})
# MERGE (cleanupPathsTest:Symbol {id:"scripts/automation/test_w16a_canvas_sse_pg_acceptance.py::test_cleanup_rejects_replaced_and_reparse_paths",kind:"test"})
# MERGE (cleanupRecheckTest:Symbol {id:"scripts/automation/test_w16a_canvas_sse_pg_acceptance.py::test_cleanup_rechecks_reparse_points_before_delete",kind:"test"})
# MERGE (summaryTest:Symbol {id:"scripts/automation/test_w16a_canvas_sse_pg_acceptance.py::test_summary_proof_requires_one_clean_pass",kind:"test"})
# MERGE (tailTest:Symbol {id:"scripts/automation/test_w16a_canvas_sse_pg_acceptance.py::test_log_tail_is_bounded_and_redacts_urls",kind:"test"})
# MERGE (redactionTest:Symbol {id:"scripts/automation/test_w16a_canvas_sse_pg_acceptance.py::test_log_redaction_rewrites_in_place_and_cleans_temp",kind:"test"})
# MERGE (streamRedactionTest:Symbol {id:"scripts/automation/test_w16a_canvas_sse_pg_acceptance.py::test_redact_stream_handles_cross_chunk_long_url",kind:"test"})
# MERGE (redactionFailureTest:Symbol {id:"scripts/automation/test_w16a_canvas_sse_pg_acceptance.py::test_redaction_failure_suppresses_raw_output_and_cleans_temp",kind:"test"})
# MERGE (liveRedactionTest:Symbol {id:"scripts/automation/test_w16a_canvas_sse_pg_acceptance.py::test_run_step_persists_only_redacted_output_while_producer_is_active",kind:"test"})
# MERGE (timeoutTest:Symbol {id:"scripts/automation/test_w16a_canvas_sse_pg_acceptance.py::test_run_step_kills_timed_out_process_and_keeps_partial_url_redacted",kind:"test"})
# MERGE (cleanupTest:Symbol {id:"scripts/automation/test_w16a_canvas_sse_pg_acceptance.py::test_failed_shutdown_preserves_only_owned_data",kind:"test"})
# MERGE (cleanupDecisionTest:Symbol {id:"scripts/automation/test_w16a_canvas_sse_pg_acceptance.py::test_data_cleanup_requires_pass_shutdown_and_ownership",kind:"test"})
# MERGE (self)-[:DEFINES]->(environmentTest),(self)-[:DEFINES]->(generatedUrlTest),(self)-[:DEFINES]->(pathTest),(self)-[:DEFINES]->(portTest),(self)-[:DEFINES]->(migrationTest),(self)-[:DEFINES]->(binaryTest),(self)-[:DEFINES]->(binaryPathTest),(self)-[:DEFINES]->(externalBinaryTest),(self)-[:DEFINES]->(binaryLayoutTest),(self)-[:DEFINES]->(attestationTest),(self)-[:DEFINES]->(cleanupPathsTest),(self)-[:DEFINES]->(cleanupRecheckTest),(self)-[:DEFINES]->(summaryTest),(self)-[:DEFINES]->(tailTest),(self)-[:DEFINES]->(redactionTest),(self)-[:DEFINES]->(streamRedactionTest),(self)-[:DEFINES]->(redactionFailureTest),(self)-[:DEFINES]->(cleanupTest),(self)-[:DEFINES]->(cleanupDecisionTest)
# MERGE (self)-[:DEFINES]->(oauthBaseTest),(self)-[:DEFINES]->(attestationSizeTest),(self)-[:DEFINES]->(liveRedactionTest),(self)-[:DEFINES]->(timeoutTest)
# MERGE (environmentTest)-[:TESTS]->(runner),(environmentTest)-[:CALLS]->(:Symbol {id:"scripts/automation/w16a_canvas_sse_pg_acceptance.py::sanitized_environment",kind:"function"})
# MERGE (generatedUrlTest)-[:TESTS]->(runner),(generatedUrlTest)-[:CALLS]->(:Symbol {id:"scripts/automation/w16a_canvas_sse_pg_acceptance.py::test_environment",kind:"function"})
# MERGE (pathTest)-[:TESTS]->(runner),(pathTest)-[:CALLS]->(:Symbol {id:"scripts/automation/w16a_canvas_sse_pg_acceptance.py::run_paths",kind:"function"}),(pathTest)-[:CALLS]->(:Symbol {id:"scripts/automation/w16a_canvas_sse_pg_acceptance.py::is_inside",kind:"function"})
# MERGE (portTest)-[:TESTS]->(runner),(portTest)-[:CALLS]->(:Symbol {id:"scripts/automation/w16a_canvas_sse_pg_acceptance.py::reserve_loopback_port",kind:"function"})
# MERGE (migrationTest)-[:TESTS]->(runner),(migrationTest)-[:CALLS]->(:Symbol {id:"scripts/automation/w16a_canvas_sse_pg_acceptance.py::migration_plan",kind:"function"})
# MERGE (oauthBaseTest)-[:TESTS]->(runner),(oauthBaseTest)-[:CALLS]->(:Symbol {id:"scripts/automation/w16a_canvas_sse_pg_acceptance.py::validate_migration_plan",kind:"function"})
# MERGE (binaryTest)-[:TESTS]->(runner),(binaryTest)-[:CALLS]->(:Symbol {id:"scripts/automation/w16a_canvas_sse_pg_acceptance.py::test_binary_command",kind:"function"})
# MERGE (binaryPathTest)-[:TESTS]->(runner),(binaryPathTest)-[:CALLS]->(:Symbol {id:"scripts/automation/w16a_canvas_sse_pg_acceptance.py::find_test_binary",kind:"function"})
# MERGE (externalBinaryTest)-[:TESTS]->(runner),(externalBinaryTest)-[:CALLS]->(:Symbol {id:"scripts/automation/w16a_canvas_sse_pg_acceptance.py::find_test_binary",kind:"function"}),(externalBinaryTest)-[:CALLS]->(:Symbol {id:"scripts/automation/w16a_canvas_sse_pg_acceptance.py::validate_test_binary",kind:"function"})
# MERGE (binaryLayoutTest)-[:TESTS]->(runner),(binaryLayoutTest)-[:CALLS]->(:Symbol {id:"scripts/automation/w16a_canvas_sse_pg_acceptance.py::validate_test_binary",kind:"function"})
# MERGE (attestationTest)-[:TESTS]->(runner),(attestationTest)-[:CALLS]->(:Symbol {id:"scripts/automation/w16a_canvas_sse_pg_acceptance.py::validate_build_attestation",kind:"function"})
# MERGE (attestationSizeTest)-[:TESTS]->(runner),(attestationSizeTest)-[:CALLS]->(:Symbol {id:"scripts/automation/w16a_canvas_sse_pg_acceptance.py::load_build_attestation",kind:"function"})
# MERGE (cleanupPathsTest)-[:TESTS]->(runner),(cleanupPathsTest)-[:CALLS]->(:Symbol {id:"scripts/automation/w16a_canvas_sse_pg_acceptance.py::validate_owned_cleanup_paths",kind:"function"})
# MERGE (cleanupRecheckTest)-[:TESTS]->(runner),(cleanupRecheckTest)-[:CALLS]->(:Symbol {id:"scripts/automation/w16a_canvas_sse_pg_acceptance.py::stop_and_cleanup",kind:"function"})
# MERGE (summaryTest)-[:TESTS]->(runner),(summaryTest)-[:CALLS]->(:Symbol {id:"scripts/automation/w16a_canvas_sse_pg_acceptance.py::accepted_test_output",kind:"function"})
# MERGE (tailTest)-[:TESTS]->(runner),(tailTest)-[:CALLS]->(:Symbol {id:"scripts/automation/w16a_canvas_sse_pg_acceptance.py::read_tail",kind:"function"})
# MERGE (redactionTest)-[:TESTS]->(runner),(redactionTest)-[:CALLS]->(:Symbol {id:"scripts/automation/w16a_canvas_sse_pg_acceptance.py::redact_log_file",kind:"function"})
# MERGE (streamRedactionTest)-[:TESTS]->(runner),(streamRedactionTest)-[:CALLS]->(:Symbol {id:"scripts/automation/w16a_canvas_sse_pg_acceptance.py::redact_stream",kind:"function"})
# MERGE (redactionFailureTest)-[:TESTS]->(runner),(redactionFailureTest)-[:CALLS]->(:Symbol {id:"scripts/automation/w16a_canvas_sse_pg_acceptance.py::run_step",kind:"function"})
# MERGE (liveRedactionTest)-[:TESTS]->(runner),(liveRedactionTest)-[:CALLS]->(:Symbol {id:"scripts/automation/w16a_canvas_sse_pg_acceptance.py::run_step",kind:"function"})
# MERGE (timeoutTest)-[:TESTS]->(runner),(timeoutTest)-[:CALLS]->(:Symbol {id:"scripts/automation/w16a_canvas_sse_pg_acceptance.py::run_step",kind:"function"})
# MERGE (cleanupTest)-[:TESTS]->(runner),(cleanupTest)-[:CALLS]->(:Symbol {id:"scripts/automation/w16a_canvas_sse_pg_acceptance.py::stop_and_cleanup",kind:"function"})
# MERGE (cleanupDecisionTest)-[:TESTS]->(runner),(cleanupDecisionTest)-[:CALLS]->(:Symbol {id:"scripts/automation/w16a_canvas_sse_pg_acceptance.py::should_remove_data",kind:"function"})
# @endcypher

from __future__ import annotations

import queue
import tempfile
import threading
import unittest
from contextlib import redirect_stdout
from io import BytesIO, StringIO
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

from scripts.automation import w16a_canvas_sse_pg_acceptance as acceptance


class W16ACanvasSseRunnerTests(unittest.TestCase):
    def test_environment_sanitization_removes_inherited_database_targets(self) -> None:
        cleaned = acceptance.sanitized_environment(
            {
                "PATH": "safe-path",
                "PGHOST": "remote-host",
                "PGPORT": "5432",
                "PGPASSWORD": "secret-placeholder",
                "DATABASE_URL": "postgres://remote",
                "TEST_DATABASE_URL": "postgres://remote-test",
                "STAR_W16_DEVICE_AUTH_TEST_DATABASE_URL": "postgres://remote-w16",
                "SQLX_OFFLINE": "true",
            }
        )
        self.assertEqual(cleaned["PATH"], "safe-path")
        self.assertEqual(cleaned["CARGO_TERM_COLOR"], "never")
        self.assertEqual(cleaned["NO_COLOR"], "1")
        self.assertFalse(any(key.upper().startswith("PG") for key in cleaned))
        self.assertFalse(any("DATABASE_URL" in key.upper() for key in cleaned))
        self.assertNotIn("SQLX_OFFLINE", cleaned)

    def test_test_environment_sets_only_generated_loopback_url(self) -> None:
        env = acceptance.test_environment(
            {
                "PATH": "safe-path",
                "DATABASE_URL": "postgres://remote",
                acceptance.DATABASE_URL_ENV: "postgres://another-target",
            },
            49123,
        )
        self.assertEqual(
            env[acceptance.DATABASE_URL_ENV],
            "postgres://postgres@127.0.0.1:49123/w16a_canvas_sse_acceptance",
        )
        self.assertEqual(
            [key for key in env if "DATABASE_URL" in key.upper()],
            [acceptance.DATABASE_URL_ENV],
        )
        with self.assertRaises(ValueError):
            acceptance.generated_database_url(70000)

    def test_run_paths_are_private_descendants(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary) / "external-runs"
            run_dir, data_dir, logs_dir = acceptance.create_run_paths("a" * 32, root)
            self.assertTrue(acceptance.is_inside(run_dir, root))
            self.assertTrue(acceptance.is_inside(data_dir, run_dir))
            self.assertTrue(acceptance.is_inside(logs_dir, run_dir))
            self.assertFalse(acceptance.is_inside(root, root))
            with self.assertRaises(ValueError):
                acceptance.run_paths("../outside", root)
            with self.assertRaises(FileExistsError):
                acceptance.run_paths("a" * 32, root)

    def test_port_reservation_is_ipv4_loopback(self) -> None:
        port, reservation = acceptance.reserve_loopback_port()
        try:
            address = reservation.getsockname()
            self.assertEqual(address[0], "127.0.0.1")
            self.assertEqual(address[1], port)
            self.assertGreaterEqual(port, 1)
            self.assertLessEqual(port, 65535)
        finally:
            reservation.close()

    def test_migration_plan_keeps_required_canvas_order(self) -> None:
        plan = acceptance.migration_plan()
        filenames = [path.name for _, path in plan]
        helper = acceptance.directory_helper()
        helper_files = [filename for _, filename in helper.directory_migration_plan(run_binding=True)]
        self.assertEqual(filenames[1 : 1 + len(helper_files)], helper_files)
        self.assertGreater(filenames.index(acceptance.WORKTREE_PHASE_2D.name), len(helper_files))
        self.assertLess(
            filenames.index(acceptance.WORKTREE_PHASE_2D.name),
            filenames.index(acceptance.CANVAS_PHASE_3.name),
        )
        self.assertLess(
            filenames.index(acceptance.CANVAS_PHASE_3.name),
            filenames.index(acceptance.W16_MIGRATION.name),
        )
        self.assertLess(
            filenames.index(acceptance.W16_MIGRATION.name),
            filenames.index(acceptance.CANVAS_WAKE_MIGRATION.name),
        )
        self.assertNotIn("2026-10-05-worktree-group-canvas-stream-offset.sql", filenames)
        acceptance.validate_migration_plan(plan)

    def test_migration_plan_requires_oauth_base_at_position_zero(self) -> None:
        plan = acceptance.migration_plan()
        self.assertEqual(plan[0], ("postgres-oauth-base", acceptance.BASE_MIGRATION))
        acceptance.validate_migration_plan(plan)

        with self.assertRaisesRegex(acceptance.AcceptanceError, "OAuth base migration"):
            acceptance.validate_migration_plan(plan[1:] + plan[:1])
        wrong_name = [("not-oauth-base", acceptance.BASE_MIGRATION), *plan[1:]]
        with self.assertRaisesRegex(acceptance.AcceptanceError, "OAuth base migration"):
            acceptance.validate_migration_plan(wrong_name)

    def test_binary_command_selects_only_the_exact_ignored_case(self) -> None:
        binary = Path("target/debug/deps/canvas_device_auth_pg-deadbeef.exe")
        command = acceptance.test_binary_command(binary)
        self.assertEqual(
            command,
            [
                str(binary),
                acceptance.EXPECTED_TEST,
                "--ignored",
                "--exact",
                "--test-threads=1",
            ],
        )
        self.assertNotIn("cargo", [part.lower() for part in command])

    def test_binary_discovery_rejects_ambiguous_artifacts(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            deps = Path(temporary) / "target" / "debug" / "deps"
            deps.mkdir(parents=True)
            one = deps / "canvas_device_auth_pg-0000000000001111.exe"
            one.write_bytes(b"test binary placeholder")
            target_root = Path(temporary) / "target"
            self.assertEqual(acceptance.find_test_binary(target_root), one.resolve())
            two = deps / "canvas_device_auth_pg-0000000000002222.exe"
            two.write_bytes(b"second placeholder")
            with self.assertRaises(acceptance.AcceptanceError):
                acceptance.find_test_binary(target_root)

    def test_external_target_directory_and_binary_are_accepted(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            target_root = Path(temporary) / "cargo-cache" / "target"
            deps = target_root / "debug" / "deps"
            deps.mkdir(parents=True)
            binary = deps / "canvas_device_auth_pg-dd3246f8f37904d5.exe"
            binary.write_bytes(b"compiled test binary placeholder")

            self.assertEqual(acceptance.find_test_binary(target_root), binary.resolve())
            self.assertEqual(
                acceptance.validate_test_binary(binary, target_root),
                (binary.resolve(), target_root.resolve()),
            )
            self.assertEqual(acceptance.validate_test_binary(binary)[1], target_root.resolve())

    def test_explicit_binary_requires_exact_name_layout_and_target(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            target_root = Path(temporary) / "target"
            deps = target_root / "debug" / "deps"
            deps.mkdir(parents=True)
            binary = deps / "canvas_device_auth_pg-dd3246f8f37904d5.exe"
            binary.write_bytes(b"compiled test binary placeholder")
            other_target = Path(temporary) / "other-target"

            with self.assertRaises(acceptance.AcceptanceError):
                acceptance.validate_test_binary(binary, other_target)

            wrong_name = deps / "canvas_device_auth_pg-not-a-cargo-hash.exe"
            wrong_name.write_bytes(b"not the requested Cargo target")
            with self.assertRaises(acceptance.AcceptanceError):
                acceptance.validate_test_binary(wrong_name, target_root)

    def test_build_attestation_rejects_head_and_hash_mismatches(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            binary = Path(temporary) / "canvas-device-test.exe"
            binary.write_bytes(b"candidate test executable")
            source = Path(temporary) / "canvas_device_auth_pg.rs"
            source.write_bytes(b"candidate test source")
            expected_head = "a" * 40
            valid_record = {
                "head": expected_head,
                "canvas_test_source_sha256": acceptance.sha256_file(source),
                "test_binary_sha256": acceptance.sha256_file(binary),
                "rustc_version": acceptance.EXPECTED_RUSTC_VERSION,
                "build_command": acceptance.EXPECTED_BUILD_COMMAND,
                "build_exit_code": 0,
                "build_elapsed_seconds": 18.8,
            }
            self.assertEqual(
                acceptance.validate_build_attestation(valid_record, binary, expected_head, source),
                {
                    "head": expected_head,
                    "canvas_test_source_sha256": acceptance.sha256_file(source),
                    "test_binary_sha256": acceptance.sha256_file(binary),
                    "rustc_version": acceptance.EXPECTED_RUSTC_VERSION,
                    "build_command": acceptance.EXPECTED_BUILD_COMMAND,
                    "build_exit_code": 0,
                    "build_elapsed_seconds": 18.8,
                },
            )

            mismatches = (
                ({**valid_record, "head": "b" * 40}, "head"),
                ({**valid_record, "canvas_test_source_sha256": "0" * 64}, "source"),
                ({**valid_record, "test_binary_sha256": "0" * 64}, "binary"),
                ({**valid_record, "build_command": "cargo test --no-run"}, "command"),
                ({**valid_record, "build_exit_code": 1}, "build exit"),
                ({**valid_record, "build_elapsed_seconds": float("nan")}, "build duration"),
            )
            for invalid_record, _label in mismatches:
                with self.subTest(field=_label), self.assertRaises(acceptance.AcceptanceError):
                    acceptance.validate_build_attestation(invalid_record, binary, expected_head, source)

    def test_oversized_build_attestation_is_rejected_before_json_parse(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            deps = root / "target" / "debug" / "deps"
            deps.mkdir(parents=True)
            binary = deps / "canvas_device_auth_pg-dd3246f8f37904d5.exe"
            binary.write_bytes(b"compiled test binary placeholder")
            attestation = root / "operator-attestation.json"
            attestation.write_bytes(b"{" + b" " * acceptance.MAX_BUILD_ATTESTATION_BYTES)

            with patch.object(acceptance.json, "loads") as parse_json:
                with self.assertRaisesRegex(acceptance.AcceptanceError, "64 KiB"):
                    acceptance.load_build_attestation(attestation, binary)
                parse_json.assert_not_called()

    def test_summary_proof_requires_one_clean_pass(self) -> None:
        output = (
            "running 1 test\n"
            f"test {acceptance.EXPECTED_TEST} ... ok\n"
            "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.02s\n"
        )
        self.assertEqual(acceptance.accepted_test_output(output), (True, []))
        ignored = output.replace("1 passed; 0 failed; 0 ignored", "0 passed; 0 failed; 1 ignored")
        accepted, issues = acceptance.accepted_test_output(ignored)
        self.assertFalse(accepted)
        self.assertTrue(issues)
        extra_test = output.replace("running 1 test", "running 2 tests").replace(
            "4 filtered out", "3 filtered out"
        )
        self.assertFalse(acceptance.accepted_test_output(extra_test)[0])

    def test_log_tail_is_bounded_and_redacts_urls(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "large.log"
            path.write_bytes(b"x" * (acceptance.MAX_LOG_TAIL_BYTES + 32) + b" postgres://user:pass@127.0.0.1/db tail")
            tail = acceptance.read_tail(path)
        self.assertLessEqual(len(tail.encode("utf-8")), acceptance.MAX_LOG_TAIL_BYTES)
        self.assertNotIn("postgres://user:pass", tail)
        self.assertIn("<redacted-database-url>", tail)

    def test_redact_stream_handles_cross_chunk_long_url(self) -> None:
        secret = b"POSTGRESQL://" + b"x" * (acceptance.REDACTION_CHUNK_BYTES * 2 + 9)
        source = BytesIO(b"before " + secret + b" tail\n")
        destination = BytesIO()
        acceptance.redact_stream(source, destination, chunk_size=7)
        self.assertEqual(destination.getvalue(), b"before <redacted-database-url> tail\n")

    def test_log_redaction_rewrites_in_place_and_cleans_temp(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "postgres-start.log"
            raw_url = b"postgres://operator:private@127.0.0.1/database"
            path.write_bytes(b"startup " + raw_url + b"\nready\n")
            with patch.object(Path, "replace", side_effect=AssertionError("log paths must not be replaced")):
                acceptance.redact_log_file(path)

            redacted = path.read_bytes()
            self.assertNotIn(raw_url, redacted)
            self.assertIn(b"<redacted-database-url>", redacted)
            self.assertEqual(list(path.parent.glob(f".{path.name}.*.redacted.tmp")), [])

    def test_redaction_failure_suppresses_raw_output_and_cleans_temp(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            logs_dir = Path(temporary) / "logs"
            raw_url = "postgres://operator:private@127.0.0.1/database"
            stdout = StringIO()
            process = SimpleNamespace(
                stdout=BytesIO(f"DB target: {raw_url}\n".encode()),
                wait=lambda timeout=None: 1,
                poll=lambda: 1,
                kill=lambda: None,
            )

            with (
                patch.object(acceptance.subprocess, "Popen", return_value=process),
                patch.object(acceptance.DatabaseUrlRedactor, "feed", side_effect=OSError(raw_url)),
                redirect_stdout(stdout),
            ):
                with self.assertRaises(acceptance.AcceptanceError) as error:
                    acceptance.run_step("redaction-failure", ["fake-command"], {}, logs_dir, 5)

            log_path = logs_dir / "redaction-failure.log"
            self.assertNotIn(raw_url, stdout.getvalue())
            self.assertNotIn(raw_url, str(error.exception))
            self.assertNotIn(raw_url.encode(), log_path.read_bytes())
            self.assertEqual(list(logs_dir.glob(f".{log_path.name}.*.redacted.tmp")), [])

    def test_run_step_persists_only_redacted_output_while_producer_is_active(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            logs_dir = Path(temporary) / "logs"
            log_path = logs_dir / "live-redaction.log"
            producer_chunks: queue.Queue[bytes | None] = queue.Queue()
            release_producer = threading.Event()
            producer_paused = threading.Event()
            producer_finished = threading.Event()
            popen_called = threading.Event()
            second_chunk_flushed = threading.Event()
            flush_count = 0
            flush_lock = threading.Lock()
            raw_url = b"postgres://operator:private@127.0.0.1/database"

            class MockProducerPipe:
                def read(self, _size: int) -> bytes:
                    chunk = producer_chunks.get()
                    return b"" if chunk is None else chunk

                def close(self) -> None:
                    producer_chunks.put(None)

            pipe = MockProducerPipe()

            class MockProcess:
                stdout = pipe

                def wait(self, timeout: float | None = None) -> int:
                    if not producer_finished.wait(timeout):
                        raise acceptance.subprocess.TimeoutExpired("mock-process", timeout)
                    return 0

                def poll(self) -> int | None:
                    return 0 if producer_finished.is_set() else None

                def kill(self) -> None:
                    release_producer.set()
                    producer_chunks.put(None)

            class ObservedWriter:
                def __init__(self, file_object) -> None:
                    self.file_object = file_object

                def __enter__(self):
                    return self

                def __exit__(self, exc_type, exc_value, traceback):
                    self.file_object.close()

                def write(self, content: bytes) -> int:
                    return self.file_object.write(content)

                def flush(self) -> None:
                    nonlocal flush_count
                    self.file_object.flush()
                    with flush_lock:
                        flush_count += 1
                        if flush_count >= 2:
                            second_chunk_flushed.set()

                def fileno(self) -> int:
                    return self.file_object.fileno()

                def seek(self, offset: int, whence: int = 0) -> int:
                    return self.file_object.seek(offset, whence)

                def truncate(self, size: int | None = None) -> int:
                    return self.file_object.truncate(size)

            original_open = Path.open

            def observed_open(path: Path, mode: str = "r", *args, **kwargs):
                file_object = original_open(path, mode, *args, **kwargs)
                if path == log_path and mode == "xb":
                    return ObservedWriter(file_object)
                return file_object

            process = MockProcess()
            step_result: list[dict[str, object]] = []
            step_errors: list[BaseException] = []

            def mocked_popen(*_args, **_kwargs):
                popen_called.set()
                return process

            def emit_output() -> None:
                producer_chunks.put(b"DB target: " + raw_url + b" ")
                producer_chunks.put(b"progress-one\n")
                producer_paused.set()
                release_producer.wait(5)
                producer_chunks.put(b"still emitting\n")
                producer_chunks.put(None)
                producer_finished.set()

            def run_step_thread() -> None:
                try:
                    step_result.append(acceptance.run_step("live-redaction", ["mock"], {}, logs_dir, 10))
                except BaseException as error:
                    step_errors.append(error)

            producer = threading.Thread(target=emit_output, daemon=True)
            worker = threading.Thread(target=run_step_thread, daemon=True)
            output_console = StringIO()
            try:
                with (
                    patch.object(acceptance.subprocess, "Popen", side_effect=mocked_popen),
                    patch.object(Path, "open", new=observed_open),
                    redirect_stdout(output_console),
                ):
                    worker.start()
                    self.assertTrue(popen_called.wait(2))
                    producer.start()
                    self.assertTrue(producer_paused.wait(2))
                    self.assertTrue(second_chunk_flushed.wait(2))
                    retained_while_active = log_path.read_bytes()
                    self.assertFalse(producer_finished.is_set())
                    self.assertNotIn(raw_url, retained_while_active)
                    self.assertIn(b"<redacted-database-url>", retained_while_active)
                    self.assertIn(b"progress-one", retained_while_active)
                    release_producer.set()
                    producer.join(3)
                    worker.join(3)
            finally:
                release_producer.set()
                if producer.ident is not None:
                    producer.join(3)
                if worker.ident is not None:
                    worker.join(3)

            self.assertFalse(producer.is_alive())
            self.assertFalse(worker.is_alive())
            self.assertEqual(step_errors, [])
            self.assertEqual(step_result[0]["exit_code"], 0)
            self.assertNotIn(raw_url.decode(), output_console.getvalue())
            self.assertNotIn(raw_url, log_path.read_bytes())

    def test_run_step_kills_timed_out_process_and_keeps_partial_url_redacted(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            logs_dir = Path(temporary) / "logs"
            release_reader = threading.Event()
            reader_started = threading.Event()
            killed = threading.Event()
            raw_prefix = b"postgres://operator:secret@127.0.0.1/database"

            class TimeoutPipe:
                def __init__(self) -> None:
                    self.first_read = True

                def read(self, _size: int) -> bytes:
                    if self.first_read:
                        self.first_read = False
                        return raw_prefix
                    reader_started.set()
                    release_reader.wait(5)
                    return b""

                def close(self) -> None:
                    release_reader.set()

            pipe = TimeoutPipe()

            class TimeoutProcess:
                stdout = pipe

                def wait(self, timeout: float | None = None) -> int:
                    if not killed.wait(timeout):
                        raise acceptance.subprocess.TimeoutExpired("mock-timeout", timeout)
                    return -9

                def poll(self) -> int | None:
                    return -9 if killed.is_set() else None

                def kill(self) -> None:
                    killed.set()
                    release_reader.set()

            process = TimeoutProcess()
            with patch.object(acceptance.subprocess, "Popen", return_value=process):
                result = acceptance.run_step("timeout", ["mock"], {}, logs_dir, 1)

            retained = (logs_dir / "timeout.log").read_bytes()
            self.assertTrue(reader_started.is_set())
            self.assertTrue(killed.is_set())
            self.assertEqual(result["exit_code"], 124)
            self.assertIn("timed out", result["error"])
            self.assertNotIn(raw_prefix, retained)
            self.assertIn(b"<redacted-database-url>", retained)

    def test_failed_shutdown_preserves_only_owned_data(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            temporary_root = Path(temporary)
            runner_root = temporary_root / "external-evidence"
            evidence_root = runner_root / "runs"
            worktree_root = temporary_root / "repo"
            evidence_root.mkdir(parents=True)
            worktree_root.mkdir()
            run_dir = evidence_root / ("d" * 32)
            data_dir = run_dir / "postgres-data"
            lexical_data_dir = run_dir.parent / run_dir.name / ".." / run_dir.name / "postgres-data"
            logs_dir = run_dir / "logs"
            data_dir.mkdir(parents=True)
            logs_dir.mkdir()
            self.assertNotEqual(str(lexical_data_dir), str(data_dir.resolve()))
            with patch.object(
                acceptance,
                "run_step",
                return_value={"name": "postgres-stop", "exit_code": 1, "output_tail": "stop failed"},
            ) as stop_step:
                result = acceptance.stop_and_cleanup(
                    Path(temporary),
                    lexical_data_dir,
                    run_dir,
                    {},
                    logs_dir,
                    start_attempted=True,
                    test_passed=False,
                    evidence_root=evidence_root,
                    runner_root=runner_root,
                    worktree_root=worktree_root,
                )
            stop_command = stop_step.call_args.args[1]
            data_flag_index = stop_command.index("-D")
            self.assertEqual(stop_command[data_flag_index + 1], str(data_dir.resolve()))
            self.assertFalse(result["shutdown_confirmed"])
            self.assertTrue(result["owned_data_dir_preserved"])
            self.assertTrue(data_dir.is_dir())

    def test_data_cleanup_requires_pass_shutdown_and_ownership(self) -> None:
        self.assertTrue(acceptance.should_remove_data(True, True, True))
        self.assertFalse(acceptance.should_remove_data(False, True, True))
        self.assertFalse(acceptance.should_remove_data(True, False, True))
        self.assertFalse(acceptance.should_remove_data(True, True, False))

    def test_cleanup_rejects_replaced_and_reparse_paths(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            temp_root = Path(temporary)
            runner_root = temp_root / "external-evidence"
            evidence_root = runner_root / "runs"
            worktree_root = temp_root / "repo"
            evidence_root.mkdir(parents=True)
            worktree_root.mkdir()
            run_dir = evidence_root / ("c" * 32)
            data_dir = run_dir / "postgres-data"
            data_dir.mkdir(parents=True)
            (run_dir / "logs").mkdir()

            self.assertEqual(
                acceptance.validate_owned_cleanup_paths(
                    run_dir,
                    data_dir,
                    evidence_root=evidence_root,
                    runner_root=runner_root,
                    worktree_root=worktree_root,
                ),
                data_dir.resolve(),
            )
            replacement = run_dir / "replacement-data"
            replacement.mkdir()
            with self.assertRaises(acceptance.AcceptanceError):
                acceptance.validate_owned_cleanup_paths(
                    run_dir,
                    replacement,
                    evidence_root=evidence_root,
                    runner_root=runner_root,
                    worktree_root=worktree_root,
                )

            original_detector = acceptance.has_reparse_point

            def reparse_in_run_tree(path: Path, *, recursive: bool = False) -> bool:
                if path == run_dir and recursive:
                    return True
                return original_detector(path, recursive=recursive)

            with patch.object(acceptance, "has_reparse_point", side_effect=reparse_in_run_tree):
                with self.assertRaises(acceptance.AcceptanceError):
                    acceptance.validate_owned_cleanup_paths(
                        run_dir,
                        data_dir,
                        evidence_root=evidence_root,
                        runner_root=runner_root,
                        worktree_root=worktree_root,
                    )

            with self.assertRaises(acceptance.AcceptanceError):
                acceptance.validate_owned_cleanup_paths(
                    run_dir,
                    data_dir,
                    evidence_root=evidence_root,
                    runner_root=runner_root,
                    worktree_root=runner_root,
                )

    def test_cleanup_rechecks_reparse_points_before_delete(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            temporary_root = Path(temporary)
            runner_root = temporary_root / "external-evidence"
            evidence_root = runner_root / "runs"
            worktree_root = temporary_root / "repo"
            evidence_root.mkdir(parents=True)
            worktree_root.mkdir()
            run_dir = evidence_root / ("e" * 32)
            data_dir = run_dir / "postgres-data"
            logs_dir = run_dir / "logs"
            data_dir.mkdir(parents=True)
            logs_dir.mkdir()
            original_detector = acceptance.has_reparse_point
            run_tree_checks = 0

            def inserted_reparse_point(path: Path, *, recursive: bool = False) -> bool:
                nonlocal run_tree_checks
                if path == run_dir and recursive:
                    run_tree_checks += 1
                    return run_tree_checks == 2
                return original_detector(path, recursive=recursive)

            with (
                patch.object(acceptance, "has_reparse_point", side_effect=inserted_reparse_point),
                patch.object(
                    acceptance,
                    "run_step",
                    return_value={"name": "postgres-stop", "exit_code": 0, "output_tail": "stopped"},
                ) as stop_step,
            ):
                result = acceptance.stop_and_cleanup(
                    Path(temporary),
                    data_dir,
                    run_dir,
                    {},
                    logs_dir,
                    start_attempted=True,
                    test_passed=True,
                    evidence_root=evidence_root,
                    runner_root=runner_root,
                    worktree_root=worktree_root,
                )

            self.assertEqual(run_tree_checks, 2)
            self.assertEqual(stop_step.call_count, 1)
            self.assertEqual(result["exit_code"], 1)
            self.assertFalse(result["owned_data_dir_removed"])
            self.assertTrue(result["owned_data_dir_preserved"])
            self.assertTrue(data_dir.is_dir())


if __name__ == "__main__":
    unittest.main()
