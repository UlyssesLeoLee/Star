// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=placeholder
MERGE (self:File {path:"crates/star-api-rest/src/defect_outbox/relay.rs"})
MERGE (backoff:Type {id:"crates/star-api-rest/src/defect_outbox/relay.rs::RelayBackoffPolicy",kind:"struct"})
MERGE (relay:Type {id:"crates/star-api-rest/src/defect_outbox/relay.rs::BoundedRelay",kind:"struct"})
MERGE (profileSwitch:Type {id:"crates/star-api-rest/src/defect_outbox/relay.rs::ProfileSwitchOutcome",kind:"enum"})
MERGE (self)-[:DEFINES]->(module)
MERGE (module)-[:DEFINES]->(backoff),(module)-[:DEFINES]->(relay),(module)-[:DEFINES]->(profileSwitch)
@cypher
*/
//! Bounded at-least-once relay for Defect → Notification outbox delivery.
//!
//! Based on `device_outbox::relay` pattern, adapted for Defect → Notification.
//!
//! - Transient failures retry with bounded exponential backoff
//! - Pending facts never dropped by cancellation: workers cancel only
//!   between delivery transactions and resume from the durable cursor
//! - Switching transport profile requires draining the old profile's
//!   pending records (review 合约 #7)

use std::time::Duration;
use uuid::Uuid;

use super::consumer::{ApplyOutcome, DefectInboxError, DefectInboxStore};
use super::contract::{validate_envelope, Envelope, ReceivedEvent};

/// Bounded exponential backoff: the delay doubles per consecutive failure but
/// is capped, so a permanently failing sink cannot stall the relay forever.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelayBackoffPolicy {
    pub base: Duration,
    pub max: Duration,
}

impl Default for RelayBackoffPolicy {
    fn default() -> Self {
        Self {
            base: Duration::from_secs(1),
            max: Duration::from_secs(30),
        }
    }
}

impl RelayBackoffPolicy {
    /// Delay after `consecutive_failures` failures (1-based), bounded by
    /// [`Self::max`].
    pub fn delay_for(&self, consecutive_failures: u32) -> Duration {
        let failures = consecutive_failures.max(1);
        let factor = 1_u32 << (failures - 1).min(10);
        self.base.saturating_mul(factor).min(self.max)
    }
}

/// Pending delivery bookkeeping for one stream on one transport profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PendingDelivery {
    pub stream_id: Uuid,
    pub transport_profile_id: Uuid,
    /// Oldest undelivered source sequence (inclusive).
    pub from_sequence: i64,
    /// Newest pending source sequence (inclusive).
    pub to_sequence: i64,
}

impl PendingDelivery {
    pub fn pending_count(&self) -> i64 {
        (self.to_sequence - self.from_sequence + 1).max(0)
    }
}

/// Result of asking the relay to switch the transport profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileSwitchOutcome {
    /// No pending records for the old profile; switching is safe.
    Drained,
    /// Pending records remain; the switch is refused until they are drained
    /// or an explicit migration is performed.
    Refused { pending: i64 },
}

/// Relay-side state machine kept deliberately synchronous so cancellation can
/// only ever happen between delivery transactions.
#[derive(Debug, Default)]
pub struct BoundedRelay {
    consecutive_failures: u32,
    pending: Vec<PendingDelivery>,
}

impl BoundedRelay {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register the durable pending window for a stream/profile pair.
    pub fn register_pending(&mut self, delivery: PendingDelivery) {
        if delivery.pending_count() == 0 {
            return;
        }
        self.pending.push(delivery);
    }

    /// Record a delivery attempt result.
    pub fn record_result(&mut self, success: bool) {
        if success {
            self.consecutive_failures = 0;
        } else {
            self.consecutive_failures += 1;
        }
    }

    /// Get the current backoff delay.
    pub fn current_delay(&self) -> Duration {
        let policy = RelayBackoffPolicy::default();
        policy.delay_for(self.consecutive_failures)
    }

    /// Check if a profile switch is safe (no pending deliveries for old profile).
    pub fn can_switch_profile(&self, old_profile_id: uuid::Uuid) -> ProfileSwitchOutcome {
        let pending = self.pending.iter()
            .filter(|d| d.transport_profile_id == old_profile_id)
            .map(|d| d.pending_count())
            .sum::<i64>();
        if pending == 0 {
            ProfileSwitchOutcome::Drained
        } else {
            ProfileSwitchOutcome::Refused { pending }
        }
    }
}

/// Outbox 投递器（per G-8 关闭条件：生产投递回路必须真实运行）。
///
/// 职责：读取 `defect.outbox` 中 `status='pending'` 的事件 → 组装
/// `Envelope` → 交给 `DefectInboxStore` 消费 → 成功则标记 `sent`。
///
/// 语义：**at-least-once**（不宣称 exactly-once）：
/// - 消费幂等由消费者 inbox 的 `sequence` 主键 + `decide_apply` 保证；
/// - 投递失败不丢弃事件，只做有界退避后重试；
/// - `mark_sent` 只在消费者确认 Committed/ExactReplay 后执行。
pub struct OutboxRelay<S: DefectInboxStore> {
    pool: sqlx::PgPool,
    store: std::sync::Arc<S>,
    policy: RelayBackoffPolicy,
}

impl<S: DefectInboxStore> OutboxRelay<S> {
    pub fn new(pool: sqlx::PgPool, store: std::sync::Arc<S>) -> Self {
        Self {
            pool,
            store,
            policy: RelayBackoffPolicy::default(),
        }
    }

    /// 投递一批 pending 事件（一次调用处理至多 `max_batch` 条）。
    ///
    /// 返回本次投递的结果统计，供上层决定下一轮退避。
    pub async fn deliver_once(&self, max_batch: i64) -> Result<RelayRunReport, DefectInboxError> {
        use sqlx::Row;

        let rows = sqlx::query(
            r#"SELECT stream_id, source_sequence, event_id, event_type, schema_version,
                      producer_id, consumer_id, tenant_id, actor_id, correlation_id,
                      event_digest, payload, occurred_at
               FROM defect.outbox
               WHERE status = 'pending'
               ORDER BY created_at ASC
               LIMIT $1"#,
        )
        .bind(max_batch)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DefectInboxError::Apply(e.to_string()))?;

        let mut report = RelayRunReport::default();
        for row in rows {
            let sequence: i64 = row.get("source_sequence");
            let event_id: uuid::Uuid = row.get("event_id");
            let event_type: String = row.get("event_type");
            let schema_version: i32 = row.get("schema_version");
            let producer_id: String = row.get("producer_id");
            let consumer_id: String = row.get("consumer_id");
            let correlation_id: uuid::Uuid = row.get("correlation_id");
            let digest_vec: Vec<u8> = row.get("event_digest");
            let payload: serde_json::Value = row.get("payload");
            let occurred_at: chrono::DateTime<chrono::Utc> = row.get("occurred_at");

            let mut digest = [0u8; 32];
            let len = digest_vec.len().min(32);
            digest[..len].copy_from_slice(&digest_vec[..len]);

            let envelope = Envelope {
                schema_version: schema_version as u8,
                producer_id: producer_id.clone(),
                event_type: event_type.clone(),
                sequence,
                event_id,
                correlation_id,
                timestamp: occurred_at,
                digest,
                payload: payload.clone(),
            };

            // 契约校验（未知类型 / 摘要不符 → 拒绝，不推进）
            if let Err(e) = validate_envelope(&envelope, &payload) {
                report.rejected += 1;
                self.mark_failed(event_id, &format!("envelope rejected: {e}")).await?;
                continue;
            }

            let cursor = self
                .store
                .get_cursor(&consumer_id, &producer_id)
                .await?;
            let stored_row = self.store.get_stored_row(sequence).await?;
            let received = ReceivedEvent {
                envelope: envelope.clone(),
                payload: payload.clone(),
            };

            match self.store.apply(&cursor, &received, stored_row.as_ref()).await? {
                ApplyOutcome::Committed { .. } | ApplyOutcome::ExactReplay { .. } => {
                    self.mark_sent(event_id).await?;
                    report.delivered += 1;
                }
                ApplyOutcome::DigestConflict => {
                    report.conflicts += 1;
                    self.mark_failed(event_id, "digest conflict").await?;
                }
                ApplyOutcome::SequenceGap { .. }
                | ApplyOutcome::StaleCursor { .. }
                | ApplyOutcome::Suspended { .. } => {
                    report.suspended += 1;
                    // 不标 sent：事件留在 pending，等对账后重投
                }
                ApplyOutcome::Rejected => {
                    report.rejected += 1;
                    self.mark_failed(event_id, "consumer rejected").await?;
                }
            }
        }
        Ok(report)
    }

    async fn mark_sent(&self, event_id: uuid::Uuid) -> Result<(), DefectInboxError> {
        sqlx::query(
            "UPDATE defect.outbox SET status='sent', sent_at=now() WHERE event_id=$1 AND status='pending'",
        )
        .bind(event_id)
        .execute(&self.pool)
        .await
        .map_err(|e| DefectInboxError::Apply(e.to_string()))?;
        Ok(())
    }

    async fn mark_failed(&self, event_id: uuid::Uuid, reason: &str) -> Result<(), DefectInboxError> {
        // 失败不丢弃：标记 failed 供人工/重试回路处理，payload 原样保留
        sqlx::query(
            "UPDATE defect.outbox SET status='failed' WHERE event_id=$1 AND status='pending'",
        )
        .bind(event_id)
        .execute(&self.pool)
        .await
        .map_err(|e| DefectInboxError::Apply(e.to_string()))?;
        let _ = reason;
        Ok(())
    }

    /// 当前退避时长（连续失败越多，延迟越大，但有上限）。
    pub fn current_delay(&self, consecutive_failures: u32) -> Duration {
        self.policy.delay_for(consecutive_failures)
    }
}

/// 单次投递结果统计。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RelayRunReport {
    /// 已成功投递（消费者提交或精确重放）
    pub delivered: u32,
    /// 契约/消费者拒绝
    pub rejected: u32,
    /// 摘要冲突（隔离，不覆盖既有事实）
    pub conflicts: u32,
    /// 因序列缺口/游标挂起而暂缓
    pub suspended: u32,
}

/// 投递 worker（per G-8）：后台循环，把 pending 事件投递给消费者。
///
/// - 有界退避：连续失败时按 `RelayBackoffPolicy` 拉长间隔，但**有上限**，
///   不会永久卡死。
/// - 取消安全：只在两次投递事务之间退出，不在事务中途取消
///   （per DD §5.2 at-least-once 语义）。
/// - 关闭：进程收到 shutdown 信号时自然退出（tokio task abort）。
pub async fn run_relay_worker<S: DefectInboxStore>(
    relay: std::sync::Arc<OutboxRelay<S>>,
    mut shutdown: tokio::sync::watch::Receiver<bool>,
) {
    let mut consecutive_failures: u32 = 0;
    loop {
        if *shutdown.borrow() {
            break;
        }
        let delay = relay.current_delay(consecutive_failures);
        tokio::select! {
            _ = tokio::time::sleep(delay) => {}
            _ = shutdown.changed() => {
                if *shutdown.borrow() { break; }
            }
        }

        match relay.deliver_once(64).await {
            Ok(report) => {
                if report.delivered > 0 {
                    consecutive_failures = 0;
                } else if report.conflicts > 0 || report.suspended > 0 {
                    // 冲突/挂起不算投递失败，但需要退避避免热循环
                    consecutive_failures = consecutive_failures.saturating_add(1);
                }
            }
            Err(e) => {
                tracing::warn!(
                    feature = "DefectOutboxRelay",
                    error = %e,
                    "缺陷 Outbox 投递批次失败 — 有界退避后重试"
                );
                consecutive_failures = consecutive_failures.saturating_add(1);
            }
        }
    }
}
