//! ACI assertion 字段验证 (对标 Python `_lib_aci_emit.py` 10 个 validator).
//!
//! 设计: 所有 validator 都是纯函数, 抛 `AciValidationError`. 由 `builder::build` 串行调用.

use crate::error::AciValidationError;
use crate::types::{ExpectActual, ExpectValueType, Layer, ScopeMap, Severity, Status, ACI_VERSION};

/// .aci.json `scope_dimensions` 6 个 dim.
pub const VALID_SCOPE_DIMS: [&str; 6] = [
    "project",
    "module",
    "domain",
    "subdomain",
    "operation",
    "http_method",
];

/// 验证 `assertion_id`: 非空 + 含 `:`.
pub fn validate_assertion_id(aid: &str) -> Result<(), AciValidationError> {
    if aid.trim().is_empty() {
        return Err(AciValidationError::EmptyAssertionId);
    }
    if !aid.contains(':') {
        return Err(AciValidationError::AssertionIdMissingColon(aid.to_string()));
    }
    Ok(())
}

/// 验证 `scope`: 非空 + 维度在 6 dims 之内 + 值非空字符串.
pub fn validate_scope(scope: &ScopeMap) -> Result<(), AciValidationError> {
    if scope.is_empty() {
        return Err(AciValidationError::EmptyScope);
    }
    let unknown: Vec<String> = scope
        .keys()
        .filter(|k| !VALID_SCOPE_DIMS.contains(&k.as_str()))
        .cloned()
        .collect();
    if !unknown.is_empty() {
        return Err(AciValidationError::UnknownScopeDim {
            unknown,
            valid: VALID_SCOPE_DIMS.to_vec(),
        });
    }
    for (k, v) in scope {
        if v.trim().is_empty() {
            return Err(AciValidationError::EmptyScopeValue(k.clone()));
        }
    }
    Ok(())
}

/// 验证 expect / actual: 必填 type / value / description, type ∈ 17 个.
pub fn validate_expect_or_actual(
    label: &'static str,
    obj: &ExpectActual,
) -> Result<(), AciValidationError> {
    if obj.ty.as_str().is_empty() {
        return Err(AciValidationError::MissingExpectActualType(label));
    }
    if ExpectValueType::parse(obj.ty.as_str()).is_none() {
        return Err(AciValidationError::InvalidExpectActualType(
            label,
            obj.ty.as_str().to_string(),
        ));
    }
    if obj.value.is_null() {
        return Err(AciValidationError::MissingExpectActualValue(label));
    }
    if obj.description.trim().is_empty() {
        return Err(AciValidationError::MissingExpectActualDescription(label));
    }
    Ok(())
}

/// 验证 status ∈ 4 档.
pub fn validate_status(status: Status) -> Result<(), AciValidationError> {
    // Status 是强类型 enum, 编译时保证值, 此处 runtime 校验保留以备 from str
    let _ = status;
    Ok(())
}

/// 验证 severity ∈ 5 档.
pub fn validate_severity(severity: Severity) -> Result<(), AciValidationError> {
    let _ = severity;
    Ok(())
}

/// 验证 layer ∈ 4 档.
pub fn validate_layer(layer: Layer) -> Result<(), AciValidationError> {
    let _ = layer;
    Ok(())
}

/// 给 scope 默认补 `project` 字段 (避免每个调用方都重复).
///
/// 跟 Python `_with_default_scope` 1:1 行为:
/// - 如果 caller 已经填了 `project`, 保留 caller 值 (setdefault 语义).
/// - 否则填入 emitter 的 project 默认值.
pub fn with_default_scope(scope: &ScopeMap, project: &str) -> ScopeMap {
    let mut out = scope.clone();
    out.entry("project".to_string())
        .or_insert_with(|| project.to_string());
    out
}

/// `captured_at` RFC3339 UTC 时间戳 (per .aci.json field_semantics.captured_at).
///
/// 跟 Python `_now_rfc3339() = _dt.datetime.now(_dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")`
/// 1:1 (秒级精度, UTC, Z 后缀).
pub fn now_rfc3339() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

/// ACI version 常量 re-export (避免重复字符串).
pub fn aci_version() -> &'static str {
    ACI_VERSION
}
