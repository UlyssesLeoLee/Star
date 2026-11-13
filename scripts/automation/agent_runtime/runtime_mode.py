#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""agent_runtime.runtime_mode — Runtime 双模式管理 (per SRS §6-§7)。

规则 (per SRS §7.1 STAR 适配: L0 调度层永远在, 模式切换只针对 L1):
    | L1 Resident Agent | 模式                      |
    | 0-8               | Lightweight               |
    | 9                 | 必须 Lightweight          |
    | 10-11             | 迟滞区 (保持当前模式)      |
    | >= 12             | ECS Eligible              |

时间稳定条件 (per SRS §7.3, 数值必须配置化 per §7.4):
    - 进入 ECS: Agent >= ecs_enable_threshold (默认 12) 且持续 >= 30s
    - 退出 ECS: Agent <= ecs_disable_threshold (默认 8) 且持续 >= 300s
    - 9 必须 Lightweight (per §7.1): ECS 模式下 count 掉到
      max(disable_threshold, enable_threshold - 3) 以下开始退出计时。

Break-even (per SRS §7.5): 无论 benchmark 结果, < 10 Agent 不得启用完整 ECS
(硬约束, per SRS §3.2)。

约束: 标准库 only; 时间注入 (now 参数) 保证测试确定性。
"""

from __future__ import annotations

from enum import Enum
from typing import Optional

from .config import RuntimeConfig


class RuntimeMode(Enum):
    """Lightweight / ECS (per SRS §6)。"""

    Lightweight = "Lightweight"
    Ecs = "ECS"


class RuntimeModeManager:
    """Runtime Mode Manager (per 02-BD §2.2 + SRS §7)。"""

    def __init__(self, config: RuntimeConfig):
        config.validate()
        self.config = config
        self.mode = RuntimeMode.Lightweight  # 默认 Lightweight (per SRS §3.1)
        self.switch_count = 0
        self._resident_n = 0
        # 当前候选切换方向的稳定计时起点 (per SRS §7.3 时间稳定条件)
        self._stable_since: Optional[float] = None

    def resident_count(self) -> int:
        """当前 resident agent 数。"""
        return self._resident_n

    def set_resident_count(self, count: int) -> None:
        """更新 resident agent 数 (L1 注册/注销时调用)。"""
        self._resident_n = max(0, int(count))

    def _exit_candidate(self, count: int) -> bool:
        """ECS 退出候选: count <= disable_threshold 或 count <= 9 (必须 Lightweight)。

        max(disable_threshold, enable_threshold - 3) 覆盖两个规则:
        - §7.3: 退出 ECS 当 count <= ecs_disable_threshold
        - §7.1: 9 必须 Lightweight (enable_threshold=12 → 上界 9)
        """
        upper = max(self.config.ecs_disable_threshold, self.config.ecs_enable_threshold - 3)
        return count <= upper

    def decide(self, now: float, count: int) -> RuntimeMode:
        """按 §7.1-§7.3 判定目标模式 (含稳定时长), 并执行切换。"""
        cfg = self.config
        if self.mode is RuntimeMode.Lightweight:
            if count >= cfg.ecs_enable_threshold:
                # 进入 ECS 候选: >= 12 持续 >= enable_stable_duration
                if self._stable_since is None:
                    self._stable_since = now
                elif now - self._stable_since >= cfg.ecs_enable_stable_duration:
                    self.mode = RuntimeMode.Ecs
                    self.switch_count += 1
                    self._stable_since = None
                # else: 保持 Lightweight (计时中)
            else:
                # 迟滞区 10-11 / 0-9: 保持 Lightweight (per §7.2 + §7.1)
                self._stable_since = None
        else:  # Ecs
            if self._exit_candidate(count):
                # 退出候选: <= disable_threshold 或 <= 9 持续 >= disable_stable_duration
                if self._stable_since is None:
                    self._stable_since = now
                elif now - self._stable_since >= cfg.ecs_disable_stable_duration:
                    self.mode = RuntimeMode.Lightweight
                    self.switch_count += 1
                    self._stable_since = None
            else:
                # 迟滞区 10-11 或更高: 保持 ECS (per §7.2)
                self._stable_since = None
        return self.mode

    def get_mode(self) -> RuntimeMode:
        return self.mode

    def hysteresis_zone(self, count: int) -> bool:
        """count 是否在迟滞区 (per SRS §7.2: enable_threshold-2 .. enable_threshold-1)。

        默认配置: 10-11。9 不属于迟滞区 (9 必须 Lightweight, per §7.1)。
        """
        lo = self.config.ecs_enable_threshold - 2
        hi = self.config.ecs_enable_threshold - 1
        return lo <= count <= hi
