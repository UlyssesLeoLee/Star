// SPDX-License-Identifier: MIT OR Apache-2.0
//! InMemoryBackend::incr stub 测试 — per OPT-WORKER-01 §3.5
//!
//! 验证 incr 错误使用 NotImplemented 变体, 含结构化字段 (feature / suggestion / p4_phase)。

use star_cache::{CacheBackend, CacheError, InMemoryBackend};

#[tokio::test]
async fn in_memory_incr_returns_not_implemented() {
    let c = InMemoryBackend::new(60);
    let result = c.incr("k", 1).await;
    match result {
        Err(CacheError::NotImplemented {
            feature,
            suggestion,
        }) => {
            assert_eq!(feature, "in_memory_incr");
            assert!(
                suggestion.contains("RedisBackend"),
                "suggestion should reference RedisBackend: {suggestion}"
            );
            assert!(
                suggestion.contains("p4_phase=G.4"),
                "suggestion should reference p4_phase=G.4: {suggestion}"
            );
        }
        other => panic!("expected NotImplemented, got {other:?}"),
    }
}

#[test]
fn cache_error_not_implemented_display() {
    let err = CacheError::NotImplemented {
        feature: "in_memory_incr".into(),
        suggestion: "use RedisBackend".into(),
    };
    let s = err.to_string();
    assert!(s.contains("not_implemented"));
    assert!(s.contains("in_memory_incr"));
    assert!(s.contains("use RedisBackend"));
}
