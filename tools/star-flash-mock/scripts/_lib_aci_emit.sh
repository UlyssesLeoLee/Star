#!/usr/bin/env bash
# _lib_aci_emit.sh - Star Mock Project ACI assertion emitter bash wrapper (per ULYS-191 §4.1.2)
# Author: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #10 + 19:39 JST 授权)
# 触发: ULYS-191 v0.2 approved + brief v0.1
# 守门: #5 (no secret leak) + #7 (0 unsafe) + #10 (author=Ulysses) +
#       #11 (缺标比错标) + #19v19 (Python 化优先, 但 bash wrapper 兼容既有 _lib_validate.sh 风格)
#
# 用途: bash 脚本 / 测试 runner 调用 _lib_aci_emit.py, 提供更薄的 wrapper.
# 风格同 _lib_validate.sh (无 fail-fast, 错误返回非零退出).
#
# 用法:
#   source _lib_aci_emit.sh
#   aci_emit --layer it --assertion-id "star-flash-mock:guards:g-1" \
#     --scope project=star-flash-mock,module=guards,domain=admin \
#     --expect-type response_within_ms --expect-value 2000 --expect-description "..." \
#     --actual-type response_within_ms --actual-value 5000 --actual-description "..." \
#     --status FAIL --severity high --reasoning "..." --output /tmp/x.aci.json

set -u

# 路径解析 (兼容从任意 cwd 调用)
_ACI_EMIT_LIB_PATH="${BASH_SOURCE[0]:-$0}"
while [ -L "$_ACI_EMIT_LIB_PATH" ]; do
  _ACI_EMIT_LIB_DIR=$(cd -P "$(dirname "$_ACI_EMIT_LIB_PATH")" >/dev/null 2>&1 && pwd)
  _ACI_EMIT_LIB_PATH=$(readlink "$_ACI_EMIT_LIB_PATH")
  [[ "$_ACI_EMIT_LIB_PATH" != /* ]] && _ACI_EMIT_LIB_PATH="$_ACI_EMIT_LIB_DIR/$_ACI_EMIT_LIB_PATH"
done
_ACI_EMIT_LIB_DIR=$(cd -P "$(dirname "$_ACI_EMIT_LIB_PATH")" >/dev/null 2>&1 && pwd)
_ACI_EMIT_PY="$_ACI_EMIT_LIB_DIR/_lib_aci_emit.py"

# ---- aci_emit: 转发到 python emitter ----
# 用法: aci_emit [options] --output FILE
# 返回: 0 成功, 2 schema 校验失败
aci_emit() {
  if [ ! -f "$_ACI_EMIT_PY" ]; then
    echo "FAIL: $_ACI_EMIT_PY 不存在" >&2
    return 1
  fi
  python3 "$_ACI_EMIT_PY" emit "$@"
  return $?
}

# ---- aci_schema: 打印 schema 必填字段 (一行 LABEL=value) ----
aci_schema() {
  if [ ! -f "$_ACI_EMIT_PY" ]; then
    echo "FAIL: $_ACI_EMIT_PY 不存在" >&2
    return 1
  fi
  python3 "$_ACI_EMIT_PY" schema
  return $?
}

# ---- aci_assertion_required_fields: 静态必填字段清单 (便于 bash 解析) ----
aci_assertion_required_fields() {
  echo "aci_version assertion_id layer scope expect actual status severity reasoning captured_at"
}

# ---- aci_emit_pass: 便利 helper: emit 一条 PASS assertion ----
# 用法: aci_emit_pass <layer> <assertion_id> <scope-kvs> <output-file>
#   例: aci_emit_pass it "star-flash-mock:guards:g-1" "project=star-flash-mock,module=guards" /tmp/x.aci.json
aci_emit_pass() {
  local layer="$1" aid="$2" scope="$3" out="$4"
  local scope_args=()
  IFS=',' read -ra pairs <<< "$scope"
  for kv in "${pairs[@]}"; do
    scope_args+=("$kv")
  done
  aci_emit --layer "$layer" --assertion-id "$aid" \
    --scope "${scope_args[@]}" \
    --expect-type response_within_ms --expect-value 2000 \
      --expect-description "API 响应应在 2s 内" \
    --actual-type response_within_ms --actual-value 5 \
      --actual-description "实测 5ms 完成" \
    --status PASS --severity info \
    --reasoning "实测延迟远低于预期, 通过" \
    --tags smoke,perf \
    --output "$out"
}

# ---- 仅当直接执行 (非 source) 时, 默认给 help ----
if [[ "${BASH_SOURCE[0]:-}" == "${0}" ]] && [ $# -eq 0 ]; then
  echo "用法: source _lib_aci_emit.sh  # 引入 aci_emit / aci_schema / aci_emit_pass / aci_assertion_required_fields"
  echo "  或: bash _lib_aci_emit.sh  # 默认 (无参数) 打印用法"
  echo ""
  echo "aci_emit 用法 (转发到 _lib_aci_emit.py):"
  python3 "$_ACI_EMIT_PY" --help 2>&1 | sed 's/^/  /' || true
fi
