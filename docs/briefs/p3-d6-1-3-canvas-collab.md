# P3-D.6 阶段 1 基础 任务 1.3 = `crates/canvas-collab/` 新 crate 骨架

> **brief ID**: `p3-d6-1-3-canvas-collab`
> **触发**: 2026-09-10 20:08 JST Ulysses 拍板"推进" (per 守门 #9 v19 Mavis 自驱 + 9/8 15:29 JST 第 7 次强化 Mavis 自驱不被动等指令, 继续 P3-D.6 阶段 1 任务 1.3 端到端闭环)
> **worktree**: `D:/Star/.worktrees/wt-p3-d6-1-3-canvas-collab` (branch `wt-p3-d6-1-3-canvas-collab`, 基于 main `8272931` 即 任务 1.2 brief catch-up commit)
> **受领方**: worker 子代理
> **关联文档** (per 守门 #1 禁回溯叙事, 不重写):
> - `docs/implementation-plans/CANVAS-IMPL-PLAN-001.md` v0.1 §3 阶段 1 基础 任务 1.3 (~0.2M tokens, 0.17 SRE·周)
> - `docs/design/DD-CANVAS-AGENT-001.md` v0.1 §3.1 module 布局 (5 域 Rust struct + trait + impl) + §4.14 A12 sub-class
> - `docs/design/DD-AGENT-RELATIONSHIP-001.md` v0.1 (ARG 源)
> - `docs/design/DD-CANVAS-001.md` v0.1.1 (C-25 `CanvasElementsBackend` + C-26 `CanvasMultiUserAudit` per v0.1.1 修复)

---

## §1 任务范围

**目标**: 在 worktree 里创建 `crates/canvas-collab/` 新 Rust crate 骨架 (5 domain Rust struct + trait + impl 0 业务方法), 为后续阶段 2 任务 2.3 A12 多人编辑 8 项业务方法 + 阶段 3 集成 任务 3.1-3.2 API + WSS 端点落地打基础.

**In-Scope**:
1. `crates/canvas-collab/Cargo.toml` (workspace 模式, serde + uuid + chrono + serde_json 引用)
2. `crates/canvas-collab/src/lib.rs` (module doc + 5 module 重导出 + CanvasCollabError enum)
3. `crates/canvas-collab/src/models/mod.rs` + 5 个子模块 (per DD-AGENT §3.1 5 domain Rust struct):
   - `element.rs`: `CanvasElementBackend` (12 字段, per DD-001 C-25)
   - `presence.rs`: `PresenceCursor` (8 字段)
   - `comment.rs`: `CanvasComment` (9 字段)
   - `permission.rs`: `CanvasPermission` (7 字段, 3 级权限 view/comment/edit)
   - `audit.rs`: `CanvasMultiUserAudit` (9 字段, per DD-001 C-26, Transaction append-only per 守门 #13 b)
4. 5-10 UT (每个 struct 1-2 UT, 0 业务方法)
5. 根 `Cargo.toml` [workspace] members 加 1 成员
6. 0 改 Cargo.toml 现有 dependencies (chrono/serde/serde_json/uuid 已在)
7. 0 改 Cargo.lock 手动
8. 1 commit `feat(canvas-collab): 阶段 1 基础 任务 1.3 crates/canvas-collab/ 新 crate 骨架 落档` (per 守门 #10 author=Ulysses)

**Out-of-Scope** (per 守门 #1 禁回溯叙事 + 守门 #19 v19 累积规不破坏 V0.1):
- 不写 业务方法 (create_element / update_cursor / post_comment / grant_permission / record_audit 等 留阶段 2 任务 2.3)
- 不写 API endpoint (留阶段 3 任务 3.1)
- 不写 WSS / WebSocket (留阶段 3 任务 3.2)
- 不动 V0.1 任何代码
- 不动现有 crates/ 任何子目录 (除 [workspace] members +1)
- 不动 Cargo.lock 手动

---

## §2 落地清单 (per DD-CANVAS-AGENT-001 v0.1 §3.1 + §4.14 + DD-CANVAS-001 v0.1.1 C-25/C-26)

### 2.1 根 `Cargo.toml` 修改 (1 处)

```toml
# [workspace] members 加 1 成员
members = [
    # ... 现有 71 crates ...,
    "crates/canvas-collab",  # 新增 (P3-D.6 阶段 1 任务 1.3)
]
```

注: 实际写前, worker 必须 `cat Cargo.toml` 确认现有 members 列表, 不重复添加 (per守门 #11 缺标比错标). 现有 dependencies (chrono/serde/serde_json/uuid) 全部已在, 0 重复加.

### 2.2 `crates/canvas-collab/Cargo.toml` (新文件)

```toml
[package]
name = "canvas-collab"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { workspace = true }
uuid = { workspace = true }
chrono = { workspace = true }
serde_json = { workspace = true }
```

注: 最小依赖, 0 业务依赖 (tokio / async-trait / sqlx / axum 等 留阶段 2/3).

### 2.3 `crates/canvas-collab/src/lib.rs` (新文件)

```rust
//! canvas-collab: A12 多人编辑 业务逻辑 (per P3-D.6 实施计划 §4.3)
//!
//! 本 crate 是 P3-D.6 启动实装 阶段 1 基础 任务 1.3, 仅骨架 (0 业务方法),
//! 阶段 2 业务 实装阶段 才落地 A12 多人编辑 8 项业务方法 (create_element / update_cursor / post_comment / grant_permission / record_audit / ...).
//!
//! 派生自 `docs/design/DD-CANVAS-AGENT-001.md` v0.1 §3.1 module 布局 + §4.14 A12 sub-class + `docs/design/DD-CANVAS-001.md` v0.1.1 C-25 `CanvasElementsBackend` + C-26 `CanvasMultiUserAudit`.
//! 5 域 Lead 真人未到位, Mavis 临时代签 (per 守门 #14 v2 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D).
//! 真人到位后追溯签字覆盖修订历史 (per 守门 #1 禁回溯叙事).

#![warn(missing_docs)]

pub mod models;

pub use models::audit::CanvasMultiUserAudit;
pub use models::comment::CanvasComment;
pub use models::element::CanvasElementBackend;
pub use models::permission::CanvasPermission;
pub use models::presence::PresenceCursor;

/// Canvas-collab 错误类型 (per 守门 #11 缺标比错标, 显式定义, 0 业务方法).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanvasCollabError {
    /// 阶段 1 基础 占位, 阶段 2 业务 落地业务错误
    NotImplemented,
}
```

### 2.4 `crates/canvas-collab/src/models/mod.rs` (新文件)

```rust
//! 域模型 (per DD-AGENT §3.1 + §4.14 A12 sub-class)

pub mod audit;
pub mod comment;
pub mod element;
pub mod permission;
pub mod presence;
```

### 2.5 5 个子模块文件 (per DD-AGENT §4.14 + DD-001 C-25/C-26)

#### 2.5.1 `crates/canvas-collab/src/models/element.rs` (新, ~150 lines, per C-25)

```rust
//! `CanvasElementBackend` 域模型 (per DD-AGENT §4.14 + DD-001 C-25, 12 字段).
//!
//! 仅 struct 字段定义 + Default impl + derive, 0 业务方法 (留阶段 2 任务 2.3).

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanvasElementBackend {
    pub id: Uuid,
    pub canvas_id: Uuid,
    pub kind: String,                // e.g. "sticky" / "frame" / "connector" / "shape" / "text" / "image"
    pub x: f32,                      // 画布坐标
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub rotation: f32,
    pub z_index: i32,                // z-order
    pub content: serde_json::Value,  // 任意 JSON content
    pub locked: bool,                 // 锁状态
    pub hidden: bool,                 // 隐藏状态
    pub version: i32,                 // optimistic lock
}

impl Default for CanvasElementBackend {
    fn default() -> Self {
        Self {
            id: Uuid::nil(),
            canvas_id: Uuid::nil(),
            kind: String::new(),
            x: 0.0, y: 0.0, width: 0.0, height: 0.0, rotation: 0.0,
            z_index: 0,
            content: serde_json::Value::Null,
            locked: false,
            hidden: false,
            version: 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canvas_element_backend_default() {
        let elem = CanvasElementBackend::default();
        assert_eq!(elem.id, Uuid::nil());
        assert_eq!(elem.canvas_id, Uuid::nil());
        assert_eq!(elem.z_index, 0);
        assert_eq!(elem.version, 1);
    }

    #[test]
    fn test_canvas_element_backend_clone() {
        let elem1 = CanvasElementBackend::default();
        let elem2 = elem1.clone();
        assert_eq!(elem1, elem2);
    }
}
```

#### 2.5.2 `crates/canvas-collab/src/models/presence.rs` (新, ~120 lines)

```rust
//! `PresenceCursor` 域模型 (per DD-AGENT §4.14, 8 字段, 0 业务方法).

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PresenceCursor {
    pub id: Uuid,
    pub canvas_id: Uuid,
    pub user_id: Uuid,
    pub user_name: String,
    pub x: f32,
    pub y: f32,
    pub color: String,                // e.g. "#FF5733" (per 12-color 调色板, color-blind 友好)
    pub last_seen: DateTime<Utc>,     // 30s heartbeat TTL (per NFR-AGENT-MU-CONS-01)
}

impl Default for PresenceCursor {
    fn default() -> Self {
        Self {
            id: Uuid::nil(),
            canvas_id: Uuid::nil(),
            user_id: Uuid::nil(),
            user_name: String::new(),
            x: 0.0, y: 0.0,
            color: "#000000".to_string(),
            last_seen: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_presence_cursor_default() {
        let cur = PresenceCursor::default();
        assert_eq!(cur.id, Uuid::nil());
        assert_eq!(cur.color, "#000000");
    }
}
```

#### 2.5.3 `crates/canvas-collab/src/models/comment.rs` (新, ~140 lines)

```rust
//! `CanvasComment` 域模型 (per DD-AGENT §4.14, 9 字段, 0 业务方法).

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanvasComment {
    pub id: Uuid,
    pub canvas_id: Uuid,
    pub thread_id: Uuid,              // 评论线程 ID (跟 comment_pin 关联)
    pub parent_id: Option<Uuid>,      // 回复评论的父评论 ID
    pub author_id: Uuid,
    pub content: String,
    pub mentioned_user_ids: Vec<Uuid>, // @ 提醒 (per A12.5)
    pub resolved: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for CanvasComment {
    fn default() -> Self {
        Self {
            id: Uuid::nil(),
            canvas_id: Uuid::nil(),
            thread_id: Uuid::nil(),
            parent_id: None,
            author_id: Uuid::nil(),
            content: String::new(),
            mentioned_user_ids: Vec::new(),
            resolved: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_canvas_comment_default() {
        let c = CanvasComment::default();
        assert_eq!(c.id, Uuid::nil());
        assert!(!c.resolved);
        assert!(c.mentioned_user_ids.is_empty());
    }
}
```

#### 2.5.4 `crates/canvas-collab/src/models/permission.rs` (新, ~110 lines)

```rust
//! `CanvasPermission` 域模型 (per DD-AGENT §4.14 + A12.7, 7 字段, 0 业务方法).
//!
//! 3 级权限: view / comment / edit, BFF middleware 强制 (per 守门 #5 + 9/1 13:03+13:05 JST envoy 偏好).

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// 3 级权限枚举 (per A12.7, BFF middleware 强制).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PermissionLevel {
    View,
    Comment,
    Edit,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanvasPermission {
    pub id: Uuid,
    pub canvas_id: Uuid,
    pub user_id: Uuid,
    pub level: PermissionLevel,
    pub granted_by: Uuid,
    pub granted_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl Default for CanvasPermission {
    fn default() -> Self {
        Self {
            id: Uuid::nil(),
            canvas_id: Uuid::nil(),
            user_id: Uuid::nil(),
            level: PermissionLevel::View,
            granted_by: Uuid::nil(),
            granted_at: Utc::now(),
            expires_at: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_canvas_permission_default() {
        let p = CanvasPermission::default();
        assert_eq!(p.level, PermissionLevel::View);
    }
}
```

#### 2.5.5 `crates/canvas-collab/src/models/audit.rs` (新, ~140 lines, per C-26 + 守门 #13 b)

```rust
//! `CanvasMultiUserAudit` 域模型 (per DD-AGENT §4.14 + DD-001 C-26, 9 字段, 0 业务方法).
//!
//! Transaction append-only (per 守门 #13 b 物理删除禁止), SCD Type 2 (per 守门 #13 c), 100% RLS 13 类 (per 守门 #13 a).

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanvasMultiUserAudit {
    pub id: Uuid,
    pub canvas_id: Uuid,
    pub actor_user_id: Uuid,            // 操作者
    pub action: String,                  // e.g. "create_element" / "update_element" / "delete_element" / "post_comment" / "resolve_comment"
    pub target_id: Uuid,                 // 操作目标 ID
    pub target_type: String,              // e.g. "element" / "comment" / "permission"
    pub payload: serde_json::Value,      // 操作 payload (before/after snapshot)
    pub version: i32,                     // SCD Type 2 version
    pub created_at: DateTime<Utc>,
}

impl Default for CanvasMultiUserAudit {
    fn default() -> Self {
        Self {
            id: Uuid::nil(),
            canvas_id: Uuid::nil(),
            actor_user_id: Uuid::nil(),
            action: String::new(),
            target_id: Uuid::nil(),
            target_type: String::new(),
            payload: serde_json::Value::Null,
            version: 1,
            created_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_canvas_multi_user_audit_default() {
        let a = CanvasMultiUserAudit::default();
        assert_eq!(a.version, 1);
        assert_eq!(a.action, "");
    }
}
```

### 2.6 1 commit (per 守门 #10 author=Ulysses + 守门 #1 禁回溯叙事)

```bash
git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' add crates/canvas-collab/ Cargo.toml
git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' commit -m 'feat(canvas-collab): 阶段 1 基础 任务 1.3 crates/canvas-collab/ 新 crate 骨架 落档 (per P3-D.6 实施计划 §3 阶段 1 基础 任务 1.3 + 守门 #9 v19 Mavis 自驱 + 守门 #10 author=Ulysses + 守门 #14 v4 Mavis 审核 author=Ulysses + 守门 #1 禁回溯叙事不重写 V0.1 任何代码 + 守门 #19 v19 累积规不破坏 V0.1 + 守门 #11 缺标比错标): crates/canvas-collab/{Cargo.toml,src/{lib.rs,models/{mod.rs,element.rs,presence.rs,comment.rs,permission.rs,audit.rs}}} 落档 (5 域 Rust struct CanvasElementBackend/PresenceCursor/CanvasComment/CanvasPermission/CanvasMultiUserAudit + 1 enum PermissionLevel view/comment/edit + 6 UT, 0 业务方法 0 API 0 endpoint 留阶段 2 业务 + 阶段 3 集成 实装) + Cargo.toml [workspace] members + 1 成员 (chrono/serde/serde_json/uiduuid 全部已在现有 0 重复) + Cargo.lock cargo 自动更新'
```

注: 上述 commit message 包含笔误 `uiduuid` 是输入错误, 实际写时改为 `uuid`.

---

## §3 守门合规清单 (per 守门 #1 累积规 + 守门 #14 v2/v3/v4 + 守门 #9 v19/v20/v27)

| # | 守门 | 实证 |
|---|---|---|
| #1 v25 | cargo test 单 crate 实证 | `cargo test -p canvas-collab --lib -j 4` = N/N PASS (期望 6/6 PASS: 5 struct × 1 UT + 1 struct 2 UT) |
| #1 禁回溯叙事 | 不重写 V0.1 任何代码, 不动现有 crates/ 任何子目录 | 仅新增 crates/canvas-collab/ + 改根 Cargo.toml 1 行 |
| #9 v20 | 子代理 dispatch 必先 brief 落档 | 本 brief 已落 `docs/briefs/p3-d6-1-3-canvas-collab.md` |
| #9 v27 | RPC 失败 fallback 3 段 | invoke → verify → collect_output |
| #10 | commit author=Ulysses | `git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' commit` |
| #11 | 缺标比错标 | `cat Cargo.toml` 验证现有 members + dependencies 不重复 |
| #14 v3 | 5 角色签字栏 | 本 brief 含 5 角色声明 (per AGENTS.md §3 7 段结构) |
| #14 v4 | v0.62 反转 Mavis 审核 | author=Ulysses (per 守门 #14 v4) |
| #19 v19 | 不破坏 V0.1 | 0 动 V0.1 任何代码, 0 重写 V0.1 game 5 份 PHASE 报告 |
| #19 v19 累积规 | V0.1 派生 | 0 动 PHASE-* 报告 |

---

## §4 返报要求 (per 守门 #9 v27 3 段 + brief §4 6 项, 简化版 6 项)

1. **实际写入文件路径 + 字节数** (7 文件 + Cargo.toml 1 行修改):
   - `crates/canvas-collab/Cargo.toml` (新, ~150 bytes)
   - `crates/canvas-collab/src/lib.rs` (新, ~500 bytes)
   - `crates/canvas-collab/src/models/mod.rs` (新, ~70 bytes)
   - `crates/canvas-collab/src/models/element.rs` (新, ~1500 bytes, 含 2 UT)
   - `crates/canvas-collab/src/models/presence.rs` (新, ~1100 bytes, 含 1 UT)
   - `crates/canvas-collab/src/models/comment.rs` (新, ~1300 bytes, 含 1 UT)
   - `crates/canvas-collab/src/models/permission.rs` (新, ~1100 bytes, 含 1 UT + PermissionLevel enum)
   - `crates/canvas-collab/src/models/audit.rs` (新, ~1200 bytes, 含 1 UT)
   - 根 `Cargo.toml` 改 +1 line (members +1)
   - 根 `Cargo.lock` cargo 自动 +N lines

2. **守门 #1 v25 实证**:
   - `cargo check -p canvas-collab --lib -j 4` = 0 err
   - `cargo test -p canvas-collab --lib -j 4` = N/N PASS (期望 6/6)
   - `cargo fmt -p canvas-collab -- --check` = 0 diff
   - `cargo clippy -p canvas-collab --lib -j 4` = 0 warnings (advisory per 守门 #7 v3)

3. **守门 #1 禁回溯叙事 实证**:
   - `git diff main --stat` 输出 (应仅 crates/canvas-collab/ 新增 + Cargo.toml 改 1 行)
   - `git diff main -- Cargo.toml` 输出 (应仅 +1 line)

4. **守门 #11 缺标比错标 实证**:
   - `grep "canvas-collab" Cargo.toml` = 1 match (新加 members)
   - `grep "serde\|uuid\|chrono\|serde_json" Cargo.toml` 现有行数 vs 新加行数 (避免重复)

5. **1 commit 落地**:
   - `git log --oneline -1` = "feat(canvas-collab): 阶段 1 基础 任务 1.3 ..."
   - `git log -1 --format='%an <%ae>'` = "Ulysses <ulysses@mavis.local>"
   - `git show HEAD --stat` 输出

6. **任何意外 / 偏离 / 简化 / 跳过 项, 显式标注**:
   - brief 模板 commit message 含 `uiduuid` 笔误 → 实际写时改为 `uuid` (显式标)
   - f32 字段不支持 Eq derive (跟任务 1.1 一样) → 5 struct derive 都不加 Eq, 仅 PartialEq (除非无 f32 字段)
   - chrono / serde_json / serde / uuid 已在根 Cargo.toml 现有, 0 重复加
   - 0 业务方法 0 API 0 endpoint 0 schema 留阶段 2/3
   - 任何编译警告 / 错误 显式列

---

## §5 拍板来源

- **2026-09-10 20:08 JST Ulysses 拍板** "推进" (per 守门 #9 v19 Mavis 自驱 + 9/8 15:29 JST 第 7 次强化 Mavis 自驱不被动等指令, 继续 P3-D.6 阶段 1 任务 1.3 端到端闭环)
- **2026-09-10 19:55 JST Ulysses 选 extend 方案** (per ask_user 拍板, 任务 1.2 extend 不新建 3 crate 因 crates/arg/ + arg-bridge/ + arg-effect/ 3 crate 已存在)
- **2026-09-10 19:40 JST Ulysses 拍板** "按照 wbs 开子代理 worktree 制作, 完成后合并到 main" (per 守门 #9 v19 Mavis 自驱 + 9/8 15:19 JST 第 6 次强化 Mavis 全权)
- **守门 #1 v15 docs 同步饱和** 第 90 次新事件触发 (开 worktree + 1 commit + merge) 仍允许
- **守门 #9 v20** 子代理 dispatch 必先 brief 落档 (本 brief 已落档, 在 main `8272931` 工作树)
- **守门 #9 v27** RPC 失败 fallback 3 段 invoke → verify → collect_output
- **守门 #1 v19 批量改** 本任务单 crate 1 commit 收官

---

## §6 后续步骤 (per 守门 #9 v27 + 守门 #12 v21 [P] docs 同步)

1. **子代理完成 1 commit** (本 brief 范围)
2. **Mavis 验证** (per 守门 #9 v27 verify 阶段, `git log -p --follow crates/canvas-collab/Cargo.toml` + `cargo test -p canvas-collab --lib -j 4`)
3. **merge to main** (per Ulysses 指令, `git checkout main && git merge wt-p3-d6-1-3-canvas-collab --no-ff -F .git/MERGE_MSG_P3_D6_1_3.tmp`)
4. **WBS v0.x + §4.x + registry v0.x 同步** (per 守门 #12 v21 [P] docs 同步必更新 §4 + registry)
5. **worktree cleanup** (`git worktree remove .worktrees/wt-p3-d6-1-3-canvas-collab` + `git branch -d wt-p3-d6-1-3-canvas-collab`)

---

> **brief 撰写完成**: 2026-09-10 20:10 JST, root session mvs_942987595a124037901d37205a548e6f
> **worktree 路径**: `D:/Star/.worktrees/wt-p3-d6-1-3-canvas-collab`
> **branch**: `wt-p3-d6-1-3-canvas-collab` (基于 main `8272931` 即 任务 1.2 brief catch-up commit)
> **下次拍板触发**: 子代理完成 1 commit → Mavis 验证 → merge to main → WBS + §4.x + registry docs 同步
