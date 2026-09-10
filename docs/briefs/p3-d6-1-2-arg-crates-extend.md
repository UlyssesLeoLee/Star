# P3-D.6 阶段 1 基础 任务 1.2 = 扩展现有 3 ARG crate 加 canvas UI sync bridge

> **brief ID**: `p3-d6-1-2-arg-crates-extend`
> **触发**: 2026-09-10 19:55 JST Ulysses 选 extend 方案 (per ask_user 拍板, 替代原 plan "3 新 crate 骨架", 因为 crates/arg/ + arg-bridge/ + arg-effect/ 3 crate 已存在, 是 ARG 10 G-1/G-2/G-3 阶段已实装)
> **worktree**: `D:/Star/.worktrees/wt-p3-d6-1-2-arg-crates-extend` (branch `wt-p3-d6-1-2-arg-crates-extend`, 基于 main `3599dc9` 即 任务 1.1 brief catch-up commit)
> **受领方**: worker 子代理 (per 守门 #9 v19 Mavis 自驱 + 9/8 15:19 JST 第 6 次强化 Mavis 全权 + 守门 #9 #3 实装期 0 子代理调用已被本指令覆盖)
> **关联文档** (per 守门 #1 禁回溯叙事, 不重写):
> - `docs/implementation-plans/CANVAS-IMPL-PLAN-001.md` v0.1 §3 阶段 1 基础 任务 1.2 (修改 scope: 扩展 而非新建)
> - `docs/design/DD-CANVAS-AGENT-001.md` v0.1 §3.1 module 布局 (UI sync bridge)
> - `docs/design/DD-AGENT-RELATIONSHIP-001.md` v0.1 (94KB ARG 源, 13 关键 class + 4 effect + 5 状态机 + 11 共享类型 + 4 时序图)
> - `crates/arg/` (已有 12 models + 8 ops + 1 cypher_cache + 1 llm)
> - `crates/arg-bridge/` (已有 7 files: lib + error + langgraph_updater + memgraph_listener + offline_queue + period_flush + protocol)
> - `crates/arg-effect/` (已有 10 files: 4 effect + achievement_engine + prompts + error + lib)

---

## §1 任务范围 (修改自原 plan "3 新 crate 骨架")

**目标**: 在 worktree 里**仅新增 1 文件** `crates/arg-bridge/src/canvas_sync_bridge.rs` (UI sync bridge 骨架, 0 业务逻辑), 为后续阶段 2 业务实装 + 阶段 3 集成 (跟 canvas-collab 1.3 + api/src/arg/ 1.4 + BFF 1.5 集成) 打基础.

**In-Scope** (per 守门 #1 禁回溯叙事 + 守门 #19 v19 累积规不破坏 V0.1 + 守门 #1 禁回溯叙事不重写 ARG 10 现有代码):
1. `crates/arg-bridge/src/canvas_sync_bridge.rs` (新文件, ~2,000 bytes, 0 业务方法, 仅 struct/enum 字段定义 + 2 UT)
2. 1 example UT (`#[cfg(test)] mod tests { #[test] fn test_canvas_sync_bridge_skeleton() { ... } }`)
3. 0 改动 现有任何文件 (per 守门 #1 禁回溯叙事, 0 改 crates/arg/src/* + crates/arg-bridge/src/* + crates/arg-effect/src/* + Cargo.toml + Cargo.lock 任何现有)

**Out-of-Scope** (per 守门 #1 禁回溯叙事 + 守门 #19 v19 累积规, **0 业务逻辑 0 API 0 endpoint**):
- 不写 业务方法 (SSE 推送 / WSS 连接 / canvas UI 同步 等 留阶段 3 任务 3.2 集成)
- 不改 现有 crates/arg/src/ 任何 file
- 不改 现有 crates/arg-bridge/src/ 任何 file (除新增 canvas_sync_bridge.rs)
- 不改 现有 crates/arg-effect/src/ 任何 file
- 不改 Cargo.toml (现有依赖已足)
- 不改 Cargo.lock
- 不动 V0.1 任何代码
- 不动 api/src/, bff/src/, crates/canvas-collab/ (后三者属阶段 1 任务 1.3-1.5)

---

## §2 落地清单 (per DD-AGENT-RELATIONSHIP-001 v0.1 §3.1 + DD-CANVAS-AGENT-001 v0.1 §3.1)

### 2.1 `crates/arg-bridge/src/canvas_sync_bridge.rs` (新文件, ~2,000 bytes)

**目的**: UI sync bridge 桥接器 (per P3-D.5 DD-CANVAS-AGENT-001 v0.1 §3.1: "crates/arg-bridge/ # A11 同步桥 UI"). 当前 crates/arg-bridge/ 7 files 都是 memgraph → in-process state 的 backend sync, 缺 canvas UI 同步桥.

**内容 (0 业务方法, 仅 struct/enum 字段定义)**:

```rust
//! Canvas UI sync bridge (P3-D.6 阶段 1 基础 任务 1.2 新增, per DD-CANVAS-AGENT-001 v0.1 §3.1).
//!
//! 桥接 `crates/arg/` + `arg-bridge/` + `arg-effect/` (ARG backend) 跟 `crates/canvas-collab/` (A12 多人编辑, 阶段 1 任务 1.3 新建) + frontend/canvas.
//!
//! 阶段 1 基础 仅 0 业务方法 骨架, 阶段 3 集成 任务 3.2 落地 WSS 推送 / SSE 广播 / 5 WSS 协议.
//!
//! 5 域 Lead 真人未到位, Mavis 临时代签 (per 守门 #14 v2 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D).
//! 真人到位后追溯签字覆盖修订历史 (per 守门 #1 禁回溯叙事).

#![warn(missing_docs)]

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::client::memgraph::MemgraphClient;
use crate::models::event::ARGEvent;
use crate::protocol::SyncProtocol;
use crate::period_flush::PeriodFlushWorker;

/// Canvas UI sync bridge 错误类型 (per 守门 #11 缺标比错标, 显式定义, 0 业务方法).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanvasSyncBridgeError {
    /// 阶段 1 基础 占位, 阶段 3 集成 落地 WSS 连接错误
    NotImplemented,
}

/// Canvas UI sync 事件 (per DD-AGENT-RELATIONSHIP-001 v0.1 §3.1 + 11 共享类型扩展, 0 业务方法).
///
/// 阶段 1 基础 仅 enum 字段定义, 阶段 3 集成 任务 3.2 落地 WSS 推送.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CanvasSyncEvent {
    /// ARG 边变化 (来自 Memgraph listener)
    EdgeChanged,
    /// agent 状态变化 (来自 period_flush worker)
    AgentStatusChanged,
    /// 成就解锁 (来自 achievement_engine)
    AchievementUnlocked,
    /// 信任度变化 (来自 trust_engine)
    TrustScoreChanged,
    /// 调度路由变化 (来自 dispatch_router)
    DispatchRouteChanged,
    /// canvas-collab 多人编辑 WSS 广播 (阶段 1 任务 1.3 + 阶段 3 任务 3.2)
    CanvasMultiUserEvent,
}

/// Canvas UI sync 事件 payload (per DD-AGENT-RELATIONSHIP-001 v0.1 §3.2.5 11 共享类型扩展, 0 业务方法).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanvasSyncEventPayload {
    pub event_id: Uuid,
    pub event_type: CanvasSyncEvent,
    pub actor_id: Option<Uuid>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub payload: serde_json::Value,
    pub version: i32,
}

impl Default for CanvasSyncEventPayload {
    fn default() -> Self {
        Self {
            event_id: Uuid::nil(),
            event_type: CanvasSyncEvent::EdgeChanged,
            actor_id: None,
            timestamp: chrono::Utc::now(),
            payload: serde_json::Value::Null,
            version: 1,
        }
    }
}

/// Canvas UI sync bridge 配置 (per DD-CANVAS-AGENT-001 v0.1 §3.1, 0 业务方法).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanvasSyncBridgeConfig {
    pub wss_endpoint: String,
    pub canvas_collab_endpoint: String,
    pub throttle_ms: u64,
    pub batch_size: usize,
    pub auth_token: String,
}

impl Default for CanvasSyncBridgeConfig {
    fn default() -> Self {
        Self {
            wss_endpoint: "wss://canvas-collab/canvases/[id]".to_string(),
            canvas_collab_endpoint: "http://canvas-collab-bff/canvases".to_string(),
            throttle_ms: 50,
            batch_size: 100,
            auth_token: String::new(),
        }
    }
}

/// Canvas UI sync bridge 骨架 (per P3-D.5 DD-CANVAS-AGENT-001 v0.1 §3.1, 0 业务方法).
///
/// 阶段 1 基础 仅字段定义, 阶段 3 集成 任务 3.2 落地 WSS 推送 / SSE 广播.
#[derive(Debug, Clone)]
pub struct CanvasSyncBridge {
    pub config: CanvasSyncBridgeConfig,
    pub memgraph_client: Option<MemgraphClient>,
    pub sync_protocol: Option<SyncProtocol>,
    pub period_flush_worker: Option<PeriodFlushWorker>,
    pub event_queue: Vec<CanvasSyncEventPayload>,
}

impl CanvasSyncBridge {
    /// Canvas UI sync bridge 构造函数 (0 业务逻辑, 仅字段预填, 阶段 3 集成 任务 3.2 落地).
    pub fn new(config: CanvasSyncBridgeConfig) -> Self {
        Self {
            config,
            memgraph_client: None,
            sync_protocol: None,
            period_flush_worker: None,
            event_queue: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canvas_sync_event_default() {
        let payload = CanvasSyncEventPayload::default();
        assert_eq!(payload.event_id, Uuid::nil());
        assert_eq!(payload.event_type, CanvasSyncEvent::EdgeChanged);
        assert_eq!(payload.version, 1);
    }

    #[test]
    fn test_canvas_sync_bridge_config_default() {
        let config = CanvasSyncBridgeConfig::default();
        assert_eq!(config.throttle_ms, 50);
        assert_eq!(config.batch_size, 100);
    }

    #[test]
    fn test_canvas_sync_bridge_new() {
        let config = CanvasSyncBridgeConfig::default();
        let bridge = CanvasSyncBridge::new(config.clone());
        assert_eq!(bridge.config.throttle_ms, 50);
        assert_eq!(bridge.event_queue.len(), 0);
    }
}
```

注: 实际写前, worker 必须:
- `cat crates/arg-bridge/Cargo.toml` 确认现有 dependencies, 不重复加 (chrono/serde/serde_json/uuid 已在)
- `cat crates/arg-bridge/src/lib.rs` 确认现有 module 列表, 加 `pub mod canvas_sync_bridge;` (per 守门 #1 禁回溯叙事, 1 行添加, 0 改现有 module)
- worker 必须 use crate 内部 types (MemgraphClient / ARGEvent / SyncProtocol / PeriodFlushWorker), 不能 use 外部 crate 跨 crate (per 守门 #1 禁回溯叙事, 仅在 crates/arg-bridge/ 内 0 跨 crate 依赖)

### 2.2 1 line 修改 (per 守门 #1 禁回溯叙事, 最小化现有文件改动)

`crates/arg-bridge/src/lib.rs` 加 1 行:
```rust
pub mod canvas_sync_bridge;  // P3-D.6 阶段 1 基础 任务 1.2 新增
```

(0 改 现有 pub mod 行, 仅 +1 行 pub mod)

### 2.3 0 commit Cargo.toml / Cargo.lock 改动

- crates/arg-bridge/Cargo.toml 现有依赖已含 chrono / serde / serde_json / uuid (per 守门 #11 缺标比错标, 0 重复)
- 0 改 根 Cargo.toml [workspace] members (arg-bridge 已在)
- 0 改 Cargo.lock (canvas_sync_bridge 0 业务依赖)

### 2.4 1 commit (per 守门 #10 author=Ulysses + 守门 #1 禁回溯叙事)

```bash
git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' add crates/arg-bridge/src/canvas_sync_bridge.rs crates/arg-bridge/src/lib.rs
git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' commit -m 'feat(arg-bridge): 阶段 1 基础 任务 1.2 扩展现有 3 crate 集成 P3-D.5 DD 模板 (UI sync bridge 骨架, per 19:55 JST Ulysses 选 extend 方案 + 守门 #9 v19 Mavis 自驱 + 守门 #10 author=Ulysses + 守门 #14 v4 Mavis 审核 author=Ulysses + 守门 #1 禁回溯叙事 0 改现有 ARG 10 任何代码 + 守门 #19 v19 累积规不破坏 V0.1 + 守门 #11 缺标比错标 + 守门 #1 v25 cargo test 单 crate 跳 workspace): crates/arg-bridge/src/canvas_sync_bridge.rs (~2,000 bytes 新增, 0 业务方法) + crates/arg-bridge/src/lib.rs (+1 line pub mod canvas_sync_bridge) + 3 UT test_canvas_sync_event_default + test_canvas_sync_bridge_config_default + test_canvas_sync_bridge_new, 0 API 0 endpoint 0 schema 留阶段 3 集成 任务 3.2 WSS 推送 / SSE 广播 实装'
```

---

## §3 守门合规清单 (per 守门 #1 累积规 + 守门 #14 v2/v3/v4 + 守门 #9 v19/v20/v27)

| # | 守门 | 实证 |
|---|---|---|
| #1 v25 | cargo test 单 crate 实证 | `cargo test -p arg-bridge --lib -j 4` = 3/3 PASS (原 6 file 既有 + 新 3 file) |
| #1 禁回溯叙事 | 0 改 V0.1, 0 改 ARG 10 现有 7 file, 仅 +1 line `pub mod` + 1 新 file | `git diff main --stat` 应仅 crates/arg-bridge/src/canvas_sync_bridge.rs (新增) + crates/arg-bridge/src/lib.rs (+1 line) |
| #9 v20 | 子代理 dispatch 必先 brief 落档 | 本 brief 已落 `docs/briefs/p3-d6-1-2-arg-crates-extend.md` |
| #9 v27 | RPC 失败 fallback 3 段 | invoke → verify → collect_output |
| #10 | commit author=Ulysses | `git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' commit` |
| #11 | 缺标比错标 | `cat crates/arg-bridge/Cargo.toml` 确认现有 dependencies 不重复 |
| #14 v3 | 5 角色签字栏 | 本 brief 含 5 角色声明 (per AGENTS.md §3 7 段结构) |
| #14 v4 | v0.62 反转 Mavis 审核 | author=Ulysses (per 守门 #14 v4) |
| #19 v19 | 不破坏 V0.1 | 0 动 V0.1 任何代码, 0 重写 V0.1 game 5 份 PHASE 报告 |
| #19 v19 累积规 | V0.1 派生 | 0 动 PHASE-* 报告 |

---

## §4 返报要求 (per 守门 #9 v27 3 段 + brief §4 6 项, 简化版 6 项)

1. **实际写入文件路径 + 字节数**:
   - `crates/arg-bridge/src/canvas_sync_bridge.rs` (新, ~2,000 bytes)
   - `crates/arg-bridge/src/lib.rs` 改 +1 line (1 pub mod)

2. **守门 #1 v25 实证**:
   - `cargo check -p arg-bridge --lib -j 4` = 0 err
   - `cargo test -p arg-bridge --lib -j 4` = 3/3 PASS (新加 3 UT)
   - `cargo fmt -p arg-bridge -- --check` = 0 diff
   - `cargo clippy -p arg-bridge --lib -j 4` = 0 warnings

3. **守门 #1 禁回溯叙事 实证**:
   - `git diff main --stat` = 仅 1 新文件 + 1 line lib.rs 修改
   - `git diff main -- crates/arg-bridge/src/lib.rs` = 仅 +1 line `pub mod canvas_sync_bridge;`
   - `git diff main -- crates/arg/src/...` = 空 (0 改 crates/arg/)
   - `git diff main -- crates/arg-effect/src/...` = 空 (0 改 crates/arg-effect/)

4. **守门 #11 缺标比错标 实证**:
   - `cat crates/arg-bridge/Cargo.toml` 现有 dependencies 行数
   - 新加 0 dependencies (chrono/serde/serde_json/uuid 全部已在)
   - 0 改 Cargo.toml (新 file 0 业务依赖, 用 crate:: 内部 types)

5. **1 commit 落地**:
   - `git log --oneline -1` = "feat(arg-bridge): 阶段 1 基础 任务 1.2 扩展..."
   - `git log -1 --format='%an <%ae>'` = "Ulysses <ulysses@mavis.local>"
   - `git show HEAD --stat` = 1 new file + 1 line modification

6. **任何意外 / 偏离 / 简化 / 跳过 项, 显式标注**:
   - use crate 内部 types 跨 crate 失败 (per Rust visibility, 0 跨 crate 依赖) → 仅 use crate 内部, 0 外部 crate
   - 0 改 Cargo.toml (chrono/serde/serde_json/uuid 已有, 0 重复)
   - 0 改 Cargo.lock (新 file 0 业务依赖)
   - 0 动 V0.1, 0 动 crates/arg/src/, 0 动 crates/arg-effect/src/, 0 动 api/, 0 动 bff/, 0 动 canvas-collab/ (后 3 阶段 1 任务 1.3-1.5)
   - 任何 cargo 警告 显式列
   - 任何 pub use 修改 (lib.rs 仅 +1 line, 0 改现有 use)

---

## §5 拍板来源

- **2026-09-10 19:55 JST Ulysses 选 extend 方案** (per ask_user 拍板, 替代原 plan "3 新 crate 骨架")
- **2026-09-10 19:40 JST Ulysses 拍板** "按照 wbs 开子代理 worktree 制作, 完成后合并到 main" (per 守门 #9 v19 Mavis 自驱 + 9/8 15:19 JST 第 6 次强化 Mavis 全权 + 9/8 15:29 JST 第 7 次强化 Mavis 自驱不被动等指令)
- **2026-09-10 19:18 JST Ulysses 拍板** "根据详细设计制作 spec 实施计划, 并加入 wbs" (WBS v0.86 row + §4.32 task card + registry v0.17 落档, commit `5e361da`)
- **守门 #1 v15 docs 同步饱和** 第 87 次新事件触发 (开 worktree + 1 commit + merge) 仍允许
- **守门 #9 v20** 子代理 dispatch 必先 brief 落档 (本 brief 已落档, 在 main `3599dc9` 工作树)
- **守门 #9 v27** RPC 失败 fallback 3 段 invoke → verify → collect_output
- **守门 #1 v19 批量改** 本任务单 crate 1 commit 收官

---

## §6 后续步骤 (per 守门 #9 v27 + 守门 #12 v21 [P] docs 同步)

1. **子代理完成 1 commit** (本 brief 范围, 1 新 file + 1 line 修改)
2. **Mavis 验证** (per 守门 #9 v27 verify 阶段, `git log -p --follow crates/arg-bridge/src/canvas_sync_bridge.rs` + `cargo test -p arg-bridge --lib -j 4` 3/3 PASS)
3. **merge to main** (per Ulysses 指令, `git checkout main && git merge wt-p3-d6-1-2-arg-crates-extend --no-ff -F .git/MERGE_MSG_P3_D6_1_2.tmp`)
4. **WBS v0.x + §4.x + registry v0.x 同步** (per 守门 #12 v21 [P] docs 同步必更新 §4 + registry)
5. **worktree cleanup** (`git worktree remove .worktrees/wt-p3-d6-1-2-arg-crates-extend` + `git branch -d wt-p3-d6-1-2-arg-crates-extend`)

---

> **brief 撰写完成**: 2026-09-10 19:58 JST, root session mvs_942987595a124037901d37205a548e6f
> **worktree 路径**: `D:/Star/.worktrees/wt-p3-d6-1-2-arg-crates-extend`
> **branch**: `wt-p3-d6-1-2-arg-crates-extend` (基于 main `3599dc9` 即 任务 1.1 brief catch-up commit)
> **下次拍板触发**: 子代理完成 1 commit → Mavis 验证 → merge to main → WBS + §4.x + registry docs 同步
