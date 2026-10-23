"""Bounded unit checks for the isolated authenticated HTTP acceptance runner."""

# @cypher schema=1 source_sha256=2144292fc224ea50da752e8e25771cb85dd65fbae0b275ab957f0f6e54b34d35
# MERGE (self:File {path:"scripts/automation/test_phase9f2_binding_http_acceptance.py"})
# MERGE (sanitization:Symbol {id:"scripts/automation/test_phase9f2_binding_http_acceptance.py::test_sanitized_environment_removes_database_overrides",kind:"test"})
# MERGE (loopback:Symbol {id:"scripts/automation/test_phase9f2_binding_http_acceptance.py::test_loopback_port_avoids_protected_ports",kind:"test"})
# MERGE (delta:Symbol {id:"scripts/automation/test_phase9f2_binding_http_acceptance.py::test_snapshot_delta_accepts_and_rejects_exact_counts",kind:"test"})
# MERGE (startupFailure:Symbol {id:"scripts/automation/test_phase9f2_binding_http_acceptance.py::test_startup_failure_still_requires_cluster_cleanup",kind:"test"})
# MERGE (readinessFailure:Symbol {id:"scripts/automation/test_phase9f2_binding_http_acceptance.py::test_readiness_failure_still_requires_cluster_cleanup",kind:"test"})
# MERGE (stopFailure:Symbol {id:"scripts/automation/test_phase9f2_binding_http_acceptance.py::test_stop_failure_preserves_private_data_directory",kind:"test"})
# MERGE (acceptanceFailure:Symbol {id:"scripts/automation/test_phase9f2_binding_http_acceptance.py::test_acceptance_failure_preserves_data_directory_after_stop",kind:"test"})
# MERGE (policyProof:Symbol {id:"scripts/automation/test_phase9f2_binding_http_acceptance.py::test_all_lock_policy_proofs_require_sqlstate_42501",kind:"test"})
# MERGE (main:Symbol {id:"scripts/automation/phase9f2_binding_http_acceptance.py::sanitized_environment",kind:"function"})
# MERGE (port:Symbol {id:"scripts/automation/phase9f2_binding_http_acceptance.py::select_loopback_port",kind:"function"})
# MERGE (assertion:Symbol {id:"scripts/automation/phase9f2_binding_http_acceptance.py::assert_snapshot_delta",kind:"function"})
# MERGE (start:Symbol {id:"scripts/automation/phase9f2_binding_http_acceptance.py::start_cluster",kind:"function"})
# MERGE (stop:Symbol {id:"scripts/automation/phase9f2_binding_http_acceptance.py::stop_and_remove_cluster",kind:"function"})
# MERGE (self)-[:DEFINES]->(sanitization)
# MERGE (self)-[:DEFINES]->(loopback)
# MERGE (self)-[:DEFINES]->(delta)
# MERGE (self)-[:DEFINES]->(startupFailure)
# MERGE (self)-[:DEFINES]->(readinessFailure)
# MERGE (self)-[:DEFINES]->(stopFailure)
# MERGE (self)-[:DEFINES]->(acceptanceFailure)
# MERGE (self)-[:DEFINES]->(policyProof)
# MERGE (sanitization)-[:CALLS]->(main)
# MERGE (loopback)-[:CALLS]->(port)
# MERGE (delta)-[:CALLS]->(assertion)
# MERGE (startupFailure)-[:CALLS]->(start)
# MERGE (readinessFailure)-[:CALLS]->(start)
# MERGE (stopFailure)-[:CALLS]->(stop)
# MERGE (policyProof)-[:CALLS]->(:Symbol {id:"scripts/automation/phase9f2_binding_http_acceptance.py::seed_fixture",kind:"function"})
# @endcypher

from __future__ import annotations

import inspect
import os
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest
from unittest.mock import MagicMock, patch

from scripts.automation import phase9f2_binding_http_acceptance as runner
from scripts.automation.phase9f2_binding_http_acceptance import (
    assert_snapshot_delta,
    sanitized_environment,
    select_loopback_port,
    start_cluster,
    stop_and_remove_cluster,
)


class BindingHttpAcceptanceRunnerTests(unittest.TestCase):
    def test_sanitized_environment_removes_database_overrides(self) -> None:
        with patch.dict(os.environ, {
            "PGHOST": "untrusted.example", "PGPORT": "5432",
            "DATABASE_URL": "postgres://must-not-leak", "STAR_BINDING_HTTP_TEST_PHASE": "old",
            "PATH": os.environ.get("PATH", ""),
        }, clear=True):
            environment = sanitized_environment()
        self.assertNotIn("PGHOST", environment)
        self.assertNotIn("PGPORT", environment)
        self.assertNotIn("DATABASE_URL", environment)
        self.assertNotIn("STAR_BINDING_HTTP_TEST_PHASE", environment)
        self.assertEqual(environment["CARGO_TERM_COLOR"], "never")

    def test_loopback_port_avoids_protected_ports(self) -> None:
        port = select_loopback_port()
        self.assertGreaterEqual(port, 1024)
        self.assertLessEqual(port, 65535)
        self.assertNotIn(port, {5432, 5555})

        for protected_port in (5432, 5555):
            blocked_listener = MagicMock()
            blocked_listener.__enter__.return_value = blocked_listener
            blocked_listener.getsockname.return_value = ("127.0.0.1", protected_port)
            allowed_listener = MagicMock()
            allowed_listener.__enter__.return_value = allowed_listener
            allowed_listener.getsockname.return_value = ("127.0.0.1", 62000)
            with patch(
                "scripts.automation.phase9f2_binding_http_acceptance.socket.socket",
                side_effect=[blocked_listener, allowed_listener],
            ) as socket_factory:
                self.assertEqual(select_loopback_port(), 62000)
            self.assertEqual(socket_factory.call_count, 2)

    def test_snapshot_delta_accepts_and_rejects_exact_counts(self) -> None:
        assert_snapshot_delta({"events": 4}, {"events": 5}, {"events": 1}, "bind")
        with self.assertRaisesRegex(AssertionError, "events delta"):
            assert_snapshot_delta({"events": 4}, {"events": 6}, {"events": 1}, "bind")

    @patch("scripts.automation.phase9f2_binding_http_acceptance.wait_for_postgres")
    @patch("scripts.automation.phase9f2_binding_http_acceptance.run_step")
    def test_startup_failure_still_requires_cluster_cleanup(self, run_step, wait_for_postgres) -> None:
        run_step.side_effect = [
            {"name": "postgres-init", "exit_code": 0},
            {"name": "postgres-start", "exit_code": 1},
        ]

        results, cleanup_required, ready = start_cluster(
            Path("postgres-bin"), 62001, Path("private-data"), {}, Path("private-logs"),
        )

        self.assertTrue(cleanup_required)
        self.assertFalse(ready)
        self.assertEqual([result["name"] for result in results], ["postgres-init", "postgres-start"])
        wait_for_postgres.assert_not_called()

    @patch("scripts.automation.phase9f2_binding_http_acceptance.wait_for_postgres")
    @patch("scripts.automation.phase9f2_binding_http_acceptance.run_step")
    def test_readiness_failure_still_requires_cluster_cleanup(self, run_step, wait_for_postgres) -> None:
        run_step.side_effect = [
            {"name": "postgres-init", "exit_code": 0},
            {"name": "postgres-start", "exit_code": 0},
        ]
        wait_for_postgres.return_value = {"name": "postgres-ready", "exit_code": 1}

        results, cleanup_required, ready = start_cluster(
            Path("postgres-bin"), 62002, Path("private-data"), {}, Path("private-logs"),
        )

        self.assertTrue(cleanup_required)
        self.assertFalse(ready)
        self.assertEqual([result["name"] for result in results], [
            "postgres-init", "postgres-start", "postgres-ready",
        ])
        wait_for_postgres.assert_called_once()

    def test_stop_failure_preserves_private_data_directory(self) -> None:
        with TemporaryDirectory() as temporary_directory:
            private_root = Path(temporary_directory) / "runner"
            data_directory = private_root / "postgres" / "data-only-this-test"
            data_directory.mkdir(parents=True)
            with patch.object(runner, "RUNNER_ROOT", private_root), patch.object(
                runner,
                "run_step",
                return_value={"name": "postgres-stop", "exit_code": 1},
            ):
                result = stop_and_remove_cluster(
                    Path("postgres-bin"), data_directory, {}, Path("private-logs"),
                    cleanup_required=True,
                )

            self.assertEqual(result["exit_code"], 1)
            self.assertTrue(data_directory.is_dir())
            self.assertTrue(result["owned_data_dir_preserved"])
            self.assertFalse(result["owned_data_dir_removed"])

    def test_acceptance_failure_preserves_data_directory_after_stop(self) -> None:
        with TemporaryDirectory() as temporary_directory:
            private_root = Path(temporary_directory) / "runner"
            data_directory = private_root / "postgres" / "data-only-this-test"
            data_directory.mkdir(parents=True)
            with patch.object(runner, "RUNNER_ROOT", private_root), patch.object(
                runner,
                "run_step",
                return_value={"name": "postgres-stop", "exit_code": 0},
            ):
                result = stop_and_remove_cluster(
                    Path("postgres-bin"), data_directory, {}, Path("private-logs"),
                    cleanup_required=True, preserve_data=True,
                )

            self.assertEqual(result["exit_code"], 0)
            self.assertTrue(data_directory.is_dir())
            self.assertTrue(result["owned_data_dir_preserved"])
            self.assertFalse(result["owned_data_dir_removed"])

    def test_all_lock_policy_proofs_require_sqlstate_42501(self) -> None:
        seed_source = inspect.getsource(runner.seed_fixture)
        self.assertEqual(seed_source.count("EXCEPTION WHEN SQLSTATE '42501'"), 5)
        self.assertEqual(seed_source.count("update_sqlstate IS DISTINCT FROM '42501'"), 5)
        self.assertNotIn("EXCEPTION WHEN insufficient_privilege", seed_source)
        self.assertIn("p.permissive='RESTRICTIVE'", seed_source)


if __name__ == "__main__":
    unittest.main()
