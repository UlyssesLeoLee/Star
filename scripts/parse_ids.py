#!/usr/bin/env python3
"""Parse frontend/src/types/ids.ts and count type members."""
import re
import sys

path = sys.argv[1]
with open(path, "r", encoding="utf-8") as f:
    text = f.read()

# Find all `export type X =` ... unions
pattern = re.compile(r"^export type (\w+)\s*=\s*\n((?:\s*\|[^\n]*\n)+)", re.MULTILINE)
print("=== Multi-line type alias union declarations ===")
for m in pattern.finditer(text):
    name = m.group(1)
    body = m.group(2)
    members = [line.strip() for line in body.split("\n") if line.strip().startswith("|")]
    count = len(members)
    members_clean = [m.lstrip("|").strip().rstrip(";") for m in members]
    print(f"  {name}: {count} members")
    for mc in members_clean:
        print(f"    - {mc}")
