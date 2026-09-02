#!/usr/bin/env python3
"""
normalize_output_md.py — 把 worker output.md 从 GBK 转 UTF-8 (per Mavis 终审)

Per 守门 #9 v3: worker 实证 trail (git log -p --follow 看到 output.md 字节级)
不会被这次规范化破坏 (commit hash 仍指向原 commit, content 改写但 git 视为
正常修改).

但 worker 自己 commit 没改 output.md, output.md 是 worker 在 worktree 写的
untracked 状态. 现在 Mavis 合并时规范化 encoding, 方便 review.
"""
from pathlib import Path

REPO = Path("D:/Star")
output_path = REPO / "docs/briefs/star-nav-completion-001.output.md"

# 用 GBK 读, 用 UTF-8 写
with open(output_path, "rb") as f:
    raw = f.read()

# 检测: 是否有 GBK 能解但 UTF-8 解不了的字节
try:
    text_gbk = raw.decode("gbk")
except UnicodeDecodeError:
    print(f"FAIL: {output_path} 不是纯 GBK")
    raise SystemExit(1)

# 如果已经是 UTF-8 编码, 不需要转
try:
    raw.decode("utf-8")
    is_utf8 = True
except UnicodeDecodeError:
    is_utf8 = False

if is_utf8:
    print(f"SKIP: {output_path} 已经是 UTF-8 编码, 不需要规范化")
    raise SystemExit(0)

# 转 UTF-8 (without BOM)
with open(output_path, "wb") as f:
    f.write(text_gbk.encode("utf-8"))

print(f"OK: {output_path} GBK → UTF-8 ({len(raw)} bytes → {len(text_gbk.encode('utf-8'))} bytes)")
