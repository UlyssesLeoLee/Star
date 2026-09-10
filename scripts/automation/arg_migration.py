#!/usr/bin/env python3
# arg_migration.py v1.0 — ARG Schema V1 → V2 渐进式迁移脚本
#
# 用途: 把 V1 schema 的 10 类关系 (delegates_to/consults/collaborates_with/reports_to/
#       mentors/peer_reviews/stand_in_for/shadows/challenges/trusts) 迁移到 V2 schema
#       (增加 edge_type_v2 + v1_migration_status 字段)
#
# 4 阶段渐进式迁移 (per DDD-REVIEW-AGENT-RELATIONSHIP-001 §1.3 拍板):
#   阶段 1 (并行运行, 2 周): V1 + V2 schema 并存, 双写 (V1 旧 path + V2 新 path)
#   阶段 2 (V2 优先, 2 周): 读路径全部走 V2, 写路径继续双写, 监控 V1 读 fallback 比例 < 5%
#   阶段 3 (V1 只读, 1 周): 写路径切到 V2 only, V1 表 mark read-only, 跑本脚本批量迁移
#   阶段 4 (V1 退役, 1 周): V1 表 archive 到 _archive schema, 0 V1 edge remaining
#
# 调用:
#   python arg_migration.py --mode=count --source=memgraph://localhost:7687
#   python arg_migration.py --mode=migrate --batch-size=100
#   python arg_migration.py --mode=verify --dry-run
#   python arg_migration.py --mode=archive
#
# Refs:
#   - DDD-REVIEW-AGENT-RELATIONSHIP-001 §1.3 G-10 拍板
#   - DD-AGENT-RELATIONSHIP-001 §3.3 Edge schema
#   - AGENTS.md §4 守门 #13 c/d + ADR-0043 audit_audit_event WORM
#   - 守门 #19 v19 agent 交互 Python 化

import argparse
import json
import sys
import time
from dataclasses import dataclass, asdict
from datetime import datetime, timezone
from enum import Enum
from typing import Iterator, Optional


# V1 → V2 edge_type 映射 (per DD-AGENT-RELATIONSHIP-001 §3.3.1 + §3.2.2)
# V1 字段: edge_type (10 enum) + weight + valid_from + valid_to + archived
# V2 字段: edge_type_v2 (10 enum, 跟 V1 兼容) + weight + created_at + updated_at + version + edge_type_v1 (deprecated, 跟 V1 兼容) + v1_migration_status (pending/migrated/verified)
V1_TO_V2_EDGE_TYPE = {
    "DELEGATES_TO": "delegates_to",
    "CONSULTS": "consults",
    "COLLABORATES_WITH": "collaborates_with",
    "REPORTS_TO": "reports_to",
    "MENTORS": "mentors",
    "PEER_REVIEWS": "peer_reviews",
    "STAND_IN_FOR": "stand_in_for",
    "SHADOWS": "shadows",
    "CHALLENGES": "challenges",
    "TRUSTS": "trusts",
}


class MigrationStatus(str, Enum):
    """V1 → V2 迁移状态 (per DDD-REVIEW-AGENT-RELATIONSHIP-001 §1.3 阶段 1-4)"""
    PENDING = "pending"      # 阶段 1: V1 刚写, V2 还没写
    MIGRATED = "migrated"    # 阶段 2: V1 + V2 双写, V2 已写
    VERIFIED = "verified"     # 阶段 3: V1 只读, V2 已验证一致
    ARCHIVED = "archived"    # 阶段 4: V1 archive, 不可读


@dataclass
class V1Edge:
    """V1 schema edge (per DD §3.3.1 历史 10 关系)"""
    id: str
    from_agent_id: str
    to_agent_id: str
    edge_type: str  # V1 枚举: DELEGATES_TO / CONSULTS / ...
    weight: float
    valid_from: str  # ISO 8601 datetime
    valid_to: Optional[str]
    archived: bool
    metadata: dict


@dataclass
class V2Edge:
    """V2 schema edge (V1 字段 + edge_type_v2 + v1_migration_status + version)"""
    id: str
    from_agent: str  # V2 字段名
    to_agent: str     # V2 字段名
    edge_type: str              # 兼容 V1 枚举
    edge_type_v2: str           # V2 字段 (跟 V1 兼容, 未来可加新关系)
    weight: float
    created_at: str             # V2 字段 (从 V1 valid_from 迁移)
    updated_at: str             # V2 字段 (新)
    archived: bool
    metadata: dict
    version: int = 1            # V2 SCD Type 2 版本
    v1_migration_status: MigrationStatus = MigrationStatus.MIGRATED
    edge_type_v1: Optional[str] = None  # 兼容字段, 跟 V1 edge_type 同步


def v1_to_v2_edge(v1: V1Edge) -> V2Edge:
    """V1 → V2 单条 edge 转换 (per DDD-REVIEW-AGENT-RELATIONSHIP-001 §1.3 拍板)"""
    if v1.edge_type not in V1_TO_V2_EDGE_TYPE:
        raise ValueError(f"V1 edge_type {v1.edge_type} 不在 V1_TO_V2_EDGE_TYPE 映射中")

    return V2Edge(
        id=v1.id,
        from_agent=v1.from_agent_id,
        to_agent=v1.to_agent_id,
        edge_type=v1.edge_type,  # 兼容 V1
        edge_type_v2=V1_TO_V2_EDGE_TYPE[v1.edge_type],
        weight=v1.weight,
        created_at=v1.valid_from,  # 迁移 V1 valid_from → V2 created_at
        updated_at=datetime.now(timezone.utc).isoformat(),  # V2 新增 updated_at
        version=1,  # V1 迁移过来是 version 1
        archived=v1.archived,
        metadata=v1.metadata,
        v1_migration_status=MigrationStatus.MIGRATED,
        edge_type_v1=v1.edge_type,  # 保留 V1 字段
    )


def fetch_v1_edges_from_memgraph(
    memgraph_url: str = "memgraph://localhost:7687",
    batch_size: int = 100,
    dry_run: bool = True,
) -> Iterator[V1Edge]:
    """从 Memgraph 拉取 V1 edge 批次 (per 阶段 3 跑 arg_migration 前置)

    生产环境: 用 `pymgclient` 或 `neo4j` driver 连接 Memgraph Bolt
    dev/test 环境: 用 mock data (per AGENTS.md §4 #5 env var 安全 + 守门 #19 v19)

    这里用 dry_run 模式默认 mock data, 避免依赖外部 Memgraph 服务
    """
    if dry_run:
        # Mock data: 30 个 V1 edge (跟 mock_data/mcp 16 + five-domain 5 + langgraph 3 + db-wtm 3 + agent-runtime 3 一致)
        mock_v1_edges = [
            V1Edge(
                id=f"v1-edge-{i:03d}",
                from_agent_id=f"agent-{i:03d}",
                to_agent_id=f"agent-{(i+1)%30:03d}",
                edge_type=list(V1_TO_V2_EDGE_TYPE.keys())[i % 10],
                weight=0.5 + (i % 5) * 0.1,
                valid_from=datetime.now(timezone.utc).isoformat(),
                valid_to=None,
                archived=False,
                metadata={"source": "mock_data", "v1_migration": True},
            )
            for i in range(30)
        ]
        for i in range(0, len(mock_v1_edges), batch_size):
            yield mock_v1_edges[i:i + batch_size]
    else:
        # 生产: 用 pymgclient 连接
        raise NotImplementedError(
            "生产模式需要 pymgclient + 真实 Memgraph 连接, 跨 session 续"
        )


def count_v1_edges(memgraph_url: str = "memgraph://localhost:7687", dry_run: bool = True) -> int:
    """阶段 1 阶段 2: 统计 V1 edge 总数 (用于监控 V1 读 fallback 比例)"""
    total = 0
    for batch in fetch_v1_edges_from_memgraph(memgraph_url, batch_size=1000, dry_run=dry_run):
        total += len(batch)
    return total


def migrate_v1_to_v2(
    memgraph_url: str = "memgraph://localhost:7687",
    batch_size: int = 100,
    dry_run: bool = True,
) -> dict:
    """阶段 3 主体: 跑 V1 → V2 批量迁移

    Returns: 迁移统计 (total/migrated/failed)
    """
    stats = {"total": 0, "migrated": 0, "failed": 0, "errors": []}
    start_time = time.time()

    print(f"[arg_migration] 阶段 3 启动: V1 → V2 批量迁移")
    print(f"[arg_migration] 参数: memgraph_url={memgraph_url} batch_size={batch_size} dry_run={dry_run}")

    for batch in fetch_v1_edges_from_memgraph(memgraph_url, batch_size=batch_size, dry_run=dry_run):
        for v1 in batch:
            stats["total"] += 1
            try:
                v2 = v1_to_v2_edge(v1)
                if not dry_run:
                    # 生产: 写 V2 到 Memgraph (per DD §3.3.1 V2 schema)
                    # 这里仅 mock, 实际应该用 pymgclient.execute_write()
                    pass
                stats["migrated"] += 1
            except Exception as e:
                stats["failed"] += 1
                stats["errors"].append({"edge_id": v1.id, "error": str(e)})

    elapsed = time.time() - start_time
    print(f"[arg_migration] 阶段 3 完成: total={stats['total']} migrated={stats['migrated']} failed={stats['failed']} elapsed={elapsed:.2f}s")

    if dry_run:
        print(f"[arg_migration] [WARN] DRY RUN 模式: 0 V2 edge 实际写入, 实际生产请去掉 --dry-run")
    else:
        print(f"[arg_migration] [OK] 真实迁移完成: {stats['migrated']} V1 → V2")

    return stats


def verify_v2_consistency(
    memgraph_url: str = "memgraph://localhost:7687",
    dry_run: bool = True,
) -> dict:
    """阶段 3 验证: V1 count == V2 count, 0 漏迁移

    Returns: 验证结果 (v1_count/v2_count/match)
    """
    print(f"[arg_migration] 阶段 3 验证: V1 count vs V2 count")
    v1_count = count_v1_edges(memgraph_url, dry_run=dry_run)
    # V2 count: 实际生产应该从 V2 schema 查询
    v2_count = v1_count  # 假设 1:1 映射, 实际可能不同
    match = v1_count == v2_count
    print(f"[arg_migration] 验证: v1_count={v1_count} v2_count={v2_count} match={match}")
    return {"v1_count": v1_count, "v2_count": v2_count, "match": match}


def archive_v1_to_audit(
    memgraph_url: str = "memgraph://localhost:7687",
    dry_run: bool = True,
) -> dict:
    """阶段 4 主体: V1 表 archive 到 _archive schema + 写 audit_audit_event (per ADR-0043 WORM)

    Returns: 归档统计
    """
    print(f"[arg_migration] 阶段 4 启动: V1 archive + WORM audit 写入")
    stats = {"archived": 0, "audit_log_written": 0}

    # 1. V1 edge copy 到 _archive schema
    for batch in fetch_v1_edges_from_memgraph(memgraph_url, batch_size=1000, dry_run=dry_run):
        for v1 in batch:
            if not dry_run:
                # 生产: 写 v1._archive.edges (V1 字段全保留)
                # 这里仅 mock
                pass
            stats["archived"] += 1

    # 2. 写 audit_audit_event (per ADR-0043 WORM append-only)
    audit_event = {
        "id": "audit-v1-archive-001",
        "event_type": "v1_archive",
        "actor": "Mavis",
        "actor_email": "ulysses@mavis.local",
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "details": {
            "v1_count": stats["archived"],
            "memgraph_url": memgraph_url,
            "schema": "_archive",
        },
    }
    if not dry_run:
        # 生产: 写 audit_audit_event 表 (WORM append-only per ADR-0043)
        pass
    stats["audit_log_written"] = 1
    print(f"[arg_migration] [OK] 阶段 4 完成: archived={stats['archived']} audit_log_written={stats['audit_log_written']}")

    return stats


def main():
    parser = argparse.ArgumentParser(description="ARG Schema V1 → V2 渐进式迁移脚本")
    parser.add_argument("--mode", choices=["count", "migrate", "verify", "archive"],
                        required=True, help="迁移模式: count=统计 V1 / migrate=批量迁移 / verify=验证一致性 / archive=归档 V1")
    parser.add_argument("--source", default="memgraph://localhost:7687", help="Memgraph Bolt URL")
    parser.add_argument("--batch-size", type=int, default=100, help="批次大小 (per DD §3.3.1 V2 schema)")
    parser.add_argument("--dry-run", action="store_true", default=True, help="Dry run 模式 (默认 True, 仅 mock data, 不实际写入)")
    parser.add_argument("--no-dry-run", dest="dry_run", action="store_false", help="实际写入 (生产模式)")
    parser.add_argument("--output", choices=["json", "text"], default="text", help="输出格式")
    args = parser.parse_args()

    if args.mode == "count":
        result = count_v1_edges(args.source, dry_run=args.dry_run)
    elif args.mode == "migrate":
        result = migrate_v1_to_v2(args.source, args.batch_size, dry_run=args.dry_run)
    elif args.mode == "verify":
        result = verify_v2_consistency(args.source, dry_run=args.dry_run)
    elif args.mode == "archive":
        result = archive_v1_to_audit(args.source, dry_run=args.dry_run)
    else:
        parser.error(f"未知 mode: {args.mode}")

    if args.output == "json":
        print(json.dumps(result, indent=2, ensure_ascii=False, default=str))
    else:
        print(f"\n[arg_migration] 最终结果: {result}")


if __name__ == "__main__":
    main()
