"""Focused tests for the fail-closed Star release consumer verifier."""

# @cypher schema=1 source_sha256=04d47497b20c3dd5a8c899344f2cca8b96a6d77315fee7943620f97454008455
# MERGE (self:File {path:"scripts/release/tests/test_verify_attested_release.py"})
# MERGE (suite:Symbol {id:"scripts/release/tests/test_verify_attested_release.py::AttestedReleaseVerifierTests",kind:"class"})
# MERGE (setup:Symbol {id:"scripts/release/tests/test_verify_attested_release.py::AttestedReleaseVerifierTests.setUp",kind:"method"})
# MERGE (run:Symbol {id:"scripts/release/tests/test_verify_attested_release.py::AttestedReleaseVerifierTests.verify",kind:"method"})
# MERGE (file_success:Symbol {id:"scripts/release/tests/test_verify_attested_release.py::AttestedReleaseVerifierTests.test_file_verification_uses_all_release_identity_constraints",kind:"method"})
# MERGE (canvas_success:Symbol {id:"scripts/release/tests/test_verify_attested_release.py::AttestedReleaseVerifierTests.test_canvas_requires_and_verifies_an_immutable_allowed_image",kind:"method"})
# MERGE (web_success:Symbol {id:"scripts/release/tests/test_verify_attested_release.py::AttestedReleaseVerifierTests.test_web_archive_name_is_bound_to_tag_and_source",kind:"method"})
# MERGE (digest_mismatch:Symbol {id:"scripts/release/tests/test_verify_attested_release.py::AttestedReleaseVerifierTests.test_local_digest_mismatch_stops_before_gh",kind:"method"})
# MERGE (invalid_identity:Symbol {id:"scripts/release/tests/test_verify_attested_release.py::AttestedReleaseVerifierTests.test_invalid_release_identity_fails_before_gh",kind:"method"})
# MERGE (gh_failure:Symbol {id:"scripts/release/tests/test_verify_attested_release.py::AttestedReleaseVerifierTests.test_nonzero_gh_result_does_not_expose_stderr",kind:"method"})
# MERGE (timeout:Symbol {id:"scripts/release/tests/test_verify_attested_release.py::AttestedReleaseVerifierTests.test_timeout_fails_closed",kind:"method"})
# MERGE (missing_gh:Symbol {id:"scripts/release/tests/test_verify_attested_release.py::AttestedReleaseVerifierTests.test_missing_gh_fails_closed",kind:"method"})
# MERGE (bad_json:Symbol {id:"scripts/release/tests/test_verify_attested_release.py::AttestedReleaseVerifierTests.test_empty_malformed_and_unrecognized_json_fail_closed",kind:"method"})
# MERGE (oversized:Symbol {id:"scripts/release/tests/test_verify_attested_release.py::AttestedReleaseVerifierTests.test_oversized_json_fails_closed",kind:"method"})
# MERGE (verifier:File {path:"scripts/release/verify_attested_release.py"})
# MERGE (self)-[:DEFINES]->(suite)
# MERGE (suite)-[:HAS_METHOD]->(setup)
# MERGE (suite)-[:HAS_METHOD]->(run)
# MERGE (suite)-[:HAS_METHOD]->(file_success)
# MERGE (suite)-[:HAS_METHOD]->(canvas_success)
# MERGE (suite)-[:HAS_METHOD]->(web_success)
# MERGE (suite)-[:HAS_METHOD]->(digest_mismatch)
# MERGE (suite)-[:HAS_METHOD]->(invalid_identity)
# MERGE (suite)-[:HAS_METHOD]->(gh_failure)
# MERGE (suite)-[:HAS_METHOD]->(timeout)
# MERGE (suite)-[:HAS_METHOD]->(missing_gh)
# MERGE (suite)-[:HAS_METHOD]->(bad_json)
# MERGE (suite)-[:HAS_METHOD]->(oversized)
# MERGE (run)-[:CALLS]->(verifier)
# MERGE (file_success)-[:CALLS]->(run)
# MERGE (canvas_success)-[:CALLS]->(run)
# MERGE (web_success)-[:CALLS]->(run)
# MERGE (digest_mismatch)-[:CALLS]->(run)
# MERGE (invalid_identity)-[:CALLS]->(run)
# MERGE (gh_failure)-[:CALLS]->(run)
# MERGE (timeout)-[:CALLS]->(run)
# MERGE (missing_gh)-[:CALLS]->(run)
# MERGE (bad_json)-[:CALLS]->(run)
# MERGE (oversized)-[:CALLS]->(run)
# MERGE (file_success)-[:TESTS]->(verifier)
# MERGE (canvas_success)-[:TESTS]->(verifier)
# MERGE (web_success)-[:TESTS]->(verifier)
# MERGE (digest_mismatch)-[:TESTS]->(verifier)
# MERGE (invalid_identity)-[:TESTS]->(verifier)
# MERGE (gh_failure)-[:TESTS]->(verifier)
# MERGE (timeout)-[:TESTS]->(verifier)
# MERGE (missing_gh)-[:TESTS]->(verifier)
# MERGE (bad_json)-[:TESTS]->(verifier)
# MERGE (oversized)-[:TESTS]->(verifier)
# @endcypher

from __future__ import annotations

import hashlib
import json
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts.release import verify_attested_release as verifier


SOURCE_COMMIT = "a" * 40
VERIFIED_JSON = json.dumps(
    [{"attestation": {"digest": "sha256:verified"}, "verificationResult": {"verified": True}}]
)


class AttestedReleaseVerifierTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary_directory = tempfile.TemporaryDirectory()
        self.addCleanup(temporary_directory.cleanup)
        self.temporary_root = Path(temporary_directory.name)
        self.artifact = self.temporary_root / "star-desktop-installer.msi"
        self.artifact.write_bytes(b"Star release artifact\n")
        self.digest = f"sha256:{hashlib.sha256(self.artifact.read_bytes()).hexdigest()}"

    def verify(self, **overrides: object) -> dict[str, str | int]:
        arguments: dict[str, object] = {
            "kind": "desktop",
            "repository": "UlyssesLeoLee/Star",
            "tag": "v1.0.6",
            "source_commit": SOURCE_COMMIT,
            "subject": str(self.artifact),
            "expected_digest": self.digest,
            "timeout": 30,
            "gh_executable": "gh-test",
        }
        arguments.update(overrides)
        return verifier.verify_attested_release(**arguments)  # type: ignore[arg-type]

    @patch.object(verifier.subprocess, "run")
    def test_file_verification_uses_all_release_identity_constraints(self, mocked_run) -> None:
        mocked_run.return_value = subprocess.CompletedProcess([], 0, stdout=VERIFIED_JSON)

        result = self.verify()

        self.assertEqual(result["digest"], self.digest)
        self.assertEqual(result["subject"], self.artifact.name)
        self.assertEqual(result["attestation_records"], 1)
        command = mocked_run.call_args.args[0]
        self.assertEqual(command[:4], ["gh-test", "attestation", "verify", str(self.artifact.resolve())])
        self.assertIn("--repo", command)
        self.assertEqual(command[command.index("--source-digest") + 1], SOURCE_COMMIT)
        self.assertEqual(command[command.index("--source-ref") + 1], "refs/tags/v1.0.6")
        self.assertEqual(
            command[command.index("--signer-workflow") + 1],
            "github.com/UlyssesLeoLee/Star/.github/workflows/star-stable-desktop.yml",
        )
        self.assertEqual(
            command[command.index("--predicate-type") + 1], verifier.PREDICATE_TYPE
        )
        self.assertEqual(command[command.index("--format") + 1], "json")
        options = mocked_run.call_args.kwargs
        self.assertFalse(options["shell"])
        self.assertEqual(options["timeout"], 30)
        self.assertIs(options["stderr"], subprocess.DEVNULL)
        self.assertIs(options["stdin"], subprocess.DEVNULL)

    @patch.object(verifier.subprocess, "run")
    def test_canvas_requires_and_verifies_an_immutable_allowed_image(self, mocked_run) -> None:
        mocked_run.return_value = subprocess.CompletedProcess([], 0, stdout=VERIFIED_JSON)
        digest = "b" * 64
        subject = f"oci://ghcr.io/ulyssesleolee/star-canvas-engine@sha256:{digest}"

        result = self.verify(kind="canvas", subject=subject, expected_digest=f"sha256:{digest}")

        self.assertEqual(result["digest"], f"sha256:{digest}")
        command = mocked_run.call_args.args[0]
        self.assertEqual(command[3], subject)
        self.assertEqual(
            command[command.index("--signer-workflow") + 1],
            "github.com/UlyssesLeoLee/Star/.github/workflows/publish-star-canvas-services.yml",
        )

    def test_web_archive_name_is_bound_to_tag_and_source(self) -> None:
        name = f"star-web-build-evidence-v1.0.6-1.0.6-{SOURCE_COMMIT}.tar.gz"
        archive = self.temporary_root / name
        archive.write_bytes(b"web release evidence")
        digest = f"sha256:{hashlib.sha256(archive.read_bytes()).hexdigest()}"
        with patch.object(verifier.subprocess, "run") as mocked_run:
            mocked_run.return_value = subprocess.CompletedProcess([], 0, stdout=VERIFIED_JSON)
            result = self.verify(kind="web", subject=str(archive), expected_digest=digest)
        self.assertEqual(result["subject"], name)

        wrong_archive = self.temporary_root / "star-web-build-evidence-v9.9.9.tar.gz"
        wrong_archive.write_bytes(b"wrong identity")
        wrong_digest = f"sha256:{hashlib.sha256(wrong_archive.read_bytes()).hexdigest()}"
        with patch.object(verifier.subprocess, "run") as mocked_run:
            with self.assertRaisesRegex(verifier.VerificationError, "filename does not match"):
                self.verify(kind="web", subject=str(wrong_archive), expected_digest=wrong_digest)
            mocked_run.assert_not_called()

    @patch.object(verifier.subprocess, "run")
    def test_local_digest_mismatch_stops_before_gh(self, mocked_run) -> None:
        with self.assertRaisesRegex(verifier.VerificationError, "does not match"):
            self.verify(expected_digest="sha256:" + "0" * 64)
        mocked_run.assert_not_called()

    @patch.object(verifier.subprocess, "run")
    def test_invalid_release_identity_fails_before_gh(self, mocked_run) -> None:
        with self.assertRaisesRegex(verifier.VerificationError, "canonical stable"):
            self.verify(tag="v1.0.6-rc.1")
        with self.assertRaisesRegex(verifier.VerificationError, "full lowercase"):
            self.verify(source_commit="A" * 40)
        mocked_run.assert_not_called()

    @patch.object(verifier.subprocess, "run")
    def test_nonzero_gh_result_does_not_expose_stderr(self, mocked_run) -> None:
        mocked_run.return_value = subprocess.CompletedProcess(
            [], 1, stdout="", stderr="token=do-not-print"
        )
        with self.assertRaises(verifier.VerificationError) as captured:
            self.verify()
        self.assertNotIn("do-not-print", str(captured.exception))

    @patch.object(verifier.subprocess, "run", side_effect=subprocess.TimeoutExpired("gh", 30))
    def test_timeout_fails_closed(self, mocked_run) -> None:
        with self.assertRaisesRegex(verifier.VerificationError, "exceeded its timeout"):
            self.verify()
        self.assertFalse(mocked_run.call_args.kwargs["shell"])

    @patch.object(verifier.subprocess, "run", side_effect=FileNotFoundError("gh missing"))
    def test_missing_gh_fails_closed(self, mocked_run) -> None:
        with self.assertRaisesRegex(verifier.VerificationError, "install gh"):
            self.verify()
        mocked_run.assert_called_once()

    @patch.object(verifier.subprocess, "run")
    def test_empty_malformed_and_unrecognized_json_fail_closed(self, mocked_run) -> None:
        for output in ("", "not-json", "[]", "{}", "[{}]", '[{"attestation":{}}]'):
            mocked_run.return_value = subprocess.CompletedProcess([], 0, stdout=output)
            with self.subTest(output=output):
                with self.assertRaises(verifier.VerificationError):
                    self.verify()

    @patch.object(verifier.subprocess, "run")
    def test_oversized_json_fails_closed(self, mocked_run) -> None:
        mocked_run.return_value = subprocess.CompletedProcess(
            [], 0, stdout=" " * (verifier.MAX_RESULT_BYTES + 1)
        )
        with self.assertRaisesRegex(verifier.VerificationError, "size limit"):
            self.verify()


if __name__ == "__main__":
    unittest.main()
