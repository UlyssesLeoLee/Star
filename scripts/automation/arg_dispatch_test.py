#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
scripts/automation/arg_dispatch_test.py - ARG.3 (P3-C W3) Effect Tier IT
(per docs/briefs/arg-03-arg-effect-crate.md + DD-AGENT-RELATIONSHIP-001 §10.1.3)

守门 #1 v19 (P 子项 Python 化, [P] 子项 `arg_dispatch_test.py`) + 守门 #5
(env 安全, Memgraph 连接串走 env) + 守门 #9 v3 (subprocess 路径, mock
LLMClient + EdgeOps) + 守门 #14 v2 (5 域 Lead Mavis 临时代签) + 守门 #21 v21
(Mavis 自驱, P 子项 docs 同步):

18 UT (per brief §2.1 A + DD §10.1.3 UT-35..UT-52):
  1. ut35_dispatch_router_delegates
  2. ut36_dispatch_router_stand_in
  3. ut37_dispatch_router_collaborators
  4. ut38_context_inject_mentors
  5. ut39_context_inject_shadows
  6. ut40_context_inject_empty
  7. ut41_trust_engine_record_success
  8. ut42_trust_engine_record_failure
  9. ut43_trust_engine_skip_verify
 10. ut44_trust_engine_no_skip_when_score_low
 11. ut45_output_challenge_round_accept
 12. ut46_output_challenge_reject_escalate
 13. ut47_output_peer_review_above_threshold
 14. ut48_achievement_topology_eval_8_cyphers
 15. ut49_achievement_behavior_eval_placeholder
 16. ut50_achievement_output_eval_placeholder
 17. ut51_achievement_unlock_idempotent
 18. ut52_achievement_3_categories_classification

走 subprocess.run 调 `cargo test -p star-arg-effect --tests -j 4` 端到端
验证 (per 守门 #9 v3 subprocess 隔离, 不派 worker 子代理).

用法:
    # 默认: 跑 `cargo test -p star-arg-effect --tests -j 4` 然后断言
    python scripts/automation/arg_dispatch_test.py

    # 跳过 cargo test, 只跑 Python 端轻量验证
    python scripts/automation/arg_dispatch_test.py --skip-cargo
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path
from typing import List, Tuple

ROOT = Path(__file__).resolve().parent.parent.parent

# 18 UT 名 (per brief §2.1 A + DD §10.1.3 UT-35..UT-52)
IT_NAMES = [
    "ut35_dispatch_router_delegates",
    "ut36_dispatch_router_stand_in",
    "ut37_dispatch_router_collaborators",
    "ut38_context_inject_mentors",
    "ut39_context_inject_shadows",
    "ut40_context_inject_empty",
    "ut41_trust_engine_record_success",
    "ut42_trust_engine_record_failure",
    "ut43_trust_engine_skip_verify",
    "ut44_trust_engine_no_skip_when_score_low",
    "ut45_output_challenge_round_accept",
    "ut46_output_challenge_reject_escalate",
    "ut47_output_peer_review_above_threshold",
    "ut48_achievement_topology_eval_8_cyphers",
    "ut49_achievement_behavior_eval_placeholder",
    "ut50_achievement_output_eval_placeholder",
    "ut51_achievement_unlock_idempotent",
    "ut52_achievement_3_categories_classification",
]

# 3 份额外 sanity test (achievement_engine::topology_codes_match_canonical_8
# + context_injector::inject_returns_base_prompt_when_no_incoming + 28 lib
# 内 unit), 用于强化 contract 测试.
EXTRA_TEST_NAMES = [
    "topology_codes_match_canonical_8",
    "inject_returns_base_prompt_when_no_incoming",
    "all_8_cyphers_avoid_apoc",
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

def test_cargo_test_all_18_it(ctx: dict) -> bool:
    """IT-1..18: 调 `cargo test -p star-arg-effect --tests -j 4`,
    解析结果, 断言 18 UT 全部 PASS (per brief §AC-2)."""
    print("[IT-1..18] cargo test -p star-arg-effect --tests -j 4")
    proc = subprocess.run(
        ["cargo", "test", "-p", "star-arg-effect", "--tests", "-j", "4", "--no-fail-fast"],
        cwd=str(ROOT),
        capture_output=True,
        text=True,
        env=__import__("os").environ.copy(),
    )
    if proc.returncode != 0:
        # 输出诊断但不 abort
        print(f"  cargo test exit={proc.returncode}", file=sys.stderr)
        print(f"  stdout={proc.stdout[-1500:]}", file=sys.stderr)
        print(f"  stderr={proc.stderr[-1500:]}", file=sys.stderr)
        return _check("test_cargo_test_all_18_it", False, f"exit={proc.returncode}")
    passed, failed = _parse_cargo_test_output(proc.stdout)
    ok = (
        len(failed) == 0
        and all(name in passed for name in IT_NAMES)
    )
    detail = (
        f"passed={len(passed)} failed={len(failed)} "
        f"(18 main UT: {sum(1 for n in IT_NAMES if n in passed)}/18, "
        f"3 extras: {sum(1 for n in EXTRA_TEST_NAMES if n in passed)}/3)"
    )
    return _check("test_cargo_test_all_18_it", ok, detail)


def test_cargo_test_lib_subcrate_only(ctx: dict) -> bool:
    """IT-19: 单 crate 模式实证 (per 守门 #1 v25)."""
    print("[IT-19] cargo test -p star-arg-effect --lib -j 4")
    proc = subprocess.run(
        ["cargo", "test", "-p", "star-arg-effect", "--lib", "-j", "4"],
        cwd=str(ROOT),
        capture_output=True,
        text=True,
        env=__import__("os").environ.copy(),
    )
    return _check(
        "test_cargo_test_lib_subcrate_only",
        proc.returncode == 0,
        f"exit={proc.returncode}",
    )


def test_cargo_check_workspace_lib(ctx: dict) -> bool:
    """IT-20: workspace 兼容 (per 守门 #1 v25)."""
    print("[IT-20] cargo check --workspace --lib -j 4")
    proc = subprocess.run(
        ["cargo", "check", "--workspace", "--lib", "-j", "4"],
        cwd=str(ROOT),
        capture_output=True,
        text=True,
        env=__import__("os").environ.copy(),
    )
    return _check(
        "test_cargo_check_workspace_lib",
        proc.returncode == 0,
        f"exit={proc.returncode}",
    )


def test_cargo_fmt_arg_effect_clean(ctx: dict) -> bool:
    """IT-21: arg-effect crate fmt 0 diff (per 守门 #1 v2)."""
    print("[IT-21] cargo fmt -p star-arg-effect -- --check")
    proc = subprocess.run(
        ["cargo", "fmt", "-p", "star-arg-effect", "--", "--check"],
        cwd=str(ROOT),
        capture_output=True,
        text=True,
        env=__import__("os").environ.copy(),
    )
    return _check(
        "test_cargo_fmt_arg_effect_clean",
        proc.returncode == 0,
        f"exit={proc.returncode}",
    )


def test_cargo_clippy_arg_effect_clean(ctx: dict) -> bool:
    """IT-22: clippy 0 err (per 守门 #7 + #1 v2)."""
    print("[IT-22] cargo clippy -p star-arg-effect --lib -j 4 -- -D warnings")
    proc = subprocess.run(
        [
            "cargo",
            "clippy",
            "-p",
            "star-arg-effect",
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
        env=__import__("os").environ.copy(),
    )
    return _check(
        "test_cargo_clippy_arg_effect_clean",
        proc.returncode == 0,
        f"exit={proc.returncode}",
    )


def test_cargo_build_release_arg_effect(ctx: dict) -> bool:
    """IT-23: release build 0 err (per brief §AC-1)."""
    print("[IT-23] cargo build --release -p star-arg-effect")
    proc = subprocess.run(
        ["cargo", "build", "--release", "-p", "star-arg-effect"],
        cwd=str(ROOT),
        capture_output=True,
        text=True,
        env=__import__("os").environ.copy(),
    )
    return _check(
        "test_cargo_build_release_arg_effect",
        proc.returncode == 0,
        f"exit={proc.returncode}",
    )


def test_no_unsafe_blocks(ctx: dict) -> bool:
    """IT-24: 0 unsafe 块 (per 守门 #7 `unsafe_code = "forbid"`)."""
    print("[IT-24] grep -r 'unsafe ' crates/arg-effect/src (must be empty)")
    src_dir = ROOT / "crates" / "arg-effect" / "src"
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
    """IT-25: workspace Cargo.toml 含 'crates/arg-effect' (per brief §AC-3)."""
    print("[IT-25] grep '\"crates/arg-effect\"' Cargo.toml")
    cargo_toml = ROOT / "Cargo.toml"
    text = cargo_toml.read_text(encoding="utf-8")
    ok = '"crates/arg-effect"' in text
    return _check(
        "test_workspace_registration",
        ok,
        f"present={ok}",
    )


def test_arg_workspace_path_dep(ctx: dict) -> bool:
    """IT-26: workspace Cargo.toml 含 'arg-effect' path dep (per brief §2.1 A)."""
    print("[IT-26] grep 'path = \"../arg' crates/arg-effect/Cargo.toml")
    cargo_toml = ROOT / "crates" / "arg-effect" / "Cargo.toml"
    text = cargo_toml.read_text(encoding="utf-8")
    ok = ('path = "../arg"' in text) and ('path = "../arg-bridge"' in text)
    return _check(
        "test_arg_workspace_path_dep",
        ok,
        f"present={ok}",
    )


def test_5_submodules_lib_rs(ctx: dict) -> bool:
    """IT-27: 5 子模块 + error + prompts 在 lib.rs 暴露 (per brief §2.1 A)."""
    print("[IT-27] grep 'pub mod' crates/arg-effect/src/lib.rs")
    lib_rs = ROOT / "crates" / "arg-effect" / "src" / "lib.rs"
    text = lib_rs.read_text(encoding="utf-8")
    required = [
        "pub mod error;",
        "pub mod prompts;",
        "pub mod dispatch_router;",
        "pub mod context_injector;",
        "pub mod trust_engine;",
        "pub mod output_evaluator;",
        "pub mod achievement_engine;",
    ]
    missing = [name for name in required if name not in text]
    return _check(
        "test_5_submodules_lib_rs",
        len(missing) == 0,
        f"missing={missing}" if missing else "7/7 present",
    )


def test_8_topology_cyphers_no_apoc(ctx: dict) -> bool:
    """IT-28: 8 拓扑成就 Cypher 模板全, 不含 apoc (per DD §6 + F-11)."""
    print("[IT-28] grep 8 TOP-001..TOP-008 in crates/arg/src/query/topology.rs (no apoc)")
    topo = ROOT / "crates" / "arg" / "src" / "query" / "topology.rs"
    text = topo.read_text(encoding="utf-8")
    required = [
        "TOP-001-MESH-5DOMAIN",
        "TOP-002-HUB-AND-SPOKE",
        "TOP-003-MESH-10",
        "TOP-004-CHAIN-5",
        "TOP-005-HIERARCHICAL-3",
        "TOP-006-REVIEW-COUNCIL",
        "TOP-007-NO-SELF-LOOP",
        "TOP-008-NO-ISLAND",
    ]
    missing = [c for c in required if c not in text]
    has_apoc = "apoc." in text
    ok = len(missing) == 0 and not has_apoc
    return _check(
        "test_8_topology_cyphers_no_apoc",
        ok,
        f"missing={missing}" if missing else f"apoc_present={has_apoc}",
    )


def test_10_challenge_prompts(ctx: dict) -> bool:
    """IT-29: 10 套 challenges prompt 模板 (5 decision × 2 tier) (per DD §7)."""
    print("[IT-29] grep 5 DecisionType + 2 TrustTier = 10 in prompts.rs")
    prompts = ROOT / "crates" / "arg-effect" / "src" / "prompts.rs"
    text = prompts.read_text(encoding="utf-8")
    required_decisions = [
        "Architectural",
        "Business",
        "Security",
        "Performance",
        "Ux",
    ]
    required_tiers = ["TrustTier::Low", "TrustTier::High"]
    missing = []
    for d in required_decisions:
        if f"DecisionType::{d}" not in text:
            missing.append(f"DecisionType::{d}")
    for t in required_tiers:
        if t not in text:
            missing.append(t)
    return _check(
        "test_10_challenge_prompts",
        len(missing) == 0,
        f"missing={missing}" if missing else "5×2 = 10 present",
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
        help="only run Python file-content checks (IT-24..IT-29)",
    )
    args = parser.parse_args()

    print("=" * 60)
    print("ARG.3 (P3-C W3) Effect Tier IT (per brief v0.1)")
    print("=" * 60)

    ctx: dict = {}
    results: List[bool] = []

    if args.only_python:
        results.append(test_no_unsafe_blocks(ctx))
        results.append(test_workspace_registration(ctx))
        results.append(test_arg_workspace_path_dep(ctx))
        results.append(test_5_submodules_lib_rs(ctx))
        results.append(test_8_topology_cyphers_no_apoc(ctx))
        results.append(test_10_challenge_prompts(ctx))
    elif args.skip_cargo:
        results.append(test_no_unsafe_blocks(ctx))
        results.append(test_workspace_registration(ctx))
        results.append(test_arg_workspace_path_dep(ctx))
        results.append(test_5_submodules_lib_rs(ctx))
        results.append(test_8_topology_cyphers_no_apoc(ctx))
        results.append(test_10_challenge_prompts(ctx))
    else:
        # 1) cargo test 端到端
        results.append(test_cargo_test_all_18_it(ctx))
        # 2) 单 crate lib 模式
        results.append(test_cargo_test_lib_subcrate_only(ctx))
        # 3) workspace 兼容
        results.append(test_cargo_check_workspace_lib(ctx))
        # 4) fmt
        results.append(test_cargo_fmt_arg_effect_clean(ctx))
        # 5) clippy
        results.append(test_cargo_clippy_arg_effect_clean(ctx))
        # 6) build release
        results.append(test_cargo_build_release_arg_effect(ctx))
        # 7) file-content checks
        results.append(test_no_unsafe_blocks(ctx))
        results.append(test_workspace_registration(ctx))
        results.append(test_arg_workspace_path_dep(ctx))
        results.append(test_5_submodules_lib_rs(ctx))
        results.append(test_8_topology_cyphers_no_apoc(ctx))
        results.append(test_10_challenge_prompts(ctx))

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
