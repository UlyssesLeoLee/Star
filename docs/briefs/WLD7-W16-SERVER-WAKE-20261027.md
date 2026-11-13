# Brief: WLD7-W16-SERVER-WAKE-20261027

**Agent**: worker
**Phase**: WLD7-W16-EVENT-PUSH-20261027
**Created**: 2026-10-27 15:31:35

---

## Objective
Implement the backend owner-triggered Canvas event wake stream as a bounded, authorization-checked adapter over the existing durable Canvas Outbox. The existing paged Outbox API remains the only source of event details and recovery truth.

## Fixed contract
- One active Worktree context per browser window uses one authenticated, multiplexed SSE connection at `GET /api/v1/worktrees/{worktree_id}/canvas-events/stream`; do not create one stream per Canvas/App tab.
- The stream emits only `canvas-wake` with an empty payload, or `resync_required` before a bounded disconnect; no document, draft, connector, patch, or resource contents are sent. The browser reads details from the existing paged Worktree Canvas Outbox API.
- PostgreSQL `NOTIFY` is a transactional wake-up only. A trigger on `canvas.canvas_group_outbox` publishes only a validated Worktree scope identifier. The Outbox table remains durable truth; notification loss, duplicates, or reordering must be recovered by authorized REST pagination.
- Run one `PgListener` per long-lived Rust API process, not one per browser and not inside a Vercel Function. Each server process receives the DB notification and wakes only its local subscribers. Vercel hosts the UI and connects to the separately hosted Rust API/gateway.
- Bound the shared broadcast queue, total SSE connections, and event metadata bytes. Slow consumers must get `resync_required` and disconnect; never silently skip durable events.
- Reauthorize the current Project/Run/Worktree scope before every wake is sent; each Outbox catch-up request independently reauthorizes its resource scope. Revoke or invalid scope => close stream. Do not compare canvas_id to tenant_id.

## Owned files
Only edit: `crates/star-api-rest/src/group_api.rs`, `crates/star-api-rest/src/group_api/canvas.rs`, `crates/star-api-rest/src/group_api/canvas_event_stream.rs` (new), `crates/star-api-rest/src/main.rs`, `crates/star-api-rest/Cargo.toml`, `db/migrations/2026-10-28-canvas-outbox-wakeup.sql`, and `crates/star-api-rest/tests/canvas_event_stream_pg.rs` (new). If the manifest lacks a file actually required, stop and report before editing it. Do not edit frontend or docs; the coordinator will synchronize docs after review.

## Required implementation and evidence
- Add a transaction-safe notification trigger after outbox insert. Notification payload must be small, validated, and contain no content. Same worktree/canvas notifications may coalesce within a DB transaction.
- Add one process-level listener and a bounded, shared in-process fanout. Listener failure/reconnect must force affected streams to resync/reconnect so events committed during downtime are caught up from the durable Outbox.
- The SSE request must bind to the authenticated actor and resource scope. Recheck current authorization on delivery; no stale authorization reuse.
- Add focused tests for notification commit/rollback behavior where the available PostgreSQL harness allows it, scope filtering (including tenant_id != canvas_id), auth revocation/denial, bounded lag behavior, and duplicate wake recovery. Run `rtk`-wrapped focused Rust formatting/check/test commands with `-j 4` where supported. Clearly report any DB test that cannot run because an authorized Neon or Multipass PostgreSQL endpoint is unavailable; do not invent a mock as production evidence.
- Keep Cypher relationship headers accurate and pass the repository CypherGraph guard for changed sources.

## Constraints
Base is the registered coordinator commit cut from `dev@4449bd98d39d2f7b57aceed36d484c7635cb4c28`. Work only in the backend lane path supplied by the coordinator. Preserve `D:\Star` user data, all old worktrees, `main`, and unrelated files. Do not rebase, merge, push, delete refs, or clean up; the coordinator owns integration. Use `rtk` for Git, dispatcher, and Cargo commands. Return changed files, test evidence, known gaps, and your commit SHA.


## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
