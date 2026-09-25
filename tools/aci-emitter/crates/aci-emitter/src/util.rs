//! util.rs - 小工具 (CLI value coerce + scope k=v parse), 对标 Python `_lib_aci_emit.py`.

use crate::error::AciEmitError;
use crate::types::ScopeMap;

/// 解析 CLI `--scope k=v k=v ...` 形式 (空格分隔), 输出 BTreeMap (有序 + 去重).
///
/// 跟 Python `_parse_kv_pairs` 行为 1:1:
/// - 没有 `=` → `InvalidScopeKv`
/// - 重复 key → `DuplicateScopeKey`
pub fn parse_scope_kv<I, S>(items: I) -> Result<ScopeMap, AciEmitError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut out = ScopeMap::new();
    for item in items {
        let item = item.as_ref();
        if !item.contains('=') {
            return Err(AciEmitError::InvalidScopeKv(item.to_string()));
        }
        let (k, v) = item.split_once('=').expect("contains '=' checked");
        if out.contains_key(k) {
            return Err(AciEmitError::DuplicateScopeKey(k.to_string()));
        }
        out.insert(k.to_string(), v.to_string());
    }
    Ok(out)
}

/// CLI 字符串值 → `serde_json::Value`, 优先尝试 int / float / bool / string.
///
/// 跟 Python `_coerce_value` 1:1:
/// - `"true"` / `"false"` → bool
/// - 整数 (无小数点, 全数字) → i64
/// - 浮点数 (含 `.` 或 `e`) → f64
/// - 其余 → 原字符串
pub fn coerce_value(raw: &str) -> serde_json::Value {
    let lower = raw.to_ascii_lowercase();
    if lower == "true" {
        return serde_json::Value::Bool(true);
    }
    if lower == "false" {
        return serde_json::Value::Bool(false);
    }
    // 尝试整数: 全 ASCII digit + 可选前导 `-`
    if let Ok(n) = raw.parse::<i64>() {
        return serde_json::Value::Number(serde_json::Number::from(n));
    }
    // 尝试浮点数
    if let Ok(f) = raw.parse::<f64>() {
        if let Some(num) = serde_json::Number::from_f64(f) {
            if f.is_finite() {
                return serde_json::Value::Number(num);
            }
        }
    }
    serde_json::Value::String(raw.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_scope_kv_ok() {
        let m = parse_scope_kv(["project=star-flash-mock", "module=guards"]).unwrap();
        assert_eq!(m.get("project").unwrap(), "star-flash-mock");
        assert_eq!(m.get("module").unwrap(), "guards");
    }

    #[test]
    fn parse_scope_kv_no_equals() {
        let err = parse_scope_kv(["bad"]).unwrap_err();
        assert!(matches!(err, AciEmitError::InvalidScopeKv(_)));
    }

    #[test]
    fn parse_scope_kv_dup() {
        let err = parse_scope_kv(["k=1", "k=2"]).unwrap_err();
        assert!(matches!(err, AciEmitError::DuplicateScopeKey(_)));
    }

    #[test]
    fn coerce_value_int() {
        let v = coerce_value("2000");
        assert_eq!(v, serde_json::json!(2000));
    }

    #[test]
    fn coerce_value_float() {
        let v = coerce_value("1.414");
        assert_eq!(v, serde_json::json!(1.414));
    }

    #[test]
    fn coerce_value_bool() {
        assert_eq!(coerce_value("true"), serde_json::json!(true));
        assert_eq!(coerce_value("false"), serde_json::json!(false));
        assert_eq!(coerce_value("TRUE"), serde_json::json!(true));
    }

    #[test]
    fn coerce_value_string() {
        assert_eq!(coerce_value("hello"), serde_json::json!("hello"));
    }
}
