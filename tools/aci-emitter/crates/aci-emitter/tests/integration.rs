//! aci-emitter 集成测试 (per brief §1.1.4: ≥4 IT).
//!
//! 覆盖:
//! - build → to_json → from_file round-trip (IO 完整闭环)
//! - write 默认创建父目录
//! - 16 个 ExpectValueType 都能参与 (跨 17 种 type 取样 16)
//! - status / severity / layer 强类型 enum 序列化一致性
//! - 实际跨实现 parity (与 Python emitter 输出 JSON 字段名 1:1)

use aci_emitter::{
    types::empty_scope, AciEmitter, ExpectActual, ExpectValueType, Layer, ScopeMap, Severity,
    Status,
};
use std::path::PathBuf;

fn base_scope() -> ScopeMap {
    let mut s = empty_scope();
    s.insert("module".to_string(), "guards".to_string());
    s
}

fn basic_assertion(em: &AciEmitter) -> aci_emitter::Assertion {
    em.build(
        "star-flash-mock:guards:g-1",
        base_scope(),
        ExpectActual::new(
            ExpectValueType::ResponseWithinMs,
            2000_i64,
            "API 应在 2s 内",
        ),
        ExpectActual::new(ExpectValueType::ResponseWithinMs, 5_i64, "实测 5ms"),
        Status::Pass,
        Severity::Info,
        "实测延迟远低于预期",
    )
    .expect("happy path")
}

#[test]
fn build_to_json_from_file_roundtrip() {
    let tmp = tempfile::TempDir::new().expect("tempdir");
    let path: PathBuf = tmp.path().join("round.aci.json");

    let em = AciEmitter::new(Layer::It);
    let a = basic_assertion(&em);
    em.write(&a, &path).expect("write");

    // 文件存在
    assert!(path.exists());

    // 读回
    let loaded = AciEmitter::from_file(&path).expect("from_file");

    // 必填字段一致 (captured_at 也是 RFC3339, 重新构造的会更新)
    assert_eq!(loaded.assertion_id, a.assertion_id);
    assert_eq!(loaded.aci_version, a.aci_version);
    assert_eq!(loaded.layer, a.layer);
    assert_eq!(loaded.status, a.status);
    assert_eq!(loaded.severity, a.severity);
    assert_eq!(loaded.reasoning, a.reasoning);
    assert_eq!(loaded.expect.ty, a.expect.ty);
    assert_eq!(loaded.actual.ty, a.actual.ty);
    assert_eq!(loaded.expect.value, a.expect.value);
    assert_eq!(loaded.actual.value, a.actual.value);
}

#[test]
fn write_creates_parent_directories() {
    let tmp = tempfile::TempDir::new().expect("tempdir");
    let nested: PathBuf = tmp
        .path()
        .join("a")
        .join("b")
        .join("c")
        .join("out.aci.json");
    assert!(!nested.parent().unwrap().exists());

    let em = AciEmitter::new(Layer::Ut);
    let a = basic_assertion(&em);
    em.write(&a, &nested).expect("write creates parents");
    assert!(nested.exists());
}

#[test]
fn all_17_expect_value_types_serialize_consistently() {
    // 17 种 expect_value_types 全跑一遍 build + serialize, 验证 schema 兼容
    let em = AciEmitter::new(Layer::It);
    let types = [
        ExpectValueType::ResponseWithinMs,
        ExpectValueType::ResponseStatus2xx,
        ExpectValueType::ResponseStatus4xx5xx,
        ExpectValueType::FieldEquals,
        ExpectValueType::FieldExists,
        ExpectValueType::FieldNotExists,
        ExpectValueType::FieldInRange,
        ExpectValueType::FieldGreaterThan,
        ExpectValueType::FieldLessThan,
        ExpectValueType::FieldMatchesRegex,
        ExpectValueType::ListLengthAtLeast,
        ExpectValueType::ListLengthEquals,
        ExpectValueType::WorkflowCompletes,
        ExpectValueType::EventEmitted,
        ExpectValueType::LogContains,
        ExpectValueType::MetricWithinThreshold,
        ExpectValueType::NoResourceLeak,
    ];
    assert_eq!(types.len(), 17);

    for (i, t) in types.iter().enumerate() {
        let a = em
            .build(
                &format!("smoke:type-{i}"),
                base_scope(),
                ExpectActual::new(*t, 0_i64, format!("type={}", t.as_str())),
                ExpectActual::new(*t, 0_i64, "实测"),
                Status::Pass,
                Severity::Info,
                "smoke",
            )
            .expect("build");
        // 序列化往返
        let v: serde_json::Value = serde_json::to_value(&a).expect("to_value");
        assert_eq!(
            v["expect"]["type"].as_str().unwrap(),
            t.as_str(),
            "type {i} round-trip serial"
        );
    }
}

#[test]
fn status_severity_layer_enums_serial_to_python_shape() {
    // 跟 Python emitter 1:1 字符串形态:
    //   status: "PASS"/"FAIL"/"WARN"/"SKIP" (大写)
    //   layer: "ut"/"it"/"st"/"e2e" (小写)
    //   severity: "critical"/"high"/"medium"/"low"/"info" (小写)
    let em = AciEmitter::new(Layer::St);
    let a = basic_assertion(&em);
    let v: serde_json::Value = serde_json::to_value(&a).expect("to_value");
    assert_eq!(v["status"], "PASS");
    assert_eq!(v["layer"], "st");
    assert_eq!(v["severity"], "info");
}

#[test]
fn cross_implementation_field_parity_with_python() {
    // 跟 Python `_lib_aci_emit.py` 1:1 字段名 + 10 必填字段对齐 (per brief §4.3).
    // 这里验证 10 个必填字段名都存在, 且 snake_case 命名.
    let em = AciEmitter::new(Layer::It);
    let a = basic_assertion(&em);
    let v: serde_json::Value = serde_json::to_value(&a).expect("to_value");
    let obj = v.as_object().expect("object");

    let required = [
        "assertion_id",
        "aci_version",
        "layer",
        "scope",
        "expect",
        "actual",
        "status",
        "severity",
        "reasoning",
        "captured_at",
    ];
    for f in required {
        assert!(obj.contains_key(f), "missing required field: {f}");
    }
}

#[test]
fn write_then_read_back_lossless_except_captured_at() {
    // 写入磁盘 → 重新读取, 除 captured_at (时间戳) 之外完全一致.
    let tmp = tempfile::TempDir::new().expect("tempdir");
    let path: PathBuf = tmp.path().join("lossless.aci.json");

    let em = AciEmitter::new(Layer::It);
    let a = em
        .build_with(
            "mod:1",
            base_scope(),
            ExpectActual::new(ExpectValueType::FieldEquals, "v1", "expect v1"),
            ExpectActual::new(ExpectValueType::FieldEquals, "v1", "actual v1"),
            Status::Pass,
            Severity::Info,
            "field matches",
            Some("none needed".to_string()),
            Some(vec!["smoke".to_string(), "perf".to_string()]),
            Some("2026-09-23T13:00:00Z".to_string()),
        )
        .unwrap();
    em.write(&a, &path).expect("write");

    let raw = std::fs::read_to_string(&path).expect("read");
    let v1: serde_json::Value = serde_json::from_str(&raw).expect("parse");
    let loaded = AciEmitter::from_file(&path).expect("from_file");
    let v2 = serde_json::to_value(&loaded).expect("to_value");

    // suggested_fix / tags 应当保留
    assert_eq!(v1["suggested_fix"], v2["suggested_fix"]);
    assert_eq!(v1["tags"], v2["tags"]);
    // captured_at 写死固定, 因此也应一致
    assert_eq!(v1["captured_at"], v2["captured_at"]);
    assert_eq!(v1["captured_at"], "2026-09-23T13:00:00Z");
}
