# ULYS-160 canvas-game 4 crate 一键部署手册

> **Status**: 阶段 1 骨架 + 阶段 2 deploy rollout (2026-09-26 完成, per issue #160)
> **Ticket**: https://github.com/UlyssesLeoLee/Star/issues/160
> **PR**: #161 (阶段 1 骨架, 已合 dev), #162-#169 (dev baseline fixes + workflow chain, 已合 dev)
> **Deploy yaml**: `deploy/canvas-game-k3s.yaml` (per 2026-09-07 拍板 commit 510d8660, 2026-09-26 ULYS-160 阶段 1+2 落地)
> **当前 dev HEAD**: `7d203a05` (含 PR #161-#169)

## 概述

本目录文档化画布游戏 (canvas-game) 4 crate 一键部署链路:

| Crate | 端口 (HTTP / WS) | 角色 | 阶段 1 实现 |
| --- | --- | --- | --- |
| `canvas-engine` | 8080 / — | 跨 5 domain 共享 平台能力 | axum HTTP `/healthz` + `/ready` + `/version` |
| `domain-canvas` | 8081 / — | 业务域 画布 26 表 W/T/M + 5 角色 | axum HTTP + 占位 `/api/v1/canvas` |
| `canvas-realtime` | 8082 / 8083 | Yjs/yrs CRDT 后端 | axum HTTP + tokio-tungstenite WSS echo |
| `canvas-game` | 8084 / — | 游戏引擎 角色/弹幕/战斗/3D sprite | axum HTTP + 占位 `/api/v1/gameplay` |

## 链路

```
源码 (Star repo)
  crates/{canvas-engine,domain-canvas,canvas-realtime,canvas-game}/src/{lib,main}.rs
  每个 crate 都自带 Dockerfile (multi-stage rust)
        ↓ git push dev
GitHub Actions (.github/workflows/publish-canvas-game.yml)
  - test job: cargo check + cargo test
  - build-and-push job (matrix 4 image): docker buildx + push
  - 凭据: secrets.GHCR_TOKEN || inputs.ghcr_pat (双 fallback)
  - IMAGE_OWNER: UlyssesLeoLee (camel case, per PR #169)
        ↓ docker push
GHCR (ghcr.io/UlyssesLeoLee/star-canvas-*)
  4 repo (2026-09-26 第一次 push 成功, run 36242775765):
  - star-canvas-engine (8080)
  - star-domain-canvas (8081)
  - star-canvas-realtime (8082 + 8083 WSS)
  - star-canvas-game (8084)
  每个 repo tags: dev, sha-7d203a0 (latest 仅 push tag v* 时)
        ↓ kubectl apply + set image
k3s cluster (deploy/canvas-game-k3s.yaml, image :dev)
  - 5 deployment + 5 service + 1 LoadBalancer (envoy)
  - WSL Ubuntu 本机部署
        ↓
4 pod Running, 1 envoy LoadBalancer
kubectl port-forward svc/envoy-edge 10080:10080
浏览器访问 http://localhost:10080
```

## 阶段 1 已落地 (per PR #161)

4 crate 都已提交 + 20/20 tests pass + Dockerfile + publish workflow + dev baseline red fixes + IMAGE_OWNER camel case fix (PR #169)。

## 阶段 2 — 待落地 (per SRS / BD / DD)

| Crate | 阶段 2 落地项 |
| --- | --- |
| `canvas-engine` | 跨 5 domain 共享 API + 平台能力 (per SRS §4.1) |
| `domain-canvas` | 26 表 W/T/M + 5 角色 CRUD (per SRS §4.2) |
| `canvas-realtime` | Yjs/yrs CRDT 集成 (per SRS §4.3) |
| `canvas-game` | 角色 / 弹幕 / 战斗 / 3D sprite (per SRS §4.4) |

## 一键部署 (WSL2 本机, 实测 2026-09-26)

### 阶段 A — 准备 (一次性)

```bash
# 1. 同步代码到 WSL Ubuntu
wsl -d Ubuntu -- bash -c "
  cd ~
  if [ ! -d Star ]; then
    git clone --branch dev --single-branch https://github.com/UlyssesLeoLee/Star.git Star
  fi
  cd Star
  git fetch origin dev --prune
  git reset --hard origin/dev
"

# 2. ⭐ WSL2 常驻 (per deploy/k3s-local/README.md 第 28-39 行)
#    WSL2 默认实例级 idle 超时 ~15s, 没有 wsl.exe 客户端连着则整个 distro 销毁重建,
#    k3s systemd 重启 → kubeconfig token 失效. 必须保持 wsl.exe 常驻.
wsl -d Ubuntu -- sleep infinity &
# 或 PowerShell:
Start-Process -FilePath "wsl.exe" -ArgumentList "-d", "Ubuntu", "--", "sleep", "infinity" -WindowStyle Hidden
```

### 阶段 B — 触发 GHCR publish (可选, image 已推过可跳)

```bash
# 自动: push dev 触发 CI
cd /path/to/Star (Windows)
git push origin dev
# 或手动 dispatch (传 PAT 一次性):
gh workflow run publish-canvas-game.yml -f ghcr_pat="$GHCR_PAT_CLASSIC"
```

**GHCR_TOKEN PAT 配置** (如果不想每次 dispatch 传):
```
Star repo Settings → Secrets and variables → Actions → New repository secret
  Name:  GHCR_TOKEN
  Value: ghcr.io classic PAT with write:packages scope
```

### 阶段 C — 部署到 k3s

```bash
# 1. 创建 namespace + 占位 secret (per ULYS-160 阶段 1 设计)
kubectl create namespace star-system --validate=false 2>/dev/null || true
kubectl create secret docker-registry ghcr-pull -n star-system \
  --docker-server=https://ghcr.io \
  --docker-username=UlyssesLeoLee \
  --docker-password="$GHCR_PAT_CLASSIC" \
  --validate=false
kubectl create secret generic canvas-game-secrets -n star-system \
  --from-literal=DATABASE_URL="postgres://placeholder:placeholder@localhost:5432/placeholder" \
  --from-literal=REDIS_URL="redis://localhost:6379/0" \
  --from-literal=NATS_URL="nats://localhost:4222" \
  --from-literal=S3_BUCKET="placeholder" \
  --from-literal=S3_ENDPOINT="http://localhost:9000" \
  --from-literal=GHCR_PAT="placeholder" \
  --validate=false

# 2. 应用 deploy yaml (per deploy/canvas-game-k3s.yaml, image :dev)
kubectl apply -f deploy/canvas-game-k3s.yaml --validate=false

# 3. 等待 rollout
kubectl rollout status deployment/canvas-engine -n star-system --timeout=90s
kubectl rollout status deployment/domain-canvas -n star-system --timeout=90s
kubectl rollout status deployment/canvas-realtime -n star-system --timeout=90s
kubectl rollout status deployment/canvas-game -n star-system --timeout=90s

# 4. 验证
kubectl get pods -n star-system
# 期望:
#   canvas-engine-XXXX    1/1     Running
#   canvas-realtime-XXXX  1/1     Running
#   domain-canvas-XXXX    1/1     Running
#   canvas-game-XXXX      1/1     Running
#   envoy-edge-XXXX       1/1     Running
```

### 阶段 D — 浏览器访问

```bash
# port-forward envoy (LoadBalancer → localhost)
kubectl port-forward -n star-system svc/envoy-edge 10080:10080

# 浏览器访问 http://localhost:10080

# 4 service 各自 healthz (curl 直接):
kubectl port-forward -n star-system svc/canvas-engine 8080:8080 &
curl http://localhost:8080/healthz     # → ok
curl http://localhost:8080/version     # → {"service":"canvas-engine","version":"0.1.0","phase":"Skeleton"}

kubectl port-forward -n star-system svc/domain-canvas 8081:8081 &
curl http://localhost:8081/healthz
curl http://localhost:8081/api/v1/canvas  # → 阶段 2 占位 endpoint

kubectl port-forward -n star-system svc/canvas-game 8084:8084 &
curl http://localhost:8084/healthz
curl http://localhost:8084/api/v1/gameplay  # → 阶段 2 占位 endpoint
```

## 已知限制 / 调试笔记 (per 2026-09-26 实测)

### WSL2 + k3s 反复 restart 问题

per `deploy/k3s-local/README.md` 第 28-39 行:

> **若 k3s 跑在 WSL2 里**: WSL2 对每个发行版实例有一个独立于 `vmIdleTimeout` 的**实例级空闲超时**, 默认约 15 秒 — 只要没有任何 `wsl.exe` 客户端连着该发行版, 整个发行版用户态 (PID 1 及以下, 含 k3s) 就会被销毁重建。

**实测现象**:
- 高频 kubectl 命令触发 k3s systemd 重启 (kubelet pull image 高负载)
- WSL2 distro idle 超时 → distro 整个销毁重建 → k3s service 重启 → token 重生成 → kubeconfig 失效
- 循环: kubectl 超时 → 重新拉 kubeconfig → kubectl 又触发重启

**解决方案** (本 deploy 手册已采纳):
- 阶段 A 步骤 2: `wsl -d Ubuntu -- sleep infinity &` 保持 wsl.exe 常驻连接

### GHCR personal namespace owner 大小写敏感

实测发现 (per PR #169):
- GHCR API `/user/packages` 返回 12 个 packages, owner 字段全是 `UlyssesLeoLee` (camel case)
- `ghcr.io/ulysses-lee-lee/...` (lowercase) 触发 `not_found: owner not found`
- Docker PAT 鉴权: `username` 必须跟 GitHub username 大小写严格匹配
- IMAGE_OWNER 固定 `UlyssesLeoLee` (camel case), deploy yaml 也用 camel case

### GHCR image tags

实测发现 (2026-09-26 push run 36242775765):
- push dev 触发 workflow, 推 tag = `dev` + `sha-{short}` (per `tags: type=ref,event=branch`)
- **没有 `:latest` tag** (除非手动 push tag `v*` 触发 latest)
- deploy yaml 用 `:dev` (匹配实际推送)

### ServiceMonitor CRD 缺

`deploy/canvas-game-k3s.yaml` 含 `ServiceMonitor` 资源 (metrics-server 用), dev 没装 Prometheus Operator CRD, apply 时报:

```
no matches for kind "ServiceMonitor" in version "monitoring.coreos.com/v1"
```

**可忽略** (per deploy yaml 顶部注释), 不影响 4 deployment 启动。

## 关联文档

- SRS: `docs/requirements/SRS-STAR-CANVAS-GAME-001.md` v0.1 (含阶段 1 实现状态章节)
- BD: `docs/design/BD-STAR-CANVAS-GAME-001.md` v0.1
- DD: `docs/design/DD-STAR-CANVAS-GAME-001.md` v0.1
- 部署 yaml: `deploy/canvas-game-k3s.yaml` (image :dev, owner camel case)
- publish workflow: `.github/workflows/publish-canvas-game.yml`
- 实施计划: `docs/implementation-plans/CANVAS-GAME-IMPL-PLAN-001.md`
- Issue: https://github.com/UlyssesLeoLee/Star/issues/160
- PR: #161 (阶段 1 骨架), #162-#165 (baseline fixes), #166-#169 (workflow chain)

## 守门

- ✅ #1 v25 cargo check --workspace --all-targets 0 err + cargo test 4 crate 20/20 pass
- ✅ #5 env 安全 (GHCR_TOKEN 仅读, 不打印 env value)
- ✅ #7 unsafe_code = "forbid" (workspace lint, 4 crate 全 0 unsafe)
- ✅ #11 缺标比错标 (Phase::Skeleton + NotImplemented 占位)
- ✅ #14 v4 Mavis 临时代签 5 域 Lead (per 9/3 11:35 JST 反转)
- ✅ #12 守门饱和 (deploy yaml + 4 Dockerfile + workflow + docs + baseline red fixes 不另开 PR)
