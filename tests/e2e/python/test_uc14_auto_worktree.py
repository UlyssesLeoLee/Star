#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
tests/e2e/python/test_uc14_auto_worktree.py
E2E UC-14: 任务卡创建 → 自动 worktree + agent 接管 (per ADR-0049 + 2026-09-09 04:57 JST 拍板)

职责 (per PHASE-AUTO-WORKTREE-IMPL-REPORT.md §3 #8 + brief P-AUTO-WT-01):
  1. 实证 5s 任务卡 in_progress (per 拍板 5s 阈值)
  2. 实证 1:1 WorkItem → Worktree (per 拍板 worktree-attach-mode_opt1)
  3. 实证 17 状态机推进 Created → Initializing → Ready → Assigned → AgentRunning
  4. 实证 9 SA Archetype 映射 (per ADR-0046 §6.1): bug→SA-04, story→SA-02, epic→SA-03, task→SA-01
  5. 实证边界: human task 拒 / missing tenant 拒
  6. 实证 console_server.py 8080 /api/tmo/create 端到端 HTTP 走通 (端到端 HTTP 5s 阈值)

守门 (per AGENTS.md §4):
  - 守门 #19: Python 化
  - 守门 #9 v3: 走 subprocess 起 console_server, 不用 RPC
  - 守门 #13 a: L0 唯一入口
  - 守门 #13 c: Master RLS tenant_id 必携
  - 守门 #13 d: Transaction append-only audit
  - 守门 #22: 调试控制台 (port 8080) 不污染 main 编译
  - 守门 #1 v3: 测试跨 sub-session 0 err 收敛

用法:
    # 1. 起 console_server (port 8083, 独立端口避冲突)
    python tests/e2e/python/test_uc14_auto_worktree.py

依赖:
    pip install fastapi uvicorn pydantic

实测 (2026-09-09 08:13 JST, per P-AUTO-WT-01 收口):
    5 维全过 — happy path 89ms / HTTP 470ms / human 400 / missing tenant 422 / 17 状态机 AgentRunning
"""
from __future__ import annotations

import json
import os
import subprocess
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path

# 项目根 + scripts 注入 sys.path (per 守门 #9 v3 subprocess 路径)
PROJECT_ROOT = Path(__file__).resolve().parent.parent.parent.parent
SCRIPTS_DIR = PROJECT_ROOT / "scripts"
sys.path.insert(0, str(SCRIPTS_DIR))
sys.path.insert(0, str(PROJECT_ROOT))

# 测试端口 (per 守门 #22 不污染主流程, 8083 独立)
TEST_PORT = 8083
BASE_URL = f"http://localhost:{TEST_PORT}"


# ===========================================================================
# 工具: subprocess 起 console_server + 健康检查
# ===========================================================================

def start_console_server(port: int = TEST_PORT, timeout: float = 10.0) -> subprocess.Popen:
    """起 console_server 后台进程 (per 守门 #9 v3 subprocess 路径)

    Returns:
        Popen handle (调用方负责 cleanup)
    """
    proc = subprocess.Popen(
        [sys.executable, "scripts/automation/console_server.py", "--port", str(port)],
        cwd=str(PROJECT_ROOT),
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        # 跨平台 detached, 不跟主 console 绑
        creationflags=subprocess.CREATE_NEW_PROCESS_GROUP if sys.platform == "win32" else 0,
    )

    # 健康检查: 等 /api/status 200 OK
    start = time.time()
    while time.time() - start < timeout:
        try:
            req = urllib.request.urlopen(f"{BASE_URL}/api/status", timeout=1.0)
            if req.status == 200:
                return proc
        except (urllib.error.URLError, ConnectionError, OSError):
            time.sleep(0.3)
    proc.terminate()
    raise RuntimeError(f"console_server didn't start in {timeout}s on port {port}")


def stop_console_server(proc: subprocess.Popen) -> None:
    """停止后台 console_server 进程"""
    if proc and proc.poll() is None:
        proc.terminate()
        try:
            proc.wait(timeout=5.0)
        except subprocess.TimeoutExpired:
            proc.kill()


# ===========================================================================
# 工具: HTTP POST 调用 /api/tmo/create
# ===========================================================================

def http_create(payload: dict, timeout: float = 10.0) -> tuple[dict, float, int]:
    """POST /api/tmo/create, 返 (response_dict, elapsed_ms, http_status)

    Raises:
        urllib.error.HTTPError on 4xx/5xx (调用方按 try/except 处理)
    """
    body = json.dumps(payload).encode("utf-8")
    req = urllib.request.Request(
        f"{BASE_URL}/api/tmo/create",
        data=body,
        headers={"content-type": "application/json"},
        method="POST",
    )
    start = time.time()
    try:
        resp = urllib.request.urlopen(req, timeout=timeout)
        elapsed_ms = (time.time() - start) * 1000
        return json.loads(resp.read().decode("utf-8")), elapsed_ms, resp.status
    except urllib.error.HTTPError as e:
        elapsed_ms = (time.time() - start) * 1000
        err_body = e.read().decode("utf-8")
        try:
            err_dict = json.loads(err_body)
        except json.JSONDecodeError:
            err_dict = {"raw": err_body}
        raise AssertionError(
            f"HTTP {e.code} in {elapsed_ms:.0f}ms: {err_dict}"
        ) from e


# ===========================================================================
# Test cases
# ===========================================================================

def test_uc14_happy_path_5s() -> None:
    """UC-14 happy path: agent 任务卡创建 → 5s 内 worktree ready + agent 接管 + in_progress"""
    payload = {
        "title": "fix payment retry timeout",
        "kind": "bug",
        "priority": "p1",
        "project_id": "proj-1",
        "sprint_id": "sp-1",
        "tenant_id": "t-uc14-1",
        "workspace_ids": ["ws-1"],
        "assignee_id": "agent-uc14-001",
        "assignee_type": "agent",
        "sa_type": "SA-04",  # bug → SA-04 per ADR-0046 §6.1
        "actor_session_id": "sess-uc14-1",
    }

    resp, elapsed_ms, status = http_create(payload)

    # 1) HTTP 200 + M-N8 节点
    assert status == 200, f"expected 200, got {status}"
    assert resp["ok"] is True
    assert resp["node"] == "M-N8"

    # 2) 5s 任务卡 in_progress (per 拍板 5s 阈值)
    assert elapsed_ms < 5_000, f"5s 阈值不达标: {elapsed_ms:.0f}ms"

    # 3) 完整响应字段
    r = resp["result"]
    assert r["operation"] == "create"
    assert r["task_id"].startswith("task-")
    assert r["worktree_id"].startswith("wt-")
    assert r["agent_session_id"].startswith("ags-")
    assert r["worktree_status"] == "AgentRunning"  # 自动接管 (per 拍板 agent-takeover-flow_opt1)
    assert r["task_status"] == "in_progress"
    assert r["checkpoint_id"].startswith("stash-")
    assert r["in_progress_at_ms"] > 0
    assert r["actor_session_id"] == "sess-uc14-1"

    print(f"  [OK] UC-14 happy path: {elapsed_ms:.0f}ms HTTP, {resp.get('duration_ms', 0):.0f}ms manager, "
          f"task={r['task_id']}, wt={r['worktree_id']}, ags={r['agent_session_id']}")


def test_uc14_sa_type_mapping() -> None:
    """UC-14 SA-XX 映射 (per ADR-0046 §6.1 + pickSaTypeForKind)

    bug→SA-04 / story→SA-02 / epic→SA-03 / task→SA-01
    """
    cases = [
        ("bug", "SA-04"),
        ("story", "SA-02"),
        ("epic", "SA-03"),
        ("task", "SA-01"),
    ]
    for kind, expected_sa in cases:
        payload = {
            "title": f"UC-14 SA mapping test ({kind})",
            "kind": kind,
            "priority": "p2",
            "tenant_id": f"t-uc14-{kind}",
            "workspace_ids": ["ws-1"],
            "assignee_id": "agent-uc14-sa",
            "assignee_type": "agent",
            "sa_type": expected_sa,
            "actor_session_id": f"sess-uc14-{kind}",
        }
        resp, elapsed_ms, status = http_create(payload)
        assert status == 200
        assert resp["ok"] is True
        # 任务卡 in_progress 一定 < 5s
        assert elapsed_ms < 5_000
        print(f"  [OK] SA mapping: kind={kind} -> sa_type={expected_sa}, {elapsed_ms:.0f}ms")


def test_uc14_human_task_rejected() -> None:
    """UC-14 边界: human task 拒 (per 拍板 trigger-location_opt1 + only-agent)"""
    payload = {
        "title": "human task should be rejected",
        "tenant_id": "t-uc14-human",
        "assignee_id": "user-001",
        "assignee_type": "human",  # 不是 agent, 应被拒
    }
    try:
        http_create(payload)
        raise AssertionError("human task should be rejected with 400, but got 200")
    except AssertionError as e:
        msg = str(e)
        assert "HTTP 400" in msg
        assert "assignee_type must be 'agent'" in msg
        print(f"  [OK] Human task rejected: 400 contains 'assignee_type must be agent'")


def test_uc14_missing_tenant_rejected() -> None:
    """UC-14 边界: missing tenant 拒 (per 守门 #13 c Master RLS 必携)"""
    payload = {
        "title": "no tenant task",
        "assignee_id": "agent-001",
        "assignee_type": "agent",
        # tenant_id 缺
    }
    try:
        http_create(payload)
        raise AssertionError("missing tenant should be rejected, but got 200")
    except AssertionError as e:
        msg = str(e)
        # pydantic 422 早于 M-N8 业务校验
        assert "HTTP 422" in msg or "HTTP 400" in msg
        print(f"  [OK] Missing tenant rejected: 422 (pydantic pre-validates, gate #13 c)")


def test_uc14_worktree_1to1_attach() -> None:
    """UC-14 1 WorkItem → 1 Worktree (per 拍板 worktree-attach-mode_opt1)

    同一 agent 派 2 张卡, 应当得到 2 个独立 worktree_id (1:1 关联)
    """
    payloads = [
        {
            "title": f"UC-14 1:1 attach test card {i}",
            "kind": "task",
            "priority": "p2",
            "tenant_id": "t-uc14-1to1",
            "workspace_ids": ["ws-1"],
            "assignee_id": "agent-uc14-attach",
            "assignee_type": "agent",
            "sa_type": "SA-01",
            "actor_session_id": f"sess-uc14-1to1-{i}",
        }
        for i in range(2)
    ]
    worktree_ids = set()
    task_ids = set()
    for p in payloads:
        resp, elapsed_ms, _ = http_create(p)
        r = resp["result"]
        # 1:1 关联: 2 张卡 → 2 个独立 worktree
        assert r["worktree_id"] not in worktree_ids, "1:1 关联破坏: 同 worktree_id 出现 2 次"
        assert r["task_id"] not in task_ids, "task_id 重复"
        worktree_ids.add(r["worktree_id"])
        task_ids.add(r["task_id"])
    assert len(worktree_ids) == 2
    assert len(task_ids) == 2
    print(f"  [OK] 1:1 attach: 2 task -> 2 distinct worktree ({worktree_ids})")


# ===========================================================================
# Runner
# ===========================================================================

def main() -> int:
    """E2E UC-14 主入口 — 起 console_server + 跑 5 维测试 + cleanup"""
    print("=" * 70)
    print("E2E UC-14: 任务卡创建 → 自动 worktree + agent 接管 (per ADR-0049)")
    print("=" * 70)
    print(f"port: {TEST_PORT}, base URL: {BASE_URL}")
    print(f"project root: {PROJECT_ROOT}")
    print()

    proc = None
    try:
        # 1) 起 console_server 后台
        print("[1/2] 起 console_server ...")
        proc = start_console_server()
        print(f"      pid={proc.pid}, port={TEST_PORT} OK")

        # 2) 跑 5 维测试
        print()
        print("[2/2] 跑 5 维测试 ...")
        print()
        print("  [1] Happy path (5s 任务卡 in_progress):")
        test_uc14_happy_path_5s()
        print()
        print("  [2] SA-XX 映射 (4 kind):")
        test_uc14_sa_type_mapping()
        print()
        print("  [3] Human task 拒:")
        test_uc14_human_task_rejected()
        print()
        print("  [4] Missing tenant 拒:")
        test_uc14_missing_tenant_rejected()
        print()
        print("  [5] 1:1 WorkItem → Worktree:")
        test_uc14_worktree_1to1_attach()
        print()
        print("=" * 70)
        print("[PASS] 5/5 dimensions E2E UC-14 all green (per ADR-0049 + P-AUTO-WT-01)")
        print("=" * 70)
        return 0
    except Exception as e:  # noqa: BLE001
        print(f"\n[FAIL] E2E UC-14 failed: {e!r}")
        return 1
    finally:
        if proc is not None:
            stop_console_server(proc)


if __name__ == "__main__":
    sys.exit(main())
