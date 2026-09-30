#!/bin/bash
# =====================================================================
# scripts/verify-5-services.sh — Star Desktop P7 E2E: 5 services HTTP 200 verification
# =====================================================================
# Per docs/architecture/2026-09-29-upgrade/rust-app-end-research.md §3.2 P7
# 验证 5 k3s services 启动 + 端口转发 + HTTP 200 (per session memory: 5 services)
#
# Usage:
#   bash scripts/verify-5-services.sh           # default: verify + cleanup
#   bash scripts/verify-5-services.sh --no-cleanup # verify only (port-forwards stay)
#
# 5 services (per session memory PR #170 + USV000):
#   1. canvas-game       port 12080 → svc 8084 (HTTP /healthz)
#   2. canvas-engine     port 13080 → svc 8080 (HTTP /healthz)
#   3. domain-canvas     port 14080 → svc 8081 (HTTP /healthz)
#   4. canvas-realtime   port 15080 → svc 8082 (HTTP /healthz)
#   5. star-api-rest     port  8081 → svc 8081 (HTTP /api/v1/health)
#
# Requirements:
#   - WSL2 + k3s running (per session memory: wsl --shutdown + restart after cni0 deadlock)
#   - kubectl configured (KUBECONFIG=/etc/rancher/k3s/k3s.yaml)
#   - 5 deployments Running in namespace star-system
# =====================================================================
set -uo pipefail

CLEANUP=true
while [[ $# -gt 0 ]]; do
  case "$1" in
    --no-cleanup) CLEANUP=false; shift ;;
    --help|-h)
      echo "Usage: $0 [--no-cleanup]"
      exit 0 ;;
    *) echo "Unknown arg: $1"; exit 1 ;;
  esac
done

# Services array
SERVICES=(
  "canvas-game|12080|8084|/healthz"
  "canvas-engine|13080|8080|/healthz"
  "domain-canvas|14080|8081|/healthz"
  "canvas-realtime|15080|8082|/healthz"
  "star-api-rest|8081|8081|/api/v1/health"
)

PASS=0
FAIL=0
PID_FNS=""

start_port_forward() {
  local svc="$1"
  local local_port="$2"
  local remote_port="$3"
  local pid_file="/tmp/pf_${svc}.pid"

  echo "Starting port-forward for ${svc}..."
  wsl -d Ubuntu -- bash -c "export KUBECONFIG=/etc/rancher/k3s/k3s.yaml; nohup sudo /usr/local/bin/k3s kubectl port-forward --address=0.0.0.0 -n star-system svc/${svc} ${local_port}:${remote_port} > /tmp/pf_${svc}.log 2>&1 & echo \$!" > "${pid_file}"
  cat "${pid_file}"
}

stop_all() {
  echo ""
  echo "=== Cleanup: killing all port-forwards ==="
  wsl -d Ubuntu -- bash -c "pkill -9 -f port-forward" 2>/dev/null || true
}

verify_service() {
  local svc="$1"
  local local_port="$2"
  local health_path="$3"
  local url="http://localhost:${local_port}${health_path}"

  sleep 3  # wait for port-forward to stabilize

  echo "Verifying ${svc} via ${url}..."
  local http_code
  http_code=$(curl -s -m 5 -o /dev/null -w "%{http_code}" "${url}" 2>/dev/null || echo "000")

  if [[ "${http_code}" == "200" ]]; then
    echo "  ✅ ${svc}: HTTP ${http_code}"
    PASS=$((PASS + 1))
  else
    echo "  ❌ ${svc}: HTTP ${http_code} (expected 200)"
    FAIL=$((FAIL + 1))
  fi
}

# Main flow
echo "=== Step 1: Cleanup existing port-forwards ==="
stop_all
sleep 2

echo ""
echo "=== Step 2: Start port-forwards for 5 services ==="
for entry in "${SERVICES[@]}"; do
  IFS="|" read -r svc local_port remote_port health <<< "${entry}"
  PID=$(start_port_forward "${svc}" "${local_port}" "${remote_port}" 2>/dev/null)
  if [[ -n "${PID}" ]]; then
    echo "  Started ${svc}: PID ${PID} (local ${local_port} → remote ${remote_port})"
  else
    echo "  ❌ Failed to start port-forward ${svc}"
    FAIL=$((FAIL + 1))
  fi
done

sleep 5

echo ""
echo "=== Step 3: Verify 5 services HTTP 200 ==="
for entry in "${SERVICES[@]}"; do
  IFS="|" read -r svc local_port remote_port health_path <<< "${entry}"
  verify_service "${svc}" "${local_port}" "${health_path}"
done

echo ""
echo "=== Summary ==="
echo "  PASS: ${PASS}/5"
echo "  FAIL: ${FAIL}/5"

if [[ "${CLEANUP}" == "true" ]]; then
  echo ""
  stop_all
fi

# Exit code
if [[ "${FAIL}" -gt 0 ]]; then
  exit 1
fi

exit 0
