#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""5 wt 状态检查 (per 守门 #9 git 实证 + 守门 #12)"""

import subprocess
from pathlib import Path

WTS = [
    ("wt-p-h2-1", "P3-H2-1"),
    ("wt-p-b5", "P3-B.5"),
    ("wt-p-b6", "P3-B.6"),
    ("wt-p-c6", "P3-C.6"),
    ("wt-p-f6", "P3-F.6"),
]

WT_BASE = Path(r"D:\Star\.worktrees")

for wt_dir, task_id in WTS:
    wt_path = WT_BASE / wt_dir
    print(f"=== {wt_dir} ({task_id}) ===")
    # git log -3
    r = subprocess.run(
        ["git", "log", "-3", "--oneline"],
        capture_output=True, cwd=str(wt_path), timeout=10,
    )
    log_out = r.stdout.decode('utf-8', errors='replace').strip()[:200]
    print(f"  log: {log_out}")
    # git status
    r = subprocess.run(
        ["git", "status", "--short"],
        capture_output=True, cwd=str(wt_path), timeout=10,
    )
    status_out = r.stdout.decode('utf-8', errors='replace').strip()[:300]
    status_lines = status_out if status_out else "(clean)"
    print(f"  status: {status_lines}")
    print()
