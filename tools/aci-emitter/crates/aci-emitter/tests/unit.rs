//! aci-emitter 单元测试 (per brief §1.1.4: ≥8 单测).
//!
//! 覆盖:
//! - 10 必填字段构造 (PASS / FAIL)
//! - 各种 validation 错误 (assertion_id / scope / expect / actual / FAIL 必填 reasoning)
//! - with_tags dedupe 行为
//! - JSON 输出 (sort_keys + ensure_ascii=false 兼容)
//! - to_value 反序列化 round-trip

use aci_emitter::{
    types::empty_scope, AciEmitter, AciValidationError, ExpectActual, ExpectValueType, Layer,
    ScopeMap, Severity, Status, ACI_VERSION,
};

fn base_scope() -> ScopeMap {
    let mut s = empty_scope();
    s.insert("module".to_string(), "guards".to_string());
    s
}

fn small_expect() -> ExpectActual {
    ExpectActual::new(
        ExpectValueType::ResponseWithinMs,
        2000_i64,
        "API 应在 2s 内",
    )
}

fn small_actual(value: i64) -> ExpectActual {
    ExpectActual::new(ExpectValueType::ResponseWithinMs, value, "实测")
}

#[test]
fn happy_path_pass_minimal() {
    let em = AciEmitter::new(Layer::It);
    let a = em
        .build(
            "star-flash-mock:guards:g-1",
            base_scope(),
            small_expect(),
            small_actual(5),
            Status::Pass,
            Severity::Info,
            "实测 5ms 远低于预期 2000ms",
        )
        .expect("happy path");

    assert_eq!(a.assertion_id, "star-flash-mock:guards:g-1");
    assert_eq!(a.aci_version, ACI_VERSION);
    assert_eq!(a.layer, Layer::It);
    assert_eq!(a.status, Status::Pass);
    assert_eq!(a.severity, Severity::Info);
    assert_eq!(a.scope.get("project").unwrap(), "aci-emitter"); // default project
    assert_eq!(a.scope.get("module").unwrap(), "guards");
    assert!(a.suggested_fix.is_none());
    assert!(a.tags.is_none());
    // captured_at RFC3339 UTC Z
    assert!(a.captured_at.ends_with('Z'));
    assert!(a.captured_at.len() >= 20);
}

#[test]
fn happy_path_fail_with_reasoning_and_suggested_fix() {
    let em = AciEmitter::new(Layer::St);
    let mut scope = empty_scope();
    scope.insert("domain".to_string(), "kms".to_string());

    let a = em
        .build_with(
            "star-flash-mock:kms:timeout-1",
            scope,
            small_expect(),
            small_actual(5000),
            Status::Fail,
            Severity::High,
            "实测 5000ms 超 2000ms 预期",
            Some("增加 KMS 连接池大小 + 加 retry-with-jitter".to_string()),
            Some(vec![
                "perf".to_string(),
                "kms".to_string(),
                "timeout".to_string(),
            ]),
            Some("2026-09-23T13:00:00Z".to_string()),
        )
        .expect("fail with reasoning");

    assert_eq!(a.status, Status::Fail);
    assert!(a.suggested_fix.is_some());
    assert_eq!(a.tags.unwrap().len(), 3);
    assert_eq!(a.captured_at, "2026-09-23T13:00:00Z");
}

#[test]
fn fail_without_reasoning_rejected() {
    let em = AciEmitter::new(Layer::It);
    let err = em
        .build(
            "mod:1",
            base_scope(),
            small_expect(),
            small_actual(5),
            Status::Fail,
            Severity::High,
            "",
        )
        .unwrap_err();

    assert!(matches!(err, AciValidationError::FailRequiresReasoning));
}

#[test]
fn empty_assertion_id_rejected() {
    let em = AciEmitter::new(Layer::It);
    let err = em
        .build(
            "",
            base_scope(),
            small_expect(),
            small_actual(5),
            Status::Pass,
            Severity::Info,
            "ok",
        )
        .unwrap_err();
    assert!(matches!(err, AciValidationError::EmptyAssertionId));
}

#[test]
fn assertion_id_missing_colon_rejected() {
    let em = AciEmitter::new(Layer::It);
    let err = em
        .build(
            "no-colon",
            base_scope(),
            small_expect(),
            small_actual(5),
            Status::Pass,
            Severity::Info,
            "ok",
        )
        .unwrap_err();
    assert!(matches!(
        err,
        AciValidationError::AssertionIdMissingColon(_)
    ));
}

#[test]
fn empty_scope_rejected() {
    let em = AciEmitter::new(Layer::It);
    let err = em
        .build(
            "mod:1",
            empty_scope(),
            small_expect(),
            small_actual(5),
            Status::Pass,
            Severity::Info,
            "ok",
        )
        .unwrap_err();
    assert!(matches!(err, AciValidationError::EmptyScope));
}

#[test]
fn unknown_scope_dim_rejected() {
    let em = AciEmitter::new(Layer::It);
    let mut bad = empty_scope();
    bad.insert("bogus_dim".to_string(), "x".to_string());
    let err = em
        .build(
            "mod:1",
            bad,
            small_expect(),
            small_actual(5),
            Status::Pass,
            Severity::Info,
            "ok",
        )
        .unwrap_err();
    match err {
        AciValidationError::UnknownScopeDim { unknown, valid } => {
            assert_eq!(unknown, vec!["bogus_dim".to_string()]);
            assert_eq!(valid.len(), 6);
        }
        _ => panic!("expected UnknownScopeDim"),
    }
}

#[test]
fn with_tags_dedupes_and_appends() {
    let em = AciEmitter::new(Layer::It);
    let mut a = em
        .build(
            "mod:1",
            base_scope(),
            small_expect(),
            small_actual(5),
            Status::Pass,
            Severity::Info,
            "ok",
        )
        .unwrap();
    a.tags = Some(vec!["perf".to_string()]);

    let merged = em.with_tags(
        a.clone(),
        vec!["smoke".to_string(), "perf".to_string(), "perf".to_string()],
    );
    let tags = merged.tags.unwrap();
    assert_eq!(tags, vec!["perf".to_string(), "smoke".to_string()]);
}

#[test]
fn json_round_trip_through_to_json() {
    let em = AciEmitter::new(Layer::It);
    let a = em
        .build(
            "mod:1",
            base_scope(),
            small_expect(),
            small_actual(5),
            Status::Pass,
            Severity::Info,
            "ok",
        )
        .unwrap();

    let json = em.to_json(&a);
    // sort_keys 应当包含 10 个字段 + 嵌套结构 (per 守门 #13 W/T/M)
    assert!(json.contains("\"assertion_id\""));
    assert!(json.contains("\"aci_version\""));
    assert!(json.contains("\"layer\""));
    assert!(json.contains("\"scope\""));
    assert!(json.contains("\"expect\""));
    assert!(json.contains("\"actual\""));
    assert!(json.contains("\"status\""));
    assert!(json.contains("\"severity\""));
    assert!(json.contains("\"reasoning\""));
    assert!(json.contains("\"captured_at\""));
    // 中文 description 不应被 ascii 转义 (ensure_ascii=false)
    assert!(json.contains("API 应在 2s 内"));
}

#[test]
fn serde_round_trip_via_value() {
    let em = AciEmitter::new(Layer::It);
    let a = em
        .build(
            "mod:1",
            base_scope(),
            small_expect(),
            small_actual(5),
            Status::Pass,
            Severity::Info,
            "ok",
        )
        .unwrap();

    let v: serde_json::Value = em.to_value(&a);
    assert_eq!(v["assertion_id"], "mod:1");
    assert_eq!(v["status"], "PASS"); // 强类型 enum 序列化为大写
    assert_eq!(v["layer"], "it");
    assert_eq!(v["severity"], "info");
    assert_eq!(v["expect"]["type"], "response_within_ms");
    assert_eq!(v["expect"]["value"], 2000);
}

#[test]
fn custom_project_via_with_project() {
    let em = AciEmitter::new(Layer::Ut).with_project("star-flash-mock");
    let a = em
        .build(
            "mod:1",
            base_scope(),
            small_expect(),
            small_actual(5),
            Status::Pass,
            Severity::Info,
            "ok",
        )
        .unwrap();
    assert_eq!(a.scope.get("project").unwrap(), "star-flash-mock");
}

#[test]
fn layer_default_scope_keeps_caller_project() {
    let em = AciEmitter::new(Layer::It).with_project("star-flash-mock");
    let mut scope = empty_scope();
    scope.insert("project".to_string(), "caller-project".to_string());
    scope.insert("module".to_string(), "guards".to_string());

    let a = em
        .build(
            "mod:1",
            scope,
            small_expect(),
            small_actual(5),
            Status::Pass,
            Severity::Info,
            "ok",
        )
        .unwrap();
    // caller 提供的 project 优先
    assert_eq!(a.scope.get("project").unwrap(), "caller-project");
}
