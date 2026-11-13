//! tests/unit/test_audit.rs — audit 模块单测 (per DD §5.1: 6 TC)
//!
//! JSON Lines 格式 / append-only / parseable / composite / fail-closed / PG URL 校验。

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use sandboxd::audit::{AuditEvent, AuditSink, CompositeAuditSink, JsonLinesAuditSink, PgAuditSink};
use sandboxd::error::SandboxdError;
use uuid::Uuid;

fn sample_event(table: sandboxd::audit::AuditTable, event_type: &str) -> AuditEvent {
    let id = Uuid::new_v4();
    match table {
        sandboxd::audit::AuditTable::Session => AuditEvent::session_event(
            id,
            "dispatcher.py",
            "task-1",
            None,
            event_type,
            Some(serde_json::json!({"resource": {"memory_mb": 1024}})),
            Uuid::nil(),
            Uuid::nil(),
            Uuid::nil(),
        ),
        sandboxd::audit::AuditTable::Audit => AuditEvent::audit_event(
            id,
            "dispatcher.py",
            "task-1",
            event_type,
            Some(12),
            vec!["fs: test".to_string()],
            Uuid::nil(),
            Uuid::nil(),
            Uuid::nil(),
        ),
    }
}

#[tokio::test]
async fn jsonl_sink_writes_parseable_lines() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("audit.jsonl");
    let sink = JsonLinesAuditSink::new(path.to_str().unwrap()).unwrap();
    sink.record(&sample_event(
        sandboxd::audit::AuditTable::Session,
        "Created",
    ))
    .await
    .unwrap();
    sink.record(&sample_event(
        sandboxd::audit::AuditTable::Audit,
        "ProcessStart",
    ))
    .await
    .unwrap();

    let content = std::fs::read_to_string(&path).unwrap();
    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(lines.len(), 2, "append-only: {content}");
    for line in lines {
        // NFR-M-3: 100% parseable JSON Lines
        let v: serde_json::Value = serde_json::from_str(line).unwrap();
        assert!(v.get("session_id").is_some());
        assert!(v.get("event_type").is_some());
        assert!(v.get("timestamp").is_some());
        assert!(v.get("tenant_id").is_some());
    }
}

#[tokio::test]
async fn jsonl_sink_append_only_across_reopen() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("audit2.jsonl");
    let sink = JsonLinesAuditSink::new(path.to_str().unwrap()).unwrap();
    sink.record(&sample_event(
        sandboxd::audit::AuditTable::Audit,
        "ProcessExit",
    ))
    .await
    .unwrap();
    drop(sink);
    // 重新打开 (模拟 daemon 重启) → 不截断
    let sink2 = JsonLinesAuditSink::new(path.to_str().unwrap()).unwrap();
    sink2
        .record(&sample_event(
            sandboxd::audit::AuditTable::Audit,
            "ProcessExit",
        ))
        .await
        .unwrap();
    let content = std::fs::read_to_string(&path).unwrap();
    assert_eq!(content.lines().count(), 2);
}

#[tokio::test]
async fn composite_all_written() {
    // 组合 sink: 全写, 任一失败 → Err (FR-7.2 fail-closed)
    struct CountSink {
        count: Arc<Mutex<usize>>,
        fail: bool,
    }
    #[async_trait]
    impl AuditSink for CountSink {
        async fn record(&self, _event: &AuditEvent) -> Result<(), SandboxdError> {
            if self.fail {
                return Err(SandboxdError::AuditLogWriteFailed("boom".to_string()));
            }
            *self.count.lock().unwrap() += 1;
            Ok(())
        }
    }
    let count = Arc::new(Mutex::new(0usize));
    let ok = CompositeAuditSink::new(vec![
        Box::new(CountSink {
            count: Arc::clone(&count),
            fail: false,
        }),
        Box::new(CountSink {
            count: Arc::clone(&count),
            fail: false,
        }),
    ]);
    ok.record(&sample_event(
        sandboxd::audit::AuditTable::Audit,
        "ProcessStart",
    ))
    .await
    .unwrap();
    assert_eq!(*count.lock().unwrap(), 2);

    // 任一 sink 失败 → Err
    let bad = CompositeAuditSink::new(vec![
        Box::new(CountSink {
            count: Arc::clone(&count),
            fail: false,
        }),
        Box::new(CountSink {
            count: Arc::clone(&count),
            fail: true,
        }),
    ]);
    let err = bad
        .record(&sample_event(
            sandboxd::audit::AuditTable::Audit,
            "ProcessStart",
        ))
        .await
        .unwrap_err();
    assert!(matches!(err, SandboxdError::AuditLogWriteFailed(_)));
}

#[tokio::test]
async fn pg_sink_empty_url_errors() {
    // 空 URL → 立即 Err (由启动逻辑决定 file-only 降级)
    let err = PgAuditSink::connect("")
        .await
        .err()
        .expect("expected error");
    assert!(matches!(err, SandboxdError::AuditLogWriteFailed(_)));
}

#[tokio::test]
async fn audit_event_has_11_fields() {
    // FR-6.2: 11 字段全部可序列化
    let ev = sample_event(sandboxd::audit::AuditTable::Audit, "NetworkBlock");
    let v = serde_json::to_value(&ev).unwrap();
    for field in [
        "session_id",
        "client_id",
        "task_id",
        "event_type",
        "resource_used",
        "network_violations",
        "fs_violations",
        "capability_violations",
        "timestamp",
        "latency_ms",
        "hit_violations",
    ] {
        assert!(v.get(field).is_some(), "missing field {field}");
    }
}
