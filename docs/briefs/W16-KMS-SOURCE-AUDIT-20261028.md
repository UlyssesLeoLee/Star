# Brief: W16-KMS-SOURCE-AUDIT-20261028

**Agent**: explorer
**Phase**: W16-KMS-SOURCE-AUDIT-20261028
**Created**: 2026-10-28 04:12:55

---

## Assignment

Perform a read-only audit of Star's KMS/payload-protection boundary and candidate local Rust server modules, including E:\AssetsLake. Write only `docs/reviews/W16-KMS-SOURCE-AUDIT-20261028.md` plus this dispatcher-created brief and its dispatcher audit log. Do not edit, stage, or commit any source or other documentation. Do not copy or alter any local Rust project.

## Product and source-preservation constraints

- Star's business authorization, owner-local persistence, retention/RLS, resource binding and durable acceptance remain Star-owned. Any external component is a replaceable adapter.
- No plaintext durable draft/conflict payload, process-only key, `LocalMockKms`, Noop, memory queue, mock or demo adapter may be described as production.
- For every candidate, record repo HEAD, branch, full dirty status summary, exact relevant working-file paths and SHA-256, license/ownership evidence, current implementation state, and dependency closure evidence or gaps. If a source worktree is dirty, do not edit it; hash the actual working files.
- Do not read environment variables, credentials, tokens, .env files, or private keys. The D:\Star main worktree is user-dirty; use this assigned clean worktree for Star and inspect other repositories read-only.
- Commercial use is not automatically disallowed by copyleft licensing. Evaluate obligations for modification/linking/deployment/distribution and flag unresolved rights; do not equate AGPL/GPL with a commercial-use ban.

## Scope

1. Map `KmsClient`, `EncryptedBlob`, `LocalMockKms`, `TranscriptBodyProtector`, and the production composition/wiring path. Determine whether any concrete protector or persistent KMS provider exists in the bounded `crates/` scope.
2. Assess the fit and gaps of the current KmsClient contract for WLD-7: resource-context AAD, tenant separation, wrapped DEK persistence, key version/rotation, restart decrypt, backup restore, cancellation/zeroization, retention and error/fail-closed behavior.
3. Read-only survey E:\AssetsLake and other discoverable local Rust backend/service repositories. For each relevant module, record HEAD/branch/dirty state, working-file hashes, declared ownership/license evidence, code purpose, deployment state and dependency/lockfile/license closure. Do not claim exhaustive inventory unless the searched roots and exclusions prove it.
4. Decide whether any local module is suitable to reuse, adapt, or use only as architectural reference, and specify the exact conditions before reuse. Do not implement or copy anything.
5. Recommend Star-side provider seams and acceptance tests without selecting an external vendor; another lane handles current external provider research.

## Parent evidence

Current dev is 3b873d3e. Parent CodeGraph found `KmsClient` at `crates/domain-kms/src/lib.rs:116`, implemented by `LocalMockKms`; `LocalMockKms` creates a random master key on startup, does not persist it, and stores DEKs in process memory. Parent bounded source search found `TranscriptBodyProtector` as a trait in `crates/star-api-rest/src/group_api/scoped_chat_store.rs`, with no concrete implementation found in `crates/`; this is a scoped search, not proof about repositories outside it. E:\AssetsLake was read-only inspected earlier: HEAD `0156aca56c0d7364120d7389ebe1ccf0cb64b750`, main ahead 2 / behind 6 versus origin/main, dirty worktree; working hashes were recorded in the current WBS §23.8. Its root LICENSE declares AGPL-3.0; its S3 storage service uses AWS S3 APIs for object CRUD, which does not prove a KMS/envelope-encryption provider. Revalidate all facts now; do not rely on these notes as current without checking.

## Deliverable format

Write: (1) decision summary, (2) source repository/module evidence table, (3) KMS/protector contract gap matrix, (4) reuse recommendation by module, (5) dependency/license/ownership limitations, (6) implementation and verification order. Include source links and hash evidence. No tests/builds are required for this read-only audit; do not run them.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
