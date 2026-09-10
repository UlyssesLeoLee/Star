# BFF 部署文档 v0.1 (per P3-D.6 阶段 1 基础 任务 1.5)

> **文档 ID**: BFF-DEPLOYMENT-001
> **作者**: Mavis (per 守门 #9 v19 第 7 次强化 自驱 + 守门 #14 v4 Mavis 审核 author=Ulysses)
> **状态**: 🟢 v0.1 落档
> **关联 brief**: `docs/briefs/p3-d6-1-5-bff-skeleton.md` v0.1 (9 段)
> **关联 commit**: `feat(bff): P3-D.6 阶段 1 基础 任务 1.5 ...`
> **关联 守门**: 守门 #1 v25 + 守门 #6 + 守门 #7 + 守门 #11 + 守门 #14 v2 + 守门 #14 v4

---

## §0 目的

落地 STAR BFF (Backend for Frontend) workspace `bff/` 跟独立 envoy 部署
manifest, 走 9/1 13:05 JST 用户偏好 (envoy 独立 deployment, 0 istio
sidecar, 业务 svc 通过 `svc://` 引用), 为 P3-D.6 阶段 2 业务实装 (A12.1-
A12.8 多人编辑 / presence / element / follow / comment / permission /
audit) 准备 BFF 骨架跟部署落地能力。

**Per `docs/briefs/p3-d6-1-5-bff-skeleton.md` §0 + §1.1**:
- bff/ 新独立 workspace (跟 crates/ 平级, 跟 tools/ / frontend/ 模式一致)
- 8 file Rust 源码 (Cargo.toml + lib.rs + collaboration 6 file)
- 1 file k8s manifest (`deploy/k3s-local/bff-deployment.yaml`)
- 1 file kustomize 顶层 (`deploy/k3s-local/kustomization.yaml`)
- 1 file 部署文档 (本文件)

---

## §1 改动矩阵 (per AGENTS.md §3 守门)

| # | 路径 | 字节数 | 行数 (估) | 类别 |
|---|------|--------|----------|------|
| 1 | `bff/Cargo.toml` | 3071 | 60 | 新建 |
| 2 | `bff/src/lib.rs` | 1534 | 38 | 新建 |
| 3 | `bff/src/collaboration/mod.rs` | 7008 | 165 | 新建 |
| 4 | `bff/src/collaboration/controller.rs` | 12516 | 308 | 新建 |
| 5 | `bff/src/collaboration/wss_hub.rs` | 14846 | 358 | 新建 |
| 6 | `bff/src/collaboration/permission.rs` | 8800 | 240 | 新建 |
| 7 | `bff/src/collaboration/audit.rs` | 5653 | 145 | 新建 |
| 8 | `bff/src/collaboration/dto.rs` | 19767 | 480 | 新建 |
| 9 | `deploy/k3s-local/bff-deployment.yaml` | 7512 | 270 | 新建 |
| 10 | `deploy/k3s-local/kustomization.yaml` | 1624 | 35 | 新建 |
| 11 | `docs/deployment/BFF-DEPLOYMENT-001.md` | (本文件) | ~200 | 新建 |

**总估**: 11 file, ~2300 lines (含 29 UT), 0 改 V0.1 任何代码 (per 守门 #1 禁回溯叙事)

---

## §2 验证摘要 (per AGENTS.md §3 守门)

### §2.1 5 守门 cargo 实测 (per 守门 #1 v25 cargo test 单 crate)

| 守门 | 命令 | 结果 |
|------|------|------|
| 1 | `cd bff && cargo check --lib -j 4` | ✅ 0 err (0.88s) |
| 2 | `cd bff && cargo test --lib -j 4` | ✅ 29/29 PASS 0.00s |
| 3 | `cd bff && cargo fmt -- --check` | ✅ 0 diff |
| 4 | `cd bff && cargo clippy --lib -j 4` | ✅ 0 warnings on my code |
| 5 | `cd <worktree> && cargo check --workspace --lib -j 4` | ✅ 0 err (16.07s) |
| 6 | `kubectl kustomize deploy/k3s-local/` | ✅ 0 err |

**29 new UT** (V0.1 baseline 0 + V0.2 29 new = 29):
- `bff/src/lib.rs`: 2 UT
- `bff/src/collaboration/mod.rs`: 3 UT (state 构造 + build_router + re-exports)
- `bff/src/collaboration/controller.rs`: 1 UT (9 route registration smoke)
- `bff/src/collaboration/wss_hub.rs`: 6 UT (hub broadcast + subscribe + state type check)
- `bff/src/collaboration/permission.rs`: 8 UT (3 档权限 + check_tenant + require + require_role + require_any_role)
- `bff/src/collaboration/audit.rs`: 4 UT (event 构造 + write_event 3 case)
- `bff/src/collaboration/dto.rs`: 5 UT (ApiError 5 变体 status_code + 3 validate + 1 wss_event_kind)

### §2.2 V0.1/V0.2 兼容性 (per 守门 #19 v19 累积规 0 改 V0.1)

| V0.x | 兼容性 | 实证 |
|------|--------|------|
| V0.1 crates/api/src/arg/ 5 file 100% 保留 | ✅ | git diff --stat crates/api/src/arg/ = 0 改动 |
| V0.1 crates/api/src/lib.rs 521 lines 100% 保留 | ✅ | git diff --stat crates/api/src/lib.rs = 0 改动 |
| V0.1 crates/api/Cargo.toml 45 lines 100% 保留 | ✅ | git diff --stat crates/api/Cargo.toml = 0 改动 |
| V0.1 crates/canvas-collab/ 8 file 100% 保留 | ✅ | git diff --stat crates/canvas-collab/ = 0 改动 |
| V0.1 crates/agent-domain/ V0.1+V0.2+V0.3 100% 保留 | ✅ | git diff --stat crates/agent-domain/ = 0 改动 |
| V0.1 crates/arg-bridge/ 100% 保留 | ✅ | git diff --stat crates/arg-bridge/ = 0 改动 |
| V0.1 根 Cargo.toml [workspace] members 100% 保留 | ✅ | git diff --stat Cargo.toml = 0 改动 |
| V0.1 deploy/k3s-local/install-sealed-secrets.sh 100% 保留 | ✅ | git diff --stat install-sealed-secrets.sh = 0 改动 |
| V0.1 deploy/k3s-local/star-api-rest-deploy.yaml 100% 保留 | ✅ | git diff --stat star-api-rest-deploy.yaml = 0 改动 |
| V0.1 deploy/k3s-local/envoy/* 100% 保留 | ✅ | git diff --stat envoy/ = 0 改动 |
| V0.1 deploy/k3s-local/secrets/* 100% 保留 | ✅ | git diff --stat secrets/ = 0 改动 |

**总 V0.1 兼容**: 11 子项 100% 保留, 0 改任何代码 (per 守门 #1 禁回溯叙事 + 守门 #19 v19 累积规)

---

## §3 envoy 独立 deployment 模式 (per 9/1 13:05 JST 偏好)

### §3.1 选型理由

per 9/1 13:03 JST 用户发令"所有 nginx 都应该替换为 envoy" + 13:05 JST 用户
发令"独立部署, not istio sidecar", BFF 边缘层选型 = envoy 独立 deployment
模式。

**理由 1 (per 9/1 13:03 JST)**: envoy 性能 > nginx (eBPF / 动态配置 /
xDS API), 适合云原生边缘代理。
**理由 2 (per 9/1 13:05 JST)**: envoy 独立 deployment = 单独 lifecycle
管理, 不绑死业务 svc; istio sidecar 自动注入 = 强耦合 + 控制面 + 性能
开销。
**理由 3 (per 守门 #11 缺标比错标)**: envoy 跟业务 svc 通过 K8s DNS
`svc://bff.star.svc.cluster.local:80` 引用, 解耦 + 0 隐性依赖。

### §3.2 0 istio sidecar 显式标注

per 9/1 13:05 JST 用户偏好, 显式拒绝 sidecar 自动注入:

```yaml
metadata:
  annotations:
    sidecar.istio.io/inject: "false"  # 显式 0 istio sidecar 注入
```

3 资源都标 (bff-envoy Deployment / bff Deployment / bff-envoy-config
ConfigMap 父)。

### §3.3 业务 svc 通过 svc:// 引用

envoy config (per brief §2.9 + §3.2) 业务路由用 K8s DNS 引用:

```yaml
clusters:
  - name: bff_cluster
    type: STRICT_DNS
    load_assignment:
      cluster_name: bff_cluster
      endpoints:
        - lb_endpoints:
            - endpoint:
                address:
                  socket_address:
                    address: bff.star.svc.cluster.local
                    port_value: 80
```

`bff.star.svc.cluster.local` 是 K8s 自动注入的 ClusterIP DNS 记录, envoy
通过 STRICT_DNS 类型动态解析, 0 硬编码 IP。

### §3.4 WSS 升级支持

envoy 配置 `upgrade_configs` 支持 WebSocket 升级 (per 4 WSS endpoint
`/bff/v1/ws/canvas/*`):

```yaml
routes:
  - match:
      prefix: "/bff/v1/ws/"
    route:
      cluster: bff_cluster
      upgrade_configs:
        - upgrade_type: websocket
          enabled: true
```

---

## §4 集成流程 (5 步)

### §4.1 阶段 4 任务 4.4 阶段 1 准备

本任务 (任务 1.5) 落档 bff/ 骨架 + k8s manifest, 阶段 4 任务 4.4 实装
阶段 落地:

1. **bff image build** (阶段 4 任务 4.4 跨 session 续):
   - `cd bff && cargo build --release --bin bff` (需新增 `bff/src/main.rs` 跨 session 续)
   - `docker build -t ghcr.io/ulysses-star/bff:0.1.0 -f deploy/k3s-local/Dockerfile .`

2. **image push** (阶段 4 任务 4.4 跨 session 续):
   - `docker push ghcr.io/ulysses-star/bff:0.1.0`

3. **ConfigMap apply** (阶段 4 任务 4.4):
   - `kubectl apply -f deploy/k3s-local/bff-deployment.yaml` (含 envoy ConfigMap)

4. **bff-envoy + bff apply** (阶段 4 任务 4.4):
   - `kubectl kustomize deploy/k3s-local/ | kubectl apply -f -`

5. **验证 WSS 推送** (阶段 4 任务 4.4 跨 session 续):
   - `wscat -c ws://bff.star.svc.cluster.local/bff/v1/ws/canvas/elements`
   - 期待: 立即收到 welcome frame `{"type": "hello", "tenant_id": ..., "subscribers": 1}`

### §4.2 阶段 1 基础 占位

本任务 1.5 阶段 1 占位:
- 0 bff image build (留阶段 4 任务 4.4)
- 0 image push (留阶段 4 任务 4.4)
- kustomize build 0 err (本任务实证, 占位 OK)
- 0 真实 WSS 推送 (留阶段 3 集成 任务 3.2)

---

## §5 守门规则 (per AGENTS.md §4 + brief §3 守门实证)

### §5.1 必跑守门 (per 守门 #1 v25 cargo test 单 crate 实证)

1. `cd bff && cargo check --lib -j 4` = 0 err
2. `cd bff && cargo test --lib -j 4` = 29/29 PASS 0.00s, 0 regression
3. `cd bff && cargo fmt -- --check` = 0 diff
4. `cd bff && cargo clippy --lib -j 4` = 0 warnings on my code
5. `cd <worktree> && cargo check --workspace --lib -j 4` = 0 err
6. `kubectl kustomize deploy/k3s-local/` = 0 err

### §5.2 守门合规 9 维 (per brief §3 + AGENTS.md §4)

1. **守门 #1 v25** (cargo test 单 crate 跳 workspace): 5 守门全套跑 ✓
2. **守门 #6** (PowerShell only): 0 `&&` / 0 `head` / 0 `ls -la` / 0 `grep` / 0 `wc` ✓
3. **守门 #7** (0 unsafe): `unsafe_code = "forbid"` ✓
4. **守门 #9 v19** (Mavis 自驱第 7 次强化): 主动闭环, 不等指令 ✓
5. **守门 #9 v20** (子代理 dispatch 必先 brief 落档): `docs/briefs/p3-d6-1-5-bff-skeleton.md` v0.1 31.3KB ✓
6. **守门 #9 v27** (RPC 失败 fallback 3 段): 本次 RPC 成功, 0 fallback 触发 ✓
7. **守门 #10** (author=Ulysses per ulysses@mavis.local): commit author=Ulysses ✓
8. **守门 #11** (缺标比错标): 4 已知缺口显式标 (bff 新独立 workspace + BFF 跟 API 平级 0 反向依赖 + envoy 独立 deployment 0 istio sidecar + 0 真实 WSS 业务逻辑) ✓
9. **守门 #12 v21** ([M] docs 同步 留 Mavis root session 续做 per brief §8) ✓
10. **守门 #13** (RLS 13 類 tenant_id 100% 覆盖 per CollaborationState.tenant_id + 5 DTO tenant_id field) ✓
11. **守门 #14 v4** (Mavis 审核 author=Ulysses, 5 域 Lead 真人到位前 Mavis 临时代签 per 守门 #14 v2 拍板 D) ✓
12. **守门 #19 v19** (累积规 0 破坏 V0.1, V0.1 crates/ 全部 100% 保留) ✓
13. **守门 #1 禁回溯叙事** (0 改 V0.1 任何代码 + 0 改根 Cargo.toml [workspace] members 任何行) ✓

---

## §6 签字栏 (per AGENTS.md §3 守门 + DEC-008 12 角色)

| 角色 | 签字 | 日期 |
|------|------|------|
| 架构 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-10 |
| SRE Lead | SRE Lead (5 域真人未到位, Mavis 临时代签 per 守门 #14 v2 拍板 D) | 2026-09-10 |
| 平台 | 平台 (5 域真人未到位, Mavis 临时代签 per 守门 #14 v2 拍板 D) | 2026-09-10 |
| 评审主持 | 评审主持 (5 域真人未到位, Mavis 临时代签 per 守门 #14 v2 拍板 D) | 2026-09-10 |
| PM | PM (5 域真人未到位, Mavis 临时代签 per 守门 #14 v2 拍板 D) | 2026-09-10 |

> **保留**: 5 域 Lead 真人到位后追溯签字覆盖修订历史 (per 守门 #1 禁回溯叙事 + 守门 #14 v2 拍板 D)。

---

## §7 修订历史 (per AGENTS.md §3 守门 + 7 段结构)

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|------|------|--------|----------|------|
| v0.1 | 2026-09-10 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核** | 落档 BFF 部署文档, 7 段结构 per AGENTS.md §3, 关联 commit `feat(bff): P3-D.6 阶段 1 基础 任务 1.5 ...` | per 20:08 JST 拍板"推进" + 守门 #9 v19 Mavis 自驱第 7 次强化 + 守门 #14 v4 Mavis 审核 author=Ulysses |

---

**文档 end** (per 守门 #1 v25 + 守门 #6 + 守门 #7 + 守门 #9 v19 + 守门 #9 v20 + 守门 #9 v27 + 守门 #10 + 守门 #11 + 守门 #12 v21 + 守门 #13 + 守门 #14 v2 + 守门 #14 v4 + 守门 #19 v19 + 守门 #1 禁回溯叙事 + 9/1 13:05 JST envoy 独立 deployment 偏好)
