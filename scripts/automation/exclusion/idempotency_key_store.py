"""幂等键存储 (IdempotencyKeyStore) - per Star-EI F-07 + F-09 + F-10 双键 dedup.

per docs/architecture/2026-09-07-exclusion-idempotency/03-detailed-design.md §2.2.1
per PHASE-EXCLUSION-IDEMPOTENCY-IMPL-REPORT.md §1.1 EX-03

双键模式 (per ADR-0048 D-02):
1. 优先用 client_uuid (客户端 SDK 集成)
2. 缺失 fallback business_hash (server-side: (tenant_id, resource_type, resource_id, operation, version))
3. 两者都缺失则拒绝 (返回 IDEMPOTENCY_KEY_MISSING 400)

守门合规:
- 守门 #5 env 安全
- 守门 #13 a L0 协调
- 守门 #19 v19 Python 化
"""

import hashlib
import json
import uuid
from dataclasses import dataclass
from typing import Any, Optional


@dataclass
class IdempotencyResult:
    """幂等查询结果."""
    found: bool
    response_status: Optional[int] = None
    response_body: Optional[dict] = None
    is_dedup: bool = False  # True = 命中 dedup (返回首次响应)
    key_type: str = "client"  # client / business / dual


class IdempotencyKeyMissing(Exception):
    """Idempotency-Key 缺失, 双键都缺失时拒绝 (per F-10)."""
    pass


class IdempotencyKeyReuse(Exception):
    """key 复用但 request_fingerprint 不匹配 (per F-07)."""
    pass


class IdempotencyKeyStore:
    """双键幂等键存储 (per ADR-0048 D-02).
    
    用法:
        async with IdempotencyKeyStore(pool) as store:
            # 1. 检查是否已存在
            existing = await store.get(client_key, business_key, fingerprint, tenant_id)
            if existing.found:
                return existing.response_body  # 命中 dedup, 返回首次响应
            
            # 2. 执行业务逻辑
            result = await do_business()
            
            # 3. 写 idempotency_keys (首次执行)
            await store.put(client_key, business_key, fingerprint, tenant_id, result)
    """

    def __init__(self, pool):
        self.pool = pool

    @staticmethod
    def compute_business_hash(
        tenant_id: uuid.UUID,
        resource_type: str,
        resource_id: str,
        operation: str,
        version: int,
    ) -> str:
        """计算业务主键 hash (per F-09 双键 fallback).
        
        算法: SHA-256(tenant_id|resource_type|resource_id|operation|version)
        """
        h = hashlib.sha256(
            f"{tenant_id}|{resource_type}|{resource_id}|{operation}|{version}".encode()
        ).hexdigest()
        return h

    async def get(
        self,
        client_key: Optional[str],
        business_key: Optional[str],
        request_fingerprint: str,
        tenant_id: uuid.UUID,
    ) -> IdempotencyResult:
        """查询幂等键, 命中返回首次响应.
        
        优先 client_key, 缺失 fallback business_key, 两者都缺失抛 IdempotencyKeyMissing.
        """
        if not client_key and not business_key:
            raise IdempotencyKeyMissing(
                "Idempotency-Key header 缺失, 双键都缺失, 拒绝 (per F-10)"
            )

        # 1. 优先查 client_key
        if client_key:
            row = await self.pool.fetchrow(
                """
                SELECT response_status, response_body, request_fingerprint, key_type
                FROM idempotency_keys
                WHERE key = $1 AND tenant_id = $2 AND expires_at > NOW()
                """,
                client_key, tenant_id,
            )
            if row:
                # 校验 fingerprint (防 key 复用但内容不同, per F-07)
                if row["request_fingerprint"] != request_fingerprint:
                    raise IdempotencyKeyReuse(
                        f"key={client_key} 已被使用, 但 request_fingerprint 不匹配"
                    )
                return IdempotencyResult(
                    found=True,
                    response_status=row["response_status"],
                    response_body=json.loads(row["response_body"]) if row["response_body"] else None,
                    is_dedup=True,
                    key_type=row["key_type"],
                )

        # 2. fallback business_key
        if business_key:
            row = await self.pool.fetchrow(
                """
                SELECT response_status, response_body, request_fingerprint, key_type
                FROM idempotency_keys
                WHERE key = $1 AND tenant_id = $2 AND key_type = 'business' AND expires_at > NOW()
                """,
                business_key, tenant_id,
            )
            if row:
                if row["request_fingerprint"] != request_fingerprint:
                    raise IdempotencyKeyReuse(
                        f"business_key={business_key} 已被使用, 但 request_fingerprint 不匹配"
                    )
                return IdempotencyResult(
                    found=True,
                    response_status=row["response_status"],
                    response_body=json.loads(row["response_body"]) if row["response_body"] else None,
                    is_dedup=True,
                    key_type="business",
                )

        return IdempotencyResult(found=False)

    async def put(
        self,
        client_key: Optional[str],
        business_key: Optional[str],
        request_fingerprint: str,
        tenant_id: uuid.UUID,
        workspace_id: uuid.UUID,
        actor_id: uuid.UUID,
        trace_id: uuid.UUID,
        response_status: int,
        response_body: Any,
    ) -> None:
        """写幂等键 (首次执行完成后).
        
        至少需要 client_key 或 business_key, 否则抛 IdempotencyKeyMissing.
        """
        if not client_key and not business_key:
            raise IdempotencyKeyMissing(
                "Idempotency-Key 缺失, 双键都缺失, 拒绝写入 (per F-10)"
            )

        # 双键都存在 → key_type=dual
        if client_key and business_key:
            key = client_key
            key_type = "dual"
        elif client_key:
            key = client_key
            key_type = "client"
        else:
            key = business_key
            key_type = "business"

        await self.pool.execute(
            """
            INSERT INTO idempotency_keys (
                key, key_type, request_fingerprint,
                response_status, response_body,
                tenant_id, workspace_id, actor_id, trace_id, expires_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW() + INTERVAL '24 hours')
            ON CONFLICT (key, tenant_id) DO NOTHING
            """,
            key, key_type, request_fingerprint,
            response_status, json.dumps(response_body, default=str),
            tenant_id, workspace_id, actor_id, trace_id,
        )
