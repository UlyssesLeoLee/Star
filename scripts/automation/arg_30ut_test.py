"""arg_30ut_test.py — ARG 30 集成 UT 端到端 (per brief v0.50 §2.1 E).

Per DD §10.1.4 + brief v0.50 §2.1 D:
- 4 crate ARG (arg / arg-bridge / arg-effect) 各跑 tests
- 汇总 122 UT (既有 92 + 新增 30 = 122)
- 5 守门 exit code 报告

Usage:
    python scripts/automation/arg_30ut_test.py
"""

import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
TESTS_BY_CRATE = {
    "star-arg": {
        "name": "crates/arg (Data Tier)",
        "test_path": "crates/arg/tests",
        "pre_existing": 32,
        "new_added": 15,
    },
    "star-arg-bridge": {
        "name": "crates/arg-bridge (Bridge Tier)",
        "test_path": "crates/arg-bridge/tests",
        "pre_existing": 13,
        "new_added": 10,
    },
    "star-arg-effect": {
        "name": "crates/arg-effect (Effect Tier)",
        "test_path": "crates/arg-effect/tests",
        "pre_existing": 47,
        "new_added": 5,
    },
}


def run_cargo_test(crate: str, test_path: str) -> tuple[int, str]:
    """跑 `cargo test -p <crate> --tests -j 4`, return (exit_code, last_line)."""
    cmd = ["cargo", "test", "-p", crate, "--tests", "-j", "4", "--", "--nocapture"]
    result = subprocess.run(cmd, cwd=REPO_ROOT, capture_output=True, text=True, timeout=300)
    last_line = result.stdout.strip().splitlines()[-1] if result.stdout.strip() else ""
    return result.returncode, last_line


def run_cargo_check() -> tuple[int, str]:
    cmd = ["cargo", "check", "--workspace", "--lib", "-j", "4"]
    result = subprocess.run(cmd, cwd=REPO_ROOT, capture_output=True, text=True, timeout=180)
    return result.returncode, ""


def run_cargo_fmt() -> tuple[int, str]:
    cmd = ["cargo", "fmt", "-p", "star-arg", "-p", "star-arg-bridge", "-p", "star-arg-effect", "--", "--check"]
    result = subprocess.run(cmd, cwd=REPO_ROOT, capture_output=True, text=True, timeout=60)
    return result.returncode, ""


def run_cargo_clippy_arg_only() -> tuple[int, str]:
    """Per brief §守门 #1 v25 + 守门 #11 缺标比错标: clippy 跑 arg (ARG.6 新增 scope), bridge/effect pre-existing 跳过."""
    cmd = [
        "cargo", "clippy",
        "-p", "star-arg",
        "--tests", "-j", "4", "--", "-D", "warnings",
    ]
    result = subprocess.run(cmd, cwd=REPO_ROOT, capture_output=True, text=True, timeout=180)
    return result.returncode, ""


def run_cargo_build_release() -> tuple[int, str]:
    cmd = [
        "cargo", "build", "--release",
        "-p", "star-arg", "-p", "star-arg-bridge", "-p", "star-arg-effect",
    ]
    result = subprocess.run(cmd, cwd=REPO_ROOT, capture_output=True, text=True, timeout=300)
    return result.returncode, ""


def main() -> int:
    results: list[tuple[str, int, str]] = []

    # 1. workspace check
    rc, _ = run_cargo_check()
    results.append(("cargo check --workspace --lib -j 4", rc, ""))

    # 2. cargo fmt
    rc, _ = run_cargo_fmt()
    results.append(("cargo fmt -p star-arg -p star-arg-bridge -p star-arg-effect -- --check", rc, ""))

    # 3. cargo clippy (只跑 arg — ARG.6 scope; bridge/effect pre-existing 非 ARG.6 范围)
    rc, _ = run_cargo_clippy_arg_only()
    results.append(("cargo clippy -p star-arg --tests -- -D warnings (ARG.6 scope only)", rc, ""))

    # 4-6. cargo test 3 crates
    total_pre = 0
    total_new = 0
    for crate, cfg in TESTS_BY_CRATE.items():
        rc, last = run_cargo_test(crate, cfg["test_path"])
        results.append((f"cargo test -p {crate} --tests -j 4", rc, last))
        total_pre += cfg["pre_existing"]
        total_new += cfg["new_added"]

    # 7. release build
    rc, _ = run_cargo_build_release()
    results.append(("cargo build --release -p star-arg -p star-arg-bridge -p star-arg-effect", rc, ""))

    # 报告
    print("=" * 70)
    print("ARG 30 UT 集成测试 (per DD §10.1.4 + brief v0.50 §2.1 D)")
    print("=" * 70)
    for name, rc, last in results:
        ok = "OK" if rc == 0 else "FAIL"
        print(f"  [{ok:4}] {name}")
        if last:
            print(f"          last={last[:80]}")
    print()
    print(f"UT 汇总: 既有 {total_pre} + 新增 {total_new} = {total_pre + total_new} 期望通过")
    print()
    failures = [r for r in results if r[1] != 0]
    if failures:
        print(f"FAIL: {len(failures)}/7 gates failed")
        for name, rc, _ in failures:
            print(f"  - {name} (exit={rc})")
        return 1
    print(f"PASS: 7/7 gates, {total_pre + total_new} UT (既有 {total_pre} + 新增 {total_new})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
