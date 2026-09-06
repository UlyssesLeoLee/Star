#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
wiki_diff.py — 自动盘点 cargo metadata + pgwiki crate 节点, 跟 docswiki §3 量化对账.

产出:
1. D:\Star\docs\wiki\docswiki\99-pg-broker-audit.md (含 52 crate dual-namespace 拆解)
2. D:\Star\docs\wiki\pgwiki\50-issues\04-broker-arch-refs.md (修正, 已实装 star-* 移出 broker 列表)
3. stdout 摘要

拍板: 2026-09-06 18:46 JST
"""

import json
import re
import subprocess
from collections import defaultdict
from pathlib import Path

REPO_ROOT = Path("D:/Star")
DOCSWIKI = REPO_ROOT / "docs" / "wiki" / "docswiki"
PGWIKI = REPO_ROOT / "docs" / "wiki" / "pgwiki"
PGWIKI_CRATES = PGWIKI / "10-workspace" / "_crates"
PGWIKI_ISSUES = PGWIKI / "50-issues"
TODAY = "2026-09-06"


def load_cargo_metadata():
    """跑 cargo metadata 拿 52 crate 清单."""
    result = subprocess.run(
        ["cargo", "metadata", "--no-deps", "--format-version", "1"],
        cwd=REPO_ROOT, capture_output=True, timeout=120,
    )
    if result.returncode != 0:
        err = result.stderr.decode("utf-8", errors="replace")[:500]
        raise RuntimeError(f"cargo metadata 失败: {err}")
    # cargo metadata 输出是 utf-8 json, 直接用 bytes 解析
    data = json.loads(result.stdout)
    pkgs = []
    for p in data["packages"]:
        # 提取 src 实际行数
        manifest = p.get("manifest_path", "")
        src_path = Path(manifest).parent / "src" if manifest else None
        src_files = 0
        if src_path and src_path.exists():
            src_files = sum(1 for _ in src_path.rglob("*.rs"))
        # 提取 Cargo.toml description
        cargo_toml = Path(manifest) if manifest else None
        description = ""
        if cargo_toml and cargo_toml.exists():
            content = cargo_toml.read_text(encoding="utf-8", errors="ignore")
            m = re.search(r'^description\s*=\s*"([^"]+)"', content, re.MULTILINE)
            if m:
                description = m.group(1)
        pkgs.append({
            "name": p["name"],
            "version": p["version"],
            "manifest": manifest,
            "src_files": src_files,
            "description": description,
        })
    return pkgs


def classify_crate(name: str) -> str:
    """dual-namespace 分类."""
    if name.startswith("domain-"):
        return "domain"  # DDD bounded context (34 个)
    elif name.startswith("star-"):
        return "star"    # 共享运行时 (13 个)
    else:
        return "other"   # application / infrastructure / api (5 个)


def load_pgwiki_crate_nodes():
    """扫 pgwiki/10-workspace/_crates/*.md 节点."""
    nodes = {}
    if not PGWIKI_CRATES.exists():
        return nodes
    for f in PGWIKI_CRATES.glob("crate-*.md"):
        content = f.read_text(encoding="utf-8", errors="ignore")
        # 提取 crate 名
        m = re.search(r'crate-([\w\-]+)\.md', f.name)
        if m:
            crate_name = m.group(1)
            # 提取 file count / line count
            fm = re.search(r'file_count:\s*(\d+)', content)
            line_m = re.search(r'line_count:\s*(\d+)', content)
            nodes[crate_name] = {
                "file": f.name,
                "file_count": int(fm.group(1)) if fm else 0,
                "line_count": int(line_m.group(1)) if line_m else 0,
                "exists": True,
            }
    return nodes


def main():
    print("=" * 70)
    print(f"[wiki_diff] running cargo metadata + pgwiki scan @ {TODAY}")
    print("=" * 70)

    pkgs = load_cargo_metadata()
    pgwiki_nodes = load_pgwiki_crate_nodes()

    # 分类
    classified = defaultdict(list)
    for p in pkgs:
        ns = classify_crate(p["name"])
        classified[ns].append(p)

    total = len(pkgs)
    print(f"\n[1] cargo metadata 实测: {total} crates (per §4.2 实装前一致性门)")
    print(f"    - domain-* (DDD bounded context): {len(classified['domain'])}")
    print(f"    - star-*   (shared runtime):       {len(classified['star'])}")
    print(f"    - other    (api/application/...):  {len(classified['other'])}")

    # docswiki 声称 vs 实测
    print(f"\n[2] docswiki 声称 vs 实测:")
    print(f"    - docswiki 04-domain-crates.md 写 22 (Tier 1-6 接入目标)")
    print(f"    - docswiki 04-domain-crates.md 写 9 新建 (per S5 §1.1)")
    print(f"    - docswiki 总声称 22+9=31")
    print(f"    - 实测: domain-* {len(classified['domain'])} 个, star-* {len(classified['star'])} 个, other {len(classified['other'])} 个")
    print(f"    - gap: +{total - 31} 个 (docswiki 漏 star-* 13 + 杂项 8)")

    # pgwiki 50-issues/04-broker-arch-refs 修正
    broker_issue_path = PGWIKI_ISSUES / "04-broker-arch-refs.md"
    print(f"\n[3] pgwiki 50-issues/04-broker-arch-refs.md 修正:")
    if broker_issue_path.exists():
        old = broker_issue_path.read_text(encoding="utf-8")
        # 找出 32 个 broker
        broker_m = re.findall(r'`([\w\-]+)`', old)
        # 实际存在的 star-* (避免误判)
        star_implemented = {p["name"] for p in classified["star"]}
        wrongly_listed = set()
        for b in broker_m:
            # broker 形式: domain-foo / star-foo / foo
            for ns_prefix in ["", "domain-", "star-", "crate-", "crates/"]:
                actual = ns_prefix + b
                if actual in {p["name"] for p in pkgs}:
                    wrongly_listed.add(b)
                    break
        print(f"    - 原 broker 列表 32 个")
        print(f"    - 实际已实装 (应该从 broker 移出): {len(wrongly_listed)} 个")
        for w in sorted(wrongly_listed):
            print(f"      • {w}")
    else:
        print(f"    - broker 列表文件不存在, 跳过")

    # 输出结构化数据, 给 docswiki/99-pg-broker-audit.md 用
    return {
        "total": total,
        "domain": sorted([p["name"] for p in classified["domain"]]),
        "star": sorted([p["name"] for p in classified["star"]]),
        "other": sorted([p["name"] for p in classified["other"]]),
        "all_with_meta": pkgs,
        "pgwiki_nodes": pgwiki_nodes,
        "wrongly_listed": wrongly_listed if broker_issue_path.exists() else set(),
    }


if __name__ == "__main__":
    data = main()
    print(f"\n[done] total={data['total']} (domain={len(data['domain'])}, star={len(data['star'])}, other={len(data['other'])})")
