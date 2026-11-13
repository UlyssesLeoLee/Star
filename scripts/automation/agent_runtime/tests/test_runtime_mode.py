#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""test_runtime_mode — 双模式切换 (per SRS §7.1-§7.5)。"""

import unittest

from automation.agent_runtime.config import RuntimeConfig
from automation.agent_runtime.runtime_mode import RuntimeMode, RuntimeModeManager


def make_manager(**kwargs):
    cfg = RuntimeConfig(**kwargs).validate()
    return RuntimeModeManager(cfg)


class TestRuntimeMode(unittest.TestCase):
    def test_default_lightweight(self):
        m = make_manager()
        self.assertEqual(m.get_mode(), RuntimeMode.Lightweight)

    def test_small_scale_stays_lightweight(self):
        m = make_manager()
        for t, count in ((0, 8), (100, 8), (200, 8)):
            self.assertEqual(m.decide(now=t, count=count), RuntimeMode.Lightweight)

    def test_9_must_be_lightweight_from_ecs(self):
        """per SRS §7.1: 9 必须 Lightweight — ECS 下掉到 9 触发退出计时。"""
        m = make_manager()
        # 进入 ECS: 12 agents 持续 30s
        m.decide(now=0.0, count=12)
        self.assertEqual(m.decide(now=30.0, count=12), RuntimeMode.Ecs)
        # 掉到 9: 退出候选 (enable-3=9), 持续 300s 后回 Lightweight
        self.assertEqual(m.decide(now=31.0, count=9), RuntimeMode.Ecs)
        self.assertEqual(m.decide(now=330.0, count=9), RuntimeMode.Ecs)
        self.assertEqual(m.decide(now=331.0, count=9), RuntimeMode.Lightweight)
        self.assertEqual(m.switch_count, 2)

    def test_hysteresis_10_11_stays_current(self):
        """per SRS §7.2: 10-11 迟滞区保持当前模式, 不反复切换。"""
        m = make_manager()
        # Lightweight 下 11: 保持 Lightweight
        self.assertEqual(m.decide(now=0.0, count=11), RuntimeMode.Lightweight)
        self.assertEqual(m.decide(now=999.0, count=11), RuntimeMode.Lightweight)
        self.assertEqual(m.switch_count, 0)
        # 进入 ECS
        m.decide(now=0.0, count=12)
        self.assertEqual(m.decide(now=30.0, count=12), RuntimeMode.Ecs)
        # ECS 下 10: 保持 ECS (迟滞区)
        for t in (31.0, 100.0, 500.0):
            self.assertEqual(m.decide(now=t, count=10), RuntimeMode.Ecs)
        self.assertEqual(m.switch_count, 1)

    def test_enter_ecs_requires_stable_duration(self):
        """per SRS §7.3: >= 12 且持续 >= 30s 才进 ECS。"""
        m = make_manager()
        self.assertEqual(m.decide(now=0.0, count=12), RuntimeMode.Lightweight)
        self.assertEqual(m.decide(now=29.0, count=12), RuntimeMode.Lightweight)
        self.assertEqual(m.decide(now=30.0, count=12), RuntimeMode.Ecs)
        self.assertEqual(m.switch_count, 1)

    def test_stable_timer_resets_on_dip(self):
        m = make_manager()
        m.decide(now=0.0, count=12)
        m.decide(now=10.0, count=12)
        # 掉回 11 (迟滞区): 计时重置
        self.assertEqual(m.decide(now=15.0, count=11), RuntimeMode.Lightweight)
        # 重新 12: 重新计时 30s
        m.decide(now=16.0, count=12)
        self.assertEqual(m.decide(now=40.0, count=12), RuntimeMode.Lightweight)
        self.assertEqual(m.decide(now=46.0, count=12), RuntimeMode.Ecs)

    def test_exit_ecs_requires_300s(self):
        """per SRS §7.3: <= 8 且持续 >= 300s 才退出 ECS。"""
        m = make_manager()
        m.decide(now=0.0, count=12)
        self.assertEqual(m.decide(now=30.0, count=12), RuntimeMode.Ecs)
        self.assertEqual(m.decide(now=31.0, count=8), RuntimeMode.Ecs)
        self.assertEqual(m.decide(now=330.0, count=8), RuntimeMode.Ecs)
        self.assertEqual(m.decide(now=331.0, count=8), RuntimeMode.Lightweight)

    def test_thresholds_configurable(self):
        """per SRS §7.4: 阈值必须配置化。"""
        m = make_manager(
            ecs_enable_threshold=20,
            ecs_disable_threshold=14,
            ecs_enable_stable_duration=5.0,
            ecs_disable_stable_duration=10.0,
        )
        m.decide(now=0.0, count=20)
        self.assertEqual(m.decide(now=5.0, count=20), RuntimeMode.Ecs)
        self.assertEqual(m.decide(now=6.0, count=14), RuntimeMode.Ecs)
        self.assertEqual(m.decide(now=16.0, count=14), RuntimeMode.Lightweight)

    def test_hysteresis_zone_bounds(self):
        m = make_manager()
        self.assertFalse(m.hysteresis_zone(9))
        self.assertTrue(m.hysteresis_zone(10))
        self.assertTrue(m.hysteresis_zone(11))
        self.assertFalse(m.hysteresis_zone(12))

    def test_resident_count_setter(self):
        m = make_manager()
        m.set_resident_count(7)
        self.assertEqual(m.resident_count(), 7)
        m.set_resident_count(-3)
        self.assertEqual(m.resident_count(), 0)


if __name__ == "__main__":
    unittest.main()
