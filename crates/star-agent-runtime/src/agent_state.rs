//! AgentState — Agent 12 状态 + 状态机 (per SRS §10 REQ-ECS-002 + 03-DD §3.1)。
//!
//! 转换图 (per 03-DD §3.1):
//! `Idle → Ready → Scheduled → Planning → WaitingLlm|WaitingTool|WaitingEvent`
//! `→ Processing → Completed → Idle`; 异常 `Failed → Ready` /
//! `Suspended → Scheduled` / `Cancelled` 终态。

use std::time::Instant;

use crate::ecs::Component;

/// Agent 12 状态 (per SRS §10 REQ-ECS-002 + 03-DD §2.1)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AgentStateEnum {
    /// 空闲。
    Idle,
    /// 就绪 (已 schedule)。
    Ready,
    /// 已排程 (已 dispatch)。
    Scheduled,
    /// 规划中。
    Planning,
    /// 等待 LLM 结果。
    WaitingLlm,
    /// 等待 Tool 结果。
    WaitingTool,
    /// 等待 Event。
    WaitingEvent,
    /// 处理中。
    Processing,
    /// 完成。
    Completed,
    /// 失败 (可 retry → Ready)。
    Failed,
    /// 挂起 (resume → Scheduled)。
    Suspended,
    /// 取消 (终态)。
    Cancelled,
}

/// AgentState Component (per 03-DD §2.1: current/prev/since/retry_count)。
#[derive(Debug, Clone)]
pub struct AgentState {
    /// 当前状态。
    pub current: AgentStateEnum,
    /// 前一状态 (状态机审计)。
    pub prev: AgentStateEnum,
    /// 进入 current 的时间。
    pub since: Instant,
    /// 重试计数 (per SRS §51)。
    pub retry_count: u32,
}

impl Component for AgentState {}

impl Default for AgentState {
    fn default() -> Self {
        Self {
            current: AgentStateEnum::Idle,
            prev: AgentStateEnum::Idle,
            since: Instant::now(),
            retry_count: 0,
        }
    }
}

impl AgentState {
    /// 新 Agent 初始状态 (Idle)。
    pub fn new() -> Self {
        Self::default()
    }

    /// 合法转换判定 (per 03-DD §3.1 转换图)。
    pub fn can_transition(current: AgentStateEnum, target: AgentStateEnum) -> bool {
        use AgentStateEnum::*;
        match (current, target) {
            // 主链 (per 03-DD §3.1)
            (Idle, Ready) | (Ready, Scheduled) | (Scheduled, Planning) => true,
            (Planning, WaitingLlm | WaitingTool | WaitingEvent) => true,
            (WaitingLlm | WaitingTool | WaitingEvent, Processing) => true,
            (Processing, Completed) => true,
            (Completed, Idle) => true,
            // 异常 (per 03-DD §3.1: Failed → Ready, Suspended → Scheduled)
            (Failed, Ready) => true,
            (Suspended, Scheduled) => true,
            // 横向异常入口
            (s, Failed) => matches!(
                s,
                Idle | Ready
                    | Scheduled
                    | Planning
                    | WaitingLlm
                    | WaitingTool
                    | WaitingEvent
                    | Processing
            ),
            (s, Suspended) => matches!(
                s,
                Idle | Ready
                    | Scheduled
                    | Planning
                    | WaitingLlm
                    | WaitingTool
                    | WaitingEvent
                    | Processing
                    | Failed
            ),
            (s, Cancelled) => matches!(
                s,
                Idle | Ready
                    | Scheduled
                    | Planning
                    | WaitingLlm
                    | WaitingTool
                    | WaitingEvent
                    | Processing
                    | Failed
                    | Suspended
            ),
            _ => false,
        }
    }

    /// 状态转换; 非法转换返回 false 不改状态 (per 03-DD §3.1)。
    pub fn transition(&mut self, target: AgentStateEnum, now: Instant) -> bool {
        if !Self::can_transition(self.current, target) {
            return false;
        }
        self.prev = self.current;
        self.current = target;
        self.since = now;
        true
    }

    /// 取消 (per SRS §50 REQ-CANCEL-001), Cancelled 终态。
    pub fn cancel(&mut self, now: Instant) -> bool {
        self.transition(AgentStateEnum::Cancelled, now)
    }

    /// 挂起 (per SRS §50), resume 回 Scheduled。
    pub fn suspend(&mut self, now: Instant) -> bool {
        self.transition(AgentStateEnum::Suspended, now)
    }

    /// 恢复 → Scheduled (per 03-DD §3.1)。
    pub fn resume(&mut self, now: Instant) -> bool {
        self.transition(AgentStateEnum::Scheduled, now)
    }

    /// 失败 (retry_count+1, per 03-DD §3.1)。
    pub fn fail(&mut self, now: Instant) -> bool {
        self.retry_count += 1;
        self.transition(AgentStateEnum::Failed, now)
    }

    /// Failed → Ready 重试 (per 03-DD §3.1 retry)。
    pub fn retry(&mut self, now: Instant) -> bool {
        self.transition(AgentStateEnum::Ready, now)
    }
}
