#!/usr/bin/env python3
"""为 MOC 中已列出但文件缺失的 view 生成占位卡。

背景：2026-10-04 把 pgwiki/30-architecture/MOC.md 的 view 索引从 5 补到 16，
但只实际创建了 2 个新 view 文档（dev-baseline / engineering-run-directory），
其余 9 个 [[view-...]] 链接指向不存在的文件 —— Obsidian 里会显示为断链。

做法：读取 docs/architecture/<view>/ 的真实文件清单，为缺失的生成卡片。
内容全部来自实测 ls-tree，不编造描述。

fail-closed：找不到对应架构目录就 exit 2，不生成空卡。
"""

from __future__ import annotations

import subprocess
import sys
from datetime import datetime, timezone, timedelta
from pathlib import Path

REPO = Path(r"C:\Users\leo19\orca\workspaces\Star\dev-2")
VIEWS = REPO / "docs" / "wiki" / "pgwiki" / "30-architecture" / "views"
ARCH = REPO / "docs" / "architecture"

# MOC 中列出但文件缺失的 view -> 一句话主题（来自 docs/architecture 实测文件名）
THEMES = {
    "view-2026-09-03-arg": "唯一完整走完 SRS→BD→DD→RACI→分阶段实现的 view；14 篇",
    "view-2026-09-07-exclusion-idempotency": "排除法与幂等性；3 篇",
    "view-2026-09-09-subtask-binding": "子任务绑定契约；3 篇",
    "view-2026-09-22-aci-mock-interface": "ACI mock 接口设计分析；1 篇",
    "view-2026-09-22-mock-switches": "mock 开关机制设计分析；1 篇",
    "view-2026-09-28-upgrade": "Rust→WASM 前端内存研究；1 篇",
    "view-2026-09-29-upgrade": "端侧 Rust app 研究 + WASM P0P1 落地；2 篇",
    "view-2026-09-30-upgrade": "Tauri 桌面端全链路 P0–P10；9 篇",
    "view-2026-10-01-upgrade": "Session Memory 协议 + Tauri PoC 索引；2 篇",
}

JST = timezone(timedelta(hours=9))
NOW = datetime.now(JST).strftime("%Y-%m-%dT%H:%M:%S+09:00")


def arch_files(view: str) -> list[str]:
    d = ARCH / view
    if not d.is_dir():
        return []
    return sorted(
        p.relative_to(d).as_posix() for p in d.rglob("*.md")
    )


def main() -> int:
    made: list[str] = []
    skipped: list[str] = []

    for stem, theme in THEMES.items():
        view = stem.replace("view-", "")
        out = VIEWS / f"{stem}.md"
        if out.exists():
            skipped.append(f"{stem} (exists)")
            continue
        files = arch_files(view)
        if not files:
            # fail-closed: 不为不存在的目录生成空卡
            print(f"[FAIL-CLOSED] {view}: docs/architecture/{view}/ 不存在或无 md")
            return 2
        body = [
            "---",
            f'title: "view: {view}"',
            f'generated: "{NOW}"',
            'node_type: "arch-view"',
            f'view_name: "{view}"',
            "---",
            "",
            f"# view: {view}",
            "",
            f"**架构 view**: `{view}`",
            f"**主题**: {theme}",
            f"**根**: `docs/architecture/{view}/`",
            f"**文档数**: {len(files)}",
            "",
            "## 文件",
            "",
        ]
        body += [f"- `{f}`" for f in files]
        body += [
            "",
            "## 已知缺口",
            "",
            "- 本卡为索引卡，仅列出文件清单；内容摘要见 GitHub wiki 的对应页面",
            "- 文件清单于 2026-10-04 对 `dev@c1ce1630` 工作区实测枚举",
            "",
        ]
        out.write_text("\n".join(body), encoding="utf-8")
        made.append(f"{stem}.md ({len(files)} files)")

    print(f"[created] {len(made)}")
    for m in made:
        print(f"  + {m}")
    if skipped:
        print(f"[skipped] {len(skipped)}: {skipped}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
