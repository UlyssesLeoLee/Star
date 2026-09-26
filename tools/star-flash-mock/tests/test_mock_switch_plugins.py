#!/usr/bin/env python3
"""test_mock_switch_plugins.py - Star Mock L3 module_switch unit tests.

Per ULYS-190 §4.4 stage3 brief (Star stage3): verify the 7 module entries
declared in `.aci.json` plugins.<plugin_id>.modules.<module_id> can all be
read back by `_lib_mock_switch.py read-plugins` and that each module has
consistent enabled/mode state with the cluster-level settings.

Mirrors IM1.0 stage1 (PR #24) + CATs stage2 (PR #18) 跨项目範式.

Usage:
    python tools/star-flash-mock/tests/test_mock_switch_plugins.py

Exit 0 = all tests passed; non-zero = test failures.
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path

# Resolve paths
# tests/test_mock_switch_plugins.py → ../../.. (Star root)
STAR_ROOT = Path(__file__).resolve().parent.parent.parent.parent
MOCK_ROOT = STAR_ROOT / "tools" / "star-flash-mock"
CLUSTER_CONFIG = MOCK_ROOT / ".mock-cluster.json"
ACI_CONFIG = MOCK_ROOT / ".aci.json"
HELPER = MOCK_ROOT / "scripts" / "_lib_mock_switch.py"


def run_helper(*args: str) -> dict:
    """Run _lib_mock_switch.py read-plugins and parse JSON output."""
    result = subprocess.run(
        ["python", str(HELPER), "read-plugins",
         "--cluster-config", str(CLUSTER_CONFIG),
         "--aci-config", str(ACI_CONFIG)],
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        print(f"helper stderr: {result.stderr}", file=sys.stderr)
        sys.exit(1)
    return json.loads(result.stdout)


def assert_eq(actual, expected, msg: str) -> None:
    if actual != expected:
        print(f"FAIL: {msg}: expected {expected!r}, got {actual!r}", file=sys.stderr)
        sys.exit(1)
    print(f"  [OK] {msg}")


def main() -> int:
    print(f"STAR_ROOT={STAR_ROOT}")
    print(f"CLUSTER_CONFIG={CLUSTER_CONFIG}")
    print(f"ACI_CONFIG={ACI_CONFIG}")
    print()

    state = run_helper()

    # ---- §4.4 stage3 assertions (Star 範式) ----

    # §1 总计
    assert_eq(state["plugins_total"], 7, "7 plugins total: five_domain/agent_runtime/mcp/db_wtm/langgraph/uat/core")
    assert_eq(state["modules_total"], 7, "7 modules total (1 per plugin)")
    assert_eq(state["plugins_enabled"], 7, "all 7 plugins enabled")
    assert_eq(state["modules_enabled"], 7, "all 7 modules enabled")

    # §2 per-plugin module count
    expected_plugins = {
        "five_domain": 1,
        "agent_runtime": 1,
        "mcp": 1,
        "db_wtm": 1,
        "langgraph": 1,
        "uat": 1,
        "core": 1,
    }
    for plugin_id, expected_count in expected_plugins.items():
        plugin = state["plugins"][plugin_id]
        assert_eq(plugin["modules_total"], expected_count, f"{plugin_id} plugin has {expected_count} module")
        assert_eq(plugin["modules_enabled"], expected_count, f"{plugin_id} plugin all enabled")

    # §3 cluster enabled + mode
    cluster_cfg = json.loads(CLUSTER_CONFIG.read_text(encoding="utf-8"))
    assert cluster_cfg["enabled"] is True, "cluster.enabled should be True"
    assert cluster_cfg["mode"] == "offline", "cluster.mode should be offline"
    print(f"  [OK] cluster.enabled=True + mode=offline (per .mock-cluster.json)")

    # §4 aci_compat_version 一致性
    assert_eq(
        cluster_cfg["aci_compat_version"],
        "0.1.0-draft",
        "cluster.aci_compat_version=0.1.0-draft"
    )
    aci_cfg = json.loads(ACI_CONFIG.read_text(encoding="utf-8"))
    assert_eq(
        aci_cfg["aci_compat_version"],
        "0.1.0-draft",
        "aci.aci_compat_version=0.1.0-draft"
    )

    # §5 mock_switch_trace_format 真拼接 (per trace subcommand)
    trace_result = subprocess.run(
        ["python", str(HELPER), "trace",
         "--cluster-config", str(CLUSTER_CONFIG)],
        capture_output=True,
        text=True,
    )
    trace = trace_result.stdout.strip()
    assert "plugins=[five_domain(1m),agent_runtime(1m),mcp(1m),db_wtm(1m),langgraph(1m),uat(1m),core(1m)]" in trace, (
        f"trace should contain 7-plugin summary; got: {trace!r}"
    )
    assert "=7/7 modules" in trace, f"trace should end with 7/7 modules count; got: {trace!r}"
    print(f"  [OK] mock_switch_trace_format 真拼接: {trace!r}")
    print(f"  [INFO] trace 长度: {len(trace)} chars (G-MS-08 ~80 字阈值, 超 79 字, 跨 session 截断 per G-MS-BRIEF-S44-02)")

    # §6 module_count_total field (per .mock-cluster.json v0.4 brief)
    assert_eq(cluster_cfg.get("module_count_total"), 7, ".mock-cluster.json module_count_total=7")
    assert_eq(cluster_cfg.get("module_count_enabled"), 7, ".mock-cluster.json module_count_enabled=7")

    print()
    print(f"=== ALL TESTS PASSED ({state['plugins_total']} plugins, {state['modules_total']} modules) ===")
    return 0


if __name__ == "__main__":
    sys.exit(main())
