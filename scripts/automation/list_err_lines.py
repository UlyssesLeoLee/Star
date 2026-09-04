#!/usr/bin/env python3
"""
scripts/automation/list_err_lines.py v0.1
Phase B.2 batch 3 工具: 从 cargo check --message-format=json 输出提取 err 位置
"""
import json
import sys
import re

# 守门 #5 v2: 强制 UTF-8
if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

if len(sys.argv) < 2:
    print("Usage: python list_err_lines.py <cargo-json-output-file>")
    sys.exit(1)

# 读文件(支持 Windows / 不支持 /dev/stdin)
import io
if sys.argv[1] in ("-", "/dev/stdin"):
    lines = io.TextIOWrapper(sys.stdin.buffer, encoding="utf-8").read().split("\n")
else:
    with open(sys.argv[1], "r", encoding="utf-8") as f:
        lines = f.read().split("\n")

errs = []
for line in lines:
    if not line.strip():
        continue
    try:
        msg = json.loads(line)
    except json.JSONDecodeError:
        continue
    if msg.get("reason") != "compiler-message":
        continue
    m = msg.get("message", {})
    if m.get("level") != "error":
        continue
    for span in m.get("spans", []):
        if not span.get("is_primary"):
            continue
        file_name = span.get("file_name", "")
        line_start = span.get("line_start")
        line_end = span.get("line_end")
        col_start = span.get("column_start")
        col_end = span.get("column_end")
        text_lines = span.get("text", [])
        text = text_lines[0].get("text", "") if text_lines else ""
        errs.append({
            "file": file_name,
            "line": line_start,
            "col": col_start,
            "text": text.strip(),
        })

# 按 file:line 排序去重
seen = set()
uniq = []
for e in errs:
    key = (e["file"], e["line"])
    if key in seen:
        continue
    seen.add(key)
    uniq.append(e)

# 输出简短列表
for e in uniq:
    print(f"{e['file']}:{e['line']}:{e['col']}: {e['text'][:80]}")
print(f"\nTOTAL: {len(uniq)} unique errs")
