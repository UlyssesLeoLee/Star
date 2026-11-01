"""Focused integration tests for the strict Star release version resolver."""

# @cypher schema=1 source_sha256=efc9d503825d1fa79e740158e7378d41da2ed8819ae6c7ea5ecb42c3008987cd
# MERGE (self:File {path:"scripts/release/tests/test_resolve_star_version.py"})
# MERGE (suite:Symbol {id:"scripts/release/tests/test_resolve_star_version.py::ResolverTests",kind:"class"})
# MERGE (setup:Symbol {id:"scripts/release/tests/test_resolve_star_version.py::ResolverTests.setUp",kind:"method"})
# MERGE (git:Symbol {id:"scripts/release/tests/test_resolve_star_version.py::ResolverTests.git",kind:"method"})
# MERGE (run:Symbol {id:"scripts/release/tests/test_resolve_star_version.py::ResolverTests.run_resolver",kind:"method"})
# MERGE (commit:Symbol {id:"scripts/release/tests/test_resolve_star_version.py::ResolverTests.commit_file",kind:"method"})
# MERGE (test_valid:Symbol {id:"scripts/release/tests/test_resolve_star_version.py::ResolverTests.test_resolves_lightweight_and_annotated_tags",kind:"method"})
# MERGE (test_invalid:Symbol {id:"scripts/release/tests/test_resolve_star_version.py::ResolverTests.test_rejects_noncanonical_tag_values",kind:"method"})
# MERGE (test_missing:Symbol {id:"scripts/release/tests/test_resolve_star_version.py::ResolverTests.test_missing_tag_ref_fails",kind:"method"})
# MERGE (test_noncommit:Symbol {id:"scripts/release/tests/test_resolve_star_version.py::ResolverTests.test_tag_peeling_to_noncommit_fails",kind:"method"})
# MERGE (test_head:Symbol {id:"scripts/release/tests/test_resolve_star_version.py::ResolverTests.test_head_mismatch_fails",kind:"method"})
# MERGE (test_output:Symbol {id:"scripts/release/tests/test_resolve_star_version.py::ResolverTests.test_output_is_written_only_after_validation",kind:"method"})
# MERGE (resolver:File {path:"scripts/release/resolve_star_version.py"})
# MERGE (self)-[:DEFINES]->(suite)
# MERGE (suite)-[:HAS_METHOD]->(setup)
# MERGE (suite)-[:HAS_METHOD]->(git)
# MERGE (suite)-[:HAS_METHOD]->(run)
# MERGE (suite)-[:HAS_METHOD]->(commit)
# MERGE (suite)-[:HAS_METHOD]->(test_valid)
# MERGE (suite)-[:HAS_METHOD]->(test_invalid)
# MERGE (suite)-[:HAS_METHOD]->(test_missing)
# MERGE (suite)-[:HAS_METHOD]->(test_noncommit)
# MERGE (suite)-[:HAS_METHOD]->(test_head)
# MERGE (suite)-[:HAS_METHOD]->(test_output)
# MERGE (setup)-[:CALLS]->(git)
# MERGE (run)-[:CALLS]->(git)
# MERGE (commit)-[:CALLS]->(git)
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
# MERGE (test_valid)-[:TESTS]->(resolver)
# MERGE (test_invalid)-[:TESTS]->(resolver)
# MERGE (test_missing)-[:TESTS]->(resolver)
# MERGE (test_noncommit)-[:TESTS]->(resolver)
# MERGE (test_head)-[:TESTS]->(resolver)
# MERGE (test_output)-[:TESTS]->(resolver)
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
        self.repository = Path(temporary_directory.name)
        self.git("init", "--quiet")
        self.git("config", "user.name", "Release Resolver Test")
        self.git("config", "user.email", "release-resolver-test@example.invalid")
        self.commit_file("initial.txt", "initial\n", "initial commit")

    def git(self, *arguments: str, input_text: str | None = None) -> str:
        result = subprocess.run(
            ["git", "-C", str(self.repository), *arguments],
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
        output_path = self.repository / "github-output.txt"
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


if __name__ == "__main__":
    unittest.main()
