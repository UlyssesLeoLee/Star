#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""agent_runtime.config — Runtime 配置 (per SRS-STAR-AGENT-RUNTIME-001 §7.4 + §100-§101)。

所有资源控制参数配置化, 不得硬编码进业务逻辑 (per SRS §7.4)。
Fail Safe (per SRS §101): 配置非法 → 抛 ConfigError, 绝不静默退化为
Unlimited Concurrency / Unlimited Queue / Unlimited Cache。

约束: 标准库 only (dataclasses), 无第三方依赖。
"""

from __future__ import annotations

from dataclasses import dataclass, replace

# 队列溢出策略 (per SRS §34 REQ-SCH-005): Reject / Delay / Drop Low Priority /
# Persist / Throttle Producer。禁止 OOM, 禁止无界增长。
OVERFLOW_POLICIES = ("Reject", "Delay", "DropLow", "Persist", "Throttle")

# Fail Safe 默认: 溢出默认 Throttle (per SRS §101, 不允许默认 Reject 之外的任何
# 无界行为; Throttle 告知 retry_after, 不丢不积压)。
DEFAULT_OVERFLOW_POLICY = "Throttle"


class ConfigError(ValueError):
    """配置非法 (per SRS §101 Fail Safe: 拒绝启动, 不静默退化)。"""

    def __init__(self, problems):
        self.problems = list(problems)
        super().__init__("; ".join(self.problems))


@dataclass(frozen=True)
class RuntimeConfig:
    """STAR Agent Runtime 资源控制参数 (per SRS §100)。

    Defaults 依据:
        - ecs_enable_threshold=12 / ecs_disable_threshold=8
          (per SRS §7.3: 进入 ECS ≥ 12 持续 ≥ 30s, 退出 ECS ≤ 8 持续 ≥ 300s)
        - 迟滞区 10-11 (per SRS §7.2), 9 必须 Lightweight (per SRS §7.1)
        - max_hot_agents=100 (per SRS §30 例)
        - process_pool_size=16 (per 02-BD §3.1: 8-16 常驻 worker)
        - hot_to_warm 60s / warm_to_cold 600s (per 03-DD §3.2)
        - 队列 retention 7 天 / DLQ 30 天 (per 03-DD §5.1/§5.2, W 类)
    """

    # Runtime Mode (per SRS §7)
    ecs_enable_threshold: int = 12
    ecs_disable_threshold: int = 8
    ecs_enable_stable_duration: float = 30.0
    ecs_disable_stable_duration: float = 300.0
    # Bounded Concurrency (per SRS §44)
    max_hot_agents: int = 100
    max_llm_requests: int = 16
    max_tool_requests: int = 32
    max_mcp_requests: int = 16
    max_rag_requests: int = 8
    max_context_loads: int = 8
    # Queue + Backpressure (per SRS §46-§47)
    max_queue_size: int = 10_000
    overflow_policy: str = DEFAULT_OVERFLOW_POLICY
    throttle_retry_after: float = 60.0
    delay_overflow_ms: float = 200.0
    # Lifecycle (per 03-DD §3.2)
    hot_to_warm_timeout: float = 60.0
    warm_to_cold_idle_timeout: float = 600.0
    restore_timeout: float = 30.0
    eviction_batch_size: int = 100
    # Retry (per SRS §51)
    max_retry: int = 3
    retry_base_delay: float = 1.0
    retry_backoff_factor: float = 2.0
    # Shared Runtime (per SRS §29)
    process_pool_size: int = 16
    rate_limit_capacity: int = 10
    rate_limit_refill_per_sec: float = 2.0
    # Event / Mailbox (per SRS §19)
    mailbox_max_size: int = 256
    # Memory budget (per SRS §62/§65)
    warm_agent_memory_budget_bytes: int = 100 * 1024
    queue_retention_seconds: int = 7 * 86400
    dlq_retention_seconds: int = 30 * 86400

    def validate(self) -> "RuntimeConfig":
        """Fail Safe 校验 (per SRS §101): 返回 self 或抛 ConfigError。"""
        problems = []
        if self.ecs_enable_threshold <= 0:
            problems.append("ecs_enable_threshold 必须 > 0")
        if self.ecs_disable_threshold < 0:
            problems.append("ecs_disable_threshold 必须 >= 0")
        # 迟滞区必须非空: enable - 3 (9 必须 Lightweight 上界) > disable
        if self.ecs_enable_threshold - 3 <= self.ecs_disable_threshold:
            problems.append(
                f"迟滞区为空: ecs_enable_threshold({self.ecs_enable_threshold}) - 3 "
                f"必须 > ecs_disable_threshold({self.ecs_disable_threshold})"
            )
        if self.ecs_enable_stable_duration < 0 or self.ecs_disable_stable_duration < 0:
            problems.append("stable_duration 必须 >= 0")
        if self.max_hot_agents < 1:
            problems.append("max_hot_agents 必须 >= 1 (禁止无 HOT 上限)")
        if self.max_queue_size < 1:
            problems.append("max_queue_size 必须 >= 1 (禁止无界队列)")
        if self.overflow_policy not in OVERFLOW_POLICIES:
            problems.append(
                f"overflow_policy 必须在 {OVERFLOW_POLICIES}, 当前: {self.overflow_policy!r}"
            )
        for name in (
            "max_llm_requests",
            "max_tool_requests",
            "max_mcp_requests",
            "max_rag_requests",
            "max_context_loads",
        ):
            if getattr(self, name) < 1:
                problems.append(f"{name} 必须 >= 1 (禁止 unlimited spawn, per SRS §44)")
        if self.hot_to_warm_timeout <= 0 or self.warm_to_cold_idle_timeout <= 0:
            problems.append("lifecycle timeout 必须 > 0")
        if self.max_retry < 0:
            problems.append("max_retry 必须 >= 0 (禁止无限 retry, per SRS §51)")
        if self.process_pool_size < 1:
            problems.append("process_pool_size 必须 >= 1")
        if self.mailbox_max_size < 1:
            problems.append("mailbox_max_size 必须 >= 1 (禁止无界 mailbox)")
        if problems:
            raise ConfigError(problems)
        return self

    def with_override(self, **kwargs) -> "RuntimeConfig":
        """派生配置副本 (frozen dataclass + replace)。"""
        return replace(self, **kwargs).validate()
