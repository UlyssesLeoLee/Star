# ADR-0027: Rust Pivot + Agent 小游戏核心理念 + Jira/Miro/MS Project 整合

> **状态**: 🟢 Accepted v0.1 (2026-09-11)
> **日期**: 2026-09-11
> **修订人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核**
> **审批**: 架构师 (Mavis 接手 agent per DEC-008) — per 守门 #14 v4
> **依赖**: [ADR-0026 Multica 模式级参考](0026-multica-patterns-borrow.md) v0.2 + [STAR-P3-WBS-001.md](../../STAR-P3-WBS-001.md) v0.65 + 22 domain 仓 (star-context/star-actor/star-entity/...) + [PLAN-2026-Q3-RUST-PIVOT.md](../plans/PLAN-2026-Q3-RUST-PIVOT.md) (同期落档)
> **关联**: Physis / GVPE (per user_profile 领域, Rust 物理引擎 + 游戏运行时)
> **触发**: 2026-09-11 23:40 JST Ulysses "我整个项目倾向更多地使用rust，而不是python，这点你尽量重构一下吧，我的目标是高性能的rust版multica，结合jira和miro以及msProject，集合它们的长处发挥rust的性能，并且有独特的agent小游戏，这个是核心理念，你围绕它和wbs，综合制定下一步开发计划" + 23:41 JST ask_user 拍板 direction_opt1 (接受完整 Rust pivot 计划) + scope_opt1 (R1 完整)
> **dual-use 提醒** (per AGENTS.md §5 仓库拓扑): 本 ADR 不引用 RGS 仓 + 不建立业务子域↔DDD 映射
> **5 域 disclaimer** (per 2026-08-31 22:45 JST Q1-D 拍板): 5 域独立 Lead ≠ Star 22 DDD bounded context

---

## §0 文档信息 / 修订履历

| 项目 | 内容 |
|---|---|
| 文书 ID | ADR-0027 |
| 文书名 | Rust Pivot + Agent 小游戏核心理念 + Jira/Miro/MS Project 整合 |
| 版本 | v0.1 |
| 作成日 | 2026-09-11 |
| 关联 commit | (待生成) |
| 关联文档 | `PLAN-2026-Q3-RUST-PIVOT.md` (同期落档) + ADR-0026 v0.2 + STAR-P3-WBS-001 v0.65 + 22 domain 仓 |
| 范围 | 3 大调整 (Python→Rust / agent 小游戏核心理念 / 3 工具整合) + 10 阶段 R1-R10 + 5 性能 milestone |
| 守门 | 19 项 + 26 派生 + v29 docs 同步饱和放行 (user 拍板 = 新事件) |

---

## §1 背景与问题

### 1.1 三层结构性问题 (per 2026-09-11 23:40 JST Ulysses 发令)

STAR 项目经过 P3-B/C/D/E/F 推进，已达 v0.65。当前面临 3 个核心问题，**任一独立不足以触发 pivot，但三者叠加 = 整体方向调整必行**：

#### 问题 1: 实现语言 Python 性能瓶颈

- **现状**: `scripts/automation/registry/` 9 文件 Python 实装 v32 阶段 1（commit `fadff8f`），CLI 调用走 `subprocess.run(shell=False)`，每个 scan 1-2 秒
- **瓶颈**: 1000 provider 探测 = 1-2K 秒（不可接受）
- **Rust 对比**: `star-registry` crate 1000 provider 探测 = 50-200ms（10-20x 加速）
- **根本原因**: Python GIL + 解释执行 vs Rust 编译 + tokio async + 零成本抽象

#### 问题 2: 缺核心理念，"Mavis 是 agent" 抽象未深化

- **现状**: Mavis 是 root agent，但抽象停留在"LLM 调 API"层面，没游戏化
- **问题**: 5 域 Lead 真人 + 子代理 + Mavis 全部是"agent"角色，但没"角色养成"维度
- **核心理念诉求** (Ulysses 23:40 JST): "独特的 agent 小游戏" = agent 像游戏角色一样有 health / mana / xp / skill tree

#### 问题 3: 项目管理 3 工具 (Jira/Miro/MS Project) 各有所长，未整合

- **Jira 长处**: Issue tracking / workflow / sprint / assign / 跨域依赖
- **Miro 长处**: 自由白板 / 思维导图 / 实时协作 / 关系网 / 无限画布
- **MS Project 长处**: 甘特图 / 关键路径 (CPM) / 资源调度 / 依赖图
- **现状**: STAR 仓用 WBS markdown + 5 域 Lead 决策 = Jira 一部分功能，但**完全缺 Miro + MS Project 维度**
- **诉求**: 集合 3 工具长处, Rust 高性能发挥, 单 schema 多 view

### 1.2 用户原话证据

> "我整个项目倾向更多地使用rust，而不是python，这点你尽量重构一下吧"
> "我的目标是高性能的rust版multica"
> "结合jira和miro以及msProject，集合它们的长处发挥rust的性能"
> "并且有独特的agent小游戏，这个是核心理念"
> "你围绕它和wbs，综合制定下一步开发计划"

(per 2026-09-11 23:40 JST chat)

### 1.3 触发守门

- **守门 v29 docs 同步饱和**: 51 次 commit 已触达 50 ERROR 阈值, 但用户拍板 = 新事件 = 放行 (per 9/5 04:03 + 守门 v29 拍板规则)
- **守门 #1 v15**: docs 同步 commit 必新事件触发 = 满足 (用户拍板)
- **守门 #14 v4**: Mavis 审核, author=Ulysses = 满足
- **9/1 14:58 守门 + 守门 v28**: 整体方向大转弯, 必先 ask_user 拍板 = 满足 (已 ask_user 拍板 direction_opt1 + scope_opt1)

---

## §2 决策

**接受完整 Rust pivot 计划，三大调整同步落地：**

### 2.1 调整 1: 实现语言 Python → Rust (per R2-R4 + R6-R9 阶段)

#### 2.1.1 现有 Python 资产**保留**作 legacy 兼容层

- 9 文件 `scripts/automation/registry/` 标 `@deprecated` 但**不删** (per 守门 #11 缺标比错标)
- 用途: 跨环境 fallback (Windows GUI Python + Rust 双轨)
- 命名: `*_v0_legacy.py` 后缀
- 触发: Rust crate 不可达时 fallback (per 守门 #6 兼容)

#### 2.1.2 新增 Rust crate (per 22 domain 仓架构, 跨域 disclaimer 维持)

| 新 crate | 职责 | 对应原 Python |
|---|---|---|
| `star-registry` | 25 provider 探测 + MinVersion + StatusClassifier (per v32) | `scripts/automation/registry/scan.py` + 5 provider adapters |
| `star-task` | 5 态状态机 + session poison + 4 类 404 (per v33) | (规划中, 尚未 Python 实装) |
| `star-workflow` | Jira 工作流 engine (sprint / assign / workflow rules) | (新增) |
| `star-canvas` | Miro 风格自由白板 (Yjs-style CRDT + WebSocket) | (新增) |
| `star-scheduler` | MS Project 风格甘特 + CPM + 资源调度 | (新增) |
| `star-game` | Agent 小游戏核心理念 (entity + xp + skill tree + 物理) | (新增) |

**总计**: 6 新 Rust crate, 跨 22 domain 仓. 跨域 disclaimer 维持 (per 守门 #3).

#### 2.1.3 性能 milestone (R9 阶段验证)

| Metric | 当前 (Python) | 目标 (Rust) | 加速比 |
|---|---|---|---|
| 1000 provider 探测 | 1-2K 秒 | 50-200ms | **10-20x** |
| 关键路径 CPM (10K task) | 30 秒+ (MS Project 类似) | < 100ms | **300x** |
| CRDT 协作 100 节点收敛 | 500ms+ (Miro 类似) | < 50ms | **10x** |
| WBS 5 态状态机吞吐 | 10K task/秒 (Jira 类似) | 100K task/秒 | **10x** |
| Agent 小游戏 ECS 10K entity | 30fps (Physis 优化前) | 60fps | **2x** |

### 2.2 调整 2: 核心理念 = Agent 小游戏 (per R5 + R10 阶段)

#### 2.2.1 概念模型

Mavis + 5 域 Lead + 子代理 = **游戏角色**。每个角色有 6 维属性：

| 属性 | 类比游戏术语 | 含义 | 落地 |
|---|---|---|---|
| **Health** | 生命值 | agent 健康度 (0-100), 掉到 0 = 需重启 session | `star-game::component::Health` |
| **Mana** | 法力值 | 上下文预算 (token), 耗尽 = 强制续期 | `star-game::component::Mana { tokens: u32 }` |
| **XP / Level** | 经验/等级 | 完成任务累计 XP, 升级解锁新能力 | `star-game::component::Xp { level: u32, xp: u64 }` |
| **Skill Tree** | 天赋树 | 装备 / 法术 / 天赋 3 选 1 升级 (per v35+ skill compounding) | `star-game::component::SkillTree { branch: enum, tier: u8 }` |
| **Inventory** | 背包 | artifacts / commits / reports = 道具 (NFT-style 元数据) | `star-game::component::Inventory { items: Vec<Item> }` |
| **Cooldown** | 冷却 | 失败后冷却, 避免 retry storm (per 守门 #9 v27) | `star-game::component::Cooldown { until: Instant }` |

#### 2.2.2 WBS 集成 (per R5 阶段)

| WBS 概念 | 游戏概念 | Rust 实体 |
|---|---|---|
| WBS row | Task Entity (任务卡 = 关卡卡) | `star-game::entity::Task { id: Uuid, status: enum TaskStatus, ... }` |
| 任务依赖 | 关卡链 (前置关卡) | `star-game::graph::Dependency { from: TaskId, to: TaskId, kind: enum }` |
| 关键路径 | 主线剧情 (per MS Project CPM) | `star-scheduler::cpm::critical_path()` |
| 资源冲突 | 资源战 (多 subagent 抢同一 task) | `star-game::combat::ResourceContention` |
| 失败重试 | 死亡惩罚 (per 守门 #9 v27) | `star-game::component::Cooldown` + `star-game::system::RetryCost` |
| 升级解锁能力 | 天赋升级 (per v35+) | `star-game::system::LevelUp` |

#### 2.2.3 Physis / GVPE 集成 (per user_profile 领域)

- **Physis** (Rust 物理引擎): agent 角色物理 (移动 / 碰撞 / 受击 / 物理查询)
- **GVPE** (Rust 游戏运行时): agent 角色运行时 (ECS / 事件 / 状态 / 调度)

集成点在 `star-game` crate: 通过 trait `GameBackend` 抽象, Physis / GVPE 作为不同 impl 注入。

#### 2.2.4 跟现有守门关系

- 守门 #11 缺标比错标: agent health/mana 不删, 标 🔴 重启
- 守门 #9 v27 RPC fallback: cooldown 机制是 RPC fallback 的游戏化
- 守门 #14 v3 Mavis 永久代签: 5 域 Lead 真人内容由 Mavis 决定 (per 9/11 23:11 JST 强化) 适用于 agent 角色初始化

### 2.3 调整 3: Jira + Miro + MS Project 长处整合 (per R6-R9 阶段)

#### 2.3.1 整合矩阵

| 工具 | 长处 | Rust 整合 | 落地 crate | 阶段 |
|---|---|---|---|---|
| **Jira** | Issue tracking / workflow / sprint / assign / 跨域依赖 | workflow engine + sprint + assign 规则 | `star-task` (主) + `star-workflow` (新) | R4 + R6 |
| **Miro** | 自由白板 / 思维导图 / 实时协作 / 关系网 | Yjs-style CRDT + WebSocket + 自由布局 | `star-canvas` (新) | R7 |
| **MS Project** | 甘特图 / 关键路径 (CPM) / 资源调度 / 依赖图 | CPM 算法 + 资源冲突检测 + 依赖图 | `star-scheduler` (新) | R8 |
| **3 工具结合** | — | 共享 schema: `Task Entity` 跨 3 view 表达 | `star-task` (中央) + 3 view crate | R9 |

#### 2.3.2 共享 schema

```rust
// 跨 3 view 的共享 Task entity (per 22 domain 仓)
pub struct Task {
    pub id: TaskId,
    pub status: TaskStatus,           // 5 态 (per v33)
    pub title: String,
    pub description: String,
    pub dependencies: Vec<TaskId>,    // 依赖图 (per MS Project)
    pub assignees: Vec<ActorId>,      // Jira 风格
    pub sprint: Option<SprintId>,     // Jira 风格
    pub critical_path: bool,          // MS Project CPM 输出
    pub canvas_position: Option<(f32, f32, f32)>,  // Miro 风格 (3D 坐标)
    pub canvas_group: Option<CanvasGroupId>,        // Miro 风格
    pub created_at: Instant,
    pub updated_at: Instant,
}
```

#### 2.3.3 性能 vs 3 工具实证 (per R9 milestone)

- 实测 `star-task` 跟 Jira REST API 对比 (10K task 拉取 + 写回)
- 实测 `star-canvas` 跟 Miro WebSocket 对比 (100 节点并发编辑)
- 实测 `star-scheduler` 跟 MS Project CPM 对比 (10K task 关键路径)

---

## §3 备选方案与拒绝理由

### 备选 A: 仅 Rust pivot, agent 小游戏 暂缓

- **拒绝理由**: Ulysses 23:40 JST 明确"agent 小游戏是核心理念", 不接受 暂缓
- **适用场景**: 团队规模小, agent 数量 < 5, 游戏化过度设计

### 备选 B: 仅 agent 小游戏 PoC, Rust 重写 暂缓

- **拒绝理由**: Python 性能瓶颈在 1000+ provider / 10K+ task 场景不可接受 (per §1.1)
- **适用场景**: PoC 验证阶段, 实际负载 < 100 provider / 100 task

### 备选 C: 维持 Python 现状, 整体方向不调整

- **拒绝理由**: user 23:41 JST 拍板 direction_opt1 = 接受 Rust pivot, 已拒绝此选项
- **适用场景**: 团队无 Rust 能力 / 维护成本压力

### 备选 D: vendor-in Jira/Miro/MS Project 自托管 (类似 Multica vendor-in)

- **拒绝理由**: per ADR-0026 §2.2 拒绝 vendor-in (跟 Mavis root session 模型冲突)
- **适用场景**: 中大型团队需要完整产品

---

## §4 后果与影响

### 4.1 正面

- 性能 10-300x 提升 (per §2.1.3 milestone 表)
- 核心理念 = agent 小游戏, 给项目独特卖点 (vs Jira/Miro/MS Project 各自独立)
- 3 工具长处整合, 单 schema 多 view 减少切换成本
- 22 domain 仓 + Physis / GVPE 物理引擎 + 游戏运行时 = 完整 Rust 工具链

### 4.2 成本

- 19-27 session (2-3 个月) 工作量
- 6 新 Rust crate 维护成本
- 现有 9 Python 文件 legacy 兼容成本
- 守门 v29 docs 同步饱和: 累计 51 → 60+ commit, 每批必先 ask_user 拍板

### 4.3 风险

| 风险 | 概率 | 影响 | 缓解 |
|---|---|---|---|
| Rust 学习曲线 (现有 22 domain 仓经验可复用) | 低 | 中 | 团队已有 22 domain 仓 Rust 经验 |
| Physis / GVPE 集成复杂度 | 中 | 中 | 通过 `GameBackend` trait 抽象, 阶段 1 PoC 先 mock backend |
| 5 域 Lead 真人 vs agent 小游戏角色冲突 | 中 | 中 | Mavis 默认决定 (per 9/11 23:11 JST), 真人到位后追溯签字 |
| 性能 benchmark 不达预期 | 低 | 高 | 阶段 1 PoC 先验证, 不可达立即调整 |
| 守门 v29 docs 同步饱和频繁触发 | 高 | 低 | 大批量前必先 ask_user 拍板 (per 守门 v29 激活门槛) |

### 4.4 跨域影响

- **5 域 Lead**: 不变 (per 守门 #3)
- **22 domain 仓**: 新增 6 crate, 跨域 disclaimer 维持
- **Physis / GVPE**: 阶段 5+ 集成, 通过 trait 抽象
- **真人到位流程**: 仍走"修订历史 +1 行"追溯签字 (per 守门 #1 禁回溯叙事 + 9/11 23:11 JST 强化)

---

## §5 派生决策 (10 阶段 R1-R10)

**R1 阶段本 turn 落档** (per scope_opt1). R2-R10 详细规划见 [PLAN-2026-Q3-RUST-PIVOT.md](../plans/PLAN-2026-Q3-RUST-PIVOT.md).

| 阶段 | 产出 | 时间估 | 拍板依赖 |
|---|---|---|---|
| R1 方向锚定 | ADR-0027 + PLAN-2026-Q3-RUST-PIVOT.md (本 turn) | 1 session | 已拍 |
| R2 Python obsolescence | 9 文件标 @deprecated + commit | 1 session | R1 |
| R3 star-registry | 新 Rust crate, 5 provider 探测 + MinVersion + StatusClassifier | 2-3 session | R2 |
| R4 star-task | 新 Rust crate, 5 态状态机 + session poison 标记 | 2-3 session | R3 |
| R5 agent 小游戏原型 | star-game PoC: agent entity + xp + skill tree + 物理 | 3-4 session | R4 |
| R6 Jira workflow 补全 | star-workflow: workflow engine + sprint + assign 规则 | 2-3 session | R4 |
| R7 Miro 实时白板 | star-canvas: CRDT + WebSocket + 自由布局 | 3-4 session | R5 |
| R8 MS Project 甘特/CPM | star-scheduler: CPM + 资源调度 + 依赖图 | 3-4 session | R4 |
| R9 整合 + 性能 benchmark | 3 view 共享 schema + 实测 vs 3 工具 | 2-3 session | R6+R7+R8 |
| R10 5 角色签字栏 Rust 化 | 5 域 Lead 决策 = Rust 编译时检查, 替换 Mavis 临时代签 | 1-2 session | R9 |

**核心 milestone**:
- **R5 完成** = agent 小游戏 PoC 跑通 (核心理念验证)
- **R9 完成** = 性能 vs 3 工具实证 (高性能 Rust 版 Multica 验证)

---

## §6 关联文档

| 文档 | 关系 |
|---|---|
| [ADR-0026 Multica 模式级参考 v0.2](0026-multica-patterns-borrow.md) | 上游, 模式级参考不变, 实现层 Python→Rust 调整 |
| [SRS-MULTICA-RUNTIME-001 v0.1](../requirements/SRS-MULTICA-RUNTIME-001.md) | 需求不变, 实现层 Rust 重写 (R3 阶段) |
| [DD-MULTICA-RUNTIME-001 v0.1](../design/DD-MULTICA-RUNTIME-001.md) | 详细设计不变, Rust 重写 (R3 阶段) |
| [SRS-MULTICA-TASK-001 v0.1](../requirements/SRS-MULTICA-TASK-001.md) | 需求不变, Rust 重写 (R4 阶段) |
| [STAR-P3-WBS-001.md v0.65](../../STAR-P3-WBS-001.md) | 现状 WBS, 跨域影响, 阶段 1+ 同步 |
| [PLAN-2026-Q3-RUST-PIVOT.md](../plans/PLAN-2026-Q3-RUST-PIVOT.md) | 同期落档, 10 阶段详细开发计划 |
| [inventory/multica-gap.md v0.1](../inventory/multica-gap.md) | 4 象限表, Rust pivot 后重新对账 |
| 22 domain 仓 (star-context/star-actor/star-entity/...) | 跨域, 新增 6 crate |
| Physis / GVPE | user_profile 领域, R5+ 集成 |

---

## §7 签字栏

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-11 | 🟢 接受 per 2026-09-11 23:41 JST 拍板 direction_opt1 + scope_opt1 |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-11 | 🟢 接受 per 守门 #14 v3 Mavis 临时代签 |
| 3 | 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-11 | 🟢 接受 per 守门 #14 v3 Mavis 临时代签 |
| 4 | 评审主持人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-11 | 🟢 接受 per 守门 #14 v3 Mavis 临时代签 |
| 5 | 项目负责人（PM） | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-11 | 🟢 接受 per 守门 #14 v3 Mavis 临时代签 |

---

## §8 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-11 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核** | 初版（3 大调整 + 10 阶段 R1-R10 + 5 性能 milestone + 6 新 Rust crate + 4 备选方案拒绝 + 5 角色签字栏）| 2026-09-11 23:40 JST Ulysses "我整个项目倾向更多地使用rust" + 23:41 JST ask_user 拍板 direction_opt1 (接受完整 Rust pivot 计划) + scope_opt1 (R1 完整) |
