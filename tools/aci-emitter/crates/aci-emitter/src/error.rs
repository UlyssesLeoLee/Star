//! ACI emitter 错误类型 (对标 Python `_lib_aci_emit.py::AciEmitError / AciValidationError`).
//!
//! 两条错误链:
//! - `AciEmitError`: 用户输入 / IO / schema 不符 (CLI 入口处抛)
//! - `AciValidationError`: assertion 字段验证失败 (field-level)

use std::io;
use thiserror::Error;

/// ACI emit 阶段错误 (用户输入 / schema 不符 / IO 失败).
#[derive(Debug, Error)]
pub enum AciEmitError {
    #[error("IO 错误: {0}")]
    Io(#[from] io::Error),

    #[error("JSON 序列化 / 反序列化失败: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Scope 形式应为 k=v, 收到: {0:?}")]
    InvalidScopeKv(String),

    #[error("Scope 重复 key: {0:?}")]
    DuplicateScopeKey(String),

    #[error("Value 形式错: {0}")]
    InvalidValue(String),

    #[error("Layer 必须是 {expected:?} 之一, 收到: {got:?}")]
    InvalidLayer {
        expected: Vec<&'static str>,
        got: String,
    },

    #[error("Status 必须是 {expected:?} 之一, 收到: {got:?}")]
    InvalidStatus {
        expected: Vec<&'static str>,
        got: String,
    },

    #[error("Severity 必须是 {expected:?} 之一, 收到: {got:?}")]
    InvalidSeverity {
        expected: Vec<&'static str>,
        got: String,
    },

    #[error("ExpectValueType 不在 17 个 schema 枚举中: {0:?}")]
    InvalidExpectValueType(String),
}

/// ACI assertion 字段验证失败.
#[derive(Debug, Error)]
pub enum AciValidationError {
    #[error("assertion_id 必填且为非空字符串")]
    EmptyAssertionId,

    #[error(
        "assertion_id 格式应为 <module>:<id>, 收到: {0:?} (per .aci.json schema_required_fields)"
    )]
    AssertionIdMissingColon(String),

    #[error("scope 必填且为非空 dict (至少含 1 个 dimension)")]
    EmptyScope,

    #[error("scope 含未声明 dimension: {unknown:?} (VALID_SCOPE_DIMS = {valid:?})")]
    UnknownScopeDim {
        unknown: Vec<String>,
        valid: Vec<&'static str>,
    },

    #[error("scope[{0:?}] 必填且为非空字符串")]
    EmptyScopeValue(String),

    #[error("{0}.type 必填 (expect_value_types 之一)")]
    MissingExpectActualType(&'static str),

    #[error("{0}.type={1:?} 不在 VALID_EXPECT_TYPES 中")]
    InvalidExpectActualType(&'static str, String),

    #[error("{0}.value 必填 (类型由 type 决定)")]
    MissingExpectActualValue(&'static str),

    #[error("{0}.description 必填 (LLM 必读)")]
    MissingExpectActualDescription(&'static str),

    #[error("status=FAIL 时 reasoning 必填 (LLM 必读「为什么 fail」)")]
    FailRequiresReasoning,
}

/// 顶层结果类型 (emit 阶段用 AciEmitError, 字段验证用 AciValidationError).
pub type AciResult<T> = Result<T, AciEmitError>;
