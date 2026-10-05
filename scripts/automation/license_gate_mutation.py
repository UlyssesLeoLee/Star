#!/usr/bin/env python3
"""cargo-deny 许可门禁变异测试 (mutation test).

用途: 验证 deny.toml 的许可门禁**有牙**, 而不只是"跑起来不报错"。
绿灯只证明"没报错"; 本脚本注入受控的许可表达式, 断言门禁的判别行为。

依据: AGENTS.md §0 商业开源依赖硬约束
  - 必须允许不限用途/行业/席位/用量的商业使用
  - 不得把 copyleft 等同于禁止商用, 也不得仅因其为 GPL/AGPL/LGPL 而一概排除
  - 排除非商业 (NC)、field-of-use、source-available

设计要点 (三要素缺一不可):
  (a) 注入违规 -> 门禁必须变红
  (b) 对照组   -> 合法许可必须仍绿 (排除"测试装置永远失败")
  (c) 反例守卫 -> copyleft 与公有领域必须放行 (排除规则写宽变成永久误报)
  额外: 每个 red 用例都断言诊断文本本身出现, 排除"因错误原因变红"。
        实战教训: 首轮注入位置错误导致 workspace manifest 解析失败,
        门禁因 `cargo metadata` 报错而红 —— 若只看退出码会误判为 PASS。

用法:
    python scripts/automation/license_gate_mutation.py

退出码:
    0 = 全部用例符合预期
    1 = 存在 FAIL (门禁行为与硬约束不符)
    2 = INVALID (装置故障: 探针注入后 manifest 不可解析 / 工具缺失)

环境要求:
    cargo-deny 在 PATH 上, 或用 CARGO_DENY_BIN 指定绝对路径
    (本机 cargo 家族工具位于 E:\\DevCache\\cargo\\bin, 默认不在 PATH 上)
"""

from __future__ import annotations

import os
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
PROBE_DIR = REPO_ROOT / "crates" / "zz-license-probe"
PROBE_MEMBER = '"crates/zz-license-probe"'

# (SPDX, 期望, 说明)  期望 G=必须放行, R=必须拒绝
CASES: list[tuple[str, str, str]] = [
    ("Apache-2.0", "G", "control: 宽松型必须放行"),
    ("MIT", "G", "control: MIT 必须放行"),
    ("MPL-2.0", "G", "counter-guard: 弱 copyleft 必须放行"),
    ("GPL-3.0-only", "G", "counter-guard: GPL 必须放行 (不得一概排除)"),
    ("AGPL-3.0-only", "G", "counter-guard: AGPL 必须放行 (不得一概排除)"),
    ("CC0-1.0", "G", "counter-guard: 真正的公有领域必须放行"),
    ("CC-BY-NC-4.0", "R", "non-commercial 必须拒绝"),
    ("BUSL-1.1", "R", "source-available 必须拒绝"),
    ("SSPL-1.0", "R", "source-available 必须拒绝"),
    ("Elastic-2.0", "R", "source-available 必须拒绝"),
]

PROBE_MANIFEST = """[package]
name = "zz-license-probe"
version = "0.1.0"
edition = "2021"
license = "{lic}"
"""

# 诊断文本特征: 用于区分"因许可原因红"与"因装置故障红"
_BROKEN_PATTERNS = (
    "cargo metadata` exited with an error",
    "failed to fetch crates",
    "key with no value",
    "failed to deserialize config",
    "failed to validate configuration",
)
_LICENSE_REASON = re.compile(r"rejected: license|not explicitly allowed")


def find_cargo_deny() -> str:
    env = os.environ.get("CARGO_DENY_BIN")
    if env and Path(env).exists():
        return env
    found = shutil.which("cargo-deny")
    if found:
        return found
    for candidate in (
        r"E:\DevCache\cargo\bin\cargo-deny.exe",
        Path.home() / ".cargo" / "bin" / "cargo-deny",
    ):
        if Path(candidate).exists():
            return str(candidate)
    sys.stderr.write("cargo-deny not found. Set CARGO_DENY_BIN.\n")
    raise SystemExit(2)


def run_gate(cargo_deny: str) -> tuple[int, str]:
    proc = subprocess.run(
        [cargo_deny, "check", "licenses", "--hide-inclusion-graph"],
        cwd=REPO_ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    return proc.returncode, proc.stdout + proc.stderr


def inject_probe(license_id: str) -> None:
    """在 workspace members 中注入探针 crate, 并写入受控的 license 字段."""
    (PROBE_DIR / "src").mkdir(parents=True, exist_ok=True)
    (PROBE_DIR / "src" / "lib.rs").write_text(
        "//! Temporary license-gate mutation probe.\n", encoding="utf-8"
    )
    (PROBE_DIR / "Cargo.toml").write_text(
        PROBE_MANIFEST.format(lic=license_id), encoding="utf-8"
    )
    cargo_toml = REPO_ROOT / "Cargo.toml"
    # bytes 而非 text: 见 main() 的换行符说明。text 往返会把 LF 全量转成 CRLF,
    # 即使随后"还原"成功, 磁盘上的 Cargo.toml 也已被改写成另一种换行风格。
    text = cargo_toml.read_bytes().decode("utf-8")
    if PROBE_MEMBER not in text:
        # 插到 members 数组**结束之前**: 定位 members 的闭合 `]`
        idx = text.index("exclude = [")
        close = text.rindex("]", 0, idx)
        patched = text[:close] + f"    {PROBE_MEMBER},\n" + text[close:]
        cargo_toml.write_bytes(patched.encode("utf-8"))


def main() -> int:
    cargo_deny = find_cargo_deny()
    # bytes 备份/还原而非 text: Python 文本模式读写在 Windows 上会把 LF 转成
    # CRLF (newline=None 读时归一化 + 写时按 os.linesep 转换), 于是"还原"动作
    # 本身就会把整个文件改写成另一种换行风格。本仓 core.autocrlf=true 会把这种
    # 差异在 git 层归一化掉, 使其**不可见** —— 但那是本仓配置的巧合, 不是脚本
    # 的正确性。换 core.autocrlf=input/false 的开发者会直接拿到一个脏工作区。
    backup = (REPO_ROOT / "Cargo.toml").read_bytes()
    # Cargo.lock 也必须备份: `cargo deny` 内部会跑 `cargo metadata`, 该步骤把临时
    # 探针写进 lock 文件 (workspace members 的一部分), 删除探针后条目会残留成幽灵
    # member。仅还原 Cargo.toml 会让仓库在脚本跑完后一直是 dirty 状态, 且下次
    # `cargo check` 会"顺手"把这条残留删掉 —— 看起来像别人改的, 实为本脚本泄漏。
    lock_path = REPO_ROOT / "Cargo.lock"
    lock_backup = lock_path.read_bytes() if lock_path.exists() else None

    passed = failed = invalid = 0
    try:
        for lic, expect, desc in CASES:
            inject_probe(lic)
            rc, out = run_gate(cargo_deny)

            broken = any(p in out for p in _BROKEN_PATTERNS)
            license_reasoned = bool(_LICENSE_REASON.search(out))

            if broken:
                verdict, invalid = "INVALID(device fault)", invalid + 1
            elif expect == "R" and rc != 0 and license_reasoned:
                verdict, passed = "PASS(red for right reason)", passed + 1
            elif expect == "G" and rc == 0:
                verdict, passed = "PASS(green)", passed + 1
            else:
                verdict = f"FAIL(expect={expect} rc={rc} reasoned={license_reasoned})"
                failed += 1

            print(f"{lic:<16} expect={expect} rc={rc:<3} {verdict:<28} | {desc}")
    finally:
        # 还原: Cargo.toml + Cargo.lock 恢复原状, 探针移入可恢复删除
        (REPO_ROOT / "Cargo.toml").write_bytes(backup)
        if lock_backup is not None:
            lock_path.write_bytes(lock_backup)
        if PROBE_DIR.exists():
            shutil.rmtree(PROBE_DIR, ignore_errors=True)

    # 后置断言 (per 守门 #30: 断言必须能区分"被测物没跑"): 还原不是尽力而为,
    # 失败必须以非零 exit 报出, 否则泄漏会以"别人的改动"形式潜伏到下一次 commit。
    # 断言的是**字节**而非文本内容: 只比对内容会把"换行符被整体改写"这种真实泄漏
    # 判成通过 —— 那正是本脚本最初漏掉的那一类。
    leaks = []
    cargo_toml_path = REPO_ROOT / "Cargo.toml"
    if PROBE_DIR.exists():
        leaks.append(f"probe dir still present: {PROBE_DIR}")
    if PROBE_MEMBER.encode("utf-8") in cargo_toml_path.read_bytes():
        leaks.append("probe member still present in Cargo.toml")
    if cargo_toml_path.read_bytes() != backup:
        leaks.append("Cargo.toml not restored to pre-run bytes")
    if lock_backup is not None and lock_path.read_bytes() != lock_backup:
        leaks.append("Cargo.lock not restored to pre-run bytes")
    if leaks:
        print("\n[LEAK] 工作区还原失败:", file=sys.stderr)
        for item in leaks:
            print(f"  - {item}", file=sys.stderr)
        return 2

    print(f"\nPASS={passed}  FAIL={failed}  INVALID={invalid}  LEAK=0")
    if invalid:
        return 2
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
