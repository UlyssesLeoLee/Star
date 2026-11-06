# PHASE-WBS-INTEGRATION-LEDGER-20261105

Status: **Partial / audit-only**  
Target: local `dev` (`a2649d3c`)
Scope: parallel WBS lane evidence, selective integration decisions, and safe worktree convergence.  
This ledger records merge readiness; it does not convert a design, mock, preview, or partial verification into a completed product capability.

## §0 Purpose

Provide one current, reviewable ledger for the parallel WBS lanes used during the `dev` convergence pass. Each row records the accepted evidence, the reason a lane is held, and the next gate. Dirty or owner-controlled worktrees remain untouched.

## §1 Lane completion matrix

| Lane | Current evidence | Decision | Next gate |
|---|---|---|---|
| Rust 1.99.1 runtime | `ebdde739` integrates the reviewed runtime subset (17 files). | **Merged to dev** | Keep source documentation lane separate; do not merge the source branch wholesale. |
| Rust 1.99.1 Vercel docs | Source `be154efb`; seven active SRS/ADR/BD/DD/TD/brief/report files are ahead of dev. | **Held** | MAGOS terminal override `facfb70d879b`, then selective copy and document review. |
| D2 Multipass/K3s maintenance | Source `56d5c976`; nine maintenance files are reusable, three deployment documents conflict with current W7 filtered-context behavior. | **Held / adapt** | MAGOS override `e6cf73316e19`; rewrite the three documents before integration. |
| WLD7 collaboration brief | `639b495b` adds one historical brief only; later dev contracts supersede its referenced versions. | **Optional historical archive** | MAGOS override `bd91fedeefce`; retain as historical, never as completion evidence. |
| VCD historical briefs | Four briefs are absent from dev; the phase report is already represented by the canonical review history. | **Held / archive-only** | MAGOS override `b583957c4cd5`; label `historical/superseded`, then recover the clean worktrees. |
| W8 native transport | `0ae31044` contains useful adapter direction but has a real PostgreSQL `source_sequence` decode defect and split terminal claims. | **Do not merge** | Build a repair lane with shared-position claim and real PostgreSQL acceptance. |
| W2 BFF lock | Branch tip is equivalent to dev; the owner worktree has an uncommitted `bff/Cargo.lock` change. | **Preserve** | Owner handoff, dependency/SBOM review, and a new committed artifact. |
| W16 device revoke | Branch was fully contained in dev; ignored evidence was archived before cleanup. | **Completed / cleaned** | Archive: `C:/Users/leo19/.codex/archives/Star/w16a-device-revoke-20261105`. |
| Star-only release artifacts | Canvas release workflow allowlists three service images and applies filtered contexts; policy tests pass. The separate `deploy/k3s-local/Dockerfile` still accepts a broad `crates` build context, so it is not yet equivalent evidence. | **Partial / held** | Classify or filter the k3s-local context, then rerun the game/legacy exclusion checks against the actual context and image. |

## §2 Verification summary

- `cargo +1.99.1 metadata --no-deps --format-version 1 --locked`: exit 0 on current dev.
- `cargo +1.99.1 check -p star-api-rest --all-targets --locked -j 4`: exit 0, 58.39s; existing warnings only.
- Canvas release-context policy suite: 35 tests, 35 passed.
- Current dev worktree: clean; no push or main mutation performed.
- Main comparison: `git cherry dev main` showed all main-only commits as patch-equivalent (`-`); no main-leading change was copied.

## §3 Known gaps

1. The three active integration lanes above cannot be created until the user registers the exact MAGOS overrides in a real interactive PowerShell terminal.
2. D2 deployment documentation still needs the current Multipass-only, filtered-context wording; the old “no supported build path” statement is not accepted.
3. W8 requires shared source-position ownership, the missing SELECT column, and real PostgreSQL/RLS/concurrency evidence.
4. Vercel build compatibility and hosted install/sign/rollback remain `NOT_RUN`.
5. The k3s-local Docker path still needs an explicit Star-only context proof; `.dockerignore` alone is not acceptance evidence.
6. Active W8/WLD1/W16/D2/W2 worktrees contain untracked, ignored, or dirty evidence and remain protected from cleanup.

## §4 Sub-agent handoff record

- Rust baseline audit: confirmed the 1.99.1 runtime subset and identified the seven-document Vercel delta; no files changed.
- D2 audit: identified nine reusable maintenance files and three documents requiring adaptation; no files changed.
- Canvas/WLD7 audit: confirmed the design is compatible in principle but requires current spec/data/test traceability; no files changed.
- W8 audit: found the PostgreSQL decode and cross-terminal claim blockers; no files changed.
- Artifact audit: confirmed the Canvas image allowlist and game/legacy fail-closed tests; flagged the k3s-local broad build-context path and README bypass for a separate filtered-context gate.
- One follow-up audit agent later reached its usage limit; no unreviewed code was accepted from that lane.

## §5 Merge and safety gates

- Integrate only reviewed, minimal file sets; never merge a source branch wholesale because it exists.
- Revalidate target `dev` HEAD and cleanliness immediately before every fast-forward.
- Preserve dirty/untracked owner content; archive evidence before removing a completed worktree.
- Keep Rust 1.99.1 aligned across active manifests, toolchains, CI, and focused gates.
- Keep Multipass as the local K3s VM provider; do not introduce Docker Desktop as a prerequisite.
- Do not treat mocks, seed data, process-local queues, or design-only contracts as production acceptance.
- Do not modify `main`, push, or release/tag from this ledger lane.

## §6 Sign-off

| Role | Decision |
|---|---|
| Architecture | Mavis review: accepted runtime merge; remaining lanes held by evidence. |
| SRE / platform | Open until Multipass, PostgreSQL, and recovery gates are independently verified. |
| Security | Open until owner authorization, RLS, and provider/profile identity checks pass. |
| Review host | Mavis review per DEC-008; no user-terminal override is implied by this report. |
| PM | Current scope remains partial; no stable release claim. |

## §7 Revision history

| Version | Date | Revision | Author |
|---|---|---|---|
| v0.1 | 2026-11-05 | Initial convergence ledger; records merged runtime, protected lanes, and exact next gates. | Ulysses (Mavis review per DEC-008) |
| v0.2 | 2026-11-05 | Rebased the target reference to current `dev` and recorded the separate k3s-local Star-only artifact gate. | Ulysses (Mavis review per DEC-008) |

