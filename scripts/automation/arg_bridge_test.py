#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
scripts/automation/arg_bridge_test.py - ARG.2 (P3-C W2) Bridge Tier IT
(per docs/briefs/arg-02-arg-bridge-crate.md + DD-AGENT-RELATIONSHIP-001 §10.1.2)

守门 #1 v19 (P 子项 Python 化, [M] 子项 `arg_bridge_test.py`) + 守门 #5
(env 安全, Memgraph 连接串走 env) + 守门 #9 v3 (subprocess 路径, mock
MemGraphClient) + 守门 #14 v2 (5 域 Lead Mavis 临时代签) + 守门 #21 v21
(Mavis 自驱, P 子项 docs 同步):

10 IT (per brief §2.1 C + DD §10.1.2):
  1. test_memgraph_listener_connect        - Bolt subscription 建立
  2. test_memgraph_listener_event          - edge.changed event 解析
  3. test_memgraph_listener_reconnect      - 断线重连
  4. test_period_flush_30s                 - 周期触发 (configurable period)
  5. test_period_flush_batch               - 批量写
  6. test_offline_queue_write              - 不可达时缓存
  7. test_offline_queue_read               - 重连后 flush
  8. test_offline_queue_full               - 满 10000 拒绝
  9. test_langgraph_state_update           - 5 Reducer dispatch
 10. test_langgraph_state_5_reducer        - 5 Reducer 语义合并

走 subprocess.run 调 `cargo test -p star-arg-bridge --tests -j 4` 端到端
验证 (per 守门 #9 v3 subprocess 隔离, 不派 worker 子代理).

用法:
    # 默认: 跑 `cargo test -p star-arg-bridge --tests -j 4` 然后断言
    python scripts/automation/arg_bridge_test.py

    # 跳过 cargo test, 只跑 Python 端轻量验证
    python scripts/automation/arg_bridge_test.py --skip-cargo
"""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import List, Tuple

ROOT = Path(__file__).resolve().parent.parent.parent

# 10 IT 名 (per brief §2.1 C + DD §10.1.2 UT-25..UT-34)
IT_NAMES = [
    "ut25_memgraph_listener_connect",
    "ut26_memgraph_listener_event",
    "ut27_memgraph_listener_reconnect",
    "ut28_period_flush_30s_default_period",
    "ut29_period_flush_batch_persists_and_dedups",
    "ut30_offline_queue_write",
    "ut31_offline_queue_read_after_reopen",
    "ut32_offline_queue_full_rejects",
    "ut33_langgraph_state_update_5_reducer_dispatch",
    "ut34_langgraph_5_reducer_semantics",
]

# 额外 sanity test (在 langgraph_test.rs / flush_test.rs 里的 extras),
# 仍算 10 main UT 之上的 5 个, 用于强化 contract 测试.
EXTRA_TEST_NAMES = [
    "ut29b_period_flush_idempotency",
    "ut32b_offline_queue_prune_expired",
    "ut34b_langgraph_dispatch_route_envelope",
]


def _check(name: str, ok: bool, detail: str = "") -> bool:
    sym = "PASS" if ok else "FAIL"
    print(f"  [{sym}] {name}: {detail}")
    return ok


def _parse_cargo_test_output(stdout: str) -> Tuple[List[str], List[str]]:
    """从 `cargo test` 输出解析 (passed, failed) 列表.

    每行 `test <name> ... ok` 或 `test <name> ... FAILED`.
    """
    passed: List[str] = []
    failed: List[str] = []
    for line in stdout.splitlines():
        m = re.match(r"^test\s+(\w+)\s+\.\.\.\s+(ok|FAILED|ignored)$", line.strip())
        if not m:
            continue
        name, status = m.group(1), m.group(2)
        if status == "ok":
            passed.append(name)
        elif status == "FAILED":
            failed.append(name)
    return passed, failed


# =====================================================================
# 1) Python-side: 跑 cargo test 端到端 (守门 #1 v25 实证)
# =====================================================================

def test_cargo_test_all_10_it(ctx: dict) -> bool:
    """IT-1..10: 调 `cargo test -p star-arg-bridge --tests -j 4`,
    解析结果, 断言 10 UT 全部 PASS (per brief §AC-2)."""
    print("[IT-1..10] cargo test -p star-arg-bridge --tests -j 4")
    proc = subprocess.run(
        ["cargo", "test", "-p", "star-arg-bridge", "--tests", "-j", "4", "--no-fail-fast"],
        cwd=str(ROOT),
        capture_output=True,
        text=True,
        env=os.environ.copy(),
    )
    if proc.returncode != 0:
        # 输出诊断但不 abort
        print(f"  cargo test exit={proc.returncode}", file=sys.stderr)
        print(f"  stdout={proc.stdout[-1500:]}", file=sys.stderr)
        print(f"  stderr={proc.stderr[-1500:]}", file=sys.stderr)
        return _check("test_cargo_test_all_10_it", False, f"exit={proc.returncode}")
    passed, failed = _parse_cargo_test_output(proc.stdout)
    ok = (
        len(failed) == 0
        and all(name in passed for name in IT_NAMES)
    )
    detail = (
        f"passed={len(passed)} failed={len(failed)} "
        f"(10 main UT: {sum(1 for n in IT_NAMES if n in passed)}/10, "
        f"3 extras: {sum(1 for n in EXTRA_TEST_NAMES if n in passed)}/3)"
    )
    return _check("test_cargo_test_all_10_it", ok, detail)


def test_cargo_test_lib_subcrate_only(ctx: dict) -> bool:
    """IT-11: 单 crate 模式实证 (per 守门 #1 v25)."""
    print("[IT-11] cargo test -p star-arg-bridge --lib -j 4")
    proc = subprocess.run(
        ["cargo", "test", "-p", "star-arg-bridge", "--lib", "-j", "4"],
        cwd=str(ROOT),
        capture_output=True,
        text=True,
        env=os.environ.copy(),
    )
    return _check(
        "test_cargo_test_lib_subcrate_only",
        proc.returncode == 0,
        f"exit={proc.returncode}",
    )


def test_cargo_check_workspace_lib(ctx: dict) -> bool:
    """IT-12: workspace 兼容 (per 守门 #1 v25)."""
    print("[IT-12] cargo check --workspace --lib -j 4")
    proc = subprocess.run(
        ["cargo", "check", "--workspace", "--lib", "-j", "4"],
        cwd=str(ROOT),
        capture_output=True,
        text=True,
        env=os.environ.copy(),
    )
    return _check(
        "test_cargo_check_workspace_lib",
        proc.returncode == 0,
        f"exit={proc.returncode}",
    )


def test_cargo_fmt_arg_bridge_clean(ctx: dict) -> bool:
    """IT-13: arg-bridge crate fmt 0 diff (per 守门 #1 v2)."""
    print("[IT-13] cargo fmt -p star-arg-bridge -- --check")
    proc = subprocess.run(
        ["cargo", "fmt", "-p", "star-arg-bridge", "--", "--check"],
        cwd=str(ROOT),
        capture_output=True,
        text=True,
        env=os.environ.copy(),
    )
    return _check(
        "test_cargo_fmt_arg_bridge_clean",
        proc.returncode == 0,
        f"exit={proc.returncode}",
    )


def test_cargo_clippy_arg_bridge_clean(ctx: dict) -> bool:
    """IT-14: clippy 0 err (per 守门 #7 + #1 v2)."""
    print("[IT-14] cargo clippy -p star-arg-bridge --lib -j 4 -- -D warnings")
    proc = subprocess.run(
        [
            "cargo",
            "clippy",
            "-p",
            "star-arg-bridge",
            "--lib",
            "-j",
            "4",
            "--",
            "-D",
            "warnings",
        ],
        cwd=str(ROOT),
        capture_output=True,
        text=True,
        env=os.environ.copy(),
    )
    return _check(
        "test_cargo_clippy_arg_bridge_clean",
        proc.returncode == 0,
        f"exit={proc.returncode}",
    )


def test_cargo_build_release_arg_bridge(ctx: dict) -> bool:
    """IT-15: release build 0 err (per brief §AC-1)."""
    print("[IT-15] cargo build --release -p star-arg-bridge")
    proc = subprocess.run(
        ["cargo", "build", "--release", "-p", "star-arg-bridge"],
        cwd=str(ROOT),
        capture_output=True,
        text=True,
        env=os.environ.copy(),
    )
    return _check(
        "test_cargo_build_release_arg_bridge",
        proc.returncode == 0,
        f"exit={proc.returncode}",
    )


def test_no_unsafe_blocks(ctx: dict) -> bool:
    """IT-16: 0 unsafe 块 (per 守门 #7 `unsafe_code = "forbid"`)."""
    print("[IT-16] grep -r 'unsafe ' crates/arg-bridge/src (must be empty)")
    src_dir = ROOT / "crates" / "arg-bridge" / "src"
    unsafe_hits: List[str] = []
    for rs in src_dir.rglob("*.rs"):
        text = rs.read_text(encoding="utf-8")
        # 找 `unsafe {` 或 `unsafe fn` 模式 (非注释)
        for line in text.splitlines():
            stripped = line.strip()
            if stripped.startswith("//"):
                continue
            if re.search(r"\bunsafe\b", line) and "{" in line and "unsafe {" in line:
                unsafe_hits.append(f"{rs.name}:{line.strip()}")
            elif re.search(r"\bunsafe\s+fn\b", line):
                unsafe_hits.append(f"{rs.name}:{line.strip()}")
    return _check(
        "test_no_unsafe_blocks",
        len(unsafe_hits) == 0,
        f"hits={len(unsafe_hits)} " + (unsafe_hits[0] if unsafe_hits else ""),
    )


def test_workspace_registration(ctx: dict) -> bool:
    """IT-17: workspace Cargo.toml 含 'crates/arg-bridge' (per brief §AC-3)."""
    print("[IT-17] grep '\"crates/arg-bridge\"' Cargo.toml")
    cargo_toml = ROOT / "Cargo.toml"
    text = cargo_toml.read_text(encoding="utf-8")
    ok = '"crates/arg-bridge"' in text
    return _check(
        "test_workspace_registration",
        ok,
        f"present={ok}",
    )


def test_sled_workspace_dep(ctx: dict) -> bool:
    """IT-18: workspace Cargo.toml 含 sled dep (per brief §2.1 A)."""
    print("[IT-18] grep '^sled =' Cargo.toml")
    cargo_toml = ROOT / "Cargo.toml"
    text = cargo_toml.read_text(encoding="utf-8")
    ok = re.search(r'^sled\s*=\s*"', text, re.MULTILINE) is not None
    return _check(
        "test_sled_workspace_dep",
        ok,
        f"present={ok}",
    )


def test_5_protocols_in_protocol_rs(ctx: dict) -> bool:
    """IT-19: 5 内部协议 schema 落地 (per arch §2.1)."""
    print("[IT-19] grep 'pub struct Arg' crates/arg-bridge/src/protocol.rs")
    proto = ROOT / "crates" / "arg-bridge" / "src" / "protocol.rs"
    text = proto.read_text(encoding="utf-8")
    required = [
        "ArgEdgeChanged",
        "ArgDispatchRoute",
        "ArgContextInject",
        "ArgTrustScoreUpdate",
        "ArgAchievementUnlocked",
    ]
    missing = [name for name in required if f"pub struct {name}" not in text]
    return _check(
        "test_5_protocols_in_protocol_rs",
        len(missing) == 0,
        f"missing={missing}" if missing else "5/5 present",
    )


def test_4_submodules_lib_rs(ctx: dict) -> bool:
    """IT-20: 4 子模块 + protocol/error 在 lib.rs 暴露 (per brief §2.1 A)."""
    print("[IT-20] grep 'pub mod' crates/arg-bridge/src/lib.rs")
    lib_rs = ROOT / "crates" / "arg-bridge" / "src" / "lib.rs"
    text = lib_rs.read_text(encoding="utf-8")
    required = [
        "pub mod protocol;",
        "pub mod error;",
        "pub mod memgraph_listener;",
        "pub mod langgraph_updater;",
        "pub mod period_flush;",
        "pub mod offline_queue;",
    ]
    missing = [name for name in required if name not in text]
    return _check(
        "test_4_submodules_lib_rs",
        len(missing) == 0,
        f"missing={missing}" if missing else "6/6 present",
    )


# =====================================================================
# 2) Main: 跑全部 IT, 失败抛 AssertionError
# =====================================================================

def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--skip-cargo",
        action="store_true",
        help="skip cargo test/check/fmt/clippy/build (only run Python-side checks)",
    )
    parser.add_argument(
        "--only-python",
        action="store_true",
        help="only run Python file-content checks (IT-16..IT-20)",
    )
    args = parser.parse_args()

    print("=" * 60)
    print("ARG.2 (P3-C W2) Bridge Tier IT (per brief v0.1)")
    print("=" * 60)

    ctx: dict = {}
    results: List[bool] = []

    if args.only_python:
        results.append(test_no_unsafe_blocks(ctx))
        results.append(test_workspace_registration(ctx))
        results.append(test_sled_workspace_dep(ctx))
        results.append(test_5_protocols_in_protocol_rs(ctx))
        results.append(test_4_submodules_lib_rs(ctx))
    elif args.skip_cargo:
        results.append(test_no_unsafe_blocks(ctx))
        results.append(test_workspace_registration(ctx))
        results.append(test_sled_workspace_dep(ctx))
        results.append(test_5_protocols_in_protocol_rs(ctx))
        results.append(test_4_submodules_lib_rs(ctx))
    else:
        # 1) cargo test 端到端
        results.append(test_cargo_test_all_10_it(ctx))
        # 2) 单 crate lib 模式
        results.append(test_cargo_test_lib_subcrate_only(ctx))
        # 3) workspace 兼容
        results.append(test_cargo_check_workspace_lib(ctx))
        # 4) fmt
        results.append(test_cargo_fmt_arg_bridge_clean(ctx))
        # 5) clippy
        results.append(test_cargo_clippy_arg_bridge_clean(ctx))
        # 6) build release
        results.append(test_cargo_build_release_arg_bridge(ctx))
        # 7) file-content checks
        results.append(test_no_unsafe_blocks(ctx))
        results.append(test_workspace_registration(ctx))
        results.append(test_sled_workspace_dep(ctx))
        results.append(test_5_protocols_in_protocol_rs(ctx))
        results.append(test_4_submodules_lib_rs(ctx))

    print("=" * 60)
    passed = sum(1 for r in results if r)
    total = len(results)
    print(f"IT summary: {passed}/{total} PASS")
    if passed == total:
        print("ALL GREEN")
        return 0
    print(f"FAIL: {total - passed} case(s) failed")
    return 1


if __name__ == "__main__":
    sys.exit(main())
