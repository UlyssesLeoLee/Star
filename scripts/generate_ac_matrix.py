#!/usr/bin/env python3
# scripts/generate_ac_matrix.py
# AC ↔ Test Case 矩阵生成器 (per docs/test-design.md §6.2.1 + AGENTS.md 守门)
#
# 输入:
#   - docs/requirements.md  (扫描 REQ-XXX-NNN + §27.2 Acceptance Coverage 提及 AC-NNN)
#   - frontend/src/**/*.{test,spec}.{ts,tsx}   (单测 / 集成测试)
#   - frontend/e2e/**/*.{test,spec}.{ts,tsx}  (e2e / 验收测试)
#
# 输出:
#   - docs/ac-test-matrix.csv  (列: AC_ID / 描述 / 关联 REQ / 单测 / 集成 / e2e / 状态)
#
# 设计约束 (per 缺标比错标安全):
#   - 标准库 only: re / csv / pathlib / argparse / sys (无第三方)
#   - 入口 `if __name__ == "__main__":` 可直接 `python scripts/generate_ac_matrix.py`
#   - REQ 找不到 AC 关联 → 状态 gap (待人工补 mapping)
#   - AC 找不到测试覆盖 → 状态 gap (per REQ-TST-002 显式指出缺口)
#
# 已知缺口 (TBD 待 basic-design §4.5.6 跟进):
#   1. requirements.md 中 AC 编号体系不完整 (只发现 AC-001 占位示例, 真实 AC ID TBD)
#      → 本版本同时扫描 REQ-* 全部 18 模块 (72 条) 作为代理行, 标 AC_ID = REQ_ID
#         (per test-design §6.2.1 应出 AC-XXX-NNN 行, 当前 mock 阶段先覆盖 REQ 全集)
#   2. 测试覆盖匹配: 启发式按 "测试文件名 / 测试内容含 REQ id 字符串" 关联, 误报可能
#   3. 集成测试 / e2e 不分层 (vitest 把 frontend/src/** 和 e2e/** 都吃, 但 e2e/cross-domain-5b.spec.ts
#      是 playwright-only), 集成列只标 frontend/src/**/.{test,spec}.ts(x) 里非 "component" / "store"
#      等 unit-only 文件 (启发式)

from __future__ import annotations

import argparse
import csv
import re
import sys
from pathlib import Path
from typing import Iterable

ROOT_DEFAULT = Path(__file__).resolve().parent.parent

# 匹配 REQ-XXX-NNN (3-5 字母模块码 + 3 位数字) 在行首 (在 `- ` 之后)
# 这样不会把 cross-reference (e.g. "见 REQ-WF-003") 误识别为新行
# 使用 re.MULTILINE 让 ^ 匹配每行行首
REQ_PATTERN = re.compile(
    r"^[\s\-]*(REQ-[A-Z]{2,5}-\d{3})(?:\s|[:\uff1a\u3001]|$)",
    re.MULTILINE,
)
# 匹配 AC-XXX-NNN (per docs/requirements.md §27.2)
AC_PATTERN = re.compile(r"\b(AC-[A-Z0-9]{2,5}-\d{3})\b")
# 匹配纯 AC-001 / AC-001A 这种简化形式
AC_SIMPLE_PATTERN = re.compile(r"\b(AC-\d{2,4}[A-Z]?)\b")

# 集成测试文件启发式: 文件名含 "integration" / "api" / "handler" / "contract"
# 其余 frontend/src/**/.{test,spec}.ts(x) 视为 unit
UNIT_HINTS = (
    "component",
    "store",
    "snapshot",
    "fixtures-sync",
    "validation-level",
    "kanban",
    "inbox",
    "agents",
    "analytics",
    "cli",
    "real-mode",
)
INTEGRATION_HINTS = ("integration", "api", "handler", "contract", "service")


def is_unit_test_file(path: Path) -> bool:
    name = path.name.lower()
    if any(h in name for h in UNIT_HINTS):
        return True
    return False


def is_integration_test_file(path: Path) -> bool:
    name = path.name.lower()
    if any(h in name for h in INTEGRATION_HINTS):
        return True
    return False


def is_e2e_test_file(path: Path) -> bool:
    return "e2e" in str(path).lower().replace("\\", "/")


def find_test_files(root: Path) -> list[Path]:
    """Discover all vitest/playwright test files."""
    results: list[Path] = []
    src_glob = root / "frontend" / "src"
    if src_glob.exists():
        for ext in ("test.ts", "test.tsx", "spec.ts", "spec.tsx"):
            results.extend(src_glob.rglob(f"*.{ext}"))
    e2e_glob = root / "frontend" / "e2e"
    if e2e_glob.exists():
        for ext in ("test.ts", "test.tsx", "spec.ts", "spec.tsx"):
            results.extend(e2e_glob.rglob(f"*.{ext}"))
    return sorted(set(results))


def read_text(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except UnicodeDecodeError:
        return path.read_text(encoding="utf-8", errors="replace")


def extract_reqs(requirements_path: Path) -> list[tuple[str, str, list[str]]]:
    """Parse requirements.md for REQ-* lines. Return (req_id, description, related_ac_ids)."""
    if not requirements_path.exists():
        return []
    text = read_text(requirements_path)
    rows: list[tuple[str, str, list[str]]] = []
    for line in text.splitlines():
        # 形态: - REQ-XXX-001: 描述文字... (ASCII or fullwidth colon)
        m = REQ_PATTERN.match(line)
        if not m:
            continue
        req_id = m.group(1)
        # 描述: 取 ":" 或 "：" 后到行尾. 注意 m.end() 是整个 match 的结束
        # (可能已吃掉 ":" 了), 用 m.end(1) 即捕获组 1 结束位置, 重新从那里找 ":"
        end_of_id = m.end(1)
        colon_idx = -1
        for candidate in (":", "："):
            i = line.find(candidate, end_of_id)
            if i != -1 and (colon_idx == -1 or i < colon_idx):
                colon_idx = i
        if colon_idx == -1:
            # 没找到, 把整行去掉 req_id 前缀作为描述
            description = line[end_of_id:].strip()
        else:
            description = line[colon_idx + 1 :].strip()
        # 描述里出现的 AC id 视为关联
        related_ac = sorted(set(AC_PATTERN.findall(description)) | set(AC_SIMPLE_PATTERN.findall(description)))
        rows.append((req_id, description, related_ac))
    return rows


def match_req_in_file(req_id: str, test_text: str) -> bool:
    """Heuristic: 文件正文是否引用 req_id 字符串."""
    return req_id in test_text


def categorize_coverage(
    req_id: str,
    test_files: list[Path],
) -> tuple[bool, bool, bool]:
    """Return (unit_covered, integration_covered, e2e_covered) by scanning each test file."""
    unit = integ = e2e = False
    for tf in test_files:
        text = read_text(tf)
        if not match_req_in_file(req_id, text):
            continue
        if is_e2e_test_file(tf):
            e2e = True
        elif is_integration_test_file(tf):
            integ = True
        elif is_unit_test_file(tf):
            unit = True
        else:
            # 默认归 unit (保守)
            unit = True
    return unit, integ, e2e


def build_matrix(
    reqs: list[tuple[str, str, list[str]]],
    test_files: list[Path],
) -> list[dict[str, str]]:
    # 去重: 同一 REQ_ID 多行只取首条 (后续行可能是 cross-ref 误判)
    seen: set[str] = set()
    rows: list[dict[str, str]] = []
    for req_id, description, related_ac in reqs:
        if req_id in seen:
            continue
        seen.add(req_id)
        unit, integ, e2e = categorize_coverage(req_id, test_files)
        # 状态: 全 covered 至少 1 列, 否则 gap
        any_covered = unit or integ or e2e
        status = "covered" if any_covered else "gap"
        # AC_ID 字段: 优先用 description 里提取的 AC, 否则退化为 REQ id 作占位
        ac_id_field = ",".join(related_ac) if related_ac else req_id
        rows.append(
            {
                "AC_ID": ac_id_field,
                "description": description[:120],  # truncate for csv
                "REQ_ID": req_id,
                "unit": "Y" if unit else "",
                "integration": "Y" if integ else "",
                "e2e": "Y" if e2e else "",
                "status": status,
            }
        )
    return rows


def write_csv(rows: list[dict[str, str]], out_path: Path) -> None:
    out_path.parent.mkdir(parents=True, exist_ok=True)
    fieldnames = ["AC_ID", "description", "REQ_ID", "unit", "integration", "e2e", "status"]
    with out_path.open("w", encoding="utf-8", newline="") as fh:
        writer = csv.DictWriter(fh, fieldnames=fieldnames)
        writer.writeheader()
        for row in rows:
            writer.writerow(row)


def parse_args(argv: Iterable[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="AC ↔ Test Case 矩阵生成器 (per docs/test-design.md §6.2.1)",
    )
    parser.add_argument(
        "--root",
        type=Path,
        default=ROOT_DEFAULT,
        help="仓库根目录 (默认: 脚本父目录)",
    )
    parser.add_argument(
        "--out",
        type=Path,
        default=None,
        help="输出 CSV 路径 (默认: <root>/docs/ac-test-matrix.csv)",
    )
    return parser.parse_args(list(argv))


def main(argv: Iterable[str]) -> int:
    args = parse_args(argv)
    root: Path = args.root
    out: Path = args.out or (root / "docs" / "ac-test-matrix.csv")
    requirements = root / "docs" / "requirements.md"
    reqs = extract_reqs(requirements)
    if not reqs:
        print(
            f"[warn] no REQ-* rows parsed from {requirements}; 输出仅含表头",
            file=sys.stderr,
        )
    test_files = find_test_files(root)
    rows = build_matrix(reqs, test_files)
    write_csv(rows, out)
    covered = sum(1 for r in rows if r["status"] == "covered")
    gap = len(rows) - covered
    print(
        f"[ok] AC 矩阵: {len(rows)} 行 (covered={covered}, gap={gap}, 测试文件={len(test_files)}) → {out}",
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
