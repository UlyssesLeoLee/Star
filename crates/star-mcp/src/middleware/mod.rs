//! # star-mcp middleware 模块
//!
//! per `docs/architecture/2026-09-07-exclusion-idempotency/03-detailed-design.md` §1.1 M-15
//! per `PHASE-EXCLUSION-IDEMPOTENCY-IMPL-REPORT.md` §1.1 EX-06
//!
//! 守门合规:
//! - 守门 #13 a: middleware 在 L0 派发层, L1 tool 不直接调
//! - 守门 #10: author=Ulysses

pub(crate) mod idempotency;

pub(crate) use idempotency::{IdempotencyKey, IdempotencyMiddleware, IdempotencyResult};
