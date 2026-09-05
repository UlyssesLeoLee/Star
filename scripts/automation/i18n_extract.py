#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
i18n_extract.py — 提取所有 page.tsx / 核心组件的硬编码 title/aria-label/placeholder,
生成字典初稿, 让 Mavis 一次补完, 避免手动复制 30+ 文件.

用法:
  python scripts/automation/i18n_extract.py --out dict_raw.json
  python scripts/automation/i18n_extract.py --out dict_raw.json --include-mocks  # 含 mocks
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
FRONTEND_SRC = ROOT / "frontend" / "src"

EXCLUDE_DIRS = {"__tests__", "lib/i18n"}

PROP_RE = re.compile(
    r'(?P<attr>(?:title|aria-label|placeholder|aria-labelledby|alt)\s*=\s*")'
    r'(?P<val>[^"\n]{2,200})'
    r'(")'
)
SINGLE_PROP_RE = re.compile(
    r"(?P<attr>(?:title|aria-label|placeholder|aria-labelledby|alt)\s*=\s*')"
    r"(?P<val>[^'\n]{2,200})"
    r"(')"
)


def is_excluded(p: Path) -> bool:
    return any(part in EXCLUDE_DIRS for part in p.parts) or ".test." in p.name


def scan() -> dict:
    out = {
        "page_titles": {},  # {page_route: {title, subtitle}}
        "aria_labels": {},  # {key: value}
        "placeholders": {},
        "registry_labels": {},  # {module_id: label}
    }

    # 1. nav registry
    reg = FRONTEND_SRC / "lib" / "nav" / "registry.ts"
    if reg.exists():
        text = reg.read_text(encoding="utf-8")
        m = re.compile(
            r"id:\s*['\"](?P<id>[\w-]+)['\"]\s*,\s*\n\s*label:\s*['\"](?P<label>[^'\"]+)['\"]"
        )
        for hit in m.finditer(text):
            out["registry_labels"][hit.group("id")] = hit.group("label")

    # 2. all tsx files
    for tsx in sorted(FRONTEND_SRC.rglob("*.tsx")):
        if is_excluded(tsx):
            continue
        try:
            text = tsx.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            continue
        rel = tsx.relative_to(ROOT).as_posix()

        # 收集 PageHeader title/subtitle
        for m in re.finditer(
            r"<PageHeader\b([^>]*?)(?:/>|>(?P<inner>.*?)</PageHeader>)",
            text,
            re.DOTALL,
        ):
            attrs = m.group(1)
            title = _extract_attr(attrs, "title")
            subtitle = _extract_attr(attrs, "subtitle")
            if title:
                # 推断 route from file path
                route = _file_to_route(tsx)
                key = route or rel
                out["page_titles"].setdefault(key, {"title": title})
                if subtitle:
                    out["page_titles"][key]["subtitle"] = subtitle

        # aria-label
        for pat in (PROP_RE, SINGLE_PROP_RE):
            for m in pat.finditer(text):
                attr = m.group("attr").split("=")[0].strip()
                val = m.group("val").strip()
                if not _is_meaningful(val):
                    continue
                key = f"{rel}::{attr}::{m.start()}"
                out["aria_labels"][key] = {"file": rel, "attr": attr, "value": val}

    return out


def _extract_attr(attrs: str, name: str) -> str | None:
    m = re.search(rf'{name}\s*=\s*"([^"]+)"', attrs)
    if m:
        return m.group(1)
    m = re.search(rf"{name}\s*=\s*'([^']+)'", attrs)
    if m:
        return m.group(1)
    return None


def _file_to_route(p: Path) -> str | None:
    if "app" not in p.parts:
        return None
    app_idx = p.parts.index("app")
    rel = Path(*p.parts[app_idx + 1 :])
    # 转 app/notification/page.tsx -> /notification
    parts = list(rel.parts)
    if parts[-1] != "page.tsx":
        return None
    parts = parts[:-1]
    if parts:
        return "/" + "/".join(parts)
    return "/"


def _is_meaningful(s: str) -> bool:
    s = s.strip()
    if not s:
        return False
    if s.startswith(("bg-", "text-", "border-", "flex", "grid", "p-", "m-", "w-", "h-")):
        return False
    if re.match(r"^[a-z]+(-[a-z0-9]+)*$", s) and s.islower():
        return False
    if not re.search(r"[A-Za-z]{2,}", s):
        return False
    if not re.search(r"[A-Z][a-z]+|[a-z]+ [a-z]+", s):
        return False
    return True


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", default="i18n_extract.json")
    args = parser.parse_args()
    try:
        sys.stdout.reconfigure(encoding="utf-8")
    except Exception:
        pass
    data = scan()
    Path(args.out).write_text(json.dumps(data, ensure_ascii=False, indent=2), encoding="utf-8")
    print(f"Wrote {args.out}")
    print(f"  page_titles: {len(data['page_titles'])}")
    print(f"  aria_labels: {len(data['aria_labels'])}")
    print(f"  registry_labels: {len(data['registry_labels'])}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
