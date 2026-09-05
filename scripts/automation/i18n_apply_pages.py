#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
i18n_apply_pages.py — 批量把 page.tsx 里的 PageHeader title/subtitle 改成 i18n 引用.

规则 (per 2026-09-05 拍板 C):
  原:   <PageHeader title="Notifications" subtitle="..." />
  改:   <PageHeader title={t.pageTitles["/notification"].title} subtitle={t.pageTitles["/notification"].subtitle} />

对 aria-label / placeholder / 单点 title 改成 t.ariaLabels.xxx / t.placeholders.xxx (按业务语义)

输出 dry-run 模式默认, 加 --apply 真正写文件.
"""
from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
FRONTEND_SRC = ROOT / "frontend" / "src"

# 业务语义映射 (从 audit 报告里抽出来)
TITLE_MAP = {
    "Notifications": "t.pageTitles['/notification'].title",
    "Inbox": "t.pageTitles['/inbox'].title",
    "Issues": "t.pageTitles['/issues'].title",
    "Agents": "t.pageTitles['/agents'].title",
    "Analytics": "t.pageTitles['/analytics'].title",
    "Settings": "t.pageTitles['/settings'].title",
    "Agent Sessions": "t.pageTitles['/agent'].title",
    "Agent": "t.pageTitles['/agent-view'].title",
    "Audit": "t.pageTitles['/audit'].title",
    "Automation": "t.pageTitles['/automation'].title",
    "Board": "t.pageTitles['/board'].title",
    "Collaboration": "t.pageTitles['/collaboration'].title",
    "Comments": "t.pageTitles['/comment'].title",
    "Context": "t.pageTitles['/context'].title",
    "Development": "t.pageTitles['/development'].title",
    "Feedback Inbox": "t.pageTitles['/feedback'].title",
    "Identity & Access": "t.pageTitles['/identity'].title",
    "Integrations": "t.pageTitles['/integration'].title",
    "Local Runtime": "t.pageTitles['/local-runtime'].title",
    "Permission": "t.pageTitles['/permission'].title",
    "Planning": "t.pageTitles['/planning'].title",
    "Projects": "t.pageTitles['/projects'].title",
    "Relation": "t.pageTitles['/relation'].title",
    "远程控制": "t.pageTitles['/remote'].title",
    "SCM": "t.pageTitles['/scm'].title",
    "Search": "t.pageTitles['/search'].title",
    "Validation": "t.pageTitles['/validation'].title",
    "Workflows": "t.pageTitles['/workflow'].title",
    "Worktree": "t.pageTitles['/worktree'].title",
    "Agent Windows": "t.pageTitles['/agent-windows'].title",
    "API Keys": "t.pageTitles['/api-keys'].title",
    "CLI Profiles": "t.pageTitles['/cli-profiles'].title",
    "Credentials": "t.pageTitles['/credentials'].title",
}

ARIA_MAP = {
    "Primary navigation": "t.ariaLabels.primaryNav",
    "Star home": "t.ariaLabels.starHome",
    "Settings": "t.ariaLabels.settings",
    "Open App Matrix (All Modules)": "t.ariaLabels.openAppMatrix",
    "Open App Matrix": "t.ariaLabels.openAppMatrixShort",
    "Open command bar (\u2318K)": "t.ariaLabels.openCommandBar",
    "Search (Cmd+K)": "t.ariaLabels.searchCmdK",
    "Notifications": "t.ariaLabels.notifications",
    "Mobile primary navigation": "t.ariaLabels.mobilePrimaryNav",
    "More navigation": "t.ariaLabels.moreNav",
    "Sidebar scope": "t.ariaLabels.sidebarScope",
    "Section navigation": "t.ariaLabels.sectionNav",
    "Subnav items": "t.ariaLabels.subnavItems",
    "Install Star App": "t.ariaLabels.installStarApp",
    "Dismiss": "t.ariaLabels.dismiss",
    "Issue view tabs": "t.ariaLabels.issueViewTabs",
    "Toggle search": "t.ariaLabels.toggleSearch",
    "Create new issue": "t.ariaLabels.createNewIssue",
    "Dismiss new issue": "t.ariaLabels.dismissNewIssue",
    "Issue detail": "t.ariaLabels.issueDetail",
    "Collapse": "t.ariaLabels.collapse",
    "Expand": "t.ariaLabels.expand",
    "Close detail": "t.ariaLabels.closeDetail",
    "worktree": "t.ariaLabels.worktree",
    "refresh": "t.ariaLabels.refresh",
    "Project switcher": "t.ariaLabels.projectSwitcher",
    "project switcher": "t.ariaLabels.projectSwitcher",
    "Cost trend (mock)": "t.ariaLabels.costTrend",
    "已配置 API Key": "t.ariaLabels.apiKeyConfigured",
    "未配置 API Key": "t.ariaLabels.apiKeyMissing",
    "Select (V)": "t.ariaLabels.canvasSelect",
    "Pan (H)": "t.ariaLabels.canvasPan",
    "Zoom in (+)": "t.ariaLabels.canvasZoomIn",
    "Zoom out (-)": "t.ariaLabels.canvasZoomOut",
    "Fit to content (1)": "t.ariaLabels.canvasFit",
    "Delete": "t.ariaLabels.canvasDelete",
    "Back": "t.ariaLabels.canvasBack",
    "Tactical Link Active": "t.ariaLabels.tacticalLinkActive",
}


def transform_file(path: Path) -> tuple[str, int]:
    """返回 (新内容, 替换次数)"""
    text = path.read_text(encoding="utf-8")
    n = 0

    # PageHeader title="X"  ->  title={t.pageTitles["/route"].title}
    for k, v in TITLE_MAP.items():
        # escape k for regex
        k_esc = re.escape(k)
        pattern = re.compile(rf'title="{k_esc}"')
        repl = f'title={{{v}}}'
        new, c = pattern.subn(repl, text)
        if c:
            text = new
            n += c

    # aria-label / placeholder / title (单点) aria-label="X"  ->  aria-label={t.ariaLabels.xxx}
    for k, v in ARIA_MAP.items():
        k_esc = re.escape(k)
        for attr in ("aria-label", "title"):
            pattern = re.compile(rf'{attr}="{k_esc}"')
            repl = f'{attr}={{{v}}}'
            new, c = pattern.subn(repl, text)
            if c:
                text = new
                n += c

    return text, n


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--apply", action="store_true", help="write changes to disk")
    args = ap.parse_args()
    try:
        sys.stdout.reconfigure(encoding="utf-8")
    except Exception:
        pass

    total = 0
    files = 0
    for tsx in sorted(FRONTEND_SRC.rglob("*.tsx")):
        if "__tests__" in tsx.parts or ".test." in tsx.name or "/lib/i18n/" in tsx.as_posix():
            continue
        new_text, n = transform_file(tsx)
        if n > 0:
            files += 1
            total += n
            if args.apply:
                tsx.write_text(new_text, encoding="utf-8")
                print(f"[apply] {tsx.relative_to(ROOT).as_posix()}: {n}")
            else:
                print(f"[dry-run] {tsx.relative_to(ROOT).as_posix()}: {n}")

    print(f"\nTotal: {total} replacements in {files} files")
    return 0


if __name__ == "__main__":
    sys.exit(main())
