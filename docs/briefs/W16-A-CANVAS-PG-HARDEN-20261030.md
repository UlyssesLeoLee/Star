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
- **Live output safeguard:** subprocess output is read in 64 KiB chunks by one reader thread into a four-chunk bounded queue. The consumer redacts literal PostgreSQL URL tokens before writing and flushing the retained log. Queue saturation applies backpressure; there is no raw temporary log and no unbounded `communicate()` buffer. On interruption, only already-redacted bytes can have reached the file; some output may be omitted. After timeout, the runner kills only the direct child; process wait, drain, and reader cleanup share a five-second grace. A descendant that inherits and holds the pipe may outlive the runner's bounded reader-thread join attempt. The runner does not guarantee termination of descendants or reader cleanup in that case. The guarantee covers `postgres://` and `postgresql://` URL tokens with the configured delimiters, not arbitrary credential formats.
- **Attestation and migration safeguards:** the runner reads no more than 64 KiB + 1 byte and rejects oversize input before JSON parsing; it also rejects a plan unless the exact OAuth base migration is first.
- **Acceptance boundary:** no Cargo or PostgreSQL command was run by this hardening worker. Actual Canvas SSE database behavior remains unverified until the coordinator executes the controlled runner.

## Validation record

- `py -3 -m unittest scripts.automation.test_w16a_canvas_sse_pg_acceptance -v` — PASS, 23 tests (1.075 s). The live-output regression confirms a second sanitized chunk is in the retained file while the mocked producer remains unfinished.
- `py -3 -m py_compile scripts/automation/w16a_canvas_sse_pg_acceptance.py scripts/automation/test_w16a_canvas_sse_pg_acceptance.py` — PASS, exit 0.
- `git diff --check` — PASS, exit 0 with no diagnostics.
- Bounded whitespace/final-newline scan — PASS for both scripts, both runner briefs, and the existing dispatcher JSONL; no trailing spaces/tabs or missing final newlines.
- `py -3 C:/Users/leo19/.codex/skills/CypherGraph-Guardian/skill/cypher-graph-guardian/scripts/cgg_guard.py --changed --stamp` — PASS, 2 source files checked. The canonical `.agents` copy is unavailable in this worktree, so the installed skill copy was used.
- Cargo and PostgreSQL — NOT RUN, as required. Canvas SSE database acceptance remains unverified. Preserved failed-run evidence was not accessed. No commit was created.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
