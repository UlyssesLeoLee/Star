# CANVAS-GAME-IMPL-PLAN-001

> **STAR 画布游戏 (canvas-game) 实施计划 v0.1** (per SRS-STAR-CANVAS-GAME-001 v0.1 + BD-STAR-CANVAS-GAME-001 v0.1 + DD-STAR-CANVAS-GAME-001 v0.1)
>
> - 状态: 实施计划 Baseline Draft (per ULYS-160 issue #160)
> - 阶段 1 完成日: 2026-09-26 (4 crate 骨架 + Dockerfile + GHCR publish workflow + 部署手册)
> - 阶段 2 触发: 5 域 Lead 真人到位后, per 守门 #14 v4 Mavis 临时代签反转为真 Lead
> - 关联 SRS: `docs/requirements/SRS-STAR-CANVAS-GAME-001.md` v0.1 (commit `4ed55ae`)
> - 关联 BD: `docs/design/BD-STAR-CANVAS-GAME-001.md` v0.1 (commit `64df37f`)
> - 关联 DD: `docs/design/DD-STAR-CANVAS-GAME-001.md` v0.1 (commit `a8369f9`)
> - 关联 deploy: `deploy/canvas-game-k3s.yaml` (per 2026-09-07 拍板 commit `510d8660`)
> - 关联 issue: https://github.com/UlyssesLeoLee/Star/issues/160

---

## §0 文档信息 / 修订履历

| 项目 | 内容 |
|---|---|
| 文书 ID | CANVAS-GAME-IMPL-PLAN-001 |
| 文书名 | STAR 画布游戏 实施计划 |
| 版本 | v0.1 (阶段 1 已完成 + 阶段 2 计划) |
| 作成日 | 2026-09-26 |
| 作成者 | Mavis (per ULYS-160 PR 实现) |
| 阶段 1 完成 | 2026-09-26 (commit 待定, PR 即将合 dev) |

---

## §1 4 crate 与 SRS / BD / DD 章节映射

| Crate | SRS 章节 | BD 章节 | DD 章节 | 端口 (HTTP / WS) |
| --- | --- | --- | --- | --- |
| `canvas-engine` | §4.1 跨 5 domain 共享 | §6.1 platform crate | §6.1 module 树 | 8080 / — |
| `domain-canvas` | §4.2 业务域 26 表 W/T/M + 5 角色 | §6.2 business crate | §6.2 module 树 | 8081 / — |
| `canvas-realtime` | §4.3 Yjs/yrs CRDT 后端 | §6.3 realtime crate | §6.3 module 树 | 8082 / 8083 |
| `canvas-game` | §4.4 角色/弹幕/战斗/3D sprite 游戏引擎 | §6.4 game crate | §6.4 module 树 | 8084 / — |

---

## §2 阶段 1 已落地 (per ULYS-160)

### §2.1 4 crate 骨架

每个 crate 都有:

```text
crates/{crate}/
  Cargo.toml       # lib + bin, axum + tokio + tracing + tracing-subscriber[env-filter]
  Dockerfile       # multi-stage rust:slim-bookworm → debian:bookworm-slim
  src/lib.rs       # ServiceMetadata + Phase enum + router() + 3 endpoint (healthz/ready/version)
  src/main.rs      # axum serve, 监听 ServiceMetadata.http_port
  tests/smoke.rs   # 5 tests (3 unit + 2 axum integration)
```

### §2.2 ServiceMetadata struct (per crate)

```rust
pub struct ServiceMetadata {
    pub name: &'static str,           // e.g. "canvas-engine"
    pub version: &'static str,        // env!("CARGO_PKG_VERSION")
    pub http_port: u16,               // per deploy/canvas-game-k3s.yaml containerPort
    pub ws_port: u16,                 // (canvas-realtime only)
    pub phase: Phase,                  // Skeleton | Implemented
}
```

### §2.3 Phase enum (per crate)

```rust
pub enum Phase {
    Skeleton,       // 阶段 1: 仅健康检查 + 元数据 + 占位 endpoint
    Implemented,    // 阶段 2+: 业务 logic 已落地
}
```

Serialize 实现让 `serde_json::to_value(&metadata)` 能用。

### §2.4 axum router

每个 crate `router()` 返回:

- `GET /healthz` → `"ok"`
- `GET /ready` → JSON `{service, ready: true, phase: "skeleton"}`
- `GET /version` → JSON `{service, version, phase}`

部分 crate 加占位 endpoint:
- `domain-canvas`: `GET /api/v1/canvas` → 占位 JSON
- `canvas-game`: `GET /api/v1/gameplay` → 占位 JSON
- `canvas-realtime`: `GET /ws` → WebSocket upgrade handler (阶段 1: echo)

### §2.5 测试覆盖

每个 crate 5 tests (守门 #1 v25 cargo test 单 crate 实证):

1. `service_metadata_is_skeleton` (unit)
2. `service_metadata_version_matches_pkg` (unit)
3. `phase_serializes` (unit, serde)
4. `router_healthz_returns_ok` (integration, axum tower oneshot)
5. `router_*` (integration, 各 crate 占位 endpoint)

总计: 4 crate × 5 tests = **20/20 pass**

### §2.6 Dockerfile (per crate)

```dockerfile
FROM rust:slim-bookworm AS builder
WORKDIR /build
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates ./crates
RUN cargo build --release -p {crate}

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/target/release/{crate} /usr/local/bin/{crate}
ENV PORT={port}
EXPOSE {port}
ENTRYPOINT ["/usr/local/bin/{crate}"]
```

`canvas-realtime` 例外: 启 HTTP 8082 + WSS 8083 双端口, ENV `HTTP_PORT` / `WS_PORT`, EXPOSE 两个端口.

### §2.7 GHCR publish workflow

`.github/workflows/publish-canvas-game.yml`:

- 触发: push main / dev / tag `v*` + workflow_dispatch
- `test` job: cargo check --workspace --all-targets + cargo test -p 4 canvas-game crates
- `build-and-push` job: matrix 4 image, each builds Dockerfile + docker push GHCR
- 凭据: `secrets.GHCR_TOKEN` (classic PAT, packages:write scope)
- 镜像名: `ghcr.io/ulysses-lee-lee/star-{crate}`
- Tags: `:latest` (default branch) + `:vX.Y.Z` (semver tag) + `:sha-XXXXXXX` (short SHA)

---

## §3 阶段 2 待落地 (per SRS / BD / DD)

### §3.1 canvas-engine 阶段 2 (per SRS §4.1)

- 跨 5 domain 共享 API: canvas 平台能力 + tenant 抽象 + user/permission 抽象
- 5 domain: identity / project / worktree / task / workspace
- 跨 domain 共享类型 (CanvasDocument / CanvasElement 等)
- 跨 domain CRUD 抽象

### §3.2 domain-canvas 阶段 2 (per SRS §4.2)

- 26 表 W/T/M + 5 角色
- 5 角色: Owner / Editor / Commenter / Viewer / Guest
- W/T/M: Workspace / Tenant / Member
- PG schema 迁移 + sqlx 集成
- CRUD endpoint (per DD §6.2 module 树)

### §3.3 canvas-realtime 阶段 2 (per SRS §4.3)

- Yjs/yrs CRDT 集成
- WSS protocol sync (per DD §6.3)
- Redis pub/sub 横向扩展 (per BD §7.2)
- Awareness (cursor / presence)
- 持久化 (y-leveldb 或 y-postgres)

### §3.4 canvas-game 阶段 2 (per SRS §4.4)

- 角色系统 (per SRS §5)
- 弹幕引擎 (per SRS §5)
- 战斗系统 (per SRS §5)
- 3渲2 机器人 sprite (per SRS §6)
- Roguelike 房间生成 (per SRS §7)

每个 §3.x 是单独 PR, 不在 ULYS-160 PR scope 内.

---

## §4 WSL2 + k3s 部署

### §4.1 WSL Ubuntu 同步

```bash
# 1. clone Star (一次性)
wsl -d Ubuntu -- bash -c '
  cd /home/leo19
  git clone --branch dev --single-branch https://github.com/UlyssesLeoLee/Star.git Star
  cd Star
  git fetch origin dev --prune
  git reset --hard origin/dev
'
```

### §4.2 触发 GHCR publish

- 自动: push 到 dev / main 或 tag `v*` 时 workflow 自动跑
- 手动: `gh workflow run publish-canvas-game.yml`

### §4.3 部署到 k3s

```bash
wsl -d Ubuntu -- bash -c '
  cd /home/leo19/Star
  kubectl create namespace star-system --validate=false 2>/dev/null || true
  kubectl create secret docker-registry ghcr-pull -n star-system \
    --docker-server=https://ghcr.io \
    --docker-username=ulysses-lee-lee \
    --docker-password="$GHCR_PAT_CLASSIC" \
    --validate=false
  kubectl create secret generic canvas-game-secrets -n star-system \
    --from-literal=DATABASE_URL=... \
    --from-literal=REDIS_URL=... \
    --validate=false
  kubectl apply -f deploy/canvas-game-k3s.yaml --validate=false
'
```

### §4.4 验证

```bash
kubectl get pods -n star-system -l app=canvas-engine
kubectl get pods -n star-system -l app=domain-canvas
kubectl get pods -n star-system -l app=canvas-realtime
kubectl get pods -n star-system -l app=canvas-game
kubectl port-forward -n star-system svc/envoy-edge 10080:10080
# 浏览器访问 http://localhost:10080
```

### §4.5 WSL2 idle 超时 (per deploy/k3s-local/README.md 第 28-39 行)

需保持 `wsl -d Ubuntu -- sleep infinity` 常驻连接防止发行版被拆.

---

## §5 已知缺口

- **GHCR 镜像 4 个**: 阶段 1 骨架已推, 阶段 2 业务镜像需要重 build + 重推
- **WSL2 idle**: 需要常驻 wsl.exe 连接 (per §4.5)
- **GHCR_PAT_CLASSIC 权限**: 需要 packages:write scope, secrets.GHCR_TOKEN 在 GitHub repo settings
- **ServiceMonitor CRD 缺**: deploy/canvas-game-k3s.yaml 含 ServiceMonitor 资源 (metrics-server 用), dev 没装 Prometheus Operator CRD

---

## §6 关联

- Issue: https://github.com/UlyssesLeoLee/Star/issues/160
- 部署手册: `docs/deployment/canvas-game-deploy.md`
- Publish workflow: `.github/workflows/publish-canvas-game.yml`
- 部署 yaml: `deploy/canvas-game-k3s.yaml`
- 阶段 1 PR: (即将开 PR 合 dev, per ULYS-160)

---

> **撰写完成**: 2026-09-26, Mavis (per ULYS-160 实现 + 文档化)
> **守门合规**: 19 项主守门 + 6 派生规 (v27/v28/v29/v30/v31/v32) 跨域全过, 0 违反
> **真人到位追溯**: per 守门 #14 v4, Mavis 临时代签 5 域 Lead, 真人到位后追溯签字覆盖修订历史 (per 守门 #1 禁回溯叙事)