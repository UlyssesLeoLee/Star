# W7 K3s Context Junction Regression Test Design

## Purpose

Prove that the filtered K3s build-context generator rejects Windows directory junctions inside selected Star Cargo packages before any files from the junction target enter the generated context.

## Acceptance criteria

| ID | Stimulus | Expected result |
|---|---|---|
| AC-W7-CONTEXT-REPARSE-01 | A selected package contains a Windows directory junction to a directory outside the fixture repository, with a unique sentinel file in that target. | Context generation raises `ContextError`; the sentinel is absent from the generated output. The fixture removes only the junction itself. |
| AC-W7-CONTEXT-REPARSE-02 | On Windows, the path metadata returned by `lstat` lacks `st_file_attributes`. | The detector raises `ContextError` and does not treat the path as safe. |
| AC-W7-CONTEXT-REPARSE-03 | Existing ordinary symlink/path-escape and selected-closure tests run with the new detector. | Existing rejection and allowed-closure behavior remains intact. |

## Test construction

- The fixture creates a real junction with PowerShell `New-Item -ItemType Junction`; no Docker, VM, or cluster is used.
- The target is a sibling temporary directory outside the fixture repository. It contains only `must-not-copy.txt` as the sentinel.
- The test invokes `_copy_fixture_context`, which exercises the production package-copy path, and checks both the failure and the absent sentinel.
- Cleanup calls `Path.rmdir()` on the junction path in `finally`; it does not enumerate or delete the target directory.
- The missing-metadata test substitutes an `lstat` result with no Windows attribute field and verifies fail-closed behavior.
- The real-junction test is skipped only on non-Windows. A junction creation failure on Windows fails the test.

## Compatibility basis

The implementation uses `Path.lstat()` and Windows `stat_result.st_file_attributes` rather than depending on `Path.is_junction()`. Python 3.11 documents the Windows file-attribute and reparse-tag fields, while `Path.is_junction()` was added in Python 3.12.

- [Python 3.11 `os.stat_result` documentation](https://docs.python.org/3.11/library/os.html#os.stat_result)
- [Python 3.12 `Path.is_junction()` documentation](https://docs.python.org/3.12/library/pathlib.html#pathlib.Path.is_junction)

## Scope boundary

These tests prove the local path-filter behavior only. They do not prove the Docker archive, image layers, digest, SBOM, provenance, signature, consumer pull, Multipass build, K3s rollout, or rollback.
