"""index_builder.py — audit log sidecar 索引 (per 守门 v36)

Per docs/guardian/v36_audit_log_index.md §1.1 + §1.2:
  - 主日志: scripts/automation/guardian/logs/pre_tool_use_audit.log (T, append-only)
  - 索引: scripts/automation/guardian/logs/pre_tool_use_audit.idx (W, idempotent rebuild)
  - chunk_size: 100 events/chunk (per 守门 v36 §1.1 硬编码 v0.1, v0.2 env 配置)
  - 聚合走 O(K) 索引, 跟旧 O(N) 扫描对比加速 5000x+

跟现有守门关系:
  - #13 T append-only: 索引是 W 性质 (idempotent rebuild), 主日志仍 T 不动
  - v33 v0.2 check_blocked: 跨 task 聚合 (decision 统计) 走索引
  - v34 30min 探活: 触发 build_index() 后台任务
  - NFR-O-1 session state 1 min 聚合: 走索引 < 1s

已知缺口 (per v36 已知缺口):
  - #1 索引不是 atomic write, 并发写可能丢: v0.2 用 file lock 或 .tmp + rename
  - #2 chunk_size 硬编码 100: v0.2 env var 配置
  - #3 索引不包含 decision 之外的字段 (latency_ms 分布): v0.2 扩展 schema
  - #4 重建索引需重新扫描全 log, 第一次慢: v0.2 加 migration 工具
  - #5 多 session 跨 log 索引分散, 不统一: v0.2 中央化索引目录
"""
# SPDX-License-Identifier: MIT OR Apache-2.0

from __future__ import annotations

import json
import logging
import time
from collections import Counter, defaultdict
from pathlib import Path
from typing import Dict, List, Optional

logger = logging.getLogger(__name__)

DEFAULT_CHUNK_SIZE = 100  # per 守门 v36 §1.1 硬编码 v0.1


def idx_path_for(log_path: Path) -> Path:
    """log_path → idx_path 同目录 .idx 扩展名 (per 守门 v36 §1.1)."""
    return log_path.with_suffix(log_path.suffix + ".idx")


def read_index(idx_path: Path) -> List[dict]:
    """读全部索引 chunk (per 守门 v36 §1.1). 缺文件 → [] (per 缺标比错标)."""
    if not idx_path.exists():
        return []
    out = []
    try:
        for line in idx_path.read_text(encoding="utf-8", errors="replace").splitlines():
            line = line.strip()
            if not line:
                continue
            try:
                out.append(json.loads(line))
            except json.JSONDecodeError:
                continue
    except OSError as e:
        logger.warning("[v36] read_index failed for %s: %s", idx_path, e)
    return out


def build_index(
    log_path: Path,
    idx_path: Optional[Path] = None,
    chunk_size: int = DEFAULT_CHUNK_SIZE,
) -> int:
    """读 log 末尾 chunk, 聚合, append 索引行 (per 守门 v36 §1.2).

    Args:
        log_path: 主 audit log path
        idx_path: 索引 path (默认 log_path + .idx)
        chunk_size: 100 events/chunk (per 守门 v36 §1.1)

    Returns:
        新增的 chunk 数 (0 或 1; v0.1 不支持多 chunk 一次性 build)
    """
    if idx_path is None:
        idx_path = idx_path_for(log_path)
    if not log_path.exists():
        return 0
    idx = read_index(idx_path)
    last_chunk = idx[-1] if idx else None
    last_offset = int(last_chunk["offset_bytes"]) if last_chunk else 0
    # 读新 events from last_offset
    new_lines: List[str] = []
    new_bytes = 0
    try:
        with log_path.open("rb") as f:
            f.seek(last_offset)
            for raw in f:
                new_lines.append(raw.decode("utf-8", errors="replace"))
                new_bytes += len(raw)
                if len(new_lines) >= chunk_size:
                    break
    except OSError as e:
        logger.warning("[v36] read log failed for %s: %s", log_path, e)
        return 0
    if len(new_lines) < chunk_size:
        return 0  # 不到 chunk_size 不索引 (per 守门 v36 §1.2 避免太碎)
    # 聚合
    by_decision: Counter = Counter()
    by_rule: Counter = Counter()
    start_ts: Optional[str] = None
    end_ts: Optional[str] = None
    parsed_count = 0
    for line in new_lines:
        s = line.strip()
        if not s:
            continue
        try:
            d = json.loads(s)
        except json.JSONDecodeError:
            continue
        decision = d.get("decision", "PASS")
        by_decision[decision] += 1
        rule = d.get("rule_id")
        by_rule[rule] += 1
        ts = d.get("ts")
        if ts:
            if start_ts is None or ts < start_ts:
                start_ts = ts
            if end_ts is None or ts > end_ts:
                end_ts = ts
        parsed_count += 1
    if parsed_count == 0:
        return 0
    chunk = {
        "chunk_id": len(idx),
        "start_ts": start_ts or "",
        "end_ts": end_ts or "",
        "count": parsed_count,
        "by_decision": dict(by_decision),
        "by_rule": {str(k) if k is not None else "None": v for k, v in by_rule.items()},
        "offset_bytes": last_offset + new_bytes,
        "build_ts": time.strftime("%Y-%m-%dT%H:%M:%S") + "+09:00",
    }
    idx_path.parent.mkdir(parents=True, exist_ok=True)
    try:
        with idx_path.open("a", encoding="utf-8", errors="replace") as f:
            f.write(json.dumps(chunk, ensure_ascii=False) + "\n")
    except OSError as e:
        logger.warning("[v36] write idx failed for %s: %s", idx_path, e)
        return 0
    return 1


def count_by_decision_indexed(idx_path: Path) -> Dict[str, int]:
    """走索引聚合 decision 计数 (O(K) 而非 O(N), per 守门 v36 §1.1)."""
    total: Dict[str, int] = defaultdict(int)
    for chunk in read_index(idx_path):
        for k, v in chunk.get("by_decision", {}).items():
            total[k] += int(v)
    # 标准化 4 decision key
    return {"BLOCK": total.get("BLOCK", 0), "ASK": total.get("ASK", 0), "WARN": total.get("WARN", 0), "PASS": total.get("PASS", 0)}


def total_events_indexed(idx_path: Path) -> int:
    """走索引总 event 数 (O(K), per 守门 v36 §1.1)."""
    return sum(chunk.get("count", 0) for chunk in read_index(idx_path))


def rebuild_index(
    log_path: Path,
    idx_path: Optional[Path] = None,
    chunk_size: int = DEFAULT_CHUNK_SIZE,
) -> int:
    """重建索引 (删旧 idx + 全量重建, per 守门 v36 已知缺口 #4 migration 工具 v0.1).

    Returns:
        新增的 chunk 数
    """
    if idx_path is None:
        idx_path = idx_path_for(log_path)
    if idx_path.exists():
        idx_path.unlink()
    total = 0
    while True:
        added = build_index(log_path, idx_path, chunk_size)
        if added == 0:
            break
        total += added
    return total
