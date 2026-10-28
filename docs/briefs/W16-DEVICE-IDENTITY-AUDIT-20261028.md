# Brief: W16-DEVICE-IDENTITY-AUDIT-20261028

**Agent**: explorer
**Phase**: W16-DEVICE-IDENTITY-AUDIT-20261028
**Created**: 2026-10-28 04:12:55

---

## Assignment

Perform a read-only, bounded source audit of how Star can obtain an authenticated device identity for WLD-7 collaboration. Write only `docs/reviews/W16-DEVICE-IDENTITY-AUDIT-20261028.md` plus this dispatcher-created brief and its dispatcher audit log. Do not edit source code, tests, schemas, or any other document. Do not commit, rebase, merge, push, or alter another worktree.

## Product and security constraints

- Owner-local draft/conflict records require actor_id and device_id from trusted authentication context. A client-supplied body/header value must never define participant identity, recipients, RLS scope, or authority.
- OAuth/JWT claims, refresh rotation, device registration/revocation, and API request context must form a traceable identity chain. Do not accept an unregistered device, silently invent a device id, or fall back to a mock identity. Keep collaboration fail closed / local_only / sync_pending until the chain is authenticated and revocation semantics are defined.
- Keep service/client-credentials, task CLI and non-device actors distinct from user-device identities. Do not broaden privileges or make the claim a bearer authorization shortcut.
- Do not read environment variables, credentials, tokens, or secret files. The primary D:\Star main worktree is user-dirty; use only this assigned clean worktree.

## Scope

1. Trace OAuth authorization-code + PKCE, refresh-token rotation, client-credentials, `issue_tokens`, JWT `Claims`, `AuthUser`, and request extractors.
2. Trace the existing `domain-identity::Device` model, register/revoke commands, persistence ports/adapters, routes, migrations, and actor/tenant checks. Determine which portions are production-backed versus in-memory/reference-only.
3. Recommend the smallest sound end-to-end identity contract: enrollment/authentication source, token/device binding, refresh/revocation behavior, multi-device and service-client behavior, failure modes, and owner-local API checks.
4. Identify exact implementation/test/design files likely needed, dependency ordering, and acceptance cases. Do not implement.
5. Record exact paths and line numbers, and distinguish verified facts from inference and unknowns. Include the baseline commit and relevant source hashes where practical.

## Parent CodeGraph evidence

The parent queried CodeGraph on the current dev checkout at 3b873d3e. It found AuthUser in `crates/star-api-rest/src/auth/mod.rs:181` with only user_id, tenant_id, roles and scope; Claims has sub/iss/aud/exp/iat/nbf/jti/tenant/user/roles/scope and no device field. OAuth authorization-code handler calls issue_tokens with user_id and tenant_id but no device; refresh reconstructs from stored refresh-token data; client_credentials has no user. CodeGraph found Device and register/revoke commands in `crates/domain-identity/src/lib.rs`. No check_index_coverage tool is exposed in the parent session. Treat graph evidence as scoped; verify your relevant source paths directly and do not make repository-wide negative claims without bounded evidence.

## Deliverable format

Write a concise review with: (1) decision summary, (2) traced current flow with file/line evidence, (3) production-vs-reference boundary, (4) recommended identity contract and threat/failure cases, (5) implementation file and test matrix, (6) unresolved questions and evidence limitations. This is an audit, not evidence that any implementation or test has passed.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
