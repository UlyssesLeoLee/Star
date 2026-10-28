# PHASE-W16-B-KMS-CONTRACT-IMPLEMENTATION-20261028

> Status: Provider-neutral KMS contract implemented and package-scoped checks passed; W16-B remains open for trusted digest resolution, provider approval/operations, and W16-C persistence evidence.
> Date: 2026-10-28
> Base: `dev@05be8c61e85fe19b97a0ff11acdf972d5026492f`
> Lane: `codex/w16-b-kms-contract-20261028`
> Worktree: `C:\Users\leo19\.codex\worktrees\w16-b-kms-contract-20261028\Star`

## 0. Purpose

Add an additive Rust contract for data-key generation, wrapping, unwrapping, rewrapping, capability discovery, and immutable profile identity. Existing `KmsClient` callers and `LocalMockKms` remain unchanged. This lane adds no provider implementation, chooses no production provider, and does not claim durable key management.

## 1. Requirement and file matrix

| Requirement | Implementation | Status |
|---|---|---|
| Keep existing KMS callers compatible | Added modules and re-exports in `crates/domain-kms/src/lib.rs`; existing `KmsClient` and `LocalMockKms` implementation body unchanged. | `star-credential` and `platform-messaging-runtime` library checks passed. |
| Pin provider/profile identity | `profile.rs`: validated provider id/version, positive revision, opaque endpoint/key references, capability snapshot, and fixed 32-byte resolver-supplied digest. Serialization validates deserialized identities; debug output redacts endpoint/key references. | Unit tests cover validation, digest sentinel, round-trip, and redaction. |
| Define provider-neutral operations | `provider_contract.rs`: adapter trait and validating facade for generate, wrap, unwrap, rewrap, and capability discovery; typed provider-neutral error classes. | Unit tests cover capability denial, provider/profile mismatch, response limits, and rejection of cross-provider rewrap. |
| Keep persisted wrapped metadata safe and recoverable | `WrappedDataKey` stores exact profile identity, bounded opaque wrapped bytes, and provider key version; plaintext keys are non-serializable and redacted in `Debug`. | Focused tests passed. Memory clearing is best-effort and does not claim an audited zeroization guarantee. |
| Keep provider selection and durable claims open | No Cargo dependency, manifest, production provider, provider configuration, or fallback was added. | Provider approval, artifact license/SBOM closure, operational readiness, and persistence/recovery remain open. |
| Keep source graph current | Added evidence-backed Cypher relationship headers to the three changed Rust source files. | CypherGraph Guardian passed for 3 source files. |

## 2. Verification summary

- Toolchain: `rustc 1.99.0 (b940084d7 2026-09-28)`.
- `rtk proxy cargo fmt --manifest-path crates/domain-kms/Cargo.toml`: passed.
- `rtk proxy cargo fmt --manifest-path crates/domain-kms/Cargo.toml -- --check`: passed.
- `rtk proxy cargo check -p domain-kms --lib -j 4 --offline --locked`: passed.
- `rtk proxy cargo test -p domain-kms --lib -j 4 --offline --locked`: 12 passed, 0 failed. The shared Cargo target produced a linker `LNK1104` on one attempt; rerunning with a lane-specific `CARGO_TARGET_DIR=E:/DevCache/cargo/target-w16-b-kms` passed. The transient cause was not established.
- `rtk proxy cargo check -p star-credential -p platform-messaging-runtime --lib -j 4 --offline --locked`: passed, confirming existing callers still compile.
- `rtk proxy py -3 "C:\Users\leo19\.codex\skills\CypherGraph-Guardian\skill\cypher-graph-guardian\scripts\cgg_guard.py" --changed --stamp`: passed (3 source files checked).
- The focused Cargo commands emit three existing `aes_gcm::Nonce::from_slice` deprecation warnings from the pre-existing `LocalMockKms` code in `lib.rs`; this lane did not modify those calls.
- `rtk proxy git diff --cached --check`: passed for all six registered paths.

## 3. Known gaps

1. Canonical profile serialization and digest generation are not implemented. `ProfileDigest` must be supplied by a trusted resolver; the crate only enforces a fixed-size nonzero value.
2. No production provider is selected, implemented, or enabled. Provider-specific security review, release dependency/image/chart license closure, SPDX/SBOM, and operational recovery proof remain open.
3. The provider adapter receives a finite timeout in `KmsCallContext`; adapters are required to enforce it and cancel provider work. This contract-only slice has no production adapter with which to verify timeout enforcement.
4. `SecretDataKey` uses safe-code best-effort clearing because this manifest does not include a direct audited zeroization dependency. No memory-zeroization guarantee is claimed.
5. `LocalMockKms` remains process-local and non-durable and is not a production fallback.
6. W16-C must supply persistent envelope metadata, restart/recovery, rotation, and restore evidence before payload encryption is enabled.
7. This lane did not modify canonical SRS/BD/DD/TD, manifests, or the shared lockfile. No workspace-wide check, clippy, provider integration, or database test was run.

## 4. Subagent results and takeover

- This worker did not dispatch child agents. W16-A and W16-C are independently coordinated lanes; their outputs were not imported or changed here.
- The first shared-target test link attempt failed with Windows linker `LNK1104` opening the shared `domain_kms` test executable. The focused test was rerun with a lane-specific target directory and passed all 12 tests; no process cleanup or build-artifact deletion was performed.
- Coordinator review and cross-lane integration remain outside this implementation lane.

## 5. Guard rules

1. Business callers should enter through `KmsProviderClient`; the raw `KmsProvider` trait is the adapter boundary.
2. Provider/version identity and exact profile identity are checked before unwrap; generated, wrapped, and rewrapped results must retain the expected profile identity.
3. The effective capability set is the trusted profile snapshot. Provider-reported capabilities cannot grant operations omitted by that snapshot.
4. Rewrap is limited to the same provider id/version and endpoint reference. Cross-provider migration needs a separately authorized migration flow.
5. Endpoint/key references use a restricted opaque identifier grammar; do not place credentials, bearer tokens, URL query strings, plaintext DEKs, or application payloads in profile/log metadata.
6. Wrapped bytes and provider responses are bounded by the per-operation limit and the contract maximum.
7. Provider errors contain only safe classification; provider response text and secret material must not be propagated into error metadata.
8. Profile digests remain resolver-supplied; never label the current type as canonical serialization or digest production.
9. `LocalMockKms` is not a production fallback or persistence proof.
10. Preserve Rust 1.99.0, avoid dependency/manifest edits in this lane, and keep W16-B open until its provider and W16-C evidence gates close.

## 6. Review roles

| Role | Review |
|---|---|
| Architect | Coordinator review pending. |
| SRE Lead | Coordinator review pending; production timeout/cancellation and provider recovery remain unverified. |
| Platform | Coordinator review pending; Rust 1.99.0 and focused caller compatibility were checked. |
| Review host | Coordinator review pending; focused tests and graph guard passed. |
| PM | Coordinator review pending; phase remains open with provider and persistence gates outstanding. |

## 7. Revision history

| Version | Date | Editor | Change | Trigger |
|---|---|---|---|---|
| v0.1 | 2026-10-28 | W16-B implementation worker | Record additive provider-neutral KMS operations, pinned profile identity, bounded metadata, focused Rust evidence, and provider/digest/persistence open items. | W16-B dispatcher brief. |
