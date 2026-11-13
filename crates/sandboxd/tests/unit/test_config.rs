//! tests/unit/test_config.rs — config 模块单元测试 (per DD §5.1 test_config: 6 TC)
//!
//! YAML 加载 / 校验 / 边界 / 缺字段 / 类型错误 / 默认值。

use sandboxd::config::SandboxdConfig;

/// 写临时 YAML 配置文件
fn write_yaml(dir: &std::path::Path, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, content).unwrap();
    path
}

#[test]
fn yaml_load_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_yaml(
        dir.path(),
        "ok.yaml",
        r#"
server:
  grpc_port: 50051
  http_port: 50052
  max_concurrent_sessions: 100
  default_session_timeout_sec: 1800
observability:
  audit_log_file: "audit.jsonl"
  audit_log_pg_url: ""
"#,
    );
    let config = SandboxdConfig::from_yaml(&path).unwrap();
    assert_eq!(config.server.grpc_port, 50051);
    assert_eq!(config.server.http_port, 50052);
    assert_eq!(config.server.max_concurrent_sessions, 100);
    assert_eq!(config.observability.audit_log_file, "audit.jsonl");
    assert!(config.observability.audit_log_pg_url.is_empty());
}

#[test]
fn yaml_partial_uses_defaults() {
    // 缺字段 → serde default 填充 (per FR-1.1 配置缺省容错)
    let dir = tempfile::tempdir().unwrap();
    let path = write_yaml(dir.path(), "partial.yaml", "server:\n  grpc_port: 6000\n");
    let config = SandboxdConfig::from_yaml(&path).unwrap();
    assert_eq!(config.server.grpc_port, 6000);
    assert_eq!(config.server.max_concurrent_sessions, 100); // default
    assert_eq!(config.isolation.resource.default_memory_mb, 1024); // default
    assert_eq!(config.observability.audit_log_retention_days, 90); // default
    assert_eq!(config.server.default_session_timeout_sec, 1800);
}

#[test]
fn validate_rejects_zero_max_sessions() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_yaml(
        dir.path(),
        "bad.yaml",
        "server:\n  max_concurrent_sessions: 0\nobservability:\n  audit_log_file: \"a.jsonl\"\n",
    );
    let err = SandboxdConfig::from_yaml(&path).unwrap_err();
    assert!(err.to_string().contains("max_concurrent_sessions"), "{err}");
}

#[test]
fn validate_rejects_retention_zero() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_yaml(
        dir.path(),
        "bad2.yaml",
        "observability:\n  audit_log_retention_days: 0\n  audit_log_file: \"a.jsonl\"\n",
    );
    let err = SandboxdConfig::from_yaml(&path).unwrap_err();
    assert!(err.to_string().contains("retention_days"), "{err}");
}

#[test]
fn validate_rejects_no_audit_sink() {
    // FR-7.2: audit 必须至少 1 sink (file 或 pg)
    let dir = tempfile::tempdir().unwrap();
    let path = write_yaml(
        dir.path(),
        "nosink.yaml",
        "observability:\n  audit_log_file: \"\"\n  audit_log_pg_url: \"\"\n",
    );
    let err = SandboxdConfig::from_yaml(&path).unwrap_err();
    assert!(err.to_string().contains("audit"), "{err}");
}

#[test]
fn validate_rejects_same_ports() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_yaml(
        dir.path(),
        "ports.yaml",
        "server:\n  grpc_port: 50051\n  http_port: 50051\nobservability:\n  audit_log_file: \"a.jsonl\"\n",
    );
    let err = SandboxdConfig::from_yaml(&path).unwrap_err();
    assert!(err.to_string().contains("differ"), "{err}");
}

#[test]
fn invalid_yaml_type_errors() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_yaml(
        dir.path(),
        "type.yaml",
        "server:\n  grpc_port: \"not-a-port\"\n",
    );
    assert!(SandboxdConfig::from_yaml(&path).is_err());
}

#[test]
fn defaults_are_valid() {
    let config = SandboxdConfig::default();
    config.validate().unwrap();
    assert_eq!(config.grpc_addr(), "127.0.0.1:50051");
    assert_eq!(config.http_addr(), "127.0.0.1:50052");
    assert_eq!(config.server.max_concurrent_sessions, 100);
}
