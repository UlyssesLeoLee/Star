#!/bin/bash
# =====================================================================
# scripts/generate-tauri-keys.sh — Generate Tauri updater signing keypair
# =====================================================================
# Per docs/architecture/2026-09-30-upgrade/star-desktop-p8-signing-updater-guide.md §6
# 一次性生成 keypair: private_key.pem (CI secret) + public_key.pem (tauri.conf.json)
#
# Usage:
#   bash scripts/generate-tauri-keys.sh                    # default output to ./keys/
#   bash scripts/generate-tauri-keys.sh --out-dir ./secrets # custom output
#   bash scripts/generate-tauri-keys.sh --help
#
# Requirements:
#   - openssl 1.0+ (Linux/macOS preinstalled; Windows Git Bash has it)
#
# 守门:
#   - #5 env 安全 fingerprint 仅 env name 不读 value
#     (本脚本仅本地生成 + 提示用户手动保管, 不上传任何 secret)
#   - private key 严守: 0 commit, 仅本地 + GitHub Actions secret
# =====================================================================
set -euo pipefail

OUT_DIR="./keys"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --out-dir) OUT_DIR="$2"; shift 2 ;;
    --help|-h)
      echo "Usage: $0 [--out-dir DIR]"
      exit 0 ;;
    *) echo "Unknown arg: $1"; exit 1 ;;
  esac
done

# Sanity check
if ! command -v openssl >/dev/null 2>&1; then
  echo "error: openssl not found (install OpenSSL 1.0+ first)"
  exit 1
fi

mkdir -p "$OUT_DIR"
cd "$OUT_DIR"

echo "=== Step 1: Generate private key (2048-bit RSA) ==="
openssl genrsa -out private_key.pem 2048

echo "=== Step 2: Extract public key ==="
openssl rsa -in private_key.pem -pubout -out public_key.pem

echo "=== Step 3: base64 encode public key (for tauri.conf.json) ==="
PUBKEY_B64=$(base64 -i public_key.pem 2>/dev/null || base64 public_key.pem)

echo ""
echo "=== Generated files ==="
echo "  private_key.pem: $(stat -c %s private_key.pem 2>/dev/null || stat -f %z private_key.pem) bytes"
echo "  public_key.pem:  $(stat -c %s public_key.pem 2>/dev/null || stat -f %z public_key.pem) bytes"
echo ""
echo "=== tauri.conf.json pubkey (replace REPLACE_WITH_BASE64_PUBLIC_KEY) ==="
echo "$PUBKEY_B64"
echo ""
echo "=== Next steps ==="
echo "1. NEVER commit private_key.pem to git"
echo "2. Store private_key.pem in:"
echo "   - Local password manager (1Password / LastPass)"
echo "   - GitHub Actions secret TAURI_SIGNING_PRIVATE_KEY (multi-line secret, base64 encode)"
echo "3. Replace tauri.conf.json plugins.updater.pubkey with the public key above"
echo "4. Add public_key.pem to a secure backup (encrypted USB / HSM)"
echo ""
echo "✅ Tauri signing keypair generated in: $(realpath --bind=. 2>/dev/null || pwd)"
