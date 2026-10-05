#!/bin/bash
# Kargo 安装 (Helm chart 路线)
# 背景: 首版 cloud-init 用了
#   github.com/akuity/kargo/releases/download/v1.12.1/install.yaml
#   → 实测 HTTP 404。Kargo 无 install.yaml, 官方是 OCI Helm chart:
#   oci://ghcr.io/akuity/kargo-charts/kargo
#   且 api.adminAccount.passwordHash / tokenSigningKey 无默认值, 必须提供。
#
# 守门 #5 v2: 凭据现场生成于 VM 内, 绝不写入仓库 / user-data。
set -x
LOG=/var/log/star-kargo-install.log
exec >> "$LOG" 2>&1

VER=1.12.1
CREDS=/tmp/kargo-creds
PASSFILE=/root/kargo-admin-pass.txt

echo "=== [$(date -Is)] kargo install start (ver ${VER}) ==="

command -v helm || { echo "FAIL:helm-missing"; exit 1; }

# ---------- 0. KUBECONFIG (关键) ----------
# Helm 默认找 ~/.kube/config 或 $KUBECONFIG, 两者都不存在 →
# "Kubernetes cluster unreachable: Get http://localhost:8080/version"。
# k3s 的 kubeconfig 在 /etc/rancher/k3s/k3s.yaml (首版 install 用了
# --write-kubeconfig-mode 644, 故非 root 也可读)。
export KUBECONFIG=/etc/rancher/k3s/k3s.yaml
k3s kubectl version --short 2>/dev/null || kubectl version
echo "KUBECONFIG=$KUBECONFIG"
k3s kubectl get ns >/dev/null && echo "DONE:kubeconfig-ok" || { echo "FAIL:kubeconfig-ok"; exit 1; }

# ---------- 1. 现场生成一次性凭据 ----------
PASS=$(openssl rand -base64 48 | tr -d "=+/" | head -c 32)
SIGN=$(openssl rand -base64 48 | tr -d "=+/" | head -c 32)
if ! command -v htpasswd >/dev/null 2>&1; then
  apt-get install -y apache2-utils >/dev/null 2>&1
fi
if command -v htpasswd >/dev/null 2>&1; then
  HASH=$(htpasswd -bnBC 10 "" "$PASS" | tr -d ':\n')
else
  HASH=$(openssl passwd -6 "$PASS")
fi
printf '%s\n%s\n' "$HASH" "$SIGN" > "$CREDS"
chmod 600 "$CREDS"
printf '%s' "$PASS" > "$PASSFILE"
chmod 600 "$PASSFILE"
echo "DONE:creds-generated"

# ---------- 2. helm install ----------
helm install kargo oci://ghcr.io/akuity/kargo-charts/kargo \
  --version "$VER" \
  --namespace kargo --create-namespace \
  --set api.adminAccount.passwordHash="$HASH" \
  --set api.adminAccount.tokenSigningKey="$SIGN" \
  --kubeconfig "$KUBECONFIG" --wait --timeout 12m
RC=$?
echo "HELM_RC=$RC"
if [ $RC -ne 0 ]; then
  echo "FAIL:kargo-apply"
  helm --kubeconfig "$KUBECONFIG" list -A
  helm -n kargo get events --sort-by=.lastTimestamp 2>&1 | tail -n 20
  exit 1
fi
echo "DONE:kargo-apply"

# ---------- 3. 等就绪 ----------
for i in $(seq 1 60); do
  total=$(k3s kubectl -n kargo get deploy --no-headers 2>/dev/null | wc -l)
  ready=$(k3s kubectl -n kargo get deploy -o jsonpath='{range .items[*]}{.status.availableReplicas}{" "}{end}' 2>/dev/null | wc -w)
  echo "poll ${i}: ready ${ready}/${total}"
  [ -n "$total" ] && [ "$ready" -ge 3 ] && [ "$ready" -eq "$total" ] && break
  sleep 10
done

echo "--- kargo pods ---"
k3s kubectl -n kargo get pods -o wide
echo "--- helm list ---"
helm --kubeconfig "$KUBECONFIG" list -A
echo "=== [$(date -Is)] kargo install end ==="
touch /var/log/star-kargo-install.complete
