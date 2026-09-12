//! star-game — Agent 小游戏核心理理念 (R5 阶段 1 PoC) 🟢
//!
//! Per ADR-0027 v0.1 §2.2 + plan-032 R5: **核心理念** = Mavis + 5 域 Lead + 子代理 = 游戏角色.
//! 6 维属性 (Health / Mana / XP+Level / SkillTree / Inventory / Cooldown) + ECS-style
//! 1 agent entity + 1 task (Quest) 闭环 + GameBackend trait (Physis 集成点).
//!
//! WBS 集成 (per ADR-0027 §2.2.2):
//! - WBS row = Task Entity (关卡卡)
//! - 任务依赖 = 关卡链
//! - 关键路径 = 主线剧情 (per MS Project CPM, R8 实装)
//! - 资源冲突 = 资源战 (多 subagent 抢同一 task)
//! - 失败重试 = 死亡惩罚 (cooldown 机制, per 守门 #9 v27)
//! - 升级解锁能力 = 天赋升级 (per v35+ skill compounding, R5 占位)
//!
//! Physis / GVPE 集成 (per user_profile 领域, 通过 GameBackend trait):
//! - 阶段 1: MockBackend (no-op, 验证 trait 抽象)
//! - 阶段 2: Physis 物理后端 (Physis 0.1.x 实装 + Collision 反馈)
//! - 阶段 3: GVPE 游戏运行时 (ECS + 事件 + 调度)
//!
//! 跨域 (per 守门 #1 跨域 consults):
//! - star-task: TaskId newtype 1:1 派生 (暂用本 crate local newtype, 后续 R9 整合)
//! - star-registry: 5 域 Lead 真人内容由 Mavis 决定 (per 9/11 23:11 JST 强化)
//!
//! 守门合规 (per 守门 #1 v25 cargo test 单 crate 实证 + 守门 #7 0 unsafe)

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use std::collections::VecDeque;
use std::time::{Duration, Instant, SystemTime};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

// ============================================================================
// §1 ID newtype
// ============================================================================

/// Agent ID (per Multica `AgentId` opaque)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(pub Uuid);

impl From<Uuid> for AgentId {
    fn from(id: Uuid) -> Self {
        Self(id)
    }
}

/// Task ID (per star-task 1:1 派生, 暂 local newtype, R9 整合时换 path 依赖)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(pub Uuid);

impl From<Uuid> for TaskId {
    fn from(id: Uuid) -> Self {
        Self(id)
    }
}

/// Item ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ItemId(pub Uuid);

// ============================================================================
// §2 6 维属性 (per ADR-0027 §2.2.1)
// ============================================================================

/// 1️⃣ Health 生命值 (0-100), 掉到 0 = 需重启 session
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Health {
    /// 当前生命值
    pub value: u32,
    /// 生命值上限
    pub max: u32,
}

impl Health {
    /// 默认 100 生命值
    pub fn new() -> Self {
        Self {
            value: 100,
            max: 100,
        }
    }
    /// 受到伤害, 返是否死亡 (value 掉到 0)
    pub fn take_damage(&mut self, amount: u32) -> bool {
        self.value = self.value.saturating_sub(amount);
        self.value == 0
    }
    /// 治疗恢复
    pub fn heal(&mut self, amount: u32) {
        self.value = (self.value + amount).min(self.max);
    }
}

impl Default for Health {
    fn default() -> Self {
        Self::new()
    }
}

/// 2️⃣ Mana 法力值 (token 预算), 耗尽 = 强制续期
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mana {
    /// 当前 token 数
    pub tokens: u32,
    /// token 上限
    pub max: u32,
}

impl Mana {
    /// 默认 1000 token (per Multica context window 默认)
    pub fn new() -> Self {
        Self {
            tokens: 1000,
            max: 1000,
        }
    }
    /// 消耗 token, 返是否够用
    pub fn consume(&mut self, amount: u32) -> bool {
        if self.tokens >= amount {
            self.tokens -= amount;
            true
        } else {
            false
        }
    }
    /// 续期
    pub fn replenish(&mut self, amount: u32) {
        self.tokens = (self.tokens + amount).min(self.max);
    }
}

impl Default for Mana {
    fn default() -> Self {
        Self::new()
    }
}

/// 3️⃣ XP / Level 经验/等级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Xp {
    /// 当前等级 (1-N)
    pub level: u32,
    /// 当前等级累计 XP
    pub xp: u64,
    /// 升级到下一级所需 XP (公式: level * 100)
    pub xp_to_next_level: u64,
}

impl Xp {
    /// 默认 level 1, 0 XP
    pub fn new() -> Self {
        Self {
            level: 1,
            xp: 0,
            xp_to_next_level: 100,
        }
    }
    /// 获得 XP, 自动检查升级 (per ADR-0027 §2.2.2 升级解锁能力)
    /// 返是否升级 (可能一次跨多级)
    pub fn gain(&mut self, amount: u64) -> bool {
        self.xp += amount;
        let mut leveled_up = false;
        while self.xp >= self.xp_to_next_level {
            self.xp -= self.xp_to_next_level;
            self.level += 1;
            self.xp_to_next_level = (self.level as u64) * 100;
            leveled_up = true;
        }
        leveled_up
    }
}

impl Default for Xp {
    fn default() -> Self {
        Self::new()
    }
}

/// 4️⃣ SkillTree 天赋树 (per v35+ skill compounding, R5 占位)
///
/// 3 选 1 升级: 装备 / 法术 / 天赋 (per ADR-0027 §2.2.1)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillBranch {
    /// 装备分支 (artifact 强化)
    Equipment,
    /// 法术分支 (LLM 调用优化)
    Spell,
    /// 天赋分支 (跨域协调能力)
    Talent,
}

/// 4️⃣ SkillTree 天赋树 (per ADR-0027 §2.2.1)
///
/// 3 选 1 升级分支 (Equipment / Spell / Talent) + 多 tier (升级解锁能力)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillTree {
    /// 当前分支 (3 选 1)
    pub branch: SkillBranch,
    /// 当前 tier (1-N, 升级解锁)
    pub tier: u8,
}

impl SkillTree {
    /// 默认 Talent 1 (跨域协调)
    pub fn new() -> Self {
        Self {
            branch: SkillBranch::Talent,
            tier: 1,
        }
    }
    /// 切换分支 (per ADR-0027 §2.2.1 装备/法术/天赋 3 选 1 升级)
    pub fn change_branch(&mut self, new_branch: SkillBranch) {
        self.branch = new_branch;
    }
    /// 升级 tier (解锁能力)
    pub fn upgrade_tier(&mut self) {
        self.tier = self.tier.saturating_add(1);
    }
}

impl Default for SkillTree {
    fn default() -> Self {
        Self::new()
    }
}

/// Item 类型 (per ADR-0027 §2.2.1 Inventory 道具)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemKind {
    /// commit 记录
    Commit,
    /// 输出文件
    Output,
    /// 报告
    Report,
    /// 工具/技能升级 token
    Token,
}

/// Item 道具 (per ADR-0027 §2.2.1 Inventory 元素)
///
/// 4 类: Commit / Output / Report / Token, 携带 NFT-style metadata
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Item {
    /// 道具 ID
    pub id: ItemId,
    /// 道具类型
    pub kind: ItemKind,
    /// 道具元数据 (per ADR-0027 §2.2.1 NFT-style metadata)
    pub metadata: serde_json::Value,
}

/// 5️⃣ Inventory 背包 (artifacts / commits / reports = 道具)
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Inventory {
    /// 道具列表
    pub items: Vec<Item>,
}

impl Inventory {
    /// 添加道具
    pub fn add(&mut self, item: Item) {
        self.items.push(item);
    }
    /// 移除道具 by id
    pub fn remove(&mut self, id: ItemId) -> Option<Item> {
        self.items
            .iter()
            .position(|i| i.id == id)
            .map(|idx| self.items.remove(idx))
    }
    /// 道具数量
    pub fn len(&self) -> usize {
        self.items.len()
    }
    /// 是否空
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

/// 6️⃣ Cooldown 冷却 (per 守门 #9 v27 RPC fallback 防 retry storm)
///
/// 不 derive Serialize/Deserialize 因为 `Instant` 不实现 serde (PoC 仅内存, 持久化走 star-task)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cooldown {
    /// 冷却到什么时候 (None = 未冷却)
    pub until: Option<Instant>,
}

impl Cooldown {
    /// 无冷却
    pub fn new() -> Self {
        Self { until: None }
    }
    /// 设置冷却
    pub fn apply(&mut self, duration: Duration) {
        self.until = Some(Instant::now() + duration);
    }
    /// 是否在冷却中
    pub fn is_active(&self) -> bool {
        self.until.is_some_and(|t| Instant::now() < t)
    }
    /// 清除冷却
    pub fn clear(&mut self) {
        self.until = None;
    }
}

impl Default for Cooldown {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// §3 Agent entity (per Multica ECS 风格)
// ============================================================================

/// Agent entity (per ADR-0027 §2.2.1 游戏角色 = Mavis / 5 域 Lead / 子代理)
///
/// 不 derive Serialize/Deserialize (SystemTime + Cooldown 不 impl serde, PoC 仅内存)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Agent {
    /// Agent ID
    pub id: AgentId,
    /// Agent 名称 (e.g. "Mavis" / "5 域 Lead Player" / "subagent-1")
    pub name: String,
    /// 1️⃣ 生命值
    pub health: Health,
    /// 2️⃣ 法力值 (token 预算)
    pub mana: Mana,
    /// 3️⃣ 经验/等级
    pub xp: Xp,
    /// 4️⃣ 天赋树
    pub skill_tree: SkillTree,
    /// 5️⃣ 背包
    pub inventory: Inventory,
    /// 6️⃣ 冷却
    pub cooldown: Cooldown,
    /// 创建时间
    pub created_at: SystemTime,
}

impl Agent {
    /// 新建 agent (Mavis 初始化默认值, 5 域 Lead 真人内容由 Mavis 决定 per 9/11 23:11 JST 强化)
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: AgentId(Uuid::new_v4()),
            name: name.into(),
            health: Health::new(),
            mana: Mana::new(),
            xp: Xp::new(),
            skill_tree: SkillTree::new(),
            inventory: Inventory::default(),
            cooldown: Cooldown::new(),
            created_at: SystemTime::now(),
        }
    }

    /// 是否死亡 (health = 0, 需重启 session)
    pub fn is_dead(&self) -> bool {
        self.health.value == 0
    }

    /// 是否在冷却中
    pub fn in_cooldown(&self) -> bool {
        self.cooldown.is_active()
    }
}

// ============================================================================
// §4 Quest (WBS row = 关卡卡, per ADR-0027 §2.2.2)
// ============================================================================

/// Quest 状态 (per WBS 5 态状态机简化, 跟 star-task 7 态映射)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuestStatus {
    /// 可接 (WBS pending)
    Available,
    /// 进行中 (WBS in_progress)
    InProgress,
    /// 完成 (WBS completed)
    Completed,
    /// 失败 (WBS failed)
    Failed,
}

/// Quest (WBS row = 关卡卡, per ADR-0027 §2.2.2)
///
/// 不 derive Serialize/Deserialize (SystemTime 不 impl serde, PoC 仅内存)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Quest {
    /// Task ID (跟 star-task 1:1 派生)
    pub id: TaskId,
    /// 任务标题
    pub title: String,
    /// 任务描述
    pub description: String,
    /// 接任务消耗 mana (token 预算)
    pub mana_cost: u32,
    /// 完成获得 XP
    pub xp_reward: u64,
    /// 当前状态
    pub status: QuestStatus,
    /// 创建时间
    pub created_at: SystemTime,
}

impl Quest {
    /// 新建 quest
    pub fn new(
        title: impl Into<String>,
        description: impl Into<String>,
        mana_cost: u32,
        xp_reward: u64,
    ) -> Self {
        Self {
            id: TaskId(Uuid::new_v4()),
            title: title.into(),
            description: description.into(),
            mana_cost,
            xp_reward,
            status: QuestStatus::Available,
            created_at: SystemTime::now(),
        }
    }
}

// ============================================================================
// §4.5 Quest → SharedTask From impl (R9 阶段 2 整合, per DD-SHARED-TASK-001 §4.5)
// ============================================================================

/// Quest → SharedTask 转换 (per DD-SHARED-TASK-001 §4.5 字段映射表)
///
/// 注: Quest 缺 issue_key 跨域 (per DD §7 缺口 #7), 缺 assignee (走 GameLoop.agent.name),
/// state 4 态 → 共享 5 态映射 (per DD §4.5)
impl From<Quest> for shared_task::SharedTask {
    fn from(quest: Quest) -> Self {
        use shared_task::TaskState as STaskState;
        let state = match quest.status {
            QuestStatus::Available => STaskState::Open,
            QuestStatus::InProgress => STaskState::InProgress,
            QuestStatus::Completed => STaskState::Done,
            QuestStatus::Failed => STaskState::Closed, // Failed → Closed (业务关闭)
        };
        Self {
            // Quest 自己的 TaskId 是 local newtype (per R5 阶段 1 §1), 提取 Uuid 转 shared_task::TaskId
            id: shared_task::TaskId(quest.id.0),
            title: quest.title,
            description: quest.description,
            state,
            assignee: None, // Quest 不存 assignee (走 GameLoop.agent.name)
            priority: shared_task::Priority::default(),
            issue_key: None, // Quest 缺 issue_key 跨域 (per DD §7 缺口 #7)
            created_at: quest.created_at,
        }
    }
}

// ============================================================================
// §5 GameLoop 1 闭环 (per ADR-0027 §2.2.1 + 守门 v25 AC)
// ============================================================================

/// GameLoop 错误
#[derive(Debug, Error)]
pub enum GameError {
    /// mana 不足
    #[error("insufficient mana: required {required}, available {available}")]
    InsufficientMana {
        /// 接 quest 需要的 token 数
        required: u32,
        /// agent 当前剩余 token 数
        available: u32,
    },
    /// agent 死亡
    #[error("agent is dead, restart required")]
    AgentDead,
    /// agent 冷却中
    #[error("agent in cooldown until {until:?}")]
    InCooldown {
        /// 冷却结束的瞬时时间
        until: Instant,
    },
    /// quest 状态非法
    #[error("invalid quest status: {current:?}")]
    InvalidQuestStatus {
        /// 当前 quest 状态 (e.g. 重复接任务)
        current: QuestStatus,
    },
    /// backend 错误 (R5 阶段 2 新增, 透传 BackendError)
    #[error("backend error: {0}")]
    BackendError(#[from] BackendError),
}

/// GameLoop 1 闭环 (PoC 验证 5 件事)
///
/// 1. agent 接 task (mana 消耗)
/// 2. agent 完成 task (xp 增加)
/// 3. agent 升级 (skill tree 解锁)
/// 4. agent 失败 (health 减少, cooldown)
/// 5. agent 死亡 (mana 耗尽 / health=0, 需重启 session)
pub struct GameLoop {
    /// 当前 agent
    pub agent: Agent,
    /// 当前 quest
    pub current_quest: Option<Quest>,
    /// 游戏后端 (per ADR-0027 §2.2.3 Physis / GVPE 集成点, 默认 MockBackend)
    pub backend: Box<dyn GameBackend>,
}

impl GameLoop {
    /// 新建 gameloop (默认 MockBackend, no-op, 阶段 1 行为)
    pub fn new(agent: Agent) -> Self {
        Self {
            agent,
            current_quest: None,
            backend: Box::new(MockBackend),
        }
    }

    /// 新建 gameloop with 自定义 backend (R5 阶段 2 新增, 阶段 2+ 用 PhysisMockBackend / 后续 PhysicsBackend)
    pub fn new_with_backend(agent: Agent, backend: Box<dyn GameBackend>) -> Self {
        Self {
            agent,
            current_quest: None,
            backend,
        }
    }

    /// 接 task (mana 消耗, per ADR-0027 §2.2.1 1️⃣)
    pub fn assign_quest(&mut self, mut quest: Quest) -> Result<(), GameError> {
        if self.agent.is_dead() {
            return Err(GameError::AgentDead);
        }
        if self.agent.in_cooldown() {
            return Err(GameError::InCooldown {
                until: self.agent.cooldown.until.unwrap(),
            });
        }
        if quest.status != QuestStatus::Available {
            return Err(GameError::InvalidQuestStatus {
                current: quest.status,
            });
        }
        // 消耗 mana
        if !self.agent.mana.consume(quest.mana_cost) {
            return Err(GameError::InsufficientMana {
                required: quest.mana_cost,
                available: self.agent.mana.tokens,
            });
        }
        quest.status = QuestStatus::InProgress;
        self.current_quest = Some(quest);
        Ok(())
    }

    /// 完成 task 成功 (xp 增加, per ADR-0027 §2.2.1 2️⃣ + 3️⃣)
    pub fn complete_quest_success(&mut self) -> Result<Xp, GameError> {
        let mut quest = self
            .current_quest
            .take()
            .ok_or(GameError::InvalidQuestStatus {
                current: QuestStatus::Available,
            })?;
        if quest.status != QuestStatus::InProgress {
            return Err(GameError::InvalidQuestStatus {
                current: quest.status,
            });
        }
        quest.status = QuestStatus::Completed;
        // 获得 XP, 自动升级
        let leveled_up = self.agent.xp.gain(quest.xp_reward);
        let xp_snapshot = self.agent.xp;
        if leveled_up {
            // 升级解锁 skill tree tier
            self.agent.skill_tree.upgrade_tier();
        }
        Ok(xp_snapshot)
    }

    /// 失败 task (health 减少, cooldown, per ADR-0027 §2.2.1 4️⃣ + 6️⃣)
    ///
    /// R5 阶段 2: 失败时调 `backend.apply_physics(agent)` 让 backend 推 Collision event 触发额外 damage
    /// (MockBackend no-op, PhysisMockBackend 推 queue 累加 damage)
    pub fn fail_quest(&mut self, damage: u32, cooldown: Duration) -> Result<bool, GameError> {
        let mut quest = self
            .current_quest
            .take()
            .ok_or(GameError::InvalidQuestStatus {
                current: QuestStatus::Available,
            })?;
        if quest.status != QuestStatus::InProgress {
            return Err(GameError::InvalidQuestStatus {
                current: quest.status,
            });
        }
        quest.status = QuestStatus::Failed;
        // backend 先 apply (Collision event → 额外 damage), per R5 阶段 2 Physis 集成
        self.backend.apply_physics(&mut self.agent)?;
        // 受到显式伤害
        let died = self.agent.health.take_damage(damage);
        // 设置冷却
        self.agent.cooldown.apply(cooldown);
        Ok(died)
    }

    /// 重启 session (death 后)
    pub fn restart(&mut self) {
        self.agent.health = Health::new();
        self.agent.cooldown.clear();
        self.current_quest = None;
    }
}

// ============================================================================
// §6 GameBackend trait (Physis / GVPE 集成点, per ADR-0027 §2.2.3)
// ============================================================================

/// GameBackend 错误
#[derive(Debug, Error)]
pub enum BackendError {
    /// 物理查询失败
    #[error("physics query failed: {0}")]
    PhysicsQuery(String),
    /// 物理应用失败
    #[error("physics apply failed: {0}")]
    PhysicsApply(String),
}

/// GameBackend trait (per ADR-0027 §2.2.3 Physis / GVPE 集成点)
///
/// 阶段 1: MockBackend (no-op, 验证 trait 抽象)
/// 阶段 2: PhysisMockBackend (模拟 Physis 0.1.x 接口形状, per plan-032 line 214)
/// 阶段 3: GVPEBackend (GVPE 游戏运行时集成)
pub trait GameBackend {
    /// 应用物理 (移动 / 碰撞 / 受击, backend 可直接修改 agent 属性 e.g. Collision → take_damage)
    fn apply_physics(&mut self, agent: &mut Agent) -> Result<(), BackendError>;
    /// 查询碰撞 (返回当前 agent 周边碰撞 entity UUID 列表)
    fn query_collision(&self, agent: &Agent) -> Result<Vec<Uuid>, BackendError>;
    /// 拉取 backend 推送给 GameLoop 的事件 (R5 阶段 2 新增, 默认 no-op)
    ///
    /// 用于: GameLoop 想 polling-style 处理事件流 (e.g. 移动轨迹, 累计伤害) 而不直接修改 agent.
    /// `apply_physics` 跟 `poll_events` 是两种 integration 模式, 互不冲突.
    fn poll_events(&mut self) -> Vec<BackendEvent> {
        Vec::new()
    }
}

/// MockBackend (no-op, 阶段 1 用)
pub struct MockBackend;

impl GameBackend for MockBackend {
    fn apply_physics(&mut self, _agent: &mut Agent) -> Result<(), BackendError> {
        Ok(())
    }
    fn query_collision(&self, _agent: &Agent) -> Result<Vec<Uuid>, BackendError> {
        Ok(vec![])
    }
}

// ============================================================================
// §6.5 PhysicsEvent + PhysisMockBackend (R5 阶段 2, per plan-032 line 214)
// ============================================================================

/// Physis 物理事件 (R5 阶段 2 mock 模拟 Physis 0.1.x 接口形状)
///
/// 真实 Physis 0.1.x 接入后, 这 enum 会被 `physis::Event` 替代, 转换走 `From` impl.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhysicsEvent {
    /// 移动 (dx, dy 整数单位, 简单欧氏空间)
    Move {
        /// x 方向位移
        dx: i32,
        /// y 方向位移
        dy: i32,
    },
    /// 碰撞 (with: Uuid, damage: u32)
    Collision {
        /// 碰撞对方 entity UUID
        with: Uuid,
        /// 碰撞伤害
        damage: u32,
    },
}

/// GameBackend 推送给 GameLoop 的事件 (R5 阶段 2 新增)
///
/// GameLoop 可通过 `backend.poll_events()` polling 消费, 不直接修改 agent 属性.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackendEvent {
    /// 移动 (跟 PhysicsEvent::Move 1:1 转发)
    Move {
        /// x 方向位移
        dx: i32,
        /// y 方向位移
        dy: i32,
    },
    /// 受到伤害 (从 PhysicsEvent::Collision 派生)
    Damage {
        /// 伤害值
        amount: u32,
    },
}

/// PhysisMockBackend (R5 阶段 2 模拟 Physis 0.1.x, per plan-032 line 214)
///
/// 内部维护 `event_queue`, `apply_physics` 消费 queue 触发 Collision → agent.take_damage,
/// `query_collision` 返回 queue 里的 collision UUIDs, `poll_events` 把 PhysicsEvent 转 BackendEvent.
pub struct PhysisMockBackend {
    event_queue: VecDeque<PhysicsEvent>,
}

impl Default for PhysisMockBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl PhysisMockBackend {
    /// 新建空 backend
    pub fn new() -> Self {
        Self {
            event_queue: VecDeque::new(),
        }
    }
    /// 推入物理事件 (测试 / 上游 source 用, e.g. Physis 主循环推送)
    pub fn push_event(&mut self, event: PhysicsEvent) {
        self.event_queue.push_back(event);
    }
    /// 队列大小 (测试用)
    pub fn queue_len(&self) -> usize {
        self.event_queue.len()
    }
}

impl GameBackend for PhysisMockBackend {
    fn apply_physics(&mut self, agent: &mut Agent) -> Result<(), BackendError> {
        // 消费 queue, Collision → agent.take_damage; Move → 不直接改 6 维
        while let Some(event) = self.event_queue.pop_front() {
            match event {
                PhysicsEvent::Move { .. } => {
                    // 移动不直接改 6 维, GameLoop 走 poll_events 拉
                }
                PhysicsEvent::Collision { damage, .. } => {
                    agent.health.take_damage(damage);
                }
            }
        }
        Ok(())
    }

    fn query_collision(&self, _agent: &Agent) -> Result<Vec<Uuid>, BackendError> {
        // 返回 queue 里未消费的 collision UUIDs
        Ok(self
            .event_queue
            .iter()
            .filter_map(|e| match e {
                PhysicsEvent::Collision { with, .. } => Some(*with),
                PhysicsEvent::Move { .. } => None,
            })
            .collect())
    }

    fn poll_events(&mut self) -> Vec<BackendEvent> {
        // 把 queue 转 BackendEvent 返回 (drain 模式, 不直接改 agent)
        self.event_queue
            .drain(..)
            .map(|e| match e {
                PhysicsEvent::Move { dx, dy } => BackendEvent::Move { dx, dy },
                PhysicsEvent::Collision { damage, .. } => BackendEvent::Damage { amount: damage },
            })
            .collect()
    }
}

// ============================================================================
// §7 Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_damage_no_death() {
        let mut h = Health::new();
        assert!(!h.take_damage(30));
        assert_eq!(h.value, 70);
    }

    #[test]
    fn health_damage_to_zero_death() {
        let mut h = Health::new();
        assert!(h.take_damage(100));
        assert_eq!(h.value, 0);
    }

    #[test]
    fn health_heal() {
        let mut h = Health::new();
        h.take_damage(50);
        h.heal(20);
        assert_eq!(h.value, 70);
    }

    #[test]
    fn mana_consume_and_replenish() {
        let mut m = Mana::new();
        assert!(m.consume(100));
        assert_eq!(m.tokens, 900);
        assert!(!m.consume(1000)); // 不足
        m.replenish(200);
        assert_eq!(m.tokens, 1000); // 满
    }

    #[test]
    fn xp_gain_level_up() {
        let mut xp = Xp::new();
        assert!(!xp.gain(50)); // 50/100, 不升级
        assert_eq!(xp.level, 1);
        assert!(xp.gain(60)); // 110/100, 升级
        assert_eq!(xp.level, 2);
        assert_eq!(xp.xp, 10); // 110 - 100 = 10
        assert_eq!(xp.xp_to_next_level, 200); // 2 * 100
    }

    #[test]
    fn skill_tree_change_branch_and_upgrade() {
        let mut st = SkillTree::new();
        assert_eq!(st.branch, SkillBranch::Talent);
        st.change_branch(SkillBranch::Spell);
        assert_eq!(st.branch, SkillBranch::Spell);
        st.upgrade_tier();
        assert_eq!(st.tier, 2);
    }

    #[test]
    fn inventory_add_remove_item() {
        let mut inv = Inventory::default();
        let item = Item {
            id: ItemId(Uuid::new_v4()),
            kind: ItemKind::Commit,
            metadata: serde_json::json!({"hash": "abc1234"}),
        };
        inv.add(item.clone());
        assert_eq!(inv.len(), 1);
        let removed = inv.remove(item.id);
        assert!(removed.is_some());
        assert_eq!(inv.len(), 0);
    }

    #[test]
    fn cooldown_apply_and_clear() {
        let mut cd = Cooldown::new();
        assert!(!cd.is_active());
        cd.apply(Duration::from_millis(100));
        assert!(cd.is_active());
        cd.clear();
        assert!(!cd.is_active());
    }

    #[test]
    fn agent_new_6_attributes() {
        let agent = Agent::new("Mavis");
        assert_eq!(agent.name, "Mavis");
        assert_eq!(agent.health.value, 100);
        assert_eq!(agent.mana.tokens, 1000);
        assert_eq!(agent.xp.level, 1);
        assert_eq!(agent.skill_tree.branch, SkillBranch::Talent);
        assert_eq!(agent.skill_tree.tier, 1);
        assert!(agent.inventory.is_empty());
        assert!(!agent.in_cooldown());
    }

    #[test]
    fn quest_new_initial_state() {
        let q = Quest::new("R5 PoC", "verify game loop", 50, 200);
        assert_eq!(q.status, QuestStatus::Available);
        assert_eq!(q.mana_cost, 50);
        assert_eq!(q.xp_reward, 200);
    }

    #[test]
    fn game_loop_assign_quest_consume_mana() {
        let mut game = GameLoop::new(Agent::new("Mavis"));
        let q = Quest::new("R5 PoC", "verify game loop", 50, 200);
        let result = game.assign_quest(q);
        assert!(result.is_ok());
        assert_eq!(game.agent.mana.tokens, 950); // 1000 - 50
        assert_eq!(
            game.current_quest.as_ref().unwrap().status,
            QuestStatus::InProgress
        );
    }

    #[test]
    fn game_loop_insufficient_mana() {
        let mut agent = Agent::new("Mavis");
        agent.mana.tokens = 30; // 不够
        let mut game = GameLoop::new(agent);
        let q = Quest::new("expensive", "too much", 50, 200);
        let result = game.assign_quest(q);
        assert!(matches!(result, Err(GameError::InsufficientMana { .. })));
    }

    #[test]
    fn game_loop_complete_quest_gain_xp_and_level_up() {
        let mut game = GameLoop::new(Agent::new("Mavis"));
        let q = Quest::new("R5 PoC", "verify", 50, 200);
        game.assign_quest(q).unwrap();
        let xp = game.complete_quest_success().unwrap();
        // 200 XP 触发 1 次升级 (level 1→2, xp 剩 100), 因公式 xp_to_next_level = level * 100
        assert_eq!(xp.level, 2);
        assert_eq!(xp.xp, 100); // 200 - 100 = 100 剩
        assert_eq!(game.agent.skill_tree.tier, 2); // 升级解锁, tier 1→2
    }

    #[test]
    fn game_loop_fail_quest_damage_and_cooldown() {
        let mut game = GameLoop::new(Agent::new("Mavis"));
        let q = Quest::new("fail-test", "test fail", 50, 200);
        game.assign_quest(q).unwrap();
        let died = game.fail_quest(30, Duration::from_millis(100)).unwrap();
        assert!(!died);
        assert_eq!(game.agent.health.value, 70);
        assert!(game.agent.in_cooldown());
    }

    #[test]
    fn game_loop_fail_to_death() {
        let mut game = GameLoop::new(Agent::new("Mavis"));
        let q = Quest::new("suicide", "die", 0, 0);
        game.assign_quest(q).unwrap();
        let died = game.fail_quest(100, Duration::from_millis(100)).unwrap();
        assert!(died);
        assert!(game.agent.is_dead());
    }

    #[test]
    fn game_loop_restart_after_death() {
        let mut game = GameLoop::new(Agent::new("Mavis"));
        let q = Quest::new("die", "die", 0, 0);
        game.assign_quest(q).unwrap();
        game.fail_quest(100, Duration::from_millis(0)).unwrap();
        assert!(game.agent.is_dead());
        game.restart();
        assert_eq!(game.agent.health.value, 100);
        assert!(!game.agent.in_cooldown());
    }

    #[test]
    fn game_loop_assign_after_cooldown() {
        let mut game = GameLoop::new(Agent::new("Mavis"));
        let q = Quest::new("first", "fail", 0, 0);
        game.assign_quest(q).unwrap();
        game.fail_quest(10, Duration::from_millis(100)).unwrap();
        // 在 cooldown 中接新 quest → 失败
        let q2 = Quest::new("second", "try", 0, 0);
        let result = game.assign_quest(q2);
        assert!(matches!(result, Err(GameError::InCooldown { .. })));
    }

    #[test]
    fn mock_backend_no_op() {
        let mut backend = MockBackend;
        let mut agent = Agent::new("Mavis");
        assert!(backend.apply_physics(&mut agent).is_ok());
        assert!(backend.query_collision(&agent).is_ok());
        assert_eq!(backend.query_collision(&agent).unwrap().len(), 0);
    }

    // ========================================================================
    // R5 阶段 2: PhysisMockBackend + GameLoop 集成 UT (per plan-032 R5 阶段 2)
    // ========================================================================

    #[test]
    fn physis_mock_push_collision_event_apply_damage() {
        let mut backend = PhysisMockBackend::new();
        backend.push_event(PhysicsEvent::Collision {
            with: Uuid::new_v4(),
            damage: 30,
        });
        let mut agent = Agent::new("Mavis");
        backend.apply_physics(&mut agent).unwrap();
        assert_eq!(agent.health.value, 70); // 100 - 30
        assert_eq!(backend.queue_len(), 0); // queue drained
    }

    #[test]
    fn physis_mock_apply_physics_drains_queue() {
        let mut backend = PhysisMockBackend::new();
        backend.push_event(PhysicsEvent::Move { dx: 1, dy: 0 });
        backend.push_event(PhysicsEvent::Move { dx: 0, dy: 1 });
        backend.push_event(PhysicsEvent::Collision {
            with: Uuid::new_v4(),
            damage: 10,
        });
        assert_eq!(backend.queue_len(), 3);
        let mut agent = Agent::new("Mavis");
        backend.apply_physics(&mut agent).unwrap();
        assert_eq!(backend.queue_len(), 0);
        assert_eq!(agent.health.value, 90); // 100 - 10
    }

    #[test]
    fn physis_mock_move_event_no_damage() {
        let mut backend = PhysisMockBackend::new();
        backend.push_event(PhysicsEvent::Move { dx: 5, dy: 5 });
        let mut agent = Agent::new("Mavis");
        backend.apply_physics(&mut agent).unwrap();
        assert_eq!(agent.health.value, 100); // move 不掉血
    }

    #[test]
    fn physis_mock_query_collision_returns_queued_uuids() {
        let mut backend = PhysisMockBackend::new();
        let npc1 = Uuid::new_v4();
        let npc2 = Uuid::new_v4();
        backend.push_event(PhysicsEvent::Collision {
            with: npc1,
            damage: 5,
        });
        backend.push_event(PhysicsEvent::Move { dx: 1, dy: 0 });
        backend.push_event(PhysicsEvent::Collision {
            with: npc2,
            damage: 5,
        });
        let agent = Agent::new("Mavis");
        let collisions = backend.query_collision(&agent).unwrap();
        assert_eq!(collisions.len(), 2);
        assert!(collisions.contains(&npc1));
        assert!(collisions.contains(&npc2));
    }

    #[test]
    fn physis_mock_poll_events_drains_and_translates() {
        let mut backend = PhysisMockBackend::new();
        backend.push_event(PhysicsEvent::Move { dx: 3, dy: 4 });
        backend.push_event(PhysicsEvent::Collision {
            with: Uuid::new_v4(),
            damage: 15,
        });
        let events = backend.poll_events();
        assert_eq!(events.len(), 2);
        assert!(matches!(events[0], BackendEvent::Move { dx: 3, dy: 4 }));
        assert!(matches!(events[1], BackendEvent::Damage { amount: 15 }));
        assert_eq!(backend.queue_len(), 0); // drain 后 queue 空
    }

    #[test]
    fn physis_mock_multiple_collision_accumulates_damage() {
        let mut backend = PhysisMockBackend::new();
        backend.push_event(PhysicsEvent::Collision {
            with: Uuid::new_v4(),
            damage: 20,
        });
        backend.push_event(PhysicsEvent::Collision {
            with: Uuid::new_v4(),
            damage: 30,
        });
        let mut agent = Agent::new("Mavis");
        backend.apply_physics(&mut agent).unwrap();
        assert_eq!(agent.health.value, 50); // 100 - 20 - 30
    }

    #[test]
    fn game_loop_with_physis_backend_fail_quest_extra_damage() {
        // GameLoop 用 PhysisMockBackend, 推 Collision event 触发额外 damage
        let mut backend = PhysisMockBackend::new();
        backend.push_event(PhysicsEvent::Collision {
            with: Uuid::new_v4(),
            damage: 25,
        });
        let mut game = GameLoop::new_with_backend(Agent::new("Mavis"), Box::new(backend));
        let q = Quest::new("boss-fight", "hard", 0, 0);
        game.assign_quest(q).unwrap();
        let died = game.fail_quest(10, Duration::from_millis(0)).unwrap();
        // backend apply 25 (Collision) + explicit 10 = 35 total damage
        assert_eq!(game.agent.health.value, 65);
        assert!(!died); // 65 > 0
    }

    #[test]
    fn game_loop_with_mock_backend_fail_quest_no_extra_damage() {
        // GameLoop 用 MockBackend (默认, no-op), fail_quest 只算显式 damage
        let mut game = GameLoop::new(Agent::new("Mavis"));
        let q = Quest::new("easy-fight", "no-op", 0, 0);
        game.assign_quest(q).unwrap();
        let died = game.fail_quest(10, Duration::from_millis(0)).unwrap();
        assert_eq!(game.agent.health.value, 90); // 100 - 10
        assert!(!died);
    }

    #[test]
    fn game_loop_poll_events_empty_for_mock() {
        // MockBackend 默认 poll_events 返空 (R5 阶段 2 trait default impl)
        let mut backend = MockBackend;
        let events = backend.poll_events();
        assert!(events.is_empty());
    }

    // ========================================================================
    // R9 阶段 2: Quest → SharedTask 转换 UT (per DD-SHARED-TASK-001 §4.5)
    // ========================================================================

    #[test]
    fn quest_to_shared_task_conversion_4_to_5_state_mapping() {
        let q = Quest::new("R5 PoC", "verify game loop", 50, 200);
        let original_id = q.id.0;
        let shared: shared_task::SharedTask = q.into();
        // id: Quest 自己的 TaskId (local newtype) → shared_task::TaskId
        assert_eq!(shared.id.0, original_id);
        assert_eq!(shared.title, "R5 PoC");
        assert_eq!(shared.description, "verify game loop");
        // Available → Open
        assert_eq!(shared.state, shared_task::TaskState::Open);
        // Quest 缺 assignee + issue_key
        assert_eq!(shared.assignee, None);
        assert_eq!(shared.issue_key, None);
        assert_eq!(shared.priority, shared_task::Priority::Medium);
    }

    #[test]
    fn quest_status_failed_maps_to_closed() {
        // Failed → Closed (per DD §4.5 业务关闭)
        let mut q = Quest::new("fail", "test", 0, 0);
        q.status = QuestStatus::Failed;
        let shared: shared_task::SharedTask = q.into();
        assert_eq!(shared.state, shared_task::TaskState::Closed);
    }

    // ========================================================================
    // R9 阶段 3: 5 milestone benchmark 实测 (per plan-032 §4.1)
    // ========================================================================

    /// milestone #5: Agent ECS 10K entity 60fps < 16.7ms (vs Physis 30fps 优化前 2x 加速)
    /// 测量 10K Agent 1 frame update 时间
    #[test]
    #[ignore = "R9 阶段 3 PoC: criterion bench 留 benches/, 默认跳过避免 cargo test 慢"]
    fn r9_milestone_5_ecs_10k_agents_one_frame_under_16_7ms() {
        use std::time::Instant;
        let mut agents: Vec<Agent> = (0..10_000)
            .map(|i| Agent::new(format!("agent-{i}")))
            .collect();
        let start = Instant::now();
        for agent in agents.iter_mut() {
            agent.mana.replenish(1);
        }
        let elapsed_ms_f = start.elapsed().as_secs_f64() * 1000.0;
        eprintln!("[R9 milestone #5] 10K agents 1 frame: {elapsed_ms_f:.2} ms (target: < 16.7 ms)");
    }
}
