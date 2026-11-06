# PHASE — STAR Rust 1.99.1 implementation candidate review

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
4. The initial scaffold `--help` invocation wrote to the unrelated generated path D:\Star-worktrees\dev-skeleton. This review did not access that path. Read-only inspection of the generator confirms it may have overwritten same-name pre-existing files; their prior contents and actual impact are unknown, and the path remains untouched. The implementation source branch remained clean after focused suites.
5. The final review does not close hosted CI, all-targets/workspace build, final image, target database, Vercel, Multipass/K3s, or release readiness.

## §4 子代理失败接手清单

No child agents were dispatched by the review lane. The separate implementation worker's final combined commits were reviewed after coordinator follow-ups.

## §5 守门规则

1. Review only c640eb34/be154efb for this pre-audit; do not integrate or clean any worktree.
2. Preserve historical compiler/test evidence exactly as run.
3. Require the runner to reject Rust 1.99.0 when claiming a 1.99.1 minimum.
4. Distinguish config/manifest identity checks from actual build, CI, database and runtime acceptance.
5. Do not integrate the whole branch containing unrelated de29ef04.
6. The implementation worker commits were reviewed against dev@655f675a and the registered manifest; any later integration still requires a fresh MAGOS check.
7. Keep unsupported execution/release gates fail closed.

## §6 签字栏

| Role | Decision |
|---|---|
| Architecture | Mavis 接手审核 — the 1.99.1 direction is valid; runner and acceptance-wording adaptations remain open. |
| SRE Lead | Pending full workspace and hosted CI evidence. |
| Platform | Pending actual Star image/context build and target runtime evidence. |
| Review主持 | ACCEPT for selective local-dev integration after a fresh allowed MAGOS check; normative docs sync and product/release gates remain open. |
| PM | User-directed current toolchain is Rust 1.99.1. |

## §7 修订历史

| Version | Date | 修订人 | Change |
|---|---|---|---|
| v0.2 | 2026-11-06 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Review fac360a904bafdec794247e3dc38c2781785d6ef, 67e4beb83556d76a89cd256df71b2fb6665c33b8, and 00dbd30076e8c3ac8ab105328c83e4f736bdaadf; independently rerun six focused suites; confirm 15-path scope, 1.99.0 rejection and preserved historical results; record remaining normative doc gap and side-effect impact as unknown. |
| v0.1 | 2026-11-06 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Compare c640eb34/be154efb with dev; record file-group dispositions, focused evidence, runner minimum defect, active acceptance-wording drift, and integration holds. |

## 8. Final implementation review

- **Candidate:** implementation lane HEAD `00dbd30076e8c3ac8ab105328c83e4f736bdaadf`; base is `655f675ad6381ba5838fa37f191d5329a7b2a463`. Its three commits change exactly the 15 manifest paths. Worker worktree is clean; `rtk git diff --check` passes.
- **P0/P1:** none found. Default and minimum are Rust 1.99.1; `selected_toolchain("1.99.0", None)` is rejected and `selected_toolchain("1.99.1", None)` is accepted.
- **Independent focused tests:** W16A device revocation 16/16; Canvas SSE 25/25; OAuth 5/5; phase 9F2 binding 11/11; K3s context 19 pass/1 WinError 1314 skip; Canvas context policy 35/35. Total 111 passed, 1 skipped, 0 failed. Implementation code did not change in the last two documentation-only commits.
- **Resolved review findings:** WBS W2/W7 forward-looking gates now say 1.99.1, while recorded 1.99.0 failures and historical WLD-3 47/47 evidence remain unchanged. The report now states that the initial scaffold invocation may have overwritten same-name files under D:\Star-worktrees\dev-skeleton; prior content and impact are unknown, and the path was not inspected or further touched.
- **Open P2:** requirements AC-INFRA-002-R/S/U/V/W, basic-design §§16.44–16.47, Group DD AC-INFRA-002-U/V, and root AGENTS.md still contain normative 1.99.0 wording outside this lane's manifest. Track a separate registered documentation lane; do not claim baseline traceability complete.
- **Integration:** ACCEPT the implementation/WBS lane for selective local-dev integration after a fresh allowed MAGOS plan/check. A previous read-only MAGOS check reported overlap with older Rust/W16 lanes; this review does not override it. Full Rust, CI, DB, runtime, Multipass/K3s and release gates remain open. CodeRabbit could not run because its installed executable returned Permission denied; no source was sent externally.
