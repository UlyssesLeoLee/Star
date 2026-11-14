# Lane E dedupe report (scripts / tools / .github / deploy) - 2026-11-13

Branch `dedupe/lane-e` (base local `dev`). Behaviour-preserving; AGENTS.md audit-evidence rule respected (no script deleted or moved).

## Baseline checks (before == after, no script touched)
- `scripts/automation/registry_check.py`: exit 0; OK 52 / WARN 263 (unregistered scripts) / ERR 0.
- `scripts/automation/smoke_test.py`: 4/4 passed (dispatcher, cli_helper, refactor_template, judge). Note: it rewrites `docs/briefs/smoke/smoke-dispatcher-001.*` and `docs/reports/*.log` as side effects; reverted via `git checkout`.
- Tracked file counts: scripts/automation 327 (315 .py), tools 535 (458 are star-flash-mock/mock_data), deploy+.github 45.

## Inventory of scripts/automation (320 scripts: .py/.ps1/.sh)
Classification by cross-reference grep (other file mentions the basename, excluding registry.md self-listing):
- Referenced by another file (active or documented): 235
- Listed only in registry.md: 3
- Unreferenced anywhere: 82 (mostly historic one-offs, plus unit tests for packages run by path: `test_*.py`, `*_v0_legacy.py`, `test_ut_*`)
- One-off by name pattern (fix_/patch_/_insert/_sync/backout/wbs_v*/registry_v*/opt_*/list_err_* ...): 74 (list in the Archive candidates below)
- Exact-duplicate files (md5): 0 across scripts/automation, tools, deploy, .github.

## CI / package.json reference check
All script paths referenced from `.github/workflows/*.yml` exist (scripts/ci/*, scripts/release/*, scripts/automation/{console_server,dispatcher}.py, guardian tests, tools/mock-switch-validate.py, tools/star-flash-mock/scripts/run-all.sh, scripts/build-*.sh). No file touched, so nothing can have regressed.

## Merges performed
None. Evidence that no merge met the "clearly equivalent" bar:
- `wbs_v101_p2_runbook.py` vs `wbs_v103_fmt_fix.py` (and the `registry_*` / `automation_design_*` pairs): same idempotent-insert template but each embeds a distinct historic row payload and anchor; they are one-off audit evidence, so not merged.
- `fix_b4_batch_v2..v15`: 14 sequential iterations of one fixer, each with different patch tables (41-214 lines); historic evidence, not deleted.
- Function-body duplicates (whitespace-normalised hash): `_run` x4 (arg_behavior_eval/arg_e2e_test/arg_perf_bench/arg_ui_test), `_parse_cargo_test_output` x2 (arg_bridge_test/arg_dispatch_test), `run_guard` x4 (contextpanel_5pages/activity/comment/relation), `stop_nats_server` x2 (phase9f4d_*). Factoring them needs a new shared module plus a `sys.path` shim (scripts run standalone, `-I` drops the script dir) and none are covered by smoke_test or runnable here without cargo/pnpm/NATS, so deferred.
- `.github/workflows`: toolchain/checkout steps repeat across 8 workflows (all pin `dtolnay/rust-toolchain@683b588...`); a composite action would unify them but cannot be validated without running Actions (new workflows cannot be dispatched pre-merge), so reported only.
- deploy/: `multipass/cloud-init-{k3s-gitops,fix-gitops}.yaml`, `k3s-local/*` and `helm/star` have no byte-identical files; semantic overlap not proven.

## Deferred / proposals
1. Archive layout (move, not delete, after owner sign-off; leave stub redirect only if referenced): `scripts/automation/archive/{wbs_inserts,registry_inserts,design_inserts,fix_b4,p0_fixes,opt_one_offs,legacy_v0}/`. Candidates (unreferenced one-offs): `_append_v033, backout_b4_misindent, fix_b4_batch_v2..v4,v6..v10,v12..v14, fix_b4_sed20, gen_missing_view_cards, opt_adr_promote_v02, opt_create_4_crates, opt_wtm_resolve_4mixed, opt_wtm_update_counts, wbs_v083/084/087/088_1/096/099/0991/0993/0995/101/103, registry_v101/v103, automation_design_v101/v103, *_v0_legacy`. Referenced-but-historic (need ref update first): `fix_b4_batch_v5/v11/v15, registry_v0_36_sync, wbs_v082/v0996, patch_h1..h3/e1, h2_refactor, list_err_*`.
2. Shared helper module (`scripts/automation/_common.py`: `run()`, `parse_cargo_test_output()`, `run_guard()`, `stop_nats_server()`) after adding smoke coverage.
3. Composite GitHub Action for rust-toolchain + cache steps.
4. registry.md lists only 52 of 320 scripts (263 WARN); registry backfill is a separate docs task.
5. tools/star-flash-mock/mock_data (408 json) not analysed for semantic duplication.
