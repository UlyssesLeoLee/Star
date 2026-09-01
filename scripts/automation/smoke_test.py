#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
scripts/automation/smoke_test.py 鈥?4 鍩虹被 smoke 楠岃瘉
(per docs/automation-design.md 搂6.6)

璺戦€?4 涓熀绫?(dispatcher / cli_helper / refactor_template / generate_ac_matrix) 鐨?鏈€灏忓彲杩愯妗堜緥, 鏃犲壇浣滅敤, 楠岃瘉 import + class 瀹炰緥鍖?+ method 璋冪敤閮介€氳繃銆?
鐢ㄦ硶:
    python scripts/automation/smoke_test.py

杈撳嚭:
    - stdout: 姣忎釜 case 鐨?OK / FAIL
    - audit_log: docs/reports/automation-smoke.log
    - exit code: 0 = 鍏ㄩ儴 OK, 1 = 鏈?FAIL

绾︽潫 (per 瀹堥棬 #1 v1):
    - 鏍囧噯搴?only
    - 鏃犲壇浣滅敤 (涓嶇湡璺?cargo / git, 鐢?stub 妯″紡)
    - audit_log 蹇呭～
"""

from __future__ import annotations

import importlib
import sys
import time
import traceback
from pathlib import Path
from typing import Callable, Tuple

# 鎶?scripts/ 鍔犲埌 sys.path, 杩欐牱 `import automation.dispatcher` 鎵嶈兘鎵惧埌
# (per 瀹堥棬 #1 v1: 鏍囧噯搴?only, 涓嶅紩鍏?setup.py / pyproject.toml)
ROOT_DEFAULT = Path(__file__).resolve().parent.parent.parent
SCRIPTS_DIR = ROOT_DEFAULT / "scripts"
if str(SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPTS_DIR))

REPORTS_DIR_DEFAULT = ROOT_DEFAULT / "docs" / "reports"


def case(name: str) -> Tuple[bool, str]:
    """璺戝崟涓?smoke case, 杩?(ok, msg)"""
    try:
        if name == "dispatcher":
            from automation.dispatcher import SubagentDispatcher
            d = SubagentDispatcher(
                phase="smoke-test",
                audit_log=REPORTS_DIR_DEFAULT / "automation-smoke.log",
                briefs_dir=ROOT_DEFAULT / "docs" / "briefs" / "smoke",
            )
            brief_path = d.brief(
                task_id="smoke-dispatcher-001",
                content="smoke test content",
                agent="worker",
            )
            assert brief_path.exists(), "brief path should exist"
            handle = d.invoke(brief_path, timeout=10, agent="worker")
            assert handle.task_id == "smoke-dispatcher-001", "task_id mismatch"
            ok = d.verify("smoke-dispatcher-001")
            assert isinstance(ok, bool), "verify should return bool"
            output_path = d.collect_output("smoke-dispatcher-001")
            assert output_path.exists(), "output path should exist"
            return True, f"brief={brief_path.name} status=OK"

        elif name == "cli_helper":
            from automation.cli_helper.base import CliHelper
            h = CliHelper(
                phase="smoke-test",
                audit_log=REPORTS_DIR_DEFAULT / "automation-smoke.log",
            )
            # 璺戜竴涓棤瀹崇殑鍛戒护 (python --version)
            result = h.run([sys.executable, "--version"], retries=0, timeout=5)
            assert result.exit_code == 0, f"exit_code={result.exit_code}"
            assert "Python" in result.stdout, "should contain 'Python'"
            return True, f"python_version={result.stdout.strip()}"

        elif name == "refactor_template":
            from automation.refactor_template import (
                RefactorTemplate, Action, ApplyResult, VerifyResult, FinalReport,
                ExampleRemoveActorCtx,
            )
            r = ExampleRemoveActorCtx(
                report_path=Path("dummy"),
                phase="smoke-test",
                dry_run=True,
                audit_log=REPORTS_DIR_DEFAULT / "automation-smoke.log",
            )
            actions = r.parse_report()
            assert len(actions) > 0, "should parse at least 1 action"
            result = r.apply(actions[0])
            assert result.success, f"apply should succeed: {result.error}"
            final = r.run_full()
            assert final.total_actions == len(actions), "total_actions mismatch"
            return True, f"actions={len(actions)} dry_run=True"

        elif name == "judge":
            from automation.judge import judge, judge_all, DIMENSIONS, VERDICTS
            # 鍗曟潯鍒ゅ畾
            r = judge("P3-B.5", ["R", "V", "A"], note="smoke test")
            assert r.verdict == "P", f"P3-B.5 should be [P], got {r.verdict}"
            assert r.score == 3, f"score should be 3, got {r.score}"
            # 鍏?WBS 鍒ゅ畾
            all_results = judge_all()
            assert len(all_results) > 0, "should judge all tasks"
            summary = {
                "P": sum(1 for x in all_results if x.verdict == "P"),
                "M": sum(1 for x in all_results if x.verdict == "M"),
                "S": sum(1 for x in all_results if x.verdict == "S"),
            }
            return True, f"all={len(all_results)} P={summary['P']} M={summary['M']} S={summary['S']}"

        else:
            return False, f"unknown case: {name}"

    except Exception as e:
        return False, f"exception: {type(e).__name__}: {e}\n{traceback.format_exc()}"


def main():
    """璺?4 涓?smoke case"""
    print("=== automation smoke test ===")
    print(f"phase=smoke-test")
    print(f"audit_log={REPORTS_DIR_DEFAULT / 'automation-smoke.log'}")
    print()

    cases = ["dispatcher", "cli_helper", "refactor_template", "judge"]
    passed = 0
    failed = 0
    audit_log_path = REPORTS_DIR_DEFAULT / "automation-smoke.log"
    audit_log_path.parent.mkdir(parents=True, exist_ok=True)

    with audit_log_path.open("a", encoding="utf-8") as f:
        f.write(f"\n=== smoke_test.py @ {time.strftime('%Y-%m-%d %H:%M:%S')} ===\n")

        for name in cases:
            start = time.time()
            ok, msg = case(name)
            duration = time.time() - start
            status = "OK" if ok else "FAIL"
            print(f"[{status}] {name}: {msg} ({duration:.2f}s)")

            f.write(
                f'{{"timestamp": {start}, "case": "{name}", "status": "{status}", '
                f'"msg": "{msg}", "duration": {duration:.2f}}}\n'
            )

            if ok:
                passed += 1
            else:
                failed += 1

    print()
    print(f"=== Result: {passed}/{len(cases)} passed, {failed} failed ===")
    sys.exit(0 if failed == 0 else 1)


if __name__ == "__main__":
    main()
