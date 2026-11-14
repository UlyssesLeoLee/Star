# Dedupe 2026-11-13 Lane A (exact duplicates / root clutter)

Branch: dedupe/lane-a (from dev). Verified with `git grep -I -F` over the whole tree.

## Removed (33 files)
- Root scratch, no references anywhere: `_ir.json _iss.json _ls.json iss-156/177/191/209/210/211/213.json trace_ids.txt` (11 files)
- `ip-list.json`: only mention is a prose note in docs/requirements/SRS-STAR-VERCEL-DEPLOY-001.md (FR-VC-017) saying such a root IP whitelist is invalid on Vercel; no code/script/CI reads it. Removed (1 file).
- `docs/reports/wiremock-openclaw/*.json` (20 files): content-identical (git blob hash match) to `tools/star-flash-mock/mock_data/openclaw/*` (the tools copy uses `v1--openclaw-*` names). Only prose mentions of the old path remain (migration notes); nothing reads it. Added README.md pointing to the canonical location.
- `docs/reports/STAR-STABLE-W5-RUN-SCOPE-20261101.log`: identical to its `WBS-DISPATCH-...jsonl` sibling; its path is not referenced anywhere.

## Skipped
- `STAR-STABLE-W5-I18N-AGENT-LABELS-20261101.log`, `...VITEST-DISCOVERY-20261101.log`: identical to jsonl siblings but cited by their PHASE-*.md reports as evidence paths.
- `STAR-STABLE-W5-KANBAN-CONTRACT-20261101.log`, `...FRONTEND-MISC-20261101.log`: not identical to the jsonl (extra/different lines); KANBAN-CONTRACT also cited by its PHASE report.
