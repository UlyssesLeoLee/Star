"""Unit checks for the W16-A PostgreSQL runner's fail-closed guards."""

# @cypher schema=1 source_sha256=97d1da01f0a638f79b1fa2c027a5ef4cec615c5695340100c13c6f36aa5544e9
# MERGE (self:File {path:"scripts/automation/test_w16a_oauth_pg_acceptance.py"})
# MERGE (sanitizeTest:Symbol {id:"scripts/automation/test_w16a_oauth_pg_acceptance.py::test_environment_sanitization_removes_external_targets",kind:"test"})
# MERGE (outputTest:Symbol {id:"scripts/automation/test_w16a_oauth_pg_acceptance.py::test_output_requires_every_expected_test",kind:"test"})
# MERGE (commandTest:Symbol {id:"scripts/automation/test_w16a_oauth_pg_acceptance.py::test_cargo_invocation_uses_rtk_proxy_and_four_jobs",kind:"test"})
# MERGE (tailTest:Symbol {id:"scripts/automation/test_w16a_oauth_pg_acceptance.py::test_log_tail_is_bounded",kind:"test"})
# MERGE (pathTest:Symbol {id:"scripts/automation/test_w16a_oauth_pg_acceptance.py::test_path_guard_requires_a_private_descendant",kind:"test"})
# MERGE (runner:File {path:"scripts/automation/w16a_oauth_pg_acceptance.py"})
# MERGE (self)-[:DEFINES]->(sanitizeTest),(self)-[:DEFINES]->(outputTest),(self)-[:DEFINES]->(commandTest),(self)-[:DEFINES]->(tailTest),(self)-[:DEFINES]->(pathTest)
# MERGE (sanitizeTest)-[:TESTS]->(runner),(sanitizeTest)-[:CALLS]->(:Symbol {id:"scripts/automation/w16a_oauth_pg_acceptance.py::sanitized_environment",kind:"function"})
# MERGE (outputTest)-[:TESTS]->(runner),(outputTest)-[:CALLS]->(:Symbol {id:"scripts/automation/w16a_oauth_pg_acceptance.py::accepted_test_output",kind:"function"})
# MERGE (commandTest)-[:TESTS]->(runner),(commandTest)-[:CALLS]->(:Symbol {id:"scripts/automation/w16a_oauth_pg_acceptance.py::cargo_test_command",kind:"function"})
# MERGE (tailTest)-[:TESTS]->(runner),(tailTest)-[:CALLS]->(:Symbol {id:"scripts/automation/w16a_oauth_pg_acceptance.py::read_tail",kind:"function"})
# MERGE (pathTest)-[:TESTS]->(runner),(pathTest)-[:CALLS]->(:Symbol {id:"scripts/automation/w16a_oauth_pg_acceptance.py::is_inside",kind:"function"})
# @endcypher

from pathlib import Path
import tempfile
import unittest

from scripts.automation import w16a_oauth_pg_acceptance as acceptance


class W16AOAuthPostgresRunnerTests(unittest.TestCase):
    def test_environment_sanitization_removes_external_targets(self) -> None:
        cleaned = acceptance.sanitized_environment(
            {
                "PATH": "safe-path",
                "PGHOST": "remote-host",
                "PGPASSWORD": "secret-placeholder",
                "DATABASE_URL": "postgres://remote",
                "STAR_W16_DEVICE_AUTH_TEST_DATABASE_URL": "postgres://remote-test",
                "SQLX_OFFLINE": "true",
            }
        )
        self.assertEqual(cleaned["PATH"], "safe-path")
        self.assertEqual(cleaned["CARGO_TERM_COLOR"], "never")
        self.assertFalse(any(key.upper().startswith("PG") for key in cleaned))
        self.assertNotIn("DATABASE_URL", cleaned)
        self.assertNotIn("STAR_W16_DEVICE_AUTH_TEST_DATABASE_URL", cleaned)
        self.assertNotIn("SQLX_OFFLINE", cleaned)

    def test_output_requires_every_expected_test(self) -> None:
        expected = acceptance.EXPECTED_PG_TESTS
        passing_output = (
            "\n".join(f"test {name} ... ok" for name in acceptance.EXPECTED_PG_TESTS)
            + f"\ntest result: ok. {len(acceptance.EXPECTED_PG_TESTS)} passed; 0 failed; 0 ignored; 0 measured; 9 filtered out"
        )
        self.assertEqual(acceptance.accepted_test_output(passing_output), (True, []))
        missing_one = passing_output.replace(f"test {expected[1]} ... ok\n", "")
        accepted, missing = acceptance.accepted_test_output(missing_one)
        self.assertFalse(accepted)
        self.assertIn(expected[1], missing)

    def test_cargo_invocation_uses_rtk_proxy_and_four_jobs(self) -> None:
        command = acceptance.cargo_test_command("rtk", "cargo")
        self.assertEqual(command[:3], ["rtk", "proxy", "cargo"])
        self.assertEqual(command[command.index("-j") + 1], "4")
        self.assertEqual(command[command.index("--test") + 1], "oauth_device_auth_pg")
        self.assertIn("--ignored", command)

    def test_log_tail_is_bounded(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            log_path = Path(temporary) / "large.log"
            log_path.write_bytes(b"x" * (acceptance.MAX_LOG_TAIL_BYTES + 32) + b"tail")
            tail = acceptance.read_tail(log_path)
        self.assertLessEqual(len(tail.encode("utf-8")), acceptance.MAX_LOG_TAIL_BYTES)
        self.assertTrue(tail.endswith("tail"))

    def test_path_guard_requires_a_private_descendant(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            child = root / "owned" / "postgres-data"
            outside = root.parent / f"{root.name}-outside"
            self.assertTrue(acceptance.is_inside(child, root))
            self.assertFalse(acceptance.is_inside(root, root))
            self.assertFalse(acceptance.is_inside(outside, root))


if __name__ == "__main__":
    unittest.main()
