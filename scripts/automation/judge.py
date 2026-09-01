#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
scripts/automation/judge.py — 任务卡 [P]/[S]/[M] 判定 CLI
(per docs/automation-design.md §2.3 + §6.5)

辅助工具, 给任务卡判定提供打分界面, 输出 JSON, 不自动应用
(per 拍板决策必须用选项 9/1 14:58 JST 拍板)。

用法:
    # 单条
    python scripts/automation/judge.py --task-id P3-B.5 --hits R,V --note "..."

    # 全 WBS 任务卡 (per §4 任务卡表)
    python scripts/automation/judge.py --all

约束 (per 守门 #1 v1):
    - 标准库 only: argparse / json / re / pathlib
    - 命中维度: R (Rerunnable) / V (Volume) / S (Structural) / A (Audit-trail)
    - 输出 JSON, 不自动写 WBS
"""

from __future__ import annotations

import argparse
import json
import sys
from dataclasses import dataclass, asdict
from pathlib import Path
from typing import Optional

ROOT_DEFAULT = Path(__file__).resolve().parent.parent.parent

# === 4 个筛选维度定义 (per §2.1) ===

DIMENSIONS = {
    "R": {
        "name": "Rerunnable (可复现性)",
        "definition": "同一脚本可针对不同 input (commit hash / branch / file list / REQ ID) 重跑, 产出 deterministic output",
        "threshold": "输入参数化 (CLI args / config yaml) + 输出有迹 (stdout / file / exit code)",
    },
    "V": {
        "name": "Volume (数据量阈值)",
        "definition": "改动文件数 ≥ 10 或行数 ≥ 200 或 token 输出 ≥ 5K",
        "threshold": "任一满足",
    },
    "S": {
        "name": "Structural (结构性)",
        "definition": "重复模式 ≥ 3 (例: 22 domain 全部改 pub mod context → use star_context)",
        "threshold": "启发式 + AST / regex 操作比逐个 Edit 快 ≥ 2x",
    },
    "A": {
        "name": "Audit-trail (审计可观测)",
        "definition": "Python 脚本 stderr/stdout 可定向到 docs/reports/<phase>.log 入档",
        "threshold": "必填 audit_log 参数 + log schema (timestamp / phase / action / input / output / error)",
    },
}

# === 3 档判定 (per §2.2) ===

VERDICTS = {
    "P": {
        "label": "[P] Python 化",
        "score_range": "≥ 3 维命中",
        "action": "强制走 scripts/automation/<purpose>.py 落地, commit message 含脚本路径",
        "mandatory": True,
    },
    "M": {
        "label": "[M] Mixed (混合)",
        "score_range": "= 2 维命中",
        "action": "部分走脚本 + 部分 Shell / Edit, 在 scripts/automation/<purpose>.py 落主调用 + 注释标注 ad-hoc 步骤",
        "mandatory": False,
    },
    "S": {
        "label": "[S] Shell / Edit 直接",
        "score_range": "≤ 1 维命中",
        "action": "不需要脚本化, agent 主上下文直接处理",
        "mandatory": False,
    },
}


@dataclass
class JudgeResult:
    """判定结果 (per §2.3 输出)"""

    task_id: str
    hits: list
    score: int
    verdict: str  # P / M / S
    verdict_label: str
    rationale: str
    automation_path: Optional[str]
    note: Optional[str]


def judge(task_id: str, hits: list, note: Optional[str] = None) -> JudgeResult:
    """核心判定函数 (per §2.2 3 档判定)"""
    # 验证 hits
    valid_hits = [h.upper() for h in hits if h.upper() in DIMENSIONS]
    invalid = [h for h in hits if h.upper() not in DIMENSIONS]
    if invalid:
        return JudgeResult(
            task_id=task_id,
            hits=valid_hits,
            score=len(valid_hits),
            verdict="S",
            verdict_label=VERDICTS["S"]["label"],
            rationale=f"无效维度: {invalid}, 有效: {valid_hits}",
            automation_path=None,
            note=note,
        )

    # 打分
    score = len(valid_hits)
    if score >= 3:
        verdict = "P"
    elif score == 2:
        verdict = "M"
    else:
        verdict = "S"

    # 自动化路径建议 (per §4 任务卡表)
    automation_path = suggest_automation_path(task_id)

    # rationale
    dim_names = [DIMENSIONS[h]["name"] for h in valid_hits]
    rationale = (
        f"命中 {score} 维 ({', '.join(valid_hits)}: {', '.join(dim_names)}); "
        f"未命中: {', '.join(set(DIMENSIONS.keys()) - set(valid_hits))}"
    )
    if note:
        rationale += f"; 备注: {note}"

    return JudgeResult(
        task_id=task_id,
        hits=valid_hits,
        score=score,
        verdict=verdict,
        verdict_label=VERDICTS[verdict]["label"],
        rationale=rationale,
        automation_path=automation_path,
        note=note,
    )


def suggest_automation_path(task_id: str) -> Optional[str]:
    """建议 automation 路径 (per §4 任务卡表)"""
    # 简化的映射表, 真实实装可读 WBS 任务卡表
    mapping = {
        "P3-B.1": "scripts/automation/integration_test.py",
        "P3-B.2": "scripts/automation/integration_test.py",
        "P3-B.5": "scripts/automation/integration_e2e.py",
        "P3-B.6": "scripts/automation/integration_e2e.py",
        "P3-B.7": "scripts/automation/quota_test.py",
        "P3-B.8": "scripts/automation/fallback_chain.py",
        "P3-B.9": "scripts/automation/audit_log.py",
        "P3-C.6": "scripts/automation/saga_e2e.py",
        "P3-C.7": "scripts/automation/migration_runner.py",
        "P3-D.2": "scripts/automation/cross_platform_e2e.py",
        "P3-D.3": "scripts/automation/playwright_runner.py",
        "P3-D.5": "scripts/automation/msw_switch.py",
        "P3-D.6": "scripts/automation/ci_runner.py",
        "P3-E.4": "scripts/automation/kms_rotate.py",
        "P3-E.6": "scripts/automation/saga_e2e.py",
        "P3-E.7": "scripts/automation/ddd_review.py",
        "P3-F.2": "scripts/automation/cross_domain_e2e.py",
        "P3-F.3": "scripts/automation/changelog_gen.py",
        "P3-F.4": "scripts/automation/mermaid_gen.py",
        "P3-F.5": "scripts/automation/quality_gate.py",
        "P3-F.6": "scripts/automation/git_push.py",
        "H2-1": "scripts/automation/refactor_template.py",
        "H2-2": "scripts/automation/refactor_template.py",
        "H2-3": "scripts/automation/refactor_template.py",
        "H2-4": "scripts/automation/refactor_template.py",
        "H2-5": "scripts/automation/refactor_template.py",
    }
    return mapping.get(task_id)


def judge_all() -> list:
    """跑 WBS 任务卡全判定 (per §4 任务卡表)"""
    # 简化: 列出 §4 任务卡表的判定初判
    # 真实实装可读 STAR-P3-WBS-001.md 解析
    initial_judgments = [
        ("P3-B.1", ["R", "V"], "5 endpoint × 4 method"),
        ("P3-B.2", ["R", "V"], "mock 备选"),
        ("P3-B.3", ["S"], "schema 5 字段, 单文件"),
        ("P3-B.4", ["S"], "schema 5 字段, 单文件"),
        ("P3-B.5", ["R", "V", "A"], "5 endpoint, mock 备选"),
        ("P3-B.6", ["R", "V", "A"], "同 B.5"),
        ("P3-B.7", ["R", "S"], "backoff + 抖动"),
        ("P3-B.8", ["R", "S", "A"], "fallback 链路"),
        ("P3-B.9", ["R", "A"], "接入 domain-audit"),
        ("P3-C.1", ["S"], "单域"),
        ("P3-C.2", ["S"], "单域"),
        ("P3-C.3", ["S"], "单域"),
        ("P3-C.4", ["S"], "单域"),
        ("P3-C.5", ["S"], "单域"),
        ("P3-C.6", ["R", "S", "A"], "跨 5 域补偿"),
        ("P3-C.7", ["R", "S", "A"], "per-tenant schema 隔离"),
        ("P3-C.8", ["S"], "单域"),
        ("P3-C.9", [], "真人寻访"),
        ("P3-D.1", ["S"], "单文件改入口"),
        ("P3-D.2", ["R", "V", "A"], "windows/macos 矩阵"),
        ("P3-D.3", ["R", "V", "S", "A"], "Playwright 4 维全命中"),
        ("P3-D.4", ["S"], "单函数包装"),
        ("P3-D.5", ["R", "V", "S"], "3 handler real-mode"),
        ("P3-D.6", ["R", "A"], "CI runner"),
        ("P3-D.7", ["S"], "单 UI 组件"),
        ("P3-E.1", ["S"], "单域"),
        ("P3-E.2", ["S"], "单域"),
        ("P3-E.3", ["S"], "单域"),
        ("P3-E.4", ["R", "V", "A"], "KMS 凭证"),
        ("P3-E.5", [], "真人寻访"),
        ("P3-E.6", ["R", "S", "A"], "跨域编排"),
        ("P3-E.7", ["R", "A"], "docs 阶段"),
        ("P3-F.1", [], "真人寻访"),
        ("P3-F.2", ["R", "V", "S", "A"], "5 域 E2E 4 维全命中"),
        ("P3-F.3", ["R", "A"], "CHANGELOG 跨域"),
        ("P3-F.4", ["R", "A"], "mermaid 化"),
        ("P3-F.5", ["R", "V", "A"], "质量门 5 维"),
        ("P3-F.6", ["R", "A"], "推 origin"),
        ("H2-1", ["R", "S", "A"], "已落地 commit 68ae5ff"),
        ("H2-2", ["R", "V", "S", "A"], "117+ err 实证"),
        ("H2-3", ["R", "V", "S", "A"], "净修 507 err"),
        ("H2-4", ["R", "V", "S", "A"], "强类型重构"),
        ("H2-5", ["R", "V", "S", "A"], "150+ call sites"),
    ]
    return [judge(tid, hits, note) for tid, hits, note in initial_judgments]


def main():
    parser = argparse.ArgumentParser(description="任务卡 [P]/[S]/[M] 判定 CLI")
    parser.add_argument("--task-id", help="任务 ID, 例: P3-B.5")
    parser.add_argument("--hits", nargs="+", default=[], help="命中维度: R V S A 任意组合")
    parser.add_argument("--note", help="备注")
    parser.add_argument("--all", action="store_true", help="跑 WBS 任务卡全判定")
    args = parser.parse_args()

    if args.all:
        results = judge_all()
        output = {
            "total": len(results),
            "summary": {
                "P": sum(1 for r in results if r.verdict == "P"),
                "M": sum(1 for r in results if r.verdict == "M"),
                "S": sum(1 for r in results if r.verdict == "S"),
            },
            "results": [asdict(r) for r in results],
        }
        print(json.dumps(output, indent=2, ensure_ascii=False))
        return

    if not args.task_id:
        parser.error("必须 --task-id 或 --all")

    result = judge(args.task_id, args.hits, args.note)
    print(json.dumps(asdict(result), indent=2, ensure_ascii=False))


if __name__ == "__main__":
    main()
