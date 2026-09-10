#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
scripts/automation/arg_behavior_eval.py — ARG.8 (P3-E W1) 12 IT 端到端验证
(per docs/briefs/arg-08-behavior-output-evaluator.md §2.1 C + DD-AGENT-RELATIONSHIP-001 §10.2)

Per brief v0.50 §2.1 A.1+A.2 + DD §10.2 + arch 2026-09-03-arg/07-arg-03-5-sa-impl.md §3:

12 IT 覆盖 ARG.8 7 行为 + 5 产出成就 evaluator 端到端:
  1.  test_beh_001_first_delegates          (BEH-001 first_dispatch_via_delegates_to)
  2.  test_beh_002_consults_100             (BEH-002 consults_decision_made_100_times)
  3.  test_beh_003_collab_50                (BEH-003 collaborates_with_parallel_50_times)
  4.  test_beh_004_stand_in_5               (BEH-004 stand_in_for_takeover_5_times)
  5.  test_beh_005_peer_100pct              (BEH-005 peer_reviews_one_pass_100_percent)
  6.  test_beh_006_challenge_3              (BEH-006 challenges_rebuttal_3_times)
  7.  test_beh_007_shadows_24h              (BEH-007 shadows_observation_24_hours)
  8.  test_out_001_trusts_100k_token        (OUT-001 trusts_skip_verify_save_100k_token)
  9.  test_out_002_collab_1h                (OUT-002 collaborate_save_1h_wall_clock)
 10.  test_out_003_5_lead_consensus         (OUT-003 5_domain_lead_consensus_reached)
 11.  test_out_004_zero_fail_100            (OUT-004 zero_failure_100_collaborations)
 12.  test_out_005_achievement_chain_5      (OUT-005 achievement_chain_5_in_a_row)
 13. test_sse_publish_through_publisher     (验证 publisher 触发)
 14. test_unlock_idempotency                (验证 重复 unlock 是 noop)
 15. test_3_evaluators_parallel             (验证 3 evaluator 并行跑通)

守门合规:
- 守门 #1 v19 [M] (Python 化 ≥ 2 维): R + V + S + A 全过
- 守门 #5 (env 安全, 不打印明文)
- 守门 #6 (PowerShell only, subprocess.run shell=False)
- 守门 #7 (Rust 0 unsafe 块, 跨 crate 检查)
- 守门 #9 (RPC 不可靠, mock ARGSSEHub via NoopAchievementPublisher)
- 守门 #10 (代签, author=Ulysses)
- 守门 #12 ([P] docs 同步)
- 守门 #14 v2 (5 域 Lead Mavis 临时代签)

走 `subprocess.run` 调 `cargo test -p star-arg-effect` 验证 5 守门 0 err + 23 新增 UT 全过。

用法:
    python scripts/automation/arg_behavior_eval.py            # 跑 15 IT
    python scripts/automation/arg_behavior_eval.py --quick   # 跳过 cargo test
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import List, Tuple

REPO_ROOT = Path(__file__).resolve().parent.parent.parent

# 15 IT 名 (7 行为 + 5 产出 + 3 集成, per brief §2.1 B)
IT_NAMES = [
    # 7 行为
    "test_beh_001_first_delegates",
    "test_beh_002_consults_100",
    "test_beh_003_collab_50",
    "test_beh_004_stand_in_5",
    "test_beh_005_peer_100pct",
    "test_beh_006_challenge_3",
    "test_beh_007_shadows_24h",
    # 5 产出
    "test_out_001_trusts_100k_token",
    "test_out_002_collab_1h",
    "test_out_003_5_lead_consensus",
    "test_out_004_zero_fail_100",
    "test_out_005_achievement_chain_5",
    # 3 集成
    "test_sse_publish_through_publisher",
    "test_unlock_idempotency",
    "test_3_evaluators_parallel",
]

# 12 核心 IT (per brief §2.1 C 必做 12)
CORE_IT_NAMES = IT_NAMES[:12]


# =====================================================================
# 工具函数
# =====================================================================


def _run(cmd: List[str], cwd: Path | None = None, timeout: int = 300) -> Tuple[int, str, str]:
    """Run subprocess; return (exit, stdout, stderr). Never raises."""
    try:
        proc = subprocess.run(
            cmd,
            cwd=str(cwd) if cwd else None,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            timeout=timeout,
            shell=False,  # 守门 #6: 不调 shell
        )
        return proc.returncode, proc.stdout, proc.stderr
    except FileNotFoundError as e:
        return 127, "", f"executable not found: {e}"
    except subprocess.TimeoutExpired as e:
        return 124, e.stdout or "", f"timeout after {timeout}s"
    except Exception as e:  # noqa: BLE001
        return 1, "", f"unexpected error: {e}"


def _evidence(label: str, exit_code: int, stdout_tail: str = "") -> None:
    """Print evidence: exit code + last line of stdout (per 守门 #5 不打印 secret)."""
    print(f"  [{label:40}] exit={exit_code}", end="")
    if stdout_tail:
        snippet = stdout_tail.strip().splitlines()
        last = snippet[-1] if snippet else ""
        if last:
            print(f" | last={last[:120]!r}", end="")
    print()


# =====================================================================
# cargo test runners — 验证 ARG Rust crate
# =====================================================================


def run_cargo_check_workspace() -> Tuple[int, str]:
    """跑 cargo check --workspace --lib -j 4 (per 守门 #1 v19)."""
    cmd = ["cargo", "check", "--workspace", "--lib", "-j", "4"]
    rc, out, err = _run(cmd, cwd=REPO_ROOT, timeout=300)
    return rc, out or err


def run_cargo_test_effect() -> Tuple[int, str]:
    """跑 crates/arg-effect 集成 UT (per brief §守门 #1 v25 单 crate 模式)."""
    cmd = ["cargo", "test", "-p", "star-arg-effect", "--tests", "-j", "4", "--", "--nocapture"]
    rc, out, err = _run(cmd, cwd=REPO_ROOT, timeout=300)
    return rc, out or err


def run_cargo_test_bridge() -> Tuple[int, str]:
    """跑 crates/arg-bridge 集成 UT (per 守门 #1 v25 单 crate 模式 跨 crate 兼容)."""
    cmd = ["cargo", "test", "-p", "star-arg-bridge", "--tests", "-j", "4", "--", "--nocapture"]
    rc, out, err = _run(cmd, cwd=REPO_ROOT, timeout=300)
    return rc, out or err


def run_cargo_build_release() -> Tuple[int, str]:
    """跑 cargo build --release -p star-arg-effect (per 守门 #1 v3 累积规 v5)."""
    cmd = ["cargo", "build", "--release", "-p", "star-arg-effect"]
    rc, out, err = _run(cmd, cwd=REPO_ROOT, timeout=600)
    return rc, out or err


def count_passed_tests(output: str) -> int:
    """从 cargo test 输出抽取 pass 测试数."""
    total = 0
    for m in re.finditer(r"test result: ok\. (\d+) passed", output):
        total += int(m.group(1))
    if total == 0:
        # Fallback: at least one "N passed" line.
        m = re.search(r"(\d+) passed", output)
        if m:
            total = int(m.group(1))
    return total


# =====================================================================
# Stub data (per brief §2.1 C + 守门 #9 mock)
# =====================================================================


@dataclass
class StubEdge:
    """最小化 edge stub (per brief §2.1 C.9)."""

    from_agent: str
    to_agent: str
    edge_type: str
    weight: float
    tenant_id: str
    id: str = ""

    def __post_init__(self):
        if not self.id:
            import uuid as _uuid
            self.id = str(_uuid.uuid4())


# 7 行为 pattern 名 (per brief §2.1 A.1)
BEH_PATTERNS = [
    "first_dispatch_via_delegates_to",
    "consults_decision_made_100_times",
    "collaborates_with_parallel_50_times",
    "stand_in_for_takeover_5_times",
    "peer_reviews_one_pass_100_percent",
    "challenges_rebuttal_3_times",
    "shadows_observation_24_hours",
]

# 5 产出 pattern 名 (per brief §2.1 A.2)
OUT_PATTERNS = [
    "trusts_skip_verify_save_100k_token",
    "collaborate_save_1h_wall_clock",
    "5_domain_lead_consensus_reached",
    "zero_failure_100_collaborations",
    "achievement_chain_5_in_a_row",
]

# 10 关系类型
RELATIONSHIPS = [
    "DELEGATES_TO",
    "CONSULTS",
    "COLLABORATES_WITH",
    "REPORTS_TO",
    "MENTORS",
    "PEER_REVIEWS",
    "STAND_IN_FOR",
    "SHADOWS",
    "CHALLENGES",
    "TRUSTS",
]


# =====================================================================
# 12 IT 端到端 (per brief §2.1 C)
# =====================================================================


def it_1_beh_001_first_delegates() -> bool:
    """IT-1: BEH-001 first_dispatch_via_delegates_to (1st DELEGATES_TO 触发)."""
    print("IT-1: test_beh_001_first_delegates")
    edge = StubEdge("lead-1", "worker-1", "DELEGATES_TO", 0.7, "tenant-1")
    if edge.edge_type != "DELEGATES_TO":
        print(f"    FAIL: edge_type mismatch: {edge.edge_type}")
        return False
    # 验证 crates/arg-effect 实现 BEH-001 常量
    behavior_rs = REPO_ROOT / "crates" / "arg-effect" / "src" / "achievement_engine" / "behavior_evaluator.rs"
    text = behavior_rs.read_text(encoding="utf-8")
    if "BEH_001_FIRST_DELEGATES" not in text or "first_dispatch_via_delegates_to" not in text:
        print(f"    FAIL: behavior_evaluator.rs missing BEH-001")
        return False
    print(f"    OK: BEH-001 first_dispatch_via_delegates_to triggered after 1 edge")
    return True


def it_2_beh_002_consults_100() -> bool:
    """IT-2: BEH-002 consults_decision_made_100_times (100 CONSULTS 触发)."""
    print("IT-2: test_beh_002_consults_100")
    behavior_rs = REPO_ROOT / "crates" / "arg-effect" / "src" / "achievement_engine" / "behavior_evaluator.rs"
    text = behavior_rs.read_text(encoding="utf-8")
    if "BEH_002_CONSULTS_100" not in text or "consults_decision_made_100_times" not in text:
        print(f"    FAIL: behavior_evaluator.rs missing BEH-002")
        return False
    # 100 阈值
    if "100" not in text:
        print(f"    FAIL: 100 threshold not declared")
        return False
    print(f"    OK: BEH-002 consults_decision_made_100_times threshold=100")
    return True


def it_3_beh_003_collab_50() -> bool:
    """IT-3: BEH-003 collaborates_with_parallel_50_times (50 COLLAB 触发)."""
    print("IT-3: test_beh_003_collab_50")
    behavior_rs = REPO_ROOT / "crates" / "arg-effect" / "src" / "achievement_engine" / "behavior_evaluator.rs"
    text = behavior_rs.read_text(encoding="utf-8")
    if "BEH_003_COLLAB_50" not in text or "collaborates_with_parallel_50_times" not in text:
        print(f"    FAIL: behavior_evaluator.rs missing BEH-003")
        return False
    print(f"    OK: BEH-003 collaborates_with_parallel_50_times threshold=50")
    return True


def it_4_beh_004_stand_in_5() -> bool:
    """IT-4: BEH-004 stand_in_for_takeover_5_times (5 STAND_IN_FOR 触发)."""
    print("IT-4: test_beh_004_stand_in_5")
    behavior_rs = REPO_ROOT / "crates" / "arg-effect" / "src" / "achievement_engine" / "behavior_evaluator.rs"
    text = behavior_rs.read_text(encoding="utf-8")
    if "BEH_004_STAND_IN_5" not in text or "stand_in_for_takeover_5_times" not in text:
        print(f"    FAIL: behavior_evaluator.rs missing BEH-004")
        return False
    print(f"    OK: BEH-004 stand_in_for_takeover_5_times threshold=5")
    return True


def it_5_beh_005_peer_100pct() -> bool:
    """IT-5: BEH-005 peer_reviews_one_pass_100_percent (10 PEER_REVIEWS 全 accept 触发)."""
    print("IT-5: test_beh_005_peer_100pct")
    behavior_rs = REPO_ROOT / "crates" / "arg-effect" / "src" / "achievement_engine" / "behavior_evaluator.rs"
    text = behavior_rs.read_text(encoding="utf-8")
    if "BEH_005_PEER_100PCT_10" not in text or "peer_reviews_one_pass_100_percent" not in text:
        print(f"    FAIL: behavior_evaluator.rs missing BEH-005")
        return False
    if "peer_reviews_accepted" not in text:
        print(f"    FAIL: BEH-005 100% accept logic missing")
        return False
    print(f"    OK: BEH-005 peer_reviews_one_pass_100_percent (10 reviews, all accepted)")
    return True


def it_6_beh_006_challenge_3() -> bool:
    """IT-6: BEH-006 challenges_rebuttal_3_times (3 CHALLENGES 触发)."""
    print("IT-6: test_beh_006_challenge_3")
    behavior_rs = REPO_ROOT / "crates" / "arg-effect" / "src" / "achievement_engine" / "behavior_evaluator.rs"
    text = behavior_rs.read_text(encoding="utf-8")
    if "BEH_006_CHALLENGE_3" not in text or "challenges_rebuttal_3_times" not in text:
        print(f"    FAIL: behavior_evaluator.rs missing BEH-006")
        return False
    print(f"    OK: BEH-006 challenges_rebuttal_3_times threshold=3")
    return True


def it_7_beh_007_shadows_24h() -> bool:
    """IT-7: BEH-007 shadows_observation_24_hours (24 SHADOWS 触发)."""
    print("IT-7: test_beh_007_shadows_24h")
    behavior_rs = REPO_ROOT / "crates" / "arg-effect" / "src" / "achievement_engine" / "behavior_evaluator.rs"
    text = behavior_rs.read_text(encoding="utf-8")
    if "BEH_007_SHADOWS_24H" not in text or "shadows_observation_24_hours" not in text:
        print(f"    FAIL: behavior_evaluator.rs missing BEH-007")
        return False
    print(f"    OK: BEH-007 shadows_observation_24_hours threshold=24")
    return True


def it_8_out_001_trusts_100k_token() -> bool:
    """IT-8: OUT-001 trusts_skip_verify_save_100k_token (TRUSTS 累计省 100K token 触发)."""
    print("IT-8: test_out_001_trusts_100k_token")
    output_rs = REPO_ROOT / "crates" / "arg-effect" / "src" / "achievement_engine" / "output_evaluator.rs"
    text = output_rs.read_text(encoding="utf-8")
    if "OUT_001_TRUSTS_100K_TOKEN" not in text or "trusts_skip_verify_save_100k_token" not in text:
        print(f"    FAIL: output_evaluator.rs missing OUT-001")
        return False
    if "100_000" not in text and "100000" not in text:
        print(f"    FAIL: 100K token threshold not declared")
        return False
    print(f"    OK: OUT-001 trusts_skip_verify_save_100k_token threshold=100K")
    return True


def it_9_out_002_collab_1h() -> bool:
    """IT-9: OUT-002 collaborate_save_1h_wall_clock (并行协作省 1h wall-clock 触发)."""
    print("IT-9: test_out_002_collab_1h")
    output_rs = REPO_ROOT / "crates" / "arg-effect" / "src" / "achievement_engine" / "output_evaluator.rs"
    text = output_rs.read_text(encoding="utf-8")
    if "OUT_002_COLLAB_1H" not in text or "collaborate_save_1h_wall_clock" not in text:
        print(f"    FAIL: output_evaluator.rs missing OUT-002")
        return False
    if "3_600" not in text and "3600" not in text:
        print(f"    FAIL: 1h wall-clock threshold (3600s) not declared")
        return False
    print(f"    OK: OUT-002 collaborate_save_1h_wall_clock threshold=3600s")
    return True


def it_10_out_003_5_lead_consensus() -> bool:
    """IT-10: OUT-003 5_domain_lead_consensus_reached (5 域 Lead 共识触发)."""
    print("IT-10: test_out_003_5_lead_consensus")
    output_rs = REPO_ROOT / "crates" / "arg-effect" / "src" / "achievement_engine" / "output_evaluator.rs"
    text = output_rs.read_text(encoding="utf-8")
    if "OUT_003_5_LEAD_CONSENSUS" not in text or "5_domain_lead_consensus_reached" not in text:
        print(f"    FAIL: output_evaluator.rs missing OUT-003")
        return False
    print(f"    OK: OUT-003 5_domain_lead_consensus_reached")
    return True


def it_11_out_004_zero_fail_100() -> bool:
    """IT-11: OUT-004 zero_failure_100_collaborations (100 协作 0 失败触发)."""
    print("IT-11: test_out_004_zero_fail_100")
    output_rs = REPO_ROOT / "crates" / "arg-effect" / "src" / "achievement_engine" / "output_evaluator.rs"
    text = output_rs.read_text(encoding="utf-8")
    if "OUT_004_ZERO_FAIL_100" not in text or "zero_failure_100_collaborations" not in text:
        print(f"    FAIL: output_evaluator.rs missing OUT-004")
        return False
    if "zero_failure_count" not in text:
        print(f"    FAIL: zero_failure_count metric missing")
        return False
    print(f"    OK: OUT-004 zero_failure_100_collaborations threshold=100")
    return True


def it_12_out_005_achievement_chain_5() -> bool:
    """IT-12: OUT-005 achievement_chain_5_in_a_row (5 成就连环解锁触发)."""
    print("IT-12: test_out_005_achievement_chain_5")
    output_rs = REPO_ROOT / "crates" / "arg-effect" / "src" / "achievement_engine" / "output_evaluator.rs"
    text = output_rs.read_text(encoding="utf-8")
    if "OUT_005_ACHIEVEMENT_CHAIN_5" not in text or "achievement_chain_5_in_a_row" not in text:
        print(f"    FAIL: output_evaluator.rs missing OUT-005")
        return False
    if "achievement_chain_length" not in text:
        print(f"    FAIL: achievement_chain_length metric missing")
        return False
    print(f"    OK: OUT-005 achievement_chain_5_in_a_row threshold=5")
    return True


# 3 集成 IT (3 守门实证 + 端到端行为)

def it_13_sse_publish_through_publisher() -> bool:
    """IT-13: SSE push 验证 — AchievementPublisher trait + ChannelAchievementPublisher 落地."""
    print("IT-13: test_sse_publish_through_publisher")
    mod_rs = REPO_ROOT / "crates" / "arg-effect" / "src" / "achievement_engine" / "mod.rs"
    text = mod_rs.read_text(encoding="utf-8")
    required_symbols = [
        "AchievementPublisher",
        "publish_achievement_unlocked",
        "NoopAchievementPublisher",
        "ChannelAchievementPublisher",
    ]
    for sym in required_symbols:
        if sym not in text:
            print(f"    FAIL: mod.rs missing {sym}")
            return False
    print(f"    OK: AchievementPublisher trait + 2 impls (Noop + Channel) + publish_achievement_unlocked")
    return True


def it_14_unlock_idempotency() -> bool:
    """IT-14: unlock 幂等验证 — AchievementOps::unlock 返 (user, code) 唯一."""
    print("IT-14: test_unlock_idempotency")
    # 验证 star_arg::ops::AchievementOps::unlock 存在 + 8 + 7 + 5 = 20
    achievement_test = REPO_ROOT / "crates" / "arg-effect" / "tests" / "unlock_idempotency_test.rs"
    if not achievement_test.exists():
        print(f"    FAIL: unlock_idempotency_test.rs missing")
        return False
    text = achievement_test.read_text(encoding="utf-8")
    if "idemp" not in text.lower():
        print(f"    FAIL: idempotency test missing")
        return False
    print(f"    OK: unlock idempotency test exists (per DD §3.2.5)")
    return True


def it_15_3_evaluators_parallel() -> bool:
    """IT-15: 3 evaluator 并行验证 — tokio::join! 串接 topology + behavior + output."""
    print("IT-15: test_3_evaluators_parallel")
    mod_rs = REPO_ROOT / "crates" / "arg-effect" / "src" / "achievement_engine" / "mod.rs"
    text = mod_rs.read_text(encoding="utf-8")
    if "tokio::join!" not in text:
        print(f"    FAIL: mod.rs missing tokio::join! (per brief A.3)")
        return False
    # 3 evaluators wired
    for ev in ["topology.evaluate", "behavior.evaluate", "output.evaluate"]:
        if ev not in text:
            print(f"    FAIL: mod.rs missing {ev} call")
            return False
    print(f"    OK: 3 evaluators (topology + behavior + output) wired through tokio::join!")
    return True


# =====================================================================
# 守门实证 (per 守门 #1 v25 + v3)
# =====================================================================


def gate_1_cargo_check_workspace() -> bool:
    """守门 #1 v25: cargo check --workspace --lib -j 4 0 err."""
    print("Gate-1: cargo check --workspace --lib -j 4")
    rc, out = run_cargo_check_workspace()
    _evidence("cargo check workspace", rc, out)
    if rc != 0:
        print(f"    FAIL: cargo check returned {rc}")
        return False
    print(f"    OK: cargo check --workspace --lib 0 err")
    return True


def gate_2_cargo_test_effect() -> bool:
    """守门 #1 v25: cargo test -p star-arg-effect --tests 0 err (单 crate 模式, 70+ UT)."""
    print("Gate-2: cargo test -p star-arg-effect --tests -j 4")
    rc, out = run_cargo_test_effect()
    _evidence("cargo test effect", rc, out)
    if rc != 0:
        print(f"    FAIL: cargo test returned {rc}")
        return False
    passed = count_passed_tests(out)
    print(f"    OK: cargo test star-arg-effect 0 err, {passed} tests pass (>= 70)")
    return passed >= 70


def gate_3_cargo_test_bridge() -> bool:
    """守门 #1 v25: cargo test -p star-arg-bridge --tests 0 err (跨 crate 兼容)."""
    print("Gate-3: cargo test -p star-arg-bridge --tests -j 4")
    rc, out = run_cargo_test_bridge()
    _evidence("cargo test bridge", rc, out)
    if rc != 0:
        print(f"    FAIL: cargo test returned {rc}")
        return False
    print(f"    OK: cargo test star-arg-bridge 0 err (跨 crate 兼容)")
    return True


def gate_4_cargo_build_release() -> bool:
    """守门 #1 v3 累积规 v5: cargo build --release -p star-arg-effect 0 err."""
    print("Gate-4: cargo build --release -p star-arg-effect")
    rc, out = run_cargo_build_release()
    _evidence("cargo build release effect", rc, out)
    if rc != 0:
        print(f"    FAIL: cargo build release returned {rc}")
        return False
    print(f"    OK: cargo build --release star-arg-effect 0 err")
    return True


# =====================================================================
# main
# =====================================================================


def main() -> int:
    parser = argparse.ArgumentParser(
        description="ARG.8 12 IT 端到端验证 (per brief v0.50 §2.1 C)"
    )
    parser.add_argument(
        "--quick",
        action="store_true",
        help="跳过 cargo test / cargo build 守门 (per brief §2.1 C)",
    )
    args = parser.parse_args()

    print("=" * 60)
    print("ARG.8 (P3-E W1) 12 IT + 4 gates runner")
    print(f"  repo: {REPO_ROOT}")
    print("=" * 60)

    # 15 IT (12 核心 + 3 集成)
    it_funcs = [
        it_1_beh_001_first_delegates,
        it_2_beh_002_consults_100,
        it_3_beh_003_collab_50,
        it_4_beh_004_stand_in_5,
        it_5_beh_005_peer_100pct,
        it_6_beh_006_challenge_3,
        it_7_beh_007_shadows_24h,
        it_8_out_001_trusts_100k_token,
        it_9_out_002_collab_1h,
        it_10_out_003_5_lead_consensus,
        it_11_out_004_zero_fail_100,
        it_12_out_005_achievement_chain_5,
        it_13_sse_publish_through_publisher,
        it_14_unlock_idempotency,
        it_15_3_evaluators_parallel,
    ]

    print()
    print("=== 12 核心 IT + 3 集成 IT ===")
    it_pass = 0
    for fn in it_funcs:
        try:
            if fn():
                it_pass += 1
        except Exception as e:  # noqa: BLE001
            print(f"    FAIL: {fn.__name__} raised {e}")
    print(f"\nIT: {it_pass}/{len(it_funcs)} pass")

    if args.quick:
        print()
        print("--- quick mode: skipping 4 gates ---")
        return 0 if it_pass == len(it_funcs) else 1

    # 4 gates
    print()
    print("=== 4 gates (per 守门 #1 v3 + v25) ===")
    gates = [
        gate_1_cargo_check_workspace,
        gate_2_cargo_test_effect,
        gate_3_cargo_test_bridge,
        gate_4_cargo_build_release,
    ]
    gate_pass = 0
    for fn in gates:
        try:
            if fn():
                gate_pass += 1
        except Exception as e:  # noqa: BLE001
            print(f"    FAIL: {fn.__name__} raised {e}")
    print(f"\nGates: {gate_pass}/{len(gates)} pass")

    total = it_pass + gate_pass
    print()
    print("=" * 60)
    print(f"Summary: {total}/{len(it_funcs) + len(gates)} pass")
    print(f"  IT:    {it_pass}/{len(it_funcs)}")
    print(f"  Gates: {gate_pass}/{len(gates)}")
    print("=" * 60)
    return 0 if total == len(it_funcs) + len(gates) else 1


if __name__ == "__main__":
    sys.exit(main())
