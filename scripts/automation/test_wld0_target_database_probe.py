# @cypher schema=1 source_sha256=9fa33628dce262f45c8d1d01a8cfa5edd72287378bce05e48f624b0bb9f34b7c
# MERGE (self:File {path:"scripts/automation/test_wld0_target_database_probe.py"})
# MERGE (case:Symbol {id:"scripts/automation/test_wld0_target_database_probe.py::Wld0TargetDatabaseProbeTests",kind:"class"})
# MERGE (snapshot:Symbol {id:"scripts/automation/test_wld0_target_database_probe.py::snapshot_output",kind:"function"})
# MERGE (missing:Symbol {id:"scripts/automation/test_wld0_target_database_probe.py::Wld0TargetDatabaseProbeTests.test_missing_service_and_identity_do_not_spawn",kind:"method"})
# MERGE (service:Symbol {id:"scripts/automation/test_wld0_target_database_probe.py::Wld0TargetDatabaseProbeTests.test_service_name_rejects_connection_text",kind:"method"})
# MERGE (psql:Symbol {id:"scripts/automation/test_wld0_target_database_probe.py::Wld0TargetDatabaseProbeTests.test_missing_psql_is_blocked",kind:"method"})
# MERGE (shape:Symbol {id:"scripts/automation/test_wld0_target_database_probe.py::Wld0TargetDatabaseProbeTests.test_command_is_fixed_and_read_only",kind:"method"})
# MERGE (secrets:Symbol {id:"scripts/automation/test_wld0_target_database_probe.py::Wld0TargetDatabaseProbeTests.test_errors_do_not_echo_stderr",kind:"method"})
# MERGE (timeout:Symbol {id:"scripts/automation/test_wld0_target_database_probe.py::Wld0TargetDatabaseProbeTests.test_timeout_is_blocked",kind:"method"})
# MERGE (unknown_ledger:Symbol {id:"scripts/automation/test_wld0_target_database_probe.py::Wld0TargetDatabaseProbeTests.test_unknown_ledger_is_not_reconciled",kind:"method"})
# MERGE (unknown_rls:Symbol {id:"scripts/automation/test_wld0_target_database_probe.py::Wld0TargetDatabaseProbeTests.test_unknown_rls_is_not_accepted",kind:"method"})
# MERGE (disabled_rls:Symbol {id:"scripts/automation/test_wld0_target_database_probe.py::Wld0TargetDatabaseProbeTests.test_disabled_rls_is_blocked",kind:"method"})
# MERGE (observed:Symbol {id:"scripts/automation/test_wld0_target_database_probe.py::Wld0TargetDatabaseProbeTests.test_complete_fixture_is_observation_only",kind:"method"})
# MERGE (truncated:Symbol {id:"scripts/automation/test_wld0_target_database_probe.py::Wld0TargetDatabaseProbeTests.test_truncated_migration_version_is_unknown",kind:"method"})
# MERGE (membership:Symbol {id:"scripts/automation/test_wld0_target_database_probe.py::Wld0TargetDatabaseProbeTests.test_elevated_owner_role_membership_is_blocked",kind:"method"})
# MERGE (privilege:Symbol {id:"scripts/automation/test_wld0_target_database_probe.py::Wld0TargetDatabaseProbeTests.test_unexpected_owner_table_privilege_is_blocked",kind:"method"})
# MERGE (self)-[:DEFINES]->(case)
# MERGE (self)-[:DEFINES]->(snapshot)
# MERGE (case)-[:HAS_METHOD]->(missing)
# MERGE (case)-[:HAS_METHOD]->(service)
# MERGE (case)-[:HAS_METHOD]->(psql)
# MERGE (case)-[:HAS_METHOD]->(shape)
# MERGE (case)-[:HAS_METHOD]->(secrets)
# MERGE (case)-[:HAS_METHOD]->(timeout)
# MERGE (case)-[:HAS_METHOD]->(unknown_ledger)
# MERGE (case)-[:HAS_METHOD]->(unknown_rls)
# MERGE (case)-[:HAS_METHOD]->(disabled_rls)
# MERGE (case)-[:HAS_METHOD]->(observed)
# MERGE (case)-[:HAS_METHOD]->(truncated)
# MERGE (case)-[:HAS_METHOD]->(membership)
# MERGE (case)-[:HAS_METHOD]->(privilege)
# MERGE (jsonDumps:ExternalService {name:"Python stdlib json.dumps"})
# MERGE (snapshot)-[:CALLS]->(jsonDumps)
# MERGE (missing)-[:CALLS]->(:Symbol {id:"scripts/automation/wld0_target_database_probe.py::run_probe",kind:"function"})
# MERGE (service)-[:CALLS]->(:Symbol {id:"scripts/automation/wld0_target_database_probe.py::validate_inputs",kind:"function"})
# MERGE (psql)-[:CALLS]->(:Symbol {id:"scripts/automation/wld0_target_database_probe.py::run_probe",kind:"function"})
# MERGE (shape)-[:CALLS]->(:Symbol {id:"scripts/automation/wld0_target_database_probe.py::run_probe",kind:"function"})
# MERGE (secrets)-[:CALLS]->(:Symbol {id:"scripts/automation/wld0_target_database_probe.py::run_probe",kind:"function"})
# MERGE (timeout)-[:CALLS]->(:Symbol {id:"scripts/automation/wld0_target_database_probe.py::run_probe",kind:"function"})
# MERGE (unknown_ledger)-[:CALLS]->(:Symbol {id:"scripts/automation/wld0_target_database_probe.py::inspect_snapshot",kind:"function"})
# MERGE (unknown_rls)-[:CALLS]->(:Symbol {id:"scripts/automation/wld0_target_database_probe.py::inspect_snapshot",kind:"function"})
# MERGE (disabled_rls)-[:CALLS]->(:Symbol {id:"scripts/automation/wld0_target_database_probe.py::inspect_snapshot",kind:"function"})
# MERGE (observed)-[:CALLS]->(:Symbol {id:"scripts/automation/wld0_target_database_probe.py::inspect_snapshot",kind:"function"})
# MERGE (truncated)-[:CALLS]->(:Symbol {id:"scripts/automation/wld0_target_database_probe.py::inspect_snapshot",kind:"function"})
# MERGE (membership)-[:CALLS]->(:Symbol {id:"scripts/automation/wld0_target_database_probe.py::inspect_snapshot",kind:"function"})
# MERGE (privilege)-[:CALLS]->(:Symbol {id:"scripts/automation/wld0_target_database_probe.py::inspect_snapshot",kind:"function"})
# MERGE (case)-[:TESTS]->(:Symbol {id:"scripts/automation/wld0_target_database_probe.py::run_probe",kind:"function"})
# MERGE (case)-[:TESTS]->(:Symbol {id:"scripts/automation/wld0_target_database_probe.py::inspect_snapshot",kind:"function"})
# @endcypher
"""Focused boundary tests for the WLD-0 target database read-only probe."""
from __future__ import annotations

import argparse
import json
import subprocess
import unittest
from unittest.mock import Mock

import wld0_target_database_probe as probe


EXPECTED = argparse.Namespace(
    service="wld0_target_ro",
    expected_database="star_prod",
    expected_login="star_api_ro",
    expected_address="127.0.0.1",
    expected_port="5432",
)


def snapshot_output(*, ledger_known: bool = True, rls_value: bool | None = True) -> str:
    rows = [
        {"kind": "identity", "database": "star_prod", "user": "star_api_ro", "address": "127.0.0.1", "port": 5432},
        {"kind": "login_role", "name": "star_api_ro", "can_login": True, "superuser": False, "bypass_rls": False, "inherit": False, "createdb": False, "createrole": False, "replication": False, "owner_member": True},
        {"kind": "owner_role", "present": True, "can_login": False, "superuser": False, "bypass_rls": False, "inherit": False, "createdb": False, "createrole": False, "replication": False, "has_memberships": False, "owns_objects": False, "direct_schema_create": False, "direct_database_privileges": False},
        {
            "kind": "ledger_meta",
            "candidate_count": 1 if ledger_known else 0,
            "star_ledger_present": "t" if ledger_known else "f",
            "recognized_star_v1": "t" if ledger_known else "f",
        },
    ]
    if ledger_known:
        rows.append({"kind": "ledger_entry", "version": "2026-10-15-run-host-independent-directory.sql", "version_truncated": False, "applied_at": "2026-10-15T10:00:00+00:00"})
    else:
        rows.append({"kind": "ledger_unknown"})
    rows.append({"kind": "role_switch", "active": True, "login": "star_api_ro"})
    for schema, name, column, expected_using in probe.LOCK_GRANTS:
        rows.append(
            {
                "kind": "lock_grant",
                "schema": schema,
                "name": name,
                "column": column,
                "expected_using": expected_using,
                "owner_select": True,
                "owner_column_update": True,
                "restrictive_update_denial": True,
            }
        )
    for schema, name in probe.EXPECTED_OBJECTS:
        privilege_names = ("select", "insert", "update", "delete", "truncate", "references", "trigger")
        privilege_values = probe.OWNER_TABLE_PRIVILEGES[(schema, name)]
        rows.append(
            {
                "kind": "object",
                "schema": schema,
                "name": name,
                "present": True,
                "row_security": rls_value,
                "force_row_security": rls_value,
                "owner": "star_engineering_directory_owner",
                "policy_count": 1,
                "login_effective_privileges": {"select": True, "insert": False, "update": False, "delete": False},
                "owner_effective_privileges": dict(zip(privilege_names, privilege_values)),
                "owner_column_valid_to_update": (schema, name) == ("multica", "engineering_run_worktree_binding"),
                "owner_column_expires_at_update": (schema, name) == ("multica", "engineering_run_command_idempotency"),
            }
        )
    return "\n".join(json.dumps(row, separators=(",", ":")) for row in rows) + "\n"


class Wld0TargetDatabaseProbeTests(unittest.TestCase):
    def test_missing_service_and_identity_do_not_spawn(self) -> None:
        args = argparse.Namespace(service=None, expected_database=None, expected_login=None, expected_address=None, expected_port=None)
        runner = Mock()
        result, code = probe.run_probe(args, which=Mock(), runner=runner)
        self.assertEqual((result["status"], result["reason"], code), ("BLOCKED", "missing_or_invalid_service_name", 2))
        runner.assert_not_called()

    def test_service_name_rejects_connection_text(self) -> None:
        args = argparse.Namespace(**{**vars(EXPECTED), "service": "safe;host=attacker"})
        result, code = probe.run_probe(args, which=Mock(), runner=Mock())
        self.assertEqual(result["reason"], "missing_or_invalid_service_name")
        self.assertEqual(code, 2)

    def test_missing_psql_is_blocked(self) -> None:
        runner = Mock()
        result, code = probe.run_probe(EXPECTED, which=lambda _name: None, runner=runner)
        self.assertEqual((result["status"], result["reason"], code), ("BLOCKED", "psql_not_found", 2))
        runner.assert_not_called()

    def test_command_is_fixed_and_read_only(self) -> None:
        completed = subprocess.CompletedProcess([], 0, snapshot_output(), "")
        runner = Mock(return_value=completed)
        result, code = probe.run_probe(EXPECTED, which=lambda _name: "C:\\PostgreSQL\\bin\\psql.exe", runner=runner)
        self.assertEqual(result["status"], "OBSERVED_READ_ONLY")
        self.assertEqual(code, 0)
        call = runner.call_args
        command = call.args[0]
        self.assertIn("--dbname=service=wld0_target_ro", command)
        self.assertIn("-w", command)
        self.assertNotIn("--command", command)
        self.assertFalse(call.kwargs["shell"])
        self.assertIs(call.kwargs["stderr"], subprocess.DEVNULL)
        self.assertEqual(call.kwargs["timeout"], probe.PSQL_TIMEOUT_SECONDS)
        self.assertEqual(call.kwargs["input"], probe.PSQL_SCRIPT)
        self.assertIn("BEGIN ISOLATION LEVEL REPEATABLE READ READ ONLY", probe.PSQL_SCRIPT)
        self.assertIn("statement_timeout = '4000ms'", probe.PSQL_SCRIPT)
        self.assertIn("LIMIT 257", probe.PSQL_SCRIPT)
        self.assertNotIn("CREATE TABLE", probe.PSQL_SCRIPT.upper())
        self.assertNotIn("INSERT INTO", probe.PSQL_SCRIPT.upper())

    def test_errors_do_not_echo_stderr(self) -> None:
        secret = "SENTINEL_SECRET_FROM_LIBPQ"
        runner = Mock(return_value=subprocess.CompletedProcess([], 1, "", secret))
        result, code = probe.run_probe(EXPECTED, which=lambda _name: "psql", runner=runner)
        rendered = json.dumps(result)
        self.assertEqual((result["status"], code), ("BLOCKED", 2))
        self.assertNotIn(secret, rendered)

    def test_timeout_is_blocked(self) -> None:
        runner = Mock(side_effect=subprocess.TimeoutExpired("psql", probe.PSQL_TIMEOUT_SECONDS, stderr="SENTINEL"))
        result, code = probe.run_probe(EXPECTED, which=lambda _name: "psql", runner=runner)
        self.assertEqual((result["status"], result["reason"], code), ("BLOCKED", "probe_timeout", 2))
        self.assertNotIn("SENTINEL", json.dumps(result))

    def test_unknown_ledger_is_not_reconciled(self) -> None:
        result, code = probe.run_probe(
            EXPECTED,
            which=lambda _name: "psql",
            runner=Mock(return_value=subprocess.CompletedProcess([], 0, snapshot_output(ledger_known=False), "")),
        )
        self.assertEqual((result["status"], result["reason"], code), ("UNKNOWN", "missing_or_unrecognized_migration_ledger", 3))
        self.assertNotIn("migration_reconciliation", result)

    def test_truncated_migration_version_is_unknown(self) -> None:
        rows = [json.loads(line) for line in snapshot_output().splitlines()]
        for row in rows:
            if row["kind"] == "ledger_entry":
                row["version_truncated"] = True
        stdout = "\n".join(json.dumps(row) for row in rows)
        result, code = probe.run_probe(
            EXPECTED,
            which=lambda _name: "psql",
            runner=Mock(return_value=subprocess.CompletedProcess([], 0, stdout, "")),
        )
        self.assertEqual((result["status"], result["reason"], code), ("UNKNOWN", "migration_ledger_snapshot_truncated", 3))

    def test_unknown_rls_is_not_accepted(self) -> None:
        stdout = snapshot_output(rls_value=None)
        completed = subprocess.CompletedProcess([], 0, stdout, "")
        result, code = probe.run_probe(EXPECTED, which=lambda _name: "psql", runner=Mock(return_value=completed))
        self.assertEqual((result["status"], result["reason"], code), ("UNKNOWN", "wld_rls_catalog_state_unavailable", 3))

    def test_disabled_rls_is_blocked(self) -> None:
        result, code = probe.run_probe(
            EXPECTED,
            which=lambda _name: "psql",
            runner=Mock(return_value=subprocess.CompletedProcess([], 0, snapshot_output(rls_value=False), "")),
        )
        self.assertEqual((result["status"], result["reason"], code), ("BLOCKED", "wld_rls_or_force_rls_disabled", 2))

    def test_complete_fixture_is_observation_only(self) -> None:
        result, code = probe.run_probe(
            EXPECTED,
            which=lambda _name: "psql",
            runner=Mock(return_value=subprocess.CompletedProcess([], 0, snapshot_output(), "")),
        )
        self.assertEqual(code, 0)
        self.assertEqual(result["migration_reconciliation"]["status"], "BLOCKED")
        self.assertEqual(result["authenticated_binding_checks"], "NOT_PERFORMED_READ_ONLY_PROBE")
        self.assertFalse(result["ledger"]["stores_source_checksum"])

    def test_failed_owner_role_switch_is_blocked(self) -> None:
        rows = [json.loads(line) for line in snapshot_output().splitlines()]
        for row in rows:
            if row["kind"] == "role_switch":
                row["active"] = False
        stdout = "\n".join(json.dumps(row) for row in rows)
        result, code = probe.run_probe(
            EXPECTED,
            which=lambda _name: "psql",
            runner=Mock(return_value=subprocess.CompletedProcess([], 0, stdout, "")),
        )
        self.assertEqual((result["status"], result["reason"], code), ("BLOCKED", "authenticated_owner_role_switch_failed", 2))

    def test_elevated_owner_role_membership_is_blocked(self) -> None:
        rows = [json.loads(line) for line in snapshot_output().splitlines()]
        for row in rows:
            if row["kind"] == "owner_role":
                row["has_memberships"] = True
        stdout = "\n".join(json.dumps(row) for row in rows)
        result, code = probe.run_probe(
            EXPECTED,
            which=lambda _name: "psql",
            runner=Mock(return_value=subprocess.CompletedProcess([], 0, stdout, "")),
        )
        self.assertEqual((result["status"], result["reason"], code), ("BLOCKED", "engineering_directory_owner_role_attributes", 2))

    def test_unexpected_owner_table_privilege_is_blocked(self) -> None:
        rows = [json.loads(line) for line in snapshot_output().splitlines()]
        for row in rows:
            if row["kind"] == "object" and row["name"] == "engineering_run_outbox":
                row["owner_effective_privileges"]["select"] = True
        stdout = "\n".join(json.dumps(row) for row in rows)
        result, code = probe.run_probe(
            EXPECTED,
            which=lambda _name: "psql",
            runner=Mock(return_value=subprocess.CompletedProcess([], 0, stdout, "")),
        )
        self.assertEqual((result["status"], result["reason"], code), ("BLOCKED", "owner_table_privilege_contract_mismatch", 2))

    def test_identity_mismatch_does_not_disclose_observed_identity(self) -> None:
        rows = [json.loads(line) for line in snapshot_output().splitlines()]
        for row in rows:
            if row["kind"] == "identity":
                row["database"] = "unexpected_database"
        stdout = "\n".join(json.dumps(row) for row in rows)
        result, code = probe.run_probe(
            EXPECTED,
            which=lambda _name: "psql",
            runner=Mock(return_value=subprocess.CompletedProcess([], 0, stdout, "")),
        )
        self.assertEqual((result["status"], result["reason"], code), ("BLOCKED", "target_identity_mismatch", 2))
        self.assertNotIn("unexpected_database", json.dumps(result))


if __name__ == "__main__":
    unittest.main()
