# STAR Rust 1.99.1 Candidate Review — Pre-audit

**Review date:** 2026-11-06
**Target:** dev@655f675ad6381ba5838fa37f191d5329a7b2a463
**Candidate source:** codex/star-rust-1991-baseline-20261105@be154efb6cff5dd6eea6c85f5c5a5ccc00d8ccd6
**Reviewed commits:** c640eb34 and be154efb only
**Disposition:** **ADAPT; do not integrate yet.** This is the pre-audit only; the separate implementation lane still needs the runner fix and a subsequent review.

## Scope and evidence boundary

The source worktree was inspected read-only. Its tracked status was clean after the focused checks. The branch also contains the unrelated ancestor commit de29ef04 Refine LANE workspace navigation and presentation; it was excluded from this Rust-baseline review and must not be swept into any selected integration. No branch/worktree was merged or cleaned. The separately reported D:\Star-worktrees\dev-skeleton generated path was not inspected or changed; it has no effect on this review.

This review independently checked the active Rust 1.99.1 release source, the pinned action commit, Docker registry manifest identities, selected acceptance suites, and the runner's minimum-version gate. It did not run workspace Cargo build/test, hosted CI, image builds, PostgreSQL, Multipass, or K3s acceptance.

## File-group disposition

| File group | Decision | Review result |
|---|---|---|
| rust-toolchain.toml, root/BFF/crate Cargo metadata, AGENTS.md, supported CI workflows | **ACCEPT** | Active pins and minimum declarations move to 1.99.1. Rust's official release page confirms 1.99.1; the upstream dtolnay/rust-toolchain commit 683b588c487a69d01f6c6ddf40f036ff682cc2c8 is titled “Add 1.99.1 patch release.” This validates the selected version and action ref, not hosted CI. |
| Canvas and k3s-local Rust builder image pins | **ACCEPT, build evidence open** | Anonymous Docker Registry v2 HEAD checks returned 200 and the same Docker-Content-Digest for both candidate image-index digests. No Star image/context build or runtime proof was run. |
| Acceptance runner defaults, selectors, fixture versions, scaffold generated rust-version | **ADAPT** | Most active defaults and fixtures move to 1.99.1, but the device-revocation runner retains MIN_RUST_TOOLCHAIN = (1, 99, 0); its public error/help text says 1.99.1. Directly calling selected_toolchain('1.99.0', None) still accepts 1.99.0. See P1 below. No scaffold output-generation test was run; only its template diff was inspected. |
| Current requirements/design and WBS baseline statements | **ADAPT** | Current WBS gates and Vercel design targets were updated, and actual historical 1.99.0 run records were generally retained. However, active acceptance rows in Requirements AC-INFRA-002-R/S/U/V/W and the corresponding Basic Design §16.44–§16.47 / Group DD AC text still require 1.99.0 while the newly added overview notes say 1.99.1. See P2 below. |
| Dated historical test reports and prior-run evidence | **ACCEPT** | Preserve compiler versions that actually ran those historical tests. Do not rewrite old 1.99.0 results as 1.99.1 results. Update only the current/future acceptance requirement or explicitly label its version as historical evidence. |
| Whole source branch | **HOLD** | The review is limited to commits c640eb34/be154efb. Do not integrate the branch as a unit because it also carries unrelated de29ef04; any later integration must use the separately reviewed lane/selected patches and recheck their base. |

## Findings

### P1 — The current runner minimum still accepts Rust 1.99.0

In scripts/automation/w16a_device_revocation_http_pg_acceptance.py, DEFAULT_RUST_TOOLCHAIN is 1.99.1 and error/help text claims a 1.99.1 minimum, but MIN_RUST_TOOLCHAIN remains (1, 99, 0). The test only rejects 1.98.9. The exact check

    rtk python -c "from scripts.automation.w16a_device_revocation_http_pg_acceptance import selected_toolchain; print(selected_toolchain('1.99.0', None))"

returned ('1.99.0', None). That permits an explicit old-version run to proceed under a runner that claims a 1.99.1 floor, and weakens evidence attribution.

**Required adaptation:** set the actual minimum to 1.99.1 and add an explicit 1.99.0 rejection assertion, retaining the existing 1.98.9 coverage. Re-run the focused suite and record the result before integration.

### P2 — Current acceptance wording conflicts with the new baseline

The new notes in Requirements, Basic Design, Group DD, and WBS declare 1.99.1, but these normative rows remain on 1.99.0:

- docs/requirements.md: AC-INFRA-002-R/S/U/V/W.
- docs/basic-design.md: §16.44–§16.47, including AC-INFRA-002-T/U/V/W.
- docs/design/DD-WORKTREE-GROUP-001.md: AC-INFRA-002-U/V and adjacent acceptance wording.

Those old literals may be valid as-run evidence, but the rows are acceptance requirements, not just a dated run transcript. To keep evidence intact and eliminate ambiguity, update forward-looking requirements to 1.99.1 and preserve the actual Rust 1.99.0 outcomes as dated evidence (or explicitly mark those rows historical). The candidate's blanket statement that remaining 1.99.0 matches are historical is therefore too broad.

## Validation performed

- Official Rust release page: Rust 1.99.1, published 2026-10-15: https://blog.rust-lang.org/2026/10/15/Rust-1.99.1/.
- Upstream action source: dtolnay/rust-toolchain commit 683b588c487a69d01f6c6ddf40f036ff682cc2c8, titled “Add 1.99.1 patch release”: https://github.com/dtolnay/rust-toolchain/commit/683b588c487a69d01f6c6ddf40f036ff682cc2c8.
- Local toolchain probes: rtk rustc +1.99.1 -V → rustc 1.99.1 (58cf8f9c3 2026-10-13); rtk cargo +1.99.1 -V → cargo 1.99.1 (5f94df478 2026-08-27).
- Docker Registry v2 anonymous HEAD checks returned HTTP 200 with a matching digest and application/vnd.oci.image.index.v1+json for both sha256:cfcbed05398d0f6f81b7c3a24579475c567f97007ae7837ab421f9ca3e66333d and sha256:7d53bccd41fde2f509a276b37a8ef541f8e75fc3e1f85b059230386f512ed6c1.
- rtk python -m unittest discover -s scripts/automation -p test_w16a_device_revocation_http_pg_acceptance.py -v → 16/16 passed; this suite does not catch acceptance of 1.99.0.
- rtk python -m unittest discover -s scripts/automation -p test_star_k3s_context.py -v → 20 tests run, 19 passed, 1 skipped because Windows symlink creation returned WinError 1314.
- rtk python -m unittest discover -s scripts/release/tests -p test_star_canvas_context_policy.py -v → 35/35 passed.
- Candidate source status after checks: clean. CodeRabbit could not run because its installed executable was denied permission by the environment; no source was sent to an external review service.

## Integration recommendation

Do not integrate yet. After the P1 runner fix and regression test, reconcile the forward-looking AC wording (or explicitly assign a separate docs lane), rerun focused tests, and review the worker's resulting commit against the then-current dev SHA and registered file manifest. Keep all previously recorded 1.99.0 execution results unchanged. This pre-audit does not approve or review the pending implementation worker commit.
