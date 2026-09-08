#!/usr/bin/env bash
# scripts/automation/helm_canary_mock.sh — Helm 灰度/回滚 mock
# (per docs/requirements/SRS-STAR-OPS-001.md §4 F-01 + docs/basic-design/OPS-BASIC-DESIGN-001.md §3.1)
#
# 守门 #1 R-05: 不动生产 (K8s cluster / 真实 helm), 仅 mock
# 守门 #19 v19: agent 跟外部交互走 scripts/automation/<purpose>.sh
# 守门 #24 v2: subprocess 替代 RPC, 可重放可观测
# 守门 #1 R-05: 真实 K8s 连接 / helm exec 不在 F-01 范围
#
# 子命令 (互斥):
#   list                        - 列出所有 mock release
#   canary   -r <name> -w <0-100> [-R <revision>]
#                               - 模拟 helm upgrade --canary --weight W
#   rollback -r <name> -R <revision>
#                               - 模拟 helm rollback <name> <revision>
#   status   -r <name>          - 模拟 kubectl get pods / helm status
#
# 输出: 全部 JSON 到 stdout (守门 #24 v2: 解析 stdout JSON)
# 退出码: 0 = 成功, 1 = 参数错, 2 = release 不存在
#
# 用法:
#   bash scripts/automation/helm_canary_mock.sh list
#   bash scripts/automation/helm_canary_mock.sh canary -r star-mcp -w 10
#   bash scripts/automation/helm_canary_mock.sh canary -r star-mcp -w 50 -R 4
#   bash scripts/automation/helm_canary_mock.sh rollback -r star-mcp -R 2
#   bash scripts/automation/helm_canary_mock.sh status -r star-mcp
set -euo pipefail

# ============ 颜色 (守门 #5: 输出友好, CI 也能看) ============
readonly C_RESET=$'\033[0m'
readonly C_INFO=$'\033[36m'
readonly C_OK=$'\033[32m'
readonly C_ERR=$'\033[31m'
readonly C_WARN=$'\033[33m'

# ============ Mock release 数据库 (in-memory, per 守门 #1 R-05) ============
# 真实场景会从 kube-rs 调 Secret/ConfigMap, mock 阶段硬编码 3 个 release
MOCK_RELEASES=(
  "star-mcp|default|star-mcp-0.1.0|3|healthy"
  "star-api|default|star-api-0.2.1|7|degraded"
  "star-warehouse|staging|star-warehouse-1.0.0|12|failed"
)

# ============ helper ============
log_info() { echo "${C_INFO}[helm-canary-mock]${C_RESET} $*" >&2; }
log_ok()   { echo "${C_OK}[helm-canary-mock OK]${C_RESET} $*" >&2; }
log_err()  { echo "${C_ERR}[helm-canary-mock ERR]${C_RESET} $*" >&2; }

# mock release 查表
find_release() {
  local name="$1"
  for row in "${MOCK_RELEASES[@]}"; do
    IFS='|' read -r r_name r_ns r_chart r_rev r_status <<< "$row"
    if [ "$r_name" = "$name" ]; then
      echo "$r_name|$r_ns|$r_chart|$r_rev|$r_status"
      return 0
    fi
  done
  return 1
}

now_iso() {
  # ISO-8601 UTC, 跨平台 (Linux date -u /date)
  if date -u +"%Y-%m-%dT%H:%M:%SZ" 2>/dev/null; then
    return
  fi
  # WSL/某些 bash 3.x 兼容
  date -u +"%Y-%m-%dT%H:%M:%SZ"
}

gen_id() {
  # 16 字符 hex 随机 (类似 UUIDv4 后缀), bash-only
  od -An -tx1 -N8 /dev/urandom 2>/dev/null | tr -d ' \n' || echo "0000000000000000"
}

# ============ 子命令 ============
cmd_list() {
  local json_rows=""
  for row in "${MOCK_RELEASES[@]}"; do
    IFS='|' read -r r_name r_ns r_chart r_rev r_status <<< "$row"
    [ -n "$json_rows" ] && json_rows+=","
    json_rows+=$(cat <<EOF
{
  "name": "${r_name}",
  "namespace": "${r_ns}",
  "chart": "${r_chart}",
  "revision": ${r_rev},
  "status": "${r_status}",
  "updated_at": "$(now_iso)",
  "canary_weight": 0
}
EOF
)
  done
  cat <<EOF
{
  "command": "list",
  "ok": true,
  "count": ${#MOCK_RELEASES[@]},
  "releases": [${json_rows}],
  "generated_at": "$(now_iso)",
  "channel": "mock"
}
EOF
}

cmd_canary() {
  local release="" weight="" revision=""
  while [ $# -gt 0 ]; do
    case "$1" in
      -r|--release)   release="$2"; shift 2 ;;
      -w|--weight)    weight="$2";  shift 2 ;;
      -R|--revision|--target)  revision="$2"; shift 2 ;;
      *) log_err "canary: 未知参数 $1"; return 1 ;;
    esac
  done
  if [ -z "$release" ] || [ -z "$weight" ]; then
    log_err "canary: -r <release> -w <weight> 必填"
    return 1
  fi
  # 守门 #5 v2: 数值校验
  if ! [[ "$weight" =~ ^[0-9]+$ ]] || [ "$weight" -gt 100 ]; then
    log_err "canary: weight 必为 0-100 整数, got '$weight'"
    return 1
  fi
  if [ -z "$revision" ]; then revision="latest"; fi

  if ! find_release "$release" >/dev/null; then
    log_err "canary: release '$release' 不存在 (mock 阶段仅支持 star-mcp/star-api/star-warehouse)"
    return 2
  fi

  local action_id
  action_id=$(gen_id)

  log_ok "canary $release → weight ${weight}%, revision $revision"

  cat <<EOF
{
  "command": "canary",
  "ok": true,
  "action_id": "${action_id}",
  "release": "${release}",
  "canary_weight": ${weight},
  "target_revision": "${revision}",
  "status": "pending",
  "created_at": "$(now_iso)",
  "channel": "mock"
}
EOF
}

cmd_rollback() {
  local release="" revision=""
  while [ $# -gt 0 ]; do
    case "$1" in
      -r|--release)   release="$2"; shift 2 ;;
      -R|--revision|--target)  revision="$2"; shift 2 ;;
      *) log_err "rollback: 未知参数 $1"; return 1 ;;
    esac
  done
  if [ -z "$release" ] || [ -z "$revision" ]; then
    log_err "rollback: -r <release> -R <revision> 必填"
    return 1
  fi
  if ! [[ "$revision" =~ ^[0-9]+$ ]] || [ "$revision" -eq 0 ]; then
    log_err "rollback: revision 必为正整数, got '$revision'"
    return 1
  fi
  if ! find_release "$release" >/dev/null; then
    log_err "rollback: release '$release' 不存在"
    return 2
  fi

  local action_id
  action_id=$(gen_id)

  log_ok "rollback $release → revision $revision"

  cat <<EOF
{
  "command": "rollback",
  "ok": true,
  "action_id": "${action_id}",
  "release": "${release}",
  "target_revision": ${revision},
  "status": "pending",
  "created_at": "$(now_iso)",
  "channel": "mock"
}
EOF
}

cmd_status() {
  local release=""
  while [ $# -gt 0 ]; do
    case "$1" in
      -r|--release) release="$2"; shift 2 ;;
      *) log_err "status: 未知参数 $1"; return 1 ;;
    esac
  done
  if [ -z "$release" ]; then
    log_err "status: -r <release> 必填"
    return 1
  fi
  if ! find_release "$release" >/dev/null; then
    log_err "status: release '$release' 不存在"
    return 2
  fi

  log_ok "status $release"

  # Mock replicas: 默认 3 ready / 3 desired; star-api degraded → 1/3; star-warehouse failed → 0/2
  local phase="Healthy" ready=3 desired=3
  case "$release" in
    star-api)        phase="Degraded"; ready=1; desired=3 ;;
    star-warehouse)  phase="Failed";    ready=0; desired=2 ;;
  esac

  cat <<EOF
{
  "command": "status",
  "ok": true,
  "release": "${release}",
  "phase": "${phase}",
  "replicas": {
    "ready": ${ready},
    "desired": ${desired}
  },
  "checked_at": "$(now_iso)",
  "channel": "mock"
}
EOF
}

cmd_help() {
  cat <<'EOF'
helm_canary_mock.sh — F-01 集群子域 mock (守门 #1 R-05)
用法:
  list                                    列出 mock release
  canary  -r <name> -w <0-100> [-R <rev>] 模拟 helm upgrade --canary
  rollback -r <name> -R <revision>        模拟 helm rollback
  status  -r <name>                       模拟 kubectl get pods
  help                                    本帮助
EOF
}

# ============ main ============
if [ $# -eq 0 ]; then
  cmd_help; exit 1
fi
subcmd="$1"; shift

case "$subcmd" in
  list)     cmd_list "$@" ;;
  canary)   cmd_canary "$@" ;;
  rollback) cmd_rollback "$@" ;;
  status)   cmd_status "$@" ;;
  help|-h|--help) cmd_help; exit 0 ;;
  *)
    log_err "未知子命令: $subcmd (list|canary|rollback|status|help)"
    cmd_help >&2
    exit 1
    ;;
esac
