# Brief: W16-A-CANVAS-PG-HARDEN-20261030

**Agent**: worker
**Phase**: W16-A Canvas PG runner security hardening
**Created**: 2026-10-30 06:37:50

---

## Task
Address the independent runner review findings before any PostgreSQL rerun. At the start of this hardening pass the branch HEAD was `5f437c06933f9f84ac26de6f13f1b7cf916cfb23`; the accepted runner and test source hashes after hardening are recorded in the review brief. The reviewer's additional warning about descendants retaining the output pipe is recorded below; cleanup is a bounded attempt, not a process-tree termination guarantee.

## Required changes
1. Ensure subprocess output is redacted in a bounded streaming path before it is persisted. A hard interruption must not leave a raw database URL in the retained log. Do not use unbounded communicate() or raw temporary log files. Preserve timeout/kill behavior and bounded backpressure. Add a unit regression that inspects the retained file while a mocked producer is still emitting output.
2. Reject an oversized external build-attestation file before parsing it; cap at 64 KiB (or justify an equally strict bound) and add a regression test.
3. Make migration-plan validation and tests explicitly require the OAuth base migration at position zero, in addition to the currently validated order.
4. Update the runner briefs to describe the actual live redaction guarantee and residual limits precisely; remove absolute claims that are not supported. Record the two review warnings and the new safeguards.

## Owned files
Only `scripts/automation/w16a_canvas_sse_pg_acceptance.py`, `scripts/automation/test_w16a_canvas_sse_pg_acceptance.py`, `docs/briefs/W16-A-CANVAS-PG-RUNNER-20261029.md`, this follow-up brief, and the already-claimed WBS dispatch JSONL. The MAGOS candidate claim has been amended to 11 paths.

## Validation and boundaries
Run py_compile, the focused Python unit suite, whitespace checks for tracked and untracked owned files, and Cypher Guardian. Report exact outcomes and final hashes. Do not run Cargo or PostgreSQL; coordinator will do the PG run after independent review. Do not edit source test, database migrations, SRS/BD/DD, WBS, phase report, automation design, or registry. Preserve the earlier failed run evidence and do not commit or clean any run/worktree. You are not alone in the repository: accommodate other lane work and do not revert any unrelated change.

## Review warning register and implementation evidence

- **Warning 1 — build attestation is not authenticated:** current HEAD and source/binary SHA-256 values are recomputed and compared with the external JSON. The claimed Cargo invocation, Rust version, exit code, and elapsed time remain a coordinator/operator assertion, not signed provenance or independent CI evidence.
- **Warning 2 — path checks have a same-user race:** canonical run/data paths and reparse points are checked again before deletion, and PostgreSQL receives the canonical data path. A same-user TOCTOU gap remains between validation and path use because this local runner does not use handle-relative or descriptor-anchored filesystem operations.
- **Live output safeguard:** captured subprocess output is read in 64 KiB chunks by one reader thread into a four-chunk bounded queue. The consumer redacts literal PostgreSQL URL tokens before writing and flushing the retained step log. Queue saturation applies backpressure; there is no unredacted capture file and no unbounded `communicate()` buffer. PostgreSQL startup uses `capture_output=False`, redirects both child streams to the OS null device so a daemon descendant cannot hold the runner's pipe, and verifies readiness through a separate `psql` step. The server log remains inside the run-owned data directory and follows its preserve/delete rules.
- For captured steps, only already-redacted bytes can reach their retained step log; some output may be omitted. After timeout the runner kills only the direct child; process wait, drain, and reader cleanup share a five-second grace, and a future captured command with a pipe-holding descendant can still outlive this bounded attempt. In the suppressed-output path, timeout or interruption performs a best-effort direct-child kill and bounded wait; the original interruption is re-raised. Redaction covers `postgres://` and `postgresql://` URL tokens with configured delimiters, not arbitrary credential formats.
- **Attestation and migration safeguards:** the runner reads no more than 64 KiB + 1 byte and rejects oversize input before JSON parsing; it also rejects a plan unless the exact OAuth base migration is first.
- **Acceptance boundary at worker handoff:** this hardening worker did not run Cargo or PostgreSQL. The coordinator later ran the controlled acceptance; its evidence and the limits of that result are recorded below.

## Validation record

- `py -3 -m unittest scripts.automation.test_w16a_canvas_sse_pg_acceptance -v` — PASS, 23 tests (1.075 s). The live-output regression confirms a second sanitized chunk is in the retained file while the mocked producer remains unfinished.
- `py -3 -m py_compile scripts/automation/w16a_canvas_sse_pg_acceptance.py scripts/automation/test_w16a_canvas_sse_pg_acceptance.py` — PASS, exit 0.
- `git diff --check` — PASS, exit 0 with no diagnostics.
- Bounded whitespace/final-newline scan — PASS for both scripts, both runner briefs, and the existing dispatcher JSONL; no trailing spaces/tabs or missing final newlines.
- `py -3 C:/Users/leo19/.codex/skills/CypherGraph-Guardian/skill/cypher-graph-guardian/scripts/cgg_guard.py --changed --stamp` — PASS, 2 source files checked. The canonical `.agents` copy is unavailable in this worktree, so the installed skill copy was used.
- Cargo and PostgreSQL — NOT RUN by this worker, as required. At worker handoff, Canvas SSE database acceptance was unverified. Preserved failed-run evidence was not accessed. No commit was created. The later coordinator acceptance is recorded below.

## Coordinator follow-up after live runner execution

- The first correctly attested live attempt, run `fe4ac38e09bb45a1931a1194f97a5892`, confirmed PostgreSQL 18 and initialized the private cluster, but `postgres-start` returned timeout 124 even though the server had started. The detached server retained the captured pipe until the timeout path was unblocked. The coordinator verified PID 44396 against that run's `postmaster.pid`, stopped only that run's exact data directory, and preserved its summary, logs, and data directory. The runner consequently recorded `failed_cleanup` because its later stop step observed the already-stopped cluster. This is not a database acceptance.
- `capture_output=False` now prevents the PostgreSQL startup descendant from inheriting the captured pipe, and it kills/waits the direct child before propagating interruption. The updated focused suite passed 25 tests, including direct-child timeout and `KeyboardInterrupt` cleanup. The later accepted PostgreSQL run is recorded in the next bullet.
- The independent reviewer approved a controlled rerun and found no high-severity blocker. Residual limits: direct-child termination is not Windows process-tree termination; if a descendant survives or kill fails, the outer cleanup attempts `pg_ctl stop` against the exact validated data directory and preserves data when shutdown is unconfirmed. A second Ctrl+C during timeout cleanup can interrupt the bounded wait after the kill request.
- The coordinator's accepted rerun was run `49b512fbfe334f0e98e46fe7eae7ea39` on PostgreSQL 18.6: all 41 recorded steps exited 0; `active_canvas_sse_closes_after_device_binding_revocation` passed 1/1 with 0 failed, 0 ignored, 3 filtered, in 0.17 seconds. PostgreSQL stop exited 0 and confirmed shutdown; the owned data directory was removed, while summary/log evidence was retained. The binary was built at candidate HEAD `d1269a7928c9323f94403f3f1bb7888048c1dbac` under Rust 1.99.0. Its external build attestation is unsigned. This single disposable-superuser case uses no-op test-only audit/SCD2 prerequisites and does not prove production grants/RLS, audit semantics, endpoint audit, multi-instance behavior, p99, or full W16-A closure.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
