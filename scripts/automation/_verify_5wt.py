#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""5 wt 守门 #9 二次验证 (git log --follow 实证 subagent 真有 commit)"""

import subprocess
from pathlib import Path

WTS = [
    ("wt-p-h2-1", "P3-H2-1", "bg_3ae66b96-7601-4ccc-b422-f9e01ee9417d"),
    ("wt-p-b5", "P3-B.5", "bg_0e244790-0ef9-4a67-a7d7-590769e238e9"),
    ("wt-p-b6", "P3-B.6", "bg_f8dc2b00-5bd7-411d-8f83-a0a569cc96eb"),
    ("wt-p-c6", "P3-C.6", "bg_7af055f5-3eb4-4735-b0a6-761600fa3113"),
    ("wt-p-f6", "P3-F.6", "bg_c06c3bc5-5ed9-4a84-b9f3-74a792e979ed"),
]

WT_BASE = Path(r"D:\Star\.worktrees")

print("=== 守门 #9 二次验证 (subagent 实证) ===")
print()
for wt_dir, task_id, bg_id in WTS:
    wt_path = WT_BASE / wt_dir
    r = subprocess.run(
        ["git", "log", "--oneline", "-5", "--", "scripts/automation/"],
        capture_output=True, cwd=str(wt_path), timeout=10,
    )
    log = r.stdout.decode('utf-8', errors='replace').strip()
    has_new = "094284b" not in log or len(log.split("\n")) > 1
    print(f"  {task_id} ({bg_id[:8]}): {wt_dir}")
    if log:
        for line in log.split("\n"):
            print(f"    {line}")
    else:
        print("    (no commits on scripts/automation/)")
    print(f"    new_commit_progress: {has_new}")
    print()
