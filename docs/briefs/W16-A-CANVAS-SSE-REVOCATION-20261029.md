# Brief: W16-A-CANVAS-SSE-REVOCATION-20261029

**Agent**: Codex worker Canvas SSE revocation PG acceptance
**Phase**: W16-A Canvas SSE authorization revocation acceptance
**Created**: 2026-10-29 13:20:27

---

Objective: add a real PostgreSQL integration acceptance test proving that an already-open Worktree Canvas SSE stream is closed promptly after the bound device authorization is revoked or its generation changes. Baseline is dev@8704466be335d2a3e9c12ba7a8d7da44e9e6767c.

Evidence/context: Parent CodeGraph found CanvasEventWakeHub uses one PgListener, a bounded broadcast queue, a PostgreSQL device-binding wake channel, and reauthorizes an active stream after DeviceBindingChanged; errors close the stream. The existing authorization helper is reauthorize_worktree_canvas_read. The graph did not find a covering revocation test. Existing ignored PG tests use STAR_W16_DEVICE_AUTH_TEST_DATABASE_URL. The feature already exists; this task proves active-stream revocation behavior through a real PG trigger/NOTIFY path.

Implement only crates/star-api-rest/tests/canvas_device_auth_pg.rs and the unique brief/report/dispatch files in this lane manifest. Create an authorized Run/Worktree/device fixture using persisted DB facts and production router/hub path as feasible. Open the SSE response, consume/confirm stream setup, then atomically update/revoke device binding and assert the stream terminates within a bounded deadline after PostgreSQL notification. Avoid polling loops, arbitrary sleeps, in-memory authorization doubles, or testing only a later HTTP request. Exercise the trigger installed by the current W16 migration. Make the test ignored only when it requires the explicit isolated test DB env variable; the acceptance runner later must execute it rather than silently skip it. If a production source file needs modification, stop and report the exact path and reason because it is outside this lane manifest.

Do not edit shared WBS, test-design, requirements, basic/detailed design, automation registry, scripts, or files outside this manifest. Record exact verification commands/results and limits in the phase report. Do not claim W16-A complete. You are not alone in the repository: stay inside this lane, preserve all unrelated work, and never revert other agents' edits.

The repository-required dispatcher brief has been created before dispatch.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
