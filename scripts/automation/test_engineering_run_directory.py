# @cypher schema=1 source_sha256=e2b51e930771d826586a86022581e07076619e89c1e319c9bcfb629d15e95b6a
# MERGE (self:File {path:"scripts/automation/test_engineering_run_directory.py"})
# MERGE (suite:Symbol {id:"scripts/automation/test_engineering_run_directory.py::DirectoryMigrationPlanTests",kind:"class"})
# MERGE (base_plan:Symbol {id:"scripts/automation/test_engineering_run_directory.py::DirectoryMigrationPlanTests.test_base_plan_preserves_existing_chain",kind:"test"})
# MERGE (binding_plan:Symbol {id:"scripts/automation/test_engineering_run_directory.py::DirectoryMigrationPlanTests.test_run_binding_migrations_are_ordered_and_single_apply",kind:"test"})
# MERGE (stop_on_error:Symbol {id:"scripts/automation/test_engineering_run_directory.py::DirectoryMigrationPlanTests.test_migration_failure_stops_following_steps",kind:"test"})
# MERGE (fingerprint_failure:Symbol {id:"scripts/automation/test_engineering_run_directory.py::DirectoryMigrationPlanTests.test_fingerprint_failure_stops_following_steps",kind:"test"})
# MERGE (apply_step:Symbol {id:"scripts/automation/test_engineering_run_directory.py::DirectoryMigrationPlanTests.test_migration_failure_stops_following_steps.apply_step",kind:"function"})
# MERGE (fingerprint_step:Symbol {id:"scripts/automation/test_engineering_run_directory.py::DirectoryMigrationPlanTests.test_migration_failure_stops_following_steps.fingerprint_step",kind:"function"})
# MERGE (fingerprint_apply_step:Symbol {id:"scripts/automation/test_engineering_run_directory.py::DirectoryMigrationPlanTests.test_fingerprint_failure_stops_following_steps.apply_step",kind:"function"})
# MERGE (fingerprint_probe:Symbol {id:"scripts/automation/test_engineering_run_directory.py::DirectoryMigrationPlanTests.test_fingerprint_failure_stops_following_steps.fingerprint_step",kind:"function"})
# MERGE (assert_equal:ExternalService {id:"python.unittest.TestCase.assertEqual"})
# MERGE (unittest:ExternalService {id:"python.unittest"})
# MERGE (runner:File {path:"scripts/automation/engineering_run_directory.py"})
# MERGE (migration_plan:Symbol {id:"scripts/automation/engineering_run_directory.py::directory_migration_plan",kind:"function"})
# MERGE (apply_plan:Symbol {id:"scripts/automation/engineering_run_directory.py::apply_migration_plan",kind:"function"})
# MERGE (self)-[:DEFINES]->(suite)
# MERGE (suite)-[:HAS_METHOD]->(base_plan)
# MERGE (suite)-[:HAS_METHOD]->(binding_plan)
# MERGE (suite)-[:HAS_METHOD]->(stop_on_error)
# MERGE (suite)-[:HAS_METHOD]->(fingerprint_failure)
# MERGE (self)-[:TESTS]->(runner)
# MERGE (self)-[:IMPORTS]->(unittest)
# MERGE (self)-[:IMPORTS]->(runner)
# MERGE (suite)-[:EXTENDS]->(unittest)
# MERGE (base_plan)-[:CALLS]->(assert_equal)
# MERGE (binding_plan)-[:CALLS]->(assert_equal)
# MERGE (stop_on_error)-[:CONTAINS]->(apply_step)
# MERGE (stop_on_error)-[:CONTAINS]->(fingerprint_step)
# MERGE (fingerprint_failure)-[:CONTAINS]->(fingerprint_apply_step)
# MERGE (fingerprint_failure)-[:CONTAINS]->(fingerprint_probe)
# MERGE (stop_on_error)-[:CALLS]->(assert_equal)
# MERGE (base_plan)-[:CALLS]->(migration_plan)
# MERGE (binding_plan)-[:CALLS]->(migration_plan)
# MERGE (stop_on_error)-[:CALLS]->(apply_plan)
# MERGE (fingerprint_failure)-[:CALLS]->(apply_plan)
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

        self.assertEqual(names[-2:], [
            "2026-10-15-run-host-independent-directory.sql",
            "2026-10-19-run-worktree-bindings.sql",
        ])
        self.assertEqual(names.count("2026-10-15-run-host-independent-directory.sql"), 1)
        self.assertEqual(names.count("2026-10-19-run-worktree-bindings.sql"), 1)

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


if __name__ == "__main__":
    unittest.main()
