//! Runtime 双模式管理 (per SRS §6-§7)。
//!
//! 规则 (per SRS §7.1 STAR 适配: L0 调度层永远在, 模式切换只针对 L1):
//! 0-8 Lightweight / 9 必须 Lightweight / 10-11 迟滞区 / ≥12 ECS Eligible。
//! 时间稳定条件 (per SRS §7.3, 可配置 per §7.4):
//! 进 ECS ≥ 12 持续 ≥ 30s; 退 ECS ≤ 8 持续 ≥ 300s。
//! 硬约束 (per SRS §3.2): < 10 Agent 禁止启用完整 ECS Mode, 无论 benchmark。

use std::fmt;
use std::time::{Duration, Instant};

/// Runtime 模式 (per SRS §6)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuntimeMode {
    /// 少量 Agent + 简单状态模型 + Shared Runtime (per SRS §6.1)。
    Lightweight,
    /// ECS World / Archetype / 批量系统 (per SRS §6.2)。
    Ecs,
}

/// 模式阈值配置 (per SRS §7.4: 不得硬编码进业务逻辑)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeModeConfig {
    /// 进入 ECS 阈值 (默认 12)。
    pub ecs_enable_threshold: usize,
    /// 退出 ECS 阈值 (默认 8)。
    pub ecs_disable_threshold: usize,
    /// 进入稳定时长 (默认 30s)。
    pub ecs_enable_stable_duration: Duration,
    /// 退出稳定时长 (默认 300s)。
    pub ecs_disable_stable_duration: Duration,
}

impl Default for RuntimeModeConfig {
    fn default() -> Self {
        Self {
            ecs_enable_threshold: 12,
            ecs_disable_threshold: 8,
            ecs_enable_stable_duration: Duration::from_secs(30),
            ecs_disable_stable_duration: Duration::from_secs(300),
        }
    }
}

/// 配置错误 (per SRS §101 Fail Safe: 非法即拒)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeModeError {
    /// enable 阈值必须 > 0。
    ZeroEnableThreshold,
    /// 迟滞区为空: enable - 3 必须 > disable。
    EmptyHysteresisZone,
}

impl fmt::Display for RuntimeModeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeModeError::ZeroEnableThreshold => write!(f, "ecs_enable_threshold 必须 > 0"),
            RuntimeModeError::EmptyHysteresisZone => {
                write!(
                    f,
                    "迟滞区为空: ecs_enable_threshold - 3 必须 > ecs_disable_threshold"
                )
            }
        }
    }
}

impl std::error::Error for RuntimeModeError {}

impl RuntimeModeConfig {
    /// Fail Safe 校验 (per SRS §101)。
    pub fn validate(&self) -> Result<(), RuntimeModeError> {
        if self.ecs_enable_threshold == 0 {
            return Err(RuntimeModeError::ZeroEnableThreshold);
        }
        if self.ecs_enable_threshold - 3 <= self.ecs_disable_threshold {
            return Err(RuntimeModeError::EmptyHysteresisZone);
        }
        Ok(())
    }
}

/// RuntimeModeManager (per 02-BD §2.2 + SRS §7)。
#[derive(Debug, Clone)]
pub struct RuntimeModeManager {
    config: RuntimeModeConfig,
    mode: RuntimeMode,
    switch_count: u64,
    stable_since: Option<Instant>,
}

impl Default for RuntimeModeManager {
    fn default() -> Self {
        Self::new(RuntimeModeConfig::default())
    }
}

impl RuntimeModeManager {
    /// 新建 (默认 Lightweight, per SRS §3.1)。
    pub fn new(config: RuntimeModeConfig) -> Self {
        assert!(
            config.validate().is_ok(),
            "非法 RuntimeModeConfig (Fail Safe)"
        );
        Self {
            config,
            mode: RuntimeMode::Lightweight,
            switch_count: 0,
            stable_since: None,
        }
    }

    /// 当前模式。
    pub fn get_mode(&self) -> RuntimeMode {
        self.mode
    }

    /// 切换次数 (per SRS §74 ecs_switch_count)。
    pub fn switch_count(&self) -> u64 {
        self.switch_count
    }

    /// ECS 退出候选 (per SRS §7.1 + §7.3):
    /// count ≤ disable_threshold, 或 count ≤ enable-3 (=9, 必须 Lightweight)。
    fn exit_candidate(&self, count: usize) -> bool {
        let upper = self
            .config
            .ecs_disable_threshold
            .max(self.config.ecs_enable_threshold - 3);
        count <= upper
    }

    /// 按 §7.1-§7.3 判定并执行模式切换。
    pub fn decide(&mut self, now: Instant, count: usize) -> RuntimeMode {
        match self.mode {
            RuntimeMode::Lightweight => {
                if count >= self.config.ecs_enable_threshold {
                    match self.stable_since {
                        None => self.stable_since = Some(now),
                        Some(since)
                            if now.duration_since(since)
                                >= self.config.ecs_enable_stable_duration =>
                        {
                            self.mode = RuntimeMode::Ecs;
                            self.switch_count += 1;
                            self.stable_since = None;
                        }
                        Some(_) => {}
                    }
                } else {
                    // 0-9 / 迟滞区 10-11: 保持 Lightweight (per §7.1/§7.2)
                    self.stable_since = None;
                }
            }
            RuntimeMode::Ecs => {
                if self.exit_candidate(count) {
                    match self.stable_since {
                        None => self.stable_since = Some(now),
                        Some(since)
                            if now.duration_since(since)
                                >= self.config.ecs_disable_stable_duration =>
                        {
                            self.mode = RuntimeMode::Lightweight;
                            self.switch_count += 1;
                            self.stable_since = None;
                        }
                        Some(_) => {}
                    }
                } else {
                    // 迟滞区 10-11 或更高: 保持 ECS (per §7.2)
                    self.stable_since = None;
                }
            }
        }
        self.mode
    }

    /// count 是否在迟滞区 (enable-2 .. enable-1; 默认 10-11, 9 不算 per §7.1)。
    pub fn hysteresis_zone(&self, count: usize) -> bool {
        let lo = self.config.ecs_enable_threshold - 2;
        let hi = self.config.ecs_enable_threshold - 1;
        (lo..=hi).contains(&count)
    }
}
