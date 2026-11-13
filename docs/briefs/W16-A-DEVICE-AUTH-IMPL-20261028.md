# Brief: W16-A-DEVICE-AUTH-IMPL-20261028

**Agent**: worker
**Phase**: W16-A-DEVICE-AUTH
**Created**: 2026-10-28 05:41:31

---

# W16-A: Device identity and atomic OAuth consumption

Task:
Implement the W16-A identity/security foundation on this isolated lane. Start by reading the repository AGENTS.md, W16 SRS/BD/DD companion documents, and W16 test-design additions. Use the repository code graph before source discovery when available; supplied findings below are provisional and exact source/code fallback is required for uncovered areas.

Owner:
Assigned W16-A worker. Only this worktree is writable.

Branch / Worktree:
- Branch: codex/w16-a-device-auth-20261028
- Worktree: C:\Users\leo19\.codex\worktrees\w16-a-device-auth-20261028\Star
- Base: dev@05be8c61e85fe19b97a0ff11acdf972d5026492f
- Dependencies: W16 design and test-design commits already in dev; no dependency on W16-B code.

Graph/source findings from coordinator:
- CodeGraph returned current source for Claims/AuthUser in crates/star-api-rest/src/auth/mod.rs and bearer extraction in crates/star-api-rest/src/auth/oauth/middleware.rs; Claims and AuthUser currently do not carry device identity.
- CodeGraph returned OAuth repository source in crates/star-pg-adapter/src/repository/oauth_authorization_codes.rs and oauth_refresh_tokens.rs. Current code reads a credential then separately updates it; auth-code mark_consumed ignores rows_affected, so concurrent consumers can both proceed.
- CodeGraph returned KmsClient source in crates/domain-kms/src/lib.rs; unrelated to this lane.
- Exact-source fallback found Device and DeviceBinding models in crates/domain-identity/src/lib.rs, but no Device PostgreSQL repository or enrollment route was found in the bounded source search.
- codebase-memory-mcp coverage tools were unavailable in this session; graph output is provisional and not proof of exhaustive call sites.

Scope:
1. Implement a durable device binding/enrollment foundation that obtains actor/tenant from authenticated identity, binds an opaque server-issued device id and generation to a verifiable device public credential, and never trusts client-supplied actor/tenant/device identity.
2. Add a signed-token device context without unnecessarily breaking existing AuthUser callers. Device identity must be extracted only from verified issuer claims. Legacy sessions without device context must fail closed for shared collaboration writes.
3. Make OAuth authorization-code and refresh-token redemption an atomic single-consumer operation. Concurrent replay must produce at most one winner. A zero-row conditional update is a failed redemption; never ignore it. Preserve client, tenant, actor, scope, and device binding through refresh rotation.
4. Revoke a device by advancing its generation/status and revoking its refresh family in an owner transaction where supported. Existing authenticated collaboration SSE sessions must be invalidated or the affected collaborative operation must fail closed pending the durable invalidation path.
5. Add RLS/audit-aware DDL and focused tests. Database tests must use a real PostgreSQL endpoint; do not claim mocks, SQLite, or unit tests prove PostgreSQL concurrency/RLS behavior.

Acceptance:
- Tests prove exactly one winner for concurrent code and refresh redemption, replay rejection, cross-tenant denial, generation mismatch/revocation denial, and no success receipt for zero-row consumption.
- New endpoints/repositories use server-authenticated identity and safe public errors; secrets, raw proof material, raw tokens, and private keys never enter logs/audit.
- No shared write or subscription is enabled when device generation cannot be checked. Report any unfinished part as OPEN/NOT_RUN; do not claim W16-A complete without required PostgreSQL and revocation/SSE evidence.
- Preserve current OAuth behavior outside this scope and add focused regression coverage.

Files:
Only modify files in the registered manifest. Do not modify Cargo.lock, canonical SRS/BD/DD/test-design, other branches/worktrees, D:\Star user files, or main. If implementation requires an unlisted file, stop and report the exact paths to the coordinator for a MAGOS manifest amendment before writing there.

Validation and delivery:
- Rust toolchain baseline is 1.99.0. Run commands through RTK (rtk proxy cargo ...) and keep Cargo concurrency at -j 4.
- Run formatting/checks and focused tests for changed crates. Run PostgreSQL tests only when a configured real endpoint is available; never print or inspect secret env values.
- Run git diff --check, review the full diff, verify CypherGraph Guardian headers for changed Rust relationships, and run the repository CGG check if available.
- Write PHASE-W16-A-DEVICE-AUTH-IMPLEMENTATION-20261028.md with a file/requirement matrix, exact validation commands/results, known gaps, and status. Commit only in this lane. Commit message must cite docs/briefs/W16-A-DEVICE-AUTH-IMPLEMENTATION-20261028.md. Never rebase, merge, push, or edit dev/main.
- Return commit SHA, changed files, commands/results, unresolved gaps, and any reason the registered manifest needs amendment.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
