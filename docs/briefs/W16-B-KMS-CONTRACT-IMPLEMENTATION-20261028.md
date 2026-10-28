# Brief: W16-B-KMS-CONTRACT-IMPLEMENTATION-20261028

**Agent**: worker
**Phase**: W16-B-KMS-CONTRACT
**Created**: 2026-10-28 05:42:48

---

# W16-B: Provider-neutral KMS contract and profile identity

Task:
Implement the W16-B Rust contract foundation on this isolated lane. Read repository AGENTS.md and W16 SRS/BD/DD companion documents first. Use the repository code graph before source discovery when available; inspect the existing KMS callers and keep compatibility unless a reviewed migration is explicitly needed.

Owner:
Assigned W16-B worker. Only this worktree is writable.

Branch / Worktree:
- Branch: codex/w16-b-kms-contract-20261028
- Worktree: C:\Users\leo19\.codex\worktrees\w16-b-kms-contract-20261028\Star
- Base: dev@05be8c61e85fe19b97a0ff11acdf972d5026492f
- Dependencies: W16 design and test-design commits already in dev; no dependency on W16-A code.

Graph/source findings from coordinator:
- CodeGraph returned crates/domain-kms/src/lib.rs. KmsClient is currently generate_dek/encrypt/decrypt/rotate_dek/health, and the only implementation found is LocalMockKms.
- CodeGraph found KmsClient call sites in platform-messaging-runtime and star-credential. Do not break them in this additive contract phase.
- TranscriptBodyProtector exists only as a trait in crates/star-api-rest/src/group_api/scoped_chat_store.rs; implementation is a later W16-C concern.
- LocalMockKms has process-local random master material and is unsuitable for persistent collaborative payloads.
- Provider research recommends OpenBao Transit as a self-hosted candidate for evaluation, but no provider is approved. Its full release dependency/image/chart license closure and operational proof remain open. Do not add or deploy a provider in this lane.
- codebase-memory-mcp coverage tools were unavailable in this session; graph output is provisional and not proof of exhaustive call sites.

Scope:
1. Add an additive, provider-neutral async interface covering generate-data-key, wrap, unwrap, rewrap, and capability discovery. Model typed results/errors and bounded operation metadata without tying business callers to Vault/OpenBao/AWS/NATS-specific APIs.
2. Define immutable KmsProfile identity: provider id/version, profile revision, endpoint/key reference, capability set, and a fixed 32-byte profile digest. The digest must be supplied only by a trusted profile-resolution boundary in this phase; do not pretend this crate computes a canonical digest unless a cryptographic hashing dependency is already directly available. Mark canonical serialization/digest production as an open W16-B item if not implementable without a manifest amendment.
3. Ensure wrapped-key metadata carries provider/profile/key-version identity needed for later recovery/rewrap and is safe to persist. Never include credentials, bearer tokens, plaintext DEKs, or payload data in serializable profile or log/health structures.
4. Keep the existing KmsClient and LocalMockKms behavior only for existing dev/test callers; never label it durable or allow it as an automatic production fallback. Do not implement a mock provider as proof of production availability.
5. Add focused tests for profile validation, capability checks, identity pinning, and provider-neutral error mapping.

Acceptance:
- The new contract does not depend on provider-specific request/response types or transport mechanisms.
- Unknown provider/profile/capability mismatch fails closed. Profile identity does not silently change during a logical envelope operation.
- Existing KmsClient callers compile unchanged.
- No new third-party dependencies, Cargo.toml changes, Cargo.lock changes, production provider implementation, or canonical document edits in this lane. If a required dependency or file is discovered, stop and request a manifest amendment before touching it.
- W16-B remains OPEN until trusted canonical profile digest generation, full distributable dependency/image/chart SPDX/SBOM and NOTICE closure, provider operational security, and W16-C persistent restart/rotation/restore evidence are complete.

Files:
Only modify files in the registered manifest. Do not edit Cargo manifests/lockfile, canonical SRS/BD/DD/test-design, other branches/worktrees, D:\Star user files, or main. If implementation requires an unlisted path, stop and give the exact path to the coordinator for a MAGOS manifest amendment.

Validation and delivery:
- Rust toolchain baseline is 1.99.0. Run commands through RTK (rtk proxy cargo ...) and keep Cargo concurrency at -j 4.
- Run formatting and focused domain-kms tests; run git diff --check.
- Verify CypherGraph Guardian headers for changed Rust relationships and run the repository CGG check if available.
- Write PHASE-W16-B-KMS-CONTRACT-IMPLEMENTATION-20261028.md with requirement/file matrix, exact validation results, and open items. Commit only in this lane. Commit message must cite docs/briefs/W16-B-KMS-CONTRACT-IMPLEMENTATION-20261028.md. Never rebase, merge, push, or edit dev/main.
- Return commit SHA, changed files, commands/results, unresolved gaps, and any reason the registered manifest needs amendment.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
