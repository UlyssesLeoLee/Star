# Sealed Secrets Setup (v0.98 P0-4 Stage 4.2)

> **Status**: 🟡 Active (P0-4 阶段脚本 + 文档, P2 阶段 worker 子代理实跑)
> **Created**: 2026-09-10
> **Authority**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核**
> **关联**: deploy/k3s-local/install-sealed-secrets.sh + deploy/k3s-local/secrets/platform-admin-secret.yaml (per v0.89) + scripts/secrets/platform_admin_password_gen.py (per v0.89)
> **For**: v0.89 已知缺口 (c) 'kubeseal CLI 不在 P0-4 阶段 dev env (P2 阶段 k3s-deployable 安装)' 跨 session 续做

---

## §0 目的

Per 守门 #5 v2 env 安全 + 守门 #13 d T 100% audit, 在 P2 阶段 k3s-deployable 上跑 `deploy/k3s-local/install-sealed-secrets.sh` 安装:
1. **kubeseal CLI** (v0.28.x, 跟 sealed-secrets controller 兼容)
2. **sealed-secrets controller** (per bitnami-labs/sealed-secrets helm chart v2.16.1)
3. **验证** (per 守门 #1 v25, kubectl rollout status)

P0-4 阶段仅提供脚本 + 文档, 实际跑在 P2 阶段 worker 子代理 + k3s-deployable 上 (避免 P0-4 阶段 dev env 误触生产 sealed-secrets controller).

## §1 改动矩阵

| # | 文件 | 行数 | 内容 |
|---|---|---|---|
| 1 | `deploy/k3s-local/install-sealed-secrets.sh` | 3KB (新) | bash 脚本: install kubeseal CLI + helm install sealed-secrets + kubectl rollout verify |
| 2 | `docs/deployment/SEALED-SECRETS-SETUP-001.md` | 6KB (本文件, 新) | 8 段结构 per AGENTS.md §3: §0 目的 + §1 改动矩阵 + §2 验证摘要 + §3 阶段 (P0-4 文档 / P2 实跑) + §4 集成流程 (3 步 install kubeseal + install controller + verify) + §5 守门规则 + §6 签字栏 + §7 修订历史 + §3 已知缺口 4 条 |

## §2 验证摘要

- 1 bash 脚本 (3KB) + 1 文档 (6KB), 0 代码改动
- `cargo test -p star-pg-adapter --lib -j 4` = 35/35 PASS (per v0.85 baseline)
- `cargo check --workspace --lib -j 4` = 0 err
- `cargo fmt --check` = 0
- bash 脚本: `bash -n deploy/k3s-local/install-sealed-secrets.sh` = 0 syntax error
- P2 阶段实跑验证: k3s cluster 上 install + rollout status + get pods

## §3 2 阶段落地

### 3.1 P0-4 阶段 (本 commit 声明)

- bash 脚本 + 文档
- 0 实际跑 (避免 P0-4 阶段 dev env 误触生产 sealed-secrets controller)
- 占位 SEALED_SECRETS_VERSION=0.28.0 + HELM_CHART_VERSION=2.16.1

### 3.2 P2 阶段 (worker 子代理 + k3s-deployable 实跑)

```bash
# 1. 跑脚本 (per §4 集成流程)
bash deploy/k3s-local/install-sealed-secrets.sh

# 2. 验证 sealed-secrets controller 跑起来
kubectl -n kube-system get pods -l app.kubernetes.io/name=sealed-secrets

# 3. 跑 platform_admin_password_gen.py 生成 password + SealedSecret
python scripts/secrets/platform_admin_password_gen.py \
    --output .env.star-platform-admin-password \
    --sealed-secret deploy/k3s-local/secrets/platform-admin-secret.sealed.yaml

# 4. 用 kubeseal CLI 加密 (per v0.89 已知缺口 (b) 走 .env → SealedSecret)
kubeseal --format yaml \
    --secret .env.star-platform-admin-password \
    > deploy/k3s-local/secrets/platform-admin-secret.sealed.yaml

# 5. 部署到 k3s
kubectl apply -f deploy/k3s-local/secrets/platform-admin-secret.sealed.yaml

# 6. 验证 secret 已加密 + application 层读
kubectl get secret platform-admin-db-password -o jsonpath='{.data.password}' | base64 -d
# 必返 32 字节 url-safe 随机 password (per v0.89 Python 脚本)
```

## §4 集成流程 (3 步)

### 4.1 第 1 步: install kubeseal CLI

```bash
# bash 脚本自动:
# 1. 下载 https://github.com/bitnami-labs/sealed-secrets/releases/download/v0.28.0/kubeseal-0.28.0-linux-amd64.tar.gz
# 2. tar -xzf + install 到 /usr/local/bin/kubeseal
# 3. 验证: kubeseal --version
```

### 4.2 第 2 步: install sealed-secrets controller (helm)

```bash
# bash 脚本自动:
# 1. helm repo add bitnami-labs https://charts.bitnami.com/bitnami
# 2. helm upgrade --install sealed-secrets bitnami-labs/sealed-secrets \
#      --namespace kube-system \
#      --version 2.16.1 \
#      --set controller.fullnameOverride=sealed-secrets \
#      --wait
```

### 4.3 第 3 步: verify (per 守门 #1 v25)

```bash
# bash 脚本自动:
# 1. kubectl -n kube-system rollout status deployment/sealed-secrets-controller --timeout=60s
# 2. kubectl -n kube-system get pods -l app.kubernetes.io/name=sealed-secrets
# 3. 期望: 2/2 Running
```

## §5 守门规则 (per AGENTS.md §4)

| # | 守门 | 应用 |
|---|---|---|
| 1 | 禁回溯叙事 | 2 个新文件, 不重写 v0.89 platform-admin-secret.yaml + v0.89 platform_admin_password_gen.py |
| 1 v15 | docs 同步饱和 | 本 doc + WBS row = 第 86 次新事件触发 |
| 5 v2 | env 安全 | bash 脚本不打印 SECRET, 只 log status; kubeseal 加密后才入 git (per v0.89 .gitignore `*.sealed.yaml.tmp`) |
| 11 | 缺标比错标 | P0-4 阶段 0 实际跑 (避免 P0-4 dev env 误触生产 controller), P2 阶段 worker 子代理实跑 |
| 14 v4 | Mavis 审核 author=Ulysses | 5 角色全部 author=Ulysses |

## §6 签字栏

| 角色 | 签字 | 日期 |
|---|---|---|
| 架构师 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核** | 2026-09-10 |
| SRE Lead | (同上) | (同上) |
| 平台 | (同上) | (同上) |
| 评审主持 | (同上) | (同上) |
| PM | (同上) | (同上) |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 |
|---|---|---|---|
| v0.1 | 2026-09-10 20:10 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核** | sealed-secrets 集成脚本 + 文档落档 (per v0.89 已知缺口 (c) 跨 session 续做) |

---

## §3 已知缺口 (per 守门 #11 缺标比错标)

1. **P0-4 阶段 0 实际跑**: bash 脚本 + 文档声明落地, P2 阶段 worker 子代理 + k3s-deployable 实跑. P0-4 阶段不跑避免 P0-4 dev env 误触生产 sealed-secrets controller
2. **kubeseal CLI 版本固定 v0.28.0**: P2 阶段实跑前需 sync sealed-secrets controller 版本 (P0-4 阶段 v0.28.0, 实际可能更新到 v0.30+)
3. **k3s cluster 状态依赖**: bash 脚本假设 kubectl 能连 k3s cluster + helm 已安装 + k8s 1.27+. P2 阶段实跑前需 k3s cluster health check
4. **kubeseal 公私钥对配对**: sealed-secrets controller 用自己生成的公私钥对 (per namespace), P2 阶段首次 install 后 controller 自动生成, P2 阶段 worker 子代理用新公钥加密 (旧 controller key ring 不匹配, P2 阶段 rotate 后需重加密所有 SealedSecret)

## §4 子代理失败接手清单

N/A (本 commit 走 root session 直接实装, 纯 bash 脚本 + docs, 无子代理 dispatch, per 守门 #9 v3 subprocess 替代 RPC)

## §5 守门规则 (per AGENTS.md §4)

详见 §5 表.

## §7 修订历史

详见 §7 表.
