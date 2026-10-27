# Brief: WLD7-W16-CLIENT-WAKE-20261027

**Agent**: worker
**Phase**: WLD7-W16-EVENT-PUSH-20261027
**Created**: 2026-10-27 15:31:35

---

## Objective
Replace the fixed-interval Canvas Outbox poller with an event-triggered client that uses the backend authenticated SSE wake stream and the existing durable Outbox page API for all content and recovery.

## Fixed contract
- Connect to `GET /api/v1/worktrees/{worktree_id}/canvases/{canvas_id}/events/stream` using authenticated `fetch()` streaming so the existing injected token callback/generation can be used; do not rely on EventSource if it cannot carry the bearer token.
- The stream emits `canvas-wake` with no document content. On open, run one authorized catch-up; on each wake, call the existing bounded Outbox page reader and coalesce concurrent wakes. On `resync_required`, stream close, network failure, or reconnect, re-open with bounded exponential backoff and catch up from the last cursor.
- Keep browser cursor state as an untrusted resume hint only. Every REST page remains freshly authorized. A missing token/401 invalidates the current injected host-session generation; 403 remains scoped authorization and no mutation is replayed.
- Abort/cleanup must stop the fetch reader, reconnect timer, and pending page reads when the selected Canvas/Worktree changes or component unmounts. There is no fixed-interval polling fallback.

## Owned files
Only edit: `frontend/src/lib/group/worktreeGroupApi.ts`, `frontend/src/lib/group/groupProjection.tsx`, and `frontend/src/lib/group/__tests__/worktreeGroupApi.test.ts`. If the manifest lacks a file actually required, stop and report before editing it. Do not edit backend or docs; the coordinator will integrate against the fixed contract and synchronize docs.

## Required implementation and evidence
- Preserve existing Canvas projection semantics while eliminating the 1-second timer path.
- Bound reconnect delay, in-flight page reads, queued wake signals, and local cursor state. Duplicate wakes may coalesce; REST offset pagination must still process all committed events.
- Add focused tests for wake-to-catch-up, duplicate coalescing, reconnect/backoff, resync, abort on scope change, fresh token on reconnect, 401 invalidation, and 403 retention.
- Run `rtk`-wrapped focused frontend test/typecheck commands. Report exact results, including any unavailable or pre-existing failure.
- Keep Cypher relationship headers accurate and pass the repository CypherGraph guard for changed sources.

## Constraints
Base is the registered coordinator commit cut from `dev@4449bd98d39d2f7b57aceed36d484c7635cb4c28`. Work only in the frontend lane path supplied by the coordinator. Preserve `D:\Star` user data, all old worktrees, `main`, and unrelated files. Do not rebase, merge, push, delete refs, or clean up; the coordinator owns integration. Use `rtk` for Git, dispatcher, and Cargo commands. Return changed files, test evidence, known gaps, and your commit SHA.


## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
