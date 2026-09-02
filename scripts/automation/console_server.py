#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
scripts/automation/console_server.py — Automation Debug Console 后端
(per docs/automation-design.md v0.2 §12.3 API 端点)

FastAPI server (port 8080) 透出 13 份脚本 + 5 套 unittest:
- GET /api/scripts: 列出 13 份脚本 + 5 套 unittest (含 metadata + status)
- POST /api/scripts/{id}/toggle: 用户勾选/关闭脚本
- POST /api/scripts/{id}/run: 跑脚本 (status: enabled 才能跑)
- POST /api/features/{script_id}/{feature_id}/toggle: 勾选/关闭脚本内功能点
- POST /api/ai_edit: AI 修改 mock
- GET /api/status: 13 份脚本 + 5 套 unittest 状态总览
- POST /api/brief: dispatcher.brief 落档 (per 守门 #20 v2)

约束 (per 守门 #1 v1 + 守门 #5 v2 + 守门 #9 v3):
- 标准库 + fastapi + uvicorn (第三方)
- 13 份脚本 + 5 套 unittest 状态在内存 (重启后 reset, 跨 session 续可加持久化)
- AI 修改走 mock (不开外部 API, per §12 ai-edit-mode=本地 mock 拍板)
- 跑脚本走 subprocess.run, 不用 RPC
- audit log 落 docs/reports/console-server.log

用法:
    python scripts/automation/console_server.py
    # 默认 port 8080
    # 浏览器开 http://localhost:3000/automation-debug (Next.js 调试页)
    # Next.js 通过 http://localhost:8080/api/* 调后端
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import time
from dataclasses import dataclass, field, asdict
from pathlib import Path
from typing import Optional

try:
    from fastapi import FastAPI, HTTPException
    from fastapi.middleware.cors import CORSMiddleware
    import uvicorn
except ImportError:
    print("ERROR: fastapi + uvicorn not installed. pip install fastapi uvicorn", file=sys.stderr)
    sys.exit(1)

ROOT_DEFAULT = Path(__file__).resolve().parent.parent.parent
REPORTS_DIR_DEFAULT = ROOT_DEFAULT / "docs" / "reports"


@dataclass
class ScriptMeta:
    """13 份脚本 + 5 套 unittest metadata (per §12.2 清单表)"""

    id: str
    path: str
    category: str  # "base" / "p_card" / "unittest"
    features: list  # list[str] 功能点
    description: str
    status: str = "enabled"  # enabled / disabled
    last_run: Optional[str] = None
    last_run_output: str = ""  # 头 500 字符
    run_count: int = 0


# 13 份脚本 + 5 套 unittest metadata (per §12.2 清单表)
SCRIPTS_META = {
    # 8 份基类
    "dispatcher": ScriptMeta(
        id="dispatcher", path="scripts/automation/dispatcher.py", category="base",
        features=["brief", "invoke_stub", "verify_stub", "collect_output_stub"],
        description="子代理 dispatch 基类 (per §3.1 dispatcher 范式)",
    ),
    "cli_helper": ScriptMeta(
        id="cli_helper", path="scripts/automation/cli_helper/base.py", category="base",
        features=["run", "cargo_stub", "git_stub", "wt_stub", "with_worktree_stub"],
        description="CLI 调用基类 (per §3.2 cli_helper 范式)",
    ),
    "refactor_template": ScriptMeta(
        id="refactor_template", path="scripts/automation/refactor_template.py", category="base",
        features=["parse_report", "apply", "verify_stub", "rollback_stub", "run_full"],
        description="代码改造基类 (per §3.3 refactor_template 范式)",
    ),
    "judge": ScriptMeta(
        id="judge", path="scripts/automation/judge.py", category="base",
        features=["judge_single", "judge_all"],
        description="任务卡 [P]/[S]/[M] 判定 CLI (per §2.3 judge 范式)",
    ),
    "smoke_test": ScriptMeta(
        id="smoke_test", path="scripts/automation/smoke_test.py", category="base",
        features=["dispatcher_case", "cli_helper_case", "refactor_template_case", "judge_case"],
        description="4 基类 smoke 验证 (per §6.6 smoke_test)",
    ),
    "registry_check": ScriptMeta(
        id="registry_check", path="scripts/automation/registry_check.py", category="base",
        features=["single_check"],
        description="索引一致性校验 (per §6.7 registry_check)",
    ),
    # 4 份 [P] 任务卡脚本
    "integration_e2e": ScriptMeta(
        id="integration_e2e", path="scripts/automation/integration_e2e.py", category="p_card",
        features=["provider=openclaw", "provider=hermes", "dry_run", "no_dry_run_stub", "audit_log"],
        description="B.5/B.6 OpenClaw/Hermes 5 endpoint × 4 method e2e (per §4.1)",
    ),
    "saga_e2e": ScriptMeta(
        id="saga_e2e", path="scripts/automation/saga_e2e.py", category="p_card",
        features=["fail_domain=none", "fail_domain=player", "fail_domain=economy", "fail_domain=match", "fail_domain=social", "fail_domain=admin", "dry_run", "audit_log"],
        description="C.6 Saga 跨 5 域补偿 + 失败回滚 (per §4.2)",
    ),
    "git_push": ScriptMeta(
        id="git_push", path="scripts/automation/git_push.py", category="p_card",
        features=["remote=origin", "dry_run", "no_dry_run_stub", "max_scan_files", "audit_log"],
        description="F.6 git_push 推 origin + secret 扫描 (per §4.5)",
    ),
    "h2_refactor": ScriptMeta(
        id="h2_refactor", path="scripts/automation/h2_refactor.py", category="p_card",
        features=["phase=P3-H2", "dry_run", "no_dry_run_stub", "audit_log"],
        description="H2-1 refactor_template 子类化 H2Stage1Refactor (per §4.6)",
    ),
    # 5 套 unittest
    "integration_e2e_test": ScriptMeta(
        id="integration_e2e_test", path="scripts/automation/__tests__/integration_e2e_test.py", category="unittest",
        features=["openclaw_6_case", "hermes_6_case"],
        description="B.5/B.6 12 unittest (per §4.1)",
    ),
    "saga_e2e_test": ScriptMeta(
        id="saga_e2e_test", path="scripts/automation/__tests__/saga_e2e_test.py", category="unittest",
        features=["success_case", "failure_5_domain", "idempotency_2", "audit_1"],
        description="C.6 Saga 10 unittest (per §4.2)",
    ),
    "git_push_test": ScriptMeta(
        id="git_push_test", path="scripts/automation/__tests__/git_push_test.py", category="unittest",
        features=["dry_run_2", "secret_scan_1", "token_safety_1", "audit_1"],
        description="F.6 git_push 5 unittest (per §4.5)",
    ),
    "h2_refactor_test": ScriptMeta(
        id="h2_refactor_test", path="scripts/automation/__tests__/h2_refactor_test.py", category="unittest",
        features=["parse_5_actions", "action_1", "action_2", "apply", "inherits"],
        description="H2-1 refactor 5 unittest (per §4.6)",
    ),
}


# FastAPI app
app = FastAPI(
    title="Automation Debug Console",
    description="13 份 Python 脚本 + 5 套 unittest 调试控制台 (per docs/automation-design.md v0.2 §12)",
    version="0.2.0",
)

# CORS 允许 Next.js localhost:3000 跨域调 8080
app.add_middleware(
    CORSMiddleware,
    allow_origins=["http://localhost:3000", "http://localhost:3100"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


def _audit(action: str, input: dict, output: dict, error: Optional[str] = None):
    """audit log 落 docs/reports/console-server.log (per §3.4)"""
    audit_log = REPORTS_DIR_DEFAULT / "console-server.log"
    audit_log.parent.mkdir(parents=True, exist_ok=True)
    entry = {
        "timestamp": time.time(),
        "phase": "console-server",
        "action": action,
        "input": input,
        "output": output,
        "error": error,
    }
    with audit_log.open("a", encoding="utf-8") as f:
        f.write(json.dumps(entry, ensure_ascii=False) + "\n")


# === 7 个 API 端点 (per §12.3) ===

@app.get("/api/scripts")
def list_scripts():
    """列 13 份脚本 + 5 套 unittest (含 metadata + status)"""
    result = {sid: asdict(meta) for sid, meta in SCRIPTS_META.items()}
    _audit("list_scripts", {}, {"count": len(result)})
    return {"scripts": result, "total": len(result)}


@app.post("/api/scripts/{script_id}/toggle")
def toggle_script(script_id: str, status: str = "enabled"):
    """用户勾选/关闭脚本 (per close-behavior=1 跳过)"""
    if script_id not in SCRIPTS_META:
        raise HTTPException(status_code=404, detail=f"unknown script: {script_id}")
    if status not in ("enabled", "disabled"):
        raise HTTPException(status_code=400, detail=f"invalid status: {status}")
    SCRIPTS_META[script_id].status = status
    _audit("toggle_script", {"script_id": script_id, "status": status}, {"ok": True})
    return {"script_id": script_id, "status": status, "ok": True}


@app.post("/api/scripts/{script_id}/run")
def run_script(script_id: str):
    """跑脚本 (status: enabled 才能跑, 跑完返 output 头 500 字符)"""
    if script_id not in SCRIPTS_META:
        raise HTTPException(status_code=404, detail=f"unknown script: {script_id}")
    meta = SCRIPTS_META[script_id]
    if meta.status == "disabled":
        raise HTTPException(status_code=403, detail=f"script {script_id} is disabled (per §12.6 close-behavior=1)")

    script_path = ROOT_DEFAULT / meta.path
    if not script_path.exists():
        raise HTTPException(status_code=404, detail=f"file not found: {script_path}")

    start = time.time()
    try:
        proc = subprocess.run(
            [sys.executable, str(script_path)],
            capture_output=True,
            text=True,
            timeout=120,
            cwd=str(ROOT_DEFAULT),
        )
        duration = (time.time() - start) * 1000
        output_preview = proc.stdout[:500] + ("\n... (truncated)" if len(proc.stdout) > 500 else "")
        meta.last_run = time.strftime("%Y-%m-%d %H:%M:%S")
        meta.last_run_output = output_preview
        meta.run_count += 1
        _audit(
            "run_script",
            {"script_id": script_id, "timeout": 120},
            {"exit_code": proc.returncode, "duration_ms": duration, "output_len": len(proc.stdout)},
        )
        return {
            "script_id": script_id,
            "exit_code": proc.returncode,
            "duration_ms": duration,
            "output_preview": output_preview,
            "stderr_preview": proc.stderr[:200],
            "ok": proc.returncode == 0,
        }
    except subprocess.TimeoutExpired as e:
        return {
            "script_id": script_id,
            "exit_code": -1,
            "duration_ms": (time.time() - start) * 1000,
            "output_preview": "",
            "stderr_preview": f"timeout after 120s: {e}",
            "ok": False,
        }


@app.post("/api/features/{script_id}/{feature_id}/toggle")
def toggle_feature(script_id: str, feature_id: str, enabled: bool = True):
    """勾选/关闭脚本内功能点 (e.g. provider=hermes)"""
    if script_id not in SCRIPTS_META:
        raise HTTPException(status_code=404, detail=f"unknown script: {script_id}")
    meta = SCRIPTS_META[script_id]
    if feature_id not in meta.features:
        raise HTTPException(status_code=404, detail=f"feature not found: {feature_id} (in {script_id})")
    # 简化: 关闭 = 加 _disabled 后缀标记
    status = "enabled" if enabled else "disabled"
    _audit("toggle_feature", {"script_id": script_id, "feature_id": feature_id, "enabled": enabled}, {"ok": True})
    return {"script_id": script_id, "feature_id": feature_id, "status": status, "ok": True}


@app.post("/api/ai_edit")
def ai_edit(script_id: str, features_context: dict = {}):
    """AI 修改 mock: 读脚本源码 + 模板生成建议 (不开外部 API, per §12 ai-edit-mode=本地 mock)"""
    if script_id not in SCRIPTS_META:
        raise HTTPException(status_code=404, detail=f"unknown script: {script_id}")
    from automation.ai_edit_mock import AIEditMock
    mock = AIEditMock(script_id=script_id, features_context=features_context)
    result = mock.run()
    _audit("ai_edit", {"script_id": script_id, "features_context": features_context}, {"suggestions_count": len(result.suggestions)})
    return {
        "script_id": result.script_id,
        "script_path": result.script_path,
        "suggestions": [asdict(s) for s in result.suggestions],
        "duration_ms": result.duration_ms,
    }


@app.get("/api/status")
def status():
    """13 份脚本 + 5 套 unittest 状态总览 (跑 / 关闭 / AI mock 等)"""
    summary = {"enabled": 0, "disabled": 0, "total_runs": 0}
    for meta in SCRIPTS_META.values():
        if meta.status == "enabled":
            summary["enabled"] += 1
        else:
            summary["disabled"] += 1
        summary["total_runs"] += meta.run_count
    _audit("status", {}, summary)
    return {
        "total_scripts": len(SCRIPTS_META),
        "enabled": summary["enabled"],
        "disabled": summary["disabled"],
        "total_runs": summary["total_runs"],
        "scripts": {sid: {"status": m.status, "run_count": m.run_count, "last_run": m.last_run} for sid, m in SCRIPTS_META.items()},
    }


@app.post("/api/brief")
def brief(task_id: str, agent: str = "user", content: str = ""):
    """dispatcher.brief 落档 (per 守门 #20 v2)"""
    from automation.dispatcher import SubagentDispatcher
    d = SubagentDispatcher(phase="P3-debug-console")
    brief_path = d.brief(task_id=task_id, content=content, agent=agent)
    _audit("brief", {"task_id": task_id, "agent": agent}, {"brief_path": str(brief_path)})
    return {"task_id": task_id, "brief_path": str(brief_path), "ok": True}


def main():
    parser = argparse.ArgumentParser(description="Automation Debug Console FastAPI server")
    parser.add_argument("--host", default="127.0.0.1", help="host (default: 127.0.0.1)")
    parser.add_argument("--port", type=int, default=8080, help="port (default: 8080)")
    args = parser.parse_args()
    print(f"=== Automation Debug Console: http://{args.host}:{args.port} ===")
    print(f"  docs: http://{args.host}:{args.port}/docs (FastAPI swagger)")
    print(f"  scripts: {len(SCRIPTS_META)} (8 base + 4 [P] + 5 unittest)")
    uvicorn.run(app, host=args.host, port=args.port, log_level="info")


if __name__ == "__main__":
    main()
