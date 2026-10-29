# Brief: W16-A-DEVICE-AUTH-IMPLEMENTATION-20261028

**Agent**: worker
**Phase**: W16-A-DEVICE-AUTH
**Created**: 2026-10-28 18:00:11

---

Security-review follow-up, still within the registered W16-A lane. Complete least-privilege PostgreSQL lifecycle access for device enrollment and revocation, not only active-device reads. Current verified gap: PgDeviceBindingRepository::complete_challenge and ::revoke write oauth_device_enrollment_challenges, oauth_device_bindings, oauth_device_binding_audit, and oauth_refresh_tokens without SET LOCAL ROLE; W16 grants only a read/lock path to star_engineering_directory_owner, so those operations are not proven to work for the NOINHERIT runtime login. Use the existing OAuth-owned star_oauth_credential_owner role for OAuth/device lifecycle if its schema/RLS contract permits; explicitly SET LOCAL ROLE in every repository transaction that reads/writes these OAuth-owned tables. Add only required table/column grants and tenant+actor RLS checks; keep audit append-only. Grant runtime membership only in the isolated acceptance fixture, never direct table ACLs to the login. Keep star_engineering_directory_owner read-only on oauth_device_bindings for Run/Worktree generation locking; remove any unnecessary UPDATE privilege there while retaining defense-in-depth policy if appropriate. Align route/PG adapter probes and the separate harness role assertions; coordinate exact role and privilege names with the harness owner. Update Cypher headers, run focused Rust checks/tests, local PG migration/role probes if feasible, rustfmt, cgg_guard, and diff check. Modify only already registered W16-A paths, including device_bindings.rs, migration, existing PG/route tests, phase report, brief and WBS. Do not commit or merge; report every real validation and any unrun database test exactly.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
