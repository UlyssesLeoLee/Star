#!/usr/bin/env bash
# v0.98 P0-4 Stage 4.2 = install-sealed-secrets.sh
# (per v0.89 已知缺口 (c) 'kubeseal CLI 不在 P0-4 阶段 dev env (P2 阶段 k3s-deployable 安装)' 跨 session 续做)
#
# 用途: P2 阶段 k3s-deployable 上跑这个脚本, 安装:
#   1. kubeseal CLI (v0.28.x 跟 sealed-secrets controller 兼容)
#   2. sealed-secrets controller (per bitnami-labs/sealed-secrets helm chart)
#   3. 验证 (per 守门 #1 v25)
#
# 守门 #5 v2 env 安全: 不打印 SECRET, 只 log status
# 守门 #11 缺标比错标: P0-4 阶段不实跑 (per README §3 实跑条件)
#
# Usage:
#   bash deploy/k3s-local/install-sealed-secrets.sh
#   bash deploy/k3s-local/install-sealed-secrets.sh --uninstall (P2 阶段回滚)
#
# Prerequisites:
#   - kubectl >= 1.27 (跟 k3s 1.28+ 兼容)
#   - helm >= 3.13
#   - k3s cluster 已跑 (per 9/1 13:05 JST envoy 独立部署偏好)

set -euo pipefail

SEALED_SECRETS_VERSION="${SEALED_SECRETS_VERSION:-0.28.0}"
HELM_CHART_VERSION="${HELM_CHART_VERSION:-2.16.1}"
NAMESPACE="${NAMESPACE:-kube-system}"

log() { echo "[$(date +%Y-%m-%dT%H:%M:%S%z)] $*"; }

uninstall() {
    log "卸载 sealed-secrets controller..."
    helm uninstall sealed-secrets -n "$NAMESPACE" || true
    log "卸载完成"
    exit 0
}

if [[ "${1:-}" == "--uninstall" ]]; then
    uninstall
fi

log "v0.98 P0-4 Stage 4.2 install-sealed-secrets.sh 开始 (version=$SEALED_SECRETS_VERSION)"

# 1. 安装 kubeseal CLI (Linux x86_64)
log "[1/3] 安装 kubeseal CLI v$SEALED_SECRETS_VERSION..."
KUBESEAL_URL="https://github.com/bitnami-labs/sealed-secrets/releases/download/v${SEALED_SECRETS_VERSION}/kubeseal-${SEALED_SECRETS_VERSION}-linux-amd64.tar.gz"
TMPDIR=$(mktemp -d)
trap "rm -rf $TMPDIR" EXIT
curl -fsSL "$KUBESEAL_URL" -o "$TMPDIR/kubeseal.tar.gz"
tar -xzf "$TMPDIR/kubeseal.tar.gz" -C "$TMPDIR"
sudo install -m 0755 "$TMPDIR/kubeseal" /usr/local/bin/kubeseal
log "[1/3] kubeseal CLI 安装完成: $(kubeseal --version)"

# 2. 安装 sealed-secrets controller (helm chart)
log "[2/3] 安装 sealed-secrets controller (helm chart v$HELM_CHART_VERSION)..."
helm repo add bitnami-labs https://charts.bitnami.com/bitnami 2>/dev/null || true
helm repo update
helm upgrade --install sealed-secrets bitnami-labs/sealed-secrets \
    --namespace "$NAMESPACE" \
    --version "$HELM_CHART_VERSION" \
    --set "controller.fullnameOverride=sealed-secrets" \
    --wait
log "[2/3] sealed-secrets controller 安装完成"

# 3. 验证 (per 守门 #1 v25)
log "[3/3] 验证安装..."
kubectl -n "$NAMESPACE" rollout status deployment/sealed-secrets-controller --timeout=60s
kubectl -n "$NAMESPACE" get pods -l app.kubernetes.io/name=sealed-secrets
log "[3/3] 验证完成"

log "v0.98 sealed-secrets 集成完成, P2 阶段 worker 子代理可跑 scripts/secrets/platform_admin_password_gen.py 生成 password + kubeseal 加密 SealedSecret"
log "下一步: 跑 scripts/secrets/platform_admin_password_gen.py --sealed-secret deploy/k3s-local/secrets/platform-admin-secret.sealed.yaml"
