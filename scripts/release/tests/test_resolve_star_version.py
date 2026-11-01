"""Focused integration tests for the strict Star release version resolver."""

# @cypher schema=1 source_sha256=dc75062df5241ec26cbf610454c33b8be5ac3797d320e64178f6d1a9a3ee17a4
# MERGE (self:File {path:"scripts/release/tests/test_resolve_star_version.py"})
# MERGE (suite:Symbol {id:"scripts/release/tests/test_resolve_star_version.py::ResolverTests",kind:"class"})
# MERGE (setup:Symbol {id:"scripts/release/tests/test_resolve_star_version.py::ResolverTests.setUp",kind:"method"})
# MERGE (git:Symbol {id:"scripts/release/tests/test_resolve_star_version.py::ResolverTests.git",kind:"method"})
# MERGE (git_at:Symbol {id:"scripts/release/tests/test_resolve_star_version.py::ResolverTests.git_at",kind:"method"})
# MERGE (run:Symbol {id:"scripts/release/tests/test_resolve_star_version.py::ResolverTests.run_resolver",kind:"method"})
# MERGE (commit:Symbol {id:"scripts/release/tests/test_resolve_star_version.py::ResolverTests.commit_file",kind:"method"})
# MERGE (assert_dirty:Symbol {id:"scripts/release/tests/test_resolve_star_version.py::ResolverTests.assert_dirty_checkout_rejected_without_output_change",kind:"method"})
# MERGE (test_valid:Symbol {id:"scripts/release/tests/test_resolve_star_version.py::ResolverTests.test_resolves_lightweight_and_annotated_tags",kind:"method"})
# MERGE (test_invalid:Symbol {id:"scripts/release/tests/test_resolve_star_version.py::ResolverTests.test_rejects_noncanonical_tag_values",kind:"method"})
# MERGE (test_missing:Symbol {id:"scripts/release/tests/test_resolve_star_version.py::ResolverTests.test_missing_tag_ref_fails",kind:"method"})
# MERGE (test_noncommit:Symbol {id:"scripts/release/tests/test_resolve_star_version.py::ResolverTests.test_tag_peeling_to_noncommit_fails",kind:"method"})
# MERGE (test_head:Symbol {id:"scripts/release/tests/test_resolve_star_version.py::ResolverTests.test_head_mismatch_fails",kind:"method"})
# MERGE (test_output:Symbol {id:"scripts/release/tests/test_resolve_star_version.py::ResolverTests.test_output_is_written_only_after_validation",kind:"method"})
# MERGE (test_unstaged:Symbol {id:"scripts/release/tests/test_resolve_star_version.py::ResolverTests.test_unstaged_tracked_changes_rejected",kind:"method"})
# MERGE (test_staged:Symbol {id:"scripts/release/tests/test_resolve_star_version.py::ResolverTests.test_staged_tracked_changes_rejected",kind:"method"})
# MERGE (test_untracked:Symbol {id:"scripts/release/tests/test_resolve_star_version.py::ResolverTests.test_untracked_files_rejected",kind:"method"})
# MERGE (test_submodule:Symbol {id:"scripts/release/tests/test_resolve_star_version.py::ResolverTests.test_dirty_submodule_rejected",kind:"method"})
# MERGE (resolver:File {path:"scripts/release/resolve_star_version.py"})
# MERGE (self)-[:DEFINES]->(suite)
# MERGE (suite)-[:HAS_METHOD]->(setup)
# MERGE (suite)-[:HAS_METHOD]->(git)
# MERGE (suite)-[:HAS_METHOD]->(git_at)
# MERGE (suite)-[:HAS_METHOD]->(run)
# MERGE (suite)-[:HAS_METHOD]->(commit)
# MERGE (suite)-[:HAS_METHOD]->(assert_dirty)
# MERGE (suite)-[:HAS_METHOD]->(test_valid)
# MERGE (suite)-[:HAS_METHOD]->(test_invalid)
# MERGE (suite)-[:HAS_METHOD]->(test_missing)
# MERGE (suite)-[:HAS_METHOD]->(test_noncommit)
# MERGE (suite)-[:HAS_METHOD]->(test_head)
# MERGE (suite)-[:HAS_METHOD]->(test_output)
# MERGE (suite)-[:HAS_METHOD]->(test_unstaged)
# MERGE (suite)-[:HAS_METHOD]->(test_staged)
# MERGE (suite)-[:HAS_METHOD]->(test_untracked)
# MERGE (suite)-[:HAS_METHOD]->(test_submodule)
# MERGE (setup)-[:CALLS]->(git)
# MERGE (git)-[:CALLS]->(git_at)
# MERGE (run)-[:CALLS]->(git)
# MERGE (commit)-[:CALLS]->(git)
# MERGE (assert_dirty)-[:CALLS]->(run)
# MERGE (test_valid)-[:CALLS]->(run)
# MERGE (test_valid)-[:CALLS]->(git)
# MERGE (test_invalid)-[:CALLS]->(run)
# MERGE (test_missing)-[:CALLS]->(run)
# MERGE (test_noncommit)-[:CALLS]->(git)
# MERGE (test_noncommit)-[:CALLS]->(run)
# MERGE (test_head)-[:CALLS]->(commit)
# MERGE (test_head)-[:CALLS]->(run)
# MERGE (test_output)-[:CALLS]->(run)
# MERGE (test_output)-[:CALLS]->(commit)
# MERGE (test_unstaged)-[:CALLS]->(git)
# MERGE (test_unstaged)-[:CALLS]->(assert_dirty)
# MERGE (test_staged)-[:CALLS]->(git)
# MERGE (test_staged)-[:CALLS]->(assert_dirty)
# MERGE (test_untracked)-[:CALLS]->(assert_dirty)
# MERGE (test_submodule)-[:CALLS]->(git_at)
# MERGE (test_submodule)-[:CALLS]->(git)
# MERGE (test_submodule)-[:CALLS]->(assert_dirty)
# MERGE (test_valid)-[:TESTS]->(resolver)
# MERGE (test_invalid)-[:TESTS]->(resolver)
# MERGE (test_missing)-[:TESTS]->(resolver)
# MERGE (test_noncommit)-[:TESTS]->(resolver)
# MERGE (test_head)-[:TESTS]->(resolver)
# MERGE (test_output)-[:TESTS]->(resolver)
# MERGE (test_unstaged)-[:TESTS]->(resolver)
# MERGE (test_staged)-[:TESTS]->(resolver)
# MERGE (test_untracked)-[:TESTS]->(resolver)
# MERGE (test_submodule)-[:TESTS]->(resolver)
# @endcypher

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


REPOSITORY_ROOT = Path(__file__).resolve().parents[3]
RESOLVER = REPOSITORY_ROOT / "scripts" / "release" / "resolve_star_version.py"


class ResolverTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary_directory = tempfile.TemporaryDirectory()
        self.addCleanup(temporary_directory.cleanup)
        self.temporary_root = Path(temporary_directory.name)
        self.repository = self.temporary_root / "repo"
        self.repository.mkdir()
        self.git("init", "--quiet")
        self.git("config", "user.name", "Release Resolver Test")
        self.git("config", "user.email", "release-resolver-test@example.invalid")
        self.commit_file("initial.txt", "initial\n", "initial commit")

    def git(self, *arguments: str, input_text: str | None = None) -> str:
        return self.git_at(self.repository, *arguments, input_text=input_text)

    def git_at(
        self, repository: Path, *arguments: str, input_text: str | None = None
    ) -> str:
        result = subprocess.run(
            ["git", "-C", str(repository), *arguments],
            check=True,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            input=input_text,
            shell=False,
        )
        return result.stdout.strip()

    def run_resolver(self, tag: str, *arguments: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [
                sys.executable,
                str(RESOLVER),
                "--tag",
                tag,
                "--repo",
                str(self.repository),
                *arguments,
            ],
            check=False,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            shell=False,
        )

    def commit_file(self, name: str, contents: str, message: str) -> str:
        (self.repository / name).write_text(contents, encoding="utf-8")
        self.git("add", name)
        self.git("commit", "--quiet", "-m", message)
        return self.git("rev-parse", "HEAD")

    def assert_dirty_checkout_rejected_without_output_change(self) -> None:
        output_path = self.temporary_root / "github-output.txt"
        existing_output = "existing=value\n"
        output_path.write_text(existing_output, encoding="utf-8")

        result = self.run_resolver("v1.2.3", "--output", str(output_path))
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("working tree must be clean", result.stderr)
        self.assertEqual(output_path.read_text(encoding="utf-8"), existing_output)

    def test_resolves_lightweight_and_annotated_tags(self) -> None:
        commit_sha = self.git("rev-parse", "HEAD")
        self.git("tag", "v1.2.3")
        lightweight = self.run_resolver("v1.2.3")
        self.assertEqual(lightweight.returncode, 0, lightweight.stderr)
        self.assertEqual(
            json.loads(lightweight.stdout),
            {"tag": "v1.2.3", "version": "1.2.3", "source_sha": commit_sha},
        )

        self.git("tag", "-a", "v2.3.4", "-m", "release 2.3.4")
        annotated = self.run_resolver("v2.3.4")
        self.assertEqual(annotated.returncode, 0, annotated.stderr)
        self.assertEqual(
            json.loads(annotated.stdout),
            {"tag": "v2.3.4", "version": "2.3.4", "source_sha": commit_sha},
        )

    def test_rejects_noncanonical_tag_values(self) -> None:
        malformed_tags = (
            "1.2.3",
            "v1.2",
            "v1.2.3.4",
            "v1.2.3-alpha.1",
            "v1.2.3+build.1",
            "v01.2.3",
            "v1.02.3",
            "v1.2.03",
            "v1.2.3 ",
            " v1.2.3",
        )
        for tag in malformed_tags:
            with self.subTest(tag=tag):
                result = self.run_resolver(tag)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("expected canonical vMAJOR.MINOR.PATCH", result.stderr)
                self.assertEqual(result.stdout, "")

    def test_missing_tag_ref_fails(self) -> None:
        result = self.run_resolver("v9.8.7")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("does not exist as an exact tag ref", result.stderr)

    def test_tag_peeling_to_noncommit_fails(self) -> None:
        blob_sha = self.git("hash-object", "-w", "--stdin", input_text="blob target\n")
        self.git("tag", "-a", "v1.2.3", "-m", "tag to blob", blob_sha)

        result = self.run_resolver("v1.2.3")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("peels to a blob, not a commit", result.stderr)

    def test_head_mismatch_fails(self) -> None:
        tagged_sha = self.git("rev-parse", "HEAD")
        self.git("tag", "v1.2.3")
        head_sha = self.commit_file("later.txt", "later\n", "later commit")

        result = self.run_resolver("v1.2.3")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(f"tag 'v1.2.3' resolves to {tagged_sha}", result.stderr)
        self.assertIn(f"checked-out HEAD is {head_sha}", result.stderr)

    def test_output_is_written_only_after_validation(self) -> None:
        output_path = self.temporary_root / "github-output.txt"
        output_path.write_text("existing=value\n", encoding="utf-8")
        commit_sha = self.git("rev-parse", "HEAD")
        self.git("tag", "v1.2.3")

        valid = self.run_resolver("v1.2.3", "--output", str(output_path))
        self.assertEqual(valid.returncode, 0, valid.stderr)
        expected_output = (
            "existing=value\n"
            "STAR_RELEASE_TAG=v1.2.3\n"
            "STAR_RELEASE_VERSION=1.2.3\n"
            f"STAR_RELEASE_SOURCE_SHA={commit_sha}\n"
        )
        self.assertEqual(output_path.read_text(encoding="utf-8"), expected_output)

        self.commit_file("later.txt", "later\n", "later commit")
        rejected = self.run_resolver("v1.2.3", "--output", str(output_path))
        self.assertNotEqual(rejected.returncode, 0)
        self.assertIn("checked-out HEAD is", rejected.stderr)
        self.assertEqual(output_path.read_text(encoding="utf-8"), expected_output)

    def test_unstaged_tracked_changes_rejected(self) -> None:
        self.git("tag", "v1.2.3")
        (self.repository / "initial.txt").write_text("modified\n", encoding="utf-8")

        self.assert_dirty_checkout_rejected_without_output_change()

    def test_staged_tracked_changes_rejected(self) -> None:
        self.git("tag", "v1.2.3")
        (self.repository / "initial.txt").write_text("staged modification\n", encoding="utf-8")
        self.git("add", "initial.txt")

        self.assert_dirty_checkout_rejected_without_output_change()

    def test_untracked_files_rejected(self) -> None:
        self.git("tag", "v1.2.3")
        (self.repository / "untracked.txt").write_text("untracked\n", encoding="utf-8")

        self.assert_dirty_checkout_rejected_without_output_change()

    def test_dirty_submodule_rejected(self) -> None:
        source_repository = self.temporary_root / "submodule-source"
        source_repository.mkdir()
        self.git_at(source_repository, "init", "--quiet")
        self.git_at(source_repository, "config", "user.name", "Submodule Resolver Test")
        self.git_at(
            source_repository,
            "config",
            "user.email",
            "submodule-resolver-test@example.invalid",
        )
        (source_repository / "tracked.txt").write_text("initial\n", encoding="utf-8")
        self.git_at(source_repository, "add", "tracked.txt")
        self.git_at(source_repository, "commit", "--quiet", "-m", "submodule commit")

        self.git(
            "-c",
            "protocol.file.allow=always",
            "submodule",
            "add",
            "--quiet",
            str(source_repository),
            "nested",
        )
        self.git("commit", "--quiet", "-m", "add submodule")
        self.git("tag", "v1.2.3")
        (self.repository / "nested" / "tracked.txt").write_text("dirty\n", encoding="utf-8")

        self.assert_dirty_checkout_rejected_without_output_change()


if __name__ == "__main__":
    unittest.main()
