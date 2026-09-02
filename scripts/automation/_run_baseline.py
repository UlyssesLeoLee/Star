#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""7 步守门基线全跑 (per docs/automation-design.md §5.2)"""

import subprocess
import sys
from pathlib import Path

WT = Path(r"D:\Star")

def run(label, args, **kwargs):
    sys.stdout.buffer.write(f"=== {label} ===\n".encode('utf-8'))
    r = subprocess.run(args, capture_output=True, cwd=str(WT), **kwargs)
    out = r.stdout.decode('utf-8', errors='replace').strip()
    if r.stderr:
        out += "\n  stderr: " + r.stderr.decode('utf-8', errors='replace').strip()[:300]
    sys.stdout.buffer.write((out[-1000:] + "\n").encode('utf-8', errors='replace'))
    sys.stdout.buffer.write(f"  exit_code: {r.returncode}\n\n".encode('utf-8'))
    sys.stdout.buffer.flush()
    return r.returncode


ec1 = run("守门 #1 v1: cargo check --workspace --lib", ["cargo", "check", "--workspace", "--lib"], timeout=120)
ec2 = run("守门 #1 fmt check", ["cargo", "fmt", "--check"], timeout=60)

ec3 = run("守门 #19+#20+#21: smoke_test", ["python", "scripts/automation/smoke_test.py"], timeout=60)
ec4 = run("守门 #19+#20+#21: registry_check", ["python", "scripts/automation/registry_check.py"], timeout=60)

print("=== 守门 #19: 5 套 unittest (B.5/B.6/C.6/F.6/H2-1) ===")
unittest_results = []
for t in ["integration_e2e_test", "saga_e2e_test", "git_push_test", "h2_refactor_test"]:
    r = subprocess.run(
        ["python", f"scripts/automation/__tests__/{t}"],
        capture_output=True, cwd=str(WT), timeout=120,
    )
    out = r.stdout.decode('utf-8', errors='replace').strip()
    last_3 = "\n".join(out.splitlines()[-5:])
    sys.stdout.buffer.write(f"  --- {t} ---\n".encode('utf-8'))
    sys.stdout.buffer.write(f"  {last_3}\n".encode('utf-8', errors='replace'))
    sys.stdout.buffer.write(f"  exit_code: {r.returncode}\n\n".encode('utf-8'))
    sys.stdout.buffer.flush()
    unittest_results.append((t, r.returncode))

sys.stdout.buffer.write("=== 总结 ===\n".encode('utf-8'))
sys.stdout.buffer.write(f"  cargo check --workspace --lib: exit={ec1}\n".encode('utf-8'))
sys.stdout.buffer.write(f"  cargo fmt --check: exit={ec2}\n".encode('utf-8'))
sys.stdout.buffer.write(f"  smoke_test: exit={ec3}\n".encode('utf-8'))
sys.stdout.buffer.write(f"  registry_check: exit={ec4}\n".encode('utf-8'))
for t, ec in unittest_results:
    sys.stdout.buffer.write(f"  {t}: exit={ec}\n".encode('utf-8'))

all_ec = [ec1, ec2, ec3, ec4] + [ec for _, ec in unittest_results]
nonzero = [ec for ec in all_ec if ec != 0]
if nonzero:
    sys.stdout.buffer.write(f"\n  守门基线 FAIL: {len(nonzero)} 个 step 失败 (exit code {nonzero})\n".encode('utf-8'))
    sys.exit(1)
else:
    sys.stdout.buffer.write("\n  守门基线 PASS: 8 步全过\n".encode('utf-8'))
    sys.exit(0)
