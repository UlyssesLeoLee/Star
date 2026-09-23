//! # aci-emitter
//!
//! 跨项目 ACI (Assertion Contract Interface) emitter helper crate (per ULYS-191 §4.2.1).
//!
//! 对标 Python 实现: [`tools/star-flash-mock/scripts/_lib_aci_emit.py`](https://github.com/UlyssesLeoLee/Star/tree/main/tools/star-flash-mock/scripts/_lib_aci_emit.py)
//! (per 决策 #5: 各自拷贝 emitter, 跨项目共享 schema).
//!
//! ## 快速上手
//!
//! ```no_run
//! use aci_emitter::{AciEmitter, ExpectActual, ExpectValueType, Layer, Severity, Status};
//! use aci_emitter::types::empty_scope;
//!
//! let em = AciEmitter::new(Layer::It);
//! let mut scope = empty_scope();
//! scope.insert("module".to_string(), "guards".to_string());
//!
//! let assertion = em.build(
//!     "star-flash-mock:guards:g-1",
//!     scope,
//!     ExpectActual::new(ExpectValueType::ResponseWithinMs, 2000, "API 应在 2s 内响应"),
//!     ExpectActual::new(ExpectValueType::ResponseWithinMs, 5, "实测 5ms"),
//!     Status::Pass,
//!     Severity::Info,
//!     "实测延迟远低于预期, 通过",
//! ).unwrap();
//!
//! let json = em.to_json(&assertion);
//! println!("{}", json);
//! ```
//!
//! ## 跨项目集成
//!
//! 在其他 Rust 项目 `Cargo.toml` 加 (per 决策 #5, path = "../aci-emitter"):
//!
//! ```toml
//! [dependencies]
//! aci-emitter = { path = "../aci-emitter/crates/aci-emitter" }
//! ```
//!
//! ## 守门对齐 (per AGENTS.md §4)
//!
//! | 守门 | 对齐 |
//! |------|------|
//! | #5 env 安全 | ✅ 纯 schema + emitter, 0 env |
//! | #7 0 unsafe | ✅ rustc 默认 (无 unsafe block) |
//! | #24 vendor 中立 | ✅ 仅 serde + serde_json + chrono + thiserror + uuid |

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod builder;
pub mod error;
pub mod types;
pub mod util;
pub mod validators;

pub use builder::AciEmitter;
pub use error::{AciEmitError, AciResult, AciValidationError};
pub use types::{
    empty_scope, Assertion, ExpectActual, ExpectValueType, Layer, ScopeMap, Severity, Status,
    ACI_VERSION,
};
