"""Unit tests for fail-closed W16 HTTP PostgreSQL acceptance runner guards."""

# @cypher schema=1 source_sha256=2bad75772a52da7d73b45cab57c3215332a87d91e017db12aeb4abff391dc1fd
# MERGE (self:File {path:"scripts/automation/test_w16a_device_revocation_http_pg_acceptance.py"})
# MERGE (tests:Type {id:"scripts/automation/test_w16a_device_revocation_http_pg_acceptance.py::RunnerGuardTests",kind:"test_class"})
# MERGE (envTest:Symbol {id:"scripts/automation/test_w16a_device_revocation_http_pg_acceptance.py::RunnerGuardTests.test_environment_sanitization_removes_external_targets",kind:"test"})
# MERGE (urlTest:Symbol {id:"scripts/automation/test_w16a_device_revocation_http_pg_acceptance.py::RunnerGuardTests.test_urls_are_loopback_and_use_distinct_login_roles",kind:"test"})
# MERGE (outputTest:Symbol {id:"scripts/automation/test_w16a_device_revocation_http_pg_acceptance.py::RunnerGuardTests.test_output_requires_all_named_cases",kind:"test"})
# MERGE (commandTest:Symbol {id:"scripts/automation/test_w16a_device_revocation_http_pg_acceptance.py::RunnerGuardTests.test_command_is_locked_and_bounded",kind:"test"})
# MERGE (pathTest:Symbol {id:"scripts/automation/test_w16a_device_revocation_http_pg_acceptance.py::RunnerGuardTests.test_private_path_guard",kind:"test"})
# MERGE (cleanupTest:Symbol {id:"scripts/automation/test_w16a_device_revocation_http_pg_acceptance.py::RunnerGuardTests.test_unstarted_partial_cluster_is_removed_only_inside_cache",kind:"test"})
# MERGE (stoppedTest:Symbol {id:"scripts/automation/test_w16a_device_revocation_http_pg_acceptance.py::RunnerGuardTests.test_status_three_removes_stopped_cluster",kind:"test"})
# MERGE (unknownTest:Symbol {id:"scripts/automation/test_w16a_device_revocation_http_pg_acceptance.py::RunnerGuardTests.test_unknown_status_retains_cluster_data",kind:"test"})
# MERGE (toolchainTest:Symbol {id:"scripts/automation/test_w16a_device_revocation_http_pg_acceptance.py::RunnerGuardTests.test_rust_toolchain_preflight_fails_closed",kind:"test"})
# MERGE (toolchainOkTest:Symbol {id:"scripts/automation/test_w16a_device_revocation_http_pg_acceptance.py::RunnerGuardTests.test_rust_toolchain_preflight_accepts_selected_compiler",kind:"test"})
# MERGE (cargoToolchainTest:Symbol {id:"scripts/automation/test_w16a_device_revocation_http_pg_acceptance.py::RunnerGuardTests.test_cargo_toolchain_preflight_accepts_selected_alias",kind:"test"})
# MERGE (versionTest:Symbol {id:"scripts/automation/test_w16a_device_revocation_http_pg_acceptance.py::RunnerGuardTests.test_toolchain_override_requires_stable_semver_at_least_minimum",kind:"test"})
# MERGE (prereqTest:Symbol {id:"scripts/automation/test_w16a_device_revocation_http_pg_acceptance.py::RunnerGuardTests.test_required_tools_include_rustc_preflight_error",kind:"test"})
# MERGE (blockedSummaryTest:Symbol {id:"scripts/automation/test_w16a_device_revocation_http_pg_acceptance.py::RunnerGuardTests.test_missing_toolchain_writes_blocked_summary_without_starting_postgres",kind:"test"})
# MERGE (boundedTailTest:Symbol {id:"scripts/automation/test_w16a_device_revocation_http_pg_acceptance.py::RunnerGuardTests.test_child_logs_are_bounded",kind:"test"})
# MERGE (timeoutTreeTest:Symbol {id:"scripts/automation/test_w16a_device_revocation_http_pg_acceptance.py::RunnerGuardTests.test_timeout_kills_child_process_tree",kind:"test"})
# MERGE (runner:File {path:"scripts/automation/w16a_device_revocation_http_pg_acceptance.py"})
# MERGE (self)-[:DEFINES]->(tests),(tests)-[:DEFINES]->(envTest),(tests)-[:DEFINES]->(urlTest),(tests)-[:DEFINES]->(outputTest),(tests)-[:DEFINES]->(commandTest),(tests)-[:DEFINES]->(pathTest),(tests)-[:DEFINES]->(cleanupTest),(tests)-[:DEFINES]->(stoppedTest),(tests)-[:DEFINES]->(unknownTest),(tests)-[:DEFINES]->(toolchainTest),(tests)-[:DEFINES]->(toolchainOkTest),(tests)-[:DEFINES]->(cargoToolchainTest),(tests)-[:DEFINES]->(versionTest),(tests)-[:DEFINES]->(prereqTest),(tests)-[:DEFINES]->(blockedSummaryTest),(tests)-[:DEFINES]->(boundedTailTest),(tests)-[:DEFINES]->(timeoutTreeTest)
# MERGE (envTest)-[:TESTS]->(runner),(envTest)-[:CALLS]->(:Symbol {id:"scripts/automation/w16a_device_revocation_http_pg_acceptance.py::sanitized_environment",kind:"function"})
# MERGE (urlTest)-[:TESTS]->(runner),(urlTest)-[:CALLS]->(:Symbol {id:"scripts/automation/w16a_device_revocation_http_pg_acceptance.py::database_urls",kind:"function"})
# MERGE (outputTest)-[:TESTS]->(runner),(outputTest)-[:CALLS]->(:Symbol {id:"scripts/automation/w16a_device_revocation_http_pg_acceptance.py::accepted_test_output",kind:"function"})
# MERGE (commandTest)-[:TESTS]->(runner),(commandTest)-[:CALLS]->(:Symbol {id:"scripts/automation/w16a_device_revocation_http_pg_acceptance.py::cargo_test_command",kind:"function"})
# MERGE (pathTest)-[:TESTS]->(runner),(pathTest)-[:CALLS]->(:Symbol {id:"scripts/automation/w16a_device_revocation_http_pg_acceptance.py::is_inside",kind:"function"})
# MERGE (cleanupTest)-[:TESTS]->(runner),(cleanupTest)-[:CALLS]->(:Symbol {id:"scripts/automation/w16a_device_revocation_http_pg_acceptance.py::stop_cluster",kind:"function"})
# MERGE (stoppedTest)-[:TESTS]->(runner),(stoppedTest)-[:CALLS]->(:Symbol {id:"scripts/automation/w16a_device_revocation_http_pg_acceptance.py::stop_cluster",kind:"function"})
# MERGE (unknownTest)-[:TESTS]->(runner),(unknownTest)-[:CALLS]->(:Symbol {id:"scripts/automation/w16a_device_revocation_http_pg_acceptance.py::stop_cluster",kind:"function"})
# MERGE (toolchainTest)-[:TESTS]->(runner),(toolchainTest)-[:CALLS]->(:Symbol {id:"scripts/automation/w16a_device_revocation_http_pg_acceptance.py::rust_toolchain_probe",kind:"function"})
# MERGE (toolchainOkTest)-[:TESTS]->(runner),(toolchainOkTest)-[:CALLS]->(:Symbol {id:"scripts/automation/w16a_device_revocation_http_pg_acceptance.py::rust_toolchain_probe",kind:"function"})
# MERGE (cargoToolchainTest)-[:TESTS]->(runner),(cargoToolchainTest)-[:CALLS]->(:Symbol {id:"scripts/automation/w16a_device_revocation_http_pg_acceptance.py::cargo_toolchain_probe",kind:"function"})
# MERGE (versionTest)-[:TESTS]->(runner),(versionTest)-[:CALLS]->(:Symbol {id:"scripts/automation/w16a_device_revocation_http_pg_acceptance.py::parse_stable_toolchain",kind:"function"}),(versionTest)-[:CALLS]->(:Symbol {id:"scripts/automation/w16a_device_revocation_http_pg_acceptance.py::selected_toolchain",kind:"function"})
# MERGE (prereqTest)-[:TESTS]->(runner),(prereqTest)-[:CALLS]->(:Symbol {id:"scripts/automation/w16a_device_revocation_http_pg_acceptance.py::required_tools_missing",kind:"function"})
# MERGE (blockedSummaryTest)-[:TESTS]->(runner),(blockedSummaryTest)-[:CALLS]->(:Symbol {id:"scripts/automation/w16a_device_revocation_http_pg_acceptance.py::main",kind:"function"})
# MERGE (boundedTailTest)-[:TESTS]->(runner),(boundedTailTest)-[:CALLS]->(:Symbol {id:"scripts/automation/w16a_device_revocation_http_pg_acceptance.py::run_step",kind:"function"})
# MERGE (timeoutTreeTest)-[:TESTS]->(runner),(timeoutTreeTest)-[:CALLS]->(:Symbol {id:"scripts/automation/w16a_device_revocation_http_pg_acceptance.py::run_step",kind:"function"})
# @endcypher

import json
import os
import sys
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from scripts.automation import w16a_device_revocation_http_pg_acceptance as acceptance


class RunnerGuardTests(unittest.TestCase):
    def test_child_logs_are_bounded(self) -> None:
        child = (
            "import sys; chunk = 'x' * 65536; "
            "[sys.stdout.write(chunk) for _ in range(32)]; "
            "[sys.stderr.write(chunk.replace('x', 'y')) for _ in range(32)]"
        )
        with tempfile.TemporaryDirectory(dir=acceptance.ROOT) as temporary:
            logs = Path(temporary) / "logs"
            result = acceptance.run_step(
                "large-output",
                [sys.executable, "-c", child],
                acceptance.sanitized_environment({}),
                logs,
                timeout=30,
            )
            self.assertEqual(result["exit_code"], 0)
            stdout = (logs / "large-output.stdout.log").read_bytes()
            stderr = (logs / "large-output.stderr.log").read_bytes()
            self.assertLessEqual(len(stdout), acceptance.MAX_LOG_BYTES)
            self.assertLessEqual(len(stderr), acceptance.MAX_LOG_BYTES)
            self.assertTrue(stdout.endswith(b"x" * 32))
            self.assertTrue(stderr.endswith(b"y" * 32))

    @unittest.skipUnless(os.name == "nt", "the disposable runner process-tree guard is Windows-specific")
    def test_timeout_kills_child_process_tree(self) -> None:
        parent = (
            "import subprocess, sys, time; "
            "child = subprocess.Popen([sys.executable, '-c', 'import time; time.sleep(60)']); "
            "print(f'spawned_pid={child.pid}', flush=True); time.sleep(60)"
        )
        with tempfile.TemporaryDirectory(dir=acceptance.ROOT) as temporary:
            logs = Path(temporary) / "logs"
            result = acceptance.run_step(
                "timeout-tree",
                [sys.executable, "-c", parent],
                acceptance.sanitized_environment({}),
                logs,
                timeout=1,
            )
            self.assertEqual(result["exit_code"], 124)
            self.assertNotIn("capture_error", result)
            stdout = (logs / "timeout-tree.stdout.log").read_text(encoding="utf-8")
            pid = int(stdout.split("spawned_pid=", 1)[1].splitlines()[0])
            try:
                check = acceptance.subprocess.run(
                    ["tasklist.exe", "/FI", f"PID eq {pid}", "/FO", "CSV", "/NH"],
                    capture_output=True,
                    text=True,
                    check=False,
                )
                self.assertNotIn(str(pid), check.stdout)
            finally:
                acceptance.subprocess.run(
                    ["taskkill.exe", "/PID", str(pid), "/T", "/F"],
                    stdin=acceptance.subprocess.DEVNULL,
                    stdout=acceptance.subprocess.DEVNULL,
                    stderr=acceptance.subprocess.DEVNULL,
                    check=False,
                )

    def test_environment_sanitization_removes_external_targets(self) -> None:
        clean = acceptance.sanitized_environment(
            {
                "PATH": "safe-path",
                "PGHOST": "remote.example",
                "PGPASSWORD": "not-a-real-secret",
                "DATABASE_URL": "postgres://remote",
                "STAR_W16_DEVICE_AUTH_TEST_DATABASE_URL": "postgres://remote",
                "STAR_W16_DEVICE_AUTH_ADMIN_TEST_DATABASE_URL": "postgres://remote-admin",
                "SQLX_OFFLINE": "true",
            }
        )
        self.assertEqual(clean["PATH"], "safe-path")
        self.assertEqual(clean["CARGO_TERM_COLOR"], "never")
        self.assertFalse(any(key.upper().startswith("PG") for key in clean))
        self.assertNotIn("DATABASE_URL", clean)
        self.assertNotIn("STAR_W16_DEVICE_AUTH_TEST_DATABASE_URL", clean)
        self.assertNotIn("STAR_W16_DEVICE_AUTH_ADMIN_TEST_DATABASE_URL", clean)
        self.assertNotIn("SQLX_OFFLINE", clean)

    def test_urls_are_loopback_and_use_distinct_login_roles(self) -> None:
        admin, app = acceptance.database_urls(54321)
        self.assertEqual(admin, "postgres://postgres@127.0.0.1:54321/w16a_device_revoke_http_acceptance")
        self.assertEqual(app, "postgres://star_w16_device_revoke_app@127.0.0.1:54321/w16a_device_revoke_http_acceptance")
        self.assertNotEqual(admin.split("@", 1)[0], app.split("@", 1)[0])

    def test_output_requires_all_named_cases(self) -> None:
        output = "\n".join(f"test {name} ... ok" for name in acceptance.EXPECTED_TESTS)
        output += f"\ntest result: ok. {len(acceptance.EXPECTED_TESTS)} passed; 0 failed; 0 ignored; 0 measured; 0 filtered out"
        self.assertEqual(acceptance.accepted_test_output(output), (True, []))
        missing_case = output.replace(f"test {acceptance.EXPECTED_TESTS[1]} ... ok\n", "")
        ok, missing = acceptance.accepted_test_output(missing_case)
        self.assertFalse(ok)
        self.assertIn(acceptance.EXPECTED_TESTS[1], missing)

    def test_command_is_locked_and_bounded(self) -> None:
        command = acceptance.cargo_test_command("rtk", "cargo", "1.99.1", "stable")
        self.assertEqual(command[:3], ["rtk", "proxy", "cargo"])
        self.assertIn("+stable", command)
        default_command = acceptance.cargo_test_command("rtk", "cargo")
        self.assertIn("+1.99.1", default_command)
        self.assertEqual(command[command.index("-j") + 1], "4")
        self.assertEqual(command[command.index("--test") + 1], acceptance.TEST_BIN)
        self.assertIn("--locked", command)
        self.assertIn("--ignored", command)

    def test_rust_toolchain_preflight_fails_closed(self) -> None:
        with patch.object(
            acceptance.subprocess,
            "run",
            return_value=acceptance.subprocess.CompletedProcess(
                args=["rtk", "proxy", "rustc"],
                returncode=1,
                stdout="toolchain rustc component missing",
                stderr="[rtk] warning: no hook installed",
            ),
        ) as run:
            result = acceptance.rust_toolchain_probe("rtk", "rustc", "1.99.1", "stable")
        self.assertEqual(result["available"], False)
        self.assertEqual(result["probe_exit_code"], 1)
        self.assertIn("Rust 1.99.1 rustc unavailable (exit 1)", result["error"] or "")
        self.assertIn("toolchain rustc component missing", result["error"] or "")
        self.assertEqual(
            run.call_args.args[0],
            ["rtk", "proxy", "rustc", "+stable", "-V"],
        )

    def test_rust_toolchain_preflight_accepts_selected_compiler(self) -> None:
        with patch.object(
            acceptance.subprocess,
            "run",
            return_value=acceptance.subprocess.CompletedProcess(
                args=["rtk", "proxy", "rustc"],
                returncode=0,
                stdout="rustc 1.99.1 (test build)",
                stderr="",
            ),
        ):
            result = acceptance.rust_toolchain_probe("rtk", "rustc", "1.99.1", "stable")
        self.assertEqual(result["available"], True)
        self.assertEqual(result["rustc_version"], "rustc 1.99.1 (test build)")
        self.assertEqual(result["selector"], "stable")
        with patch.object(
            acceptance.subprocess,
            "run",
            return_value=acceptance.subprocess.CompletedProcess(
                args=["rtk", "proxy", "rustc"],
                returncode=0,
                stdout="rustc 1.99.0 (wrong selected compiler)",
                stderr="",
            ),
        ):
            mismatch = acceptance.rust_toolchain_probe("rtk", "rustc", "1.99.1", "stable")
        self.assertFalse(mismatch["available"])
        self.assertIn("reported 1.99.0", mismatch["error"] or "")

    def test_cargo_toolchain_preflight_accepts_selected_alias(self) -> None:
        with patch.object(
            acceptance.subprocess,
            "run",
            return_value=acceptance.subprocess.CompletedProcess(
                args=["rtk", "proxy", "cargo"],
                returncode=0,
                stdout="cargo 1.99.1 (test build)",
                stderr="",
            ),
        ) as run:
            result = acceptance.cargo_toolchain_probe("rtk", "cargo", "1.99.1", "stable")
        self.assertTrue(result["available"])
        self.assertEqual(result["cargo_version"], "cargo 1.99.1 (test build)")
        self.assertEqual(run.call_args.args[0], ["rtk", "proxy", "cargo", "+stable", "-V"])
        with patch.object(
            acceptance.subprocess,
            "run",
            return_value=acceptance.subprocess.CompletedProcess(
                args=["rtk", "proxy", "cargo"],
                returncode=0,
                stdout="cargo 1.99.2 (wrong selected tool)",
                stderr="",
            ),
        ):
            mismatch = acceptance.cargo_toolchain_probe("rtk", "cargo", "1.99.1", "stable")
        self.assertFalse(mismatch["available"])
        self.assertIn("reported 1.99.2", mismatch["error"] or "")

    def test_toolchain_override_requires_stable_semver_at_least_minimum(self) -> None:
        with patch.object(acceptance.subprocess, "run") as run:
            below = acceptance.rust_toolchain_probe("rtk", "rustc", "1.98.9")
            previous_baseline = acceptance.rust_toolchain_probe("rtk", "rustc", "1.99.0")
            alias = acceptance.rust_toolchain_probe("rtk", "rustc", "stable")
            unapproved = acceptance.rust_toolchain_probe("rtk", "rustc", "1.99.1", "nightly")
        self.assertIn("below minimum 1.99.1", below["error"] or "")
        self.assertIn("below minimum 1.99.1", previous_baseline["error"] or "")
        self.assertIn("stable X.Y.Z", alias["error"] or "")
        self.assertIn("requested version or the explicit 'stable' alias", unapproved["error"] or "")
        run.assert_not_called()

    def test_required_tools_include_rustc_preflight_error(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            pg_bin = Path(temporary)
            for name in acceptance.PG_TOOLS:
                (pg_bin / name).write_text("fixture", encoding="utf-8")
            with patch.object(
                acceptance.shutil,
                "which",
                side_effect=lambda command: f"resolved-{command}",
            ), patch.object(
                acceptance,
                "rust_toolchain_probe",
                return_value={"requested": "1.99.1", "selector": "stable", "rustc_version": None, "probe_exit_code": 1, "available": False, "error": "Rust 1.99.1 rustc unavailable (exit 1): test diagnostic"},
            ) as rust_probe, patch.object(
                acceptance,
                "cargo_toolchain_probe",
                return_value={"requested": "1.99.1", "selector": "stable", "cargo_version": None, "probe_exit_code": 1, "available": False, "error": "Rust 1.99.1 cargo unavailable (exit 1): test diagnostic"},
            ) as cargo_probe:
                info: dict[str, object] = {}
                missing = acceptance.required_tools_missing(pg_bin, "cargo", "rtk", "1.99.1", "stable", info)
        self.assertEqual(
            missing,
            ["Rust 1.99.1 rustc unavailable (exit 1): test diagnostic; Rust 1.99.1 cargo unavailable (exit 1): test diagnostic"],
        )
        self.assertEqual(info["requested"], "1.99.1")
        self.assertEqual(info["selector"], "stable")
        rust_probe.assert_called_once_with("rtk", "resolved-rustc", "1.99.1", "stable")
        cargo_probe.assert_called_once_with("rtk", "resolved-cargo", "1.99.1", "stable")

    def test_missing_toolchain_writes_blocked_summary_without_starting_postgres(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            cache = Path(temporary)
            with patch.object(acceptance, "CACHE", cache), patch.object(
                acceptance,
                "required_tools_missing",
                return_value=["Rust 1.99.1 rustc unavailable (exit 1): test diagnostic"],
            ), patch.object(acceptance.sys, "argv", ["w16a_device_revocation_http_pg_acceptance.py", "--rust-toolchain", "1.99.1", "--rust-toolchain-selector", "stable"]), patch(
                "builtins.print"
            ):
                self.assertEqual(acceptance.main(), 2)
            summaries = list(cache.glob("runs/*/acceptance-summary.json"))
            self.assertEqual(len(summaries), 1)
            summary = json.loads(summaries[0].read_text(encoding="utf-8"))
            self.assertEqual(summary["status"], "BLOCKED")
            self.assertEqual(summary["rust_toolchain"]["requested"], "1.99.1")
            self.assertEqual(summary["rust_toolchain"]["selector"], "stable")
            self.assertEqual(summary["steps"], [])
            self.assertIn("Rust 1.99.1 rustc unavailable", summary["missing_prerequisites"][0])
            self.assertFalse((summaries[0].parent / "postgres-data").exists())

    def test_private_path_guard(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            self.assertTrue(acceptance.is_inside(root / "run" / "postgres-data", root))
            self.assertFalse(acceptance.is_inside(root, root))
            self.assertFalse(acceptance.is_inside(root.parent / f"{root.name}-outside", root))

    def test_unstarted_partial_cluster_is_removed_only_inside_cache(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            data = root / "runs" / "partial" / "postgres-data"
            data.mkdir(parents=True)
            with patch.object(acceptance, "CACHE", root):
                result = acceptance.stop_cluster(Path("unused"), data, {}, root / "logs", False)
            self.assertEqual(result["status"], "not-started")
            self.assertTrue(result["data_removed"])
            self.assertFalse(data.exists())

    def test_status_three_removes_stopped_cluster(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            data = root / "runs" / "stopped" / "postgres-data"
            logs = root / "runs" / "stopped" / "logs"
            data.mkdir(parents=True)
            logs.mkdir(parents=True)
            (data / "postgres-server.log").write_text("stopped", encoding="utf-8")
            with patch.object(acceptance, "CACHE", root), patch.object(
                acceptance, "run_step", return_value={"exit_code": 3}
            ) as run_step:
                result = acceptance.stop_cluster(Path("unused"), data, {}, logs, True)
            run_step.assert_called_once()
            self.assertEqual(result["status"], "not-running")
            self.assertEqual(result["exit_code"], 0)
            self.assertTrue(result["data_removed"])
            self.assertFalse(data.exists())
            self.assertEqual((logs / "postgres-server.log").read_text(encoding="utf-8"), "stopped")

    def test_unknown_status_retains_cluster_data(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            data = root / "runs" / "unknown" / "postgres-data"
            logs = root / "runs" / "unknown" / "logs"
            data.mkdir(parents=True)
            logs.mkdir(parents=True)
            (data / "postgres-server.log").write_text("inspect me", encoding="utf-8")
            with patch.object(acceptance, "CACHE", root), patch.object(
                acceptance, "run_step", return_value={"exit_code": 4}
            ) as run_step:
                result = acceptance.stop_cluster(Path("unused"), data, {}, logs, True)
            run_step.assert_called_once()
            self.assertEqual(result["status"], "unknown")
            self.assertNotEqual(result["exit_code"], 0)
            self.assertTrue(result["data_retained"])
            self.assertTrue(data.exists())
            self.assertEqual((logs / "postgres-server.log").read_text(encoding="utf-8"), "inspect me")


if __name__ == "__main__":
    unittest.main()
