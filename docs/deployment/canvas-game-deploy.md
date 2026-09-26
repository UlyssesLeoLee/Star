# ULYS-160 canvas-game 4 crate 一键部署手册

> **Status**: 阶段 1 骨架 (2026-09-26 完成, per issue #160)
> **Ticket**: https://github.com/UlyssesLeoLee/Star/issues/160
> **PR**: (即将开 PR 合 dev)
> **部署 yaml**: `deploy/canvas-game-k3s.yaml` (per 2026-09-07 拍板 commit 510d8660, 2026-09-26 ULYS-160 阶段 1 落地)

## 概述

本目录文档化画布游戏 (canvas-game) 4 crate 的一键部署链路:

| Crate | 端口 (HTTP / WS) | 角色 | 阶段 1 实现 |
| --- | --- | --- | --- |
| `canvas-engine` | 8080 / — | 跨 5 domain 共享 平台能力 | axum HTTP `/healthz` + `/ready` + `/version` |
| `domain-canvas` | 8081 / — | 业务域 画布 26 表 W/T/M + 5 角色 | axum HTTP + 占位 `/api/v1/canvas` |
| `canvas-realtime` | 8082 / 8083 | Yjs/yrs CRDT 后端 | axum HTTP + tokio-tungstenite WSS echo |
| `canvas-game` | 8084 / — | 游戏引擎 角色/弹幕/战斗/3D sprite | axum HTTP + 占位 `/api/v1/gameplay` |

## 链路

```
┌─────────────────────────────────────────────────────────────────────┐
│ 1. 源码 (Star repo)                                                  │
│    crates/canvas-engine/src/{lib,main}.rs                          │
│    crates/domain-canvas/src/{lib,main}.rs                          │
│    crates/canvas-realtime/src/{lib,main}.rs                        │
│    crates/canvas-game/src/{lib,main}.rs                           │
│    每个 crate 都自带 Dockerfile (multi-stage rust)                    │
└─────────────────────────────────────────────────────────────────────┘
                              ↓ git push dev
┌─────────────────────────────────────────────────────────────────────┐
│ 2. GitHub Actions (.github/workflows/publish-canvas-game.yml)       │
│    - test job: cargo check + cargo test (4 crate 单测)              │
│    - build-and-push job (matrix 4 image): docker buildx + push       │
│    - 触发: push main/dev + push tag v* + workflow_dispatch            │
│    - 凭据: secrets.GHCR_TOKEN (classic PAT, packages:write)         │
└─────────────────────────────────────────────────────────────────────┘
                              ↓ docker push
┌─────────────────────────────────────────────────────────────────────┐
│ 3. GHCR (ghcr.io/ulysses-lee-lee/star-canvas-*)                      │
│    4 个 repo, 每个 repo 多 tag: latest + v0.1 + sha-xxx              │
│    私有不公开 (per namespace ulysses-lee-lee 默认)                   │
└─────────────────────────────────────────────────────────────────────┘
                              ↓ kubectl apply
┌─────────────────────────────────────────────────────────────────────┐
│ 4. k3s cluster (deploy/canvas-game-k3s.yaml)                        │
│    - 5 deployment + 5 service + 1 LoadBalancer (envoy)               │
│    - docker-registry secret ghcr-pull 拉镜像认证                      │
│    - 启动需先: 创建 namespace + 占位 secret + kubectl apply            │
│    - WSL Ubuntu 本机部署 (per README §WSL2 idle 超时需保持 wsl.exe   │
│      常驻连接)                                                       │
└─────────────────────────────────────────────────────────────────────┘
                              ↓
                   4 个 pod Running, 1 个 envoy LoadBalancer
```

## 阶段 1 骨架 — 已落地

4 个 crate 都已提交,各自实现:

- ✅ `ServiceMetadata` struct (name / version / http_port / phase)
- ✅ `Phase` enum (Skeleton / Implemented, serde 序列化)
- ✅ `router()` 函数返回 axum::Router
- ✅ `main.rs` 启动 axum server 监听 ServiceMetadata.http_port
- ✅ 5 个单元测试 (tests/smoke.rs) — service metadata + Phase serialize + axum router healthz/version
- ✅ Dockerfile (multi-stage rust:slim-bookworm builder + debian:bookworm-slim runtime)
- ✅ publish-canvas-game.yml (4-image matrix + 推 :latest + :v0.1 + :sha-xxx)

## 阶段 2 — 待落地 (per SRS-STAR-CANVAS-GAME-001 v0.1)

| Crate | 阶段 2 落地项 |
| --- | --- |
| `canvas-engine` | 跨 5 domain 共享 API + 平台能力 (per SRS §4.1) |
| `domain-canvas` | 26 表 W/T/M + 5 角色 CRUD (per SRS §4.2) |
| `canvas-realtime` | Yjs/yrs CRDT 集成 (per SRS §4.3) |
| `canvas-game` | 角色 / 弹幕 / 战斗 / 3D sprite (per SRS §4.4) |

每个阶段 2 落地都是单独 PR, 不在本 ULYS-160 PR scope。

## 一键部署 (WSL2 本机)

```bash
# 1. 同步代码到 WSL Ubuntu
wsl -d Ubuntu -- bash -c 'cd ~/Star && git pull origin dev'

# 2a. 触发 GHCR publish workflow (push dev 时自动触发, 或手动 dispatch)
gh workflow run publish-canvas-game.yml -f ghcr_pat="$GHCR_PAT_CLASSIC"
# 注: workflow 加了 `inputs.ghcr_pat` fallback (per ULYS-160 PR #166 followup),
# 因为 secrets.GHCR_TOKEN 在 repo settings 不存在. 设了 secret 后用:
gh workflow run publish-canvas-game.yml
# 也可直接 push dev 让 workflow 自动跑 (默认 trigger)

# 3. 等镜像推到 GHCR 后, 部署到 k3s
cd ~/Star
kubectl apply -f deploy/canvas-game-k3s.yaml --validate=false

# 4. 验证
kubectl get pods -n star-system -l app=canvas-engine
kubectl get pods -n star-system -l app=canvas-game
kubectl port-forward -n star-system svc/envoy-edge 10080:10080
# 浏览器访问 http://localhost:10080 看 envoy 入口
```

## 已知限制

- **WSL2 idle 超时**: per `deploy/k3s-local/README.md` 第 28-39 行, 需保持 `wsl -d Ubuntu -- sleep infinity` 常驻连接防止发行版被拆
- **GHCR PAT 权限**: 需要 classic PAT with `packages:write` scope, secrets.GHCR_TOKEN 在 GitHub repo settings
- **ServiceMonitor CRD 缺**: `deploy/canvas-game-k3s.yaml` 含 `ServiceMonitor` 资源 (metrics-server 用), dev 没装 Prometheus Operator CRD, apply 时报 `no matches for kind "ServiceMonitor"`, 可忽略

## 关联文档

- SRS: `docs/requirements/SRS-STAR-CANVAS-GAME-001.md` v0.1
- BD: `docs/design/BD-STAR-CANVAS-GAME-001.md` v0.1
- DD: `docs/design/DD-STAR-CANVAS-GAME-001.md` v0.1
- 部署 yaml: `deploy/canvas-game-k3s.yaml`
- publish workflow: `.github/workflows/publish-canvas-game.yml`
- Issue: https://github.com/UlyssesLeoLee/Star/issues/160

## 守门

- ✅ #1 v25 cargo check --workspace --all-targets 0 err
- ✅ #1 v25 cargo test -p 4 crates 20/20 pass
- ✅ #5 env 安全 (GHCR_TOKEN 仅读, 不打印)
- ✅ #7 unsafe_code = "forbid" (workspace lint 全局 0 unsafe)
- ✅ #11 缺标比错标 (NotImplemented 占位 + Phase::Skeleton marker)
- ✅ #13 a/b/c/d 跨域协调 (4 image matrix 单 PR 不交叉)
- ⚠️ #14 v4 Mavis 临时代签 5 域 Lead (per 9/3 11:35 JST 反转, 真人到位追溯签字覆盖修订历史)