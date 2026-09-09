#!/usr/bin/env bash
# 一键构建 + 导入 + 部署 star-api-rest 到本机 k3s (namespace: star-system)
#
# 前提:
#   - 本机已装 docker (或兼容 CLI)、k3s (自带 containerd + crictl)、kubectl
#   - sudo 对 /usr/local/bin/k3s 和 /usr/local/bin/crictl 免密, 例如:
#       visudo -f /etc/sudoers.d/k3s-local
#       <user> ALL=(root) NOPASSWD: /usr/local/bin/k3s, /usr/local/bin/crictl
#     没配免密的话第 2/3 步会失败并打印手动命令, 自己 sudo 跑一遍即可
#
# 用法: ./deploy/k3s-local/build-and-deploy.sh
# 在 WSL/Linux (k3s 所在环境) 里跑, 不是 Windows PowerShell。

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
IMAGE="star-api-rest:local"   # 必须和 star-api-rest-deploy.yaml 里的 image: 字段一致
MANIFEST="${SCRIPT_DIR}/star-api-rest-deploy.yaml"

BUILD_CTX="$(mktemp -d)"
trap 'rm -rf "${BUILD_CTX}"' EXIT
# 关键坑: 不能直接拿 REPO_ROOT 当 docker build context —— 仓库根目录混了大量
# 与镜像构建无关的内容 (docs/、.git/ 等), 且经 /mnt drvfs 访问巨慢。这里现拼一个
# 只含 Dockerfile 实际需要的 4 项的临时目录 (Cargo.toml/Cargo.lock/rust-toolchain.toml/crates/)。
cp "${REPO_ROOT}/Cargo.toml" "${REPO_ROOT}/Cargo.lock" "${REPO_ROOT}/rust-toolchain.toml" "${BUILD_CTX}/"
cp -r "${REPO_ROOT}/crates" "${BUILD_CTX}/crates"

echo "==> [1/4] docker build -t ${IMAGE}  (context: ${BUILD_CTX}, staged from ${REPO_ROOT})"
docker build -t "${IMAGE}" -f "${SCRIPT_DIR}/Dockerfile" "${BUILD_CTX}"

echo "==> [2/4] 导入镜像到 k3s containerd 的 k8s.io namespace"
# 关键坑: `k3s ctr images import` 默认落 containerd 的 "default" namespace,
# 但 kubelet 只认 "k8s.io" namespace —— 必须显式 -n k8s.io, 否则
# imagePullPolicy: Never 的 Pod 会卡 ErrImageNeverPull。
if ! docker save "${IMAGE}" | sudo -n /usr/local/bin/k3s ctr -n k8s.io images import -; then
  echo "!! 导入失败 (可能 sudo 未免密), 请手动执行:"
  echo "   docker save ${IMAGE} | sudo /usr/local/bin/k3s ctr -n k8s.io images import -"
  exit 1
fi

echo "==> [3/4] 校验 kubelet 侧可见性 (k3s crictl images, 不是 ctr images ls)"
if ! sudo -n /usr/local/bin/k3s crictl images 2>/dev/null | grep -q '^docker.io/library/star-api-rest[[:space:]]\+local[[:space:]]'; then
  echo "!! crictl images 看不到 ${IMAGE}, kubelet 侧不可见, 部署会卡 ErrImageNeverPull"
  echo "   手动核实: sudo /usr/local/bin/k3s crictl images | grep star-api-rest"
  exit 1
fi
echo "   OK: kubelet 可见 ${IMAGE}"

echo "==> [4/4] kubectl apply + 等待 rollout"
# 关键坑: /usr/local/bin/kubectl 是 k3s 的符号链接, 默认读 /etc/rancher/k3s/k3s.yaml
# (root:root 0600), 普通用户直接跑 kubectl 会 permission denied, 且各机器/各 shell
# 不一定配了 KUBECONFIG。sudoers 里对 /usr/local/bin/k3s 免密已是前提, 直接用它。
KUBECTL="sudo -n /usr/local/bin/k3s kubectl"
${KUBECTL} apply -f "${MANIFEST}"
${KUBECTL} -n star-system rollout status deployment/star-api-rest --timeout=120s

echo "==> 完成。Pod 状态:"
${KUBECTL} -n star-system get pods -o wide -l app=star-api-rest
