# Brief wt-arg-04-api-13rest-1ws: P3-C W4 ARG.4 crates/api/src/arg/ 13 REST + 1 WebSocket + RLS 13 類 (per automation-design §4.18)

> **Status**: 🟡 Active (per 2026-09-09 22:31 JST 用户发令"开子代理和worktree并行处理" + ask_4b06eee1bba60b2727e8bccb 拍板 4 个 wt 并行 + 逐个 rebase + ff merge, 推荐项)
> **Created**: 2026-09-09
> **Authority**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3 永久代签)
> **Worktree**: `wt-arg-04-api-13rest-1ws` (新)
> **依赖**: 现有 wt-arg-01-arg-crate worktree `43c1f0c` + brief `arg-04-api-13rest-1ws.md` (12.4KB 已落档 per 9/9 21:16 JST) + automation-design §4.17 + §4.18
> **跨 session 续**: per 守门 #20 v20 + 守门 #27 v27

---

## 0. 任务目标 (Objective)

在 `wt-arg-04-api-13rest-1ws` worktree 实装 ARG.4 路径: `crates/api/src/arg/` 子模块新增 13 REST endpoint + 1 WebSocket endpoint + RLS 13 類必携 (tenant_id / workspace_id / user_id / session_id / device_id / agent_id / role_id / permission_id / policy_id / event_id / tag_id / category_id / status_id), per [automation-design.md §4.18](../../docs/automation-design.md) + [brief v0.45 arg-04-api-13rest-1ws.md 已落档](../../docs/briefs/arg-04-api-13rest-1ws.md).

## 1. 已知事实 (Known Facts)

- **wt-arg-01-arg-crate 已存在** (`43c1f0c`, commit `b7ec06e`), 6 子模块 ArgCrate / SubAgentOrchestrator / LLMService / AgentLease / AgentRuntime 框架已落 (per 9/4 19:00 JST)
- **brief `arg-04-api-13rest-1ws.md` 12.4KB 已落档** (per 9/9 21:16 JST)
- **automation-design §4.17 ARG.1**: `crates/arg` 6 子模块起立, brief v0.45 文档完整化
- **automation-design §4.18 ARG.4**: `crates/api/src/arg/` 13 REST + 1 WebSocket + RLS 13 類必携
- **P3-C 9 子项** (per automation-design §4.2): ARG.4 是 W4 子项
- **守门 #19 v19**: agent 交互 Python 化 (走 scripts/automation/)
- **守门 #13 c/d**: W/T/M 横展開, RLS 13 類必携

## 2. 路径 (Ruled-out Paths)

- ❌ **重写 ARG.1 6 子模块** (per 守门 #1 禁回溯叙事, ARG.1 已落)
- ❌ **跨域 P3-B 启动** (per HANDOFF-ST-001 §5.3 Blocker #5)
- ❌ **跳过 RLS 13 類** (违反守门 #13 c/d)
- ❌ **主分支直接实装** (在 worktree 操作)
- ❌ **派 worker 子代理** (RPC 不可靠, 走 subprocess 替代)

## 3. 范围 (Exact Scope)

### 3.1 在 worktree `wt-arg-04-api-13rest-1ws` 新建 / 修改

```
crates/api/src/arg/                              # 新增子目录
├── mod.rs                                       # 13 REST + 1 WebSocket route 汇总
├── rest/                                        # 13 REST endpoints
│   ├── agent.rs                                 # POST /api/arg/agents
│   ├── lease.rs                                 # GET /api/arg/leases
│   ├── session.rs                               # POST /api/arg/sessions
│   ├── orchestrator.rs                          # PUT /api/arg/orchestrator/{id}
│   ├── llm_service.rs                           # GET /api/arg/llm/{id}
│   ├── runtime.rs                               # GET /api/arg/runtime/{id}
│   ├── context.rs                                # POST /api/arg/context
│   ├── device.rs                                # GET /api/arg/devices
│   ├── tenant.rs                                # POST /api/arg/tenants
│   ├── workspace.rs                             # GET /api/arg/workspaces
│   ├── policy.rs                                # PUT /api/arg/policy/{id}
│   ├── event.rs                                 # GET /api/arg/events
│   └── health.rs                                # GET /api/arg/health
├── websocket.rs                                 # 1 WebSocket endpoint (WS /ws/arg)
└── rls.rs                                       # RLS 13 類 middleware (tenant_id + workspace_id + ... 13 维)

crates/api/src/middleware/rls_13.rs              # 全局 RLS middleware (13 维隔离)
crates/api/src/main.rs                           # route 注册 + RLS middleware 接入
```

### 3.2 13 REST endpoints (per automation-design §4.18)

| # | Endpoint | Method | 用途 | RLS 13 類 |
|---|---|---|---|---|
| 1 | `/api/arg/agents` | POST | 创建 agent | ✅ |
| 2 | `/api/arg/leases` | GET | 查询 lease | ✅ |
| 3 | `/api/arg/sessions` | POST | 创建 session | ✅ |
| 4 | `/api/arg/orchestrator/{id}` | PUT | 更新编排 | ✅ |
| 5 | `/api/arg/llm/{id}` | GET | 查询 LLM service | ✅ |
| 6 | `/api/arg/runtime/{id}` | GET | 查询 runtime | ✅ |
| 7 | `/api/arg/context` | POST | 注入 context | ✅ |
| 8 | `/api/arg/devices` | GET | 查询 device | ✅ |
| 9 | `/api/arg/tenants` | POST | 创建 tenant | ✅ |
| 10 | `/api/arg/workspaces` | GET | 查询 workspace | ✅ |
| 11 | `/api/arg/policy/{id}` | PUT | 更新 policy | ✅ |
| 12 | `/api/arg/events` | GET | 查询 event | ✅ |
| 13 | `/api/arg/health` | GET | health check | N/A |

### 3.3 1 WebSocket endpoint

| Endpoint | Method | 用途 |
|---|---|---|
| `/ws/arg` | WS | 实时 event 推送 (per actor state change) |

### 3.4 RLS 13 類 (per 守门 #13 c/d)

```rust
// crates/api/src/middleware/rls_13.rs
pub struct RlsContext {
    pub tenant_id: Uuid,
    pub workspace_id: Uuid,
    pub user_id: Uuid,
    pub session_id: Uuid,
    pub device_id: DeviceId,
    pub agent_id: Uuid,
    pub role_id: Uuid,
    pub permission_id: Uuid,
    pub policy_id: Uuid,
    pub event_id: Uuid,
    pub tag_id: Uuid,
    pub category_id: Uuid,
    pub status_id: Uuid,
}
```

## 4. 验收 (Acceptance Criteria)

### 4.1 守门合规

| # | 守门 | 验证 |
|---|---|---|
| 1 | `cargo check --workspace --all-targets -j 4` 0 err (per 守门 #1 v2 + v19) | 实装完成后 |
| 2 | `cargo fmt --check` 0 diff | 实装完成后 |
| 3 | `cargo clippy --all-targets -- -D warnings` 0 warning (per 守门 #7) | 实装完成后 |
| 4 | `cargo test -p star-api-rest --lib -j 4` 0 fail (per 守门 #1 v25) | 实装完成后 |
| 5 | 13 REST + 1 WebSocket + RLS 13 類 100% 覆盖 | 实装完成后 |
| 6 | 5 域 Lead 真人未到位, Mavis 临时代签 (per 守门 #14 v3) | commit author=Ulysses |

### 4.2 跨 session 续做

- **本次 session**: worktree + brief (本 brief) + 报告 v0.1 (Phase A 准备)
- **下次 session #1**: RLS middleware + 13 REST route handler (~0.3M tokens)
- **下次 session #2**: WebSocket 接入 + 测试 (~0.2M tokens)
- **下次 session #3**: cargo check 0 err + merge

### 4.3 commit message 引用 brief 路径 (per 守门 #21 v21)

```bash
git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' commit -m "feat(arg): crates/api/src/arg/ 13 REST + 1 WebSocket + RLS 13 類

  per docs/briefs/wt-arg-04-api-13rest-1ws.md (ARG.4 brief) + docs/briefs/arg-04-api-13rest-1ws.md (前序 brief).
  ..."
```

## 5. 守门硬约束

- 守门 #9 v3: 调试控制台走 subprocess 替代 RPC
- 守门 #9 v19: agent 交互 Python 化
- 守门 #13 c/d: W/T/M 横展開 + RLS 13 類必携
- 守门 #14 v3: Mavis 永久代签
- 守门 #19 v19: agent 交互 Python 化
- 守门 #20 v20: 子代理 dispatch 必先 brief
- 守门 #21 v21: [P] docs 同步必更新 §4 + registry
- 守门 #27 v27 候选: 子代理 RPC 失败 fallback
- 守门 #1 v25: cargo test 改单 crate

## 6. 引用 (References)

- [automation-design §4.17 ARG.1 6 子模块](../../docs/automation-design.md) — 前序基础
- [automation-design §4.18 ARG.4 13 REST + 1 WS](../../docs/automation-design.md) — 本次任务
- [brief arg-04-api-13rest-1ws.md 12.4KB 已落档](../../docs/briefs/arg-04-api-13rest-1ws.md) — 详细实施
- wt-arg-01-arg-crate worktree 已存在 `43c1f0c` — 6 子模块基础
- [AGENTS.md §4 守门 #13 W/T/M 横展開 + RLS 13 類](../../AGENTS.md)
- [PHASE-LANGGRAPH-TMO-IMPL-REPORT v0.3.2](../reports/PHASE-LANGGRAPH-TMO-IMPL-REPORT.md) — TMO 7 节点实装 phase 范式

---

**per 守门 #14 v3 Mavis 永久代签**: author = Ulysses <ulysses@mavis.local>, 修订人 = Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手, 审批 = 架构师 (Mavis 接手 agent per DEC-008).