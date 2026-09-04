#!/usr/bin/env python3
"""Show full error message (rendered) for a file/line."""
import json
import subprocess
import sys

filt = sys.argv[1] if len(sys.argv) > 1 else "domain-integration"
proc = subprocess.run(
    ["cargo", "check", "--workspace", "--all-targets", "-j", "4", "--message-format=json"],
    capture_output=True, text=True, encoding="utf-8",
)
shown = 0
for line in proc.stdout.splitlines():
    line = line.strip()
    if not line.startswith("{"):
        continue
    try: obj = json.loads(line)
    except: continue
    if obj.get("reason") != "compiler-message": continue
    m = obj.get("message", {})
    if m.get("level") != "error": continue
    spans = m.get("spans", [])
    if not spans: continue
    sp = spans[0]
    fn = sp.get("file_name", "")
    if filt not in fn:
        continue
    print(f"\n--- {fn}:L{sp.get('line_start')}:C{sp.get('column_start')} ---")
    print(f"MSG: {m.get('message', '')[:400]}")
    for ch in m.get("children", [])[:3]:
        print(f"  HINT: {ch.get('message','')[:200]}")
    shown += 1
    if shown >= 20:
        break
print(f"\n[shown {shown} errs from {filt}]")
