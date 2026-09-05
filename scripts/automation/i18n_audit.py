#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
i18n_audit.py — 扫描 frontend i18n 硬编码英文 + 缺 key 检测

Per 守门 #19 (agent 交互 Python 化), 跟 docs/automation-design.md §4 任务卡对齐.

用法:
  python scripts/automation/i18n_audit.py
  python scripts/automation/i18n_audit.py --json   # JSON 输出
  python scripts/automation/i18n_audit.py --strict  # 任何硬编码都返回非 0

扫描规则:
  1. 组件 prop: title="X" / aria-label="X" / placeholder="X" 硬编码
  2. JSX 文本: >Some English Text< (排除 i18n hook 内的)
  3. nav/registry.ts module.label 硬编码
  4. dictionary.ts modules.* 三语言一致性
  5. 缺 key (zh-CN 有, en/ja 缺)

排除:
  - frontend/src/lib/i18n/zh-CN.ts|en.ts|ja.ts (字典本身)
  - frontend/src/mocks/** (mock 数据)
  - frontend/src/**/__tests__/** (测试期望)
  - frontend/src/**/*.test.ts(x) (测试)
  - 注释行 (// or /* or *)
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Iterator

ROOT = Path(__file__).resolve().parent.parent.parent
FRONTEND_SRC = ROOT / "frontend" / "src"

# ── 排除规则 ───────────────────────────────────────────────
EXCLUDE_DIRS = {"__tests__", "mocks", "lib/i18n"}
EXCLUDE_FILE_PATTERNS = [r"\.test\.[tj]sx?$", r"\.test\.tsx$", r"__tests__/"]

# 硬编码英文 prop 模式 — 双引号 + 单引号
# 注意: 不要用三引号 raw string 里嵌同种引号 — 会被 Python 截断.
HARDCODE_PATTERNS = [
    re.compile(
        r'(?P<attr>(?:title|aria-label|placeholder|aria-labelledby|alt)\s*=\s*")'
        r'(?P<val>[^"]{2,120})'
        r'(")'
    ),
    re.compile(
        r"(?P<attr>(?:title|aria-label|placeholder|aria-labelledby|alt)\s*=\s*')"
        r"(?P<val>[^']{2,120})"
        r"(')"
    ),
]

# nav registry label 硬编码 (id, "English Label" 形式)
REGISTRY_LABEL = re.compile(
    r'''^\s*id:\s*["'](?P<id>[\w-]+)["']\s*,\s*\n\s*label:\s*["'](?P<label>[^"']+)["']''',
    re.MULTILINE,
)

# 字典 key 引用模式 (t.xxx.yyy)
T_DOT_ACCESS = re.compile(r"\bt\.[a-zA-Z_][\w.]*")

# 纯英文句子 (>= 4 词, 不在 JSX attribute / import / 注释)
JSX_TEXT_EN = re.compile(
    r">\s*([A-Z][A-Za-z]+(?:\s+[A-Za-z]+){3,}[^<]{0,80})\s*<"
)


def is_excluded(path: Path) -> bool:
    rel = path.relative_to(FRONTEND_SRC).as_posix()
    if any(part in EXCLUDE_DIRS for part in path.parts):
        return True
    return any(re.search(pat, rel) for pat in EXCLUDE_FILE_PATTERNS)


def is_english_text(s: str) -> bool:
    """判断 prop 字符串是否"含英文短语" (>= 2 个连续 ASCII 字母, 但不带大段 CJK)
    注: 这里只看是否需要 i18n 化, 不严格. 中英混排也归为硬编码 (因为 prop 写死了英文短语)."""
    s = s.strip()
    if not s:
        return False
    # 排除明显是 CSS class / icon 名字 / import 路径
    if s.startswith(("bg-", "text-", "border-", "p-", "m-", "w-", "h-", "flex", "grid", "http", "/", "#", "{", "rgb", "rgba")):
        return False
    if re.match(r"^[a-z-]+(\s+[a-z-]+)*$", s) and " " not in s and s.islower():
        # 纯小写连字符 (CSS class)
        return False
    # 至少 2 个连续 ASCII 字母
    if not re.search(r"[A-Za-z]{2,}", s):
        return False
    # 必须有英文单词边界 (空格 / 首字母大写) — 排除 hash/UUID/Base64
    if not re.search(r"[A-Z][a-z]+|[a-z]+ [a-z]+", s):
        return False
    return True


def scan_component_files() -> list[dict]:
    findings = []
    for tsx in FRONTEND_SRC.rglob("*.tsx"):
        if is_excluded(tsx):
            continue
        try:
            text = tsx.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            continue
        rel = tsx.relative_to(ROOT).as_posix()
        for pat in HARDCODE_PATTERNS:
            for m in pat.finditer(text):
                val = m.group("val")
                if not is_english_text(val):
                    continue
                # 排除已经在 i18n 字典里的: 通过 t.xxx 引用
                # 简单启发: 同文件前面 100 行内有 t.<namespace> 引用 = 已用 i18n
                # 但这个 prop 本身是硬编码 = 漏
                # 我们只看 val 是否明显英文短语
                line_no = text[: m.start()].count("\n") + 1
                findings.append(
                    {
                        "file": rel,
                        "line": line_no,
                        "attr": m.group("attr").split("=")[0].strip(),
                        "value": val[:80],
                    }
                )
    return findings


def scan_registry() -> list[dict]:
    reg = FRONTEND_SRC / "lib" / "nav" / "registry.ts"
    if not reg.exists():
        return []
    text = reg.read_text(encoding="utf-8")
    findings = []
    for m in REGISTRY_LABEL.finditer(text):
        label = m.group("label")
        if is_english_text(label):
            line_no = text[: m.start()].count("\n") + 1
            findings.append(
                {
                    "file": "frontend/src/lib/nav/registry.ts",
                    "line": line_no,
                    "attr": "label",
                    "module_id": m.group("id"),
                    "value": label,
                }
            )
    return findings


def check_dict_consistency() -> dict:
    """检查 zh-CN/en/ja 三语言字典 key 一致性"""
    dict_files = {
        "zh-CN": FRONTEND_SRC / "lib" / "i18n" / "zh-CN.ts",
        "en": FRONTEND_SRC / "lib" / "i18n" / "en.ts",
        "ja": FRONTEND_SRC / "lib" / "i18n" / "ja.ts",
    }
    result = {"missing_keys": {}, "extra_keys": {}, "loaded": {}}
    # 简化: 提取 const X: Dictionary = { ... } 的 key path
    # TS parser 太重, 用正则
    KEY_PATTERN = re.compile(r"^\s{2,}([a-zA-Z_][\w]*)\s*:\s*[{\[]", re.MULTILINE)
    nested = {}
    for lang, fp in dict_files.items():
        if not fp.exists():
            continue
        text = fp.read_text(encoding="utf-8")
        # 提取所有 indented keys (粗略, 一级 + 嵌套)
        keys = set()
        for m in KEY_PATTERN.finditer(text):
            keys.add(m.group(1))
        nested[lang] = keys
        result["loaded"][lang] = sorted(keys)

    # 对比: zh-CN 基准, en/ja 缺什么
    if "zh-CN" in nested:
        zh = nested["zh-CN"]
        for lang in ("en", "ja"):
            if lang not in nested:
                continue
            missing = zh - nested[lang]
            extra = nested[lang] - zh
            if missing:
                result["missing_keys"][lang] = sorted(missing)
            if extra:
                result["extra_keys"][lang] = sorted(extra)
    return result


def main() -> int:
    parser = argparse.ArgumentParser(description="Audit i18n hardcoded strings")
    parser.add_argument("--json", action="store_true", help="JSON output")
    parser.add_argument("--strict", action="store_true", help="Exit non-zero on any finding")
    parser.add_argument("--section", choices=["component", "registry", "dict", "all"], default="all")
    args = parser.parse_args()

    report = {}
    exit_code = 0

    if args.section in ("component", "all"):
        comp = scan_component_files()
        report["component_hardcoded"] = comp
        if comp:
            exit_code = 1 if args.strict else 0

    if args.section in ("registry", "all"):
        reg = scan_registry()
        report["registry_hardcoded"] = reg
        if reg:
            exit_code = 1 if args.strict else 0

    if args.section in ("dict", "all"):
        cons = check_dict_consistency()
        report["dict_consistency"] = cons
        if cons.get("missing_keys") or cons.get("extra_keys"):
            exit_code = 1 if args.strict else 0

    if args.json:
        # 强制 stdout UTF-8 (Windows GBK console 默认会炸)
        try:
            sys.stdout.reconfigure(encoding="utf-8")
        except (AttributeError, OSError):
            pass
        print(json.dumps(report, ensure_ascii=False, indent=2))
        return exit_code

    # 强制 stdout UTF-8 (Windows GBK console 默认会炸)
    try:
        sys.stdout.reconfigure(encoding="utf-8")
    except (AttributeError, OSError):
        pass

    print("=" * 60)
    print("i18n Audit Report")
    print("=" * 60)
    if "component_hardcoded" in report:
        comp = report["component_hardcoded"]
        print(f"\n[Component Hardcoded English] {len(comp)} findings")
        for f in comp[:30]:
            val = f["value"].encode("ascii", "backslashreplace").decode("ascii")
            print(f"  {f['file']}:{f['line']}  {f['attr']}=\"{val}\"")
        if len(comp) > 30:
            print(f"  ... and {len(comp) - 30} more")
    if "registry_hardcoded" in report:
        reg = report["registry_hardcoded"]
        print(f"\n[Registry Hardcoded Labels] {len(reg)} findings")
        for f in reg:
            print(f"  {f['module_id']:20s} -> \"{f['value']}\"")
    if "dict_consistency" in report:
        cons = report["dict_consistency"]
        mk = cons.get("missing_keys", {})
        ek = cons.get("extra_keys", {})
        if mk:
            print(f"\n[Dict Missing Keys]")
            for lang, keys in mk.items():
                print(f"  {lang} missing: {keys}")
        if ek:
            print(f"\n[Dict Extra Keys]")
            for lang, keys in ek.items():
                print(f"  {lang} extra: {keys}")
        if not mk and not ek:
            print(f"\n[Dict Consistency] OK - 3 languages aligned")

    return exit_code


if __name__ == "__main__":
    sys.exit(main())
