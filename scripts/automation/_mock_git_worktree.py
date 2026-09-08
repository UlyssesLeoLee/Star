#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
scripts/automation/_mock_git_worktree.py
Mock git worktree CLI wrapper (per 守门 #22 + ADR-0049 v0.1)

职责:
  - 模拟 git worktree add 调用 (PoC 阶段)
  - 真实 Git 集成推 G-WT-02 (per HANDOFF-ST-001 H2 阻塞解除后启动)
  - 不污染 main 编译链 (per 守门 #22)
  - 走 subprocess, M-N8 create_node 异步 fire-and-forget 调用

用法 (per create_node.py _invoke_mock_git_worktree):
    python _mock_git_worktree.py add \\
        --worktree-id=wt-abc123 \\
        --task-id=task-xyz \\
        --tenant-id=t1

后续 (G-WT-02):
    替换为 subprocess.run(["git", "worktree", "add", ...], check=True)
    + per-worktree branch (per INV-WT-03 Runtime Anchor)
    + per-tenant 路径隔离 (per INV-WT-08 tenant_id 必带)
"""
from __future__ import annotations

import argparse
import json
import logging
import sys
import time
from pathlib import Path

logger = logging.getLogger("mock_git_worktree")

# 默认 worktree 根目录 (PoC 内存版, 真实路径走 G-WT-01 配置)
POC_WORKTREE_ROOT = Path("D:/Star/.worktrees/auto-agent")


def cmd_add(args: argparse.Namespace) -> int:
    """mock git worktree add (PoC)

    真实接入 (G-WT-02):
        1. worktree 路径 = ${POC_WORKTREE_ROOT}/{tenant_id}/{worktree_id}
        2. branch 名 = agent/{sa_type}/{task_id}
        3. 真实 git worktree add -b {branch} {path}
        4. 落 audit log (per 守门 #13 d Transaction)
    """
    worktree_id = args.worktree_id
    task_id = args.task_id
    tenant_id = args.tenant_id

    # PoC: 写一个标记文件, 真实 worktree 目录待 G-WT-02 拍板
    worktree_path = POC_WORKTREE_ROOT / tenant_id / worktree_id
    worktree_path.mkdir(parents=True, exist_ok=True)
    marker = worktree_path / ".MOCK_WORKTREE"
    marker.write_text(
        json.dumps({
            "worktree_id": worktree_id,
            "task_id": task_id,
            "tenant_id": tenant_id,
            "created_at_ms": int(time.time() * 1000),
            "note": "Mock git worktree (per 守门 #22 + ADR-0049 v0.1). Real git integration: G-WT-02.",
        }, ensure_ascii=False, indent=2),
        encoding="utf-8",
    )

    logger.info("mock_git_worktree.add: wt=%s task=%s tenant=%s path=%s", worktree_id, task_id, tenant_id, worktree_path)
    print(json.dumps({
        "ok": True,
        "operation": "add",
        "worktree_id": worktree_id,
        "task_id": task_id,
        "tenant_id": tenant_id,
        "path": str(worktree_path),
        "mock": True,
    }, ensure_ascii=False))
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description="Mock git worktree CLI (per ADR-0049 + 守门 #22)")
    subparsers = parser.add_subparsers(dest="command", required=True)

    add_p = subparsers.add_parser("add", help="mock git worktree add")
    add_p.add_argument("--worktree-id", required=True)
    add_p.add_argument("--task-id", required=True)
    add_p.add_argument("--tenant-id", required=True)

    args = parser.parse_args()
    if args.command == "add":
        return cmd_add(args)
    parser.error(f"unknown command: {args.command}")
    return 1


if __name__ == "__main__":
    sys.exit(main())
