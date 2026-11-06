# PHASE — STAR Rust 1.99.1 implementation candidate pre-audit

## §0 目的

Before integrating a Rust 1.99.1 baseline change, compare candidate commits c640eb34 and be154efb against dev@655f675ad6381ba5838fa37f191d5329a7b2a463. This is a pre-audit of the earlier candidate, not the later implementation-worker review and not integration approval.

## §1 改动矩阵

| Area | Candidate result | Decision |
|---|---|---|
| Rust toolchain, Cargo rust-version, AGENTS, supported CI | Active values align to 1.99.1; official release and upstream action ref verified | ACCEPT; hosted CI remains open |
| Canvas/k3s-local Rust builder image indices | Docker Registry v2 HEAD verified both pinned manifest-list digests; no Star image was built | ACCEPT config; build/runtime gates remain open |
| Acceptance runners, fixtures, scaffold template | Defaults/selectors and template mostly align; actual device-runner minimum remains 1.99.0 | ADAPT before integration |
| Requirements/Basic Design/Group DD/WBS | WBS and Vercel current gates updated; some normative Infrastructure acceptance rows still say 1.99.0 | ADAPT current wording; preserve actual historical outcomes |
| Old reports and dated WBS evidence | Prior-version run results remain intact | ACCEPT; do not rewrite history |
| Entire source branch | Contains unrelated ancestor de29ef04 outside reviewed scope | HOLD whole-branch integration |

## §2 验证摘要

- Rust official release page confirms version 1.99.1 and publication date 2026-10-15: https://blog.rust-lang.org/2026/10/15/Rust-1.99.1/.
- Upstream action commit 683b588c487a69d01f6c6ddf40f036ff682cc2c8 is titled “Add 1.99.1 patch release”: https://github.com/dtolnay/rust-toolchain/commit/683b588c487a69d01f6c6ddf40f036ff682cc2c8.
- rtk rustc +1.99.1 -V passed: rustc 1.99.1 (58cf8f9c3 2026-10-13).
- rtk cargo +1.99.1 -V passed: cargo 1.99.1 (5f94df478 2026-08-27).
- Docker Registry v2 anonymous HEAD returned 200 and matching Docker-Content-Digest for candidate image-index digests sha256:cfcbed05398d0f6f81b7c3a24579475c567f97007ae7837ab421f9ca3e66333d and sha256:7d53bccd41fde2f509a276b37a8ef541f8e75fc3e1f85b059230386f512ed6c1.
- rtk python -m unittest discover -s scripts/automation -p test_w16a_device_revocation_http_pg_acceptance.py -v: 16/16 passed.
- rtk python -m unittest discover -s scripts/automation -p test_star_k3s_context.py -v: 20 run, 19 passed, 1 Windows WinError 1314 symlink-permission skip.
- rtk python -m unittest discover -s scripts/release/tests -p test_star_canvas_context_policy.py -v: 35/35 passed.
- rtk python -c "from scripts.automation.w16a_device_revocation_http_pg_acceptance import selected_toolchain; print(selected_toolchain('1.99.0', None))" returned ('1.99.0', None), confirming the P1 minimum-version defect despite the suite passing.
- CodeRabbit was unavailable because the installed executable was denied permission by the environment. No workspace Cargo build/test, hosted CI, image build, PostgreSQL, Multipass or K3s runtime gate was run.

## §3 已知缺口

1. MIN_RUST_TOOLCHAIN is still (1, 99, 0) while runner help/error text claims 1.99.1. Add an explicit 1.99.0 rejection test and fix before integration.
2. Requirements AC-INFRA-002-R/S/U/V/W and related Basic Design §16.44–§16.47 / Group DD acceptance text still name 1.99.0. Preserve dated test results, but update forward-looking requirements or mark the acceptance rows as historical.
3. The previous phase report's claim that all remaining 1.99.0 strings are historical is overbroad until the normative rows above are reconciled.
4. The unrelated generated path D:\Star-worktrees\dev-skeleton was reported by the coordinator and not accessed; it does not affect this review. The source branch was clean by tracked Git status after focused suites.
5. No decision is made here on hosted CI, all-targets/workspace build, final image, target database, Vercel, Multipass/K3s, or release readiness.

## §4 子代理失败接手清单

No child agents were dispatched by the review lane. A separate implementation worker owns a different registered worktree; its result has not been reviewed here and remains pending the coordinator's follow-up.

## §5 守门规则

1. Review only c640eb34/be154efb for this pre-audit; do not integrate or clean any worktree.
2. Preserve historical compiler/test evidence exactly as run.
3. Require the runner to reject Rust 1.99.0 when claiming a 1.99.1 minimum.
4. Distinguish config/manifest identity checks from actual build, CI, database and runtime acceptance.
5. Do not integrate the whole branch containing unrelated de29ef04.
6. Re-review the implementation worker commit against the current dev SHA and manifest before any later integration decision.
7. Keep unsupported execution/release gates fail closed.

## §6 签字栏

| Role | Decision |
|---|---|
| Architecture | Mavis 接手审核 — the 1.99.1 direction is valid; runner and acceptance-wording adaptations remain open. |
| SRE Lead | Pending full workspace and hosted CI evidence. |
| Platform | Pending actual Star image/context build and target runtime evidence. |
| Review主持 | ADAPT; pre-audit only, not integration approval. |
| PM | User-directed current toolchain is Rust 1.99.1. |

## §7 修订历史

| Version | Date | 修订人 | Change |
|---|---|---|---|
| v0.1 | 2026-11-06 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Compare c640eb34/be154efb with dev; record file-group dispositions, focused evidence, runner minimum defect, active acceptance-wording drift, and integration holds. |
