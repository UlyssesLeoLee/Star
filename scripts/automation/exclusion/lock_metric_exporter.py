"""5 Prometheus 锁 metric exporter (per F-13 + 02 §7.4).

per docs/architecture/2026-09-07-exclusion-idempotency/03-detailed-design.md §2.4.3
per PHASE-EXCLUSION-IDEMPOTENCY-IMPL-REPORT.md §1.1 EX-07

5 metric (per 02 §7.4):
- star_lock_acquired_total (counter, 按 tier/result)
- star_lock_wait_seconds (histogram)
- star_lock_hold_seconds (histogram)
- star_lock_timeout_total (counter, 按 tier)
- star_idempotency_dedup_total (counter, 按 key_type)

守门合规:
- 守门 #5 env 安全
- 守门 #19 v19 Python 化
- 守门 #23 AI mock: 5 metric 暴露走 prometheus_client 标准格式, 不开真实 Prometheus server
"""

import time
from collections import defaultdict
from dataclasses import dataclass, field
from enum import Enum
from typing import Optional


class LockTier(str, Enum):
    L0_UI = "L0_UI"
    L1_TopAgent = "L1_TopAgent"
    L2_SubAgent = "L2_SubAgent"
    L3_Domain = "L3_Domain"


class AcquireResult(str, Enum):
    SUCCESS = "success"
    TIMEOUT = "timeout"
    VERSION_MISMATCH = "version_mismatch"
    HELD = "held"


@dataclass
class LockMetrics:
    """5 metric 累计 (mock 实现, 实际生产用 prometheus_client)."""

    acquired_total: dict = field(default_factory=lambda: defaultdict(int))
    wait_seconds: list = field(default_factory=list)
    hold_seconds: list = field(default_factory=list)
    timeout_total: dict = field(default_factory=lambda: defaultdict(int))
    idempotency_dedup_total: dict = field(default_factory=lambda: defaultdict(int))

    def record_acquired(self, tier: LockTier, result: AcquireResult, wait_seconds: float = 0.0) -> None:
        self.acquired_total[(tier.value, result.value)] += 1
        if wait_seconds > 0:
            self.wait_seconds.append(wait_seconds)

    def record_hold(self, tier: LockTier, hold_seconds: float) -> None:
        if hold_seconds > 0:
            self.hold_seconds.append(hold_seconds)

    def record_timeout(self, tier: LockTier) -> None:
        self.timeout_total[tier.value] += 1

    def record_dedup(self, key_type: str) -> None:
        self.idempotency_dedup_total[key_type] += 1

    def to_prometheus_text(self) -> str:
        """转换为 Prometheus text 暴露格式 (mock)."""
        lines = []
        # acquired_total
        lines.append("# HELP star_lock_acquired_total Total lock acquisitions")
        lines.append("# TYPE star_lock_acquired_total counter")
        for (tier, result), count in self.acquired_total.items():
            lines.append(f'star_lock_acquired_total{{tier="{tier}",result="{result}"}} {count}')

        # wait_seconds (histogram → p50/p95/p99 summary)
        lines.append("# HELP star_lock_wait_seconds Lock wait time")
        lines.append("# TYPE star_lock_wait_seconds summary")
        if self.wait_seconds:
            sorted_ws = sorted(self.wait_seconds)
            n = len(sorted_ws)
            p50 = sorted_ws[n // 2]
            p95 = sorted_ws[min(n - 1, int(n * 0.95))]
            p99 = sorted_ws[min(n - 1, int(n * 0.99))]
            lines.append(f'star_lock_wait_seconds{{quantile="0.5"}} {p50}')
            lines.append(f'star_lock_wait_seconds{{quantile="0.95"}} {p95}')
            lines.append(f'star_lock_wait_seconds{{quantile="0.99"}} {p99}')

        # hold_seconds
        lines.append("# HELP star_lock_hold_seconds Lock hold time")
        lines.append("# TYPE star_lock_hold_seconds summary")
        if self.hold_seconds:
            sorted_hs = sorted(self.hold_seconds)
            n = len(sorted_hs)
            p50 = sorted_hs[n // 2]
            p95 = sorted_hs[min(n - 1, int(n * 0.95))]
            p99 = sorted_hs[min(n - 1, int(n * 0.99))]
            lines.append(f'star_lock_hold_seconds{{quantile="0.5"}} {p50}')
            lines.append(f'star_lock_hold_seconds{{quantile="0.95"}} {p95}')
            lines.append(f'star_lock_hold_seconds{{quantile="0.99"}} {p99}')

        # timeout_total
        lines.append("# HELP star_lock_timeout_total Lock timeouts")
        lines.append("# TYPE star_lock_timeout_total counter")
        for tier, count in self.timeout_total.items():
            lines.append(f'star_lock_timeout_total{{tier="{tier}"}} {count}')

        # idempotency_dedup_total
        lines.append("# HELP star_idempotency_dedup_total Idempotency dedup hits")
        lines.append("# TYPE star_idempotency_dedup_total counter")
        for key_type, count in self.idempotency_dedup_total.items():
            lines.append(f'star_idempotency_dedup_total{{key_type="{key_type}"}} {count}')

        return "\n".join(lines)


class LockMetricExporter:
    """5 metric exporter 入口."""

    def __init__(self):
        self.metrics = LockMetrics()

    def record_acquired(
        self,
        tier: LockTier,
        result: AcquireResult,
        wait_seconds: float = 0.0,
    ) -> None:
        self.metrics.record_acquired(tier, result, wait_seconds)

    def record_hold(self, tier: LockTier, hold_seconds: float) -> None:
        self.metrics.record_hold(tier, hold_seconds)

    def record_timeout(self, tier: LockTier) -> None:
        self.metrics.record_timeout(tier)

    def record_dedup(self, key_type: str) -> None:
        self.metrics.record_dedup(key_type)

    def export(self) -> str:
        return self.metrics.to_prometheus_text()

    def reset(self) -> None:
        """测用: 重置所有 metric."""
        self.metrics = LockMetrics()
