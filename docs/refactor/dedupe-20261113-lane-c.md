# Lane C frontend dedupe + dead code (2026-11-13)

Branch `dedupe/lane-c`. Scope: `frontend/src/**`. Behaviour-preserving.

## Baseline vs after
| | tsc errors | vitest files | vitest tests |
|---|---|---|---|
| baseline | 0 | 128 pass | 1064 pass |
| after | 0 | 128 pass | 1064 pass |

## Deleted (evidence)
- `frontend/src/components/.hermes-tmp.suY3mW`: 0-byte stray temp file, no references anywhere.
- `frontend/src/components/worktree-shared/StartWorktreeButton.tsx`: zero importers (static/relative/alias scan of src, e2e, config), zero mentions in docs/crates/scripts.

## Duplicate groups found, NOT merged (implementations diverge; merging would change behaviour)
Same-basename pairs, all verified non-identical (line counts differ):
- `components/DiffAnnotator.tsx` (352) vs `components/diff/DiffAnnotator.tsx` (203); both have importers/tests.
- `components/StartFromPicker.tsx` (358) vs `components/worktree-shared/StartFromPicker.tsx` (304) (+ divergent tests).
- `lib/agent/credentials.ts` (129) vs `lib/api/credentials.ts` (117).
- `lib/remote/wsClient.ts` (144) vs `lib/terminal/wsClient.ts` (276).
- `lib/seed.ts` (513) vs `mocks/seed.ts` (22); `lib/api.ts` vs `lib/arg/api.ts`; `lib/store.ts` vs `lib/arg/store.ts`; `components/Tabs.tsx` vs `components/ui/tabs.tsx`.
- Local helper copies with different semantics (`formatDate` in DocsTab vs AuthenticatedWorktreeIndex, `formatDuration` x2, `relativeTime`): output formats differ, left as is.

## Zero-importer files deferred (not deleted)
Static scan found 40 candidates with no importers. Each is either specified by design docs/briefs/crate code as planned product surface, or is a barrel/entry referenced by e2e/config, so deletion is a product decision:
- Charts `components/charts/Chart01..15`, `shared/ChartFrame.tsx`, `shared/mod.ts` (docs/basic-design/charts-and-reports.md).
- `AnimeCore3D`, `SpawnSubtaskModal`, `AgentRunner`, `CredentialTab`, `useChatStream`, `ComposerPanel`, `LockConflictToast`, `LockStatusPanel`, `PanelPlaceholder`, `XtermPane`, `useKms`, `useWorkItemSync`, `lib/exclusion/trace_propagator.ts` (all named in docs/briefs/ADRs).
- Barrels/entries: `diff/index.ts`, `editor/index.ts`, `terminal/index.ts`, `lib/arg/index.ts`, `mocks/index.ts`, `lib/redirects.ts` (used by next.config/e2e), `lib/utils.ts` (cn), `WorktreeCard`, `ProjectWorktreeIndexLink`.
