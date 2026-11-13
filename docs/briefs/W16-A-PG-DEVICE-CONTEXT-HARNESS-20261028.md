# Brief: W16-A-PG-DEVICE-CONTEXT-HARNESS-20261028

**Agent**: worker
**Phase**: W16-A-PG-DEVICE-CONTEXT
**Created**: 2026-10-28 18:00:48

---

Security-review follow-up: the backend lane has been reopened because device challenge creation, enrollment completion, and revocation write OAuth tables without an explicit owner-role path. Expected owner role is the existing star_oauth_credential_owner, pending exact confirmation from the backend owner. Update only the registered harness files. Grant the NOINHERIT test login membership to the accepted OAuth owner role in the disposable fixture, assert owner-role table/column grants and no direct runtime table ACLs, and keep separate star_engineering_directory_owner membership/SELECT for Run/Worktree lock probes. Extend the local runner to invoke the ignored star-pg-adapter device_auth_pg tests on the same isolated local PostgreSQL database using the non-superuser test login; ensure fixture SQL/tests use SET LOCAL ROLE for OAuth owner operations and seed/cleanup only scoped random data. Keep the existing HTTP Run/Worktree phases. Do not change migrations or Rust in this lane. Coordinate exact role/grant contract with the backend owner before finalizing. Rerun full unit+migration+role/RLS+device repository PG tests+HTTP acceptance; update the PHASE report with exact results. Preserve failed evidence, do not start Multipass, and do not claim K3s/Neon validation. Do not commit or merge.

### Backend contract clarification

The backend owner confirmed `star_oauth_credential_owner` as the role set by all five `DeviceBindingRepository` transactions (`create_challenge`, `find_challenge`, `complete_challenge`, `is_active`, and `revoke`) before transaction-local tenant/actor scope is applied. The disposable `erun_directory_api_test` login is non-superuser, `NOINHERIT`, and `NOBYPASSRLS`; it receives explicit membership with `SET ROLE` capability in the OAuth owner and `star_engineering_directory_owner`. It must have no direct device/refresh table or column ACL, and `PUBLIC` must have none.

The exact OAuth-owner ACL allowlist is: `oauth_device_enrollment_challenges` table `SELECT`, `INSERT`, `DELETE` plus column `UPDATE(consumed_at)`; `oauth_device_bindings` table `SELECT`, `INSERT` plus `UPDATE(generation,status,revoked_at)`; `oauth_device_binding_audit` `INSERT` only; and `oauth_refresh_tokens` `SELECT` plus `UPDATE(revoked_at)`. `star_engineering_directory_owner` has table-level `SELECT` and only column-level `UPDATE(generation)` on `oauth_device_bindings` so PostgreSQL `SELECT ... FOR SHARE` can lock the generation column; The current restrictive UPDATE policy scopes USING by both tenant and actor and sets WITH CHECK (false): it permits the scoped row lock but rejects an actual update with SQLSTATE 42501. Refresh-token updates require an OAuth-owner-targeted permissive policy and a restrictive policy that checks both `tenant_id=app.tenant_id` and `user_id=app.actor_id`, without an OR escape. All device owner transactions use `SET LOCAL ROLE star_oauth_credential_owner`.

The harness checks the complete table privilege allowlist, column-level UPDATE allowlist, direct ACLs and grant options, role posture/membership, forced RLS policy shape, and a three-row cross-tenant/cross-actor refresh-token behavior probe, plus four device-binding rows with unfiltered-by-device-id visibility probes for same-tenant other-actor and other-tenant same-actor isolation. It then invokes `star-pg-adapter --test device_auth_pg` with ignored tests enabled using the same disposable local cluster and NOINHERIT login, followed by the existing HTTP Run/Worktree phases. The actual W16 table name is `oauth_device_enrollment_challenges`. Final assertions must match the source lane's accepted migration SQL; do not claim real Neon/Vercel, K3s, or Multipass validation.

## Coordinator update (2026-10-29)

The initial harness assignment said not to commit or merge from the worker lane. The user later asked the coordinator to complete validated work and merge it to local dev; the coordinator owns that integration decision. The source review corrected the Directory lock ACL to SELECT plus UPDATE(generation), with tenant/actor-scoped restrictive USING and WITH CHECK (false); the runner checks cross-actor invisibility, FOR SHARE, and direct-update rejection with SQLSTATE 42501. It seeds the historical weak refresh CHECK before applying the current W16 migration. No main modification or push is authorized.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
