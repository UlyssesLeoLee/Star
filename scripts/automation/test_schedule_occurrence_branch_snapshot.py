# @cypher schema=1 source_sha256=7b70dd1594c89d3ee170e657bbeb50c902e1ea65406f5249ee706c19a553d16c
# MERGE (self:File {path:"scripts/automation/test_schedule_occurrence_branch_snapshot.py"})
# MERGE (suite:Symbol {id:"scripts/automation/test_schedule_occurrence_branch_snapshot.py::OccurrenceBranchSnapshotMigrationTests",kind:"class"})
# MERGE (migration_test:Symbol {id:"scripts/automation/test_schedule_occurrence_branch_snapshot.py::OccurrenceBranchSnapshotMigrationTests.test_historical_migration_is_preserved_and_forward_correction_is_idempotent",kind:"test"})
# MERGE (boundary_test:Symbol {id:"scripts/automation/test_schedule_occurrence_branch_snapshot.py::OccurrenceBranchSnapshotMigrationTests.test_harness_covers_valid_and_invalid_branch_snapshots",kind:"test"})
# MERGE (rule_test:Symbol {id:"scripts/automation/test_schedule_occurrence_branch_snapshot.py::OccurrenceBranchSnapshotMigrationTests.test_branchless_rule_remains_storable",kind:"test"})
# MERGE (unittest:ExternalService {id:"python.unittest",kind:"module"})
# MERGE (test_case:ExternalService {id:"python.unittest.TestCase",kind:"type"})
# MERGE (pathlib:ExternalService {id:"python.pathlib.Path",kind:"type"})
# MERGE (assert_in:ExternalService {id:"python.unittest.TestCase.assertIn",kind:"method"})
# MERGE (read_text:ExternalService {id:"python.pathlib.Path.read_text",kind:"method"})
# MERGE (migration:File {path:"db/migrations/2026-10-22-schedule-occurrence-branch-snapshot.sql"})
# MERGE (correction_migration:File {path:"db/migrations/2026-10-23-schedule-occurrence-branch-snapshot-correction.sql"})
# MERGE (harness:File {path:"scripts/automation/phase9f3_schedule.py"})
# MERGE (self)-[:DEFINES]->(suite)
# MERGE (self)-[:IMPORTS]->(unittest)
# MERGE (self)-[:IMPORTS]->(pathlib)
# MERGE (suite)-[:EXTENDS]->(test_case)
# MERGE (suite)-[:HAS_METHOD]->(migration_test)
# MERGE (suite)-[:HAS_METHOD]->(boundary_test)
# MERGE (suite)-[:HAS_METHOD]->(rule_test)
# MERGE (migration_test)-[:TESTS]->(migration)
# MERGE (migration_test)-[:TESTS]->(correction_migration)
# MERGE (boundary_test)-[:TESTS]->(harness)
# MERGE (rule_test)-[:TESTS]->(harness)
# MERGE (migration_test)-[:CALLS]->(assert_in)
# MERGE (migration_test)-[:CALLS]->(read_text)
# MERGE (boundary_test)-[:CALLS]->(assert_in)
# MERGE (boundary_test)-[:CALLS]->(read_text)
# MERGE (rule_test)-[:CALLS]->(assert_in)
# MERGE (rule_test)-[:CALLS]->(read_text)
# @endcypher

from __future__ import annotations

import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
HISTORICAL_MIGRATION = ROOT / "db/migrations/2026-10-22-schedule-occurrence-branch-snapshot.sql"
CORRECTION_MIGRATION = ROOT / "db/migrations/2026-10-23-schedule-occurrence-branch-snapshot-correction.sql"
HARNESS = ROOT / "scripts/automation/phase9f3_schedule.py"


class OccurrenceBranchSnapshotMigrationTests(unittest.TestCase):
    def test_historical_migration_is_preserved_and_forward_correction_is_idempotent(self) -> None:
        historical = HISTORICAL_MIGRATION.read_text(encoding="utf-8")
        correction = CORRECTION_MIGRATION.read_text(encoding="utf-8")
        self.assertIn("occurrence_target_snapshot_branch_uuid_check", historical)
        self.assertIn("IF NOT EXISTS (", historical)
        self.assertIn("{rule_snapshot,target,branch_id}", historical)
        self.assertIn("DROP CONSTRAINT IF EXISTS occurrence_target_snapshot_branch_uuid_check", correction)
        self.assertIn("ADD CONSTRAINT occurrence_target_snapshot_branch_uuid_check", correction)
        self.assertIn("NOT VALID", correction)
        self.assertIn("{branch_id}", correction)
        self.assertNotIn("{rule_snapshot,target,branch_id}", correction)
        self.assertIn("~*", correction)
        self.assertIn("THEN (target_snapshot #>> '{branch_id}')::uuid", correction)
        self.assertIn("ELSE FALSE", correction)
        self.assertIn("BEGIN;", correction)
        self.assertIn("COMMIT;", correction)
        self.assertNotIn("UPDATE automation.occurrence", correction)
        self.assertNotIn("VALIDATE CONSTRAINT", correction)

    def test_harness_covers_valid_and_invalid_branch_snapshots(self) -> None:
        source = HARNESS.read_text(encoding="utf-8")
        sql = CORRECTION_MIGRATION.read_text(encoding="utf-8")
        historical_path = "2026-10-22-schedule-occurrence-branch-snapshot.sql"
        correction_path = "2026-10-23-schedule-occurrence-branch-snapshot-correction.sql"
        self.assertIn(historical_path, source)
        self.assertIn(correction_path, source)
        self.assertLess(source.index(historical_path), source.index(correction_path))
        self.assertIn('for pass_name in ("first", "repeat")', source)
        self.assertIn('server_options = f"-h 127.0.0.1 -p {self.port}"', source)
        self.assertIn('if os.name != "nt":', source)
        self.assertIn('tempfile.mkdtemp(prefix="star-schedule-pg-")', source)
        self.assertIn('-k "{self.socket_dir}"', source)
        self.assertIn("is_contained(self.socket_dir, temp_root)", source)
        self.assertIn("stop_result = stop", source)
        self.assertIn("return stop_result", source)
        self.assertIn("if cleanup is not None:\n            steps.append(cleanup)", source)
        self.assertIn('"postgres-schedule-occurrence-branch-snapshot"', source)
        for invalid in (
            "'{}'::jsonb",
            '"branch_id":null',
            '"branch_id":""',
            '"branch_id":"not-a-uuid"',
            '"branch_id":"00000000-0000-0000-0000-000000000000"',
        ):
            self.assertIn(invalid, source)
        self.assertIn('"branch_id":"40000000-0000-4000-8000-000000000001"', source)
        self.assertIn('"branch_id":"ABCDEF12-3456-4ABC-8DEF-1234567890AB"', source)
        self.assertIn("CASE", sql)

    def test_branchless_rule_remains_storable(self) -> None:
        source = HARNESS.read_text(encoding="utf-8")
        self.assertIn("branch_id', NULL", source)
        self.assertIn("'changed_by', rule.run_as_actor_id", source)
        self.assertIn("'a0000000-0000-4000-8000-000000000001', clock_timestamp() + interval '1 hour'", source)
        self.assertIn("branchless Schedule Rule configuration must remain storable", source)
        self.assertIn("RETURNING rule_id, branch_id INTO hostless_rule_id, hostless_branch_id", source)


if __name__ == "__main__":
    unittest.main()
