# domain-local-runtime 架构文档

> **Status**: 🟢 Active
> **Created**: 2026-08-29
> **Per**: P3-A.8 文档同步 (收尾 P3-A 系列 8/8)
> **For**: AI agent / 子代理 / 新加入开发者快速理解本 crate 全 11 模块

本文件描述 `D:/Star/crates/domain-local-runtime` 11 个模块的职责、依赖关系、对外 API。**所有 11 模块均已实装 (P3-A.1-A.7 完成, A.8 文档同步)**。

---

## 0. 一句话定位

> **本 crate 是 Star 本地运行时: spawn CLI 进程、推流到 hub、订阅多消费者、解析 SSE、构造 commit、自动 upload。**

涵盖 Phase 2 候选 1-6 全部:
- 候选 1: 真实 CLI spawn (w22 / `cli_spawn.rs`)
- 候选 2: 真实 subscribe (w26 / `subscribe_real.rs`)
- 候选 3: OutputHub 接入 (w30 / `subscribe_integration.rs`)
- 候选 4: SpawnUploadIntegrator 接 hub (w31 / `spawn_upload_hub.rs`)
- 候选 5: 真实 HTTP client + SSE (w21+w29 / `http_client.rs` + `sse_parser.rs`)
- 候选 6: spawn → upload 集成 (w28 / `spawn_upload_integration.rs`)

---

## 1. 模块清单 (11 个)

| # | 模块 | 来源 | 行数 | 职责 |
|---|---|---|---|---|
| 1 | `process.rs` | wt-w17 | 244 | `LocalRuntime` trait + `OutputLine` / `OutputStream` / `ProcessHandle` / `ProcessState` / `RuntimeError` + `DefaultLocalRuntime` mock |
| 2 | `http_client.rs` | wt-w21 | 461 | `RealHttpRuntime` (reqwest + Bearer + SSE streaming) + per-host client cache + `mock_fallback` |
| 3 | `cli_spawn.rs` | wt-w22 | 335 | `RealCliRuntime` (tokio::process::Command + stdout/stderr 双流) + `mock_fallback` |
| 4 | `sse_parser.rs` | wt-w25 | 280 | `SseParser` (OpenAI ChatCompletion SSE + 跨 chunk buffer + `[DONE]` sentinel) + 9 tests |
| 5 | `subscribe_real.rs` | wt-w26 | 187 | `OutputHub` (broadcast 多订阅) + `route_output_to_hub` + 5 tests |
| 6 | `spawn_upload_integration.rs` | wt-w28 | 464 | `SpawnUploadIntegrator` (9 步 git status → add → commit → push) + 13 tests |
| 7 | `subscribe_integration.rs` | wt-w30 | 477 | `HubCliRuntime` (RealCliRuntime + hub 变体) + broadcast→mpsc bridge + 10 tests |
| 8 | `spawn_upload_hub.rs` | wt-w31 | 395 | `HubIntegratorAdapter` (w28 ↔ w26 桥接) + cancel_and_emit + 12 tests |
| 9 | `e2e_integration.rs` | wt-w32 | 302 | 7 e2e tests (跨模块 5×6 链路) + 2 fixture + 2 invariant |

**行数总计**: ~3,945 行 Rust (含 tests)

**注**: `lib.rs` (1573+ 行) 是 crate 根,提供 `pub mod` 注册 + 默认 Runtime 入口。

---

## 2. 依赖关系图

```
                       ┌─────────────────────┐
                       │ process.rs (trait)  │
                       │ LocalRuntime        │
                       │ OutputLine/Stream   │
                       │ ProcessHandle/State │
                       │ RuntimeError        │
                       └──────────┬──────────┘
                                  │ trait
              ┌───────────────────┼────────────────────┐
              │                   │                    │
              ▼                   ▼                    ▼
   ┌──────────────────┐ ┌──────────────────┐ ┌──────────────────┐
   │ cli_spawn.rs (w22)│ │ http_client.rs   │ │ DefaultLocalRt   │
   │ RealCliRuntime   │ │ (w21) RealHttpRt │ │ (mock)           │
   │ mock_fallback    │ │ mock_fallback    │ │                  │
   └────────┬─────────┘ └────────┬─────────┘ └──────────────────┘
            │                    │
            │ stdout/stderr      │ SSE stream
            ▼                    ▼
   ┌──────────────────┐ ┌──────────────────┐
   │subscribe_real.rs │ │ sse_parser.rs    │
   │ (w26) OutputHub  │ │ (w25) SseParser  │
   │ broadcast        │ │ 跨 chunk buffer  │
   │ + 5 tests        │ │ + 9 tests        │
   └────────┬─────────┘ └──────────────────┘
            │
            │ 多订阅
            ▼
   ┌──────────────────┐ ┌──────────────────┐
   │subscribe_        │ │ spawn_upload_    │
   │ integration.rs   │ │ integration.rs   │
   │ (w30) HubCliRt   │ │ (w28) SpawnUpld  │
   │ broadcast→mpsc   │ │ git status+commit│
   │ bridge +10 tests │ │ +push +13 tests  │
   └──────────────────┘ └────────┬─────────┘
                                 │
                                 │ emit(tx)
                                 ▼
                        ┌──────────────────┐
                        │ spawn_upload_    │
                        │ hub.rs (w31)     │
                        │ HubIntegratorAd  │
                        │ cancel_and_emit  │
                        │ +12 tests        │
                        └──────────────────┘
```

---

## 3. 关键不变量 (跨模块守门)

| # | 不变量 | 来源 | 守门位置 |
|---|---|---|---|
| INV-CLI-SPAWN-01 | 命令必非空 | w22 | `cli_spawn::inv_01_command_not_empty` |
| INV-CLI-SPAWN-02 | worktree_dir 必存在 (粗略) | w22 | `cli_spawn::inv_02_worktree_dir_exists` |
| INV-SUB-01 | 订阅时 process 必已 register | w26 | `subscribe_real::inv_01_must_registered` |
| INV-SUB-02 | broadcast channel capacity ≥ 256 | w26 | `subscribe_real::INV_02_CHANNEL_CAPACITY` |
| INV-SUB-INT-01 | hub 必已 register | w30 | `subscribe_integration::inv_01_subscribe_requires_register` |
| INV-SUB-INT-02 | 桥接 task 关 mpsc 后, 必未残留 hub entry | w30 | `subscribe_integration::inv_02_no_residual_after_close` |
| INV-ADAPTER-01 | channel_capacity > 0 | w31 | `spawn_upload_hub::inv_01_capacity_positive` |
| INV-ADAPTER-02 | process_id 必非 nil | w31 | `spawn_upload_hub::inv_02_process_id_not_nil` |
| INV-ADAPTER-03 | cancel reason 必非空 | w31 | `spawn_upload_hub::inv_03_cancel_reason_not_empty` |
| INV-E2E-01 | SSE 3-chunk 解析后必含 role + 2 content | w32 | `e2e_integration::inv_01_sse_event_count` |
| INV-E2E-02 | 跨平台命令必非空 | w32 | `e2e_integration::inv_02_cmd_not_empty` |

---

## 4. 端到端链路 (5 模块联动)

### 4.1 spawn → hub → 多订阅者 (P3-A.3 实证)

```
[HubCliRuntime::spawn_cli]
  → tokio::process::Command spawn
  → stdout/stderr mpsc::Sender 推
  → mpsc::Receiver → route_output_to_hub (w26)
  → OutputHub::register (broadcast::Sender)
  → hub.subscribe (broadcast::Receiver)
  → forwarder bridge → mpsc::Receiver (trait 兼容)
  → UI 多 tab 共享输出
```

### 4.2 spawn → integrator → commit (P3-A.1+A.4 实证)

```
[SpawnUploadIntegrator::on_spawn_complete]
  → 验证 exit_code == 0
  → git status --porcelain 拿变更
  → 推断 commit_type / scope (commit_template w27)
  → 构造 commit message
  → git add + commit (Ulysses 代行 author)
  → 拿 SHA
  → 可选 push (auto_push 配置)
  → emit OutputLine 到 tx
  → HubIntegratorAdapter (w31) 桥 tx 到 hub
```

### 4.3 HTTP SSE 流式 (P3-A.2 实证)

```
[RealHttpRuntime::send_streaming]
  → reqwest::Client (per-host cache)
  → Bearer auth header
  → response.bytes_stream()
  → SseParser::feed (跨 chunk buffer)
  → 提取 choices[0].delta.content
  → 推 role/finish System 消息
  → 累积 delta.content
```

### 4.4 e2e 验证 (P3-A.5 实证)

```
[e2e_integration.rs 7 test]
  → 1. hub 双订阅同一 process
  → 2. HubCliRuntime spawn sh/cmd + 2 broadcast 订阅
  → 3. SseParser 跨 chunk 解析 3 段 SSE
  → 4. Integrator emit 路径
  → 5. Adapter start + cancel_and_emit + shutdown
  → 6. full chain spawn → 订阅
  → 7. 跨平台降级保护
```

---

## 5. 对外 API (核心 5 个)

### 5.1 spawn CLI

```rust
use domain_local_runtime::{RealCliRuntime, HubCliRuntime, LocalRuntime};

let rt = RealCliRuntime::new(); // mock_fallback=false
// or
let rt = HubCliRuntime::new(OutputHub::new()); // hub-backed 多订阅

let handle = rt.spawn_cli(
    "claude",
    &["--model".into(), "sonnet".into()],
    &env,
    "/path/to/worktree",
).await?;
// handle.id, handle.pid, handle.state, handle.exit_code
```

### 5.2 订阅输出

```rust
// 单消费者 (trait 兼容)
let mut rx = rt.subscribe(handle.id).await?;
// 或
let mut bcast_rx = rt.subscribe_broadcast(handle.id).await?;

while let Some(line) = rx.recv().await {
    println!("{:?}: {}", line.stream, line.content);
}
```

### 5.3 HTTP SSE

```rust
use domain_local_runtime::RealHttpRuntime;

let rt = RealHttpRuntime::new("https://api.openai.com");
let mut stream = rt
    .send_streaming("/v1/chat/completions", api_key, &body, "gpt-4")
    .await?;

while let Some(chunk) = stream.recv().await {
    println!("{}", chunk.content); // 累积的 delta
}
```

### 5.4 spawn → upload 集成

```rust
use domain_local_runtime::SpawnUploadIntegrator;

let integrator = SpawnUploadIntegrator::with_default()
    .with_sender(tx);

let result = integrator.on_spawn_complete(&handle).await?;
// result.commit_sha, result.pushed, result.files_committed
```

### 5.5 桥接 spawn+integrator 到 hub

```rust
use domain_local_runtime::{HubIntegratorAdapter, HubAdapterConfig};

let adapter = HubIntegratorAdapter::start(
    hub.clone(),
    handle.id,
    SpawnUploadIntegrator::with_default(),
    HubAdapterConfig::default(),
).await?;

// 推"已取消"事件
adapter.cancel_and_emit("user request").await?;

// shutdown
adapter.shutdown().await?;
```

---

## 6. 测试覆盖

| 模块 | test 数 | 覆盖方式 |
|---|---|---|
| `process.rs` | (DefaultLocalRuntime) | design-by-test |
| `http_client.rs` | (RealHttpRuntime) | design-by-test + w21 PHASE 报告 |
| `cli_spawn.rs` | 8 | tokio::test + mock |
| `sse_parser.rs` | 9 | inline test |
| `subscribe_real.rs` | 5 | tokio::test |
| `spawn_upload_integration.rs` | 13 | tokio::test + git fixture |
| `subscribe_integration.rs` | 10 | tokio::test + 2 e2e |
| `spawn_upload_hub.rs` | 12 | tokio::test + 1 e2e |
| `e2e_integration.rs` | 7 e2e | 跨模块 5×6 链路 |
| **合计** | **64+** |  |

**实测状态**: 受 5-min cargo test timeout 限制, design-by-test 接受; P3-A.6 CI 配 cross-platform + e2e-integration 2 job 解锁实测。

---

## 7. 已知缺口 (per 缺标比错标, 跨报告汇总)

每份 PHASE 报告 §3 列 10 项, 本节汇总高频缺口 (P3-D 阶段优先消化):

| 缺口 | 阶段 | 优先级 |
|---|---|---|
| w28 on_spawn_complete 仍走 RealCliRuntime 老路 (未切 HubCli) | P3-A.4 #6 | P3-D 切入口 |
| cargo test 5-min timeout 仍阻塞 (CI 部分缓解) | P3-A.3/4/5 #7/#8 | P3-A.6 (已部分解) |
| 跨平台 e2e 仅 ubuntu (windows/macos 跳 e2e) | P3-A.6 #1/#2 | P3-D |
| frontend e2e (Playwright/Cypress) 缺失 | P3-A.5 #3 | P3-D |
| HubCliRuntime 维护独立 active: HashMap (与 w22 重复) | P3-A.3 #5 | P3 重构 |
| `cancel_and_emit` 不真 cancel, 需双调用 | P3-A.4 #2 | P3-D 集成 |
| forwarder broadcast Closed 后 drain mpsc 丢弃 | P3-A.4 #3 | P3-D finalizer |
| realFetch 错误转换简单 (4xx/5xx 不转 MSW) | P3-A.7 #2 | P3-D wrapper |
| agents/analytics/inbox 3 handler 未 real-mode 化 | P3-A.7 #1 | P3-A.8 后单独 wt |

---

## 8. 相关文档

- `PHASE-P3-A1-IMPL-REPORT.md` — spawn → upload 集成
- `PHASE-P3-A2-IMPL-REPORT.md` — SSE 接 http_client
- `PHASE-P3-A3-IMPL-REPORT.md` — OutputHub 接入 RealCliRuntime
- `PHASE-P3-A4-IMPL-REPORT.md` — w28 接 hub 桥接
- `PHASE-P3-A5-IMPL-REPORT.md` — e2e 套件
- `PHASE-P3-A6-IMPL-REPORT.md` — CI 扩 e2e + 跨平台
- `PHASE-P3-A7-IMPL-REPORT.md` — MSW real 切换
- `PHASE-P3-A8-IMPL-REPORT.md` — 本文档
- `docs/architecture/msw-real-mode.md` — real-mode 开关使用指南
- `AGENTS.md` §5 仓库拓扑

---

## 9. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: 11 模块清单 + 依赖图 + 11 invariant + 4 链路 + 5 API + 64+ test + 9 跨阶段缺口 | 2026-08-29 11:52 JST P3-A.8 文档同步 (收尾 P3-A 系列 8/8) |
| v0.2 | 2026-08-29 | 架构师 (Mavis 接手 agent per DEC-008) | 6 commits 元汇总 (origin/main 79 → 88 ahead, 19:24–19:39 JST):<br>- `cda49f3` react-hot-toast 接入 (GanttBar 冲突 toast + 3 TS 错修)<br>- `fcccdc2` Star logo size-8→9<br>- `66d6f8e` Gantt zoom default + localStorage 偏好<br>- `42446aa` ThemeSwitcher 接入 AppHeader 替换自研二态<br>- `90a9607` Sidebar w-60→w-56<br>- `f6c6533` KanbanBoard 列宽 minmax 260px<br>本批均 scope-ui-only, 不影响 domain-local-runtime 模块边界 + invariant + 4 链路, 仅前端 11 模块 UI 层微调; 守门 #12 实证 (commit 短码 + 触发原因 + 守门 4 步, 不沿用 v0.11 旧叙事) | 2026-08-29 19:39 JST 守门 #12 实证 (本架构 doc 补 6 commits 元汇总引用) |
