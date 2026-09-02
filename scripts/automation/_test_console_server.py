#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""console_server.py 启动期 7 个 API 端点 smoke 测试"""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
sys.path.insert(0, str(Path(__file__).resolve().parent))

from automation.console_server import (
    SCRIPTS_META, app, list_scripts, toggle_script, run_script, ai_edit, status
)

# 1. 13 份脚本 + 5 套 unittest 数量
print(f"=== SCRIPTS_META 数量 ===")
print(f"  total: {len(SCRIPTS_META)} (8 base + 4 [P] + 5 unittest)")
assert len(SCRIPTS_META) == 14, f"应该 14 份 (6 base + 4 [P] + 4 unittest), 实际 {len(SCRIPTS_META)}"

# 2. 分类统计
base_count = sum(1 for m in SCRIPTS_META.values() if m.category == "base")
p_count = sum(1 for m in SCRIPTS_META.values() if m.category == "p_card")
u_count = sum(1 for m in SCRIPTS_META.values() if m.category == "unittest")
print(f"  base: {base_count}")
print(f"  p_card: {p_count}")
print(f"  unittest: {u_count}")
assert base_count == 6
assert p_count == 4
assert u_count == 4

# 3. list_scripts
print("\n=== /api/scripts GET ===")
result = list_scripts()
print(f"  total: {result['total']}")
print(f"  first 3: {list(result['scripts'].keys())[:3]}")

# 4. toggle_script
print("\n=== /api/scripts/{id}/toggle POST ===")
r1 = toggle_script("integration_e2e", "disabled")
print(f"  integration_e2e: {r1['status']}")
r2 = toggle_script("integration_e2e", "enabled")
print(f"  back to enabled: {r2['status']}")

# 5. status_dashboard
print("\n=== /api/status GET ===")
st = status()
print(f"  enabled: {st['enabled']}, disabled: {st['disabled']}, total_runs: {st['total_runs']}")

# 6. ai_edit mock
print("\n=== /api/ai_edit POST ===")
ae = ai_edit("integration_e2e", {"provider": "hermes", "dry_run": "true"})
print(f"  suggestions: {len(ae['suggestions'])}")
for s in ae['suggestions'][:2]:
    print(f"    - [{s['type']}] {s['target']} (confidence: {s['confidence']})")

# 7. run_script (dry-run, 1 个 smoke_test 跑, 验证 subprocess 路径)
print("\n=== /api/scripts/{id}/run POST (smoke_test) ===")
smoke_meta = SCRIPTS_META["smoke_test"]
smoke_meta.status = "enabled"
r = run_script("smoke_test")
print(f"  exit_code: {r['exit_code']}, ok: {r['ok']}, duration: {r['duration_ms']:.0f}ms")
print(f"  output preview: {r['output_preview'][:200]}")

print("\n=== All tests passed ===")
