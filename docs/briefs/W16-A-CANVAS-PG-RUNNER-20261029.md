# Brief: W16-A-CANVAS-PG-RUNNER-20261029

**Agent**: worker
**Phase**: PHASE-W16-A-CANVAS-PG-RUNNER-20261029
**Created**: 2026-10-29 14:53:36

---

## Objective
Add a repeatable, fail-closed local disposable PostgreSQL runner for the single Canvas SSE revocation acceptance test already compiled in this isolated lane. Use an independently owned loopback-only PostgreSQL 18 cluster and execute exactly the ignored case active_canvas_sse_closes_after_device_binding_revocation.

## Current verified state
- Current candidate HEAD is `5f437c06933f9f84ac26de6f13f1b7cf916cfb23`. The source SHA-256 is `66F46A17FAA287E2D350CA6B24D1B00076EADDC0FCF8F11FA8D0E7C311614F37`; the prebuilt test binary SHA-256 is `51F49610DFB585F21612373630C89040187783ABA5B343DAC2BEBF78BD1309BA`. The coordinator reran `rtk proxy cargo +1.99.0 test -p star-api-rest --test canvas_device_auth_pg --no-run --locked --offline -j 1` with Rust `1.99.0 (b940084d7 2026-09-28)` and observed exit 0 in 18.80 seconds; the resulting binary SHA-256 did not change. The runner recalculates current HEAD and source/binary hashes and compares them with an external unsigned coordinator/operator build attestation. The stated build command, toolchain, exit code and duration remain operator assertions; they are not cryptographically signed or independently verified CI proof. The runner never builds the test itself.
- OAuth runner database 1b4673269a04435daa1fab5c93333133 contains only OAuth/W16 migration setup and is insufficient for the Project/Branch/Engineering Run/Worktree/Canvas fixtures. Do not reuse it as if fully migrated.
- A separate active W16 device-context worktree contains untracked .cache; preserve it and do not read, modify, start, or clean its database data.
- The W16-A and full E2E gates remain open until actual Canvas PostgreSQL behavior passes.
- The coordinator's first runner attempt was blocked during Windows log redaction: `redacted_path.replace(path)` raised WinError 5 while processing `postgres-start.log`. Its summary records that PostgreSQL 18 detection and `initdb` passed, `postgres-stop` succeeded, and the Canvas test never started. The run summary, logs, and data remain at `C:\Users\leo19\AppData\Local\Temp\star-w16a-canvas-sse-pg-acceptance\runs\1b15bedeff5e4eb3a6b1790b760fdf8f`; preserve this evidence without deleting or reusing it.

## Requirements
- The runner must create a unique private data directory and logs under `%TEMP%/star-w16a-canvas-sse-pg-acceptance/runs/<run-id>`, outside the Git worktree. Reserve a random IPv4-loopback port, verify worktree/path disjointness and run-directory containment, initialize/start/stop only that cluster, and preserve the data directory if shutdown cannot be confirmed. Keep the summary and logs after the run; print the absolute evidence path.
- Strip inherited database-target environment variables and set only STAR_W16_DEVICE_AUTH_TEST_DATABASE_URL to the URL generated for the owned loopback cluster. Never accept an arbitrary database URL, remote host, port, Multipass, Docker, or target environment.
- Apply only a reviewed migration sequence sufficient for the fixture tables and both W16 device-auth and Canvas Outbox wakeup triggers: OAuth base, the stable `directory_migration_plan(run_binding=True)` sequence, Worktree Group 2D, Canvas Phase 3, the test-only legacy refresh-pair prerequisite, the directory owner/grant acceptance, W16 device-auth, and Canvas Outbox wakeup. Phase 2D must follow the complete directory plan and precede Canvas Phase 3; Canvas Phase 3 precedes W16 and Canvas wake migrations. Omit `2026-10-05-worktree-group-canvas-stream-offset.sql` because this case does not read replay offsets. Do not edit the active phase9f2 runner or another lane. Use clearly labeled test-only placeholders for audit/SCD2 dependencies. If the complete required schema cannot be installed safely, fail closed and report the exact missing prerequisite.
- Migration validation requires the exact OAuth base step and migration file at position zero, followed by the Directory helper sequence and reviewed Canvas/W16 order.
- Run only the exact already-compiled `canvas_device_auth_pg` binary case with `--ignored --exact`; do not invoke Cargo. Require one passed test, zero failures, and zero ignored. Bound each step and stored log tail. The runner does not deliberately print the generated database URL; log redaction covers literal `postgres://` and `postgresql://` tokens through the next whitespace, quote, or angle bracket. Other credential formats are outside that redactor's guarantee.
- Capture combined child stdout/stderr through a pipe. A reader handles fixed 64 KiB chunks and feeds a four-chunk bounded queue; the runner applies the stateful URL redactor before each retained-file write and flush. Queue saturation blocks the reader and supplies bounded backpressure. No unredacted temporary log or `communicate()` buffer is used. If the process or runner is interrupted, any output already written to the retained file has passed through the redactor; partial output may be omitted. After the configured timeout, the runner kills only the direct child process; process wait, output draining, and reader cleanup share a five-second grace period. A descendant that inherits and keeps the pipe open can outlive the runner's bounded join attempt, so this is not a guarantee that the reader thread or subprocess descendants have exited. Incomplete capture fails the step. The redaction guarantee is limited to PostgreSQL URL tokens matching the stated pattern.
- Resolve the compiled test artifact from `--target-dir` (default: this checkout's `target/`) or an explicit `--test-binary`. An explicit artifact must be a non-symlink `canvas_device_auth_pg-<16-hex-hash>.exe` directly under the standard `<target-dir>/debug/deps` layout; when both options are provided, their target roots must match. The reviewed precompiled artifact is `E:\DevCache\cargo\target\debug\deps\canvas_device_auth_pg-dd3246f8f37904d5.exe`; pass it with `--test-binary` so the runner does not depend on the checkout's Cargo target directory.
- `--build-attestation` is required and names an external unsigned coordinator/operator JSON attestation containing exactly `head`, `canvas_test_source_sha256`, `test_binary_sha256`, `rustc_version`, `build_command`, `build_exit_code`, and `build_elapsed_seconds`. The runner recomputes current HEAD and both SHA-256 values and checks them against the attestation; it does not authenticate the build event. The command, toolchain, exit code and elapsed time are operator assertions, not signed provenance or CI proof. The runner records this limitation in the summary and refuses HEAD/hash/command/toolchain/exit mismatches. Example: `py -3 scripts/automation/w16a_canvas_sse_pg_acceptance.py --test-binary E:\DevCache\cargo\target\debug\deps\canvas_device_auth_pg-dd3246f8f37904d5.exe --build-attestation <external-attestation.json>`.
- The external attestation is read through a 64 KiB + 1 byte bounded read and rejected before JSON parsing when it exceeds 64 KiB.
- The Rust test pool connects as `postgres` in this disposable cluster. The case proves persisted HTTP authorization and the active SSE revoke/close path through the production route/hub; it does not prove runtime `NOBYPASSRLS` behavior or production grants. Do not claim replay-offset coverage.
- The runner installs test-only audit/SCD2 prerequisites. Its generic audit trigger and SCD2-close function are no-op placeholders; the simplified audit table/function does not represent production audit semantics. The summary must record this boundary. These prerequisites do not prove production audit/SCD2 behavior, RLS, or grants.
- Keep evidence at the actual absolute `%TEMP%/star-w16a-canvas-sse-pg-acceptance/runs/<run-id>` path emitted by the runner. Remove only this run's validated `postgres-data` directory after a proven test pass and confirmed shutdown; preserve data on a failed acceptance or unconfirmed shutdown. Always retain summary and logs.
- Before `initdb` and each `pg_ctl` start/stop, pass only the canonical data directory returned by the strict path validator. Immediately before removing `postgres-data`, recheck that the evidence root remains disjoint from the Git worktree, the run is the exact direct child of `runs`, the canonical data path is exactly `<run>/postgres-data`, and the run tree has no symlink, junction, or reparse point. Path-based validation still has a same-user TOCTOU window between checking a path and using it; this local tool does not provide handle-relative/descriptor-anchored protection.
- After the live streaming-redaction hardening, rerun with the preserved compiled binary and a coordinator-supplied external attestation file. The blocked run above is not an acceptance. Command template (replace the quoted attestation path with the supplied file): `py -3 scripts/automation/w16a_canvas_sse_pg_acceptance.py --postgres-bin D:\PostgreSQL\18\bin --test-binary E:\DevCache\cargo\target\debug\deps\canvas_device_auth_pg-dd3246f8f37904d5.exe --build-attestation "<coordinator-supplied-external-attestation-json>"`.
- Add unit tests for environment sanitization, path ownership, command/test-selection, summary proof, and failure cleanup behavior. Do not mock PostgreSQL success.
- Do not run Cargo, PostgreSQL, or alter any other worktree from this worker. The coordinator will review the runner and perform a controlled DB run after implementation.

## Review warnings and safeguards

- **Unsigned build attestation:** the runner recomputes current HEAD and source/binary SHA-256 values and compares them to the external operator attestation. The claimed Cargo command, toolchain, exit code, and duration remain an operator assertion, not signed provenance or CI proof.
- **Path-based cleanup race:** canonical path and reparse-point checks are repeated before deletion and canonical paths are passed to PostgreSQL tools. A same-user TOCTOU window remains between path validation and use; the runner has no handle-relative or descriptor-anchored protection.
- **Live redaction scope:** child output is persisted only after bounded streaming redaction of literal `postgres://` and `postgresql://` tokens. The bounded queue provides backpressure; other credential formats are outside the guarantee.

## Owned files
The amended Canvas manifest remains authoritative. This hardening follow-up may edit only:
1. scripts/automation/w16a_canvas_sse_pg_acceptance.py
2. scripts/automation/test_w16a_canvas_sse_pg_acceptance.py
3. docs/briefs/W16-A-CANVAS-PG-RUNNER-20261029.md
4. docs/briefs/W16-A-CANVAS-PG-HARDEN-20261030.md

The follow-up claim also includes its own frozen brief. Do not alter the already-present WBS dispatch JSONL during this hardening work; its current diff is coordinator-owned and contains no required code evidence for these changes.

Do not modify the acceptance test source, shared SRS/BD/DD/WBS/test design/registry, existing phase9f2 runner, dispatcher JSONL, or any other unregistered file.

## Validation
Run only Python unit tests and py_compile for the two runner scripts, plus whitespace checks for tracked and untracked owned files and CypherGraph Guardian. Report exact commands/results and limitations. Do not run Cargo or PostgreSQL and do not commit; coordinator will review the runner, run the disposable PostgreSQL acceptance, record the emitted evidence path and proof boundary, update shared docs, and commit the final accepted lane.


## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
