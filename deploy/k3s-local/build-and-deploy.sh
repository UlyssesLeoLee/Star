#!/usr/bin/env bash
# @cypher schema=1 source_sha256=b514faceeb142172fc7b0ddae75777956e730ab34b64a125339cfed6aa05b562
# MERGE (self:File {path:"deploy/k3s-local/build-and-deploy.sh"})
# MERGE (context_generator:File {path:"scripts/automation/star_k3s_context.py"})
# MERGE (self)-[:DEPENDS_ON]->(context_generator)
# @endcypher
# 一键在明确选择的 Multipass guest 中构建 + 导入 + 部署 star-api-rest。
#
# 前提:
#   - 已安装 Multipass CLI；所选 guest 必须已存在且正在运行
#   - guest 内已配置 Docker build engine、K3s server 和非交互式 sudo
#   - 此脚本不会创建/启动/停止/删除 VM，不会安装 guest 工具，也不会使用宿主 Docker/K3s
#
# 用法: ./deploy/k3s-local/build-and-deploy.sh <multipass-instance> [image-tag]
# 例: ./deploy/k3s-local/build-and-deploy.sh star-k3s-test v1-local
# 该脚本在能访问 Multipass CLI 的 Linux/WSL shell 中运行。

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
if [[ $# -lt 1 || $# -gt 2 || -z "${1:-}" ]]; then
  echo "用法: $0 <multipass-instance> [image-tag]" >&2
  exit 2
fi

INSTANCE="$1"
IMAGE_TAG="${2:-local}"
echo "==> 目标 guest: ${INSTANCE}；构建镜像: star-api-rest:${IMAGE_TAG}"
echo "==> 仅从已验证的 Star Cargo 闭包生成上下文；构建、导入、部署均在该 guest 内执行。"
python3 "${REPO_ROOT}/scripts/automation/star_k3s_context.py" \
  --repo "${REPO_ROOT}" \
  --build-in-multipass "${INSTANCE}" \
  --image-tag "${IMAGE_TAG}"
