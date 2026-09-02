#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
scripts/automation/ai_edit_mock.py — AI 修改 mock (per docs/automation-design.md v0.2 §12)
(per ai-edit-mode=本地 mock 拍板, 不开外部 AI, 仅调用脚本生成模板建议)

读脚本源码 + 静态分析, 产生 3 条 edit suggestion:
- add field (含 type, 字段名建议)
- remove method (含方法名)
- rename class (含 class 名 + 新名建议)

per 守门 #5 v2: 不开外部 API, 不读 env, 不打印 API key

用法:
    python scripts/automation/ai_edit_mock.py --script scripts/automation/integration_e2e.py
    python scripts/automation/ai_edit_mock.py --script-id integration_e2e --features "provider=hermes"
"""

from __future__ import annotations

import argparse
import json
import re
import sys
import time
from dataclasses import dataclass, asdict
from pathlib import Path
from typing import Optional

ROOT_DEFAULT = Path(__file__).resolve().parent.parent.parent
REPORTS_DIR_DEFAULT = ROOT_DEFAULT / "docs" / "reports"


@dataclass
class EditSuggestion:
    """AI 修改建议 (mock)"""

    type: str  # "add_field" / "remove_method" / "rename_class" / "add_helper"
    target: str  # 文件 + 行
    rationale: str
    diff_preview: str
    confidence: float  # 0.0 - 1.0 (mock 永远 < 0.5, 提示用户手动 review)


@dataclass
class AIEditResult:
    """AI 修改 mock 整体结果"""

    script_id: str
    script_path: str
    suggestions: list  # list[EditSuggestion]
    features_context: dict
    duration_ms: float


class AIEditMock:
    """AI 修改 mock 基类 (per docs/automation-design.md v0.2 §12.4)"""

    # 13 份脚本 + 5 套 unittest metadata (per §12.2 清单表)
    SCRIPTS = {
        # 基类
        "dispatcher": "scripts/automation/dispatcher.py",
        "cli_helper": "scripts/automation/cli_helper/base.py",
        "refactor_template": "scripts/automation/refactor_template.py",
        "judge": "scripts/automation/judge.py",
        "smoke_test": "scripts/automation/smoke_test.py",
        "registry_check": "scripts/automation/registry_check.py",
        # [P] 任务卡
        "integration_e2e": "scripts/automation/integration_e2e.py",
        "saga_e2e": "scripts/automation/saga_e2e.py",
        "git_push": "scripts/automation/git_push.py",
        "h2_refactor": "scripts/automation/h2_refactor.py",
        # unittest
        "integration_e2e_test": "scripts/automation/__tests__/integration_e2e_test.py",
        "saga_e2e_test": "scripts/automation/__tests__/saga_e2e_test.py",
        "git_push_test": "scripts/automation/__tests__/git_push_test.py",
        "h2_refactor_test": "scripts/automation/__tests__/h2_refactor_test.py",
    }

    def __init__(self, script_id: str, features_context: dict = None, audit_log: Optional[Path] = None):
        self.script_id = script_id
        self.features_context = features_context or {}
        self.audit_log = audit_log or (REPORTS_DIR_DEFAULT / "ai-edit-mock.log")
        self.audit_log.parent.mkdir(parents=True, exist_ok=True)

    def run(self) -> AIEditResult:
        """跑 AI 修改 mock: 读脚本 + 静态分析 + 产生 3 条建议"""
        start = time.time()
        if self.script_id not in self.SCRIPTS:
            raise ValueError(f"unknown script_id: {self.script_id} (per §12.2 清单表)")

        script_path = ROOT_DEFAULT / self.SCRIPTS[self.script_id]
        if not script_path.exists():
            raise FileNotFoundError(f"script not found: {script_path}")

        content = script_path.read_text(encoding="utf-8")

        # 3 条建议 (mock 永远 confidence < 0.5, 提示用户手动 review)
        suggestions = []

        # 1. add_field: 找第一个 @dataclass, 建议加 audit_log 字段
        dataclass_match = re.search(r"@dataclass\s*\nclass\s+(\w+)", content)
        if dataclass_match:
            class_name = dataclass_match.group(1)
            suggestions.append(EditSuggestion(
                type="add_field",
                target=f"{self.SCRIPTS[self.script_id]}:{class_name}",
                rationale=f"Mock 建议: 给 {class_name} 加 audit_log: Optional[Path] = None 字段, 统一 audit log 模式 (per docs/automation-design.md §3.4)",
                diff_preview=f"+    audit_log: Optional[Path] = None\n     {class_name}.audit_log.parent.mkdir(...)",
                confidence=0.3,
            ))

        # 2. remove_method: 找第一个 def stub_func, 建议删 (per §6 已知缺口 stub)
        stub_match = re.search(r"def\s+(stub_\w+)\([^)]*\)[^:]*:\s*[\"']{3}", content)
        if stub_match:
            method_name = stub_match.group(1)
            suggestions.append(EditSuggestion(
                type="remove_method",
                target=f"{self.SCRIPTS[self.script_id]}:{method_name}",
                rationale=f"Mock 建议: 删 {method_name} (per docs/automation-design.md §6 已知缺口, stub 应在跨 session 续实装)",
                diff_preview=f"-    def {method_name}(...):\n-        # stub\n-        pass\n+    # {method_name} removed (per §6 stub 实装跨 session 续)",
                confidence=0.2,
            ))

        # 3. rename_class: 找第一个 dataclass, 建议改名 (mock 永远 confidence < 0.3)
        if dataclass_match:
            old_name = dataclass_match.group(1)
            new_name = f"{old_name}V2"
            suggestions.append(EditSuggestion(
                type="rename_class",
                target=f"{self.SCRIPTS[self.script_id]}:{old_name}",
                rationale=f"Mock 建议: 改 {old_name} → {new_name} (V2 命名, per 守门 #3 5 域版本命名)",
                diff_preview=f"-class {old_name}:\n+class {new_name}:",
                confidence=0.15,
            ))

        # 4. (per features_context) add_helper: 5 域 mock 跟 features 联动
        if self.features_context.get("provider") == "hermes":
            suggestions.append(EditSuggestion(
                type="add_helper",
                target=f"{self.SCRIPTS[self.script_id]}:EndpointConfig",
                rationale="Mock 建议: 加 HermesConfig dataclass (跟 B.5 OpenClaw 共享, 但 base_url 改 /v2/hermes/ 前缀)",
                diff_preview="+@dataclass\n+class HermesConfig:\n+    base_url: str = 'https://api.hermes.local/v2/hermes'\n+    auth_header: str = 'X-Hermes-Auth'",
                confidence=0.4,
            ))

        duration = (time.time() - start) * 1000
        result = AIEditResult(
            script_id=self.script_id,
            script_path=str(script_path),
            suggestions=suggestions,
            features_context=self.features_context,
            duration_ms=duration,
        )
        self._audit(result)
        return result

    def _audit(self, result: AIEditResult):
        entry = {
            "timestamp": time.time(),
            "phase": "ai-edit-mock",
            "action": "run",
            "input": {"script_id": result.script_id, "features_context": result.features_context},
            "output": {
                "suggestions_count": len(result.suggestions),
                "duration_ms": result.duration_ms,
            },
        }
        with self.audit_log.open("a", encoding="utf-8") as f:
            f.write(json.dumps(entry, ensure_ascii=False) + "\n")


def main():
    parser = argparse.ArgumentParser(description="AI 修改 mock (per §12 ai-edit-mode=本地 mock)")
    parser.add_argument("--script-id", help="script_id (per §12.2 清单表)")
    parser.add_argument("--script", help="直接传 script path (优先于 --script-id)")
    parser.add_argument("--features", help="features context, e.g. 'provider=hermes,dry_run=true'")
    parser.add_argument("--audit-log", type=Path, help="审计日志路径")
    args = parser.parse_args()

    features_context = {}
    if args.features:
        for kv in args.features.split(","):
            if "=" in kv:
                k, v = kv.split("=", 1)
                features_context[k.strip()] = v.strip()

    script_id = args.script_id
    if args.script:
        # 反向查 SCRIPTS
        for sid, path in AIEditMock.SCRIPTS.items():
            if path == args.script:
                script_id = sid
                break
        if not script_id:
            print(f"unknown script: {args.script}", file=sys.stderr)
            sys.exit(1)

    mock = AIEditMock(script_id=script_id, features_context=features_context, audit_log=args.audit_log)
    result = mock.run()
    print(f"=== AI Edit Mock: {result.script_id} ===")
    print(f"path: {result.script_path}")
    print(f"duration: {result.duration_ms:.2f}ms")
    print(f"suggestions: {len(result.suggestions)}")
    for s in result.suggestions:
        print(f"\n  [{s.type}] {s.target} (confidence: {s.confidence})")
        print(f"    {s.rationale[:200]}")
        print(f"    diff: {s.diff_preview[:200]}")


if __name__ == "__main__":
    main()
