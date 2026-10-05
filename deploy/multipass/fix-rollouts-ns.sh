#!/bin/bash
# 诊断 + 修复: Argo Rollouts 装到官方 namespace
# 一次性脚本, 传进 VM 用 sudo bash 执行。
# 背景: 首版把 install.yaml 装到 default ns, namespaced RBAC 全部不生效 →
#       "configmaps argo-rollouts-config is forbidden" → CrashLoopBackOff。
#       且 `kubectl -k <remote-url>` 不支持远程 URL (会当成本地路径)。
set -x
LOG=/var/log/star-rollouts-fix.log
exec >> "$LOG" 2>&1

NS=argo-rollouts
VER=v1.10.0

echo "=== [$(date -Is)] rollouts namespace fix start ==="

# ---------- 1. 探测 upstream 提供了哪些资源 ----------
for f in install.yaml manifests-install.yaml; do
  code=$(curl -s -o /tmp/ro-$f -w '%{http_code}' -L \
    "https://github.com/argoproj/argo-rollouts/releases/download/${VER}/${f}")
  echo "PROBE ${f} -> HTTP ${code} ($(stat -c%s /tmp/ro-$f 2>/dev/null || echo 0) bytes)"
done

echo "--- install.yaml kinds ---"
grep '^kind:' /tmp/ro-install.yaml | sort | uniq -c
echo "--- install.yaml namespaces ---"
grep 'namespace:' /tmp/ro-install.yaml | sort -u

# ---------- 2. 装到官方 ns ----------
kubectl create namespace "$NS" --dry-run=client -o yaml | kubectl apply -f -
echo "APPLY_NS_RC=$?"

kubectl apply -n "$NS" -f /tmp/ro-install.yaml
echo "APPLY_RC=$?"

# ---------- 3. 显式补 namespaced RBAC ----------
# upstream ClusterRole 不覆盖 namespaced configmap 读, 这里补最小权限。
kubectl create role "${NS}-config-reader" -n "$NS" \
  --verb=get,list,watch --resource=configmaps,secrets \
  --dry-run=client -o yaml | kubectl apply -f -
kubectl create rolebinding "${NS}-config-reader" -n "$NS" \
  --role="${NS}-config-reader" --serviceaccount="${NS}:${NS}" \
  --dry-run=client -o yaml | kubectl apply -f -
echo "RBAC_RC=$?"

# ---------- 4. 重启让 RBAC 生效 ----------
kubectl -n "$NS" delete pod -l app.kubernetes.io/name=argo-rollouts --ignore-not-found=true
sleep 5

# ---------- 5. 等就绪 ----------
for i in $(seq 1 40); do
  ready=$(kubectl -n "$NS" get deploy argo-rollouts -o jsonpath='{.status.availableReplicas}' 2>/dev/null)
  echo "poll ${i}: availableReplicas=${ready:-0}"
  [ "$ready" = "1" ] && break
  sleep 10
done

echo "--- final pod state ---"
kubectl -n "$NS" get pods -o wide
echo "--- final logs (if still failing) ---"
kubectl -n "$NS" logs deploy/argo-rollouts --tail=20 2>&1 || true
echo "=== [$(date -Is)] rollouts namespace fix end ==="
touch /var/log/star-rollouts-fix.complete
