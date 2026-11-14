//! ═══════════════════════════════════════════════════════════════════
//! Ported from UlyssesLeoLee/SandTree v1.1 (Apache-2.0)
//! per REUSE-MAP-COLLAB-SANDBOX-001 §3 (直接复用, 保留 NOTICE).
//! Source: E:\SandTree\crates\{model,observation-model,resource-graph}
//! ═══════════════════════════════════════════════════════════════════

//! Stable error codes (DD-DATA §8; `schemas/error_codes.csv`, `schemas/observation_error_codes.csv`).
//!
//! The kernel and every UI surface branch on these codes. Provider raw messages
//! are carried in `detail` only and must never be string-matched by UI logic
//! (DD-SW §10, DD-DATA §8).
//!
//! # The registry is frozen
//!
//! `schemas/error_codes.csv` carries one code per failure **category**, and
//! the design baseline pins its contents. A new feature that needs a new
//! failure *mode* does not get a new code — it gets a typed
//! [`LockFailure`] reason next to the code it already has (ADR-061).

use crate::model::resource::LockFailure;

/// A stable, namespaced error code.
///
/// The inner value is always one of the codes in the shipped CSV registry;
/// deserialization rejects unknown codes so a typo in a plugin payload cannot
/// masquerade as a valid SandTree error (DD-DATA §8).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ErrorCode(&'static str);

impl serde::Serialize for ErrorCode {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.0)
    }
}

impl<'de> serde::Deserialize<'de> for ErrorCode {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let raw = <String as serde::Deserialize>::deserialize(d)?;
        match ErrorCode::parse(&raw) {
            Some(code) => Ok(code),
            None => Err(serde::de::Error::custom(format!(
                "unknown SandTree error code {raw:?}"
            ))),
        }
    }
}

impl ErrorCode {
    /// Construct from a static code string; the string must exist in the CSV registry.
    pub const fn new(code: &'static str) -> Self {
        Self(code)
    }

    /// Parse an arbitrary code string against the shipped registry.
    pub fn parse(raw: &str) -> Option<Self> {
        CORE_CODES
            .iter()
            .chain(OBSERVATION_CODES.iter())
            .find(|c| c.0 == raw)
            .copied()
    }

    /// Code text, e.g. `ST-VFS-001`.
    pub fn as_str(self) -> &'static str {
        self.0
    }

    /// Category prefix, e.g. `VFS`.
    /// Category name, using the vocabulary of `schemas/error_codes.csv`.
    ///
    /// The code prefix is an abbreviation (`ST-POL-001`) while the registry
    /// spells the category out (`POLICY`), so the two are mapped rather than
    /// derived. Deriving is what this used to do, and it made every category
    /// name in the API disagree with the published registry.
    pub fn category(self) -> &'static str {
        match self.0.split('-').nth(1).unwrap_or("CORE") {
            "POL" => "POLICY",
            "PLG" => "PLUGIN",
            "DKR" => "DOCKER",
            "SBX" => "SANDBOX",
            "DB" => "STORE",
            other => other,
        }
    }

    /// Whether retrying the same request could succeed.
    pub fn is_retryable(self) -> bool {
        retryable(self.0)
    }

    /// All core codes from `schemas/error_codes.csv`.
    pub fn all() -> &'static [ErrorCode] {
        CORE_CODES
    }

    /// All observation codes from `schemas/observation_error_codes.csv`.
    pub fn all_observation() -> &'static [ErrorCode] {
        OBSERVATION_CODES
    }

    // --- CORE ---
    /// Invalid request or broken invariant.
    pub const CORE_INVALID: ErrorCode = ErrorCode::new("ST-CORE-001");
    /// Capability denied by policy.
    pub const POLICY_DENIED: ErrorCode = ErrorCode::new("ST-POL-001");
    /// Plugin manifest invalid.
    pub const PLUGIN_MANIFEST_INVALID: ErrorCode = ErrorCode::new("ST-PLG-001");
    /// Plugin health probe failed.
    pub const PLUGIN_HEALTH_FAILED: ErrorCode = ErrorCode::new("ST-PLG-002");
    /// Hot swap migration rejected.
    pub const PLUGIN_HOTSWAP_REJECTED: ErrorCode = ErrorCode::new("ST-PLG-003");
    /// Docker endpoint unavailable.
    pub const DOCKER_ENDPOINT_UNAVAILABLE: ErrorCode = ErrorCode::new("ST-DKR-001");
    /// Docker API version incompatible.
    pub const DOCKER_API_INCOMPATIBLE: ErrorCode = ErrorCode::new("ST-DKR-002");
    /// Docker resource conflict / in use.
    pub const DOCKER_CONFLICT: ErrorCode = ErrorCode::new("ST-DKR-003");
    /// Sandbox provider unavailable.
    pub const SANDBOX_PROVIDER_UNAVAILABLE: ErrorCode = ErrorCode::new("ST-SBX-001");
    /// Operation unsupported by provider.
    pub const SANDBOX_UNSUPPORTED: ErrorCode = ErrorCode::new("ST-SBX-002");
    /// VFS path escapes workspace root.
    pub const VFS_PATH_ESCAPE: ErrorCode = ErrorCode::new("ST-VFS-001");
    /// VFS resource/file not found.
    pub const VFS_NOT_FOUND: ErrorCode = ErrorCode::new("ST-VFS-002");
    /// Store transaction failed.
    pub const STORE_TRANSACTION_FAILED: ErrorCode = ErrorCode::new("ST-DB-001");
    /// IPC protocol/frame invalid.
    pub const IPC_INVALID_FRAME: ErrorCode = ErrorCode::new("ST-IPC-001");

    // --- OBS ---
    /// No observation strategy available.
    pub const OBS_NO_STRATEGY: ErrorCode = ErrorCode::new("ST-OBS-001");
    /// Observation deadline exceeded.
    pub const OBS_DEADLINE_EXCEEDED: ErrorCode = ErrorCode::new("ST-OBS-002");
    /// Snapshot stale.
    pub const OBS_STALE: ErrorCode = ErrorCode::new("ST-OBS-003");
    /// Probe bootstrap failed.
    pub const OBS_PROBE_BOOTSTRAP: ErrorCode = ErrorCode::new("ST-OBS-004");
    /// Probe envelope invalid.
    pub const OBS_ENVELOPE_INVALID: ErrorCode = ErrorCode::new("ST-OBS-005");
    /// Provider native credential denied.
    pub const OBS_CREDENTIAL_DENIED: ErrorCode = ErrorCode::new("ST-OBS-006");
    /// Collector output too large.
    pub const OBS_OUTPUT_TOO_LARGE: ErrorCode = ErrorCode::new("ST-OBS-007");
    /// Guest path escaped configured root.
    pub const OBS_GUEST_PATH_ESCAPE: ErrorCode = ErrorCode::new("ST-OBS-008");
    /// Source trust below policy threshold.
    pub const OBS_TRUST_BELOW_THRESHOLD: ErrorCode = ErrorCode::new("ST-OBS-009");
    /// Nested Docker endpoint unavailable.
    pub const OBS_NESTED_DOCKER_UNAVAILABLE: ErrorCode = ErrorCode::new("ST-OBS-010");
}

const CORE_CODES: &[ErrorCode] = &[
    ErrorCode::new("ST-CORE-001"),
    ErrorCode::new("ST-POL-001"),
    ErrorCode::new("ST-PLG-001"),
    ErrorCode::new("ST-PLG-002"),
    ErrorCode::new("ST-PLG-003"),
    ErrorCode::new("ST-DKR-001"),
    ErrorCode::new("ST-DKR-002"),
    ErrorCode::new("ST-DKR-003"),
    ErrorCode::new("ST-SBX-001"),
    ErrorCode::new("ST-SBX-002"),
    ErrorCode::new("ST-VFS-001"),
    ErrorCode::new("ST-VFS-002"),
    ErrorCode::new("ST-DB-001"),
    ErrorCode::new("ST-IPC-001"),
];

const OBSERVATION_CODES: &[ErrorCode] = &[
    ErrorCode::new("ST-OBS-001"),
    ErrorCode::new("ST-OBS-002"),
    ErrorCode::new("ST-OBS-003"),
    ErrorCode::new("ST-OBS-004"),
    ErrorCode::new("ST-OBS-005"),
    ErrorCode::new("ST-OBS-006"),
    ErrorCode::new("ST-OBS-007"),
    ErrorCode::new("ST-OBS-008"),
    ErrorCode::new("ST-OBS-009"),
    ErrorCode::new("ST-OBS-010"),
];

fn retryable(code: &str) -> bool {
    matches!(
        code,
        "ST-PLG-002"
            | "ST-DKR-001"
            | "ST-SBX-001"
            | "ST-DB-001"
            | "ST-OBS-002"
            | "ST-OBS-003"
            | "ST-OBS-004"
            | "ST-OBS-010"
    )
}

/// The kernel-level error type surfaced over IPC.
#[derive(Debug)]
pub struct DomainError {
    /// Stable code from the registry.
    pub code: ErrorCode,
    /// Human-facing summary. Must not contain secrets (NFR-S03).
    pub message: String,
    /// Provider raw detail. Never parsed by UI.
    pub detail: Option<String>,
    /// Machine-readable cause, when the code alone is not enough.
    ///
    /// One code covers several refusals — `ST-POL-001` says "the capability
    /// was denied", and a lock needs four distinguishable denials (locked /
    /// wrong secret / not configured / throttled). The code says *that*
    /// something was refused; this says *which* one.
    ///
    /// It is a closed enum on purpose. A free-text reason would have to be
    /// matched by string comparison in every consumer, and a contract that
    /// only holds if nobody rewords it is not a contract (ADR-061).
    ///
    /// **Boxed on purpose**: `DomainError` is returned by value from every
    /// `Result` in the kernel, so its size is the size of every error path.
    /// Inlining the reason pushed it past clippy's `result_large_err`
    /// threshold — the lint that exists precisely to keep the microkernel's
    /// error type small. The reason is absent on most errors, so the
    /// allocation only happens when there is one.
    pub reason: Option<Box<LockFailure>>,
}

impl std::fmt::Display for DomainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for DomainError {}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

impl DomainError {
    /// Construct with code and message.
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            detail: None,
            reason: None,
        }
    }

    /// Attach provider raw detail.
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Attach the machine-readable cause.
    pub fn with_reason(mut self, reason: LockFailure) -> Self {
        self.reason = Some(Box::new(reason));
        self
    }

    /// The machine-readable cause, if there is one.
    pub fn reason(&self) -> Option<&LockFailure> {
        self.reason.as_deref()
    }

    /// Whether the caller may retry.
    pub fn is_retryable(&self) -> bool {
        self.code.is_retryable()
    }

    /// Convenience constructors for the most frequent codes.
    pub fn core_invalid(msg: impl Into<String>) -> Self {
        Self::new(ErrorCode::CORE_INVALID, msg)
    }
    /// `ST-POL-001` capability denied.
    pub fn policy_denied(msg: impl Into<String>) -> Self {
        Self::new(ErrorCode::POLICY_DENIED, msg)
    }
    /// `ST-VFS-002` resource/file not found.
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::new(ErrorCode::VFS_NOT_FOUND, msg)
    }
    /// `ST-VFS-001` path escapes the workspace root.
    pub fn path_escape(msg: impl Into<String>) -> Self {
        Self::new(ErrorCode::VFS_PATH_ESCAPE, msg)
    }
}

impl serde::Serialize for DomainError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut st = s.serialize_struct("DomainError", 4)?;
        st.serialize_field("code", self.code.as_str())?;
        st.serialize_field("message", &self.message)?;
        st.serialize_field("detail", &self.detail)?;
        st.serialize_field("reason", &self.reason)?;
        st.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_code_is_unique_and_prefixed() {
        let mut all: Vec<&str> = ErrorCode::all()
            .iter()
            .chain(ErrorCode::all_observation())
            .map(|c| c.as_str())
            .collect();
        let count = all.len();
        all.sort_unstable();
        all.dedup();
        assert_eq!(all.len(), count, "duplicate error codes in registry");
        assert!(all.iter().all(|c| c.starts_with("ST-")));
    }

    #[test]
    fn category_uses_the_frozen_long_names() {
        // `category()` must return the category names in schemas/error_codes.csv,
        // NOT the short code prefix: `ST-DKR-003` is category `DOCKER`, not
        // `DKR`. The prefix and the category are deliberately different strings,
        // which is exactly why this is worth a test.
        assert_eq!(ErrorCode::DOCKER_CONFLICT.category(), "DOCKER");
        assert_eq!(ErrorCode::VFS_PATH_ESCAPE.category(), "VFS");
        assert_eq!(ErrorCode::OBS_DEADLINE_EXCEEDED.category(), "OBS");
        assert_eq!(ErrorCode::POLICY_DENIED.category(), "POLICY");
        assert_eq!(ErrorCode::PLUGIN_MANIFEST_INVALID.category(), "PLUGIN");
        assert_eq!(
            ErrorCode::SANDBOX_PROVIDER_UNAVAILABLE.category(),
            "SANDBOX"
        );
        assert_eq!(ErrorCode::STORE_TRANSACTION_FAILED.category(), "STORE");
        assert_eq!(ErrorCode::IPC_INVALID_FRAME.category(), "IPC");
    }

    #[test]
    fn every_code_matches_the_frozen_csv_category() {
        // Belt and braces over the single-case test above: derive the expected
        // category for EVERY registered code from its own prefix, and require the
        // hand-written mapping table to agree on all of them. A new code added to
        // the registry without a mapping entry fails here.
        // Returns `String`, not `&'static str`: the catch-all arm hands back a
        // slice of the input, which cannot satisfy a `'static` return.
        fn expected_from_prefix(code: &str) -> String {
            let prefix = code.split('-').nth(1).unwrap_or("CORE").to_string();
            match prefix.as_str() {
                "POL" => "POLICY".to_string(),
                "PLG" => "PLUGIN".to_string(),
                "DKR" => "DOCKER".to_string(),
                "SBX" => "SANDBOX".to_string(),
                "DB" => "STORE".to_string(),
                other => other.to_string(),
            }
        }
        for c in ErrorCode::all()
            .iter()
            .chain(ErrorCode::all_observation().iter())
        {
            assert_eq!(
                c.category(),
                expected_from_prefix(c.as_str()),
                "{} has the wrong category",
                c.as_str()
            );
        }
    }

    #[test]
    fn retryable_matches_csv() {
        assert!(ErrorCode::DOCKER_ENDPOINT_UNAVAILABLE.is_retryable());
        assert!(ErrorCode::OBS_STALE.is_retryable());
        assert!(!ErrorCode::POLICY_DENIED.is_retryable());
        assert!(!ErrorCode::VFS_PATH_ESCAPE.is_retryable());
    }

    #[test]
    fn error_serializes_with_code_message_detail() {
        let e = DomainError::new(ErrorCode::SANDBOX_PROVIDER_UNAVAILABLE, "multipass missing")
            .with_detail("exit status 1");
        let v = serde_json::to_value(&e).unwrap();
        assert_eq!(v["code"], "ST-SBX-001");
        assert_eq!(v["message"], "multipass missing");
        assert_eq!(v["detail"], "exit status 1");
    }

    /// 原因要**过线**：内核产出的原因，客户端拿到的必须是同一个。
    ///
    /// 只测「原因在不在」不够 —— 它可能是 `null`、可能是另一种原因、
    /// 也可能整个字段没被送上线。这条测的是**值本身**。
    #[test]
    fn the_lock_reason_travels_on_the_error() {
        use crate::model::resource::{LockFailure, LockKind};

        let reason = LockFailure::ResourceLocked {
            lock_kind: LockKind::Manual,
            reason: "还在用".to_string(),
        };
        let e = DomainError::new(ErrorCode::POLICY_DENIED, "locked").with_reason(reason.clone());
        assert_eq!(e.reason(), Some(&reason));

        let v = serde_json::to_value(&e).unwrap();
        assert_eq!(v["code"], "ST-POL-001");
        assert_eq!(v["reason"]["kind"], "resource_locked");
        assert_eq!(v["reason"]["reason"], "还在用");

        // 没有原因时是 `null` —— 不是空字符串，也不是被猜出来的一个。
        let plain = DomainError::new(ErrorCode::CORE_INVALID, "nope");
        assert!(plain.reason().is_none());
        assert!(serde_json::to_value(&plain).unwrap()["reason"].is_null());
    }
}
