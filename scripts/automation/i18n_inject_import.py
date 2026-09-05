#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
i18n_inject_import.py — 给用了 t.xxx 但没 import useTranslation 的文件注入 import.

规则: 检测 t.pageTitles / t.ariaLabels / t.placeholders / t.navModules 引用,
     在文件顶部 import { useTranslation } from "@/lib/i18n";
     并在 client component 顶部加 const { t } = useTranslation();
"""
from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
FRONTEND_SRC = ROOT / "frontend" / "src"

# t.xxx.yyy 引用模式
T_REF = re.compile(r"\bt\.(pageTitles|ariaLabels|placeholders|navModules|categoryNames)\b")

# 已有 import 模式
IMPORT_USE = re.compile(
    r'import\s+(?:\{[^}]*\buseTranslation\b[^}]*\}\s+from\s+["\']@/lib/i18n["\']|useTranslation\s+from\s+["\']@/lib/i18n["\'])'
)


def needs_inject(text: str) -> bool:
    if T_REF.search(text):
        return True
    return False


def already_imported(text: str) -> bool:
    return bool(IMPORT_USE.search(text))


def inject(path: Path) -> int:
    text = path.read_text(encoding="utf-8")
    if not needs_inject(text):
        return 0
    if already_imported(text):
        return 0
    # 找最后一行 import
    lines = text.split("\n")
    last_import = -1
    for i, line in enumerate(lines):
        if line.startswith("import ") or line.startswith("} from "):
            last_import = i
    if last_import < 0:
        # 没 import 行 — 在"use client" 后插
        for i, line in enumerate(lines):
            if line.strip() == '"use client";':
                last_import = i
                break
    if last_import < 0:
        return 0
    # 插入 import
    new_import = 'import { useTranslation } from "@/lib/i18n";'
    lines.insert(last_import + 1, new_import)
    # 在文件顶部 (在 "use client" 和 imports 之后) 加 const { t } = useTranslation();
    # 找插入位置: 在最后一行 import 后
    new_lines = "\n".join(lines) + "\n"
    # 找最后 import 位置
    last_import_after = -1
    for i, line in enumerate(lines):
        if line.startswith("import ") or line.startswith("} from "):
            last_import_after = i
    # 找第一个 export function / export const 位置 (组件)
    component_start = -1
    for i, line in enumerate(lines):
        if re.match(r"export\s+(default\s+)?(function|const)\s+[A-Z]", line.strip()):
            component_start = i
            break
    if component_start < 0:
        return 0
    # 在 component 起始函数体前注入 const { t } = useTranslation();
    # 找第一个 { (函数体开始)
    insert_at = component_start
    # 跳过 props 块
    for i in range(component_start, min(component_start + 30, len(lines))):
        if "{" in lines[i] and "}" not in lines[i]:
            insert_at = i
            break
        if "{" in lines[i] and "}" in lines[i] and lines[i].index("{") < lines[i].index("}"):
            insert_at = i
            break
    # 在 insert_at 行末插 const { t } = useTranslation();
    lines.insert(insert_at + 1, "  const { t } = useTranslation();")
    text = "\n".join(lines)
    path.write_text(text, encoding="utf-8")
    return 1


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--apply", action="store_true")
    args = ap.parse_args()
    try:
        sys.stdout.reconfigure(encoding="utf-8")
    except Exception:
        pass
    n = 0
    for tsx in sorted(FRONTEND_SRC.rglob("*.tsx")):
        if "__tests__" in tsx.parts or ".test." in tsx.name or "/lib/i18n/" in tsx.as_posix():
            continue
        if args.apply:
            c = inject(tsx)
            if c:
                print(f"[inject] {tsx.relative_to(ROOT).as_posix()}")
                n += c
        else:
            # dry run
            text = tsx.read_text(encoding="utf-8")
            if needs_inject(text) and not already_imported(text):
                print(f"[dry-run] {tsx.relative_to(ROOT).as_posix()}")
                n += 1
    print(f"\nTotal: {n} files need import")
    return 0


if __name__ == "__main__":
    sys.exit(main())
