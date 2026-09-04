#!/usr/bin/env python3
"""List remaining compile errors with file:line:col + message.

Usage: python list_err_lines.py [crate_filter]
"""
import json
import re
import subprocess
import sys

filter_re = re.compile(sys.argv[1] if len(sys.argv) > 1 else r"^.*$")

proc = subprocess.run(
    [
        "cargo", "check", "--workspace", "--all-targets", "-j", "4",
        "--message-format=json",
    ],
    capture_output=True,
    text=True,
    encoding="utf-8",
)

errs = []
for line in proc.stdout.splitlines():
    line = line.strip()
    if not line.startswith("{"):
        continue
    try:
        obj = json.loads(line)
    except json.JSONDecodeError:
        continue
    if obj.get("reason") != "compiler-message":
        continue
    msg = obj.get("message") or {}
    if msg.get("level") != "error":
        continue
    spans = msg.get("spans") or []
    if not spans:
        continue
    sp = spans[0]
    fn = sp.get("file_name", "")
    if not filter_re.search(fn):
        continue
    errs.append((fn, sp.get("line_start"), sp.get("column_start"), msg.get("message", "")))

# group by file
by_file = {}
for fn, line, col, msg in errs:
    by_file.setdefault(fn, []).append((line, col, msg))

print(f"Total errors: {len(errs)} across {len(by_file)} files")
for fn, items in sorted(by_file.items()):
    print(f"\n{fn}  ({len(items)})")
    for line, col, msg in items[:20]:
        snippet = msg.split("\n")[0][:200]
        print(f"  L{line:>5}:{col:>3}  {snippet}")
    if len(items) > 20:
        print(f"  ... and {len(items)-20} more")
