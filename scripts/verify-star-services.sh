#!/usr/bin/env bash
# Verify the three Star Canvas backend services in an explicitly selected K3s context.
# This script does not provision or select Multipass instances; it uses the supplied kubeconfig context.

set -uo pipefail

if [[ -z "${STAR_K3S_CONTEXT:-}" ]]; then
  echo "ERROR: set STAR_K3S_CONTEXT to the kubeconfig context for the target K3s cluster." >&2
  exit 1
fi

if [[ -z "${STAR_K3S_KUBECONFIG:-}" ]]; then
  echo "ERROR: set STAR_K3S_KUBECONFIG to the kubeconfig file for the target K3s cluster." >&2
  exit 1
fi

if [[ ! -f "$STAR_K3S_KUBECONFIG" ]]; then
  echo "ERROR: kubeconfig file not found: $STAR_K3S_KUBECONFIG" >&2
  exit 1
fi

if ! command -v kubectl >/dev/null 2>&1; then
  echo "ERROR: kubectl was not found in PATH." >&2
  exit 1
fi

if ! command -v curl >/dev/null 2>&1; then
  echo "ERROR: curl was not found in PATH." >&2
  exit 1
fi

KUBECTL_ARGS=(--kubeconfig "$STAR_K3S_KUBECONFIG" --context "$STAR_K3S_CONTEXT")
CLEANUP=true
while [[ $# -gt 0 ]]; do
  case "$1" in
    --no-cleanup) CLEANUP=false; shift ;;
    --help|-h)
      echo "Usage: STAR_K3S_KUBECONFIG=<file> STAR_K3S_CONTEXT=<context> $0 [--no-cleanup]"
      exit 0 ;;
    *) echo "Unknown argument: $1" >&2; exit 1 ;;
  esac
done

NAMESPACE="${STAR_K3S_NAMESPACE:-star-system}"
SERVICES=(
  "canvas-engine|13080|8080|/healthz"
  "domain-canvas|14080|8081|/healthz"
  "canvas-realtime|15080|8082|/healthz"
)

PASS=0
FAIL=0
declare -a PORT_FORWARD_PIDS=()
declare -a PORT_FORWARD_LOGS=()

cleanup() {
  if [[ "${CLEANUP}" == "true" ]]; then
    for pid in "${PORT_FORWARD_PIDS[@]}"; do
      kill "$pid" 2>/dev/null || true
      wait "$pid" 2>/dev/null || true
    done
    for log in "${PORT_FORWARD_LOGS[@]}"; do
      rm -f -- "$log"
    done
  fi
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

echo "Checking K3s context '${STAR_K3S_CONTEXT}' in namespace '${NAMESPACE}'..."
if ! kubectl "${KUBECTL_ARGS[@]}" cluster-info >/dev/null; then
  echo "ERROR: cannot connect to the selected K3s context." >&2
  exit 1
fi

start_port_forward() {
  local service="$1"
  local local_port="$2"
  local remote_port="$3"
  local log

  log="$(mktemp "${TMPDIR:-/tmp}/star-pf-${service}.XXXXXX")"
  PORT_FORWARD_LOGS+=("$log")
  kubectl "${KUBECTL_ARGS[@]}" --namespace "$NAMESPACE" \
    port-forward "service/${service}" "${local_port}:${remote_port}" >"$log" 2>&1 &
  PORT_FORWARD_PIDS+=("$!")
}

verify_service() {
  local service="$1"
  local local_port="$2"
  local path="$3"
  local http_code
  local attempts=0
  local url="http://127.0.0.1:${local_port}${path}"

  while (( attempts < 10 )); do
    http_code="$(curl --silent --max-time 3 -o /dev/null -w '%{http_code}' "$url" 2>/dev/null || true)"
    if [[ "$http_code" == "200" ]]; then
      echo "  PASS ${service}: HTTP ${http_code}"
      PASS=$((PASS + 1))
      return
    fi
    sleep 1
    attempts=$((attempts + 1))
  done

  echo "  FAIL ${service}: HTTP ${http_code:-000} (expected 200)" >&2
  FAIL=$((FAIL + 1))
}

echo "Starting port forwards for three Canvas services..."
for entry in "${SERVICES[@]}"; do
  IFS='|' read -r service local_port remote_port health_path <<< "$entry"
  start_port_forward "$service" "$local_port" "$remote_port"
done

for entry in "${SERVICES[@]}"; do
  IFS='|' read -r service local_port remote_port health_path <<< "$entry"
  verify_service "$service" "$local_port" "$health_path"
done

echo "Summary: ${PASS}/3 passed, ${FAIL}/3 failed."
if (( FAIL > 0 )); then
  exit 1
fi

echo "star-api-rest is separate and is not counted in this Canvas deployment verification."
