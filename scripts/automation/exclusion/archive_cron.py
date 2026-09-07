"""ArchiveCronJob - 24h idempotency_keys 归档 (per F-07 + NFR-OPS-02).

per docs/architecture/2026-09-07-exclusion-idempotency/03-detailed-design.md §1.1 M-15
per PHASE-EXCLUSION-IDEMPOTENCY-IMPL-REPORT.md §1.1 EX-07

24h 把 idempotency_keys 中 expires_at < NOW() 的行移到 idempotency_keys_archive.

守门合规:
- 守门 #5 env 安全
- 守门 #19 v19 Python 化
- 守门 #13 d Transaction 100% audit 必携
"""

from datetime import datetime, timezone
from typing import Optional


class ArchiveCronJob:
    """24h 归档 cron job (mock 实现, per H2 v18 实证)."""

    def __init__(self, pool):
        self.pool = pool
        self.archived_count = 0
        self.last_run: Optional[datetime] = None

    async def run_once(self) -> int:
        """单次归档: 移动 expires_at < NOW() 的行到 archive."""
        async with self.pool.acquire() as conn:
            async with conn.transaction():
                # 1. 复制过期行到 archive
                count = await conn.fetchval(
                    """
                    WITH moved AS (
                        DELETE FROM idempotency_keys
                        WHERE expires_at < NOW()
                        RETURNING *
                    )
                    INSERT INTO idempotency_keys_archive
                    SELECT id, key, key_type, request_fingerprint, response_status,
                           response_body, tenant_id, workspace_id, actor_id, trace_id,
                           created_at, expires_at, NOW() AS archived_at
                    FROM moved
                    RETURNING 1
                    """,
                )
                self.archived_count = count or 0
                self.last_run = datetime.now(timezone.utc)
                return self.archived_count

    async def run_loop(self, interval_s: int = 86400) -> None:
        """循环执行, 默认 24h."""
        import asyncio
        while True:
            await self.run_once()
            await asyncio.sleep(interval_s)
