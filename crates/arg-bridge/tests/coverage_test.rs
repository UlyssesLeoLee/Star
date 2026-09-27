//! arg-bridge integration tests (per ULYS-154 PR-2).
//!
//! arg-bridge 已有 inline test (flush_test.rs, integration.rs, langgraph_test.rs).
//! 这里加独立 integration test 覆盖 error module 各 variant.

use arg_bridge::error;

#[test]
fn arg_bridge_error_display_not_implemented() {
    let e = error::ArgBridgeError::NotImplemented;
    let msg = format!("{e}");
    assert!(!msg.is_empty());
    let lmsg = msg.to_lowercase();
    assert!(lmsg.contains("not") || lmsg.contains("implement"),
        "msg should mention not/implement, got: {msg}");
}

#[test]
fn arg_bridge_error_debug_format() {
    let e = error::ArgBridgeError::NotImplemented;
    let dbg = format!("{:?}", e);
    assert!(dbg.contains("NotImplemented"), "debug: {dbg}");
}

#[test]
fn arg_bridge_error_clone() {
    let e = error::ArgBridgeError::NotImplemented;
    let _e2 = e.clone();
}
