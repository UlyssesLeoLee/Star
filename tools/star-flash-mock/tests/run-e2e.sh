#!/usr/bin/env bash
# run-e2e.sh — Star Mock Project 沙箱 Worktree 隔离 E2E 回归 (Playwright)
#
# 保存位置: tools/star-flash-mock/tests/run-e2e.sh
# 集成: tools/star-flash-mock/scripts/run-all.sh §ET 层
#
# 前提:
#   - frontend dev server 已启动 (npm run dev, 端口 3001)
#   - 后端已启动 (端口 9002，POST /worktrees 端口可达)
#   - Podman 已安装并在系统 PATH 中 (安装版本地执行容器挂载)
#
# 守门:
#   - tsc --noEmit 0 错 (前端)
#   - vitest 0 失败 (前端)
#   - Playwright sandbox-worktree-isolation 全过
#
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
TESTS_DIR="$(cd "$(dirname "$0")" && pwd)"
FRONTEND_DIR="$REPO_ROOT/frontend"

PYTHON_CMD="${PYTHON:-python3}"

echo "==== Star Mock Project E2E: Sandbox Worktree Isolation ===="
echo "REPO_ROOT: $REPO_ROOT"
echo "TESTS_DIR: $TESTS_DIR"
echo "FRONTEND_DIR: $FRONTEND_DIR"
echo "Started at: $(date -u +'%Y-%m-%dT%H:%M:%SZ')"
echo ""

# ===== 0. 前置检查: frontend dev server 是否可达 =====
echo "--- 0. 前置检查: frontend dev server (localhost:3001) ==="
if curl -sf "http://localhost:3001" >/dev/null 2>&1; then
  echo "  [OK] frontend dev server 可达 (3001)"
else
  echo "  [WARN] frontend dev server 不可达 (3001)，尝试启动..."
  # Windows 下用 cmd 启动 next dev
  if command -v cmd.exe >/dev/null 2>&1; then
    cmd.exe /c "cd /d $FRONTEND_DIR && npm run dev > $FRONTEND_DIR/frontend-dev-e2e.log 2>&1" &
    sleep 15
    if curl -sf "http://localhost:3001" >/dev/null 2>&1; then
      echo "  [OK] frontend dev server 已自动启动 (3001)"
    else
      echo "  [FAIL] frontend dev server 启动失败，查看 $FRONTEND_DIR/frontend-dev-e2e.log"
      exit 1
    fi
  else
    echo "  [FAIL] frontend dev server 不可达，请手动启动 npm run dev"
    exit 1
  fi
fi

# ===== 1. 运行 Playwright E2E =====
echo ""
echo "--- 1. 运行 Playwright E2E (sandbox-worktree-isolation) ==="
cd "$FRONTEND_DIR"
if npx playwright test --config "$TESTS_DIR/playwright.config.ts" "$TESTS_DIR/sandbox-worktree-isolation.spec.ts" 2>&1 | tee /tmp/e2e-output.log; then
  echo "  [OK] Playwright E2E 全部通过"
else
  echo "  [FAIL] Playwright E2E 失败"
  exit 1
fi

echo ""
echo "==== E2E 回归 PASSED ===="
echo "Finished at: $(date -u +'%Y-%m-%dT%H:%M:%SZ')"
