#!/usr/bin/env bash
# gen_tls_certs.sh - v0.71 envoy TLS 自动签发脚本
# 用途: 生成 self-signed TLS cert 用于 envoy HTTPS listener (8443)
# 调用: ./gen_tls_certs.sh [output_dir]
# 自动续期: 90 天, 配合 cron 或 k8s CronJob

set -euo pipefail

OUTPUT_DIR="${1:-./certs}"
DAYS_VALID="${DAYS_VALID:-90}"
CN="${CN:-star-mock.local}"
ORG="${ORG:-Star Project Mock}"
COUNTRY="${COUNTRY:-JP}"

mkdir -p "$OUTPUT_DIR"

# 检查 openssl
if ! command -v openssl >/dev/null 2>&1; then
    echo "ERROR: openssl not found, please install openssl first" >&2
    exit 1
fi

# 1. 生成 private key (2048-bit RSA)
openssl genrsa -out "$OUTPUT_DIR/tls.key" 2048 2>/dev/null

# 2. 生成 self-signed cert (90 天有效)
openssl req -new -x509 \
    -key "$OUTPUT_DIR/tls.key" \
    -out "$OUTPUT_DIR/tls.crt" \
    -days "$DAYS_VALID" \
    -subj "/C=$COUNTRY/O=$ORG/CN=$CN" \
    -addext "subjectAltName=DNS:localhost,DNS:star-mock-envoy,DNS:star-mock-envoy.star-mock.svc.cluster.local,IP:127.0.0.1" \
    2>/dev/null

# 3. 输出验证
echo "✅ TLS certs generated at $OUTPUT_DIR"
echo "  - tls.key (private key, 2048-bit RSA)"
echo "  - tls.crt (self-signed cert, $DAYS_VALID days valid, CN=$CN)"

# 4. 注入到 k8s secret (如果 kubectl 可用)
if command -v kubectl >/dev/null 2>&1; then
    echo "📦 Creating k8s secret star-mock-tls in namespace star-mock..."
    kubectl create secret tls star-mock-tls \
        --cert="$OUTPUT_DIR/tls.crt" \
        --key="$OUTPUT_DIR/tls.key" \
        --namespace=star-mock \
        --dry-run=client -o yaml | kubectl apply -f -
    echo "✅ k8s secret updated"
else
    echo "⚠️  kubectl not found, manual secret creation required"
    echo "   Run: kubectl create secret tls star-mock-tls --cert=$OUTPUT_DIR/tls.crt --key=$OUTPUT_DIR/tls.key -n star-mock"
fi

# 5. 输出 cert 详情
echo ""
echo "📋 Cert details:"
openssl x509 -in "$OUTPUT_DIR/tls.crt" -noout -subject -dates -issuer 2>/dev/null | sed 's/^/  /'

# 6. 续期提示
EXPIRY=$(openssl x509 -in "$OUTPUT_DIR/tls.crt" -noout -enddate 2>/dev/null | cut -d= -f2)
echo ""
echo "⏰ Cert expires: $EXPIRY"
echo "💡 Auto-renew: setup k8s CronJob or external cron to re-run this script every 60-75 days"
