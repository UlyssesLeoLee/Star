//! arg-bridge integration tests (per ULYS-154 PR-2).
//!
//! arg-bridge 已有 inline test (flush_test.rs, integration.rs, langgraph_test.rs).
//! 这里加独立 integration test 覆盖 error module 各 variant.

use star_arg_bridge::error::BridgeError;
use std::time::Duration;

#[test]
fn bridge_error_display_bolt_subscribe_failed() {
    let e = BridgeError::BoltSubscribeFailed("connection refused".to_string());
    let msg = format!("{e}");
    assert!(msg.contains("Bolt"), "msg: {msg}");
    assert!(msg.contains("connection refused"), "msg: {msg}");
}

#[test]
fn bridge_error_display_langgraph_reducer_failed() {
    let e = BridgeError::LangGraphReducerFailed("pyo3 panic".to_string());
    let msg = format!("{e}");
    assert!(msg.contains("LangGraph") || msg.contains("Reducer"), "msg: {msg}");
    assert!(msg.contains("pyo3 panic"), "msg: {msg}");
}

#[test]
fn bridge_error_display_flush_timeout() {
    let e = BridgeError::FlushTimeout(Duration::from_secs(30));
    let msg = format!("{e}");
    assert!(msg.contains("flush") || msg.contains("timed out"), "msg: {msg}");
    assert!(msg.contains("30s"), "msg: {msg}");
}

#[test]
fn bridge_error_display_offline_persist_failed() {
    let e = BridgeError::OfflinePersistFailed("disk full".to_string());
    let msg = format!("{e}");
    assert!(msg.contains("persist") || msg.contains("Offline") || msg.contains("disk"), "msg: {msg}");
}

#[test]
fn bridge_error_display_memgraph_down() {
    let e = BridgeError::MemgraphDown;
    let msg = format!("{e}");
    assert!(!msg.is_empty(), "msg should not be empty");
    assert!(msg.to_lowercase().contains("memgraph") || msg.to_lowercase().contains("down"),
        "msg: {msg}");
}

#[test]
fn bridge_error_display_internal_error() {
    let e = BridgeError::InternalError("unexpected state".to_string());
    let msg = format!("{e}");
    assert!(msg.contains("unexpected state"), "msg: {msg}");
}

#[test]
fn bridge_error_debug_format() {
    let e = BridgeError::MemgraphDown;
    let dbg = format!("{:?}", e);
    assert!(dbg.contains("MemgraphDown"), "debug: {dbg}");
}

#[test]
fn bridge_error_clone_send_sync() {
    // BridgeError 应该是 Send + Sync (per thiserror + String/Duration 字段).
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<BridgeError>();
}
