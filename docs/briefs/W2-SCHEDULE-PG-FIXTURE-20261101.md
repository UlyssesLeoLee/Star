# Brief: W2-SCHEDULE-PG-FIXTURE-20261101

**Agent**: Codex subagent: schedule-fixture
**Phase**: W2-SCHEDULE-PG-FIXTURE-20261101
**Created**: 2026-10-31 21:33:40

---

## Assignment
Fix and report the W2 Schedule PostgreSQL test-fixture compile issue on branch agent/star-stable-followup/schedule-fixture, based on dev@bde4e9d8e9dac0e25f7d983814d37fb27bc9313e.

## Owned scope
Only these registered paths may be edited:
crates/run-owner-runtime/tests/schedule_admission_pg.rs
docs/briefs/W2-SCHEDULE-PG-FIXTURE-20261101.md
docs/reports/PHASE-W2-SCHEDULE-PG-FIXTURE-20261101.md
docs/reports/WBS-DISPATCH-W2-SCHEDULE-PG-FIXTURE-20261101.jsonl

## Required work
Fix the two compile errors caused by interpolating Option<Uuid> branch_id in PG fixture setup. Require a concrete fixture Branch identity explicitly before constructing SQL. Preserve the remote-Branch fixture contract and runtime behavior; do not weaken assertions or alter production code.
Do not touch Rust toolchain files, Cargo.toml/Cargo.lock, main, remote refs, other branches, or worktrees.

## Verification and report
First attempt the exact required MSRV command: cargo +1.99.0 check -p run-owner-runtime --all-targets --locked -j 4. If the toolchain is unavailable, record the exact failure; a diagnostic attempt with 1.99.1 may identify other issues but must not be reported as MSRV success. Run relevant tests only if the command/toolchain and test database are available; do not expose environment variable values. If no PG database is configured, say the PG integration gate was not run.
Write report sections §0 purpose, §1 change matrix, §2 verification summary, §3 known gaps, §4 subagent failure handoff, §5 guard rules, §6 sign-off, §7 revision history.
Commit only changed files from the owned manifest with author Ulysses <ulysses@mavis.local>. No rebase, merge, push, cleanup, override, or main modification. Before handoff, list commit SHA and files changed.


## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
