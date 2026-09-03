#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
kanban-vmodel-jp Sprint 视图验证脚本 · [M] 档
==============================================

目的:
  验证 Sprint 视图 P1 阶段落地完整性。不生成代码 (代码由 Edit 工具落地),
  而是解析产物文件, 检查关键标记是否存在, 避免 agent 漏写函数 / 漏改 HTML / 漏改 CSS。

用法:
  python scripts/automation/kanban_sprint_gen.py            # 默认验证
  python scripts/automation/kanban_sprint_gen.py --strict   # 严格模式 (未通过即 exit 1)
  python scripts/automation/kanban_sprint_gen.py --json     # JSON 输出

检查项 (per docs/briefs/kanban-sprint-view-001.md §3 P1 验收):
  1. app.js 必含 Sprint 函数 (10+ 项)
  2. app.js 必含 vmodel-sprints-v1 localStorage key
  3. app.js 必含 setView('sprint') 路由
  4. index.html 必含 data-view="sprint" 区块 + .seg__btn[data-view="sprint"] tab
  5. index.html 必含 sprintCreateBtn / sprintHeader / sprintBoard / sprintList 元素 id
  6. styles.css 必含 .sprint / .sprint-header / .sprint-modal / .plan-grid 等 12+ class

退出码:
  0 = 全部通过
  1 = 有未通过项 (--strict 时)
  2 = 解析错误 (文件不存在 / 编码错误)
"""
import argparse
import json
import sys
from pathlib import Path
from typing import List, Tuple

# Force UTF-8 stdout on Windows (PowerShell 5.1 defaults to GBK)
try:
    sys.stdout.reconfigure(encoding='utf-8')
    sys.stderr.reconfigure(encoding='utf-8')
except Exception:
    pass

# ----- 配置 -----
REPO_ROOT = Path(__file__).resolve().parents[2]  # scripts/automation -> repo root
DELIVERABLE = REPO_ROOT / 'deliverables' / 'kanban-vmodel-jp'

APP_JS = DELIVERABLE / 'app.js'
INDEX_HTML = DELIVERABLE / 'index.html'
STYLES_CSS = DELIVERABLE / 'styles.css'

# ----- 检查项定义 -----
APP_JS_CHECKS: List[Tuple[str, str, bool]] = [
    # (description, pattern, required)
    ('Sprint 存储 key 常量', r"SPRINT_STORAGE_KEY\s*=\s*['\"]vmodel-sprints-v1['\"]", True),
    ('state.sprints 字段', r"state\.sprints\s*[:=]", True),
    ('getActiveSprint 函数', r"function\s+getActiveSprint", True),
    ('sprintCapacity 函数', r"function\s+sprintCapacity", True),
    ('renderSprint 函数', r"function\s+renderSprint", True),
    ('renderSprintHeader 函数', r"function\s+renderSprintHeader", True),
    ('renderSprintBoard 函数', r"function\s+renderSprintBoard", True),
    ('renderSprintList 函数', r"function\s+renderSprintList", True),
    ('openSprintEditModal 函数', r"function\s+openSprintEditModal", True),
    ('openSprintPlanModal 函数', r"function\s+openSprintPlanModal", True),
    ('startSprint 函数', r"function\s+startSprint", True),
    ('completeSprint 函数', r"function\s+completeSprint", True),
    ('cancelSprint 函数', r"function\s+cancelSprint", True),
    ('addToSprint 函数', r"function\s+addToSprint", True),
    ('removeFromSprint 函数', r"function\s+removeFromSprint", True),
    ('returnSprintTasksToBacklog 函数 (Jira 設計)', r"function\s+returnSprintTasksToBacklog", True),
    ('setView 路由 sprint', r"if\s*\(\s*v\s*===\s*['\"]sprint['\"]\s*\)\s*renderSprint", True),
    ('save 持久化 sprints', r"store\.save\(SPRINT_STORAGE_KEY", True),
    ('init 同步 activeSprintId', r"state\.activeSprintId\s*=\s*activeSp", True),
    ('exportJSON 包含 sprints', r"sprints:\s*state\.sprints", True),
    ('sprintCreateBtn 事件绑定', r"sprintCreateBtn[\s\S]{0,200}addEventListener", True),
    # Jira 設計: Backlog first (P1 v0.2 追加)
    ('addToSprint 校验 backlog 状态', r"t\.status\s*!==\s*['\"]backlog['\"]", True),
    ('removeFromSprint 重置 status=backlog', r"if\s*\(\s*t\s*\)\s*t\.status\s*=\s*['\"]backlog['\"]", True),
    ('completeSprint 未完了 → backlog (onlyIncomplete)', r"returnSprintTasksToBacklog\(\s*s\s*,\s*\{\s*onlyIncomplete", True),
    ('cancelSprint 全件 → backlog', r"const\s+returned\s*=\s*returnSprintTasksToBacklog\(\s*s\s*\)", True),
    ('Sprint 計画 modal backlog filter (status=backlog)', r"t\.status\s*===\s*['\"]backlog['\"]", True),
    ('Sprint 計画 hint "Jira 設計"', r"Jira\s*設計", True),
]

INDEX_HTML_CHECKS: List[Tuple[str, str, bool]] = [
    ('Sprint tab 按钮', r'data-view="sprint"', True),
    ('Sprint 视图容器', r'id="sprintView"', True),
    ('Sprint header 容器', r'id="sprintHeader"', True),
    ('Sprint board 容器', r'id="sprintBoard"', True),
    ('Sprint list 容器', r'id="sprintList"', True),
    ('Sprint sidebar', r'class="sprint-sidebar"', True),
    ('Sprint 新規按钮', r'id="sprintCreateBtn"', True),
    ('Sprint edit modal', r'id="sprintEditModal"', True),
    ('Sprint plan modal', r'id="sprintPlanModal"', True),
    # P2 追加
    ('Sprint metrics panel (P2)', r'id="sprintMetrics"', True),
]

STYLES_CSS_CHECKS: List[Tuple[str, str, bool]] = [
    ('.sprint 容器', r'\.sprint\s*\{', True),
    ('.sprint-body 网格', r'\.sprint-body\s*\{', True),
    ('.sprint-header 样式', r'\.sprint-header\s*\{', True),
    ('.sprint-stat 样式', r'\.sprint-stat\s*\{', True),
    ('.sprint-bar 进度条', r'\.sprint-bar\s*\{', True),
    ('.sprint-status-badge', r'\.sprint-status-badge', True),
    ('.sprint-empty 空状态', r'\.sprint-empty\s*\{', True),
    ('.sprint-sidebar', r'\.sprint-sidebar\s*\{', True),
    ('.sprint-list', r'\.sprint-list\s*\{', True),
    ('.sprint-item', r'\.sprint-item\s*\{', True),
    ('.sprint-modal', r'\.sprint-modal\s*\{', True),
    ('.plan-grid', r'\.plan-grid\s*\{', True),
    ('.plan-task', r'\.plan-task\s*\{', True),
    ('.form-row', r'\.form-row\s*\{', True),
    ('响应式 1200px', r'@media\s*\(max-width:\s*1200px\)', True),
    # Jira 設計 P1 v0.2 追加
    ('.plan-hint Backlog 提示', r'\.plan-hint\s*\{', True),
    ('.plan-warn 警告', r'\.plan-warn\s*\{', True),
    ('.plan-list__empty', r'\.plan-list__empty\s*\{', True),
]


def check_file(path: Path, checks: List[Tuple[str, str, bool]], label: str) -> Tuple[int, int, List[dict]]:
    """Check a file against a list of (desc, pattern, required) tuples. Returns (passed, total, details)."""
    if not path.exists():
        return 0, len(checks), [{'label': label, 'desc': 'FILE_NOT_FOUND', 'path': str(path), 'required': True, 'passed': False}]
    try:
        content = path.read_text(encoding='utf-8')
    except Exception as e:
        return 0, len(checks), [{'label': label, 'desc': f'READ_ERROR: {e}', 'path': str(path), 'required': True, 'passed': False}]

    import re
    passed = 0
    details = []
    for desc, pattern, required in checks:
        match = re.search(pattern, content)
        ok = bool(match)
        if ok:
            passed += 1
        details.append({
            'label': label,
            'desc': desc,
            'pattern': pattern,
            'required': required,
            'passed': ok,
        })
    return passed, len(checks), details


def main():
    parser = argparse.ArgumentParser(description='kanban-vmodel-jp Sprint 视图验证')
    parser.add_argument('--strict', action='store_true', help='严格模式: 任何 required 项未通过即 exit 1')
    parser.add_argument('--json', action='store_true', help='JSON 输出')
    args = parser.parse_args()

    results = []
    total_passed = 0
    total_count = 0

    for path, checks, label in [
        (APP_JS, APP_JS_CHECKS, 'app.js'),
        (INDEX_HTML, INDEX_HTML_CHECKS, 'index.html'),
        (STYLES_CSS, STYLES_CSS_CHECKS, 'styles.css'),
    ]:
        p, t, details = check_file(path, checks, label)
        total_passed += p
        total_count += t
        results.extend(details)

    if args.json:
        print(json.dumps({
            'summary': {
                'passed': total_passed,
                'total': total_count,
                'pct': round(100.0 * total_passed / max(total_count, 1), 1),
            },
            'details': results,
        }, ensure_ascii=False, indent=2))
    else:
        print(f"=== kanban-vmodel-jp Sprint 视图验证 ===\n")
        print(f"app.js     : {APP_JS}")
        print(f"index.html : {INDEX_HTML}")
        print(f"styles.css : {STYLES_CSS}\n")

        last_label = None
        for d in results:
            if d['label'] != last_label:
                print(f"\n--- {d['label']} ---")
                last_label = d['label']
            mark = '✅' if d['passed'] else '❌'
            req = ' (必)' if d.get('required') else ''
            desc = d.get('desc', '')
            if 'FILE_NOT_FOUND' in desc or 'READ_ERROR' in desc:
                print(f"  {mark} {desc}{req}")
            else:
                print(f"  {mark} {desc}{req}")

        print(f"\n=== 总计: {total_passed}/{total_count} ({100.0 * total_passed / max(total_count, 1):.1f}%) ===\n")

    if args.strict and total_passed < total_count:
        return 1
    return 0


if __name__ == '__main__':
    sys.exit(main())
