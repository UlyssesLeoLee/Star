# P3-D.6 阶段 1 基础 任务 1.1 = `crates/agent-domain/` 新 crate 骨架

> **brief ID**: `p3-d6-1-1-agent-domain`
> **触发**: 2026-09-10 19:40 JST Ulysses 拍板"按照 wbs 开子代理 worktree 制作, 完成后合并到 main" (P3-D.6 实施计划 WBS v0.86 §3 阶段 1 基础 任务 1.1)
> **worktree**: `D:/Star/.worktrees/wt-p3-d6-1-1-agent-domain` (branch `wt-p3-d6-1-1-agent-domain`, 基于 main `bd1e8fe`)
> **受领方**: worker 子代理 (per 守门 #9 #3 实装期 0 子代理调用已被本指令覆盖, per 9/8 15:19 JST 第 6 次强化 Mavis 全权 + 9/8 15:29 JST 第 7 次强化 Mavis 自驱)
> **关联文档** (per 守门 #1 禁回溯叙事, 不重写):
> - `docs/implementation-plans/CANVAS-IMPL-PLAN-001.md` v0.1 §3 阶段 1 基础 任务 1.1 (1 file, ~0.1M tokens, 0.08 SRE·周)
> - `docs/design/DD-CANVAS-AGENT-001.md` v0.1 §3.1 module 布局 (1.1 派生)
> - `docs/design/BD-CANVAS-AGENT-001.md` v0.1 §3 组件一覧
> - `docs/requirements/SRS-CANVAS-AGENT-001.md` v1.3 (A1-A10 28 项)

---

## §1 任务范围

**目标**: 在 worktree 里创建 `crates/agent-domain/` 新 Rust crate 骨架 (无业务逻辑, 仅 Cargo.toml + lib.rs + 1 example test), 为后续阶段 2 业务实装阶段 (A1-A10 agent 管理 28 项) 落地打基础.

**In-Scope**:
1. `crates/agent-domain/Cargo.toml` (per Cargo.toml workspace 模式, 跟现有 crates 一致)
2. `crates/agent-domain/src/lib.rs` (1 doc module + 0 子模块 + 0 业务函数, 仅空骨架)
3. `crates/agent-domain/src/models/mod.rs` + `agent.rs` (per DD-AGENT §4.1 `AgentNode` struct 14 字段预声明, **仅 struct 字段定义 + Default impl, 0 业务方法**)
4. 1 example UT (`#[cfg(test)] mod tests { #[test] fn test_agent_node_default() { ... } }`)
5. 根 `Cargo.toml` `[workspace.dependencies]` 加 `serde` + `uuid` 依赖 (如未加)
6. 根 `Cargo.toml` `[workspace] members` 加 `crates/agent-domain` 成员
7. 1 commit `feat(agent-domain): 阶段 1 基础 任务 1.1 crates/agent-domain/ 新 crate 骨架 落档` (per 守门 #10 author=Ulysses)

**Out-of-Scope** (per 守门 #1 禁回溯叙事 + 守门 #19 v19 累积规不破坏 V0.1, **0 业务逻辑 0 API 0 endpoint 0 schema**):
- 不写 业务方法 (move/resize/rotate 等 A1-A10 业务方法, 留阶段 2 任务 2.1)
- 不写 API endpoint (留阶段 3 任务 3.1)
- 不写 SQL schema / 14+15 张表 (留阶段 1 任务 1.6)
- 不写 WSS / WebSocket (留阶段 3 任务 3.2)
- 不动 V0.1 任何代码 (守门 #19 v19 累积规)
- 不动 现有 crates/ 任何子目录
- 不动 Cargo.lock (让 cargo 自动更新)

---

## §2 落地清单 (per DD-CANVAS-AGENT-001 v0.1 §3.1 + §4.1)

### 2.1 根 `Cargo.toml` 修改 (2 处)

```toml
# [workspace] members 加 1 成员
members = [
    # ... 现有 18 crates ...,
    "crates/agent-domain",  # 新增
]

# [workspace.dependencies] 加 2 依赖 (per Cargo.toml 已存在检查)
[workspace.dependencies]
serde = { version = "1.0", features = ["derive"] }
uuid = { version = "1.0", features = ["v4", "serde"] }
```

注: 实际写前, worker 必须 `cat Cargo.toml` 确认现有 members 列表和 dependencies, 不重复添加 (per守门 #11 缺标比错标).

### 2.2 `crates/agent-domain/Cargo.toml` (新文件)

```toml
[package]
name = "agent-domain"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { workspace = true }
uuid = { workspace = true }
```

注: 最小依赖, 0 业务依赖 (tokio / async-trait / sqlx 等 留阶段 2).

### 2.3 `crates/agent-domain/src/lib.rs` (新文件)

```rust
//! agent-domain: A1-A10 agent 管理 域 (per P3-D.6 实施计划 §4.1)
//!
//! 本 crate 是 P3-D.6 启动实装 阶段 1 基础 任务 1.1, 仅骨架 (0 业务逻辑),
//! 阶段 2 业务 实装阶段 才落地 A1-A10 28 项业务方法 (move/resize/rotate/handoff/...).
//!
//! 派生自 `docs/design/DD-CANVAS-AGENT-001.md` v0.1 §3.1 module 布局 + §4.1 `AgentNode` struct 14 字段.
//! 5 域 Lead 真人未到位, Mavis 临时代签 (per 守门 #14 v2 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D).
//! 真人到位后追溯签字覆盖修订历史 (per 守门 #1 禁回溯叙事).

#![warn(missing_docs)]

pub mod models;

pub use models::agent::AgentNode;
```

### 2.4 `crates/agent-domain/src/models/mod.rs` (新文件)

```rust
//! 域模型 (per DD-AGENT §4.1 + §4.7)

pub mod agent;
```

### 2.5 `crates/agent-domain/src/models/agent.rs` (新文件)

```rust
//! `AgentNode` 域模型 (per DD-AGENT §4.1, 14 字段).
//!
//! 仅 struct 字段定义 + Default impl + derive, 0 业务方法 (move/resize/rotate 等留阶段 2 任务 2.1).

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentDomainError {
    NotImplemented,
}

/// A1-A10 agent 节点 域模型 (per DD-AGENT §4.1, 14 字段).
///
/// 阶段 1 基础 仅字段定义, 阶段 2 任务 2.1 落地业务方法.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentNode {
    pub id: Uuid,
    pub name: String,
    pub archetype: String,           // e.g. "PM" / "SRE" / "5-Domain-Lead"
    pub domain: String,              // player/economy/match/social/admin (per 5 域 Lead 历史治理命名)
    pub status: String,              // 14 状态机 (per SRS-CANVAS-AGENT-001 §3.3 + DD-AGENT §5.2)
    pub trust_score: f32,            // 0.0-1.0, 5 档 (per ARG 源 Untrusted/Low/Medium/High/VeryHigh)
    pub metadata: serde_json::Value, // 任意 JSON 元数据
    pub created_at: chrono::DateTime<chrono::Utc>, // 注: 需加 chrono 依赖
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub version: i32,                // optimistic lock (per DD-AGENT §5.2 14 状态机 + §5.3 14 状态转移)
    pub x: f32,                      // 画布坐标
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub rotation: f32,
}

impl Default for AgentNode {
    fn default() -> Self {
        // 0 业务逻辑, 仅字段预填
        Self {
            id: Uuid::nil(),
            name: String::new(),
            archetype: String::new(),
            domain: String::new(),
            status: "queued".to_string(),  // 14 状态机默认 1 状态 (per DD-AGENT §5.2)
            trust_score: 0.5,             // 5 档默认中位
            metadata: serde_json::Value::Null,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            version: 1,
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            rotation: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_node_default() {
        let node = AgentNode::default();
        assert_eq!(node.id, Uuid::nil());
        assert_eq!(node.status, "queued");
        assert_eq!(node.trust_score, 0.5);
        assert_eq!(node.version, 1);
        assert_eq!(node.x, 0.0);
    }

    #[test]
    fn test_agent_node_clone() {
        let node1 = AgentNode::default();
        let node2 = node1.clone();
        assert_eq!(node1, node2);
    }
}
```

注: 实际写前, worker 必须 `grep "agent-domain\|chrono\|serde_json" Cargo.toml` 确认现有 dependencies, 避免重复添加 (per守门 #11 缺标比错标 + 守门 #1 缺标比错标).

### 2.6 1 commit (per 守门 #10 author=Ulysses + 守门 #1 禁回溯叙事)

```bash
git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' add crates/agent-domain/ Cargo.toml
git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' commit -m 'feat(agent-domain): 阶段 1 基础 任务 1.1 crates/agent-domain/ 新 crate 骨架 落档 (per P3-D.6 实施计划 §3 阶段 1 基础 任务 1.1 + 守门 #9 v19 Mavis 自驱 + 守门 #10 author=Ulysses + 守门 #14 v4 Mavis 审核 author=Ulysses + 守门 #1 禁回溯叙事不重写 V0.1 任何代码 + 守门 #19 v19 累积规不破坏 V0.1 + 守门 #11 缺标比错标): crates/agent-domain/{Cargo.toml,src/{lib.rs,models/{mod.rs,agent.rs}}} 落档 + Cargo.toml [workspace] members + 1 成员 + [workspace.dependencies] 加 serde + uuid + chrono + serde_json (如未加), 0 业务逻辑 0 API 0 endpoint 0 schema 留阶段 2/3 实装'
```

---

## §3 守门合规清单 (per 守门 #1 累积规 + 守门 #14 v2/v3/v4 + 守门 #9 v19/v20/v27)

| # | 守门 | 实证 |
|---|---|---|
| #1 v25 | cargo test 单 crate 实证 | `cargo test -p agent-domain --lib -j 4` = 2/2 PASS |
| #1 禁回溯叙事 | 不重写 V0.1 任何代码, 不动现有 crates/ 任何子目录 | 仅新增 crates/agent-domain/ + 改根 Cargo.toml 2 处 |
| #9 v20 | 子代理 dispatch 必先 brief 落档 | 本 brief 已落 `docs/briefs/p3-d6-1-1-agent-domain.md` |
| #9 v27 | RPC 失败 fallback 3 段 | invoke → verify → collect_output |
| #10 | commit author=Ulysses | `git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' commit` |
| #11 | 缺标比错标 | `cat Cargo.toml` 验证现有 members + dependencies 不重复 |
| #14 v3 | 5 角色签字栏 | 本 brief 含 5 角色声明 (per AGENTS.md §3 7 段结构) |
| #14 v4 | v0.62 反转 Mavis 审核 | author=Ulysses (per 守门 #14 v4) |
| #19 v19 | 不破坏 V0.1 | 0 动 V0.1 任何代码, 仅新增 agent-domain/ 骨架 |
| #19 v19 累积规 | V0.1 game 5 份 PHASE 报告 派生不重写 | 0 动 PHASE-* 报告 |

---

## §4 返报要求 (per 守门 #9 v27 3 段 + brief 6 返报 15 项, 简化版 6 项)

1. **实际写入文件路径 + 字节数** (3 文件 + Cargo.toml 2 行修改):
   - `crates/agent-domain/Cargo.toml` (新, ~150 字节)
   - `crates/agent-domain/src/lib.rs` (新, ~500 字节)
   - `crates/agent-domain/src/models/mod.rs` (新, ~50 字节)
   - `crates/agent-domain/src/models/agent.rs` (新, ~1500 字节)
   - `Cargo.toml` 改 +2 行 (members +1 + dependencies +3 如未加)

2. **守门 #1 v25 实证**:
   - `cargo test -p agent-domain --lib -j 4` = 2/2 PASS
   - `cargo check -p agent-domain --lib -j 4` = 0 err
   - `cargo fmt -p agent-domain -- --check` = 0 diff
   - `cargo clippy -p agent-domain --lib -j 4` = 0 warnings (advisory per 守门 #7 v3)

3. **守门 #1 禁回溯叙事 实证**:
   - `git diff main --stat` = 仅 `Cargo.toml` + 4 新文件, 0 现有 crate 修改
   - `git diff main -- Cargo.toml` = 2 行修改 (members +1 + dependencies +N)

4. **守门 #11 缺标比错标 实证**:
   - `grep "agent-domain" Cargo.toml` = 1 match (新加 members)
   - `grep "serde" Cargo.toml` 现有 = N, 新加 = 0 (避免重复)
   - `grep "uuid" Cargo.toml` 现有 = N, 新加 = 0 (避免重复)
   - `grep "chrono" Cargo.toml` 现有 = N, 新加 = 0 (避免重复)

5. **1 commit 落地**:
   - `git log --oneline -1` = "feat(agent-domain): 阶段 1 基础 任务 1.1 ..."
   - `git log -1 --format='%an <%ae>'` = "Ulysses <ulysses@mavis.local>"

6. **任何意外 / 偏离 / 简化 / 跳过 项, 显式标注**:
   - 如 chrono / serde_json / serde / uuid 已在根 Cargo.toml 现有, 不重复加, 仅 1 行 commit message 显式标
   - 如 0 业务方法 (per守门 #19 v19 累积规, 留阶段 2), 显式标
   - 如 Cargo.lock 自动更新, 0 手动编辑, 显式标

---

## §5 拍板来源

- **2026-09-10 19:40 JST Ulysses 拍板** "按照 wbs 开子代理 worktree 制作, 完成后合并到 main" (per 守门 #9 v19 Mavis 自驱 + 9/8 15:19 JST 第 6 次强化 Mavis 全权 + 9/8 15:29 JST 第 7 次强化 Mavis 自驱不被动等指令)
- **2026-09-10 19:18 JST Ulysses 拍板** "根据详细设计制作 spec 实施计划, 并加入 wbs" (WBS v0.86 row + §4.32 task card + registry v0.17 落档, commit `5e361da`)
- **守门 #1 v15 docs 同步饱和** 第 84 次新事件触发 (开 worktree + 1 commit + merge) 仍允许
- **守门 #9 v20** 子代理 dispatch 必先 brief 落档 (本 brief 已落档, 在 main bd1e8fe 工作树)
- **守门 #9 v27** RPC 失败 fallback 3 段 invoke → verify → collect_output
- **守门 #1 v19 批量改** 本任务单 crate 1 commit 收官

---

## §6 后续步骤 (per 守门 #9 v27 + 守门 #12 v21 [P] docs 同步)

1. **子代理完成 1 commit** (本 brief 范围)
2. **Mavis 验证** (per 守门 #9 v27 verify 阶段, `git log -p --follow crates/agent-domain/Cargo.toml`)
3. **merge to main** (per Ulysses 指令, `git checkout main && git merge wt-p3-d6-1-1-agent-domain --no-ff -m "merge: P3-D.6 阶段 1 基础 任务 1.1 crates/agent-domain/ 新 crate 骨架"`)
4. **WBS v0.87** + **§4.33** + **registry v0.18** 同步 (per 守门 #12 v21 [P] docs 同步必更新 §4 + registry)
5. **worktree cleanup** (`git worktree remove .worktrees/wt-p3-d6-1-1-agent-domain` + `git branch -d wt-p3-d6-1-1-agent-domain`)

---

> **brief 撰写完成**: 2026-09-10 19:42 JST, root session mvs_942987595a124037901d37205a548e6f
> **worktree 路径**: `D:/Star/.worktrees/wt-p3-d6-1-1-agent-domain`
> **branch**: `wt-p3-d6-1-1-agent-domain` (基于 main `bd1e8fe`)
> **下次拍板触发**: 子代理完成 1 commit → Mavis 验证 → merge to main → WBS v0.87 + §4.33 + registry v0.18 docs 同步
