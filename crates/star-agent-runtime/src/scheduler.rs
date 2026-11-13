//! Scheduler — Ready Queue + Priority + Fairness + HOT Slot (per SRS §42-§44
//! + 03-DD §6.1/§6.4)。
//!
//! 算法 (per 03-DD §6.1): Ready Queue 按 (priority ASC, seq ASC) 取;
//! Critical/High 立即调度, Normal/Low/Background per-tenant round-robin
//! fair share (per SRS §57, 防 Background 饿死 + 防单 tenant 占满);
//! HOT Slot (per 03-DD §6.4): hot_count < max_hot_agents, 否则排队等待。

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::fmt;
use std::time::Instant;

use crate::components::Priority;
use crate::id::{AgentId, TaskId, TenantId};

/// Ready Queue 条目 (per 03-DD §6.1 fair_key)。
#[derive(Debug, Clone, PartialEq, Eq)]
struct ReadyEntry {
    priority: Priority,
    seq: u64,
    task_id: TaskId,
    tenant_id: TenantId,
}

impl Ord for ReadyEntry {
    /// max-heap: 更优先 (数值更小) 的排在前面。
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .priority
            .cmp(&self.priority)
            .then(other.seq.cmp(&self.seq))
    }
}

impl PartialOrd for ReadyEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Scheduler 错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchedulerError {
    /// max_hot_agents 必须 >= 1 (per SRS §30, 禁止无 HOT 上限)。
    ZeroHotLimit,
    /// 未知 task。
    UnknownTask(TaskId),
}

impl fmt::Display for SchedulerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SchedulerError::ZeroHotLimit => write!(f, "max_hot_agents 必须 >= 1"),
            SchedulerError::UnknownTask(id) => write!(f, "未知 task {id}"),
        }
    }
}

impl std::error::Error for SchedulerError {}

/// 一轮调度结果 (per 03-DD §6.1)。
#[derive(Debug, Default, Clone)]
pub struct ScheduleOutcome {
    /// 本轮进入 HOT 的 task。
    pub scheduled: Vec<TaskId>,
    /// 停止原因 (HOT 满 / fair share 重排)。
    pub reason: Option<String>,
}

/// Agent Ready Queue + Priority + Fairness + HOT Slot (per SRS §42-§44)。
#[derive(Debug)]
pub struct Scheduler {
    max_hot_agents: usize,
    fair_share: usize,
    seq: u64,
    ready: BinaryHeap<ReadyEntry>,
    hot: HashSet<TaskId>,
    hot_since: HashMap<TaskId, Instant>,
    tenant_turn: HashMap<TenantId, usize>,
}

impl Scheduler {
    /// 新建。max_hot_agents 必须 >= 1 (per SRS §30)。
    pub fn new(max_hot_agents: usize, fair_share: usize) -> Result<Self, SchedulerError> {
        if max_hot_agents == 0 {
            return Err(SchedulerError::ZeroHotLimit);
        }
        Ok(Self {
            max_hot_agents,
            fair_share,
            seq: 0,
            ready: BinaryHeap::new(),
            hot: HashSet::new(),
            hot_since: HashMap::new(),
            tenant_turn: HashMap::new(),
        })
    }

    /// 默认 fair_share = 2。
    pub fn with_default_fair_share(max_hot_agents: usize) -> Result<Self, SchedulerError> {
        Self::new(max_hot_agents, 2)
    }

    /// 任务入 Ready Queue (幂等: 同 task_id 不重复入)。
    pub fn enqueue_ready(&mut self, task_id: TaskId, tenant_id: TenantId, priority: Priority) {
        if self.hot.contains(&task_id) {
            return;
        }
        self.seq += 1;
        self.ready.push(ReadyEntry {
            priority,
            seq: self.seq,
            task_id,
            tenant_id,
        });
    }

    /// Ready Queue 深度 (per SRS §74 ready_queue_depth)。
    pub fn ready_queue_depth(&self) -> usize {
        self.ready.len()
    }

    /// HOT 槽位可用 (per 03-DD §6.4 check_hot_slot_available)。
    pub fn hot_slot_available(&self) -> bool {
        self.hot.len() < self.max_hot_agents
    }

    /// HOT 数量。
    pub fn hot_count(&self) -> usize {
        self.hot.len()
    }

    /// HOT 槽位使用率 0.0-1.0 (per SRS §74 hot_slot_usage)。
    pub fn hot_slot_usage(&self) -> f64 {
        self.hot.len() as f64 / self.max_hot_agents as f64
    }

    /// agent_wait_time (per SRS §74)。
    pub fn agent_wait_time(&self, task_id: TaskId) -> Option<std::time::Duration> {
        self.hot_since.get(&task_id).map(|since| since.elapsed())
    }

    /// 释放 HOT 槽位 (完成/失败时调用)。
    pub fn release_hot(&mut self, task_id: TaskId) {
        self.hot.remove(&task_id);
        self.hot_since.remove(&task_id);
    }

    /// 溢出策略 DropLow (per 03-DD §6.3): 丢最低优先级。
    pub fn drop_low_priority(&mut self, count: usize) -> Vec<TaskId> {
        // 重建堆: 保留除最低 count 个之外的条目
        let mut all: Vec<ReadyEntry> = self.ready.drain().collect();
        all.sort_by(|a, b| b.cmp(a)); // 降序 = 低优先级在前
        let dropped: Vec<TaskId> = all.iter().take(count).map(|e| e.task_id).collect();
        let keep: Vec<ReadyEntry> = all.into_iter().skip(count).collect();
        self.ready = keep.into();
        dropped
    }

    /// 清空 Ready Queue, 返回未调度 (deferred) task id。
    pub fn drain_deferred(&mut self) -> Vec<TaskId> {
        let out: Vec<TaskId> = self.ready.iter().map(|e| e.task_id).collect();
        self.ready.clear();
        out
    }

    fn fair_ok(&self, entry: &ReadyEntry) -> bool {
        if entry.priority <= Priority::High {
            return true; // Critical/High 立即调度 (per 03-DD §6.1)
        }
        self.tenant_turn.get(&entry.tenant_id).copied().unwrap_or(0) < self.fair_share
    }

    /// 执行一轮调度 (per 03-DD §6.1)。每轮 fair share 重置。
    pub fn schedule(&mut self) -> ScheduleOutcome {
        self.tenant_turn.clear();
        let mut outcome = ScheduleOutcome::default();
        let mut deferred_once: HashSet<TaskId> = HashSet::new();
        while let Some(entry) = self.ready.pop() {
            if !self.hot_slot_available() {
                outcome.reason = Some(format!(
                    "HotSlotFull ({}/{})",
                    self.hot.len(),
                    self.max_hot_agents
                ));
                self.ready.push(entry); // 放回, 等下轮
                break;
            }
            if !self.fair_ok(&entry) {
                // Re-enqueue at tail (per 03-DD §6.1), 二次碰到即停
                if !deferred_once.insert(entry.task_id) {
                    self.ready.push(entry);
                    break;
                }
                self.ready.push(entry);
                continue;
            }
            if entry.priority > Priority::High {
                // 只有 Normal/Low/Background 计入 fair 份额
                let turn = self.tenant_turn.entry(entry.tenant_id).or_insert(0);
                *turn += 1;
            }
            self.hot.insert(entry.task_id);
            self.hot_since.insert(entry.task_id, Instant::now());
            outcome.scheduled.push(entry.task_id);
        }
        outcome
    }

    /// Agent 语义别名: 用 AgentId 当 TaskId 调度 (L1 ECS 场景)。
    pub fn enqueue_agent(&mut self, agent_id: AgentId, tenant_id: TenantId, priority: Priority) {
        self.enqueue_ready(TaskId(agent_id.0), tenant_id, priority);
    }

    /// Agent 语义别名: 释放 HOT (L1 场景)。
    pub fn release_agent(&mut self, agent_id: AgentId) {
        self.release_hot(TaskId(agent_id.0));
    }
}
