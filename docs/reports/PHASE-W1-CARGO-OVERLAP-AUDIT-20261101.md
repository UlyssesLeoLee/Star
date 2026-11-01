# PHASE-W1-CARGO-OVERLAP-AUDIT-20261101 — Cargo overlap review

**Phase status: W1 BLOCKED.** This report is a bounded static audit at `dev@4ee0a7670dcba4e24b5df23eef18e90fa353e8e0`; it does not close W1, approve a release, or validate W8 execution.

## §0 Purpose

Determine whether the Cargo changes on the registered W8 and Dependabot refs can be safely accepted before the selected MIT release metadata is reconciled. Preserve all source refs and the detached W8 dirty worktree.

## §1 Change and disposition matrix

| Source / commit | Relevant change | Decision |
|---|---|---|
| MIT metadata, `9083742a63306f310976b820faaa02a400a17ab8` | 13 Star package manifests now declare MIT (enumerated in `docs/reviews/W1-CARGO-OVERLAP-AUDIT-20261101.md`). Root workspace still declares Apache-2.0. | Keep current changes; W1 remains blocked pending root reconciliation. |
| W8 `0ae310440e446f530a210ec413efb526cb038d8f` | Adds `bi-native-event-adapter` to workspace, package manifest and lock entries; owner receive/source-origin implementation and migrations. | Adapt exact W8 paths later after W1 license reconciliation and W8 gates; do not merge old branch tree. |
| W8 source-position `bd90aecc1c39bdf7e647cd9768d36d7db1d55d0c` | Adds claim migration, PostgreSQL test, phase report; no additional Cargo changes. Report records first Inbox decode failure due to missing `source_sequence` in SQLx selection. | Hold pending fix and successful PostgreSQL/RLS/concurrency verification. W8 stays OPEN. |
| Dependabot `48f261011a51d14c44ca0182f99ebdade7a2bce5` | `serial_test`, `nix`, `wasm-bindgen-test`, `tokio-tungstenite` upgrades and lock refresh. Full ref also has stale historical workspace content including `canvas-game`. | Reject full-ref merge; defer isolated dependency updates for compatibility review. |

## §2 Verification summary

| Check | Result |
|---|---|
| Fixed target and source commit identities | Read-only `git rev-parse` / `git show` confirmed the SHAs listed above. |
| License manifest inventory | Confirmed 13 package manifests changed by commit `9083742a`; `Cargo.toml:187` remains `Apache-2.0`; `LICENSE:1` is `MIT License`. |
| Stable release license gate | Static inspection of `.github/workflows/publish-star-canvas-services.yml:109-117` confirms stable tags are blocked unless root workspace metadata and LICENSE heading both equal MIT. |
| Canvas context guard | Static inspection confirms one-service generated workspace and foreign crate/game source rejection (`.github/workflows/publish-star-canvas-services.yml:120, 155-164, 207-216, 326-328`). Each Canvas manifest inherits the workspace license; the workflow enforces root workspace license and LICENSE heading as MIT and validates service name/build/version/dependency closure. No image/SBOM build was performed. |
| W8 PostgreSQL status | Read committed source-position phase report; it records first Inbox decode failure. This lane did not rerun the test. |
| Tests, compilation, generated local k3s context/image | Not run. This task was a source/ref audit, not an implementation validation. |
| Diff formatting gate | `rtk proxy git diff --check` run before commit; see commit validation below. |

## §3 Known gaps

1. Root workspace metadata is still Apache-2.0 despite the selected MIT license and 13 package manifests already set to MIT. The release tag gate is therefore expected to fail; W1 remains BLOCKED.
2. The source-position PostgreSQL decode failure is unresolved in the inspected committed report. No database, RLS, concurrency, recovery, or ACK verification was rerun here.
3. Dependabot version updates have not been checked against the current dev tree with compile/test gates or API migration review.
4. Canvas workflow checks are static; actual filtered context, image contents, and SBOM were not produced or inspected.
5. The local k3s image uses a generated context. This report did not execute the generator or inspect the resulting context/image.
6. The detached W8 adapter worktree's uncommitted contents and active MAGOS claims remain unverified and untouched.

## §4 Subagent / handoff failure list

No child agent was dispatched by this lane. The inspected W8 source-position report itself records a failed first PostgreSQL Inbox decode because the SQLx query omitted `source_sequence`; later scenarios did not run. Handoff that exact failure to the W8 owner; do not treat the report's source-level review as a passing database gate. Preserve the detached W8 source as-is.

## §5 Guardrails carried forward

1. Keep the stable license decision MIT across root Cargo metadata, package metadata, LICENSE, release workflow, and generated package contexts.
2. Do not merge an old branch tree when only one independent commit/path set is relevant; review and select exact commits.
3. Keep W8 source refs and worktrees unchanged; this phase is read-only against them.
4. Do not read the detached W8 uncommitted files or root `D:\Star` dirty document bodies.
5. A Cargo workspace member is not automatically an approved container payload; inspect resolved build contexts and final artifacts.
6. `.dockerignore` or workflow declarations alone do not prove final image contents.
7. Keep `canvas-game` and unrelated legacy packages out of Star release payloads.
8. Canvas per-service allowlist checks do not prove the separate generated k3s-local image is clean.
9. Do not claim a source-position PostgreSQL gate passed when the committed report records decode failure.
10. No Cargo/lockfile or business-source changes are permitted in this review lane.
11. Do not alter `main`, push, rebase, cherry-pick, reset, clean, delete, or merge source refs.
12. Do not call W8 or W1 complete without required runtime evidence.
13. Keep dependency updates separate from license metadata changes.
14. Do not treat a lockfile refresh as proof of API/MSRV compatibility.
15. Preserve the detached W8 worktree and active MAGOS claims.
16. Recheck target HEAD and clean status before integrating any later reviewed documentation lane.
17. Keep release approval with the accountable release owner; no tag or release was created here.

## §6 Sign-off

| Role | Status |
|---|---|
| Architecture | Mavis review: bounded findings accepted; W1 remains BLOCKED. |
| SRE / platform | Mavis review: actual generated k3s context, image and runtime gates remain open. |
| Review host | Mavis review: source refs inspected read-only; detached worktree excluded. |
| PM / release owner | Stable release not approved; MIT root metadata gate is unresolved. |

## §7 Revision history

| Version | Date | Editor | Change | Trigger |
|---|---|---|---|---|
| v0.1 | 2026-11-01 | Ulysses — Mavis review | Recorded Cargo overlap findings, MIT mismatch, W8 PostgreSQL failure evidence, and release-payload limits. | W1 Cargo overlap audit brief. |
