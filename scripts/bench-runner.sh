#!/bin/bash
# scripts/bench-runner.sh - STAR Performance Baseline Runner
# per ADR-0036 §2 D15 + 性能预算 NFR
# 用法: bash scripts/bench-runner.sh
set -e

echo "=== STAR Performance Baseline ==="
echo "Date: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo

# 1. cargo build 全 workspace
echo "[1/6] cargo build --workspace"
time cargo build --workspace 2>&1 | tail -5

# 2. cargo test 全 workspace
echo "[2/6] cargo test --workspace"
time cargo test --workspace 2>&1 | tail -5

# 3. cargo clippy
echo "[3/6] cargo clippy -D warnings"
time cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail -5

# 4. 22 domain handler read
echo "[4/6] 22 domain handler read (cargo test -p star-mcp)"
time cargo test -p star-mcp 2>&1 | tail -5

# 5. Saga 执行
echo "[5/6] Saga 5-step (cargo test -p star-saga)"
time cargo test -p star-saga 2>&1 | tail -5

# 6. Cache 操作
echo "[6/6] Cache read/write (cargo test -p star-cache)"
time cargo test -p star-cache 2>&1 | tail -5

echo
echo "=== Done ==="
