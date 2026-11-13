/*
@cypher schema=1 source_sha256=9c3ef408044e9488b76dc8094f6d3747387a7f4b24c7224843b8b2dc527a4c88
MERGE (self:File {path:"crates/star-agent-runtime/src/ecs.rs"})
@endcypher
*/
//! 自研 Minimal ECS (per 03-DD §1.3 选型表: 自研 = 备选, 极简)。
//!
//! Agent-oriented ECS (per SRS §69): **不是**游戏 ECS, 无 Frame/Scene/Transform/
//! Render。围绕 State / Lifecycle / Event / Scheduler / Resource / Policy 设计。
//!
//! 选型状态 (per 03-DD §1.3 + 已知缺口 G-2): bevy_ecs / flecs 待 P3-B 拍板;
//! 本骨架以 per-component HashMap 存储 (O(1) 增删查) 占位, 选型落地后替换本模块
//! 存储层, 上层 World API 保持不变。9 SA Type = 9 Archetype (per 02-BD §3.2),
//! ArchetypeId 按组件位掩码派生。

use std::collections::HashMap;

use crate::agent_state::AgentState;
use crate::components::{
    ContextRef, MailboxRef, McpPolicyRef, MemoryRef, ModelRef, PermissionRef, Priority,
    TokenBudget, ToolPolicyRef, WorkflowRef,
};
use crate::id::AgentId;
use crate::identity::AgentIdentity;
use crate::lifecycle::LifecycleState;

/// Component 标记 trait (per 03-DD §2.1 derive(Component))。
pub trait Component: 'static {}

/// 组件存在性位掩码 (Archetype 派生)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArchetypeId(pub u64);

// 位定义 (跟 World 存储一一对应)
const BIT_IDENTITY: u64 = 1;
const BIT_STATE: u64 = 1 << 1;
const BIT_LIFECYCLE: u64 = 1 << 2;
const BIT_PRIORITY: u64 = 1 << 3;
const BIT_TOKEN_BUDGET: u64 = 1 << 4;
const BIT_CONTEXT: u64 = 1 << 5;
const BIT_MEMORY: u64 = 1 << 6;
const BIT_MODEL: u64 = 1 << 7;
const BIT_TOOL_POLICY: u64 = 1 << 8;
const BIT_MCP_POLICY: u64 = 1 << 9;
const BIT_PERMISSION: u64 = 1 << 10;
const BIT_MAILBOX: u64 = 1 << 11;
const BIT_WORKFLOW: u64 = 1 << 12;

/// ECS World (自研 Minimal, per 03-DD §1.3)。
///
/// Agent = Entity + Components (per SRS §8: Identity + State + Refs + Policies)。
/// 单线程骨架 (Tokio 并发接线 per SRS §45 待 P3-B)。
#[derive(Debug, Default)]
pub struct World {
    archetypes: HashMap<AgentId, u64>,
    identities: HashMap<AgentId, AgentIdentity>,
    states: HashMap<AgentId, AgentState>,
    lifecycles: HashMap<AgentId, LifecycleState>,
    priorities: HashMap<AgentId, Priority>,
    token_budgets: HashMap<AgentId, TokenBudget>,
    context_refs: HashMap<AgentId, ContextRef>,
    memory_refs: HashMap<AgentId, MemoryRef>,
    model_refs: HashMap<AgentId, ModelRef>,
    tool_policy_refs: HashMap<AgentId, ToolPolicyRef>,
    mcp_policy_refs: HashMap<AgentId, McpPolicyRef>,
    permission_refs: HashMap<AgentId, PermissionRef>,
    mailbox_refs: HashMap<AgentId, MailboxRef>,
    workflow_refs: HashMap<AgentId, WorkflowRef>,
}

impl World {
    /// 空 World。
    pub fn new() -> Self {
        Self::default()
    }

    /// spawn entity (per 03-DD §4.1 UC-01: spawn entity + set Components)。
    pub fn spawn_agent(&mut self, identity: AgentIdentity) -> AgentId {
        let id = identity.agent_id;
        self.identities.insert(id, identity);
        self.archetypes.insert(id, BIT_IDENTITY);
        id
    }

    /// despawn entity (资源真正回收, per SRS §81 REQ-BENCH-LIFECYCLE)。
    pub fn despawn(&mut self, agent_id: AgentId) -> bool {
        let existed = self.archetypes.remove(&agent_id).is_some();
        if !existed {
            return false;
        }
        self.identities.remove(&agent_id);
        self.states.remove(&agent_id);
        self.lifecycles.remove(&agent_id);
        self.priorities.remove(&agent_id);
        self.token_budgets.remove(&agent_id);
        self.context_refs.remove(&agent_id);
        self.memory_refs.remove(&agent_id);
        self.model_refs.remove(&agent_id);
        self.tool_policy_refs.remove(&agent_id);
        self.mcp_policy_refs.remove(&agent_id);
        self.permission_refs.remove(&agent_id);
        self.mailbox_refs.remove(&agent_id);
        self.workflow_refs.remove(&agent_id);
        true
    }

    /// 组件插入 (泛型入口, 位掩码由 trait 实现提供)。
    pub fn insert_component<T: Component + Stored>(&mut self, agent_id: AgentId, value: T) {
        value.store(self, agent_id);
    }

    /// Resident agent 数。
    pub fn resident_count(&self) -> usize {
        self.archetypes.len()
    }

    /// Archetype (per 02-BD §3.2: 9 SA × 组件子集 → ArchetypeId)。
    pub fn archetype(&self, agent_id: AgentId) -> Option<ArchetypeId> {
        self.archetypes.get(&agent_id).copied().map(ArchetypeId)
    }

    /// identity 读。
    pub fn get_identity(&self, agent_id: AgentId) -> Option<&AgentIdentity> {
        self.identities.get(&agent_id)
    }

    /// state 读。
    pub fn get_state(&self, agent_id: AgentId) -> Option<&AgentState> {
        self.states.get(&agent_id)
    }

    /// state 可变读。
    pub fn get_state_mut(&mut self, agent_id: AgentId) -> Option<&mut AgentState> {
        self.states.get_mut(&agent_id)
    }

    /// lifecycle 读。
    pub fn get_lifecycle(&self, agent_id: AgentId) -> Option<&LifecycleState> {
        self.lifecycles.get(&agent_id)
    }

    /// lifecycle 可变读。
    pub fn get_lifecycle_mut(&mut self, agent_id: AgentId) -> Option<&mut LifecycleState> {
        self.lifecycles.get_mut(&agent_id)
    }

    /// priority 读。
    pub fn get_priority(&self, agent_id: AgentId) -> Option<Priority> {
        self.priorities.get(&agent_id).copied()
    }

    /// 按 lifecycle 计数 (per SRS §74 hot/warm/cold_agent_count)。
    pub fn lifecycle_counts(&self) -> (usize, usize, usize) {
        let mut hot = 0;
        let mut warm = 0;
        let mut cold = 0;
        for state in self.lifecycles.values() {
            match state.current {
                crate::lifecycle::LifecycleStateEnum::Hot => hot += 1,
                crate::lifecycle::LifecycleStateEnum::Warm => warm += 1,
                crate::lifecycle::LifecycleStateEnum::Cold => cold += 1,
            }
        }
        (hot, warm, cold)
    }

    /// 全部 resident agent id (Query 雏形)。
    pub fn agent_ids(&self) -> Vec<AgentId> {
        self.archetypes.keys().copied().collect()
    }
}

/// World 组件存储适配 trait (insert_component 的分发实现)。
pub trait Stored: Sized {
    /// 把 self 存进 World (按位掩码登记 archetype)。
    fn store(self, world: &mut World, agent_id: AgentId);
}

macro_rules! store_impl {
    ($ty:ty, $field:ident, $bit:expr) => {
        impl Stored for $ty {
            fn store(self, world: &mut World, agent_id: AgentId) {
                world.$field.insert(agent_id, self);
                let mask = world.archetypes.entry(agent_id).or_insert(0);
                *mask |= $bit;
            }
        }
    };
}

store_impl!(AgentState, states, BIT_STATE);
store_impl!(LifecycleState, lifecycles, BIT_LIFECYCLE);
store_impl!(Priority, priorities, BIT_PRIORITY);
store_impl!(TokenBudget, token_budgets, BIT_TOKEN_BUDGET);
store_impl!(ContextRef, context_refs, BIT_CONTEXT);
store_impl!(MemoryRef, memory_refs, BIT_MEMORY);
store_impl!(ModelRef, model_refs, BIT_MODEL);
store_impl!(ToolPolicyRef, tool_policy_refs, BIT_TOOL_POLICY);
store_impl!(McpPolicyRef, mcp_policy_refs, BIT_MCP_POLICY);
store_impl!(PermissionRef, permission_refs, BIT_PERMISSION);
store_impl!(MailboxRef, mailbox_refs, BIT_MAILBOX);
store_impl!(WorkflowRef, workflow_refs, BIT_WORKFLOW);
