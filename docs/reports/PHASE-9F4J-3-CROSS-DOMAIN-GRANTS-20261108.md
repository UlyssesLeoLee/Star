# Phase Report: 9F4J.3 Cross-Domain Runtime Grants

**Phase:** WBS-9F4J-3-CROSS-DOMAIN-GRANTS-20261108
**Base:** `dev@01563e936c0d4b051382909b5725ed67feea1cea`
**Status:** implementation and focused build complete; independent review `PASS WITH LIMITATION`; integrated locally to `dev@eccdd6f5452a`; PostgreSQL acceptance `NOT_RUN`.

## §0 Purpose

Strengthen the existing opt-in PostgreSQL grants test so a BI runtime login/current role is checked against the reviewed platform transport runtime boundary. This lane adds test evidence only: it does not change application code, database grants, migrations, or production policy.

## §1 Change Matrix

| Registered file | Change |
|---|---|
| `crates/bi-owner-runtime/tests/task_event_inbox_runtime_grants_pg.rs` | Added an ignored cross-domain privilege test. It requires the expected BI runtime membership, rejects membership or `SET` capability to any other `star_*` role, and checks effective `EXECUTE` on the seven exact transport functions from the existing migrations. Missing signatures are reported explicitly. |
| `docs/briefs/9F4J-3-CROSS-DOMAIN-GRANTS-20261108.md` | Preserved the dispatcher-created implementation brief. |
| `docs/reports/WBS-DISPATCH-9F4J-3-CROSS-DOMAIN-GRANTS-20261108.jsonl` | Added this lane's focused verification record and database `NOT_RUN` status. |
| `docs/reports/PHASE-9F4J-3-CROSS-DOMAIN-GRANTS-20261108.md` | Added this phase report. |

Independent review confirmed that the test checks both `session_user` and `current_user` against non-allowlisted Star role membership/SET paths and effective `EXECUTE` on the seven reviewed transport functions. The test lane was fast-forwarded into local `dev` as commit `eccdd6f5452a8dd88d31f397d83860ad80888cf5`; no production grant or database acceptance was claimed.

The test remains `#[ignore]` and uses the existing dedicated disposable database identity, loopback host, and non-default port checks. No database is created and no credentials are printed. Cargo manifests, lockfile, application source, migrations, canonical WBS, and all files outside the registered manifest were left untouched.

## §2 Verification Summary

| Check | Result | Evidence |
|---|---|---|
| Focused test-target compile with Rust 1.99.1 | **PASS** | `rtk cargo +1.99.1 test -p bi-owner-runtime --test task_event_inbox_runtime_grants_pg --no-run -j 4`, exit 0; target executable built in 17.18 s. |
| Targeted rustfmt | **PASS** | `rtk rustfmt +1.99.1 --edition 2021 --check crates/bi-owner-runtime/tests/task_event_inbox_runtime_grants_pg.rs`, exit 0. |
| Cypher Graph Guardian | **PASS** | `cgg_guard.py --changed --stamp`, 1 source file checked. |
| `git diff --check` | **PASS** | Exit 0. |
| Independent coordinator review | **PASS WITH LIMITATION** | No static P1/P2 blocker found; the review confirmed the exact seven function signatures against the read-only grant migrations. PostgreSQL execution remains `NOT_RUN`. |
| Integration | **PASS** | Fast-forwarded source commit `eccdd6f5452a8dd88d31f397d83860ad80888cf5` to local `dev`; target advanced from `01563e936c0d4b051382909b5725ed67feea1cea`. |
| PostgreSQL grants acceptance | **NOT_RUN** | The ignored test was not executed; no database setup was performed. |

Cargo emitted four `async_fn_in_trait` warnings from the unchanged `crates/bi-owner-runtime/src/task_event_receiver.rs`; compilation had no errors. `--no-run` verifies Rust compilation only and does not parse or execute the embedded SQL against PostgreSQL.

## §3 Known Gaps

- The new catalog assertions have not been exercised against the designated disposable PostgreSQL database. Transport-role and function-grant acceptance therefore remains unverified.
- This test lane does not modify or prove the correctness of production grants, migrations, RLS, or application behavior.
- The test's ignored status is intentional and must remain until a separately provisioned, identity-checked acceptance environment is available.

## §4 Subagent Failure / Takeover

- The implementation worker committed the four registered files as `eccdd6f5452a8dd88d31f397d83860ad80888cf5`; no worker failure required takeover.
- An independent review agent examined the exact PostgreSQL role/function assertions and their migration signatures. It found no static P1/P2 blocker and returned `PASS WITH LIMITATION` because PostgreSQL acceptance has not run.
- The coordinator fast-forwarded the reviewed commit into local `dev`. The ignored PostgreSQL test remains a separate acceptance gate.

## §5 Guardrails

- Scope is limited to the four registered files above.
- The cross-domain test remains opt-in and ignored; database identity checks precede catalog probes.
- It denies both membership and `SET` paths to non-allowlisted `star_*` roles and checks effective `EXECUTE` on the exact seven transport signatures.
- No target database creation, credential output, migration edit, grants edit, manifest/lockfile edit, or application source edit was performed.
- No changes were made to `main`, remotes, protected W8/Dependabot sources, user files, or other lanes. This lane is not integrated.

## §6 Sign-off

| Role | Status |
|---|---|
| Implementer self-review | Pass after final diff review. |
| Independent coordinator review | Pass with limitation; no static P1/P2 blocker. |
| PostgreSQL acceptance | `NOT_RUN`; not approved. |
| Integration | Fast-forwarded to local `dev` as `eccdd6f5452a8dd88d31f397d83860ad80888cf5`. |

## §7 Revision History

| Version | Revision | Owner |
|---|---|---|
| v1.0 | Added cross-domain role and exact transport-function privilege assertions; recorded focused build, formatting, graph guard, diff-check, and PostgreSQL `NOT_RUN` status. | Ulysses (Mavis handoff review pending) |
| v1.1 | Recorded independent review as pass with limitation and the local `dev` fast-forward; PostgreSQL acceptance remains `NOT_RUN`. | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 |
