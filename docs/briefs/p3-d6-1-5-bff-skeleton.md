# P3-D.6 阶段 1 基础 任务 1.5 Brief: `bff/src/collaboration/` BFF 骨架 + envoy 独立 deployment

> **任务 ID**: p3-d6-1-5-bff-skeleton
> **优先级**: P0 (P3-D.6 阶段 1 基础 第 5 任务)
> **估时**: ~0.20M tokens / 0.17 SRE·周 (per `docs/implementation-plans/CANVAS-IMPL-PLAN-001.md` §3 阶段 1 基础 任务 1.5)
> **依赖**: 任务 1.1 (`crates/agent-domain/` 已落档) + 任务 1.2 (`crates/arg-bridge/canvas_sync_bridge.rs` 已落档) + 任务 1.3 (`crates/canvas-collab/` 已落档) + 任务 1.4 (`crates/api/src/{agent,canvas_collab}/` 2 新 module 已落档)
> **作者**: Mavis (per Ulysses 19:40 JST 拍板"按照 wbs 开子代理 worktree 制作, 完成后合并到 main" + 20:08 JST 拍板"推进" + 守门 #9 v19 Mavis 自驱第 7 次强化)
> **worktree 分支**: `wt-p3-d6-1-5-bff-skeleton` 基于 main `4aba448` (任务 1.4 docs sync 之后)

---

## §0 目的

新建 `bff/` workspace (跟 `crates/` 平级, 类似 `tools/`), 加 `bff/src/collaboration/` module 含 5 REST + 4 WSS endpoint 占位 + envoy 独立 deployment yaml, 为 P3-D.6 阶段 2 业务实装 (A12.1-A12.8 多人编辑 / presence / element / follow / comment / permission / audit) 准备 BFF 骨架。

**Per `docs/implementation-plans/CANVAS-IMPL-PLAN-001.md` §3 阶段 1 基础 任务 1.5**:
> 1.5 `bff/src/collaboration/` BFF 骨架 (per DD-AGENT §3.1, envoy 独立 deployment per 9/1 13:05 JST 偏好) | 1.3 | ~0.2M | #6 PowerShell only + #14 v2

**Per `docs/design/DD-CANVAS-AGENT-001.md` §3.1 line 301-307**:
```
bff/src/collaboration/                        # NEW (A12 5 REST + 4 WSS 端点, 走 envoy 独立 deployment)
├── mod.rs
├── controller.rs                             # 5 REST endpoints
├── wss_hub.rs                                # 4 WSS endpoints (A12.1 + A12.2 + A12.3 + A12.4)
├── permission.rs                             # A12.7 3 级权限 (view/comment/edit)
├── audit.rs                                  # A12.8 100% audit
└── dto.rs                                    # request/response types
```

**envoy 独立 deployment 偏好 (per 9/1 13:03+13:05 JST 用户发令)**:
- 所有 nginx → envoy
- envoy 用独立 deployment 模式, 不是 istio sidecar
- 不引入 istio 控制面 + envoy sidecar 自动注入
- 单独 envoy deployment + ClusterIP service, 业务 svc 通过 `svc://` 引用

---

## §1 范围 / Out-of-scope

### 1.1 范围内 (要做的)

1. **`bff/Cargo.toml` 新建** (per 守门 #11 缺标比错标, 新 workspace):
   - `[package] name = "bff" version = "0.1.0"`
   - `[workspace]` (新 workspace, **不**加到根 `Cargo.toml` [workspace] members, 跟 `tools/` / `frontend/` 模式一致)
   - `[dependencies]`: axum / tokio / tokio-tungstenite (WSS) / serde / serde_json / uuid / chrono / async-trait / thiserror / star-context / star-canvas-collab / star-arg-bridge / star-api
   - **注意**: `bff` 跟 `crates/api` 是平级 (BFF vs API), 都依赖 `star-canvas-collab` + `star-arg-bridge` 但不互相依赖

2. **`bff/src/lib.rs`** (~30 lines):
   - module-level doc + `pub mod collaboration;` + re-exports + 1-2 UT

3. **`bff/src/collaboration/`** 6 新增 (per DD-AGENT §3.1 line 301-307):
   - `mod.rs` (~80 lines): module doc + 5 sub-mod pub + CollaborationState struct 8-10 字段 + new() + build_router() + 1-2 UT
   - `controller.rs` (~500 lines): 5 REST endpoint 占位 handler (A12.1-A12.5)
   - `wss_hub.rs` (~400 lines): 4 WSS endpoint 占位 handler (A12.1 + A12.2 + A12.3 + A12.4) + WSS hub 共享 broadcast channel
   - `permission.rs` (~150 lines): CollaborationPermission + PermissionLevel enum 3 变体 (View/Comment/Edit) + require(level) helper + 1-2 UT (跟 api/src/canvas_collab/permission.rs 协调, 但 BFF 强制 middleware 必填)
   - `audit.rs` (~120 lines): CollaborationAuditEvent 6 字段 + write_event 占位 (per 守门 #13 d Transaction 100% audit) + 1-2 UT
   - `dto.rs` (~400 lines): 5 REST Request/Response DTO + 4 WSS message DTO + ApiError 5 变体

4. **`deploy/k3s-local/bff-deployment.yaml`** (per 9/1 13:05 JST envoy 独立 deployment 偏好):
   - 1 个 envoy Deployment (1 replica, containerPort 8080 + 9901 admin)
   - 1 个 envoy Service ClusterIP (port 80 → targetPort 8080)
   - 1 个 bff Deployment (1 replica, containerPort 3000)
   - 1 个 bff Service ClusterIP (port 80 → targetPort 3000)
   - 1 个 ConfigMap `envoy-config` 含 envoy.yaml (HTTP filter + route BFF service `svc://bff` + static_resources + admin)
   - 1 个 Kustomization yaml (kustomize 集成)
   - 0 Istio / 0 sidecar injection

5. **`docs/deployment/BFF-DEPLOYMENT-001.md`** (per AGENTS.md §3 7 段结构):
   - §0 目的 + §1 改动矩阵 + §2 验证摘要 + §3 envoy 独立 deployment 模式 + §4 集成流程 + §5 守门规则 + §6 签字栏 + §7 修订历史

### 1.2 Out-of-scope (不做的)

1. **0 业务方法实装** (留 P3-D.6 阶段 2 任务 2.3 A12 业务实装):
   - 5 REST + 4 WSS = 9 endpoint handler body 全部占位 `Ok(Json(T::default()))` 或 `Ok(WsResponse::default())`

2. **0 真实 WSS 业务逻辑** (留 P3-D.6 阶段 3 集成 任务 3.2):
   - 0 CRDT (Yjs/Automerge) 集成 (P0 阶段跳过, P1 阶段续做)
   - 0 NATS Subject publish (留阶段 3 集成)
   - 0 真实 BroadcastChannel 集成 (per 守门 #11 缺标比错标, 仅留 channel 占位)

3. **0 改 `crates/api/` 任何代码** (per 守门 #1 禁回溯叙事):
   - 0 改 `crates/api/src/agent/` / `crates/api/src/canvas_collab/` / `crates/api/src/arg/` 任何文件

4. **0 改 `crates/canvas-collab/` 任何代码** (per 守门 #1 禁回溯叙事):
   - 0 改 `crates/canvas-collab/src/models/` 任何文件 (任务 1.3 100% 保留)

5. **0 改 `deploy/k3s-local/` 已有 yaml** (per 守门 #1 禁回溯叙事):
   - 0 改 `deploy/k3s-local/install-sealed-secrets.sh` / `deploy/k3s-local/sealed-secrets-*.yaml` 任何行
   - **新增** `deploy/k3s-local/bff-deployment.yaml` 不改现有

6. **0 改根 `Cargo.toml` [workspace] members** (per 守门 #1 禁回溯叙事):
   - `bff/` 是新独立 workspace, 跟 `crates/` 平级, **不**加到根 members

7. **0 改 `crates/api/Cargo.toml` 添加 bff 依赖**:
   - BFF 跟 API 是平级 (BFF 调 API, 不反过来), 0 反向依赖

---

## §2 详细设计

### 2.1 `bff/Cargo.toml` (新 workspace)

```toml
[package]
name = "bff"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
license.workspace = true
authors.workspace = true
repository.workspace = true

[workspace]  # 新独立 workspace, 跟 crates/ 平级

[dependencies]
# 基础依赖 (per 守门 #11 缺标比错标, 检查现有 0 重复)
axum = { workspace = true }
tokio = { workspace = true }
tokio-tungstenite = { workspace = true }  # WSS 客户端
serde = { workspace = true }
serde_json = { workspace = true }
async-trait = { workspace = true }
thiserror = { workspace = true }
uuid = { workspace = true }
chrono = { workspace = true }

# 跨 crate path 依赖
star-context = { path = "../crates/star-context" }
star-canvas-collab = { path = "../crates/canvas-collab" }
star-arg-bridge = { path = "../crates/arg-bridge" }
star-api = { path = "../crates/api" }

[lints]
workspace = true
```

**必先 `cat ../Cargo.toml` 验证**:
- `axum` / `tokio` / `tokio-tungstenite` 是否在 [workspace.dependencies]
- `serde` / `serde_json` / `async-trait` / `thiserror` / `uuid` / `chrono` 现有

### 2.2 `bff/src/lib.rs` (~30 lines)

```rust
// SPDX-License-Identifier: MIT OR Apache-2.0
//! `bff` — STAR BFF (Backend for Frontend) workspace.
//!
//! Reference: [`docs/design/DD-CANVAS-AGENT-001.md`](../../docs/design/DD-CANVAS-AGENT-001.md)
//! v0.1.1 §3.1 line 301-307. BFF 跟 `crates/api` 平级, 走 envoy
//! 独立 deployment (per 9/1 13:05 JST 偏好), 通过 `svc://` 引用
//! `crates/api` 业务 svc.
//!
//! 1 sub-module (本任务):
//! - [`collaboration`] — 5 REST + 4 WSS endpoint (A12)
//!
//! 0 业务方法实装 (per 守门 #1 禁回溯叙事 + 守门 #11 缺标比错标), 留 P3-D.6 阶段 2 任务 2.3 A12 业务实装.

pub mod collaboration;

// Re-exports
pub use collaboration::{build_router, CollaborationState};
```

### 2.3 `bff/src/collaboration/mod.rs` (~80 lines)

跟 `crates/api/src/arg/mod.rs` 同形 + WSS 共享状态:

```rust
use std::sync::Arc;
use axum::Router;
use uuid::Uuid;
use tokio::sync::broadcast;

use star_canvas_collab::models::element::CanvasElementBackend;

pub mod audit;
pub mod controller;
pub mod dto;
pub mod permission;
pub mod wss_hub;

/// BFF 共享状态 (per DD-AGENT §3.1 line 301-307, 8-10 字段).
///
/// `#[allow(dead_code)]` 不需要: 全部 8 字段都被 controller + wss_hub 访问.
pub struct CollaborationState {
    /// Canvas element CRUD (per A12.1 + A12.3 backend 持久化, 阶段 2 任务 2.3 实装)
    pub canvas_ops: Arc<star_canvas_collab::CanvasOps>,  // V0.2 强类型后续 fill
    /// Audit middleware (per A12.8 100% audit + 守门 #13 d Transaction)
    pub audit: Arc<audit::CollaborationAudit>,
    /// Permission middleware (per A12.7 3 级权限 view/comment/edit, BFF 强制)
    pub permission: Arc<permission::CollaborationPermission>,
    /// WSS hub 共享 broadcast channel (per A12.1-A12.4 4 WSS endpoint)
    pub wss_hub: Arc<wss_hub::CollaborationWssHub>,
    /// 当前 tenant_id (per 守门 #13 b RLS 13 類)
    pub tenant_id: Uuid,
    // + 3 placeholder 字段 (V0.2 强类型后续 fill)
    pub canvas_id_placeholder: Option<Uuid>,
    pub actor_id_placeholder: Option<Uuid>,
    pub presence_placeholder: Option<star_canvas_collab::models::presence::PresenceCursor>,
}

impl CollaborationState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        canvas_ops: Arc<star_canvas_collab::CanvasOps>,
        audit: Arc<audit::CollaborationAudit>,
        permission: Arc<permission::CollaborationPermission>,
        wss_hub: Arc<wss_hub::CollaborationWssHub>,
        tenant_id: Uuid,
    ) -> Arc<Self> { ... }
}

pub fn build_router(state: Arc<CollaborationState>) -> Router {
    controller::collaboration_routes(state)
}

pub use controller::collaboration_routes;
pub use dto::ApiError;
pub use permission::{CollaborationPermission, PermissionLevel};
pub use wss_hub::{CollaborationWssHub, CollaborationWssEvent};
```

### 2.4 `bff/src/collaboration/controller.rs` (~500 lines)

5 REST endpoint (A12.1-A12.5):

| Endpoint | Method | Path | Handler | A# | 业务方法 |
|---|---|---|---|---|---|
| 1 | POST | `/bff/v1/canvas/elements` | `create_element` | A12.1 | `CanvasElementBackend::create` |
| 2 | PATCH | `/bff/v1/canvas/elements/:id` | `update_element` | A12.3 | `CanvasElementBackend::update` |
| 3 | POST | `/bff/v1/canvas/elements/:id/comments` | `add_comment` | A12.5 | `CanvasComment::add` |
| 4 | GET | `/bff/v1/canvas/:id/presence` | `get_presence` | A12.2 | `PresenceCursor::list` |
| 5 | POST | `/bff/v1/canvas/:id/follow` | `set_follow` | A12.4 | `Follow::set` |

Handler 占位 pattern 跟 `crates/api/src/canvas_collab/controller.rs` 任务 1.4 同形:
```rust
async fn create_element(
    State(state): State<Arc<CollaborationState>>,
    Json(req): Json<dto::CreateElementRequest>,
) -> Result<Json<dto::ElementResponse>, dto::ApiError> {
    state.permission.require(PermissionLevel::Edit)?;
    state.permission.check_tenant(state.tenant_id)?;
    state.audit.write_event(/* ... */).await?;
    // V0.4 阶段 2 任务 2.3 实装: let result = state.canvas_ops.create(req.into(), state.tenant_id).await?;
    Ok(Json(dto::ElementResponse::default()))
}
```

### 2.5 `bff/src/collaboration/wss_hub.rs` (~400 lines)

4 WSS endpoint 占位 (per A12.1-A12.4):

| WSS Endpoint | Path | A# | 业务方法 |
|---|---|---|---|
| 1 | `/bff/v1/ws/canvas/elements` | A12.1 | element 增删改推送 |
| 2 | `/bff/v1/ws/canvas/presence` | A12.2 | presence cursor 推送 |
| 3 | `/bff/v1/ws/canvas/comments` | A12.5 | comment 推送 |
| 4 | `/bff/v1/ws/canvas/follow` | A12.4 | follow mode 推送 |

WSS hub 共享 `broadcast::channel` (per tokio):
```rust
use tokio::sync::broadcast;

pub struct CollaborationWssHub {
    /// Broadcast channel for WSS events (capacity 1024, per task scope)
    pub tx: broadcast::Sender<CollaborationWssEvent>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CollaborationWssEvent {
    ElementCreated { canvas_id: Uuid, element_id: Uuid, actor_id: Uuid, at: DateTime<Utc> },
    ElementUpdated { canvas_id: Uuid, element_id: Uuid, actor_id: Uuid, at: DateTime<Utc> },
    ElementDeleted { canvas_id: Uuid, element_id: Uuid, actor_id: Uuid, at: DateTime<Utc> },
    PresenceUpdated { canvas_id: Uuid, user_id: Uuid, x: f32, y: f32, at: DateTime<Utc> },
    CommentAdded { canvas_id: Uuid, comment_id: Uuid, actor_id: Uuid, at: DateTime<Utc> },
    FollowStarted { canvas_id: Uuid, follower_id: Uuid, followee_id: Uuid, at: DateTime<Utc> },
    FollowStopped { canvas_id: Uuid, follower_id: Uuid, at: DateTime<Utc> },
}

impl CollaborationWssHub {
    pub fn new() -> Arc<Self> {
        let (tx, _rx) = broadcast::channel(1024);
        Arc::new(Self { tx })
    }
    pub fn broadcast(&self, event: CollaborationWssEvent) {
        let _ = self.tx.send(event);  // 0 业务方法, 仅占位
    }
}
```

WSS endpoint handler 占位:
```rust
pub async fn wss_canvas_elements(
    State(state): State<Arc<CollaborationState>>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    state.permission.require(PermissionLevel::View)?;
    ws.on_upgrade(move |socket| async move {
        // V0.4 阶段 3 集成实装: 真实 WSS 业务 + broadcast channel 订阅
        // 当前占位: 仅接受连接, 立即关闭
        let _ = state;
    })
}
```

### 2.6 `bff/src/collaboration/permission.rs` (~150 lines)

跟 `crates/api/src/canvas_collab/permission.rs` 任务 1.4 同形 + **BFF 强制 middleware**:

```rust
pub enum PermissionLevel { View, Comment, Edit }
pub struct CollaborationPermission {
    current_tenant: Uuid,
    user_level: PermissionLevel,  // BFF middleware 强制
}
impl CollaborationPermission {
    pub fn new(current_tenant: Uuid, user_level: PermissionLevel) -> Self { ... }
    pub fn require(&self, level: PermissionLevel) -> Result<(), ApiError> {
        if self.user_level.rank() >= level.rank() { Ok(()) }
        else { Err(ApiError::Forbidden) }
    }
    pub fn check_tenant(&self, current: Uuid) -> Result<(), ApiError> { ... }
}
impl PermissionLevel {
    pub fn rank(&self) -> u8 { match self { View => 0, Comment => 1, Edit => 2 } }
    pub fn satisfies(&self, required: PermissionLevel) -> bool { self.rank() >= required.rank() }
}
```

### 2.7 `bff/src/collaboration/audit.rs` (~120 lines)

跟 `crates/api/src/canvas_collab/audit.rs` 任务 1.4 同形 + 100% audit (per 守门 #13 d Transaction + A12.8):

```rust
pub struct CollaborationAuditEvent {
    pub id: Uuid,
    pub canvas_id: Uuid,
    pub actor_id: Uuid,
    pub action: String,  // "element.create" / "element.update" / "comment.add" / "presence.update" / "follow.start" / "follow.stop"
    pub at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}
pub struct CollaborationAudit { /* sink placeholder */ }
impl CollaborationAudit {
    pub async fn write_event(&self, event: CollaborationAuditEvent) -> Result<(), ApiError> {
        // V0.4 阶段 2 任务 2.3 实装: 写 audit_event 表 per 守门 #13 d + ADR-0043 WORM append-only
        Ok(())
    }
}
```

### 2.8 `bff/src/collaboration/dto.rs` (~400 lines)

5 REST DTO + 4 WSS message DTO + ApiError 5 变体:
- `CreateElementRequest` (12 字段 per CanvasElementBackend) / `ElementResponse` (12 字段)
- `UpdateElementRequest` (8 字段 partial) / `DeleteElementResponse`
- `AddCommentRequest` (per CanvasComment 9 字段) / `CommentResponse`
- `PresenceListResponse` (Vec of PresenceCursor)
- `SetFollowRequest` (followee_id) / `FollowResponse`
- `WssElementMessage` / `WssPresenceMessage` / `WssCommentMessage` / `WssFollowMessage`
- `ApiError` 5 变体 (BadRequest/Forbidden/NotFound/Internal/Unimplemented) 跟 `crates/api/src/canvas_collab/dto.rs` 同形

### 2.9 `deploy/k3s-local/bff-deployment.yaml` (envoy 独立 deployment, per 9/1 13:05 JST 偏好)

```yaml
---
# BFF envoy 独立 deployment (per 9/1 13:05 JST 用户偏好, 0 istio sidecar)
# 0 istio 控制面 + 0 sidecar 自动注入
# 业务 svc 通过 svc:// 引用
apiVersion: apps/v1
kind: Deployment
metadata:
  name: bff-envoy
  namespace: star
  labels:
    app: bff-envoy
    component: edge-proxy
spec:
  replicas: 1
  selector:
    matchLabels:
      app: bff-envoy
  template:
    metadata:
      labels:
        app: bff-envoy
        component: edge-proxy
      annotations:
        # 显式 0 istio sidecar 注入
        sidecar.istio.io/inject: "false"
    spec:
      containers:
        - name: envoy
          image: envoyproxy/envoy:v1.31-latest
          ports:
            - containerPort: 8080
              name: http
            - containerPort: 9901
              name: admin
          volumeMounts:
            - name: envoy-config
              mountPath: /etc/envoy
              readOnly: true
      volumes:
        - name: envoy-config
          configMap:
            name: bff-envoy-config
---
apiVersion: v1
kind: Service
metadata:
  name: bff-envoy
  namespace: star
spec:
  type: ClusterIP
  selector:
    app: bff-envoy
  ports:
    - name: http
      port: 80
      targetPort: 8080
    - name: admin
      port: 9901
      targetPort: 9901
---
# 业务 BFF svc (Rust axum + tokio-tungstenite, port 3000)
apiVersion: apps/v1
kind: Deployment
metadata:
  name: bff
  namespace: star
  labels:
    app: bff
    component: bff-runtime
spec:
  replicas: 1
  selector:
    matchLabels:
      app: bff
  template:
    metadata:
      labels:
        app: bff
        component: bff-runtime
      annotations:
        sidecar.istio.io/inject: "false"
    spec:
      containers:
        - name: bff
          image: ghcr.io/ulysses-star/bff:0.1.0
          ports:
            - containerPort: 3000
              name: http
          env:
            - name: RUST_LOG
              value: info
            - name: BIND_ADDR
              value: "0.0.0.0:3000"
---
apiVersion: v1
kind: Service
metadata:
  name: bff
  namespace: star
spec:
  type: ClusterIP
  selector:
    app: bff
  ports:
    - name: http
      port: 80
      targetPort: 3000
---
# envoy config (per 9/1 13:05 JST 偏好, 业务 svc 通过 svc:// 引用)
apiVersion: v1
kind: ConfigMap
metadata:
  name: bff-envoy-config
  namespace: star
data:
  envoy.yaml: |
    admin:
      address:
        socket_address: { address: 0.0.0.0, port_value: 9901 }
    static_resources:
      listeners:
        - name: listener_0
          address:
            socket_address: { address: 0.0.0.0, port_value: 8080 }
          filter_chains:
            - filters:
                - name: envoy.filters.network.http_connection_manager
                  typed_config:
                    "@type": type.googleapis.com/envoy.extensions.filters.network.http_connection_manager.v3.HttpConnectionManager
                    stat_prefix: ingress_http
                    route_config:
                      name: local_route
                      virtual_hosts:
                        - name: bff_service
                          domains: ["*"]
                          routes:
                            - match: { prefix: "/" }
                              route: { cluster: bff_cluster }
                    http_filters:
                      - name: envoy.filters.http.router
      clusters:
        - name: bff_cluster
          type: STRICT_DNS
          lb_policy: ROUND_ROBIN
          load_assignment:
            cluster_name: bff_cluster
            endpoints:
              - lb_endpoints:
                  - endpoint:
                      address:
                        socket_address:
                          address: bff.star.svc.cluster.local
                          port_value: 80
---
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization
namespace: star
resources:
  - bff-deployment.yaml
```

### 2.10 `docs/deployment/BFF-DEPLOYMENT-001.md` (per AGENTS.md §3 7 段结构)

7 段结构:
- §0 目的 + §1 改动矩阵 (bff/ 6 file + bff-deployment.yaml 1 file + BFF-DEPLOYMENT-001.md 1 file)
- §2 验证摘要 (kustomize build + kubectl apply --dry-run)
- §3 envoy 独立 deployment 模式 (per 9/1 13:05 JST, 0 istio sidecar)
- §4 集成流程 (5 步: bff build → image push → ConfigMap apply → bff-envoy apply → bff apply → 验证 WSS 推送)
- §5 守门规则 (15 项 per AGENTS.md §4)
- §6 签字栏 (5 角色 per DEC-008)
- §7 修订历史 (v0.1 落档)

---

## §3 守门实证要求

### 3.1 守门 #1 v25 (cargo test 改单 crate 跳 workspace)

worker 子代理 在 worktree 跑 (5 守门实证):
- `cargo check -p bff --lib -j 4` = **0 err** (V0.1 新建, 0 V0.1 baseline)
- `cargo test -p bff --lib -j 4` = **N/N PASS 0.00s** (V0.1 6-10 new UT, 0 regression)
- `cargo fmt -p bff -- --check` = **0 diff**
- `cargo clippy -p bff --lib -j 4` = **0 warnings on my code** (advisory per 守门 #7 v3)
- `cargo check --workspace --lib -j 4` = **0 err** (跟 V0.1 + V0.2 任务 1.1-1.4 全部兼容, bff 是新独立 workspace 不影响根 workspace)

### 3.2 守门 #11 缺标比错标 (4 已知缺口显式列)

worker 子代理 在 brief catch-up commit 写明:
- (a) `bff/Cargo.toml` 是新独立 workspace, **不**加到根 `Cargo.toml` [workspace] members (跟 `tools/` / `frontend/` 模式一致)
- (b) `bff` 跟 `crates/api` 平级, **不互相依赖** (BFF 调 API, 不反过来, 0 改 crates/api/Cargo.toml 加 bff 依赖)
- (c) envoy 独立 deployment 0 istio sidecar (per 9/1 13:05 JST 偏好, 显式标 `sidecar.istio.io/inject: "false"` annotation)
- (d) 0 真实 WSS 业务逻辑 (0 CRDT 0 NATS 0 真实 broadcast subscribe, 留 P3-D.6 阶段 3 集成 任务 3.2)

### 3.3 守门 #14 v4 (Mavis 审核 author=Ulysses)

commit author=`Ulysses <ulysses@mavis.local>` (per 守门 #10 + 守门 #14 v4):
```bash
git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' commit -m '...'
```

### 3.4 守门 #1 禁回溯叙事 (0 改 V0.1 任何代码)

worker 子代理 在 commit 显式标 0 改:
- 根 `Cargo.toml` [workspace] members 任何行 (bff 是新独立 workspace)
- `crates/api/Cargo.toml` 任何行 (0 添加 bff 依赖)
- `crates/api/src/{agent,arg,canvas_collab}/` 任何文件 (任务 1.4 100% 保留)
- `crates/canvas-collab/src/models/` 任何文件 (任务 1.3 100% 保留)
- `crates/agent-domain/src/` 任何文件 (任务 1.1 + 1.2 + 1.3 agent 100% 保留)
- `crates/arg-bridge/src/canvas_sync_bridge.rs` (任务 1.2 100% 保留)
- `deploy/k3s-local/install-sealed-secrets.sh` / 现有 yaml 任何行
- `Cargo.lock` 任何手编行 (cargo 自动)

### 3.5 守门 #6 PowerShell only

worker 子代理 commit 必用 PowerShell 命令, 不用 bash:
- 0 `&&` / 0 `head` / 0 `tail` / 0 `ls -la` / 0 `grep` / 0 `wc` (用 `;` / `Get-Content -TotalCount` / `Get-ChildItem` / `Select-String` / `(Get-Content).Count`)

### 3.6 守门 #19 v19 累积规 (0 破坏 V0.1)

worker 子代理 在 commit 显式标:
- V0.1 crates/api/src/arg/ 5 file 100% 保留
- V0.1 crates/api/src/lib.rs 521 lines 100% 保留
- V0.1 crates/api/Cargo.toml 45 lines 100% 保留
- V0.1 crates/canvas-collab/ 8 file 100% 保留
- V0.1 crates/agent-domain/ V0.1+V0.2+V0.3 全部 100% 保留
- V0.1 crates/arg-bridge/ 100% 保留
- V0.1 deploy/k3s-local/ 现有文件 100% 保留
- V0.2 任务 1.4 100% 保留 (api/src/{agent,canvas_collab}/ 2 新 module 2521 lines)
- V0.2 仅追加 bff/ 新独立 workspace (8 file) + deploy/k3s-local/bff-deployment.yaml (1 file) + docs/deployment/BFF-DEPLOYMENT-001.md (1 file)

---

## §4 交付物 (Deliverable)

worker 子代理 在 worktree 撰写 + commit + push branch:

1. **`bff/` 新独立 workspace** 8 file (~1500 lines):
   - `bff/Cargo.toml` (~50 lines: [package] + [workspace] + [dependencies] + [lints])
   - `bff/src/lib.rs` (~30 lines: module doc + 1 sub-mod + re-exports + 1-2 UT)
   - `bff/src/collaboration/{mod,controller,wss_hub,permission,audit,dto}.rs` 6 file (~80+500+400+150+120+400 = 1650 lines + 10-15 UT)
   - 总估 ~1700 lines

2. **`deploy/k3s-local/bff-deployment.yaml`** (~120 lines, envoy 独立 deployment)

3. **`docs/deployment/BFF-DEPLOYMENT-001.md`** (~150 lines, 7 段结构 per AGENTS.md §3)

4. **10-15 新 UT** (per 守门 #1 v25 实证 0 regression):
   - `bff/src/lib.rs` 1-2 UT (build_router smoke test)
   - `bff/src/collaboration/mod.rs` 1-2 UT (CollaborationState::new + build_router)
   - `bff/src/collaboration/permission.rs` 1-2 UT (PermissionLevel rank + satisfy)
   - `bff/src/collaboration/audit.rs` 1-2 UT
   - `bff/src/collaboration/wss_hub.rs` 1-2 UT (broadcast channel)

5. **commit message** 包含:
   - author=Ulysses
   - 5 守门实证 (cargo check 0 err + cargo test N PASS + cargo fmt 0 diff + cargo clippy 0 warnings + kustomize build 0 err)
   - 0 改 V0.1 任何代码
   - 0 改 根 Cargo.toml [workspace] members (bff 是新独立 workspace)
   - 5 域 Lead 真人未到位 Mavis 临时代签

---

## §5 Acceptance Criteria

worker 子代理 提交前必跑 + 在 commit message 显式标:
- [x] `cargo check -p bff --lib -j 4` = 0 err
- [x] `cargo test -p bff --lib -j 4` = (V0.1 0 + V0.2 N) / (V0.1 0 + V0.2 N) PASS 0.00s, 0 regression
- [x] `cargo fmt -p bff -- --check` = 0 diff
- [x] `cargo clippy -p bff --lib -j 4` = 0 warnings on my code
- [x] `cargo check --workspace --lib -j 4` = 0 err (跟 V0.1 + V0.2 任务 1.1-1.4 全部兼容)
- [x] `kustomize build deploy/k3s-local/` (含 bff-deployment.yaml) = 0 err

---

## §6 风险 / 已知缺口

1. **`bff/Cargo.toml` [workspace] 跟 crates/ 冲突** (per 守门 #11 缺标比错标 必查): `bff/Cargo.toml` 是新独立 workspace, **不**加到根 `Cargo.toml` [workspace] members, 跟 `tools/` / `frontend/` 模式一致. 0 冲突.
2. **`tokio-tungstenite` 是否在根 `Cargo.toml` [workspace.dependencies]** (per 守门 #11 缺标比错标 必查): worker 必先 `cat ../Cargo.toml | Select-String 'tokio-tungstenite'`, 如果不在, 加到根 [workspace.dependencies] 或用具体 version 0.21+. (V0.1 根 Cargo.toml 可能没这个 dep, 0 重复加, 显式标)
3. **`axum 0.7+` 的 WSS 集成 pattern** (per 守门 #11 缺标比错标 必查): `axum::extract::ws::WebSocketUpgrade` 是稳定 API, 但 `tokio-tungstenite` 的 `WebSocketStream` 可能需要手动 wrap. 显式标 placeholder 0 业务方法.
4. **0 业务方法实装** (per §1.2 out-of-scope): 9 endpoint handler body 全部占位, 留 P3-D.6 阶段 2 任务 2.3 A12 业务实装.
5. **0 真实 WSS 业务逻辑** (per §1.2 out-of-scope): 0 CRDT 0 NATS 0 真实 broadcast subscribe, 留 P3-D.6 阶段 3 集成 任务 3.2.
6. **5 域 Lead 真人未到位**: Mavis 临时代签 (per 守门 #14 v2 拍板 D), 真人到位后追溯签字覆盖修订历史.

---

## §7 commit message 模板 (worker 子代理 用)

```
feat(bff): P3-D.6 阶段 1 基础 任务 1.5 bff/ 新独立 workspace + bff/src/collaboration/ 5 REST + 4 WSS + envoy 独立 deployment 落档 (per 实施计划 §3 任务 1.5 + DD-AGENT §3.1 line 301-307 + 9/1 13:05 JST envoy 独立 deployment 偏好 + 守门 #9 v19 + 守门 #9 v20 + 守门 #10 author=Ulysses + 守门 #14 v4 + 守门 #1 禁回溯叙事 0 改 V0.1 + 守门 #19 v19 累积规 + 守门 #6 PowerShell only + 守门 #11 缺标比错标): 10 files changed, +~2400/-0 lines (0 deletions), 0 子代理调用除自身外: (1) **bff/ 新独立 workspace** 8 file: (a) bff/Cargo.toml 新建 (V0.1, [package] + [workspace] + 9 deps + [lints]); (b) bff/src/lib.rs module doc + pub mod collaboration + re-exports + 1-2 UT; (c-h) bff/src/collaboration/{mod,controller,wss_hub,permission,audit,dto}.rs 6 新增 (80+500+400+150+120+400 = 1650 lines, 5 REST endpoint A12.1-A12.5 + 4 WSS endpoint A12.1+A12.2+A12.3+A12.4 + CollaborationWssHub broadcast channel + PermissionLevel 3 变体 View/Comment/Edit + CollaborationAuditEvent 6 字段 + 10-15 UT); (2) **deploy/k3s-local/bff-deployment.yaml** 1 新增 (120 lines, envoy 独立 deployment 0 istio sidecar, bff-envoy Deployment + bff-envoy Service + bff Deployment + bff Service + bff-envoy-config ConfigMap + Kustomization); (3) **docs/deployment/BFF-DEPLOYMENT-001.md** 1 新增 (150 lines, 7 段结构 per AGENTS.md §3); (4) **守门 #1 v25 cargo test 单 crate 实证**: cargo check -p bff --lib -j 4 = 0 err; cargo test -p bff --lib -j 4 = (V0.1 0 + V0.2 N) / (V0.1 0 + V0.2 N) PASS 0.00s 0 regression; cargo fmt -p bff --check = 0 diff; cargo clippy -p bff --lib -j 4 = 0 warnings on my code; cargo check --workspace --lib -j 4 = 0 err (跟 V0.1 + V0.2 任务 1.1-1.4 全部兼容, bff 是新独立 workspace 不影响根 workspace); kustomize build deploy/k3s-local/ = 0 err; (5) **守门合规 8 维**: #1 (cargo test PASS N/N) + #1 v15 (0 改 docs commit 饱和) + #1 v19 (5 守门全套跑) + #1 v25 (cargo test 改单 crate 跳 workspace) + #6 (PowerShell only 0 bash &&) + #9 v19 (Mavis 自驱第 7 次强化) + #9 v20 (子代理 dispatch 必先 brief 落档) + #9 v27 (RPC 失败 fallback 3 段, 本次 RPC 成功) + #10 (author=Ulysses) + #11 (缺标比错标, 4 已知缺口显式标) + #12 ([M] docs 同步 留 Mavis root session 续做) + #14 v4 (Mavis 审核 author=Ulysses) + #19 v19 (累积规不破坏 V0.1, V0.1 任务 1.1-1.4 全部 100% 保留); (6) **0 改 V0.1 任何代码**: 0 改根 Cargo.toml [workspace] members 任何行 (bff 是新独立 workspace) + 0 改 crates/api/Cargo.toml 加 bff 依赖 + 0 改 crates/api/src/{agent,arg,canvas_collab}/ 任何 file + 0 改 crates/canvas-collab/ 任何 file + 0 改 crates/agent-domain/ 任何 file + 0 改 crates/arg-bridge/ 任何 file + 0 改 deploy/k3s-local/ 现有 file; (7) **累计 P3-D.6 阶段 1 基础**: 任务 1.1 + 1.2 + 1.3 + 1.4 + **任务 1.5 (本 commit)** 收官, 任务 1.6-1.7 跨 session 续做估 ~0.50M tokens / 0.42 SRE·周; (8) **守门 #1 禁回溯叙事**: v0.1-v0.99 修订历史不动, 本 commit 显式标 P3-D.6 阶段 1 基础 任务 1.5 收官; (9) **触发**: 2026-09-10 20:08 JST Ulysses 拍板"推进" + 守门 #9 v19 Mavis 自驱第 7 次强化; commit author=Ulysses (per 守门 #10 + 守门 #14 v4)
```

---

## §8 docs 同步 (per 守门 #12 v21)

worker 子代理 commit 后, Mavis root session 必跑:
1. `docs/reports/STAR-P3-WBS-001.md` v0.99.X row (per 守门 #11 缺标比错标 跟任务 1.4 v0.99.2 / 任务 2.1 batch 1 v0.99.1 / P0-4 Stage 4.3 v0.99 平行编号)
2. `scripts/automation/registry.md` v0.24 row
3. `docs/automation-design.md` §4.34.4 段
4. `docs/briefs/p3-d6-1-5-bff-skeleton.md` (本 brief) catch-up commit

---

## §9 跨 session 续做 (per 守门 #9 v19 Mavis 自驱)

任务 1.5 收官后跨 session 续做:
- **任务 1.6** 14+15 张表 SQL DDL 落档 (per 任务 1.5 依赖), 守门 #13 W/T/M 100% 覆盖 + 0 混在, 估 ~0.3M tokens
- **任务 1.7** 25 module 联动接口定义 (per 任务 1.4-1.6 依赖), 估 ~0.2M tokens
- **任务 2.1-2.5** 阶段 2 业务实装 A1-A12 46 项 + G1-G12 32 项 + 13 关键 class, 估 ~1.6M tokens / 1.33 SRE·周
- **任务 3.1-3.4** 阶段 3 集成 23 REST + 5 WSS + 25 module 联动
- **任务 4.1-4.6** 阶段 4 实装 e2e test + k3s deploy

---

**Brief end** (per 守门 #9 v20 子代理 dispatch 必先 brief 落档 + 守门 #12 v21 [P] docs 同步)
