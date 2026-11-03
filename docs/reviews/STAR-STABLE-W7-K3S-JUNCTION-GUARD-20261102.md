# Review: STAR-STABLE-W7-K3S-JUNCTION-GUARD-20261102

## Scope

Independently reviewed implementation commits `ca61ab93` and `83bf3f96`, the test design, and the focused Python test output against the registered W7 junction-guard brief. Scope is the filtered K3s context generator and its regression tests.

## Findings

| Severity | Finding | Disposition |
|---|---|---|
| P1, fixed | The prior copier treated a Windows junction as an ordinary directory because `Path.is_symlink()` returned false, allowing an external payload into the context. | Accepted fix: the implementation uses `lstat` reparse attributes, rejects before traversal/copy, and checks repository input path components. |
| Informational | The new Windows test creates a real junction and verifies both rejection and absence of the external sentinel. | Accepted; test passed without skip on Windows. |
| Informational | Missing Windows `st_file_attributes` is explicitly rejected. | Accepted; dedicated fail-closed test passed. |
| Medium | The older symlink fixture skipped with WinError 1314 because the host does not grant symlink creation privilege. | Retained as a skip; the new junction case is independent and passed. |
| Medium | No Docker context archive/image, digest, SBOM, provenance, Multipass/K3s deployment, consumer, hosted, or rollback evidence was produced. | W7 and stable release remain OPEN / NO-GO. |

## Review decision

**Accept the local K3s context path-security fix and regression coverage.** The patch is limited to the registered source and test files, keeps the Star Cargo closure filtering, and addresses both package children and repository input ancestors. This review does not close W7 or certify Star-only final image contents.

## Verification reviewed

- Python 3.13.7.
- Focused suite: 20 tests run, 19 passed, 1 existing symlink-privilege skip.
- `py_compile`: passed.
- CypherGraph Guardian: passed after header reconciliation.
- `git diff --check`: passed.

## Review history

| Version | Date | Reviewer | Result |
|---|---|---|---|
| v0.1 | 2026-11-02 | 架构师（Mavis 接手 agent per DEC-008）— independent review | Accept local fix; W7 / release remain OPEN |
