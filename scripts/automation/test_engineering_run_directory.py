# @cypher schema=1 source_sha256=eb799ff8880b85c13afccd662c6df15187d8f097d278e34780348af43fca53c6
# MERGE (self:File {path:"scripts/automation/test_engineering_run_directory.py"})
# MERGE (suite:Symbol {id:"scripts/automation/test_engineering_run_directory.py::DirectoryMigrationPlanTests",kind:"class"})
# MERGE (reconciliation_suite:Symbol {id:"scripts/automation/test_engineering_run_directory.py::MigrationReconciliationTests",kind:"class"})
# MERGE (classify_helper:Symbol {id:"scripts/automation/test_engineering_run_directory.py::MigrationReconciliationTests.classify",kind:"method"})
# MERGE (base_plan:Symbol {id:"scripts/automation/test_engineering_run_directory.py::DirectoryMigrationPlanTests.test_base_plan_preserves_existing_chain",kind:"test"})
# MERGE (binding_plan:Symbol {id:"scripts/automation/test_engineering_run_directory.py::DirectoryMigrationPlanTests.test_run_binding_migrations_are_ordered_and_single_apply",kind:"test"})
# MERGE (role_acceptance:Symbol {id:"scripts/automation/test_engineering_run_directory.py::DirectoryMigrationPlanTests.test_role_acceptance_covers_all_lock_sources",kind:"test"})
# MERGE (grant_fingerprint:Symbol {id:"scripts/automation/test_engineering_run_directory.py::DirectoryMigrationPlanTests.test_fingerprint_includes_owner_grants",kind:"test"})
# MERGE (stop_on_error:Symbol {id:"scripts/automation/test_engineering_run_directory.py::DirectoryMigrationPlanTests.test_migration_failure_stops_following_steps",kind:"test"})
# MERGE (fingerprint_failure:Symbol {id:"scripts/automation/test_engineering_run_directory.py::DirectoryMigrationPlanTests.test_fingerprint_failure_stops_following_steps",kind:"test"})
# MERGE (apply_step:Symbol {id:"scripts/automation/test_engineering_run_directory.py::DirectoryMigrationPlanTests.test_migration_failure_stops_following_steps.apply_step",kind:"function"})
# MERGE (fingerprint_step:Symbol {id:"scripts/automation/test_engineering_run_directory.py::DirectoryMigrationPlanTests.test_migration_failure_stops_following_steps.fingerprint_step",kind:"function"})
# MERGE (fingerprint_apply_step:Symbol {id:"scripts/automation/test_engineering_run_directory.py::DirectoryMigrationPlanTests.test_fingerprint_failure_stops_following_steps.apply_step",kind:"function"})
# MERGE (fingerprint_probe:Symbol {id:"scripts/automation/test_engineering_run_directory.py::DirectoryMigrationPlanTests.test_fingerprint_failure_stops_following_steps.fingerprint_step",kind:"function"})
# MERGE (safe_predecessor:Symbol {id:"scripts/automation/test_engineering_run_directory.py::MigrationReconciliationTests.test_exact_predecessor_chain_allows_apply",kind:"test"})
# MERGE (exact_applied:Symbol {id:"scripts/automation/test_engineering_run_directory.py::MigrationReconciliationTests.test_exact_applied_ledger_and_catalog_is_skipped",kind:"test"})
# MERGE (ambiguous_committed:Symbol {id:"scripts/automation/test_engineering_run_directory.py::MigrationReconciliationTests.test_ambiguous_apply_is_reconciled_only_by_exact_later_read",kind:"test"})
# MERGE (ambiguous_uncommitted:Symbol {id:"scripts/automation/test_engineering_run_directory.py::MigrationReconciliationTests.test_ambiguous_apply_with_exact_predecessor_is_safe_to_retry",kind:"test"})
# MERGE (no_ledger_post:Symbol {id:"scripts/automation/test_engineering_run_directory.py::MigrationReconciliationTests.test_missing_ledger_with_post_fingerprint_fails_closed",kind:"test"})
# MERGE (partial_catalog:Symbol {id:"scripts/automation/test_engineering_run_directory.py::MigrationReconciliationTests.test_partial_catalog_fails_closed",kind:"test"})
# MERGE (ledger_mismatch:Symbol {id:"scripts/automation/test_engineering_run_directory.py::MigrationReconciliationTests.test_migration_checksum_mismatch_fails_closed",kind:"test"})
# MERGE (incomplete_snapshot:Symbol {id:"scripts/automation/test_engineering_run_directory.py::MigrationReconciliationTests.test_incomplete_snapshot_fails_closed",kind:"test"})
# MERGE (manifest_whitespace:Symbol {id:"scripts/automation/test_engineering_run_directory.py::MigrationReconciliationTests.test_whitespace_only_manifest_values_fail_closed",kind:"test"})
# MERGE (catalog_whitespace:Symbol {id:"scripts/automation/test_engineering_run_directory.py::MigrationReconciliationTests.test_whitespace_only_catalog_fingerprint_fails_closed",kind:"test"})
# MERGE (successor_prefix:Symbol {id:"scripts/automation/test_engineering_run_directory.py::MigrationReconciliationTests.test_expected_successor_prefix_is_accepted",kind:"test"})
# MERGE (unapproved_successor:Symbol {id:"scripts/automation/test_engineering_run_directory.py::MigrationReconciliationTests.test_unapproved_successor_history_is_reported_separately",kind:"test"})
# MERGE (successor_catalog_mismatch:Symbol {id:"scripts/automation/test_engineering_run_directory.py::MigrationReconciliationTests.test_expected_successor_catalog_mismatch_fails_closed",kind:"test"})
# MERGE (assert_equal:ExternalService {id:"python.unittest.TestCase.assertEqual"})
# MERGE (unittest:ExternalService {id:"python.unittest"})
# MERGE (runner:File {path:"scripts/automation/engineering_run_directory.py"})
# MERGE (migration_plan:Symbol {id:"scripts/automation/engineering_run_directory.py::directory_migration_plan",kind:"function"})
# MERGE (apply_plan:Symbol {id:"scripts/automation/engineering_run_directory.py::apply_migration_plan",kind:"function"})
# MERGE (reconcile:Symbol {id:"scripts/automation/engineering_run_directory.py::classify_migration_reconciliation",kind:"function"})
# MERGE (self)-[:DEFINES]->(suite)
# MERGE (suite)-[:HAS_METHOD]->(base_plan)
# MERGE (suite)-[:HAS_METHOD]->(binding_plan)
# MERGE (suite)-[:HAS_METHOD]->(role_acceptance)
# MERGE (suite)-[:HAS_METHOD]->(grant_fingerprint)
# MERGE (suite)-[:HAS_METHOD]->(stop_on_error)
# MERGE (suite)-[:HAS_METHOD]->(fingerprint_failure)
# MERGE (self)-[:DEFINES]->(reconciliation_suite)
# MERGE (reconciliation_suite)-[:HAS_METHOD]->(classify_helper)
# MERGE (reconciliation_suite)-[:HAS_METHOD]->(safe_predecessor)
# MERGE (reconciliation_suite)-[:HAS_METHOD]->(exact_applied)
# MERGE (reconciliation_suite)-[:HAS_METHOD]->(ambiguous_committed)
# MERGE (reconciliation_suite)-[:HAS_METHOD]->(ambiguous_uncommitted)
# MERGE (reconciliation_suite)-[:HAS_METHOD]->(no_ledger_post)
# MERGE (reconciliation_suite)-[:HAS_METHOD]->(partial_catalog)
# MERGE (reconciliation_suite)-[:HAS_METHOD]->(ledger_mismatch)
# MERGE (reconciliation_suite)-[:HAS_METHOD]->(incomplete_snapshot)
# MERGE (self)-[:TESTS]->(runner)
# MERGE (self)-[:IMPORTS]->(unittest)
# MERGE (self)-[:IMPORTS]->(runner)
# MERGE (suite)-[:EXTENDS]->(unittest)
# MERGE (base_plan)-[:CALLS]->(assert_equal)
# MERGE (binding_plan)-[:CALLS]->(assert_equal)
# MERGE (role_acceptance)-[:CALLS]->(assert_equal)
# MERGE (grant_fingerprint)-[:CALLS]->(assert_equal)
# MERGE (stop_on_error)-[:CONTAINS]->(apply_step)
# MERGE (stop_on_error)-[:CONTAINS]->(fingerprint_step)
# MERGE (fingerprint_failure)-[:CONTAINS]->(fingerprint_apply_step)
# MERGE (fingerprint_failure)-[:CONTAINS]->(fingerprint_probe)
# MERGE (stop_on_error)-[:CALLS]->(assert_equal)
# MERGE (base_plan)-[:CALLS]->(migration_plan)
# MERGE (binding_plan)-[:CALLS]->(migration_plan)
# MERGE (stop_on_error)-[:CALLS]->(apply_plan)
# MERGE (fingerprint_failure)-[:CALLS]->(apply_plan)
# MERGE (self)-[:TESTS]->(reconcile)
# MERGE (classify_helper)-[:CALLS]->(reconcile)
# MERGE (safe_predecessor)-[:CALLS]->(classify_helper)
# MERGE (exact_applied)-[:CALLS]->(classify_helper)
# MERGE (ambiguous_committed)-[:CALLS]->(classify_helper)
# MERGE (ambiguous_uncommitted)-[:CALLS]->(classify_helper)
# MERGE (no_ledger_post)-[:CALLS]->(classify_helper)
# MERGE (partial_catalog)-[:CALLS]->(classify_helper)
# MERGE (ledger_mismatch)-[:CALLS]->(classify_helper)
# MERGE (incomplete_snapshot)-[:CALLS]->(classify_helper)
# MERGE (manifest_whitespace)-[:CALLS]->(classify_helper)
# MERGE (catalog_whitespace)-[:CALLS]->(classify_helper)
# MERGE (successor_prefix)-[:CALLS]->(classify_helper)
# MERGE (unapproved_successor)-[:CALLS]->(classify_helper)
# MERGE (successor_catalog_mismatch)-[:CALLS]->(classify_helper)
# @endcypher

from __future__ import annotations

import unittest

import engineering_run_directory as runner


class DirectoryMigrationPlanTests(unittest.TestCase):
    def test_base_plan_preserves_existing_chain(self) -> None:
        self.assertEqual(
            runner.directory_migration_plan(),
            [
                ("postgres-worktree-base", "2026-09-16-worktree-canvas-worktree.sql"),
                ("postgres-project-binding", "2026-09-29-worktree-group-phase-2b.sql"),
                ("postgres-directory-first", "2026-10-01-engineering-run-directory.sql"),
                ("postgres-directory-repeat", "2026-10-01-engineering-run-directory.sql"),
            ],
        )

    def test_run_binding_migrations_are_ordered_and_single_apply(self) -> None:
        migrations = runner.directory_migration_plan(run_binding=True)
        names = [migration for _, migration in migrations]

        self.assertIn("2026-09-29-worktree-group-phase-2c.sql", names)
        self.assertLess(names.index("2026-09-29-worktree-group-phase-2c.sql"),
                        names.index("2026-10-15-run-host-independent-directory.sql"))
        self.assertEqual(names[-5:], [
            "2026-10-02-task-run-owner.sql",
            "2026-10-15-run-host-independent-directory.sql",
            "2026-10-19-run-worktree-bindings.sql",
            "2026-10-15-run-host-independent-task-owner.sql",
            "2026-10-20-engineering-directory-owner-role.sql",
        ])
        self.assertEqual(names.count("2026-09-29-worktree-group-phase-2c.sql"), 1)
        self.assertEqual(names.count("2026-10-15-run-host-independent-directory.sql"), 1)
        self.assertEqual(names.count("2026-10-19-run-worktree-bindings.sql"), 1)
        self.assertEqual(names.count("2026-10-02-task-run-owner.sql"), 1)
        self.assertEqual(names.count("2026-10-15-run-host-independent-task-owner.sql"), 1)
        self.assertEqual(names.count("2026-10-20-engineering-directory-owner-role.sql"), 1)

    def test_role_acceptance_covers_all_lock_sources(self) -> None:
        sql = runner.run_binding_role_acceptance_sql()
        policy_table_list = (
            sql.split("expected_using := CASE", 1)[0]
            .rsplit("FOREACH table_name IN ARRAY ARRAY[", 1)[1]
            .split("] LOOP", 1)[0]
        )
        policy_tables = {
            item.strip().strip("' ")
            for item in policy_table_list.replace("\n", "").split(",")
            if item.strip()
        }
        self.assertEqual(
            policy_tables,
            {
                "permission.project_role_binding",
                "scm.cloud_branch_revision",
                "permission.cloud_branch_role_binding",
                "permission.engineering_run_role_binding",
                "multica.engineering_run_revision",
                "multica.worktree_project_binding",
                "public.worktree_canvas_worktree",
                "multica.task_metadata",
                "multica.work_item_worktree",
            },
        )
        policy_expectations = sql.split("expected_using := CASE", 1)[1].split("END;", 1)[0]
        lock_only_case_list = (
            policy_expectations.split("THEN 'true'", 1)[0]
            .rsplit("ARRAY[", 1)[1]
            .split("]::TEXT[]", 1)[0]
        )
        lock_only_relations = {
            item.strip().strip("' ")
            for item in lock_only_case_list.replace("\n", "").split(",")
            if item.strip()
        }
        self.assertEqual(
            lock_only_relations,
            {
                "permission.project_role_binding",
                "permission.engineering_run_role_binding",
                "multica.engineering_run_revision",
                "multica.worktree_project_binding",
                "public.worktree_canvas_worktree",
            },
        )
        self.assertIn("ELSE 'false'", policy_expectations)
        self.assertIn("pg_get_expr(p.polqual,p.polrelid)=expected_using", sql)
        self.assertIn("pg_get_expr(p.polwithcheck,p.polrelid)='false'", sql)
        self.assertIn("EXCEPTION WHEN SQLSTATE '42501'", sql)
        self.assertIn("update_sqlstate IS DISTINCT FROM '42501'", sql)
        self.assertIn("lock-only UPDATE changed or hid Project membership", sql)
        for relation in (
            "permission.engineering_run_role_binding",
            "multica.engineering_run_revision",
            "multica.task_metadata",
            "multica.work_item_worktree",
        ):
            self.assertIn(relation, sql)
        self.assertIn("SET LOCAL ROLE star_engineering_directory_owner", sql)
        self.assertIn("restrictive RLS", sql)
        self.assertIn("directory owner must not be a member of another database role", sql)
        self.assertIn("Task metadata RLS allowed reads without tenant context", sql)

    def test_fingerprint_includes_owner_grants(self) -> None:
        sql = runner.migration_fingerprint_sql()
        self.assertIn("information_schema.table_privileges", sql)
        self.assertIn("information_schema.column_privileges", sql)
        self.assertIn("information_schema.routine_privileges", sql)
        self.assertIn("aclexplode", sql)
        self.assertIn("database-grant:", sql)
        self.assertIn("star_engineering_directory_owner", sql)
        for role_attribute in (
            "rolcanlogin",
            "rolsuper",
            "rolcreatedb",
            "rolcreaterole",
            "rolreplication",
            "rolbypassrls",
            "rolinherit",
        ):
            self.assertIn(role_attribute, sql)
        migration = (runner.ROOT / "db" / "migrations" /
                     "2026-10-20-engineering-directory-owner-role.sql").read_text(encoding="utf-8")
        for normalization_rule in (
            "REVOKE ALL PRIVILEGES ON SCHEMA",
            "REVOKE ALL PRIVILEGES ON ALL TABLES",
            "REVOKE ALL PRIVILEGES ON ALL SEQUENCES",
            "REVOKE ALL PRIVILEGES ON ALL ROUTINES",
            "REVOKE CONNECT, CREATE, TEMPORARY ON DATABASE",
            "FROM pg_auth_members",
            "FROM pg_shdepend",
            "must not own database objects",
        ):
            self.assertIn(normalization_rule, migration)
        self.assertIn("deptype = 'o'", migration)
        self.assertIn("engineering_directory_owner_owned_fixture",
                      runner.run_binding_role_ownership_fixture_sql())

    def test_migration_failure_stops_following_steps(self) -> None:
        applied: list[str] = []
        fingerprinted: list[str] = []

        def apply_step(name: str, migration: str) -> dict:
            applied.append(migration)
            return {"name": name, "exit_code": 1 if name == "failing" else 0}

        def fingerprint_step(name: str, migration: str) -> dict:
            fingerprinted.append(migration)
            return {"name": f"{name}-fingerprint", "exit_code": 0}

        result = runner.apply_migration_plan(
            [("first", "first.sql"), ("failing", "failing.sql"), ("never", "never.sql")],
            apply_step,
            fingerprint_step,
        )

        self.assertEqual(applied, ["first.sql", "failing.sql"])
        self.assertEqual(fingerprinted, ["first.sql"])
        self.assertEqual([row["exit_code"] for row in result], [0, 0, 1])

    def test_fingerprint_failure_stops_following_steps(self) -> None:
        applied: list[str] = []
        fingerprinted: list[str] = []

        def apply_step(name: str, migration: str) -> dict:
            applied.append(migration)
            return {"name": name, "exit_code": 0}

        def fingerprint_step(name: str, migration: str) -> dict:
            fingerprinted.append(migration)
            return {"name": f"{name}-fingerprint", "exit_code": 1}

        result = runner.apply_migration_plan(
            [("first", "first.sql"), ("never", "never.sql")],
            apply_step,
            fingerprint_step,
        )

        self.assertEqual(applied, ["first.sql"])
        self.assertEqual(fingerprinted, ["first.sql"])
        self.assertEqual([row["exit_code"] for row in result], [0, 1])


class MigrationReconciliationTests(unittest.TestCase):
    PREDECESSORS = [
        ("base.sql", "base-checksum", "predecessor-fingerprint"),
    ]

    def classify(
        self,
        ledger,
        catalog,
        *,
        complete: bool = True,
        ambiguous: bool = False,
        migration_name: str = "2026-10-15-run-host-independent-directory.sql",
        migration_checksum: str = "run-directory-checksum",
        expected_predecessors: list[runner.MigrationLedgerEntry] | None = None,
        expected_predecessor_fingerprint: str | None = "predecessor-fingerprint",
        expected_applied_fingerprint: str | None = "applied-fingerprint",
        expected_successors: list[runner.MigrationLedgerEntry] | None = None,
    ):
        predecessors = self.PREDECESSORS if expected_predecessors is None else expected_predecessors
        return runner.classify_migration_reconciliation(
            migration_name,
            migration_checksum,
            predecessors,
            ledger,
            expected_predecessor_fingerprint,
            expected_applied_fingerprint,
            catalog,
            expected_successors,
            snapshot_complete=complete,
            apply_outcome_ambiguous=ambiguous,
        )

    def test_exact_predecessor_chain_allows_apply(self) -> None:
        result = self.classify(self.PREDECESSORS, "predecessor-fingerprint")
        self.assertEqual(result["status"], "safe_to_apply")
        self.assertEqual(result["action"], "apply")
        self.assertTrue(result["safe_to_apply"])

    def test_exact_applied_ledger_and_catalog_is_skipped(self) -> None:
        ledger = self.PREDECESSORS + [
            ("2026-10-15-run-host-independent-directory.sql",
             "run-directory-checksum", "applied-fingerprint"),
        ]
        result = self.classify(ledger, "applied-fingerprint")
        self.assertEqual(result["status"], "already_applied")
        self.assertEqual(result["action"], "skip")
        self.assertTrue(result["committed"])

    def test_ambiguous_apply_is_reconciled_only_by_exact_later_read(self) -> None:
        ledger = self.PREDECESSORS + [
            ("2026-10-15-run-host-independent-directory.sql",
             "run-directory-checksum", "applied-fingerprint"),
        ]
        result = self.classify(ledger, "applied-fingerprint", ambiguous=True)
        self.assertEqual(result["status"], "committed_after_ambiguous_apply")
        self.assertTrue(result["committed"])
        self.assertFalse(result["safe_to_apply"])

    def test_ambiguous_apply_with_exact_predecessor_is_safe_to_retry(self) -> None:
        result = self.classify(self.PREDECESSORS, "predecessor-fingerprint", ambiguous=True)
        self.assertEqual(result["status"], "safe_to_apply")
        self.assertTrue(result["safe_to_apply"])

    def test_missing_ledger_with_post_fingerprint_fails_closed(self) -> None:
        result = self.classify(self.PREDECESSORS, "applied-fingerprint", ambiguous=True)
        self.assertEqual(result["status"], "blocked")
        self.assertEqual(result["reason"], "ledger_catalog_disagreement_target_fingerprint_without_entry")
        self.assertEqual(result["action"], "stop")

    def test_partial_catalog_fails_closed(self) -> None:
        result = self.classify(self.PREDECESSORS, "partial-fingerprint")
        self.assertEqual(result["status"], "blocked")
        self.assertEqual(result["reason"], "partial_or_unrecognized_catalog_state")

    def test_migration_checksum_mismatch_fails_closed(self) -> None:
        ledger = self.PREDECESSORS + [
            ("2026-10-15-run-host-independent-directory.sql",
             "different-checksum", "applied-fingerprint"),
        ]
        result = self.classify(ledger, "applied-fingerprint")
        self.assertEqual(result["status"], "blocked")
        self.assertEqual(result["reason"], "ledger_catalog_disagreement_migration_entry_mismatch")

    def test_incomplete_snapshot_fails_closed(self) -> None:
        result = self.classify(self.PREDECESSORS, "predecessor-fingerprint", complete=False)
        self.assertEqual(result["status"], "blocked")
        self.assertEqual(result["reason"], "unknown_incomplete_snapshot")

    def test_whitespace_only_manifest_values_fail_closed(self) -> None:
        invalid_values = {
            "migration_name": "  ",
            "migration_checksum": "\t",
            "expected_predecessor_fingerprint": "  ",
            "expected_applied_fingerprint": "\n",
        }
        for field, value in invalid_values.items():
            with self.subTest(field=field):
                result = self.classify(
                    self.PREDECESSORS,
                    "predecessor-fingerprint",
                    **{field: value},
                )
                self.assertEqual(result["status"], "blocked")
                self.assertEqual(result["reason"], "unknown_incomplete_expected_manifest")

        invalid_successor = [("  ", "checksum", "fingerprint")]
        successor_result = self.classify(
            self.PREDECESSORS,
            "predecessor-fingerprint",
            expected_successors=invalid_successor,
        )
        self.assertEqual(successor_result["reason"], "unknown_invalid_successor_manifest")

    def test_whitespace_only_catalog_fingerprint_fails_closed(self) -> None:
        result = self.classify(self.PREDECESSORS, " \t\n ")
        self.assertEqual(result["status"], "blocked")
        self.assertEqual(result["reason"], "unknown_missing_ledger_or_catalog")

    def test_expected_successor_prefix_is_accepted(self) -> None:
        target = (
            "2026-10-15-run-host-independent-directory.sql",
            "run-directory-checksum",
            "applied-fingerprint",
        )
        successors = [
            ("2026-10-19-run-worktree-bindings.sql", "binding-checksum", "binding-fingerprint"),
            ("2026-10-20-engineering-directory-owner-role.sql", "role-checksum", "role-fingerprint"),
        ]
        result = self.classify(
            self.PREDECESSORS + [target, successors[0]],
            "binding-fingerprint",
            expected_successors=successors,
        )
        self.assertEqual(result["status"], "already_applied")
        self.assertTrue(result["committed"])

    def test_unapproved_successor_history_is_reported_separately(self) -> None:
        target = (
            "2026-10-15-run-host-independent-directory.sql",
            "run-directory-checksum",
            "applied-fingerprint",
        )
        approved = ("2026-10-19-run-worktree-bindings.sql", "binding-checksum", "binding-fingerprint")
        unexpected = ("unknown-next.sql", "unknown-checksum", "unknown-fingerprint")
        result = self.classify(
            self.PREDECESSORS + [target, approved, unexpected],
            "unknown-fingerprint",
            expected_successors=[approved],
        )
        self.assertEqual(result["status"], "blocked")
        self.assertEqual(result["reason"], "ledger_history_contains_unapproved_successor")

    def test_expected_successor_catalog_mismatch_fails_closed(self) -> None:
        target = (
            "2026-10-15-run-host-independent-directory.sql",
            "run-directory-checksum",
            "applied-fingerprint",
        )
        successor = ("2026-10-19-run-worktree-bindings.sql", "binding-checksum", "binding-fingerprint")
        result = self.classify(
            self.PREDECESSORS + [target, successor],
            "unrecognized-fingerprint",
            expected_successors=[successor],
        )
        self.assertEqual(result["status"], "blocked")
        self.assertEqual(result["reason"], "partial_or_unrecognized_catalog_state")


if __name__ == "__main__":
    unittest.main()
