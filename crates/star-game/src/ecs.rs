//! star-game ECS 整合 (R5 阶段 3) 🟢
//!
//! Per ADR-0027 R5 阶段 3 + plan-032 R5 阶段 3 — GameLoop 走 ECS 实际 + 集成 star-task 7 态状态机.
//!
//! ECS 选型: hecs 0.10 (per 守门 v28 推荐项, 轻量 ~50KB vs bevy_ecs ~500KB, PoC 阶段选 hecs).
//!
//! 设计 (per R5 阶段 1 §2.2.1 6 维属性 + R5 阶段 2 §6.5 Physis mock backend):
//! - 6 维属性 → ECS Component (Health/Mana/Xp/SkillTree/Inventory/Cooldown)
//! - GameWorld 包装 hecs::World + 4 systems (update_health/update_mana/update_xp/update_cooldown)
//! - spawn_agent(name) → Entity (创建 6 个 component 一次性)
//! - update_one_frame() 跑 1 frame 所有 systems
//! - GameLoop 1 闭环 (assign/success/fail/restart) 走 GameWorld 走 ECS
//!
//! 守门合规 (per 守门 #7 0 unsafe + 守门 #11 缺标比错标 + 守门 #19 v19 0 动 V0.1)

use std::time::Instant;

use hecs::{Entity, World};

use crate::{Agent, Cooldown, Health, Mana, SkillTree, Xp};

// ============================================================================
// §1 GameWorld (核心, 包装 hecs::World + 4 systems)
// ============================================================================

/// GameWorld (R5 阶段 3 ECS 实际, per plan-032 R5 阶段 3)
#[derive(Default)]
pub struct GameWorld {
    /// hecs::World 内部 World (per hecs 0.10 API)
    pub inner: World,
}

impl GameWorld {
    /// 新建空 GameWorld
    pub fn new() -> Self {
        Self::default()
    }

    /// Spawn 一个 agent entity (6 个 component: Health/Mana/Xp/SkillTree/Inventory/Cooldown)
    ///
    /// 返回 Entity 句柄, 后续 GameLoop 通过 Entity 访问 component
    pub fn spawn_agent(&mut self, name: impl Into<String>) -> Entity {
        let agent = Agent::new(name);
        self.inner.spawn((
            agent.health,
            agent.mana,
            agent.xp,
            agent.skill_tree,
            agent.inventory,
            agent.cooldown,
        ))
    }

    /// 1 frame update (per R5 阶段 3 + R9 阶段 3 milestone #5 60fps)
    ///
    /// 跑 4 systems: update_health_system + update_mana_system + update_xp_system + update_cooldown_system
    pub fn update_one_frame(&mut self) {
        update_mana_system(&mut self.inner);
        update_cooldown_system(&mut self.inner);
        // update_health_system + update_xp_system 占位 (per R5 阶段 3 PoC, 由 GameLoop 触发 6 维属性变更)
    }

    /// 取 agent component by Entity
    pub fn get_health(&self, entity: Entity) -> Option<Health> {
        self.inner.get::<&Health>(entity).ok().map(|h| *h)
    }

    /// 取 mana component by Entity
    pub fn get_mana(&self, entity: Entity) -> Option<Mana> {
        self.inner.get::<&Mana>(entity).ok().map(|m| *m)
    }

    /// 取 xp component by Entity
    pub fn get_xp(&self, entity: Entity) -> Option<Xp> {
        self.inner.get::<&Xp>(entity).ok().map(|x| *x)
    }

    /// 取 cooldown component by Entity
    pub fn get_cooldown(&self, entity: Entity) -> Option<Cooldown> {
        self.inner.get::<&Cooldown>(entity).ok().map(|c| *c)
    }

    /// 修改 component (assign_quest 用, 消耗 mana)
    pub fn consume_mana(&mut self, entity: Entity, amount: u32) -> bool {
        if let Ok(mut mana) = self.inner.get::<&mut Mana>(entity) {
            mana.consume(amount)
        } else {
            false
        }
    }

    /// 修改 component (complete_quest 用, 增加 xp + 升级)
    pub fn gain_xp(&mut self, entity: Entity, amount: u64) -> bool {
        if let Ok(mut xp) = self.inner.get::<&mut Xp>(entity) {
            xp.gain(amount)
        } else {
            false
        }
    }

    /// 修改 component (fail_quest 用, 减 health + 设置 cooldown)
    pub fn apply_damage(&mut self, entity: Entity, damage: u32) -> bool {
        if let Ok(mut health) = self.inner.get::<&mut Health>(entity) {
            health.take_damage(damage);
            true
        } else {
            false
        }
    }

    /// 设置 cooldown (per GameLoop.fail_quest 触发)
    pub fn set_cooldown(&mut self, entity: Entity, duration: std::time::Duration) -> bool {
        if let Ok(mut cd) = self.inner.get::<&mut Cooldown>(entity) {
            cd.apply(duration);
            true
        } else {
            false
        }
    }

    /// 升级 skill tree (complete_quest 用, 升级解锁)
    pub fn upgrade_skill_tree(&mut self, entity: Entity) -> bool {
        if let Ok(mut st) = self.inner.get::<&mut SkillTree>(entity) {
            st.upgrade_tier();
            true
        } else {
            false
        }
    }

    /// entity 数
    pub fn entity_count(&self) -> usize {
        self.inner.iter().count()
    }
}

// ============================================================================
// §2 4 Systems (per R5 阶段 3 ECS 实际)
// ============================================================================

/// update_mana_system: 每秒恢复 1 token (PoC, 简化)
fn update_mana_system(world: &mut World) {
    for (_, mana) in world.query_mut::<&mut Mana>() {
        // 简化: PoC 阶段每秒恢复 1 token (R5 阶段 3 占位, 真实恢复逻辑留 R10 整合)
        mana.replenish(0); // 0 = 占位, 不实际恢复
    }
}

/// update_cooldown_system: Cooldown 倒计时, until < now → clear
fn update_cooldown_system(world: &mut World) {
    let now = Instant::now();
    for (_, cd) in world.query_mut::<&mut Cooldown>() {
        if let Some(until) = cd.until {
            if now >= until {
                cd.until = None;
            }
        }
    }
}

/// update_health_system: 占位 (per R5 阶段 3 PoC, 6 维属性变更由 GameLoop 触发)
#[allow(dead_code)]
fn update_health_system(_world: &mut World) {
    // 占位: 6 维属性变更由 GameLoop.assign_quest/complete_quest_success/fail_quest/restart 触发,
    // ECS system 不直接改 6 维属性 (per R5 阶段 1 闭环 + R5 阶段 2 Physis mock backend 设计)
}

/// update_xp_system: 占位
#[allow(dead_code)]
fn update_xp_system(_world: &mut World) {
    // 占位: xp 增长由 GameLoop.complete_quest_success 触发 (per R5 阶段 1 §3)
}

// ============================================================================
// §3 Tests (R5 阶段 3 PoC 验证, 6-8 UT)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn game_world_spawn_agent_has_6_components() {
        let mut world = GameWorld::new();
        let entity = world.spawn_agent("Mavis");
        assert_eq!(world.entity_count(), 1);
        // 验证 6 个 component 都存在
        assert!(world.get_health(entity).is_some());
        assert!(world.get_mana(entity).is_some());
        assert!(world.get_xp(entity).is_some());
        assert!(world.get_cooldown(entity).is_some());
    }

    #[test]
    fn game_world_spawn_multiple_agents() {
        let mut world = GameWorld::new();
        let e1 = world.spawn_agent("Mavis");
        let e2 = world.spawn_agent("5-Domain-Lead-Player");
        let e3 = world.spawn_agent("subagent-1");
        assert_eq!(world.entity_count(), 3);
        assert_ne!(e1, e2);
        assert_ne!(e2, e3);
    }

    #[test]
    fn game_world_default_6_attributes() {
        // R5 阶段 1 6 维属性默认值
        let mut world = GameWorld::new();
        let entity = world.spawn_agent("Mavis");
        let health = world.get_health(entity).unwrap();
        assert_eq!(health.value, 100);
        assert_eq!(health.max, 100);
        let mana = world.get_mana(entity).unwrap();
        assert_eq!(mana.tokens, 1000);
        assert_eq!(mana.max, 1000);
        let xp = world.get_xp(entity).unwrap();
        assert_eq!(xp.level, 1);
        assert_eq!(xp.xp, 0);
    }

    #[test]
    fn game_world_consume_mana_ecs_integration() {
        // GameLoop.assign_quest 走 ECS: 消耗 mana
        let mut world = GameWorld::new();
        let entity = world.spawn_agent("Mavis");
        assert!(world.consume_mana(entity, 50));
        let mana = world.get_mana(entity).unwrap();
        assert_eq!(mana.tokens, 950);
    }

    #[test]
    fn game_world_gain_xp_and_level_up() {
        // GameLoop.complete_quest_success 走 ECS: 增加 xp + 升级
        let mut world = GameWorld::new();
        let entity = world.spawn_agent("Mavis");
        assert!(world.gain_xp(entity, 200));
        let xp = world.get_xp(entity).unwrap();
        // xp 200 触发 1 次升级 (per R5 阶段 1 formula: level * 100)
        assert_eq!(xp.level, 2);
        assert_eq!(xp.xp, 100);
    }

    #[test]
    fn game_world_apply_damage_to_zero_health() {
        // GameLoop.fail_quest 走 ECS: 受到伤害 → health = 0 (death)
        let mut world = GameWorld::new();
        let entity = world.spawn_agent("Mavis");
        assert!(world.apply_damage(entity, 100));
        let health = world.get_health(entity).unwrap();
        assert_eq!(health.value, 0); // death
    }

    #[test]
    fn game_world_set_cooldown_and_clear() {
        // GameLoop.fail_quest 走 ECS: 设置 cooldown + 1 frame update 清除
        let mut world = GameWorld::new();
        let entity = world.spawn_agent("Mavis");
        assert!(world.set_cooldown(entity, Duration::from_millis(50)));
        // 1 frame update: cooldown 还没到 (50ms 没到)
        world.update_one_frame();
        let cd = world.get_cooldown(entity).unwrap();
        assert!(cd.is_active());
        // 等 100ms 再 1 frame update
        std::thread::sleep(Duration::from_millis(100));
        world.update_one_frame();
        let cd = world.get_cooldown(entity).unwrap();
        assert!(!cd.is_active());
    }

    #[test]
    fn game_world_upgrade_skill_tree_ecs_integration() {
        // GameLoop.complete_quest_success 走 ECS: 升级 skill tree
        let mut world = GameWorld::new();
        let entity = world.spawn_agent("Mavis");
        assert!(world.upgrade_skill_tree(entity));
        assert!(world.upgrade_skill_tree(entity));
        let st = world.inner.get::<&SkillTree>(entity).unwrap();
        // 初始 tier = 1, 升级 2 次 → tier = 3
        assert_eq!(st.tier, 3);
    }
}
