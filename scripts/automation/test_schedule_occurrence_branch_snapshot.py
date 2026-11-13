# @cypher schema=1 source_sha256=d4ccc1cec92db4fa77e4232da073a2d08df86cac9de1ad6377aa4d259a835aab
# MERGE (self:File {path:"scripts/automation/test_schedule_occurrence_branch_snapshot.py"})
# MERGE (suite:Symbol {id:"scripts/automation/test_schedule_occurrence_branch_snapshot.py::OccurrenceBranchSnapshotMigrationTests",kind:"class"})
# MERGE (migration_test:Symbol {id:"scripts/automation/test_schedule_occurrence_branch_snapshot.py::OccurrenceBranchSnapshotMigrationTests.test_historical_migration_is_preserved_and_forward_correction_is_idempotent",kind:"test"})
# MERGE (boundary_test:Symbol {id:"scripts/automation/test_schedule_occurrence_branch_snapshot.py::OccurrenceBranchSnapshotMigrationTests.test_harness_covers_branch_optional_target_and_required_identities",kind:"test"})
# MERGE (rule_test:Symbol {id:"scripts/automation/test_schedule_occurrence_branch_snapshot.py::OccurrenceBranchSnapshotMigrationTests.test_branchless_rule_and_occurrence_remain_storable",kind:"test"})
# MERGE (unittest:ExternalService {id:"python.unittest",kind:"module"})
# MERGE (test_case:ExternalService {id:"python.unittest.TestCase",kind:"type"})
# MERGE (pathlib:ExternalService {id:"python.pathlib.Path",kind:"type"})
# MERGE (assert_in:ExternalService {id:"python.unittest.TestCase.assertIn",kind:"method"})
# MERGE (read_text:ExternalService {id:"python.pathlib.Path.read_text",kind:"method"})
# MERGE (historical:File {path:"db/migrations/2026-10-22-schedule-occurrence-branch-snapshot.sql"})
# MERGE (correction:File {path:"db/migrations/2026-10-23-schedule-occurrence-branch-snapshot-correction.sql"})
# MERGE (hostless:File {path:"db/migrations/2026-10-24-schedule-branchless-occurrence.sql"})
# MERGE (harness:File {path:"scripts/automation/phase9f3_schedule.py"})
# MERGE (self)-[:DEFINES]->(suite)
# MERGE (self)-[:IMPORTS]->(unittest)
# MERGE (self)-[:IMPORTS]->(pathlib)
# MERGE (suite)-[:EXTENDS]->(test_case)
# MERGE (suite)-[:HAS_METHOD]->(migration_test)
# MERGE (suite)-[:HAS_METHOD]->(boundary_test)
# MERGE (suite)-[:HAS_METHOD]->(rule_test)
# MERGE (migration_test)-[:TESTS]->(historical)
# MERGE (migration_test)-[:TESTS]->(correction)
# MERGE (migration_test)-[:TESTS]->(hostless)
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
HOSTLESS_MIGRATION = ROOT / "db/migrations/2026-10-24-schedule-branchless-occurrence.sql"
HARNESS = ROOT / "scripts/automation/phase9f3_schedule.py"


class OccurrenceBranchSnapshotMigrationTests(unittest.TestCase):
    def test_historical_migration_is_preserved_and_forward_correction_is_idempotent(self) -> None:
        historical = HISTORICAL_MIGRATION.read_text(encoding="utf-8")
        correction = CORRECTION_MIGRATION.read_text(encoding="utf-8")
        hostless = HOSTLESS_MIGRATION.read_text(encoding="utf-8")

        self.assertIn("occurrence_target_snapshot_branch_uuid_check", historical)
        self.assertIn("IF NOT EXISTS (", historical)
        self.assertIn("{rule_snapshot,target,branch_id}", historical)
        self.assertIn("DROP CONSTRAINT IF EXISTS occurrence_target_snapshot_branch_uuid_check", correction)
        self.assertIn("ADD CONSTRAINT occurrence_target_snapshot_branch_uuid_check", correction)
        self.assertIn("NOT VALID", correction)
        self.assertIn("{branch_id}", correction)
        self.assertNotIn("{rule_snapshot,target,branch_id}", correction)
        self.assertIn("BEGIN;", correction)
        self.assertIn("COMMIT;", correction)
        self.assertNotIn("UPDATE automation.occurrence", correction)
        self.assertNotIn("VALIDATE CONSTRAINT", correction)

        self.assertIn("DROP CONSTRAINT IF EXISTS occurrence_target_snapshot_branch_uuid_check", hostless)
        self.assertIn("ADD CONSTRAINT occurrence_target_snapshot_branch_uuid_check", hostless)
        self.assertIn("NOT (target_snapshot ? 'branch_id')", hostless)
        self.assertIn("jsonb_typeof(target_snapshot -> 'branch_id') = 'null'", hostless)
        self.assertIn("occurrence_target_snapshot_identity_check", hostless)
        self.assertIn("NOT VALID", hostless)
        self.assertIn("BEGIN;", hostless)
        self.assertIn("COMMIT;", hostless)
        self.assertNotIn("UPDATE automation.occurrence", hostless)
        self.assertNotIn("VALIDATE CONSTRAINT", hostless)

    def test_harness_covers_branch_optional_target_and_required_identities(self) -> None:
        source = HARNESS.read_text(encoding="utf-8")
        hostless = HOSTLESS_MIGRATION.read_text(encoding="utf-8")
        paths = (
            "2026-10-22-schedule-occurrence-branch-snapshot.sql",
            "2026-10-23-schedule-occurrence-branch-snapshot-correction.sql",
            "2026-10-24-schedule-branchless-occurrence.sql",
        )
        for path in paths:
            self.assertIn(path, source)
        self.assertLess(source.index(paths[0]), source.index(paths[1]))
        self.assertLess(source.index(paths[1]), source.index(paths[2]))
        self.assertIn('for pass_name in ("first", "repeat")', source)
        self.assertIn('server_options = f"-h 127.0.0.1 -p {self.port}"', source)
        self.assertIn('if os.name != "nt":', source)
        self.assertIn('tempfile.mkdtemp(prefix="star-schedule-pg-")', source)
        self.assertIn('-k "{self.socket_dir}"', source)
        self.assertIn("is_contained(self.socket_dir, temp_root)", source)
        self.assertIn("stop_result = stop", source)
        self.assertIn("return stop_result", source)
        self.assertIn("if cleanup is not None:\n            steps.append(cleanup)", source)
        self.assertIn('"postgres-schedule-branchless-occurrence"', source)

        for identity in (
            "project_id",
            "engineering_run_id",
            "repository_id",
            "worktree_id",
            "work_item_id",
            "execution_profile_id",
            "execution_profile_version",
            "execution_profile_digest",
            "hook_set_version",
            "hook_set_digest",
        ):
            self.assertIn(f"target_snapshot -> '{identity}'", hostless)

        self.assertIn("target_snapshot - 'worktree_id'", source)
        self.assertIn("'{branch_id}', 'null'::jsonb", source)
        self.assertIn("target_snapshot - 'branch_id'", source)
        self.assertIn("jsonb_set(target_snapshot, '{branch_id}', '\"not-a-uuid\"'::jsonb)", source)
        self.assertIn("jsonb_set(target_snapshot, '{worktree_id}', '\"00000000-0000-0000-0000-000000000000\"'::jsonb)", source)
        self.assertIn("jsonb_set(target_snapshot, '{execution_profile_digest}', '\"not-a-digest\"'::jsonb)", source)

        for identity in (
            "project_id", "engineering_run_id", "repository_id",
            "worktree_id", "work_item_id", "execution_profile_id",
        ):
            self.assertIn(f"jsonb_set(target_snapshot, '{{{identity}}}'", source)

    def test_branchless_rule_and_occurrence_remain_storable(self) -> None:
        source = HARNESS.read_text(encoding="utf-8")
        self.assertIn("branch_id', NULL", source)
        self.assertIn("'changed_by', rule.run_as_actor_id", source)
        self.assertIn("'a0000000-0000-4000-8000-000000000001', clock_timestamp() + interval '1 hour'", source)
        self.assertIn("branchless Schedule Rule configuration must remain storable", source)
        self.assertIn("RETURNING rule_id, branch_id INTO hostless_rule_id, hostless_branch_id", source)
        self.assertIn("branchless Schedule Rule revision was not persisted", source)
        self.assertIn("jsonb_set(target_snapshot, '{branch_id}', 'null'::jsonb)", source)
        self.assertIn("target_snapshot - 'branch_id'", source)


if __name__ == "__main__":
    unittest.main()
