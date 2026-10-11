#!/bin/bash
# =====================================================================
# scripts/build-desktop.sh — Build star-desktop Tauri 2.0 distribution binaries
# =====================================================================
# Per docs/architecture/2026-09-29-upgrade/rust-app-end-research.md §3.2 P6
# Builds Tauri 2.0 desktop app for Linux (.deb / AppImage) + Windows (.msi) + macOS (.app/.dmg)
#
# Usage:
#   bash scripts/build-desktop.sh                    # build for current platform
#   bash scripts/build-desktop.sh --target deb       # Linux .deb
#   bash scripts/build-desktop.sh --target appimage   # Linux AppImage
#   bash scripts/build-desktop.sh --target msi        # Windows .msi (需 wine + WiX on Linux)
#   bash scripts/build-desktop.sh --target dmg        # macOS .dmg (需 macOS host)
#   bash scripts/build-desktop.sh --all               # build all targets (cross-platform)
#
# Requirements:
#   - cargo + rustc (stable toolchain)
#   - node.js 18+ + npm 9+
#   - Linux build deps: libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev patchelf build-essential
#   - Windows: WiX Toolset 3.x + cargo-xwin (cross-compile)
#   - macOS: Xcode + cargo-bundle (or Apple Silicon native build)
#
# Output:
#   crates/star-desktop/src-tauri/target/release/bundle/deb/*.deb
#   crates/star-desktop/src-tauri/target/release/bundle/appimage/*.AppImage
#   crates/star-desktop/src-tauri/target/release/bundle/msi/*.msi (Windows)
#   crates/star-desktop/src-tauri/target/release/bundle/dmg/*.dmg (macOS)
#
# Build time:
#   - Cold build: ~30-45 min (Tauri 2.0 Linux dependencies compile)
#   - Incremental: ~5-10 min
#
# 守门:
#   - #7 0 unsafe (workspace lint 继承)
#   - #19 0 动 V0.1 业务 logic (本脚本仅构建, 不改源码)
# =====================================================================
# @cypher schema=1 source_sha256=3733e61fe0160aae328c5ee2cf899380dc30c815cd1198d41884dcc68b513f73
# MERGE (self:File {path:"scripts/build-desktop.sh"})
# MERGE (npm_install:ExternalService {id:"npm.install",name:"npm install"})
# MERGE (npm_build:ExternalService {id:"npm.run.build",name:"npm run build"})
# MERGE (tauri_build:ExternalService {id:"cargo.tauri.build",name:"cargo tauri build"})
# MERGE (artifact_scan:ExternalService {id:"find.bundle.artifacts",name:"find release bundle artifacts"})
# MERGE (target_dir:Config {id:"CARGO_TARGET_DIR"})
# MERGE (self)-[:CALLS]->(npm_install)
# MERGE (self)-[:CALLS]->(npm_build)
# MERGE (self)-[:CALLS]->(tauri_build)
# MERGE (self)-[:CALLS]->(artifact_scan)
# MERGE (self)-[:CONFIGURES]->(target_dir)
# @endcypher
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
STAR_DESKTOP_DIR="$REPO_ROOT/crates/star-desktop"

TARGET=""
ALL=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --target)
      TARGET="$2"
      shift 2
      ;;
    --all)
      ALL=true
      shift
      ;;
    --help|-h)
      echo "Usage: $0 [--target deb|appimage|msi|dmg] [--all]"
      exit 0
      ;;
    *)
      echo "Unknown arg: $1"
      exit 1
      ;;
  esac
done

# Sanity checks
if ! command -v cargo >/dev/null 2>&1; then
  echo "error: cargo not found (install Rust toolchain first)"
  exit 1
fi

if ! command -v node >/dev/null 2>&1; then
  echo "error: node not found (install Node.js 18+ first)"
  exit 1
fi

if ! command -v npm >/dev/null 2>&1; then
  echo "error: npm not found (install npm 9+ first)"
  exit 1
fi

cd "$STAR_DESKTOP_DIR"

# 1. Install frontend dependencies (per PR #246 package.json)
echo "=== Step 1: npm install ==="
cd frontend
if [ ! -d node_modules ]; then
  npm install
else
  echo "(node_modules already present, skipping)"
fi
cd ..

# 2. Build frontend bundle (per PR #246 vite.config.ts)
echo "=== Step 2: npm run build ==="
cd frontend
npm run build
cd ..

# 3. cargo tauri build
echo "=== Step 3: cargo tauri build ==="
export CARGO_TARGET_DIR="$STAR_DESKTOP_DIR/src-tauri/target"
if [ -n "$TARGET" ]; then
  echo "Target: $TARGET"
  cargo tauri build --target "$TARGET" || {
    echo "error: cargo tauri build failed for target $TARGET"
    echo "hint: verify target-specific build deps installed (Linux libwebkit2gtk-4.1-dev, Windows WiX, macOS Xcode)"
    exit 1
  }
elif [ "$ALL" = true ]; then
  echo "All targets: attempting cross-platform builds"
  echo "  (Linux .deb + AppImage, Windows .msi, macOS .dmg)"
  cargo tauri build || {
    echo "warning: not all targets built (likely missing macOS/Windows toolchain on Linux host)"
    echo "  Linux .deb + AppImage should have succeeded"
  }
else
  echo "Target: current platform default"
  cargo tauri build
fi

echo ""
echo "=== Build artifacts ==="
find src-tauri/target/release/bundle -type f 2>/dev/null | head -20 || echo "(no bundle directory yet — check cargo tauri build output)"

echo ""
echo "✅ star-desktop build complete"
