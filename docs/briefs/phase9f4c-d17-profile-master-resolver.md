# Brief: phase9f4c-d17-profile-master-resolver

**Agent**: implementation
**Phase**: phase9f4c-d17-profile
**Created**: 2026-10-12 04:16:57

---

## Objective
Implement a bounded first D17 slice for the Platform Messaging Transport Profile Master/resolver in the current Star worktree.

## Scope
- Preserve the approved boundary: deployment+tenant profile identity; immutable SCD2 revision payload; append-only audit and versioned transactional Outbox metadata; opaque secret reference only; runtime resolver reads through trusted deployment context and verifies provider/version/schema/digest/endpoint/service binding and sender/recipient/kind scope.
- Reuse existing Star Rust/PostgreSQL patterns for owner-scoped transactions, FORCE RLS, append-only facts, deterministic typed profiles, and existing first-claim profile pinning. Do not merge the Infrastructure provider profile with Transport Profile.
- Recheck `E:/AssetsLake/backend/src/services/event_publisher_service.rs` and local Rust candidate sources read-only. Adapt only relevant typed event metadata semantics (schema/trace/causation/idempotency); do not reuse its dry-run relay. Do not copy connector code into D17 without a real transport call site. Keep source repositories unchanged and preserve licensing/NOTICE obligations for any future copied module.
- Do not add a native durable provider, turn on Schedule/Run execution, claim production identity verification, remove NATS, modify Multipass inventory, or touch user-owned root `D:/Star` changes.

## Verification
- Add focused tests for canonical digest and profile/scope rejection.
- Add a disposable PostgreSQL schema/migration gate for idempotent migration replay, append-only facts, dual deployment+tenant RLS, runtime read-only access and owner revision/CAS behavior where available.
- Run formatter and crate-level checks with the installed Rust 1.98.1 toolchain only as diagnostics (`--ignore-rust-version`); Rust 1.99.0 remains an explicit open gate.
- Update SRS, basic design, detailed design, implementation plan, and a D17 stage report to distinguish the implemented slice from remaining production identity, BI, native transport, memory/performance, and execution gates.


## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
