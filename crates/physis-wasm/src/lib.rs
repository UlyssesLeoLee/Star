//! ═══════════════════════════════════════════════════════════════════
//! Ported from UlyssesLeoLee/Physis (MIT OR Apache-2.0) — GVPE 物理引擎内核
//! per 2026-11-14: 全 3D 主题 (Physis WASM + three.js 背景层)
//! Source: E:\Physis @ dev 3e2cf93 (gvpe-math/core/memory/dynamics 4 crate)
//! ═══════════════════════════════════════════════════════════════════

//! physis-wasm — Physis GVPE 物理引擎 WASM 门面 (Star 全 3D 主题背景层)
#![allow(missing_docs, unreachable_pub, clippy::too_many_arguments)]

pub mod gvpe_core;
pub mod gvpe_dynamics;
pub mod gvpe_math;
pub mod gvpe_memory;

use crate::gvpe_core::{BodyHandle, PhysicsLodTag, PhysicsProfile, SolverTypeId};
use crate::gvpe_dynamics::{DynamicsWorld, TimeStepper};
use crate::gvpe_math::{Quat, Vec3};
use wasm_bindgen::prelude::*;

/// 浏览器侧物理世界: 浮空粒子块 (重力 + 球体碰撞 + 鼠标吸引子)
#[wasm_bindgen]
pub struct PhysisWorld {
    world: DynamicsWorld,
    handles: Vec<BodyHandle>,
}

#[wasm_bindgen]
impl PhysisWorld {
    /// 创建世界, gravity=[gx,gy,gz]
    #[wasm_bindgen(constructor)]
    pub fn new(gravity: Vec<f32>) -> Self {
        let mut world = DynamicsWorld::new();
        world.set_gravity(Vec3::new(gravity[0], gravity[1], gravity[2]));
        world.set_stepper(TimeStepper::SemiImplicitEuler);
        Self {
            world,
            handles: Vec::new(),
        }
    }

    /// 生成动态球体 (radius 用于 profile)
    pub fn spawn(&mut self, x: f32, y: f32, z: f32, radius: f32, mass: f32) -> usize {
        let profile = PhysicsProfile {
            mass,
            density: 1.0,
            inertia: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
            friction: 0.4,
            restitution: 0.6,
            damping_linear: 0.995,
            damping_angular: 0.98,
            stiffness: 0.0,
            compliance: 0.0,
            viscosity: 0.0,
            solver_type: SolverTypeId::SequentialImpulse,
            solver_iterations: 4,
            collision_profile: 0,
            approximation_level: PhysicsLodTag::Lod0Full,
            _padding: [0],
        };
        let inertia_diag = [radius * radius * 0.4 * mass; 3];
        match self.world.spawn_dynamic(
            Vec3::new(x, y, z),
            Quat::IDENTITY,
            mass,
            inertia_diag,
            profile,
        ) {
            Ok(h) => {
                self.handles.push(h);
                self.handles.len() - 1
            }
            Err(_) => usize::MAX,
        }
    }

    /// 步进 dt 秒
    pub fn step(&mut self, dt: f32) {
        let _ = self.world.step(dt);
    }

    /// 读取第 idx 个刚体位置 [x,y,z] (不存在返回 [0,0,0])
    pub fn position(&self, idx: usize) -> Vec<f32> {
        let Some(h) = self.handles.get(idx) else {
            return vec![0.0, 0.0, 0.0];
        };
        match self.world.body(*h) {
            Ok(b) => {
                let p = b.position;
                vec![p.x, p.y, p.z]
            }
            Err(_) => vec![0.0, 0.0, 0.0],
        }
    }

    /// 施加外力 (鼠标吸引子): 向 target 施加力
    pub fn attract(&mut self, idx: usize, tx: f32, ty: f32, tz: f32, k: f32) {
        let Some(h) = self.handles.get(idx) else {
            return;
        };
        let Ok(b) = self.world.body(*h) else {
            return;
        };
        let p = b.position;
        let mut dir = Vec3::new(tx - p.x, ty - p.y, tz - p.z);
        let len = dir.length();
        if len < 0.001 {
            return;
        }
        dir = dir * (1.0 / len);
        let force = dir * k * len.min(8.0);
        if let Ok(body) = self.world.body_mut(*h) {
            body.add_force(force);
        }
    }

    /// 刚体数量
    pub fn len(&self) -> usize {
        self.handles.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.handles.is_empty()
    }
}

/// 内部 World 引用 (测试用)
impl PhysisWorld {
    pub fn inner(&mut self) -> &mut DynamicsWorld {
        &mut self.world
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_and_step() {
        let mut w = PhysisWorld::new(vec![0.0, -9.81, 0.0]);
        let idx = w.spawn(0.0, 10.0, 0.0, 1.0, 1.0);
        assert!(idx < usize::MAX);
        assert_eq!(w.len(), 1);
        for _ in 0..60 {
            w.step(1.0 / 60.0);
        }
        let p = w.position(idx);
        // 重力下落: y < 10
        assert!(p[1] < 10.0, "body should fall: {:?}", p);
    }

    #[test]
    fn attract_moves_body() {
        let mut w = PhysisWorld::new(vec![0.0, 0.0, 0.0]);
        let idx = w.spawn(0.0, 0.0, 0.0, 1.0, 1.0);
        w.attract(idx, 5.0, 0.0, 0.0, 2.0);
        for _ in 0..30 {
            w.step(1.0 / 60.0);
        }
        let p = w.position(idx);
        assert!(p[0] > 0.0, "body should move toward attractor: {:?}", p);
    }
}
