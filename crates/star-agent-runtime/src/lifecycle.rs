/*
@cypher schema=1 source_sha256=cb5bcc78cf3579ef084579f1e16138cd3acddc0783a9169ea41403009bab8fa7
MERGE (self:File {path:"crates/star-agent-runtime/src/lifecycle.rs"})
@endcypher
*/
//! LifecycleState — HOT/WARM/COLD + LifecycleManager (per SRS §11/§21-§25
//! + 03-DD §3.2)。
//!
//! 转换规则 (per 03-DD §3.2):
//! `Event → HOT`; `HOT → WARM` (60s 无活动); `WARM → COLD` (600s idle);
//! `COLD → (Event/Restore) → WARM → (Schedule) → HOT`。超时可配置 (per SRS §7.4)。
//! 不得由 Agent 自己决定长期生命周期 (per SRS §25 REQ-LC-MGR)。

use std::collections::HashMap;
use std::fmt;
use std::time::{Duration, Instant};

use crate::ecs::Component;
use crate::id::AgentId;

/// 默认 HOT → WARM 超时 (per 03-DD §3.2: 60s)。
pub const DEFAULT_HOT_TIMEOUT: Duration = Duration::from_secs(60);
/// 默认 WARM → COLD idle 超时 (per 03-DD §3.2: 600s)。
pub const DEFAULT_COLD_IDLE_TIMEOUT: Duration = Duration::from_secs(600);

/// Lifecycle 状态 (per SRS §11 REQ-ECS-003)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LifecycleStateEnum {
    /// 执行中 (MB 级临时内存, 数量受限 per SRS §30)。
    Hot,
    /// 等待 (轻量状态, 目标 < 100 KB per SRS §62)。
    Warm,
    /// 持久化 (Runtime RAM ≈ 0, per SRS §23)。
    Cold,
}

/// LifecycleState Component (per 03-DD §2.1: current/last_active/timeouts)。
#[derive(Debug, Clone)]
pub struct LifecycleState {
    /// 当前状态。
    pub current: LifecycleStateEnum,
    /// 最近活动时间。
    pub last_active: Instant,
    /// HOT → WARM 超时 (可配置, per SRS §7.4)。
    pub hot_timeout: Duration,
    /// WARM → COLD 超时 (可配置)。
    pub warm_timeout: Duration,
}

impl Component for LifecycleState {}

impl Default for LifecycleState {
    fn default() -> Self {
        Self {
            current: LifecycleStateEnum::Cold,
            last_active: Instant::now(),
            hot_timeout: DEFAULT_HOT_TIMEOUT,
            warm_timeout: DEFAULT_COLD_IDLE_TIMEOUT,
        }
    }
}

impl LifecycleState {
    /// 新 Agent (COLD 起步, per 03-DD §3.2)。
    pub fn new() -> Self {
        Self::default()
    }

    /// 指定超时 (per SRS §7.4 配置化)。
    pub fn with_timeouts(hot_timeout: Duration, warm_timeout: Duration) -> Self {
        Self {
            current: LifecycleStateEnum::Cold,
            last_active: Instant::now(),
            hot_timeout,
            warm_timeout,
        }
    }

    /// Event / Schedule → HOT (per 03-DD §3.2)。COLD 先恢复。
    pub fn activate(&mut self, now: Instant) {
        self.current = LifecycleStateEnum::Hot;
        self.last_active = now;
    }

    /// 记录活动时间 (HOT 保持)。
    pub fn touch(&mut self, now: Instant) {
        self.last_active = now;
    }

    /// 周期扫描: HOT → WARM → COLD (per 03-DD §3.2)。返回新状态 (无变化 None)。
    pub fn sweep(&mut self, now: Instant) -> Option<LifecycleStateEnum> {
        let idle = now.duration_since(self.last_active);
        match self.current {
            LifecycleStateEnum::Hot if idle > self.hot_timeout => {
                self.current = LifecycleStateEnum::Warm;
                Some(LifecycleStateEnum::Warm)
            }
            LifecycleStateEnum::Warm if idle > self.warm_timeout => {
                self.current = LifecycleStateEnum::Cold;
                Some(LifecycleStateEnum::Cold)
            }
            _ => None,
        }
    }

    /// COLD → WARM (per SRS §23: Load → WARM → Schedule → HOT)。
    pub fn restore(&mut self, now: Instant) {
        if self.current == LifecycleStateEnum::Cold {
            self.current = LifecycleStateEnum::Warm;
            self.last_active = now;
        }
    }
}

/// Lifecycle 操作错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LifecycleError {
    /// Agent 不存在。
    NotFound(AgentId),
    /// 非法转换 (HOT → COLD 直跳等)。
    IllegalTransition(LifecycleStateEnum, LifecycleStateEnum),
}

impl fmt::Display for LifecycleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LifecycleError::NotFound(id) => write!(f, "agent {id} 不存在"),
            LifecycleError::IllegalTransition(from, to) => {
                write!(f, "非法 lifecycle 转换: {from:?} → {to:?}")
            }
        }
    }
}

impl std::error::Error for LifecycleError {}

/// LifecycleManager 接口 (per 02-BD §5.5 + SRS §25 REQ-LC-MGR)。
pub trait LifecycleManager {
    /// 状态转换 (HOT ↔ WARM ↔ COLD)。
    fn transition(
        &mut self,
        agent_id: AgentId,
        target: LifecycleStateEnum,
    ) -> Result<(), LifecycleError>;
    /// 持久化 (WARM → COLD, per SRS §53)。
    fn persist(&self, agent_id: AgentId) -> Result<(), LifecycleError>;
    /// 恢复 (COLD → WARM, lazy load per SRS §38)。
    fn restore(&mut self, agent_id: AgentId) -> Result<(), LifecycleError>;
    /// 超时判定 (per SRS §49 Timeout)。
    fn timeout(&self, agent_id: AgentId) -> Result<bool, LifecycleError>;
    /// 驱逐 (eviction, per SRS §66 Memory Pressure)。
    fn evict(&mut self, agent_id: AgentId) -> Result<(), LifecycleError>;
}

/// 内存实现 (骨架; 真实实现走 ECS World + P3-D 持久化层)。
#[derive(Debug, Default)]
pub struct InMemoryLifecycleManager {
    states: HashMap<AgentId, LifecycleState>,
}

impl InMemoryLifecycleManager {
    /// 空管理器。
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册 agent (spawn 时调用)。
    pub fn register(&mut self, agent_id: AgentId, state: LifecycleState) {
        self.states.insert(agent_id, state);
    }

    /// 移除 agent (despawn 时调用)。
    pub fn unregister(&mut self, agent_id: AgentId) {
        self.states.remove(&agent_id);
    }

    /// 读当前状态。
    pub fn get(&self, agent_id: AgentId) -> Option<&LifecycleState> {
        self.states.get(&agent_id)
    }

    /// 周期扫描全部 agent。
    pub fn sweep_all(&mut self, now: Instant) -> Vec<(AgentId, LifecycleStateEnum)> {
        let mut out = Vec::new();
        for (id, state) in self.states.iter_mut() {
            if let Some(new_state) = state.sweep(now) {
                out.push((*id, new_state));
            }
        }
        out
    }
}

impl LifecycleManager for InMemoryLifecycleManager {
    fn transition(
        &mut self,
        agent_id: AgentId,
        target: LifecycleStateEnum,
    ) -> Result<(), LifecycleError> {
        let state = self
            .states
            .get_mut(&agent_id)
            .ok_or(LifecycleError::NotFound(agent_id))?;
        let legal = matches!(
            (state.current, target),
            (LifecycleStateEnum::Hot, LifecycleStateEnum::Warm)
                | (LifecycleStateEnum::Warm, LifecycleStateEnum::Hot)
                | (LifecycleStateEnum::Warm, LifecycleStateEnum::Cold)
                | (LifecycleStateEnum::Cold, LifecycleStateEnum::Warm)
        );
        if !legal {
            return Err(LifecycleError::IllegalTransition(state.current, target));
        }
        state.current = target;
        state.last_active = Instant::now();
        Ok(())
    }

    fn persist(&self, agent_id: AgentId) -> Result<(), LifecycleError> {
        // 骨架: 仅验证存在性 (真实持久化 P3-D, per 已知缺口 G-7)
        self.states
            .get(&agent_id)
            .map(|_| ())
            .ok_or(LifecycleError::NotFound(agent_id))
    }

    fn restore(&mut self, agent_id: AgentId) -> Result<(), LifecycleError> {
        let state = self
            .states
            .get_mut(&agent_id)
            .ok_or(LifecycleError::NotFound(agent_id))?;
        state.restore(Instant::now());
        Ok(())
    }

    fn timeout(&self, agent_id: AgentId) -> Result<bool, LifecycleError> {
        let state = self
            .states
            .get(&agent_id)
            .ok_or(LifecycleError::NotFound(agent_id))?;
        let now = Instant::now();
        Ok(match state.current {
            LifecycleStateEnum::Hot => now.duration_since(state.last_active) > state.hot_timeout,
            LifecycleStateEnum::Warm => now.duration_since(state.last_active) > state.warm_timeout,
            LifecycleStateEnum::Cold => false,
        })
    }

    fn evict(&mut self, agent_id: AgentId) -> Result<(), LifecycleError> {
        self.states
            .remove(&agent_id)
            .map(|_| ())
            .ok_or(LifecycleError::NotFound(agent_id))
    }
}
