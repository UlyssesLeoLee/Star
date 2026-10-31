#!/bin/bash
# Build and verify Star Desktop bundles for the current host platform.
# @cypher schema=1 source_sha256=f0b312354712da41f098ccdf21395ce5b1493c6aa103c09800f271329b560ae6
# MERGE (self:File {path:"scripts/build-desktop.sh"})
# MERGE (npm_ci:ExternalService {id:"npm.ci",name:"npm ci"})
# MERGE (tauri_build:ExternalService {id:"tauri.cli.build",name:"Tauri CLI build"})
# MERGE (artifact_scan:ExternalService {id:"verify.bundle.artifacts",name:"verify release bundle artifacts"})
# MERGE (target_dir:Config {id:"CARGO_TARGET_DIR"})
# MERGE (bundle_targets:Config {id:"tauri.bundle.targets",name:"platform bundle targets"})
# MERGE (self)-[:CALLS]->(npm_ci)
# MERGE (self)-[:CALLS]->(tauri_build)
# MERGE (self)-[:CALLS]->(artifact_scan)
# MERGE (self)-[:CONFIGURES]->(target_dir)
# MERGE (self)-[:CONFIGURES]->(bundle_targets)
# @endcypher
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
STAR_DESKTOP_DIR="$REPO_ROOT/crates/star-desktop"
TARGET=""
ALL=false

usage() {
  echo "Usage: $0 [--target deb|appimage|msi|dmg] [--all]"
  echo "--all builds every supported stable bundle for this host; cross-platform artifacts are built by CI."
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --target|--bundle)
      if [[ $# -lt 2 || -n "$TARGET" ]]; then
        echo "error: --target requires one bundle and may be supplied only once"
        exit 1
      fi
      TARGET="$2"
      shift 2
      ;;
    --all)
      if [[ "$ALL" = true ]]; then
        echo "error: --all may be supplied only once"
        exit 1
      fi
      ALL=true
      shift
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      echo "error: unknown argument: $1"
      usage
      exit 1
      ;;
  esac
done

if [[ -n "$TARGET" && "$ALL" = true ]]; then
  echo "error: choose either --target or --all"
  exit 1
fi

for command in cargo node npm; do
  if ! command -v "$command" >/dev/null 2>&1; then
    echo "error: $command not found"
    exit 1
  fi
done

NODE_MAJOR="$(node -p 'Number(process.versions.node.split(".")[0])')"
if [[ "$NODE_MAJOR" -lt 24 ]]; then
  echo "error: Node.js 24 or newer is required; found $(node --version)"
  exit 1
fi

HOST_OS="$(uname -s)"
if [[ -z "$TARGET" ]]; then
  if [[ "$ALL" = true ]]; then
    case "$HOST_OS" in
      Linux) TARGET="all-linux" ;;
      MINGW*|MSYS*|CYGWIN*) TARGET="msi" ;;
      Darwin) TARGET="dmg" ;;
      *) echo "error: unsupported host platform $HOST_OS"; exit 1 ;;
    esac
  else
    case "$HOST_OS" in
      Linux) TARGET="all-linux" ;;
      MINGW*|MSYS*|CYGWIN*) TARGET="msi" ;;
      Darwin) TARGET="dmg" ;;
      *) echo "error: unsupported host platform $HOST_OS"; exit 1 ;;
    esac
  fi
fi

case "$TARGET" in
  deb|appimage)
    [[ "$HOST_OS" == "Linux" ]] || { echo "error: $TARGET bundles require a Linux host"; exit 1; }
    BUNDLES="$TARGET"
    ;;
  msi)
    [[ "$HOST_OS" == MINGW* || "$HOST_OS" == MSYS* || "$HOST_OS" == CYGWIN* ]] || { echo "error: msi bundles require a Windows host"; exit 1; }
    BUNDLES="msi"
    ;;
  dmg)
    [[ "$HOST_OS" == "Darwin" ]] || { echo "error: dmg bundles require a macOS host"; exit 1; }
    BUNDLES="dmg"
    ;;
  all-linux)
    [[ "$HOST_OS" == "Linux" ]] || { echo "error: all-linux bundles require a Linux host"; exit 1; }
    BUNDLES="deb,appimage"
    ;;
  *)
    echo "error: unsupported bundle '$TARGET' (use deb, appimage, msi, or dmg)"
    exit 1
    ;;
esac

echo "=== Installing locked frontend dependencies ==="
(cd "$STAR_DESKTOP_DIR/frontend" && npm ci --no-audit --no-fund)

export CARGO_TARGET_DIR="$STAR_DESKTOP_DIR/src-tauri/target"
echo "=== Building Tauri bundle(s): $BUNDLES ==="
cd "$STAR_DESKTOP_DIR"
frontend/node_modules/.bin/tauri build --bundles "$BUNDLES"

verify_bundle() {
  local bundle_dir="$1"
  local file_pattern="$2"
  local label="$3"
  local artifact

  artifact="$(find "$bundle_dir" -maxdepth 1 -type f -name "$file_pattern" -size +0c -print -quit 2>/dev/null || true)"
  if [[ -z "$artifact" ]]; then
    echo "error: expected a non-empty $label artifact matching $bundle_dir/$file_pattern"
    exit 1
  fi
  echo "$artifact"
}

echo "=== Verifying selected bundle artifacts ==="
case "$TARGET" in
  deb) verify_bundle "$CARGO_TARGET_DIR/release/bundle/deb" '*.deb' 'Debian' ;;
  appimage) verify_bundle "$CARGO_TARGET_DIR/release/bundle/appimage" '*.AppImage' 'AppImage' ;;
  msi) verify_bundle "$CARGO_TARGET_DIR/release/bundle/msi" '*.msi' 'Windows MSI' ;;
  dmg) verify_bundle "$CARGO_TARGET_DIR/release/bundle/dmg" '*.dmg' 'macOS disk image' ;;
  all-linux)
    verify_bundle "$CARGO_TARGET_DIR/release/bundle/deb" '*.deb' 'Debian'
    verify_bundle "$CARGO_TARGET_DIR/release/bundle/appimage" '*.AppImage' 'AppImage'
    ;;
esac

echo "=== Non-empty bundle files ==="
find "$CARGO_TARGET_DIR/release/bundle" -type f -size +0c -print
echo "Star Desktop bundle build and artifact verification complete."
