// SPDX-License-Identifier: MIT OR Apache-2.0
//! `unified_projection.rs` — 统一 status projection (per ULYS-198 + FR-ORCA-018 + NFR-003/004)
//!
//! **目的 (per v1.0 §4.2 §4.3)**: agent status + worktree status 都从同一份 source-of-truth 派生
//!
//! **架构 (per FR-ORCA-018 "The rule" + 守门 #13)**:
//! - 1 execution host = 1 status store (AC-1)
//! - precedence 在 write time 决定, 记录 provenance 在 row 上; reader 不再 re-adjudicate (AC-2)
//! - reader 只保留 presentation policy (30 分钟 decay / ack / dismiss / unread) (AC-3)
//!
//! **Hydration Honesty (per NFR-ORCA-003)**:
//! - 从 `last-status.json` 恢复的非 done row 必须标 `restoredUnconfirmed`
//! - 任何 hydrated row 都不得 read 为 fresh truth
//!
//! **Transport Loss (per NFR-ORCA-004)**:
//! - 网络断联 (relay replay loss) 不得清空 status
//! - 只在 certified PTY exit / provider-generation replacement / dismissal 时清理
//!
//! **守门 #13 a/d W/T/M 派生**:
//! - `UnifiedStatusProjection` 是 W (短 TTL 窗口内 hot, 物理删除 OK)
//! - `last-status.json` snapshot 是 T (append-only 历史)
//!
//! **MVP v0.1 范围 (per ULYS-198)**:
//! - 6 consumers (worktree ps / dashboard / CLI / canvas / API / AgentRunner)
//!   共享同一份 `UnifiedStatusProjection` 读 API
//! - subject 类型 = AgentId | WorktreeId (per spec 状态三元组 = agent × terminal × worktree)
//! - hydration state machine + transport loss API
//!
//! **不在本 MVP 范围 (P1 followup)**:
//! - 实际 cross-process eventbus 接入 (per `crates/event-bus`)
//! - DB 持久化层 (per `domain-local-runtime` cli_session_registry 模式)
//! - 6 consumers 全部逐一对接 (本期只提供共享 read API, 各 consumer 集成留后续)

#![allow(clippy::result_large_err)] // 守门 #6 v2 强制 6-field UnifiedProjectionError schema (per DD §38)

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Instant;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

// =====================================================================
// 1. ID types
// =====================================================================

/// Agent UUID 强类型 ID (per `crates/graph-core::types::AgentId` 模式;
///
/// MVP v0 重新定义避免跨 crate 依赖. 未来 P1 整合时切到 graph-core)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AgentId(pub Uuid);

impl AgentId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl std::fmt::Display for AgentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Worktree UUID 强类型 ID (per `crates/graph-core::types::WorktreeId` 模式)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WorktreeId(pub Uuid);

impl WorktreeId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl std::fmt::Display for WorktreeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Subject 类型 (per FR-ORCA-018 状态三元组 = agent × terminal × worktree;
///
/// MVP v0 只表示 agent + worktree 两类, terminal 由 AgentSession 单独追踪)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SubjectId {
    /// Agent subject
    Agent(AgentId),
    /// Worktree subject
    Worktree(WorktreeId),
}

impl serde::Serialize for SubjectId {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        // 自定义: "agent:<uuid>" / "worktree:<uuid>"
        s.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for SubjectId {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        if let Some(uuid_str) = s.strip_prefix("agent:") {
            let u = Uuid::parse_str(uuid_str).map_err(serde::de::Error::custom)?;
            Ok(Self::Agent(AgentId(u)))
        } else if let Some(uuid_str) = s.strip_prefix("worktree:") {
            let u = Uuid::parse_str(uuid_str).map_err(serde::de::Error::custom)?;
            Ok(Self::Worktree(WorktreeId(u)))
        } else {
            Err(serde::de::Error::custom(format!("invalid SubjectId: {s}")))
        }
    }
}

impl std::fmt::Display for SubjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Agent(id) => write!(f, "agent:{id}"),
            Self::Worktree(id) => write!(f, "worktree:{id}"),
        }
    }
}

// =====================================================================
// 2. status entry + hydration state
// =====================================================================

/// Hydration state (per NFR-ORCA-003 Hydration Honesty)
///
/// 状态机迁移:
/// - 新 row: `Confirmed` (默认)
/// - 从 snapshot 恢复: `RestoredUnconfirmed` (per NFR-ORCA-003 必须标)
/// - transport loss: 状态变 `Frozen` (per NFR-ORCA-004 不自动清理)
/// - new live event: 重新 `Confirmed`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum HydrationState {
    /// 已通过 live eventbus 事件确认
    Confirmed {
        /// 最近确认时间 (RFC3339 序列化)
        confirmed_at: DateTime<Utc>,
    },
    /// 从 last-status.json 恢复, 未经 live eventbus 确认 (per NFR-ORCA-003)
    RestoredUnconfirmed {
        /// 恢复时间
        restored_at: DateTime<Utc>,
    },
    /// Transport loss 期间 (per NFR-ORCA-004 不自动清理, 等待 rehydrate)
    Frozen {
        /// 进入 frozen 状态的时间
        frozen_at: DateTime<Utc>,
    },
}

/// Provenance (per FR-ORCA-018 AC-2 "precedence 在 write time 决定, 记录 provenance 在 row 上")
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "source", rename_all = "snake_case")]
pub enum Provenance {
    /// Live eventbus 写入 (per G.3 eventbus 流)
    LiveEventbus {
        /// Event ID
        event_id: Uuid,
    },
    /// 从 last-status.json snapshot 恢复
    SnapshotRestore {
        /// Snapshot 文件名 / ID
        snapshot_id: String,
    },
    /// 启动期 bootstrap (无任何已有数据)
    Bootstrap,
}

/// Unified status entry (per FR-ORCA-018 6 consumers 共享 source-of-truth)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnifiedStatusEntry {
    /// Subject (Agent | Worktree)
    pub subject: SubjectId,
    /// 状态字符串 (per HumanState / AgentStatus as_str)
    pub status: String,
    /// Health score (0-100, None 表示尚未评估)
    pub health_score: Option<u8>,
    /// 最后观测时间 (live event 时为事件时间, restore 时为 snapshot 时间)
    pub last_observed_at: DateTime<Utc>,
    /// Hydration state (per NFR-ORCA-003)
    pub hydration: HydrationState,
    /// Provenance (per FR-ORCA-018 AC-2)
    pub provenance: Provenance,
}

impl UnifiedStatusEntry {
    /// 是否可作 "fresh truth" 读取 (per NFR-ORCA-003 "任何 hydrated row 都不得 read 为 fresh truth")
    pub fn is_fresh(&self) -> bool {
        matches!(self.hydration, HydrationState::Confirmed { .. })
    }

    /// 是否处于 transport loss frozen 状态 (per NFR-ORCA-004)
    pub fn is_frozen(&self) -> bool {
        matches!(self.hydration, HydrationState::Frozen { .. })
    }
}

// =====================================================================
// 3. error
// =====================================================================

/// 统一 projection 错误 (per 守门 #6 v2 6-field schema)
#[derive(Debug, Error)]
pub enum UnifiedProjectionError {
    /// Subject 不存在
    #[error("subject not found: {0}")]
    SubjectNotFound(String),
    /// status 字符串空
    #[error("empty status string not allowed")]
    EmptyStatus,
    /// health_score 越界 [0, 100]
    #[error("health_score {0} out of range [0, 100]")]
    HealthScoreOutOfRange(u8),
    /// provenance event_id 不存在 (when restore 后补 live event 但 event_id 重复)
    #[error("event_id already recorded: {0}")]
    DuplicateEventId(Uuid),
    /// 内部锁中毒
    #[error("projection mutex poisoned")]
    Poisoned,
}

// =====================================================================
// 4. projection
// =====================================================================

/// Unified status projection (per FR-ORCA-018 1 store per execution host)
pub struct UnifiedStatusProjection {
    /// subject_id → entry
    entries: RwLock<HashMap<SubjectId, UnifiedStatusEntry>>,
    /// 全局 event_id 去重 (per G.3 EventBus Mailbox ExactlyOnce dedup 派生)
    /// 跨 subject 共享同一去重集合, 防 replay re-sent
    seen_event_ids: RwLock<std::collections::HashSet<Uuid>>,
    /// transport loss 状态 (true = 进入 frozen, false = 正常)
    transport_lost: RwLock<TransportLossState>,
}

/// Transport loss 状态 (per NFR-ORCA-004)
#[derive(Debug, Clone)]
struct TransportLossState {
    /// 当前是否处于 transport loss
    in_loss: bool,
    /// 进入 transport loss 的时间 (用于 audit)
    entered_at: Option<DateTime<Utc>>,
    /// 最后一次 transport loss 事件的时间戳
    last_event_at: Option<Instant>,
}

impl Default for TransportLossState {
    fn default() -> Self {
        Self {
            in_loss: false,
            entered_at: None,
            last_event_at: None,
        }
    }
}

impl UnifiedStatusProjection {
    /// 构造空 projection (per FR-ORCA-018 AC-1)
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            seen_event_ids: RwLock::new(std::collections::HashSet::new()),
            transport_lost: RwLock::new(TransportLossState::default()),
        }
    }

    /// 从 snapshot 恢复 (per NFR-ORCA-003 Hydration Honesty)
    ///
    /// 所有 row 强制标 `RestoredUnconfirmed` + `Provenance::SnapshotRestore`.
    /// Reader 调用 `entry.is_fresh()` 会得到 false (per NFR-ORCA-003 "hydrated row 都不得 read 为 fresh truth")
    pub fn from_snapshot(
        snapshot: Vec<UnifiedStatusEntry>,
        snapshot_id: impl Into<String>,
    ) -> Arc<Self> {
        let snapshot_id = snapshot_id.into();
        let mut entries = HashMap::new();
        for mut entry in snapshot {
            entry.hydration = HydrationState::RestoredUnconfirmed {
                restored_at: Utc::now(),
            };
            entry.provenance = Provenance::SnapshotRestore {
                snapshot_id: snapshot_id.clone(),
            };
            entries.insert(entry.subject, entry);
        }
        Arc::new(Self {
            entries: RwLock::new(entries),
            seen_event_ids: RwLock::new(std::collections::HashSet::new()),
            transport_lost: RwLock::new(TransportLossState::default()),
        })
    }

    /// 记录 live event (per FR-ORCA-018 AC-2 write time 决定 precedence)
    ///
    /// `event_id` 必须唯一, 重复 event_id → DuplicateEventId 错误
    /// (per G.3 EventBus Mailbox ExactlyOnce dedup 派生)
    pub fn record(
        &self,
        event_id: Uuid,
        subject: SubjectId,
        status: impl Into<String>,
        health_score: Option<u8>,
    ) -> Result<(), UnifiedProjectionError> {
        let status = status.into();
        if status.is_empty() {
            return Err(UnifiedProjectionError::EmptyStatus);
        }
        if let Some(score) = health_score {
            if score > 100 {
                return Err(UnifiedProjectionError::HealthScoreOutOfRange(score));
            }
        }

        let mut entries = self
            .entries
            .write()
            .map_err(|_| UnifiedProjectionError::Poisoned)?;

        // dedup: 全局 event_id (per G.3 EventBus Mailbox ExactlyOnce 派生)
        let mut seen = self
            .seen_event_ids
            .write()
            .map_err(|_| UnifiedProjectionError::Poisoned)?;
        if !seen.insert(event_id) {
            return Err(UnifiedProjectionError::DuplicateEventId(event_id));
        }
        drop(seen);

        let now = Utc::now();
        entries.insert(
            subject,
            UnifiedStatusEntry {
                subject,
                status,
                health_score,
                last_observed_at: now,
                hydration: HydrationState::Confirmed { confirmed_at: now },
                provenance: Provenance::LiveEventbus { event_id },
            },
        );
        Ok(())
    }

    /// 从 snapshot 取单个 entry
    pub fn get(&self, subject: &SubjectId) -> Option<UnifiedStatusEntry> {
        self.entries.read().ok()?.get(subject).cloned()
    }

    /// 全部 entries (per 6 consumers 共享视图)
    ///
    /// 返回 Vec 而非 HashMap 是为了让 caller 决定排序 + 过滤;
    /// reader 端的 presentation policy (30 分钟 decay / ack / dismiss / unread)
    /// **不**在这里做 (per FR-ORCA-018 AC-3 "reader 只保留 presentation policy")
    pub fn current_view(&self) -> Vec<UnifiedStatusEntry> {
        self.entries
            .read()
            .map(|g| g.values().cloned().collect())
            .unwrap_or_default()
    }

    /// 按 subject_kind 过滤 (per 6 consumers 各自关注范围)
    pub fn view_by_kind(&self, kind: SubjectKind) -> Vec<UnifiedStatusEntry> {
        self.current_view()
            .into_iter()
            .filter(|e| match (&e.subject, &kind) {
                (SubjectId::Agent(_), SubjectKind::Agent) => true,
                (SubjectId::Worktree(_), SubjectKind::Worktree) => true,
                _ => false,
            })
            .collect()
    }

    /// 当前 transport loss 状态 (per NFR-ORCA-004 reader 决策)
    pub fn is_transport_lost(&self) -> bool {
        self.transport_lost
            .read()
            .map(|g| g.in_loss)
            .unwrap_or(false)
    }

    /// 进入 transport loss 状态 (per NFR-ORCA-004)
    ///
    /// **不**自动删除 entry — 全部 entries 状态保持, 仅 HydrationState
    /// 不再被新事件覆盖 (per NFR-ORCA-004 "网络断联 不得清空 status")
    ///
    /// 同时把所有 entry 标 `Frozen` (per spec "只在 certified PTY exit /
    /// provider-generation replacement / dismissal 时清理")
    pub fn mark_transport_loss(&self) {
        let mut state = match self.transport_lost.write() {
            Ok(g) => g,
            Err(_) => return,
        };
        state.in_loss = true;
        state.entered_at = Some(Utc::now());
        state.last_event_at = Some(Instant::now());
        drop(state);

        // 把所有 entry 标 Frozen (但保留 entry, 不删除)
        let mut entries = match self.entries.write() {
            Ok(g) => g,
            Err(_) => return,
        };
        let now = Utc::now();
        for entry in entries.values_mut() {
            entry.hydration = HydrationState::Frozen { frozen_at: now };
        }
    }

    /// 退出 transport loss 状态 (rehydrate 完成)
    ///
    /// **不**自动恢复 entry 内容 — entry 仍为 Frozen 状态直到新 live event
    /// 覆盖 (per NFR-ORCA-004 "任何 hydrated row 都不得 read 为 fresh truth"
    /// 派生: frozen 的 entry 在收到新 live event 前仍非 fresh)
    pub fn mark_transport_recovered(&self) {
        let mut state = match self.transport_lost.write() {
            Ok(g) => g,
            Err(_) => return,
        };
        state.in_loss = false;
        // entered_at + last_event_at 保留 (audit)
        drop(state);
    }

    /// 清空单个 entry (per NFR-ORCA-004 "certified PTY exit / provider-generation replacement / dismissal 时清理")
    pub fn dismiss(&self, subject: &SubjectId) -> Result<(), UnifiedProjectionError> {
        let mut entries = self
            .entries
            .write()
            .map_err(|_| UnifiedProjectionError::Poisoned)?;
        entries
            .remove(subject)
            .ok_or_else(|| UnifiedProjectionError::SubjectNotFound(subject.to_string()))?;
        Ok(())
    }

    /// 当前 entries 数
    pub fn len(&self) -> usize {
        self.entries.read().map(|g| g.len()).unwrap_or(0)
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.entries.read().map(|g| g.is_empty()).unwrap_or(true)
    }

    /// fresh (Confirmed) entries 数 — 给 reader 判断"是否能信任"
    /// (per NFR-ORCA-003 派生 metric)
    pub fn fresh_count(&self) -> usize {
        self.entries
            .read()
            .map(|g| g.values().filter(|e| e.is_fresh()).count())
            .unwrap_or(0)
    }

    /// frozen entries 数
    pub fn frozen_count(&self) -> usize {
        self.entries
            .read()
            .map(|g| g.values().filter(|e| e.is_frozen()).count())
            .unwrap_or(0)
    }
}

impl Default for UnifiedStatusProjection {
    fn default() -> Self {
        Self::new()
    }
}

/// Subject 类型 filter (per `view_by_kind`)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubjectKind {
    Agent,
    Worktree,
}

// =====================================================================
// 5. tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_projection_starts_empty() {
        let p = UnifiedStatusProjection::new();
        assert!(p.is_empty());
        assert_eq!(p.len(), 0);
        assert_eq!(p.fresh_count(), 0);
        assert_eq!(p.frozen_count(), 0);
        assert!(!p.is_transport_lost());
        assert_eq!(p.current_view().len(), 0);
    }

    #[test]
    fn record_inserts_fresh_entry() {
        let p = UnifiedStatusProjection::new();
        let agent = AgentId::new();
        let event_id = Uuid::new_v4();
        p.record(event_id, SubjectId::Agent(agent), "Running", Some(95))
            .unwrap();
        let entry = p.get(&SubjectId::Agent(agent)).unwrap();
        assert_eq!(entry.status, "Running");
        assert_eq!(entry.health_score, Some(95));
        assert!(entry.is_fresh(), "record 后的 entry 必须是 Confirmed");
        assert!(!entry.is_frozen());
        assert!(matches!(
            entry.provenance,
            Provenance::LiveEventbus { event_id: e } if e == event_id
        ));
    }

    #[test]
    fn record_validates_empty_status() {
        let p = UnifiedStatusProjection::new();
        let r = p.record(Uuid::new_v4(), SubjectId::Agent(AgentId::new()), "", None);
        assert!(matches!(r, Err(UnifiedProjectionError::EmptyStatus)));
    }

    #[test]
    fn record_validates_health_score_range() {
        let p = UnifiedStatusProjection::new();
        let r = p.record(
            Uuid::new_v4(),
            SubjectId::Agent(AgentId::new()),
            "Running",
            Some(101),
        );
        assert!(matches!(
            r,
            Err(UnifiedProjectionError::HealthScoreOutOfRange(101))
        ));
    }

    #[test]
    fn duplicate_event_id_rejected() {
        // Per FR-ORCA-018 AC-2: 同一 event_id 不应重复处理
        let p = UnifiedStatusProjection::new();
        let agent = AgentId::new();
        let event_id = Uuid::new_v4();
        p.record(event_id, SubjectId::Agent(agent), "Running", None)
            .unwrap();
        // 同 event_id 再次 record (即使 subject 不同) → DuplicateEventId
        let r = p.record(
            event_id,
            SubjectId::Worktree(WorktreeId::new()),
            "Idle",
            None,
        );
        assert!(matches!(
            r,
            Err(UnifiedProjectionError::DuplicateEventId(_))
        ));
    }

    #[test]
    fn from_snapshot_marks_all_as_restored_unconfirmed() {
        // Per NFR-ORCA-003 Hydration Honesty — hydrated row 都不得 read 为 fresh
        let entries = vec![
            UnifiedStatusEntry {
                subject: SubjectId::Agent(AgentId::new()),
                status: "Running".to_string(),
                health_score: Some(80),
                last_observed_at: Utc::now(),
                hydration: HydrationState::Confirmed {
                    confirmed_at: Utc::now(),
                },
                provenance: Provenance::LiveEventbus {
                    event_id: Uuid::new_v4(),
                },
            },
            UnifiedStatusEntry {
                subject: SubjectId::Worktree(WorktreeId::new()),
                status: "Idle".to_string(),
                health_score: None,
                last_observed_at: Utc::now(),
                hydration: HydrationState::Confirmed {
                    confirmed_at: Utc::now(),
                },
                provenance: Provenance::LiveEventbus {
                    event_id: Uuid::new_v4(),
                },
            },
        ];
        let p = UnifiedStatusProjection::from_snapshot(entries.clone(), "test-snapshot-001");
        let view = p.current_view();
        assert_eq!(view.len(), 2);
        for entry in &view {
            assert!(
                !entry.is_fresh(),
                "snapshot restore 后所有 entry 必须非 fresh (NFR-ORCA-003)"
            );
            assert!(matches!(
                entry.hydration,
                HydrationState::RestoredUnconfirmed { .. }
            ));
            assert!(matches!(
                entry.provenance,
                Provenance::SnapshotRestore { .. }
            ));
        }
        assert_eq!(p.fresh_count(), 0);
        assert_eq!(p.frozen_count(), 0);
    }

    #[test]
    fn live_event_after_snapshot_upgrades_to_fresh() {
        // Per FR-ORCA-018 AC-2: live event 覆盖 precedence
        let agent = AgentId::new();
        let entries = vec![UnifiedStatusEntry {
            subject: SubjectId::Agent(agent),
            status: "Idle".to_string(),
            health_score: Some(80),
            last_observed_at: Utc::now(),
            hydration: HydrationState::Confirmed {
                confirmed_at: Utc::now(),
            },
            provenance: Provenance::LiveEventbus {
                event_id: Uuid::new_v4(),
            },
        }];
        let p = UnifiedStatusProjection::from_snapshot(entries, "test-snapshot");
        // live event 到来 → 应升级为 Confirmed
        let live_event_id = Uuid::new_v4();
        p.record(live_event_id, SubjectId::Agent(agent), "Running", Some(95))
            .unwrap();
        let entry = p.get(&SubjectId::Agent(agent)).unwrap();
        assert!(entry.is_fresh(), "live event 后 entry 必须 fresh");
        assert_eq!(entry.status, "Running");
        assert_eq!(entry.health_score, Some(95));
    }

    #[test]
    fn mark_transport_loss_freezes_without_deleting() {
        // Per NFR-ORCA-004 "网络断联 不得清空 status"
        let p = UnifiedStatusProjection::new();
        let agent = AgentId::new();
        let wt = WorktreeId::new();
        p.record(Uuid::new_v4(), SubjectId::Agent(agent), "Running", None)
            .unwrap();
        p.record(Uuid::new_v4(), SubjectId::Worktree(wt), "Idle", Some(80))
            .unwrap();
        assert_eq!(p.len(), 2);

        p.mark_transport_loss();

        assert_eq!(p.len(), 2, "transport loss 不能删除 entry (NFR-ORCA-004)");
        assert!(p.is_transport_lost());
        assert_eq!(p.frozen_count(), 2);
        assert_eq!(p.fresh_count(), 0);
    }

    #[test]
    fn mark_transport_recovered_keeps_frozen_until_new_event() {
        // Per NFR-ORCA-004 派生: frozen 的 entry 在收到新 live event 前仍非 fresh
        let p = UnifiedStatusProjection::new();
        let agent = AgentId::new();
        p.record(Uuid::new_v4(), SubjectId::Agent(agent), "Running", None)
            .unwrap();

        p.mark_transport_loss();
        p.mark_transport_recovered();
        assert!(!p.is_transport_lost());
        assert_eq!(
            p.frozen_count(),
            1,
            "recovered 后 entry 仍 Frozen, 等新 live event"
        );
        assert_eq!(p.fresh_count(), 0);
    }

    #[test]
    fn dismiss_removes_entry() {
        // Per NFR-ORCA-004 "dismissal 时清理"
        let p = UnifiedStatusProjection::new();
        let agent = AgentId::new();
        p.record(Uuid::new_v4(), SubjectId::Agent(agent), "Running", None)
            .unwrap();
        assert_eq!(p.len(), 1);
        p.dismiss(&SubjectId::Agent(agent)).unwrap();
        assert_eq!(p.len(), 0);
        let r = p.dismiss(&SubjectId::Agent(agent));
        assert!(matches!(r, Err(UnifiedProjectionError::SubjectNotFound(_))));
    }

    #[test]
    fn view_by_kind_filters() {
        let p = UnifiedStatusProjection::new();
        let a1 = AgentId::new();
        let a2 = AgentId::new();
        let w1 = WorktreeId::new();
        let w2 = WorktreeId::new();
        p.record(Uuid::new_v4(), SubjectId::Agent(a1), "A1", None)
            .unwrap();
        p.record(Uuid::new_v4(), SubjectId::Agent(a2), "A2", None)
            .unwrap();
        p.record(Uuid::new_v4(), SubjectId::Worktree(w1), "W1", None)
            .unwrap();
        p.record(Uuid::new_v4(), SubjectId::Worktree(w2), "W2", None)
            .unwrap();

        let agents = p.view_by_kind(SubjectKind::Agent);
        assert_eq!(agents.len(), 2);
        for e in &agents {
            assert!(matches!(e.subject, SubjectId::Agent(_)));
        }
        let wts = p.view_by_kind(SubjectKind::Worktree);
        assert_eq!(wts.len(), 2);
        for e in &wts {
            assert!(matches!(e.subject, SubjectId::Worktree(_)));
        }
    }

    #[test]
    fn subject_id_display() {
        let agent = AgentId::new();
        let wt = WorktreeId::new();
        assert_eq!(
            SubjectId::Agent(agent).to_string(),
            format!("agent:{agent}")
        );
        assert_eq!(
            SubjectId::Worktree(wt).to_string(),
            format!("worktree:{wt}")
        );
    }

    #[test]
    fn serde_roundtrip() {
        let entry = UnifiedStatusEntry {
            subject: SubjectId::Agent(AgentId::new()),
            status: "Running".to_string(),
            health_score: Some(85),
            last_observed_at: Utc::now(),
            hydration: HydrationState::Confirmed {
                confirmed_at: Utc::now(),
            },
            provenance: Provenance::LiveEventbus {
                event_id: Uuid::new_v4(),
            },
        };
        let json = serde_json::to_string(&entry).unwrap();
        let back: UnifiedStatusEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(back.subject, entry.subject);
        assert_eq!(back.status, entry.status);
        assert_eq!(back.health_score, entry.health_score);
    }
}
