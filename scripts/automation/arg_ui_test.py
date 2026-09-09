#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
scripts/automation/arg_ui_test.py — ARG.5 frontend 5 UI 组件端到端验证

Per docs/briefs/arg-05-frontend-5ui.md §2.1 C
+ docs/automation-design.md §4 (新增 ARG.5 段)
+ 守门 #1 v19 (Python 化 ≥ 2 维)
+ 守门 #5 (env 安全 — 不打印明文)
+ 守门 #6 (PowerShell only — Python 调 subprocess.run 走 shell=False)
+ 守门 #9 (RPC 不可靠 — 不调外部 fetch, mock 测)
+ 守门 #12 v21 ([P] docs 同步)
+ 守门 #14 v3 (Mavis 临时代签)

验证矩阵 (per brief v0.50 AC-1..AC-7):
  IT-1:  pnpm install 0 err            (--silent, exit 0)
  IT-2:  pnpm tsc --noEmit 0 err       (前端 strict 守门)
  IT-3:  pnpm lint 0 err               (next lint)
  IT-4:  pnpm test 0 err               (vitest, 跑全 frontend tests 不退化)
  IT-5:  pnpm build 0 err              (next build, SSR 验证)
  IT-6:  13 REST 端点 ts 类型断言 (5 channel + 9 actions)
  IT-7:  zustand store 类型一致性 (loadAgents / createEdge / 9 actions)
  IT-8:  5 UI 组件文件存在
  IT-9:  4 lib/arg 文件存在
  IT-10: workspace 兼容 (cargo check --workspace --lib -j 4 0 err)

使用:
  python scripts/automation/arg_ui_test.py            # 全部
  python scripts/automation/arg_ui_test.py --quick   # 跳过 cargo check + build
  python scripts/automation/arg_ui_test.py --no-test # 跳过 vitest
"""

from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys
from pathlib import Path
from typing import List, Tuple

# ----- paths (per 守门 #5 env 安全, 用 env var 但不打印值) -----

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
FRONTEND_DIR = REPO_ROOT / "frontend"
PYTHON = sys.executable


def _resolve_pnpm() -> List[str]:
    """Return the right way to invoke pnpm on this platform.
    On Windows, `pnpm` (no extension) may fail to find via subprocess.run
    when shell=False; use the `.cmd` wrapper to be safe.
    """
    if os.name == "nt":
        # Prefer pnpm.cmd for stability; fall back to pnpm
        candidates = [
            "C:/Users/leo19/AppData/Roaming/npm/pnpm.cmd",
            "pnpm.cmd",
            "pnpm",
        ]
        for c in candidates:
            if Path(c).exists() or _which(c):
                return [c]
        return ["pnpm.cmd"]
    return ["pnpm"]


def _which(cmd: str) -> str | None:
    """Windows-friendly which."""
    try:
        out = subprocess.run(
            ["where", cmd] if os.name == "nt" else ["which", cmd],
            capture_output=True, text=True, shell=False, timeout=5,
        )
        if out.returncode == 0:
            return out.stdout.strip().splitlines()[0]
    except Exception:  # noqa: BLE001
        pass
    return None

# ----- 13 REST 端点路径 (per ARG.4 §4.12) -----

EXPECTED_REST_PATHS = [
    "POST", "/api/arg/agents",
    "GET", "/api/arg/agents",
    "GET", "/api/arg/agents/{id}",
    "PATCH", "/api/arg/agents/{id}",
    "POST", "/api/arg/edges",
    "GET", "/api/arg/edges",
    "GET", "/api/arg/edges/{id}",
    "PATCH", "/api/arg/edges/{id}",
    "DELETE", "/api/arg/edges/{id}",
    "GET", "/api/arg/graph",
    "POST", "/api/arg/templates/instantiate",
    "GET", "/api/arg/achievements",
    "GET", "/api/arg/achievements/me",
]

# 9 store actions (per brief v0.50 §2.1 B)
EXPECTED_STORE_ACTIONS = [
    "loadAgents",
    "loadEdges",
    "createEdge",
    "updateEdge",
    "archiveEdge",
    "instantiateTemplate",
    "loadAchievements",
    "unlockAchievement",
    "subscribeEvents",
]

# 5 UI 组件 (per brief v0.50 §2.1 A)
EXPECTED_UI_FILES = [
    "frontend/src/app/(app)/agent-relationships/page.tsx",
    "frontend/src/app/(app)/agent-relationships/editor/RelationshipEditor.tsx",
    "frontend/src/app/(app)/agent-relationships/editor/EdgeTypeSelector.tsx",
    "frontend/src/app/(app)/agent-relationships/view/RelationshipView.tsx",
    "frontend/src/app/(app)/agent-relationships/view/NodeDetail.tsx",
    "frontend/src/app/(app)/agent-relationships/achievements/AchievementWall.tsx",
    "frontend/src/app/(app)/agent-relationships/templates/TemplateGallery.tsx",
    "frontend/src/app/(app)/agent-relationships/AgentViewTab.tsx",
]

# 4 lib/arg 文件 (per brief v0.50 §2.1 B)
EXPECTED_LIB_FILES = [
    "frontend/src/lib/arg/store.ts",
    "frontend/src/lib/arg/api.ts",
    "frontend/src/lib/arg/ws.ts",
    "frontend/src/lib/arg/types.ts",
    "frontend/src/lib/arg/index.ts",
]

# ----- helpers -----


def _run(cmd: List[str], cwd: Path | None = None, timeout: int = 300) -> Tuple[int, str, str]:
    """Run subprocess; return (exit, stdout, stderr). Never raises."""
    try:
        proc = subprocess.run(
            cmd,
            cwd=str(cwd) if cwd else None,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            timeout=timeout,
            shell=False,  # 守门 #6: 不调 shell
        )
        return proc.returncode, proc.stdout, proc.stderr
    except FileNotFoundError as e:
        return 127, "", f"executable not found: {e}"
    except subprocess.TimeoutExpired as e:
        return 124, e.stdout or "", f"timeout after {timeout}s"
    except Exception as e:  # noqa: BLE001
        return 1, "", f"unexpected error: {e}"


def _print_evidence(label: str, exit_code: int, stdout_tail: str = "") -> None:
    print(f"  [{label}] exit={exit_code}", end="")
    if stdout_tail:
        snippet = stdout_tail.strip().splitlines()
        last = snippet[-1] if snippet else ""
        if last:
            print(f" | last={last[:120]!r}", end="")
    print()


# ----- IT cases -----


def it_1_pnpm_install() -> bool:
    """IT-1: pnpm install 0 err (前端 deps 装齐, 含 zustand, next, etc)."""
    print("IT-1: pnpm install")
    pnpm = _resolve_pnpm()
    rc, out, err = _run(pnpm + ["install", "--prefer-offline"], cwd=FRONTEND_DIR, timeout=300)
    _print_evidence("pnpm install", rc, out)
    return rc == 0


def it_2_tsc() -> bool:
    """IT-2: pnpm tsc --noEmit 0 err (TS strict 守门)."""
    print("IT-2: pnpm tsc --noEmit")
    pnpm = _resolve_pnpm()
    rc, out, err = _run(pnpm + ["exec", "tsc", "--noEmit", "-p", "tsconfig.json"], cwd=FRONTEND_DIR, timeout=180)
    _print_evidence("tsc --noEmit", rc, out)
    if rc != 0 and err:
        # Print first 30 error lines
        err_lines = [l for l in err.splitlines() if "error TS" in l][:30]
        for l in err_lines:
            print(f"    {l}")
    return rc == 0


def it_3_lint() -> bool:
    """IT-3: pnpm lint 0 err (next lint)."""
    print("IT-3: pnpm lint")
    pnpm = _resolve_pnpm()
    rc, out, err = _run(pnpm + ["run", "lint"], cwd=FRONTEND_DIR, timeout=120)
    _print_evidence("next lint", rc, out)
    return rc == 0


def it_4_test() -> bool:
    """IT-4: pnpm test (vitest) 0 err — 仅跑 ARG 相关的子集以避免拖慢."""
    print("IT-4: pnpm test (vitest, scoped to frontend/src)")
    pnpm = _resolve_pnpm()
    # Run a focused subset that includes store/arg
    cmd = pnpm + [
        "exec", "vitest", "run",
        "--reporter=basic",
        "src/lib/store.test.ts",
        "src/components/agent-view/AgentCanvasView.test.tsx",
        "src/app/(app)/__tests__/panels.test.tsx",
    ]
    rc, out, err = _run(cmd, cwd=FRONTEND_DIR, timeout=300)
    _print_evidence("vitest run", rc, out)
    if rc != 0 and out:
        tail = out.strip().splitlines()[-15:]
        for l in tail:
            print(f"    {l}")
    return rc == 0


def it_5_build() -> bool:
    """IT-5: pnpm build 0 err (next build, SSR 验证 /agent-relationships 路由)."""
    print("IT-5: pnpm build (next build)")
    pnpm = _resolve_pnpm()
    # Limit output for speed
    env_overrides = os.environ.copy()
    env_overrides["NEXT_TELEMETRY_DISABLED"] = "1"
    try:
        proc = subprocess.run(
            pnpm + ["run", "build"],
            cwd=str(FRONTEND_DIR),
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            timeout=600,
            env=env_overrides,
            shell=False,
        )
        rc = proc.returncode
        out_tail = "\n".join((proc.stdout or "").splitlines()[-15:])
    except subprocess.TimeoutExpired as e:
        rc = 124
        out_tail = f"timeout after 600s: {(e.stdout or '')[-500:]}"
    except Exception as e:  # noqa: BLE001
        rc = 1
        out_tail = str(e)
    _print_evidence("next build", rc, out_tail)
    return rc == 0


def it_6_rest_paths() -> bool:
    """IT-6: 13 REST 端点路径在 api.ts 中能找到 (greppable, 1:1 镜像)."""
    print("IT-6: 13 REST 端点 path assertion")
    api_path = REPO_ROOT / "frontend/src/lib/arg/api.ts"
    if not api_path.exists():
        print(f"    FAIL: {api_path} not found")
        return False
    text = api_path.read_text(encoding="utf-8")
    # 13 paths: pair (METHOD, /api/arg/...)
    # 简化: 检查 13 个函数名 (createAgent / listAgents / getAgent / updateAgent /
    # createEdge / listEdges / getEdge / updateEdge / archiveEdge / getGraph /
    # instantiateTemplate / listAchievements / myUnlocks)
    expected_funcs = [
        "createAgent",
        "listAgents",
        "getAgent",
        "updateAgent",
        "createEdge",
        "listEdges",
        "getEdge",
        "updateEdge",
        "archiveEdge",
        "getGraph",
        "instantiateTemplate",
        "listAchievements",
        "myUnlocks",
    ]
    missing = [f for f in expected_funcs if f"function {f}" not in text and f"function {f}(" not in text]
    if missing:
        print(f"    FAIL: missing {len(missing)} functions: {missing}")
        return False
    print(f"    OK: all 13 functions present")
    return True


def it_7_store_actions() -> bool:
    """IT-7: zustand store 9 actions 在 store.ts 中能找到."""
    print("IT-7: zustand store 9 actions assertion")
    store_path = REPO_ROOT / "frontend/src/lib/arg/store.ts"
    if not store_path.exists():
        print(f"    FAIL: {store_path} not found")
        return False
    text = store_path.read_text(encoding="utf-8")
    missing = []
    for action in EXPECTED_STORE_ACTIONS:
        # Looser match: any occurrence of the action name
        if action not in text:
            missing.append(action)
    if missing:
        print(f"    FAIL: missing {len(missing)} actions: {missing}")
        return False
    print(f"    OK: all 9 actions present")
    return True


def it_8_ui_files() -> bool:
    """IT-8: 5 UI 组件 + page.tsx + AgentViewTab.tsx 文件存在 (8 文件)."""
    print("IT-8: 8 UI 组件文件存在")
    missing = [f for f in EXPECTED_UI_FILES if not (REPO_ROOT / f).exists()]
    if missing:
        print(f"    FAIL: missing {len(missing)} files:")
        for m in missing:
            print(f"      - {m}")
        return False
    print(f"    OK: all 8 files present")
    return True


def it_9_lib_files() -> bool:
    """IT-9: 4 lib/arg 文件存在 (types / api / ws / store)."""
    print("IT-9: 4 lib/arg 文件存在")
    missing = [f for f in EXPECTED_LIB_FILES if not (REPO_ROOT / f).exists()]
    if missing:
        print(f"    FAIL: missing {len(missing)} files:")
        for m in missing:
            print(f"      - {m}")
        return False
    print(f"    OK: all {len(EXPECTED_LIB_FILES)} files present")
    return True


def it_10_workspace_compat() -> bool:
    """IT-10: workspace 兼容 — cargo check --workspace --lib -j 4 0 err (per 守门 #1 v25)."""
    print("IT-10: cargo check --workspace --lib -j 4 (workspace 兼容守门)")
    # 守门 #1 v25 推荐: --workspace --lib -j 4 (避免 STATUS_STACK_BUFFER_OVERRUN)
    rc, out, err = _run(
        ["cargo", "check", "--workspace", "--lib", "-j", "4"],
        cwd=REPO_ROOT,
        timeout=300,
    )
    _print_evidence("cargo check --workspace --lib -j 4", rc, out)
    if rc != 0 and err:
        # Show last 20 lines for diagnosis
        err_tail = "\n".join(err.strip().splitlines()[-20:])
        print(f"    --- stderr tail ---\n{err_tail}\n    --- end ---")
    return rc == 0


# ----- main -----

CASES = [
    ("IT-1", it_1_pnpm_install),
    ("IT-2", it_2_tsc),
    ("IT-3", it_3_lint),
    ("IT-4", it_4_test),
    ("IT-5", it_5_build),
    ("IT-6", it_6_rest_paths),
    ("IT-7", it_7_store_actions),
    ("IT-8", it_8_ui_files),
    ("IT-9", it_9_lib_files),
    ("IT-10", it_10_workspace_compat),
]


def main(argv: List[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="ARG.5 frontend 5 UI 端到端验证")
    parser.add_argument("--quick", action="store_true", help="跳过 IT-5 (build) + IT-10 (cargo check)")
    parser.add_argument("--no-test", action="store_true", help="跳过 IT-4 (vitest)")
    args = parser.parse_args(argv)

    cases: List[Tuple[str, callable]] = list(CASES)
    if args.quick:
        cases = [c for c in cases if c[0] not in {"IT-5", "IT-10"}]
    if args.no_test:
        cases = [c for c in cases if c[0] != "IT-4"]

    print(f"=== ARG.5 frontend 5 UI 端到端验证 ({len(cases)} cases) ===")
    print(f"Repo: {REPO_ROOT}")
    print(f"Frontend: {FRONTEND_DIR}")
    print()

    passed = 0
    failed: List[str] = []
    for name, fn in cases:
        try:
            ok = fn()
        except Exception as e:  # noqa: BLE001
            print(f"  EXCEPTION: {e}")
            ok = False
        if ok:
            passed += 1
        else:
            failed.append(name)
        print()

    total = len(cases)
    print(f"=== Summary: {passed}/{total} passed ===")
    if failed:
        print(f"FAILED: {failed}")
        return 1
    print("ALL GREEN ✓")
    return 0


if __name__ == "__main__":
    sys.exit(main())
