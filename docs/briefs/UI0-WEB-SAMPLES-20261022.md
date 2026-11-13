# Brief: UI0-WEB-SAMPLES-20261022

**Agent**: ui0_bench_harness
**Phase**: UI0_WEB_SAMPLES_20261022
**Created**: 2026-10-22 07:58:15

---

# UI0-WEB-SAMPLES-20261022

## Goal
Complete the UI-0 real shared-Web measurement follow-up in `docs/implementation-plans/WBS-UI-PLAN-NEXT-20261022.md` §6 item 1. Audit the existing Playwright runs; rerun only if an existing approved local server is already available and prior data is incomplete or invalid. Produce reviewable evidence and update the phase report. This is one evidence lane, not UI-0 closure.

## Base / owner
- Base: `dev` at `df1c22f66d8d0c3b48baac7e348494ba3466569b`.
- Worktree: `C:\Users\leo19\.codex\worktrees\ui-ux-next-20261021\Star`.
- Owner: `/root/ui0_bench_harness`.
- Never modify shared `dev` or central WBS from this lane.

## Exact owned file manifest
- `scripts/ui0_bench/playwright_driver.mjs`
- `docs/reports/ui0-web-driver-samples-20261022.jsonl`
- `docs/reports/ui0-web-driver-samples-20261022-r2.jsonl`
- `docs/reports/ui0-web-driver-samples-20261022-r3.jsonl`
- `docs/reports/ui0-web-host-inventory-20261022.json`
- `docs/reports/ui0-web-measurement-run-20261022/measurement.json`
- `docs/reports/ui0-web-measurement-run-20261022/raw_samples.jsonl`
- `docs/reports/ui0-web-measurement-run-20261022-r2/measurement.json`
- `docs/reports/ui0-web-measurement-run-20261022-r2/raw_samples.jsonl`
- `docs/reports/ui0-web-measurement-run-20261022-r3/measurement.json`
- `docs/reports/ui0-web-measurement-run-20261022-r3/raw_samples.jsonl`
- `docs/reports/PHASE-UI0-BENCHMARK-HARNESS-20261022.md`
- `docs/reports/PHASE-UI0-WEB-SAMPLES-20261022.md` (new)
- `docs/briefs/UI0-WEB-SAMPLES-20261022.md` (dispatcher brief)
- `docs/reports/ui0-web-dispatch-20261022.jsonl` (dispatcher audit)

## Required work
1. Revalidate all sample schemas, run metadata, sample counts, timestamps, environment, and fixture/commit identity. Explain the first run's lower count if evidence establishes the cause; keep runs separate.
2. Verify the driver's host allowlist and bounded output; use only the existing localhost server. Do not start/stop/restart or mutate any process. If unavailable, record blocked and keep renderer/threshold decisions open.
3. Summarize valid shared-Web metrics with the method (cold/warm split, count, p50/p95); do not pool incompatible runs. Do not claim Tauri+React or Rust-native parity, release readiness, or threshold acceptance.
4. Correct the existing UI-0 report's “no real UI driver samples exist” statement with evidence; keep Rust/Tauri tracks, queue/cache/terminal/canvas counters, renderer selection, and thresholds OPEN. Add the new sample report.
5. Run existing bounded UI0 standard-library tests, syntax/whitespace checks, and CypherGraph Guardian on changed code. Record exact results and unavailable checks.
6. Make one coherent commit on this lane and return its SHA, paths, metrics, verification, and remaining gaps. Do not rebase or merge; coordinator owns integration.

## Safety
- Keep output bounded and non-secret. Do not inspect env variables or auth storage.
- Browser load timing/JS heap are not Rust desktop memory/CPU evidence.
- Mark unproven provenance UNKNOWN; do not overclaim.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
