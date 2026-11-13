#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""agent_runtime.metrics — 可观测性指标 (per SRS §61 REQ-OBS-001 + 02-BD §6.4)。

指标清单 (per SRS §61, 全量落字段):
    logical_agent_count / resident_agent_count / hot_agent_count /
    warm_agent_count / cold_agent_count / runtime_mode / ecs_switch_count /
    rss_bytes / heap_bytes / shared_runtime_bytes / ecs_world_bytes /
    context_cache_bytes / rag_cache_bytes / avg_warm_agent_bytes /
    avg_hot_agent_bytes / ready_queue_depth / schedule_latency /
    hot_slot_usage / agent_wait_time / active_llm_requests / llm_queue_depth /
    llm_latency / input_tokens / output_tokens / errors / active_tool_calls /
    tool_queue_depth / tool_latency / tool_errors

Trace (per SRS §62 REQ-TRACE-001): trace_id 贯穿 Event/Agent/Scheduler
(模型层已带 trace_id, per models.AgentEvent / Task)。

约束: 标准库 only。
"""

from __future__ import annotations

import threading
from dataclasses import dataclass
from typing import Dict, List, Optional


def percentile(sorted_samples: List[float], q: float) -> Optional[float]:
    """分位数 (P50/P95/P99, per SRS §64 benchmark 记录)。"""
    if not sorted_samples:
        return None
    s = sorted(sorted_samples)
    idx = int(q * (len(s) - 1))
    return s[idx]


@dataclass
class RuntimeMetrics:
    """Runtime 指标快照 (per SRS §61 REQ-OBS-001 字段)。"""

    logical_agent_count: int = 0
    resident_agent_count: int = 0
    hot_agent_count: int = 0
    warm_agent_count: int = 0
    cold_agent_count: int = 0
    runtime_mode: str = "Lightweight"
    ecs_switch_count: int = 0
    rss_bytes: int = 0
    heap_bytes: int = 0
    shared_runtime_bytes: int = 0
    ecs_world_bytes: int = 0
    context_cache_bytes: int = 0
    rag_cache_bytes: int = 0
    avg_warm_agent_bytes: float = 0.0
    avg_hot_agent_bytes: float = 0.0
    ready_queue_depth: int = 0
    schedule_latency_p50: Optional[float] = None
    schedule_latency_p95: Optional[float] = None
    schedule_latency_p99: Optional[float] = None
    hot_slot_usage: float = 0.0
    agent_wait_time_sec: float = 0.0
    active_llm_requests: int = 0
    llm_queue_depth: int = 0
    llm_latency_p95: Optional[float] = None
    input_tokens: int = 0
    output_tokens: int = 0
    errors: int = 0
    active_tool_calls: int = 0
    tool_queue_depth: int = 0
    tool_latency_p95: Optional[float] = None
    tool_errors: int = 0

    def to_dict(self) -> Dict[str, object]:
        """dict 投影 (JSON 友好)。"""
        return {k: v for k, v in self.__dict__.items()}


class Observer:
    """Metrics System 骨架 (per 02-BD §3.4 MetricsSystem)。

    counters + latency 采样, snapshot() 合并 Runtime 状态 (per SRS §61)。
    """

    def __init__(self):
        self._lock = threading.Lock()
        self.errors = 0
        self.tool_errors = 0
        self.input_tokens = 0
        self.output_tokens = 0
        self.dispatched = 0
        self.schedule_latency_samples: List[float] = []
        self.llm_latency_samples: List[float] = []
        self.tool_latency_samples: List[float] = []

    def inc_dispatched(self) -> None:
        """per 03-DD §2.2 Dispatcher.dispatch step 5。"""
        with self._lock:
            self.dispatched += 1

    def inc_error(self) -> None:
        with self._lock:
            self.errors += 1

    def inc_tool_error(self) -> None:
        with self._lock:
            self.tool_errors += 1

    def add_tokens(self, input_tokens: int, output_tokens: int) -> None:
        with self._lock:
            self.input_tokens += input_tokens
            self.output_tokens += output_tokens

    def record_schedule_latency(self, sec: float) -> None:
        with self._lock:
            self.schedule_latency_samples.append(sec)

    def record_llm_latency(self, sec: float) -> None:
        with self._lock:
            self.llm_latency_samples.append(sec)

    def record_tool_latency(self, sec: float) -> None:
        with self._lock:
            self.tool_latency_samples.append(sec)

    def snapshot(self, runtime_metrics: RuntimeMetrics) -> RuntimeMetrics:
        """用 Observer 采样补全 RuntimeMetrics (per SRS §61)。"""
        with self._lock:
            runtime_metrics.errors = self.errors
            runtime_metrics.tool_errors = self.tool_errors
            runtime_metrics.input_tokens = self.input_tokens
            runtime_metrics.output_tokens = self.output_tokens
            runtime_metrics.schedule_latency_p50 = percentile(self.schedule_latency_samples, 0.50)
            runtime_metrics.schedule_latency_p95 = percentile(self.schedule_latency_samples, 0.95)
            runtime_metrics.schedule_latency_p99 = percentile(self.schedule_latency_samples, 0.99)
            runtime_metrics.llm_latency_p95 = percentile(self.llm_latency_samples, 0.95)
            runtime_metrics.tool_latency_p95 = percentile(self.tool_latency_samples, 0.95)
        return runtime_metrics
