// SQLite 持久化集成测试 (per守门 #13 a W 派生 — 物理删除 / 短 TTL)
// 验证: 写入 → 关连接 → 重开 → 数据还在 (WAL persistence)
#![allow(missing_docs)]
use star_taskqueue::{AgentTask, SqliteTaskQueue, TaskQueue, TaskState};
use uuid::Uuid;

fn make_task(kind: &str) -> AgentTask {
    AgentTask {
        task_id: Uuid::new_v4(),
        tenant_id: Uuid::new_v4(),
        kind: kind.into(),
        payload: serde_json::json!({"k": "v"}),
        idempotency_key: Uuid::new_v4().to_string(),
        created_at_ms: 1_700_000_000_000,
        state: TaskState::Pending,
        state_history: vec![],
        attempt: 0,
    }
}

#[tokio::test]
async fn sqlite_persists_across_reopen() {
    // 临时文件 (WAL mode 验证)
    let dir = std::env::temp_dir().join(format!("star-taskqueue-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let db_path = dir.join("queue.db");
    let path_str = db_path.to_str().unwrap();

    // 1) 写入 3 个任务
    {
        let q = SqliteTaskQueue::new(path_str).unwrap();
        for kind in &["code_review", "doc_gen", "search"] {
            q.enqueue(make_task(kind)).await.unwrap();
        }
        assert_eq!(q.len().await.unwrap(), 3);
        // drop → 关连接
    }

    // 2) 重连 → 数据还在 (WAL persistence 验证)
    {
        let q = SqliteTaskQueue::new(path_str).unwrap();
        assert_eq!(q.len().await.unwrap(), 3, "WAL 应该保留 3 个任务");
        let t = q.dequeue().await.unwrap().expect("应该能 dequeue 1 个");
        assert_eq!(t.kind, "code_review", "FIFO: 第一个入队的先出");
        assert_eq!(t.state, TaskState::Dispatched);
    }

    // 3) 再重连 → state_history 也持久化了
    {
        let q = SqliteTaskQueue::new(path_str).unwrap();
        // 2 个 Pending + 1 个 Dispatched
        let dispatched = q.list_by_state(TaskState::Dispatched, 10).await.unwrap();
        assert_eq!(dispatched.len(), 1);
        assert_eq!(dispatched[0].kind, "code_review");
        // 状态历史有 1 条 (Pending → Dispatched)
        assert!(!dispatched[0].state_history.is_empty());
    }

    // 清理
    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn sqlite_in_memory_matches_inmemory_taskqueue() {
    // 验证 in_memory 跟持久化结果一致 (per G.1 PoC 实证)
    let q = SqliteTaskQueue::in_memory().unwrap();
    let task = make_task("test");
    q.enqueue(task.clone()).await.unwrap();
    let t = q.dequeue().await.unwrap().unwrap();
    assert_eq!(t.task_id, task.task_id);
    assert_eq!(t.state, TaskState::Dispatched);
}
