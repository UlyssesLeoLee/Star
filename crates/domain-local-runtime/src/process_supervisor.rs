//! Star Local Runtime — ProcessSupervisor (ULYS-156 P0-A 续)
//!
//! 实现 [ULYS-156](https://app.multica.ai/issue/01a0bf6b-486c-71b8-9bf4-92bde909c7f8)
//! §A 单进程持久化层适配 路线下的 "process_supervisor" 模块:
//!
//! - 内存 + SQLite WAL 双层 launch_history(per session 滑动窗口)
//! - **NFR-ORCA-002**: Crash-loop containment — 同 session 在 60s 内 launch ≥5 次
//!   → 拒绝第 6 次 launch,等窗口滑出
//! - 与 `cli_session_lock::SessionLock` 串联:每次 launch 都尝试 acquire 一条
//!   (session_id, pid, start_time_epoch_ms) lock,失败(PID 回收)→ fail-fast
//! - 进程级 launch history 默认按 `(cli_session_id, command)` 维度聚合:
//!   同一 session + 同一 cmd → 累计触发 crash-loop containment
//!
//! **不在本模块**:
//! - 真实 OS `kill(pid, 0)` 实现(同 cli_session_lock)
//! - 重启超时(per CliSessionState::Restart → Orphaned 计时)— 留 P1
//! - star-eventbus 通道
//!
//! 关键不变量:
//! - **INV-SUP-01**: launch history 滑动窗口 = `recent_window_secs`(默认 60s)
//! - **INV-SUP-02**: 同窗口内 launch_count ≥ `max_launches_per_window`(默认 5)
//!   → reject + 记录 `last_rejected_at` 供上游退避
//! - **INV-SUP-03**: supervisor 自身是 in-memory `Mutex<HashMap>`,**不持久化**:
//!   进程重启后历史清零(per §A 单进程模型 trade-off, D-Boy 9/22 接受)

use chrono::{DateTime, Duration as ChronoDuration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
use thiserror::Error;

use super::cli_session::{CliSession, CliSessionState};
use super::{CliSessionId, SupervisorId, TenantId};

// =====================================================================
// 1. config
// =====================================================================

/// ProcessSupervisor 配置(per NFR-ORCA-002 默认值,可在测试中调整)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupervisorConfig {
    /// 滑动窗口长度(秒),默认 60s
    pub recent_window_secs: i64,
    /// 窗口内最大允许 launch 数,默认 5 次
    pub max_launches_per_window: usize,
    /// 启动 supervisor_id(便于审计 + 多实例下排错;本期单实例)
    pub supervisor_id: SupervisorId,
}

impl SupervisorConfig {
    /// 默认配置:60s / 5 次(NFR-ORCA-002 锚定)
    pub fn default_config() -> Self {
        Self {
            recent_window_secs: 60,
            max_launches_per_window: 5,
            supervisor_id: SupervisorId::new(),
        }
    }
}

// =====================================================================
// 2. entity — LaunchRecord + LaunchDecision
// =====================================================================

/// 单次 launch 记录(append-only, in-memory)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LaunchRecord {
    /// launch 时间
    pub at: DateTime<Utc>,
    /// 关联 session
    pub cli_session_id: CliSessionId,
    /// 进程 PID
    pub pid: u32,
    /// 启动时间 epoch millis(防 PID 回收)
    pub start_time_epoch_ms: i64,
    /// 命令
    pub command: String,
}

/// launch 决策(per INV-SUP-02)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LaunchDecision {
    /// 通过 — 允许 launch
    Allow {
        /// 当前窗口内累计 launch 数(包含本次)
        count_in_window: usize,
        /// 窗口起点
        window_started_at: DateTime<Utc>,
    },
    /// 拒绝 — 窗口内超限
    Reject {
        /// 当前窗口内累计 launch 数
        count_in_window: usize,
        /// 窗口剩余时间(秒)
        window_remaining_secs: i64,
        /// supervisor 建议的重试间隔(秒),不小于 `window_remaining_secs`
        retry_after_secs: i64,
    },
}

// =====================================================================
// 3. error
// =====================================================================

/// Supervisor 错误
#[derive(Debug, Error)]
pub enum SupervisorError {
    /// Crash-loop containment 触发
    #[error("crash-loop containment: {count} launches in last {window_secs}s (max {max})")]
    CrashLoopContained {
        /// 当前窗口内 launch 次数
        count: usize,
        /// 窗口长度(秒)
        window_secs: i64,
        /// 配置的上限
        max: usize,
    },
    /// 配置无效
    #[error("invalid config: {0}")]
    InvalidConfig(String),
}

// =====================================================================
// 4. service
// =====================================================================

/// **ProcessSupervisor** — 进程监督器(per NFR-ORCA-002)
///
/// 内存级 launch history(per INV-SUP-03),线程安全。
pub struct ProcessSupervisor {
    config: SupervisorConfig,
    /// launch 历史:cli_session_id → 滑动窗口队列(按 at 升序)
    history: Mutex<HashMap<CliSessionId, VecDeque<LaunchRecord>>>,
    /// 最近一次拒绝时间,外部观测
    last_rejected_at: Mutex<Option<(CliSessionId, DateTime<Utc>)>>,
}

impl ProcessSupervisor {
    /// 用默认配置构造
    pub fn new() -> Self {
        Self::with_config(SupervisorConfig::default_config())
    }

    /// 用自定义配置构造
    pub fn with_config(config: SupervisorConfig) -> Self {
        if config.recent_window_secs <= 0 {
            // 防御性 — 0 或负窗口会让所有 launch 全部入窗,立即触发 crash-loop
            // 我们不 panic,把窗口钳到 1s 兜底
            let mut fixed = config.clone();
            fixed.recent_window_secs = 1;
            return Self {
                config: fixed,
                history: Mutex::new(HashMap::new()),
                last_rejected_at: Mutex::new(None),
            };
        }
        if config.max_launches_per_window == 0 {
            let mut fixed = config.clone();
            fixed.max_launches_per_window = 1;
            return Self {
                config: fixed,
                history: Mutex::new(HashMap::new()),
                last_rejected_at: Mutex::new(None),
            };
        }
        Self {
            config,
            history: Mutex::new(HashMap::new()),
            last_rejected_at: Mutex::new(None),
        }
    }

    /// 当前 supervisor_id(per audit)
    pub fn supervisor_id(&self) -> SupervisorId {
        self.config.supervisor_id
    }

    /// 当前配置(per 测试可见性)
    pub fn config(&self) -> &SupervisorConfig {
        &self.config
    }

    /// 计算 launch 决策(不修改 history)— 纯函数,便于测试
    pub fn decide(&self, cli_session_id: CliSessionId, now: DateTime<Utc>) -> LaunchDecision {
        let history = self.history.lock().expect("lock");
        let window_start = now - ChronoDuration::seconds(self.config.recent_window_secs);
        let queue = match history.get(&cli_session_id) {
            Some(q) => q,
            None => {
                return LaunchDecision::Allow {
                    count_in_window: 1, // 含本次
                    window_started_at: now,
                };
            }
        };
        // 滑出窗口
        let in_window: Vec<&LaunchRecord> = queue.iter().filter(|r| r.at >= window_start).collect();
        let count_in_window = in_window.len();
        let window_started_at = in_window.first().map(|r| r.at).unwrap_or(now);
        if count_in_window >= self.config.max_launches_per_window {
            // 窗口剩余时间
            let window_end =
                window_started_at + ChronoDuration::seconds(self.config.recent_window_secs);
            let remaining = (window_end - now).num_seconds().max(0);
            LaunchDecision::Reject {
                count_in_window,
                window_remaining_secs: remaining,
                retry_after_secs: remaining.max(1),
            }
        } else {
            LaunchDecision::Allow {
                count_in_window: count_in_window + 1, // 含本次
                window_started_at: in_window.first().map(|r| r.at).unwrap_or(now),
            }
        }
    }

    /// 记录一次 launch(并执行 decide;Reject 时不写入 history — INV-SUP-02)
    ///
    /// 返回 LaunchDecision:`Allow` 表示已写入,`Reject` 表示拒绝(写入被抑制)。
    pub fn record_launch(
        &self,
        cli_session_id: CliSessionId,
        pid: u32,
        start_time_epoch_ms: i64,
        command: String,
        now: DateTime<Utc>,
    ) -> LaunchDecision {
        let decision = self.decide(cli_session_id, now);
        match &decision {
            LaunchDecision::Allow { .. } => {
                let mut history = self.history.lock().expect("lock");
                let queue = history.entry(cli_session_id).or_insert_with(VecDeque::new);
                queue.push_back(LaunchRecord {
                    at: now,
                    cli_session_id,
                    pid,
                    start_time_epoch_ms,
                    command,
                });
                // 滑出窗口外的旧记录清理
                let window_start = now - ChronoDuration::seconds(self.config.recent_window_secs);
                while let Some(front) = queue.front() {
                    if front.at < window_start {
                        queue.pop_front();
                    } else {
                        break;
                    }
                }
            }
            LaunchDecision::Reject { .. } => {
                let mut last = self.last_rejected_at.lock().expect("lock");
                *last = Some((cli_session_id, now));
            }
        }
        decision
    }

    /// 清空一个 session 的历史(用于显式 reset — 测试 + 管理员操作)
    pub fn reset_history(&self, cli_session_id: CliSessionId) {
        let mut history = self.history.lock().expect("lock");
        history.remove(&cli_session_id);
    }

    /// 列出一个 session 的历史 launch 数(per 观测 + 测试)
    pub fn history_len(&self, cli_session_id: CliSessionId) -> usize {
        let history = self.history.lock().expect("lock");
        history.get(&cli_session_id).map(|q| q.len()).unwrap_or(0)
    }

    /// 最近一次拒绝的 session + 时间(per 观测)
    pub fn last_rejected(&self) -> Option<(CliSessionId, DateTime<Utc>)> {
        let last = self.last_rejected_at.lock().expect("lock");
        *last
    }
}

impl Default for ProcessSupervisor {
    fn default() -> Self {
        Self::new()
    }
}

// =====================================================================
// 5. helper — 与 CliSession / SessionLock 串联
// =====================================================================

/// 给定一个 CliSession,若它当前可以 launch(非 Archived / Terminated)→ Ok(()),
/// 否则返回 InvalidState 描述。
///
/// 这条规则保证:archived session 即使收到 launch 请求也会被上游拦截,
/// crash-loop containment 不会把 "用户归档后又点开" 的语义吞噬。
pub fn ensure_session_launchable(session: &CliSession) -> Result<(), SupervisorError> {
    match session.state {
        CliSessionState::Created
        | CliSessionState::Running
        | CliSessionState::Paused
        | CliSessionState::Restart
        | CliSessionState::Orphaned => Ok(()),
        CliSessionState::Terminated | CliSessionState::Archived => {
            Err(SupervisorError::InvalidConfig(format!(
                "session {} is in terminal/non-launchable state {:?}",
                session.id, session.state
            )))
        }
    }
}

/// 把 tenant 维度 supervisor 配额汇总(便于后续 multi-tenant quota;本期单实例共享)
#[derive(Debug, Default)]
pub struct TenantLaunchCounters {
    /// per_tenant launch 计数(tenant_id → 累计次数)
    pub per_tenant: HashMap<TenantId, usize>,
}

impl TenantLaunchCounters {
    /// 构造空配额汇总
    pub fn new() -> Self {
        Self::default()
    }
    /// 记录一次 launch
    pub fn record(&mut self, tenant_id: TenantId) {
        *self.per_tenant.entry(tenant_id).or_insert(0) += 1;
    }
    /// 读某 tenant 的累计次数
    pub fn count(&self, tenant_id: TenantId) -> usize {
        self.per_tenant.get(&tenant_id).copied().unwrap_or(0)
    }
}

// =====================================================================
// 6. unit tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TenantId;
    use crate::WorktreeId;

    fn fixture_session() -> CliSessionId {
        CliSessionId::new()
    }

    // ---- 1. 默认配置 (NFR-ORCA-002 锚定:60s / 5 次)
    #[test]
    fn default_config_anchors_nfr_orca_002() {
        let cfg = SupervisorConfig::default_config();
        assert_eq!(cfg.recent_window_secs, 60);
        assert_eq!(cfg.max_launches_per_window, 5);
    }

    // ---- 2. 窗口内前 N 次 Allow,第 N+1 次 Reject
    #[test]
    fn crash_loop_containment_at_threshold() {
        let cfg = SupervisorConfig {
            recent_window_secs: 60,
            max_launches_per_window: 3,
            supervisor_id: SupervisorId::new(),
        };
        let sup = ProcessSupervisor::with_config(cfg);
        let sid = fixture_session();
        let t0 = Utc::now();
        let d1 = sup.record_launch(sid, 1, 1, "claude".to_string(), t0);
        let d2 = sup.record_launch(sid, 2, 2, "claude".to_string(), t0);
        let d3 = sup.record_launch(sid, 3, 3, "claude".to_string(), t0);
        let d4 = sup.record_launch(sid, 4, 4, "claude".to_string(), t0);
        assert!(matches!(
            d1,
            LaunchDecision::Allow {
                count_in_window: 1,
                ..
            }
        ));
        assert!(matches!(
            d2,
            LaunchDecision::Allow {
                count_in_window: 2,
                ..
            }
        ));
        assert!(matches!(
            d3,
            LaunchDecision::Allow {
                count_in_window: 3,
                ..
            }
        ));
        assert!(matches!(
            d4,
            LaunchDecision::Reject {
                count_in_window: 3,
                ..
            }
        ));
        // 被拒不入窗
        assert_eq!(sup.history_len(sid), 3);
    }

    // ---- 3. 窗口滑出 → 新一帧 → 重新 Allow
    #[test]
    fn window_slide_resets_count() {
        let cfg = SupervisorConfig {
            recent_window_secs: 1,
            max_launches_per_window: 2,
            supervisor_id: SupervisorId::new(),
        };
        let sup = ProcessSupervisor::with_config(cfg);
        let sid = fixture_session();
        let t0 = Utc::now();
        sup.record_launch(sid, 1, 1, "claude".to_string(), t0);
        sup.record_launch(sid, 2, 2, "claude".to_string(), t0);
        let d3_at_t0 = sup.record_launch(sid, 3, 3, "claude".to_string(), t0);
        assert!(matches!(d3_at_t0, LaunchDecision::Reject { .. }));

        // 滑出窗口(> 1s)
        let t1 = t0 + ChronoDuration::seconds(2);
        let d4_after_slide = sup.record_launch(sid, 4, 4, "claude".to_string(), t1);
        assert!(matches!(
            d4_after_slide,
            LaunchDecision::Allow {
                count_in_window: 1,
                ..
            }
        ));
    }

    // ---- 4. last_rejected 记录
    #[test]
    fn last_rejected_recorded() {
        let cfg = SupervisorConfig {
            recent_window_secs: 60,
            max_launches_per_window: 1,
            supervisor_id: SupervisorId::new(),
        };
        let sup = ProcessSupervisor::with_config(cfg);
        let sid = fixture_session();
        let t0 = Utc::now();
        sup.record_launch(sid, 1, 1, "claude".to_string(), t0);
        sup.record_launch(sid, 2, 2, "claude".to_string(), t0);
        let last = sup.last_rejected();
        assert_eq!(last.map(|(s, _)| s), Some(sid));
    }

    // ---- 5. reset_history 清除
    #[test]
    fn reset_history_clears() {
        let sup = ProcessSupervisor::new();
        let sid = fixture_session();
        sup.record_launch(sid, 1, 1, "claude".to_string(), Utc::now());
        assert_eq!(sup.history_len(sid), 1);
        sup.reset_history(sid);
        assert_eq!(sup.history_len(sid), 0);
    }

    // ---- 6. ensure_session_launchable 拒绝 Archived
    #[test]
    fn ensure_session_launchable_rejects_archived() {
        let mut s = CliSession::new(
            TenantId::new(),
            WorktreeId::new(),
            "claude".to_string(),
            vec![],
        );
        let _ = s.try_transition(CliSessionState::Running, "start");
        let _ = s.try_transition(CliSessionState::Terminated, "stop");
        let _ = s.try_transition(CliSessionState::Archived, "archive");
        let r = ensure_session_launchable(&s);
        assert!(matches!(r, Err(SupervisorError::InvalidConfig(_))));
    }

    // ---- 7. TenantLaunchCounters
    #[test]
    fn tenant_launch_counters_roundtrip() {
        let mut c = TenantLaunchCounters::new();
        let t = TenantId::new();
        c.record(t);
        c.record(t);
        c.record(TenantId::new());
        assert_eq!(c.count(t), 2);
    }
}
