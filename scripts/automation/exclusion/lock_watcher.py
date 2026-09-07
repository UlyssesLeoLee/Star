"""LockWatcher - 锁过期检测 + 告警 (per F-14 + NFR-REL-02).

per docs/architecture/2026-09-07-exclusion-idempotency/03-detailed-design.md §1.1 M-19
per PHASE-EXCLUSION-IDEMPOTENCY-IMPL-REPORT.md §1.1 EX-04 (跟 SubAgentLock 一起)
per ask_2e8740e6779ac6a8d854c590 拍板

职责:
1. 定期扫描 lease_log, 找出已过期但 released_at IS NULL 的锁 (SubAgent 崩溃)
2. 自动标记 release_reason=timeout
3. 锁泄漏告警 (持有 > 1h 触发 Slack 告警, per G-EI-05 阈值 1h 保守值)

守门合规:
- 守门 #5 env 安全: 不打印 Slack webhook URL
- 守门 #23 AI mock: Slack 告警走 mock (不开真实 Slack API)
- 守门 #19 v19: Python 化
"""

import asyncio
import os
import uuid
from dataclasses import dataclass
from datetime import datetime, timezone
from enum import Enum
from typing import Optional, Callable


# 守门 #5: 不打印 Slack webhook URL, 仅引用
SLACK_WEBHOOK_URL = os.environ.get("STAR_SLACK_WEBHOOK_URL", "https://hooks.slack.com/services/MOCK/MOCK/MOCK")

# 守门 #11 缺标比错标: 阈值 1h 保守值, 真实值需 SRE Lead 拍板 (per G-EI-05)
LEAK_THRESHOLD_S = 3600  # 1h


class WatcherAlertSeverity(str, Enum):
    INFO = "info"
    WARNING = "warning"
    CRITICAL = "critical"


@dataclass
class WatcherAlert:
    """锁告警."""
    severity: WatcherAlertSeverity
    lease_id: Optional[uuid.UUID] = None
    task_id: Optional[uuid.UUID] = None
    subagent_id: Optional[str] = None
    message: str = ""
    detected_at: datetime = None

    def __post_init__(self):
        if self.detected_at is None:
            self.detected_at = datetime.now(timezone.utc)


class LockWatcher:
    """锁过期检测 + 告警.

    用法:
        async with LockWatcher(pool, alert_sender) as watcher:
            # 后台扫描任务, 每 60s 扫一次
            await watcher.start(scan_interval_s=60)
            # ... 业务运行
            await watcher.stop()
    """

    DEFAULT_SCAN_INTERVAL_S = 60

    def __init__(
        self,
        pool,
        alert_sender: Optional[Callable[[WatcherAlert], None]] = None,
    ):
        """Args:
            pool: asyncpg.Pool 实例.
            alert_sender: 告警发送回调 (默认 mock 写日志, 不开真实 Slack).
        """
        self.pool = pool
        self.alert_sender = alert_sender or self._default_alert_sender
        self._scan_task: Optional[asyncio.Task] = None
        self._running = False

    async def start(self, scan_interval_s: int = DEFAULT_SCAN_INTERVAL_S) -> None:
        """启动后台扫描任务."""
        if self._running:
            return
        self._running = True
        self._scan_task = asyncio.create_task(self._scan_loop(scan_interval_s))

    async def stop(self) -> None:
        """停止后台扫描任务."""
        self._running = False
        if self._scan_task:
            self._scan_task.cancel()
            try:
                await self._scan_task
            except asyncio.CancelledError:
                pass
            self._scan_task = None

    async def _scan_loop(self, interval_s: int) -> None:
        """扫描循环, 每 interval_s 秒跑一次 _scan_once."""
        while self._running:
            try:
                alerts = await self._scan_once()
                for alert in alerts:
                    self.alert_sender(alert)
            except Exception:
                # 扫描失败不抛, 静默继续
                pass
            await asyncio.sleep(interval_s)

    async def _scan_once(self) -> list[WatcherAlert]:
        """单次扫描: 检测过期未释放的 lease + 锁泄漏."""
        alerts = []

        # 1. 检测已过期但 released_at IS NULL 的 lease (SubAgent 崩溃)
        expired = await self.pool.fetch(
            """
            SELECT lease_id, task_id, actor_id
            FROM lease_log
            WHERE released_at IS NULL
              AND expires_at < NOW()
            """
        )
        for row in expired:
            # 自动标记 timeout
            await self.pool.execute(
                """
                UPDATE lease_log
                SET released_at = NOW(), release_reason = 'timeout'
                WHERE lease_id = $1 AND released_at IS NULL
                """,
                row["lease_id"],
            )
            alerts.append(WatcherAlert(
                severity=WatcherAlertSeverity.WARNING,
                lease_id=row["lease_id"],
                task_id=row["task_id"],
                message=f"lease {row['lease_id']} 已过期自动 timeout 释放 (per NFR-REL-02)",
            ))

        # 2. 锁泄漏检测: 持有 > LEAK_THRESHOLD_S
        leaked = await self.pool.fetch(
            """
            SELECT lease_id, task_id, actor_id, created_at
            FROM lease_log
            WHERE released_at IS NULL
              AND created_at < NOW() - ($1 || ' seconds')::INTERVAL
            """,
            str(LEAK_THRESHOLD_S),
        )
        for row in leaked:
            alerts.append(WatcherAlert(
                severity=WatcherAlertSeverity.CRITICAL,
                lease_id=row["lease_id"],
                task_id=row["task_id"],
                message=f"lease {row['lease_id']} 持有超 {LEAK_THRESHOLD_S}s 触发锁泄漏告警",
            ))

        return alerts

    def _default_alert_sender(self, alert: WatcherAlert) -> None:
        """默认告警发送器: mock 写日志, 不开真实 Slack (per 守门 #23)."""
        # 守门 #5: 不打印 SLACK_WEBHOOK_URL
        # 守门 #23: 不调真实 Slack API, 仅 mock 写日志
        print(f"[LockWatcher][{alert.severity.value}] {alert.message}")
