"""v27_rpc_fallback: 守门 #9 实证 P3-A.6/A.7 后备.

10 background task `net::ERR_CONNECTION_CLOSED` 但 status 报 succeeded.
拍板激活后, 子代理 dispatch 必填 3 段 fallback:
(a) `dispatcher.py invoke` 后 30s 内**必跑** `dispatcher.py verify <task_id>` 二次验证
(b) 验证失败 -> 走 `dispatcher.py collect_output` 拉真实 output + 写 `docs/briefs/<task_id>.status.json` 标 retry_count
(c) 连续 2 次 retry 失败 -> 走 root session 直接实装 (per 守门 #9 #3 实证 RPC 不可靠)

守门: #9 v20 子代理 dispatch 必先 brief + #14 v3 Mavis 永久代签.
status: 待 Ulysses 拍板激活 (per 9/1 14:58 + 9/8 16:08 拍板必带推荐项).
"""
# SPDX-License-Identifier: MIT OR Apache-2.0

import argparse
import json
import os
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path


WORKSPACE_ROOT = Path(os.environ.get("STAR_ROOT", "D:/Star"))
DISPATCHER_PY = WORKSPACE_ROOT / "scripts" / "automation" / "dispatcher.py"
STATUS_DIR = WORKSPACE_ROOT / "docs" / "briefs"
MAX_RETRIES = 2
VERIFY_TIMEOUT_SEC = 30


def _utc_now_iso() -> str:
    return datetime.now(timezone.utc).isoformat(timespec="seconds")


def _write_status(task_id: str, payload: dict) -> Path:
    STATUS_DIR.mkdir(parents=True, exist_ok=True)
    p = STATUS_DIR / f"{task_id}.status.json"
    p.write_text(json.dumps(payload, ensure_ascii=False, indent=2), encoding="utf-8")
    return p


def _run_dispatcher(*args: str) -> dict:
    """Run `python dispatcher.py <args>` and parse JSON. Raise on non-zero exit."""
    cmd = [sys.executable, str(DISPATCHER_PY), *args]
    proc = subprocess.run(cmd, cwd=str(WORKSPACE_ROOT), capture_output=True, text=True, timeout=60)
    if proc.returncode != 0:
        raise RuntimeError(f"dispatcher {args} failed: {proc.stderr[:400]}")
    out = proc.stdout.strip()
    try:
        return json.loads(out) if out else {}
    except json.JSONDecodeError:
        return {"_raw": out}


def invoke(task_id: str, brief_path: str, worktree: str) -> dict:
    """Step 1: dispatch the subagent via dispatcher.py invoke."""
    status = {
        "task_id": task_id,
        "brief": brief_path,
        "worktree": worktree,
        "phase": "invoke",
        "retry_count": 0,
        "started_at": _utc_now_iso(),
        "history": [],
    }
    _write_status(task_id, status)

    result = _run_dispatcher("invoke", task_id, brief_path, worktree)
    status["history"].append({"phase": "invoke", "result": result, "at": _utc_now_iso()})
    _write_status(task_id, status)
    return status


def verify(task_id: str) -> dict:
    """Step 2 (mandatory within 30s): verify the dispatched subagent."""
    start = time.monotonic()
    while time.monotonic() - start < VERIFY_TIMEOUT_SEC:
        result = _run_dispatcher("verify", task_id)
        if result.get("status") == "ok":
            return result
        time.sleep(2)
    raise TimeoutError(
        f"verify {task_id} did not return ok within {VERIFY_TIMEOUT_SEC}s"
    )


def collect_output(task_id: str, attempt: int) -> dict:
    """Step 3 (failure path): pull real output and bump retry counter."""
    status_path = STATUS_DIR / f"{task_id}.status.json"
    payload = json.loads(status_path.read_text(encoding="utf-8")) if status_path.exists() else {}
    payload["retry_count"] = attempt
    payload["phase"] = "collect_output"
    payload["history"].append({"phase": "collect_output", "attempt": attempt, "at": _utc_now_iso()})
    output = _run_dispatcher("collect_output", task_id)
    payload["collected_output"] = output
    _write_status(task_id, payload)
    return payload


def main() -> int:
    parser = argparse.ArgumentParser(description="v27 RPC fallback dispatcher wrapper")
    parser.add_argument("--task-id", required=True)
    parser.add_argument("--brief", required=True, help="path to brief relative to STAR_ROOT")
    parser.add_argument("--worktree", required=True)
    parser.add_argument("--skip-verify", action="store_true", help="debug only")
    args = parser.parse_args()

    invoke(args.task_id, args.brief, args.worktree)

    if args.skip_verify:
        print(json.dumps({"phase": "invoke", "skipped_verify": True}, ensure_ascii=False))
        return 0

    try:
        verify(args.task_id)
        print(json.dumps({"phase": "verified", "task_id": args.task_id}, ensure_ascii=False))
        return 0
    except (TimeoutError, RuntimeError) as e:
        payload = collect_output(args.task_id, 1)
        if payload["retry_count"] >= MAX_RETRIES:
            print(
                json.dumps(
                    {
                        "phase": "fallback_root_session",
                        "task_id": args.task_id,
                        "reason": str(e),
                        "retry_count": payload["retry_count"],
                    },
                    ensure_ascii=False,
                )
            )
            return 2
        print(json.dumps({"phase": "retry_required", "task_id": args.task_id, "reason": str(e)}, ensure_ascii=False))
        return 1


if __name__ == "__main__":
    sys.exit(main())
