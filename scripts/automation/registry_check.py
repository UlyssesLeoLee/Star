#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
scripts/automation/registry_check.py — 索引一致性校验
(per docs/automation-design.md §6.7)

校验 `scripts/automation/registry.md` 索引跟实际脚本一致
(脚本路径 / 用途 / 调用方 / 末次 commit)。

用法:
    python scripts/automation/registry_check.py

输出:
    - stdout: 校验结果 (warning 不阻塞 CI, error 阻塞)
    - exit code: 0 = 一致, 1 = 有 error
    - 不一致项输出到 docs/reports/registry-check.log

约束 (per 守门 #1 v1):
    - 标准库 only: re / pathlib / json
    - 不阻塞 CI, 仅 warning (per §6.7)
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
import time
from dataclasses import dataclass, asdict
from pathlib import Path
from typing import Optional

ROOT_DEFAULT = Path(__file__).resolve().parent.parent.parent
AUTOMATION_DIR = ROOT_DEFAULT / "scripts" / "automation"
REGISTRY_MD = AUTOMATION_DIR / "registry.md"
REPORTS_DIR_DEFAULT = ROOT_DEFAULT / "docs" / "reports"


@dataclass
class CheckResult:
    """校验结果"""

    script_path: str
    exists: bool
    in_registry: bool
    last_commit: Optional[str]
    warnings: list
    errors: list


def parse_registry() -> dict:
    """解析 registry.md → {脚本路径: {用途, 调用方, 末次 commit}}"""
    if not REGISTRY_MD.exists():
        return {}
    content = REGISTRY_MD.read_text(encoding="utf-8")
    # 解析 markdown 表格行: | `scripts/automation/<file>.py` | 用途 | 调用方 | commit | 状态 |
    pattern = re.compile(
        r"\|\s*`?(scripts/automation/[^\s|`]+?\.py)`?\s*\|\s*([^|]*?)\s*\|\s*([^|]*?)\s*\|\s*([^|]*?)\s*\|\s*([^|]*?)\s*\|",
        re.MULTILINE,
    )
    result = {}
    for m in pattern.finditer(content):
        path, purpose, caller, commit, status = m.groups()
        result[path] = {
            "purpose": purpose.strip(),
            "caller": caller.strip(),
            "commit": commit.strip(),
            "status": status.strip(),
        }
    return result


def scan_actual_scripts() -> list:
    """扫描 scripts/automation/ 实际脚本"""
    if not AUTOMATION_DIR.exists():
        return []
    scripts = []
    for fp in AUTOMATION_DIR.rglob("*.py"):
        rel = fp.relative_to(ROOT_DEFAULT).as_posix()
        scripts.append(rel)
    return sorted(scripts)


def get_last_commit(file_path: str) -> Optional[str]:
    """取文件末次 commit (stub: 真实实装需 git log 调用)"""
    try:
        proc = subprocess.run(
            ["git", "log", "-1", "--format=%h", "--", file_path],
            capture_output=True,
            text=True,
            timeout=5,
            cwd=ROOT_DEFAULT,
        )
        if proc.returncode == 0 and proc.stdout.strip():
            return proc.stdout.strip()
    except (subprocess.TimeoutExpired, FileNotFoundError):
        pass
    return None


def check() -> list:
    """主校验: 实际脚本 vs registry.md"""
    registry = parse_registry()
    actual = scan_actual_scripts()

    results = []
    for script in actual:
        in_registry = script in registry
        last_commit = get_last_commit(script)
        warnings = []
        errors = []

        if not in_registry:
            warnings.append(f"脚本 {script} 未在 registry.md 登记")

        entry = registry.get(script, {})
        if entry.get("commit", "").startswith("`") and entry.get("commit", "") != f"`{last_commit}`" if last_commit else True:
            if last_commit and entry.get("commit") and entry["commit"] != f"`{last_commit}`" and entry["commit"] != "TBD":
                warnings.append(f"registry commit ({entry['commit']}) != 实际 ({last_commit})")

        results.append(CheckResult(
            script_path=script,
            exists=True,
            in_registry=in_registry,
            last_commit=last_commit,
            warnings=warnings,
            errors=errors,
        ))

    # 反向: registry 里的脚本不存在
    for path, entry in registry.items():
        if path not in actual:
            results.append(CheckResult(
                script_path=path,
                exists=False,
                in_registry=True,
                last_commit=entry.get("commit"),
                warnings=[],
                errors=[f"registry.md 引用 {path} 但实际不存在"],
            ))

    return results


def main():
    parser = argparse.ArgumentParser(description="registry.md 一致性校验 CLI")
    parser.add_argument("--output-log", type=Path, help="校验日志路径")
    args = parser.parse_args()

    output_log = args.output_log or (REPORTS_DIR_DEFAULT / "registry-check.log")
    output_log.parent.mkdir(parents=True, exist_ok=True)

    results = check()

    total_warnings = sum(len(r.warnings) for r in results)
    total_errors = sum(len(r.errors) for r in results)

    print(f"=== registry_check ===")
    print(f"actual_scripts={len([r for r in results if r.exists])}")
    print(f"warnings={total_warnings}")
    print(f"errors={total_errors}")
    print()

    for r in results:
        status = "OK"
        if r.errors:
            status = "ERROR"
        elif r.warnings:
            status = "WARN"
        print(f"[{status}] {r.script_path}")
        for w in r.warnings:
            print(f"  WARN: {w}")
        for e in r.errors:
            print(f"  ERROR: {e}")

    # 写日志
    with output_log.open("a", encoding="utf-8") as f:
        f.write(f"\n=== registry_check @ {time.strftime('%Y-%m-%d %H:%M:%S')} ===\n")
        f.write(f"warnings={total_warnings} errors={total_errors}\n")
        for r in results:
            f.write(json.dumps(asdict(r), ensure_ascii=False) + "\n")

    sys.exit(0 if total_errors == 0 else 1)


if __name__ == "__main__":
    main()
