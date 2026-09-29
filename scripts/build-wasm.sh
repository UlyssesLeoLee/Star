#!/bin/bash
# =====================================================================
# scripts/build-wasm.sh — Build Rust→WASM crates for browser integration
# =====================================================================
# Per docs/architecture/2026-09-28-upgrade/rust-to-wasm-frontend-memory-research.md §3.1 P0/P1
# + docs/architecture/2026-09-30-upgrade/rust-wasm-build-integration-guide.md
#
# 用法:
#   bash scripts/build-wasm.sh                   # build all *-wasm crates (release)
#   bash scripts/build-wasm.sh layout-engine-wasm  # build specific crate
#   bash scripts/build-wasm.sh --debug           # build with debug symbols
#
# 输出:
#   frontend/public/wasm/<crate>_bg.wasm
#   frontend/public/wasm/<crate>.js
#   frontend/public/wasm/<crate>_bg.js (ES module glue)
#   frontend/public/wasm/<crate>.d.ts (TypeScript types)
#
# 守门 (per AGENTS.md §4):
#   - #7 0 unsafe (workspace.lints = forbid)
#   - #11 缺标比错标: deps 来自 [workspace.dependencies]
# =====================================================================

set -euo pipefail

# ----- 1. 检查 wasm-pack -----
if ! command -v wasm-pack &> /dev/null; then
  echo "❌ wasm-pack not found. Installing..."
  curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
  # Add to PATH (per installer default)
  export PATH="$HOME/.cargo/bin:$PATH"
fi

# ----- 2. 检查 wasm32 target -----
if ! rustup target list --installed | grep -q wasm32-unknown-unknown; then
  echo "❌ wasm32-unknown-unknown target missing. Installing..."
  rustup target add wasm32-unknown-unknown
fi

# ----- 3. 解析参数 -----
DEBUG=0
TARGET_CRATE=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --debug) DEBUG=1; shift ;;
    *) TARGET_CRATE="$1"; shift ;;
  esac
done

BUILD_PROFILE="release"
if [[ $DEBUG -eq 1 ]]; then
  BUILD_PROFILE="dev"
fi

# ----- 4. 找所有 *-wasm crates -----
ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
WASM_CRATES=()

if [[ -n "$TARGET_CRATE" ]]; then
  # 单 crate 模式
  if [[ -d "$ROOT_DIR/crates/$TARGET_CRATE" ]]; then
    WASM_CRATES=("$TARGET_CRATE")
  else
    echo "❌ crates/$TARGET_CRATE not found"
    exit 1
  fi
else
  # 全 crates 模式
  for dir in "$ROOT_DIR"/crates/*-wasm; do
    if [[ -d "$dir" ]] && grep -q "cdylib" "$dir/Cargo.toml" 2>/dev/null; then
      name=$(basename "$dir")
      WASM_CRATES+=("$name")
    fi
  done
fi

if [[ ${#WASM_CRATES[@]} -eq 0 ]]; then
  echo "❌ No WASM crates detected (need crates/*-wasm with cdylib)"
  exit 1
fi

echo "📦 Building WASM crates: ${WASM_CRATES[@]}"
echo "🎯 Profile: $BUILD_PROFILE"
echo ""

# ----- 5. Build 每个 crate -----
mkdir -p "$ROOT_DIR/frontend/public/wasm"

for crate in "${WASM_CRATES[@]}"; do
  echo "Building crates/$crate..."
  cd "$ROOT_DIR/crates/$crate"
  
  if [[ $DEBUG -eq 1 ]]; then
    wasm-pack build --target web --dev
  else
    wasm-pack build --target web --release
  fi
  
  # Copy to frontend/public/wasm/
  echo "  → Copying pkg/* to frontend/public/wasm/"
  cp pkg/* "$ROOT_DIR/frontend/public/wasm/"
  echo ""
done

echo "✅ WASM build complete!"
echo ""
echo "📁 Files in frontend/public/wasm/:"
ls -la "$ROOT_DIR/frontend/public/wasm/" | head -20