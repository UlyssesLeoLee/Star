# DD-SHARED-TASK-001

> **3 view 共享 Task Schema 设计 (Rust pivot 整合层) v0.1**
>
> 跨 3 view (Jira + Miro + MS Project) 共享 Task entity 表达, 准备 R9 阶段 2 整合 (7 crate 类型映射)
>
> - **状态**: 🟢 Draft v0.1 (2026-09-12, R9 阶段 1 设计阶段, 不动现有 7 crate)
> - **目标阶段**: R9 阶段 2 整合 (per plan-032 R9 line 138-144) + R9 阶段 3 benchmark (5 milestone)
> - **关联 ADR**: [ADR-0027 v0.1 §2.3.2 共享 Task schema 跨 3 view](../adr/0027-rust-pivot-agent-game.md)
> - **关联 plan**: [plan-032 R9 整合 + 性能 benchmark](../plans/plan-032-rust-pivot-agent-game.md) line 134-144
> - **关联 7 crate**: star-task / star-workflow / star-canvas / star-scheduler / star-game / star-registry
> - **修订人**: `Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核**`
> - **审批**: `架构师 (Mavis 接手 agent per DEC-008)` (per 守门 #14 v4)
> - **作者**: 2026-09-12 JST, Mavis 起草
> - **dual-use 提醒**: 本 DD 跨 Star 仓 7 crate, 不引用 RGS 仓 + 不建立业务子域↔DDD 映射 (per 守门 #3 disclaimer)

---

## §0 文档信息 / 修订历史

| 项目 | 内容 |
|---|---|
| 文档 ID | DD-SHARED-TASK-001 |
| 文档名 | 3 view 共享 Task Schema 设计 (Rust pivot 整合层) |
| 版本 | v0.1 |
| 创建日 | 2026-09-12 |
| 修订人 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核** (per DEC-008) |
| 审批 | 架构师 (Mavis 接手 agent per DEC-008) — per 守门 #14 v4 |
| dual-use | 跨 Star 仓 7 crate, 不引用 RGS 仓 + 不建立业务子域↔DDD 映射 (per 守门 #3 disclaimer) |
| 跟 5 域 disclaimer 关系 | 5 域独立 Lead ≠ Star 22 DDD bounded context (per 守门 #3 拍板, 文档加 disclaimer) |
| 跟守门 #3 关系 | "5 域独立 Lead" 是 RGS 仓历史治理命名, 不等于 Star 仓 22 DDD bounded context; 不建立业务子域↔DDD 映射 |

### 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-12 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核** | 初版 (R9 阶段 1 共享 Task schema 设计: 8 字段 + 3 view 映射表 + 7 crate 集成表 + 5 milestone 验证计划) | 2026-09-12 09:20 JST Ulysses 拍板 R9 阶段 1 + per plan-032 R9 line 138-144 |

---

## §1 范围

### 1.1 目标

设计 3 view (Jira + Miro + MS Project) 共享 Task entity schema, 跨域表达, 准备 R9 阶段 2 整合 (7 crate 类型映射) + R9 阶段 3 benchmark (5 性能 milestone 验证).

**R9 阶段 1 = 设计阶段, 不动现有 7 crate 代码**. R9 阶段 2 才动现有 crate (per ask_user 拍板, R9 阶段 1 选项说明).

### 1.2 范围 (in-scope)

| # | 范围项 | 阶段 | 来源 |
|---|---|---|---|
| 1 | 共享 Task schema 8 字段设计 | R9 阶段 1 | ADR-0027 §2.3.2 |
| 2 | 3 view 映射表 (Jira/Miro/MS Project) | R9 阶段 1 | plan-032 R9 line 141 |
| 3 | 现有 7 crate 集成表 | R9 阶段 1 | R3-R8 已落档 6 crate + R4 star-task 跨域 |
| 4 | 5 性能 milestone 验证计划 | R9 阶段 1 | plan-032 §4.1 |
| 5 | 守门核对 + 已知缺口 + 签字栏 | R9 阶段 1 | per AGENTS.md §3 7 段结构 |

### 1.3 范围外 (out-of-scope)

- **不动现有 7 crate 代码** (per R9 阶段 1 拍板, 仅设计文档)
- **R9 阶段 2+ 才动现有 crate** (类型映射, derive macro 自动生成, 整合 7 crate 字段)
- **R9 阶段 3 性能 benchmark 实测** (criterion + cargo bench) 留后续
- **真实 RGS 仓代码引用** (per dual-use 提醒 + 守门 #3 disclaimer)

---

## §2 共享 Task Schema 设计 (8 字段)

### 2.1 字段定义 (per WBS row 标准 + Multica opaque ID)

```rust
/// 共享 Task entity (WBS row, 跨 3 view 表达, per ADR-0027 §2.3.2)
pub struct SharedTask {
    /// 1️⃣ ID (TaskId newtype, per Multica opaque ID 1:1 派生)
    pub id: TaskId,
    /// 2️⃣ 标题 (e.g. "R5 阶段 1 实装", per Jira Issue.summary / Miro Node.content / MS Project Task.name)
    pub title: String,
    /// 3️⃣ 描述 (per Jira Issue.description / Miro Node.content 复用)
    pub description: String,
    /// 4️⃣ 状态 (5 态状态机, per Jira 默认 workflow)
    pub state: TaskState,
    /// 5️⃣ 负责人 (e.g. "Mavis" / "5 域 Lead 名" / "subagent-1", per 守门 #3 + 9/11 23:11 JST 强化)
    pub assignee: Option<String>,
    /// 6️⃣ 优先级 (per Jira priority scheme)
    pub priority: Priority,
    /// 7️⃣ Issue Key (业务主键, e.g. "STAR-001", WBS row 跨 3 view 表达)
    pub issue_key: Option<String>,
    /// 8️⃣ 创建时间 (per Multica created_at)
    pub created_at: SystemTime,
}
```

### 2.2 TaskState 5 态 (per Jira 默认 workflow, per plan-032 R6 line 99-108)

| 状态 | Jira 对应 | Miro 对应 | MS Project 对应 | agent (star-game) 对应 |
|---|---|---|---|---|
| `Open` | Open | (隐式, 新创建) | (隐式, ES=0) | `Available` (Quest) |
| `InProgress` | In Progress | (有 position) | (ES>0, 进行中) | `InProgress` (Quest) |
| `InReview` | In Review | (有 preview) | (ES<EF, review 中) | (无, R9 整合) |
| `Done` | Done | (有完成标记) | (EF 达成) | `Completed` (Quest) |
| `Closed` | Closed | (隐式, 不显示) | (项目结束) | `Failed` (Quest) |

**注**: star-task 7 态 (claim/queue/run/review/done/...) → 共享 TaskState 5 态映射, 详见 §4.1.

### 2.3 Priority 4 档 (per Jira priority scheme)

| 优先级 | Jira 对应 | Miro 对应 | MS Project 对应 | 备注 |
|---|---|---|---|---|
| `Low` | Low | (无) | (无) | 低优先级 |
| `Medium` | Medium (default) | (无) | (无, 资源容量代理) | 默认 |
| `High` | High | (无) | (无) | 高优先级 |
| `Critical` | Critical / Blocker | (无) | (无) | 紧急 |

### 2.4 设计原则

1. **不删现有 7 crate 私有字段** (per 守门 #11 缺标比错标) — 共享 schema 是 view-specific 字段的并集, 现有 crate 保留各自 view-specific 字段
2. **8 字段是 WBS row 标准最小集** (per ADR-0027 §2.3.2) — 3 view 各自的扩展字段 (e.g. Miro `position: Position3D`, MS Project `duration: u32`) 不进入共享 schema
3. **TaskId newtype 暂用本 crate 内部** (跟 R5 阶段 1 TaskId 同样模式), R9 阶段 3 统一 path 依赖
4. **per 守门 #3 disclaimer**: 5 域独立 Lead ≠ Star 22 DDD bounded context, 共享 Task 不建立业务子域↔DDD 映射

---

## §3 3 view 映射表 (Jira + Miro + MS Project)

### 3.1 字段映射 (per 8 字段)

| # | 字段 | 共享 Task | Jira (`star-workflow`) | Miro (`star-canvas`) | MS Project (`star-scheduler`) | agent (`star-game`) |
|---|---|---|---|---|---|---|
| 1 | ID | `TaskId` | `IssueKey` (e.g. "STAR-001") | `NodeId` (UUID) | `TaskId` (UUID) | `TaskId` (UUID, 跟 R5 阶段 1 同 newtype) |
| 2 | Title | `title` | `Issue.summary` | `Node.content` (Text/Sticky) | `Task.name` | `Quest.title` |
| 3 | Description | `description` | `Issue.description` | (Node.content 复用) | (无, 走 Schedule.description) | `Quest.description` |
| 4 | State | `state` (5 态) | `WorkflowState` (5 态, per R6) | (无显式, 通过 Node.position 隐式) | (通过 Task.duration + CpmNode 派生) | `Quest.status` (4 态) |
| 5 | Assignee | `assignee` | `Issue.assignee` | `Node.created_by` | `TaskAssignment.resource_id` | `Agent.name` |
| 6 | Priority | `priority` | `Issue.priority` (4 档) | (无) | (无, 资源容量代理) | (无, 用 `Mana.tokens` 代理) |
| 7 | Issue Key | `issue_key` | `Issue.key` (主键) | `Node.issue_key` (跨域) | `Task.issue_key` (跨域) | (无, R9 整合时加) |
| 8 | Created | `created_at` | `Issue.created_at` | (无, 走 `Cursor` 代理) | (无, 走 `Schedule.created`) | `Quest.created_at` |

### 3.2 3 view 转换规则 (R9 阶段 2 实装)

#### 3.2.1 Jira Issue → 共享 Task

```rust
// 伪代码
fn issue_to_shared_task(issue: &Issue) -> SharedTask {
    SharedTask {
        id: TaskId(issue.key.0.clone()),  // IssueKey → TaskId
        title: issue.summary.clone(),
        description: issue.description.clone(),
        state: map_jira_to_shared_state(issue.state),  // WorkflowState → TaskState (per §4.1)
        assignee: issue.assignee.clone(),
        priority: issue.priority,
        issue_key: Some(issue.key.0.clone()),
        created_at: issue.created_at,
    }
}
```

#### 3.2.2 Miro Node → 共享 Task

```rust
fn node_to_shared_task(node: &Node) -> SharedTask {
    SharedTask {
        id: TaskId(node.id.0),  // NodeId → TaskId (UUID 直转)
        title: node.content.clone(),
        description: String::new(),  // Miro 不分离 description
        state: TaskState::Open,  // Miro 无显式 state, 默认 Open
        assignee: Some(node.created_by.clone()),
        priority: Priority::default(),  // Medium
        issue_key: node.issue_key.clone(),
        created_at: SystemTime::now(),  // Miro 暂不存 created_at, R9 阶段 2 修复
    }
}
```

#### 3.2.3 MS Project Task → 共享 Task

```rust
fn scheduler_task_to_shared_task(task: &Task) -> SharedTask {
    SharedTask {
        id: task.id,  // TaskId (UUID) 直转
        title: task.name.clone(),
        description: String::new(),  // MS Project 暂不存 description, R9 阶段 2 修复
        state: TaskState::Open,  // MS Project 状态通过 CpmNode 派生, R9 阶段 2 修复
        assignee: None,  // 走 TaskAssignment 派生
        priority: Priority::default(),  // Medium
        issue_key: task.issue_key.clone(),
        created_at: SystemTime::now(),  // 走 Schedule.created 代理
    }
}
```

### 3.3 agent (star-game) Quest → 共享 Task

```rust
fn quest_to_shared_task(quest: &Quest) -> SharedTask {
    SharedTask {
        id: quest.id,  // TaskId (UUID, 跟 R5 阶段 1 同 newtype)
        title: quest.title.clone(),
        description: quest.description.clone(),
        state: map_quest_to_shared_state(quest.status),  // 4 态 → 5 态
        assignee: None,  // Quest 不存 assignee, 走 GameLoop.agent.name
        priority: Priority::default(),  // Medium
        issue_key: None,  // R9 整合时加
        created_at: quest.created_at,
    }
}
```

---

## §4 现有 7 crate 集成表

### 4.1 star-task 7 态 → 共享 TaskState 5 态映射

| star-task (7 态) | 共享 TaskState (5 态) | 备注 |
|---|---|---|
| `Pending` (0) | `Open` | 初始 |
| `Claimed` (1) | `Open` | 已认领但未开始 |
| `Queued` (2) | `Open` | 排队中 |
| `Running` (3) | `InProgress` | 执行中 |
| `Review` (4) | `InReview` | 评审中 |
| `Completed` (5) | `Done` | 完成 |
| `Failed` (6) | `Failed` → 重新打开 `Open` | 失败, retry 时回 Open |
| (无) | `Closed` | 跟 `Done` 区别: Closed = 业务关闭, Done = 技术完成 |

**映射函数** (R9 阶段 2 实装):

```rust
fn map_star_task_to_shared_state(s: star_task::TaskState) -> TaskState {
    match s {
        star_task::TaskState::Pending => TaskState::Open,
        star_task::TaskState::Claimed => TaskState::Open,
        star_task::TaskState::Queued => TaskState::Open,
        star_task::TaskState::Running => TaskState::InProgress,
        star_task::TaskState::Review => TaskState::InReview,
        star_task::TaskState::Completed => TaskState::Done,
        star_task::TaskState::Failed => TaskState::Open,  // retry 时
    }
}
```

### 4.2 star-workflow Issue ↔ 共享 Task (1:1)

| 共享 Task 字段 | star-workflow Issue 字段 | 转换 |
|---|---|---|
| `id` | `IssueKey` (e.g. "STAR-001") | `TaskId(issue.key.0)` |
| `title` | `Issue.summary` | 直转 |
| `description` | `Issue.description` | 直转 |
| `state` | `WorkflowState` (5 态) | per §4.1 映射 (WorkflowState = TaskState 几乎 1:1) |
| `assignee` | `Issue.assignee` | 直转 |
| `priority` | `Issue.priority` | 直转 (4 档相同) |
| `issue_key` | `Issue.key` | 直转 (主键) |
| `created_at` | `Issue.created_at` | 直转 |

**保留字段** (Issue 私有, 不进入共享 Task):
- `workflow_id: WorkflowId` (Workflow 关联)
- `comments: Vec<Comment>` (per Issue 评论)
- `attachments: Vec<Attachment>` (per Issue 附件)
- `cross_domain_deps: HashMap<IssueKey, Vec<IssueKey>>` (跨域依赖)

### 4.3 star-canvas Node ↔ 共享 Task (1:1)

| 共享 Task 字段 | star-canvas Node 字段 | 转换 |
|---|---|---|
| `id` | `NodeId` (UUID) | `TaskId(node.id.0)` |
| `title` | `Node.content` (Text/Sticky 节点) | 直转 |
| `description` | (无) | `String::new()` (R9 阶段 2 修复) |
| `state` | (无显式) | `TaskState::Open` 默认 (R9 阶段 2 通过 Node.position 派生) |
| `assignee` | `Node.created_by` | `Some(node.created_by.clone())` |
| `priority` | (无) | `Priority::default()` |
| `issue_key` | `Node.issue_key` (跨域) | 直转 |
| `created_at` | (无) | `SystemTime::now()` (R9 阶段 2 修复) |

**保留字段** (Node 私有, 不进入共享 Task):
- `position: Position3D` (3D 坐标, per R7 line 114)
- `size: Size3D` (3D 尺寸)
- `kind: NodeKind` (Rect/Text/Sticky/Image/Group, per R7 line 137-145)

### 4.4 star-scheduler Task ↔ 共享 Task (1:1)

| 共享 Task 字段 | star-scheduler Task 字段 | 转换 |
|---|---|---|
| `id` | `TaskId` (UUID) | 直转 |
| `title` | `Task.name` | 直转 |
| `description` | (无) | `String::new()` (R9 阶段 2 修复) |
| `state` | (无显式) | `TaskState::Open` 默认 (R9 阶段 2 通过 CpmNode 派生) |
| `assignee` | (无) | 走 `TaskAssignment.resource_id` |
| `priority` | (无) | `Priority::default()` |
| `issue_key` | `Task.issue_key` (跨域) | 直转 |
| `created_at` | (无) | `Schedule.id` 创建时间代理 (R9 阶段 2 修复) |

**保留字段** (Task 私有, 不进入共享 Task):
- `duration: u32` (per R8 line 165)
- `predecessors: Vec<TaskId>` (CPM 依赖)
- `Dependency { from, to, dep_type, lag }` (FS/SS/FF/SF 4 档)
- `Milestone` + `Resource` + `TaskAssignment` (per R8 line 200-220)

### 4.5 star-game Quest ↔ 共享 Task (1:1)

| 共享 Task 字段 | star-game Quest 字段 | 转换 |
|---|---|---|
| `id` | `TaskId` (UUID, 跟 R5 阶段 1 同 newtype) | 直转 |
| `title` | `Quest.title` | 直转 |
| `description` | `Quest.description` | 直转 |
| `state` | `Quest.status` (4 态) | per §3.3 映射 |
| `assignee` | (无) | 走 `GameLoop.agent.name` |
| `priority` | (无) | `Priority::default()` |
| `issue_key` | (无) | `None` (R9 整合时加) |
| `created_at` | `Quest.created_at` | 直转 |

**保留字段** (Quest 私有, 不进入共享 Task):
- `mana_cost: u32` (per R5 阶段 1)
- `xp_reward: u64` (per R5 阶段 1)
- `GameLoop` (assign/success/fail/restart, per R5 阶段 1 + R5 阶段 2)

**Agent 6 维属性** (per R5 阶段 1 §2.2.1, 不进入共享 Task):
- `Health` / `Mana` / `Xp` / `SkillTree` / `Inventory` / `Cooldown`
- 跟 R10 "5 角色 Rust 化" 整合 (per plan-032 R10 line 150)

### 4.6 star-registry (5 域 Lead 真人内容)

per 9/11 23:11 JST 强化 + 守门 #3 disclaimer:
- 5 域 Lead 真人内容 = 共享 `Task.assignee` 字段的合法值
- 5 域 Lead 真人姓名由 Mavis 默认决定 (per 9/11 23:11 JST 强化)
- 5 域独立 Lead ≠ Star 22 DDD bounded context (per 守门 #3 disclaimer, 不建立业务子域↔DDD 映射)
- R10 "5 角色 Rust 化" 整合 (per plan-032 R10 line 150-155): `enum Lead { Architecture, Sre, Platform, Reviewer, Pm }`

### 4.7 canvas-collab (A12 多人编辑 域 骨架, per R6 阶段 1 关联)

canvas-collab 跟 star-canvas 同源 (per AGENTS.md 守门 #3 disclaimer, 5 域独立 Lead 不引用 RGS 仓). 共享 Task 整合时 canvas-collab 走 star-canvas 间接关联.

---

## §5 5 性能 milestone 验证计划 (per plan-032 §4.1)

### 5.1 5 milestone 表

| # | Metric | 测试方法 | 目标 | Python / 商业对比 | 对应 crate |
|---|---|---|---|---|---|
| 1 | 1000 provider 探测 | `star-registry::probe_all()` 测 1000 个 mock provider | < 200ms | `scripts/automation/registry/scan.py` 1-2K 秒 (10-20x 加速) | `star-registry` |
| 2 | CPM 10K task | `star-scheduler::critical_path(10K task graph)` | < 100ms | MS Project 类似工作 30s+ (300x 加速) | `star-scheduler` |
| 3 | CRDT 100 节点并发编辑 | `star-canvas::concurrent_edit(100 client)` 收敛时间 | < 50ms | Miro 类似 500ms+ (10x 加速) | `star-canvas` |
| 4 | WBS 5 态状态机 100K task 吞吐 | `star-task::transition_batch(100K)` | > 100K task/秒 | Jira 类似 10K task/秒 (10x 加速) | `star-task` |
| 5 | Agent ECS 10K entity 60fps | `star-game::ecs::update(10K entity)` frame time | < 16.7ms (60fps) | Physis 优化前 30fps (2x 加速) | `star-game` |

### 5.2 benchmark 工具

- **Rust 端**: `criterion` + `cargo bench` (per 守门 #1 v25 cargo test 单 crate 实证)
- **对照组**: Python 端 `timeit` + Jira REST API mock + Miro WebSocket mock + MS Project CPM mock
- **报告**: `docs/reports/benchmarks/rust-pivot-v0.1.md` (per 守门 #1 v15 docs 同步)
- **不触达守门 v29**: 单 docs commit < 3, 不大批量

### 5.3 milestone 验证节奏

| 阶段 | milestone 验证项 | 状态 |
|---|---|---|
| R5 完成 | Agent ECS 10K entity 60fps (#5) | ⏳ (R5 阶段 3 ECS 实装后验证) |
| R9 完成 | 5 milestone 全过 + 实测报告落地 | ⏳ (R9 阶段 3 实测) |
| R10 完成 | 5 角色 Rust 化 + 真人到位追溯 | ⏳ (R10 阶段 1 实证) |

### 5.4 R9 阶段 1 不实测

per 9/1 14:58 守门 + 9/5 04:03 守门 + 9/8 15:29 Mavis 自驱:
- **R9 阶段 1 = 设计文档**, 不实测 5 milestone
- **R9 阶段 2 = 整合 7 crate 类型映射**, 不实测
- **R9 阶段 3 = benchmark 实测**, 走 `criterion` + `cargo bench` 实证 5 milestone

---

## §6 守门核对 (per AGENTS.md 守门 + plan-032 R9 line 138-144)

### 6.1 守门合规矩阵

| 守门 | 规则 | R9 阶段 1 合规 |
|---|---|---|
| **#1 v25** | cargo test 单 crate 实证 | ✅ 7 crate 0 err (R3-R8 已落档 41-49 UT) |
| **#1 v19** | cargo check --workspace --lib -j 4 0 err | ✅ 7 crate workspace check 0 err (R3-R8 commit 实证) |
| **#1 v15** | docs 同步饱和 必新事件触发 | ✅ R9 阶段 1 = 用户拍板 = 新事件触发 |
| **#3** | 5 域独立 Lead ≠ Star 22 DDD bounded context | ✅ §0 文档头 disclaimer + §2.4 设计原则 + §4.6 显式说明 |
| **#6** | PowerShell only | ✅ 全程 PowerShell |
| **#7** | 0 unsafe | ✅ 设计阶段不动代码 |
| **#7 fmt/clippy** | 0 diff / 0 warning | ✅ 设计阶段不动代码 |
| **#11** | 缺标比错标 | ✅ §7 已知缺口 4 项显式列 |
| **#12 v21** | [P] docs 同步 必更新 §4 + registry | ⏳ (本 DD 是设计, 非任务卡, R9 阶段 2 实装时更新) |
| **#14 v4** | Mavis 审核 author=Ulysses | ✅ 修订人/审批形式合规 |
| **#19 v19** | 0 动 V0.1 任何业务 logic | ✅ R9 阶段 1 不动现有 7 crate |
| **#29** | docs 同步饱和 30/40/50 阈值 | ✅ 单 docs commit < 3, 不触达大批量 (per R9 阶段 1 拍板放行) |

### 6.2 dual-use 提醒

- 本 DD 跨 Star 仓 7 crate, **不引用 RGS 仓**
- **不建立业务子域↔DDD 映射** (per 守门 #3 disclaimer)
- 5 域独立 Lead = RGS 仓历史治理命名, ≠ Star 22 DDD bounded context
- 跨 RGS 仓时, 优先 Star 仓 (per 9/10 20:45 JST "Ulysses 双仓并行")

### 6.3 跟 plan-032 R9 关系

- plan-032 R9 line 138-144: 3 view 共享 schema + 实测 vs Jira/Miro/MS Project + 5 性能 milestone 验证
- R9 阶段 1 = 共享 schema 设计 (本 DD)
- R9 阶段 2 = 整合 7 crate (per plan-032 R9 line 140 依赖 R6 + R7 + R8)
- R9 阶段 3 = benchmark 实测 (per plan-032 R9 line 142, criterion + cargo bench)

---

## §7 已知缺口 (per 守门 #11 缺标比错标)

### 缺口 #1: R9 阶段 1 = 设计文档, 不实装

R9 阶段 1 仅是设计文档, 不实装 7 crate 类型映射. R9 阶段 2+ 才动现有 7 crate 代码.

**缓解**: R9 阶段 2 拍板前必先 ask_user (per 9/1 14:58 守门 + 9/8 16:08 推荐项), 推荐项放第 1 位.

### 缺口 #2: 共享 TaskId newtype 暂用本 crate 内部

共享 TaskId 暂用本 DD §2.1 描述的内部 newtype (跟 R5 阶段 1 TaskId 同样模式, R4 star-task 也有 TaskId newtype), 3 处独立 newtype, 阶段 3 统一 path 依赖.

**缓解**: 阶段 3 统一 path 依赖 (跟 R5 阶段 1 决策同 per §0 已知缺口), 1 笔整合 commit.

### 缺口 #3: 3 view 字段映射是表格式手动维护

§3 字段映射表是手动维护, 阶段 2 改用 derive macro 自动生成 (e.g. `#[derive(From)]` / `#[derive(Into)]`).

**缓解**: 阶段 2 引入 `derive_more` crate + 自研 derive macro.

### 缺口 #4: 5 性能 milestone benchmark 实测留 R9 阶段 3

R9 阶段 1 仅是验证计划, 不实测 5 milestone. R9 阶段 3 走 `criterion` + `cargo bench` 实证.

**缓解**: 阶段 3 拍板前必先 ask_user (per 守门 v28 推荐项), 推荐项放第 1 位.

### 缺口 #5: Miro `description` 字段缺失

Miro Node 不分离 description (per §4.3), R9 阶段 1 默认 `String::new()`. 阶段 2 修复 (e.g. 加 `Node.description: Option<String>`).

### 缺口 #6: MS Project `created_at` 走 Schedule 代理

MS Project Task 不存 created_at, R9 阶段 1 默认 `SystemTime::now()`. 阶段 2 修复 (e.g. 加 `Task.created_at: SystemTime` 或走 `Schedule.id` 创建时间派生).

### 缺口 #7: agent Quest 缺 `issue_key` 跨域关联

star-game Quest 不存 issue_key, R9 阶段 1 默认 `None`. 阶段 2 修复 (加 `Quest.issue_key: Option<String>`, 跟 star-workflow Issue 1:1 关联).

### 缺口 #8: 真人到位 5 域 Lead 决策机制 (per 守门 #3 + 9/11 23:11)

5 域 Lead 真人内容由 Mavis 默认决定 (per 9/11 23:11 JST 强化), 真人到位后追溯签字覆盖修订历史 (per 守门 #1 v15 禁回溯叙事). R10 阶段 1 整合 `enum Lead { Architecture, Sre, Platform, Reviewer, Pm }` (per plan-032 R10 line 150).

---

## §8 签字栏 (5 角色 per AGENTS.md §3)

| 角色 | 签字 | 形式 (per 守门 #14 v4) |
|---|---|---|
| 架构 | 架构师 (Mavis 接手 agent per DEC-008) | 永久代签 per 守门 #10 + 8/27 19:39 JST 授权 + 9/3 11:35 JST 反转 |
| SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 同上, 5 域 Lead 拒绝兼任 per 8/21 JST 拍板 |
| 平台 | 架构师 (Mavis 接手 agent per DEC-008) | 同上 |
| 评审主持 | 架构师 (Mavis 接手 agent per DEC-008) | 同上 |
| PM | 架构师 (Mavis 接手 agent per DEC-008) | 同上 |

**注**: 5 角色全部由 Mavis 永久代签 (per 守门 #14 v3 + v4 + 9/3 11:35 JST 反转), author=Ulysses, 修订人=Mavis 接手**审核** (per 9/10 12:45 JST v0.62 反转 + 守门 #14 v4).

**5 域 Lead 真人到位追溯签字机制** (per 守门 #14 v2 + 9/3 19:35 JST 拍板 D + 9/5 10:43 JST 拍板 D 维持):
- 启动信号 = Ulysses 发令"5 域 Lead 真人到位流程激活"
- 追溯签字形式 = 修订历史表 +1 行 (per §1.2 T5 + §4 缺口 #2 实证)
- 真人决策 vs Mavis 代签决策 = 独立审计链 (per 守门 #1 禁回溯叙事)

---

## §9 R9 阶段 1 关联事项

### 9.1 跟 7 crate 关系

| crate | 当前状态 (R3-R8) | R9 阶段 1 (本 DD) | R9 阶段 2 (整合) |
|---|---|---|---|
| `star-task` | R4 阶段 1 (`8c2bde9`) | §4.1 7→5 态映射 | 7 crate 互引 |
| `star-workflow` | R6 阶段 1 (`86a37f5`) | §4.2 Issue ↔ 共享 Task 1:1 | 整合 |
| `star-canvas` | R7 阶段 1 (`5365a8e`) | §4.3 Node ↔ 共享 Task 1:1 | 整合 |
| `star-scheduler` | R8 阶段 1 (`0db2085`) | §4.4 Task ↔ 共享 Task 1:1 | 整合 |
| `star-game` | R5 阶段 1+2 (`c0c1b71` + `ce38e8a`) | §4.5 Quest ↔ 共享 Task 1:1 | 整合 |
| `star-registry` | R3 阶段 1 (`495d27d`) | §4.6 5 域 Lead 真人内容 | 整合 (R10 阶段 1 重点) |
| `canvas-collab` | (R6 阶段 1 关联) | §4.7 走 star-canvas 间接 | 整合 |

### 9.2 后续阶段计划

| 阶段 | 产出 | 时间估 | 依赖 |
|---|---|---|---|
| R9 阶段 1 (本 DD) | 共享 Task schema 设计 | 0.3-0.5M token | R6 + R7 + R8 (✅ 已落档) |
| R9 阶段 2 | 整合 7 crate 类型映射 | 1-1.5M token | R9 阶段 1 |
| R9 阶段 3 | 5 milestone benchmark 实测 + 实测报告 | 0.5-1M token | R9 阶段 2 + criterion + cargo bench |
| R10 阶段 1 | 5 角色 Rust 化 (`enum Lead`) | 0.5-1M token | R9 阶段 2 |
| R10 阶段 2 | 真人到位追溯签字 + 修订历史 +1 行 | 0.3-0.5M token | R10 阶段 1 + Ulysses 发令 |

### 9.3 守门 #1 v15 docs 同步饱和

R9 阶段 1 = 1 docs commit (本 DD), 累计 docs sync 计数 = 53 (R1 拍板后) + 1 (本 commit) = 54. **未触达 30/40/50 阈值**, 守门 #1 v15 + 守门 v29 ✅ 放行.

后续 R9 阶段 2/3 + R10 阶段 1/2 累计 docs sync, 触达 50 ERROR 阈值前必先 ask_user 拍板 (per 守门 v29 激活门槛).

### 9.4 守门 #6 PowerShell only

本 DD 通过 PowerShell write 工具写入, UTF-8 编码, 避免 garbled (per 守门 #6 + 系统约束).

---

## §10 参考资料

- [ADR-0027 v0.1 §2.3.2 共享 Task schema 跨 3 view](../adr/0027-rust-pivot-agent-game.md)
- [plan-032 R9 line 134-144 整合 + 性能 benchmark](../plans/plan-032-rust-pivot-agent-game.md)
- [plan-032 §4.1 5 milestone 性能 benchmark 设计](../plans/plan-032-rust-pivot-agent-game.md)
- [DD-MULTICA-TASK-001 v0.1 star-task 7 态设计](../design/DD-MULTICA-TASK-001.md)
- [star-task crate (R4 阶段 1 commit 8c2bde9)](../../crates/star-task/src/lib.rs)
- [star-workflow crate (R6 阶段 1 commit 86a37f5)](../../crates/star-workflow/src/lib.rs)
- [star-canvas crate (R7 阶段 1 commit 5365a8e)](../../crates/star-canvas/src/lib.rs)
- [star-scheduler crate (R8 阶段 1 commit 0db2085)](../../crates/star-scheduler/src/lib.rs)
- [star-game crate (R5 阶段 1+2 commit c0c1b71 + ce38e8a)](../../crates/star-game/src/lib.rs)
- [star-registry crate (R3 阶段 1 commit 495d27d)](../../crates/star-registry/src/lib.rs)
- [AGENTS.md §3 7 段结构 + §4 守门硬约束](../../AGENTS.md)
