//! star-scheduler — MS Project-like 甘特/CPM Rust 阶段 1 (R8 阶段 1) 🟢
//!
//! Per ADR-0027 v0.1 §2.3 + plan-032 R8: **MS Project 长处整合** = 关键路径 CPM + 甘特图 +
//! 资源分配 + 里程碑. **MS Project 限制避免** = 不用 MS Project 复杂 UI 改 Rust enum+match,
//! 不用第三方 CPM 库改 Rust trait 抽象 (阶段 1 抽象层, 阶段 2 引 petgraph 拓扑序加速).
//!
//! WBS 集成 (per ADR-0027 §2.3.2 共享 Task schema 跨 3 view):
//! - WBS row = Schedule Task
//! - 任务依赖 = Dependency (FS/SS/FF/SF 4 档)
//! - 关键路径 = CPM algorithm (per plan-032 R8 line 130 拓扑序 + 最早/最晚开始时间)
//! - 甘特图 = Schedule 完整视图
//!
//! 跟 star-task / star-workflow 跨域 (per 守门 #1 跨域 consults):
//! - star-task 7 态 (claim/queue/run/review/done/...) = 实现层 task lifecycle
//! - star-workflow Issue (IssueKey) ↔ star-scheduler Task (TaskId) 1:1 关联 (R9 整合)
//!
//! 守门合规 (per 守门 #1 v25 cargo test 单 crate 实证 + 守门 #7 0 unsafe + 守门 #11 缺标比错标)

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use std::collections::{HashMap, VecDeque};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

// ============================================================================
// §1 ID newtype (per Multica opaque ID 1:1 派生)
// ============================================================================

/// Schedule Task ID (per MS Project task)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(pub Uuid);

impl From<Uuid> for TaskId {
    fn from(id: Uuid) -> Self {
        Self(id)
    }
}

/// Dependency ID (per MS Project predecessor link)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DependencyId(pub Uuid);

/// Milestone ID (per MS Project milestone, 0 duration task)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MilestoneId(pub Uuid);

/// Resource ID (per MS Project resource, 人员/设备)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourceId(pub Uuid);

/// Schedule ID (一个 project schedule)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ScheduleId(pub Uuid);

// ============================================================================
// §2 DependencyType (MS Project 4 档 dependency type, per PMBOK)
// ============================================================================

/// Dependency 类型 (per MS Project 4 档 predecessor link)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum DependencyType {
    /// Finish-to-Start (FS, 最常见, 前置完成 → 后置开始, per PMBOK default)
    #[default]
    FinishToStart,
    /// Start-to-Start (SS, 前置开始 → 后置开始)
    StartToStart,
    /// Finish-to-Finish (FF, 前置完成 → 后置完成)
    FinishToFinish,
    /// Start-to-Finish (SF, 罕见, 前置开始 → 后置完成)
    StartToFinish,
}

// ============================================================================
// §3 Task (per MS Project task = id + name + duration + dependencies)
// ============================================================================

/// Schedule Task (MS Project task = WBS row, 跟 star-task 1:1 派生)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    /// Task ID
    pub id: TaskId,
    /// Task 名称
    pub name: String,
    /// 持续时间 (任意单位, e.g. 天/小时; PoC 不做单位换算)
    pub duration: u32,
    /// 前置 Task IDs (隐式 Finish-to-Start 关系, 用于拓扑序)
    pub predecessors: Vec<TaskId>,
    /// 关联 Issue Key (跟 star-workflow 跨域, e.g. "STAR-001")
    pub issue_key: Option<String>,
}

impl Task {
    /// 新建 task
    pub fn new(name: impl Into<String>, duration: u32) -> Self {
        Self {
            id: TaskId(Uuid::new_v4()),
            name: name.into(),
            duration,
            predecessors: Vec::new(),
            issue_key: None,
        }
    }
    /// 加前置 task (FS 关系, per plan-032 R8 line 130)
    pub fn add_predecessor(&mut self, predecessor: TaskId) {
        self.predecessors.push(predecessor);
    }
}

// ============================================================================
// §3.5 Task → SharedTask From impl (R9 阶段 2 整合, per DD-SHARED-TASK-001 §4.4)
// ============================================================================

/// Task → SharedTask 转换 (per DD-SHARED-TASK-001 §4.4 字段映射表)
///
/// 注: MS Project Task 不存 description / created_at (走 Schedule.created 代理),
/// state 通过 CpmNode 派生 (R9 阶段 1 默认 Open), assignee 走 TaskAssignment 派生
impl From<Task> for shared_task::SharedTask {
    fn from(task: Task) -> Self {
        Self {
            // star-scheduler::TaskId 是 local newtype (per R8 阶段 1), 提取 Uuid 转 shared_task::TaskId
            id: shared_task::TaskId(task.id.0),
            title: task.name,
            description: String::new(), // Task 不存 description (per DD §4.4)
            state: shared_task::TaskState::Open, // 通过 CpmNode 派生 (R9 阶段 2 临时, R9 阶段 3 修复 per DD §7 缺口 #6)
            assignee: None,                      // 走 TaskAssignment 派生 (per DD §4.4)
            priority: shared_task::Priority::default(),
            issue_key: task.issue_key,
            created_at: std::time::SystemTime::now(), // 走 Schedule.created 代理 (per DD §7 缺口 #6)
        }
    }
}

// ============================================================================
// §4 Dependency (per MS Project predecessor link, 4 档 dependency type)
// ============================================================================

/// Dependency (predecessor link = from → to + type + lag, per MS Project)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dependency {
    /// Dependency ID
    pub id: DependencyId,
    /// 前置 task
    pub from: TaskId,
    /// 后置 task
    pub to: TaskId,
    /// Dependency 类型 (FS/SS/FF/SF, per §2)
    pub dep_type: DependencyType,
    /// Lag (正数 = 延迟, 负数 = lead, 0 = 无 lag, per MS Project)
    pub lag: i32,
}

impl Dependency {
    /// 新建 dependency (FS 默认, lag = 0)
    pub fn new(from: TaskId, to: TaskId) -> Self {
        Self {
            id: DependencyId(Uuid::new_v4()),
            from,
            to,
            dep_type: DependencyType::FinishToStart,
            lag: 0,
        }
    }
    /// 新建 dependency with type + lag
    pub fn with_type_and_lag(from: TaskId, to: TaskId, dep_type: DependencyType, lag: i32) -> Self {
        Self {
            id: DependencyId(Uuid::new_v4()),
            from,
            to,
            dep_type,
            lag,
        }
    }
}

// ============================================================================
// §5 Milestone + Resource + TaskAssignment (per MS Project 资源/里程碑)
// ============================================================================

/// Milestone (0 duration task, 标记关键节点, per MS Project)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Milestone {
    /// Milestone ID
    pub id: MilestoneId,
    /// Milestone 名称
    pub name: String,
    /// 目标日期 (epoch seconds, per PoC 简化)
    pub target_epoch: u64,
    /// 关联 Task ID (milestone 通常挂在 task 完成时)
    pub task_id: Option<TaskId>,
}

impl Milestone {
    /// 新建 milestone
    pub fn new(name: impl Into<String>, target_epoch: u64) -> Self {
        Self {
            id: MilestoneId(Uuid::new_v4()),
            name: name.into(),
            target_epoch,
            task_id: None,
        }
    }
}

/// Resource (人员/设备, per MS Project resource)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Resource {
    /// Resource ID
    pub id: ResourceId,
    /// Resource 名称
    pub name: String,
    /// 容量 (e.g. 1.0 = 100%, 0.5 = 50%, 2.0 = 2 人)
    pub capacity: f64,
    /// 单位成本
    pub unit_cost: f64,
}

impl Resource {
    /// 新建 resource
    pub fn new(name: impl Into<String>, capacity: f64, unit_cost: f64) -> Self {
        Self {
            id: ResourceId(Uuid::new_v4()),
            name: name.into(),
            capacity,
            unit_cost,
        }
    }
}

/// TaskAssignment (task ↔ resource 分配, per MS Project assignment)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskAssignment {
    /// Task ID
    pub task_id: TaskId,
    /// Resource ID
    pub resource_id: ResourceId,
    /// 分配比例 (0.0-1.0, 1.0 = 100% allocation)
    pub allocation: f64,
}

impl TaskAssignment {
    /// 新建 assignment
    pub fn new(task_id: TaskId, resource_id: ResourceId, allocation: f64) -> Self {
        Self {
            task_id,
            resource_id,
            allocation,
        }
    }
}

// ============================================================================
// §6 Schedule (per MS Project schedule = 集合)
// ============================================================================

/// Schedule (per MS Project schedule = tasks + dependencies + milestones + resources + assignments)
#[derive(Debug)]
pub struct Schedule {
    /// Schedule ID
    pub id: ScheduleId,
    /// Schedule 名称
    pub name: String,
    /// Task 仓库 (task_id → Task)
    pub tasks: HashMap<TaskId, Task>,
    /// Dependency 仓库 (dep_id → Dependency)
    pub dependencies: HashMap<DependencyId, Dependency>,
    /// Milestone 仓库 (milestone_id → Milestone)
    pub milestones: HashMap<MilestoneId, Milestone>,
    /// Resource 仓库 (resource_id → Resource)
    pub resources: HashMap<ResourceId, Resource>,
    /// Assignment 仓库
    pub assignments: Vec<TaskAssignment>,
}

impl Schedule {
    /// 新建空 schedule
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: ScheduleId(Uuid::new_v4()),
            name: name.into(),
            tasks: HashMap::new(),
            dependencies: HashMap::new(),
            milestones: HashMap::new(),
            resources: HashMap::new(),
            assignments: Vec::new(),
        }
    }
    /// 加 task
    pub fn add_task(&mut self, task: Task) {
        self.tasks.insert(task.id, task);
    }
    /// 加 dependency
    pub fn add_dependency(&mut self, dep: Dependency) {
        self.dependencies.insert(dep.id, dep);
    }
    /// Task 数量
    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }
}

// ============================================================================
// §7 Scheduler (核心 CPM 算法, per plan-032 R8 line 130 拓扑序 + 最早/最晚开始时间)
// ============================================================================

/// Schedule 错误
#[derive(Debug, Error)]
pub enum SchedulerError {
    /// Task 不存在
    #[error("task not found: {0}")]
    TaskNotFound(String),
    /// 检测到循环依赖
    #[error("cycle detected in task graph: {0:?}")]
    CycleDetected(Vec<TaskId>),
    /// 引用不存在的 task
    #[error("references non-existent task: {0}")]
    ReferencesMissingTask(String),
}

/// 关键路径节点 (per CPM algorithm 输出)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CpmNode {
    /// Task ID
    pub task_id: TaskId,
    /// 最早开始时间 (ES, earliest start)
    pub earliest_start: u64,
    /// 最早完成时间 (EF, earliest finish)
    pub earliest_finish: u64,
    /// 最晚开始时间 (LS, latest start)
    pub latest_start: u64,
    /// 最晚完成时间 (LF, latest finish)
    pub latest_finish: u64,
    /// 总时差 (slack = LS - ES, 0 = 关键路径)
    pub slack: u64,
    /// 是否关键路径
    pub is_critical: bool,
}

/// Scheduler (CPM 核心算法, per plan-032 R8 line 130)
#[derive(Debug, Default)]
pub struct Scheduler;

impl Scheduler {
    /// 新建 scheduler
    pub fn new() -> Self {
        Self
    }

    /// 拓扑序排序 (Kahn's algorithm, per plan-032 R8 line 130)
    ///
    /// 返 Vec<TaskId> 按拓扑序 (前置在前, 后置在后)
    pub fn topologically_sort(&self, schedule: &Schedule) -> Result<Vec<TaskId>, SchedulerError> {
        // 入度表
        let mut in_degree: HashMap<TaskId, usize> = HashMap::new();
        for task_id in schedule.tasks.keys() {
            in_degree.insert(*task_id, 0);
        }
        // 邻接表 (from → [to, ...])
        let mut adj: HashMap<TaskId, Vec<TaskId>> = HashMap::new();
        for task in schedule.tasks.values() {
            for pred in &task.predecessors {
                if !schedule.tasks.contains_key(pred) {
                    return Err(SchedulerError::ReferencesMissingTask(format!("{:?}", pred)));
                }
                *in_degree.entry(task.id).or_insert(0) += 1;
                adj.entry(*pred).or_default().push(task.id);
            }
        }
        // Kahn BFS
        let mut queue: VecDeque<TaskId> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(id, _)| *id)
            .collect();
        let mut result = Vec::new();
        while let Some(task_id) = queue.pop_front() {
            result.push(task_id);
            if let Some(successors) = adj.get(&task_id) {
                for succ in successors {
                    if let Some(deg) = in_degree.get_mut(succ) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push_back(*succ);
                        }
                    }
                }
            }
        }
        if result.len() != schedule.tasks.len() {
            // 剩 in_degree > 0 的 task = 环上
            let cycle_tasks: Vec<TaskId> = in_degree
                .iter()
                .filter(|(_, &deg)| deg > 0)
                .map(|(id, _)| *id)
                .collect();
            return Err(SchedulerError::CycleDetected(cycle_tasks));
        }
        Ok(result)
    }

    /// 检测循环 (有环返 true)
    pub fn detect_cycles(&self, schedule: &Schedule) -> bool {
        matches!(
            self.topologically_sort(schedule),
            Err(SchedulerError::CycleDetected(_))
        )
    }

    /// 计算关键路径 (CPM algorithm, per plan-032 R8 line 130)
    ///
    /// 返 Vec<CpmNode> 按拓扑序, 每个 node 含 ES/EF/LS/LF/slack/is_critical
    pub fn critical_path(&self, schedule: &Schedule) -> Result<Vec<CpmNode>, SchedulerError> {
        let topo = self.topologically_sort(schedule)?;
        let mut nodes: HashMap<TaskId, CpmNode> = HashMap::new();

        // Forward pass: 计算 ES / EF
        for &task_id in &topo {
            let task = schedule
                .tasks
                .get(&task_id)
                .ok_or_else(|| SchedulerError::TaskNotFound(format!("{:?}", task_id)))?;
            let es = if task.predecessors.is_empty() {
                0
            } else {
                task.predecessors
                    .iter()
                    .map(|p| nodes.get(p).map(|n| n.earliest_finish).unwrap_or(0))
                    .max()
                    .unwrap_or(0)
            };
            let ef = es + task.duration as u64;
            nodes.insert(
                task_id,
                CpmNode {
                    task_id,
                    earliest_start: es,
                    earliest_finish: ef,
                    latest_start: 0,
                    latest_finish: 0,
                    slack: 0,
                    is_critical: false,
                },
            );
        }

        // 计算 project end (= max EF)
        let project_end = nodes.values().map(|n| n.earliest_finish).max().unwrap_or(0);

        // Backward pass: 计算 LS / LF (从拓扑逆序)
        for &task_id in topo.iter().rev() {
            let task = schedule
                .tasks
                .get(&task_id)
                .ok_or_else(|| SchedulerError::TaskNotFound(format!("{:?}", task_id)))?;
            // 找后置 task: 任何 predecessor 包含本 task 的 task
            let successors_lf: Vec<u64> = schedule
                .tasks
                .values()
                .filter(|t| t.predecessors.contains(&task_id))
                .filter_map(|t| nodes.get(&t.id).map(|n| n.latest_start))
                .collect();
            let lf = if successors_lf.is_empty() {
                project_end
            } else {
                successors_lf.iter().min().copied().unwrap_or(project_end)
            };
            let ls = lf.saturating_sub(task.duration as u64);
            let node = nodes.get_mut(&task_id).unwrap();
            node.latest_start = ls;
            node.latest_finish = lf;
            node.slack = ls.saturating_sub(node.earliest_start);
            node.is_critical = node.slack == 0;
        }

        // 按拓扑序返回
        Ok(topo
            .iter()
            .map(|id| nodes.get(id).cloned().unwrap())
            .collect())
    }

    /// 关键路径上的 task IDs (slack = 0, is_critical = true)
    pub fn critical_path_tasks(&self, schedule: &Schedule) -> Result<Vec<TaskId>, SchedulerError> {
        let cpm = self.critical_path(schedule)?;
        Ok(cpm
            .iter()
            .filter(|n| n.is_critical)
            .map(|n| n.task_id)
            .collect())
    }
}

// ============================================================================
// §8 Tests (R8 阶段 1 PoC 验证)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_schedule() -> (Schedule, TaskId, TaskId, TaskId, TaskId) {
        // A (dur 3) → B (dur 2) → D (dur 4)
        // A → C (dur 5) → D
        // 关键路径: A → C → D (3+5+4=12), B 是 3+2+4=9, slack 3
        let mut sched = Schedule::new("R8 PoC");
        let a = Task::new("A", 3);
        let mut b = Task::new("B", 2);
        let mut c = Task::new("C", 5);
        let mut d = Task::new("D", 4);
        let a_id = a.id;
        let b_id = b.id;
        let c_id = c.id;
        let d_id = d.id;
        b.add_predecessor(a_id);
        c.add_predecessor(a_id);
        d.add_predecessor(b_id);
        d.add_predecessor(c_id);
        sched.add_task(a);
        sched.add_task(b);
        sched.add_task(c);
        sched.add_task(d);
        (sched, a_id, b_id, c_id, d_id)
    }

    #[test]
    fn dependency_type_default_is_finish_to_start() {
        assert_eq!(DependencyType::default(), DependencyType::FinishToStart);
    }

    #[test]
    fn task_new_and_add_predecessor() {
        let mut task = Task::new("test", 5);
        let pred = TaskId(Uuid::new_v4());
        task.add_predecessor(pred);
        assert_eq!(task.predecessors.len(), 1);
        assert_eq!(task.predecessors[0], pred);
    }

    #[test]
    fn dependency_new_finish_to_start_default() {
        let from = TaskId(Uuid::new_v4());
        let to = TaskId(Uuid::new_v4());
        let dep = Dependency::new(from, to);
        assert_eq!(dep.dep_type, DependencyType::FinishToStart);
        assert_eq!(dep.lag, 0);
    }

    #[test]
    fn dependency_with_type_and_lag() {
        let from = TaskId(Uuid::new_v4());
        let to = TaskId(Uuid::new_v4());
        let dep = Dependency::with_type_and_lag(from, to, DependencyType::StartToStart, -2);
        assert_eq!(dep.dep_type, DependencyType::StartToStart);
        assert_eq!(dep.lag, -2);
    }

    #[test]
    fn schedule_add_task_and_dependency() {
        let mut sched = Schedule::new("test");
        let task1 = Task::new("A", 3);
        let task2 = Task::new("B", 2);
        let t1_id = task1.id;
        let dep = Dependency::new(t1_id, task2.id);
        sched.add_task(task1);
        sched.add_task(task2);
        sched.add_dependency(dep);
        assert_eq!(sched.task_count(), 2);
        assert_eq!(sched.dependencies.len(), 1);
    }

    #[test]
    fn scheduler_topological_sort_linear() {
        let (sched, a, b, _c, d) = sample_schedule();
        let scheduler = Scheduler::new();
        let topo = scheduler.topologically_sort(&sched).unwrap();
        // A 必在 B/C 前, B/C 必在 D 前
        let pos = |id: TaskId| topo.iter().position(|x| *x == id).unwrap();
        assert!(pos(a) < pos(b));
        assert!(pos(a) < pos(_c));
        assert!(pos(b) < pos(d));
        assert!(pos(_c) < pos(d));
    }

    #[test]
    fn scheduler_detect_cycles() {
        let mut sched = Schedule::new("cycle");
        let mut a = Task::new("A", 1);
        let mut b = Task::new("B", 1);
        a.add_predecessor(b.id);
        b.add_predecessor(a.id); // 循环!
        sched.add_task(a);
        sched.add_task(b);
        let scheduler = Scheduler::new();
        assert!(scheduler.detect_cycles(&sched));
    }

    #[test]
    fn scheduler_critical_path_identifies_a_c_d() {
        // A (3) → C (5) → D (4) = 12, 关键路径
        // A → B (2) → D = 9, slack 3
        let (sched, a, b, c, d) = sample_schedule();
        let scheduler = Scheduler::new();
        let cpm = scheduler.critical_path(&sched).unwrap();
        // 验证 ES/EF
        let a_node = cpm.iter().find(|n| n.task_id == a).unwrap();
        assert_eq!(a_node.earliest_start, 0);
        assert_eq!(a_node.earliest_finish, 3);
        let c_node = cpm.iter().find(|n| n.task_id == c).unwrap();
        assert_eq!(c_node.earliest_start, 3);
        assert_eq!(c_node.earliest_finish, 8);
        let d_node = cpm.iter().find(|n| n.task_id == d).unwrap();
        assert_eq!(d_node.earliest_start, 8);
        assert_eq!(d_node.earliest_finish, 12);
        // B 有 slack 3
        let b_node = cpm.iter().find(|n| n.task_id == b).unwrap();
        assert_eq!(b_node.slack, 3);
        assert!(!b_node.is_critical);
        // 关键路径 = [A, C, D]
        let crit = scheduler.critical_path_tasks(&sched).unwrap();
        assert_eq!(crit.len(), 3);
        assert!(crit.contains(&a));
        assert!(crit.contains(&c));
        assert!(crit.contains(&d));
        assert!(!crit.contains(&b));
    }

    // ========================================================================
    // R9 阶段 2: Task → SharedTask 转换 UT (per DD-SHARED-TASK-001 §4.4)
    // ========================================================================

    #[test]
    fn task_to_shared_task_conversion() {
        let mut task = Task::new("MS Project task", 5);
        task.issue_key = Some("STAR-001".to_string());
        let original_id = task.id.0;
        let shared: shared_task::SharedTask = task.into();
        // id: star-scheduler::TaskId (local newtype) → shared_task::TaskId
        assert_eq!(shared.id.0, original_id);
        assert_eq!(shared.title, "MS Project task");
        // description / created_at 走 Schedule 代理, 暂默认 (per DD §4.4 + §7 缺口 #6)
        assert_eq!(shared.description, "");
        assert_eq!(shared.state, shared_task::TaskState::Open);
        assert_eq!(shared.assignee, None);
        assert_eq!(shared.priority, shared_task::Priority::Medium);
        assert_eq!(shared.issue_key, Some("STAR-001".to_string()));
    }
}
