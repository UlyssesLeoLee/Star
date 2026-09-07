// SQLite WAL backend impl (per SRS-001 G-1, 守门 #13 a W 派生)
// Schema: task_queue W (短 TTL 作業中, completed 物理删除)
use rusqlite::{params, Connection, OptionalExtension};

use super::{AgentTask, TaskState, TaskStateTransition};

/// WAL checkpoint 间隔 (ms) — per 守门 #13 a 派生 (W 短 TTL)
pub(crate) const WAL_CHECKPOINT_INTERVAL_MS: u64 = 60_000;

/// 初始化 schema (idempotent)
pub(crate) fn init_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS task_queue (
            task_id          TEXT PRIMARY KEY NOT NULL,
            tenant_id        TEXT NOT NULL,
            kind             TEXT NOT NULL,
            payload          TEXT NOT NULL,
            idempotency_key  TEXT NOT NULL,
            created_at_ms    INTEGER NOT NULL,
            state            TEXT NOT NULL,
            state_history    TEXT NOT NULL,
            attempt          INTEGER NOT NULL DEFAULT 0
        );
        -- 索引: 派发按 (state, created_at_ms) FIFO 顺序
        CREATE INDEX IF NOT EXISTS idx_task_queue_state_created
            ON task_queue(state, created_at_ms);
        -- 索引: 租户隔离查询
        CREATE INDEX IF NOT EXISTS idx_task_queue_tenant
            ON task_queue(tenant_id);
        -- 索引: 幂等性 (per saga INV-SG-ORCH-03)
        CREATE INDEX IF NOT EXISTS idx_task_queue_idem
            ON task_queue(tenant_id, idempotency_key);
        "#,
    )?;
    Ok(())
}

pub(crate) fn insert_task(conn: &Connection, task: &AgentTask) -> rusqlite::Result<()> {
    let payload = serde_json::to_string(&task.payload)
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
    let history = serde_json::to_string(&task.state_history)
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
    let state = format!("{:?}", task.state);

    conn.execute(
        r#"INSERT INTO task_queue
           (task_id, tenant_id, kind, payload, idempotency_key, created_at_ms, state, state_history, attempt)
           VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"#,
        params![
            task.task_id.to_string(),
            task.tenant_id.to_string(),
            task.kind,
            payload,
            task.idempotency_key,
            task.created_at_ms as i64,
            state,
            history,
            task.attempt as i64,
        ],
    )?;
    Ok(())
}

pub(crate) fn dequeue_pending(conn: &Connection) -> rusqlite::Result<Option<AgentTask>> {
    // 取出最早的 Pending, 标记为 Dispatched
    let mut stmt = conn.prepare(
        r#"SELECT task_id, tenant_id, kind, payload, idempotency_key, created_at_ms, state_history, attempt
           FROM task_queue
           WHERE state = 'Pending'
           ORDER BY created_at_ms ASC
           LIMIT 1"#,
    )?;
    let row: Option<(String, String, String, String, String, i64, String, i64)> = stmt
        .query_row([], |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
                r.get(6)?,
                r.get(7)?,
            ))
        })
        .optional()?;

    let Some((task_id, tenant_id, kind, payload, idempotency_key, created_at_ms, history, attempt)) =
        row
    else {
        return Ok(None);
    };

    let mut task = AgentTask {
        task_id: uuid::Uuid::parse_str(&task_id).map_err(|e| {
            rusqlite::Error::InvalidColumnType(0, e.to_string(), rusqlite::types::Type::Text)
        })?,
        tenant_id: uuid::Uuid::parse_str(&tenant_id).map_err(|e| {
            rusqlite::Error::InvalidColumnType(1, e.to_string(), rusqlite::types::Type::Text)
        })?,
        kind,
        payload: serde_json::from_str(&payload).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        })?,
        idempotency_key,
        created_at_ms: created_at_ms as u64,
        state: TaskState::Pending,
        state_history: serde_json::from_str(&history).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        })?,
        attempt: attempt as u32,
    };

    // 标记为 Dispatched + 状态历史
    let from = task.state;
    task.state = TaskState::Dispatched;
    task.state_history.push(TaskStateTransition {
        from,
        to: TaskState::Dispatched,
        at_ms: now_ms(),
        reason: "dequeue".into(),
    });
    let new_state = format!("{:?}", task.state);
    let new_history = serde_json::to_string(&task.state_history)
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
    conn.execute(
        r#"UPDATE task_queue SET state = ?1, state_history = ?2 WHERE task_id = ?3"#,
        params![new_state, new_history, task.task_id.to_string()],
    )?;
    Ok(Some(task))
}

pub(crate) fn get_task(conn: &Connection, task_id: uuid::Uuid) -> rusqlite::Result<AgentTask> {
    let mut stmt = conn.prepare(
        r#"SELECT task_id, tenant_id, kind, payload, idempotency_key, created_at_ms, state, state_history, attempt
           FROM task_queue WHERE task_id = ?1"#,
    )?;
    stmt.query_row(params![task_id.to_string()], |r| {
        let task_id_str: String = r.get(0)?;
        let tenant_id_str: String = r.get(1)?;
        let kind: String = r.get(2)?;
        let payload: String = r.get(3)?;
        let idempotency_key: String = r.get(4)?;
        let created_at_ms: i64 = r.get(5)?;
        let state: String = r.get(6)?;
        let history: String = r.get(7)?;
        let attempt: i64 = r.get(8)?;

        Ok(AgentTask {
            task_id: uuid::Uuid::parse_str(&task_id_str).unwrap_or(uuid::Uuid::nil()),
            tenant_id: uuid::Uuid::parse_str(&tenant_id_str).unwrap_or(uuid::Uuid::nil()),
            kind,
            payload: serde_json::from_str(&payload).unwrap_or(serde_json::Value::Null),
            idempotency_key,
            created_at_ms: created_at_ms as u64,
            state: parse_state(&state),
            state_history: serde_json::from_str(&history).unwrap_or_default(),
            attempt: attempt as u32,
        })
    })
}

pub(crate) fn list_by_state(
    conn: &Connection,
    state: TaskState,
    limit: u32,
) -> rusqlite::Result<Vec<AgentTask>> {
    let state_str = format!("{:?}", state);
    let mut stmt = conn.prepare(
        r#"SELECT task_id, tenant_id, kind, payload, idempotency_key, created_at_ms, state, state_history, attempt
           FROM task_queue WHERE state = ?1 ORDER BY created_at_ms ASC LIMIT ?2"#,
    )?;
    let rows = stmt.query_map(params![state_str, limit as i64], |r| {
        let task_id_str: String = r.get(0)?;
        let tenant_id_str: String = r.get(1)?;
        let kind: String = r.get(2)?;
        let payload: String = r.get(3)?;
        let idempotency_key: String = r.get(4)?;
        let created_at_ms: i64 = r.get(5)?;
        let state: String = r.get(6)?;
        let history: String = r.get(7)?;
        let attempt: i64 = r.get(8)?;
        Ok(AgentTask {
            task_id: uuid::Uuid::parse_str(&task_id_str).unwrap_or(uuid::Uuid::nil()),
            tenant_id: uuid::Uuid::parse_str(&tenant_id_str).unwrap_or(uuid::Uuid::nil()),
            kind,
            payload: serde_json::from_str(&payload).unwrap_or(serde_json::Value::Null),
            idempotency_key,
            created_at_ms: created_at_ms as u64,
            state: parse_state(&state),
            state_history: serde_json::from_str(&history).unwrap_or_default(),
            attempt: attempt as u32,
        })
    })?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

pub(crate) fn count_all(conn: &Connection) -> rusqlite::Result<u64> {
    let n: i64 = conn.query_row(r#"SELECT COUNT(*) FROM task_queue"#, [], |r| r.get(0))?;
    Ok(n as u64)
}

pub(crate) fn transition_state(
    conn: &Connection,
    task_id: uuid::Uuid,
    from: TaskState,
    to: TaskState,
    reason: &str,
) -> rusqlite::Result<AgentTask> {
    let mut task = get_task(conn, task_id)?;
    if task.state != from {
        return Err(rusqlite::Error::InvalidQuery);
    }
    let prev = task.state;
    task.state = to;
    task.state_history.push(TaskStateTransition {
        from: prev,
        to,
        at_ms: now_ms(),
        reason: reason.to_string(),
    });
    if matches!(to, TaskState::Failed) {
        task.attempt = task.attempt.saturating_add(1);
    }
    let state_str = format!("{:?}", task.state);
    let history = serde_json::to_string(&task.state_history)
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
    conn.execute(
        r#"UPDATE task_queue SET state = ?1, state_history = ?2, attempt = ?3 WHERE task_id = ?4"#,
        params![
            state_str,
            history,
            task.attempt as i64,
            task.task_id.to_string()
        ],
    )?;
    Ok(task)
}

pub(crate) fn cleanup_completed(conn: &Connection, older_than_ms: u64) -> rusqlite::Result<u64> {
    let n = conn.execute(
        r#"DELETE FROM task_queue
           WHERE state IN ('Completed', 'Aborted', 'Failed')
             AND created_at_ms < ?1"#,
        params![older_than_ms as i64],
    )?;
    // WAL checkpoint
    let _ = conn.pragma_update(None, "wal_checkpoint", "PASSIVE");
    Ok(n as u64)
}

fn parse_state(s: &str) -> TaskState {
    match s {
        "Pending" => TaskState::Pending,
        "Dispatched" => TaskState::Dispatched,
        "Running" => TaskState::Running,
        "Completed" => TaskState::Completed,
        "Failed" => TaskState::Failed,
        "Aborted" => TaskState::Aborted,
        _ => TaskState::Pending,
    }
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
