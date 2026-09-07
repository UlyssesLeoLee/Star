# Brief: OPT-WORKER-03 — Phase B T1.7 修法 4.1+4.2+4.3+4.4 (per OPT-WBS-06..08)

**Agent**: worker
**Phase**: OPT-P4
**Created**: 2026-09-07 12:04 JST
**Author**: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses

---

## 1. 任务目标

完成 Phase B T1.7 修法 4 子项, 实证 `--workspace --all-targets -j 4` 0 err 跨 sub-session 收敛 (per `STAR-P4-UNIMPL-WBS-001.md` §3 + 守门 #1 v3 派生规)。

**4 子项 (per P4-UNIMPL-WBS §3)**:
- **B.1**: 4.1 `ActorContext::as_local_runtime(mut self) -> Self` helper 实证 51→10 err (per `65a8da0` 部分落地)
- **B.2**: 4.2 改写 star-mcp 2 份 tests (消解 50+ err, handlers/ + tools/)
- **B.3**: 4.3 守门 #1 v3 派生规 文字补全 (`--all-targets` 716 err 5.1-5.5 报告"0 行代码改动"未保持)
- **B.4**: 4.4 守门 #1 v3 实证 (`--all-targets` 0 err 跨 sub-session 收敛)

**基线 (per 9/5 报告 §2.1)**:
- `cargo check --workspace --all-targets -j 4` = 0 err (9/5 PR #12 实证, commit `9d10565`)
- 716 err → 0 err 实证已落地 (per守门 #1 v12 100% 守门覆盖)
- 任务 = **维护 0 err baseline + 完成 4 子项推进**

---

## 2. Worktree 创建

```bash
cd D:\Star
git worktree add -b feat/opt-phase-b-t17 D:/Star/.worktrees/wt-opt-phase-b main
cd D:/Star/.worktrees/wt-opt-phase-b
```

- **Worktree 路径**: `D:/Star/.worktrees/wt-opt-phase-b`
- **Branch**: `feat/opt-phase-b-t17` (基于 main HEAD `bfb0bca`)

---

## 3. 实装要求

### 3.1 B.1 — ActorContext::as_local_runtime helper 完整化

**文件**: `crates/star-context/src/actor.rs` (per H2-EXT 实证, 已有 `is_agent_session: bool` 字段 + 4 helper)

**当前状态** (per `68ae5ff` + `27a690f`):
- 已有 helper: `is_tenant_admin()` / `is_developer()` / `is_service_internal()` / `can_access_project()` + 2 builder (`with_project()` / `with_agent_session()`)

**任务**: 加 `as_local_runtime(mut self) -> Self` method (per P4-UNIMPL B.1 描述):
```rust
impl ActorContext {
    /// Convert this actor to a local runtime context.
    /// Used by star-local-runtime to inject actor identity into process spawn.
    pub fn as_local_runtime(mut self) -> Self {
        // Set runtime context fields
        self.runtime_kind = Some(RuntimeKind::Local);
        self
    }
}
```

如果 `RuntimeKind` 不存在, 加 enum:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuntimeKind { Local, Remote, Embedded }
```

加 unit test 1 个。

### 3.2 B.2 — star-mcp 2 份 tests 改写

**文件**: 
- `crates/star-mcp/tests/handlers/test_*.rs` (估 1-2 文件)
- `crates/star-mcp/tests/tools/test_*.rs` (估 1-2 文件)

**当前状态** (per守门 #1 v3 派生规 + HANDOFF H5):
- star-mcp 134 tests pass (per A.22 实证)
- 但部分 test 跟 P0-1 ActorContext 设计不兼容, 触发 50+ err 跨 session 续

**任务**:
- 找出跟 ActorContext 不兼容的 test (per `git grep "ActorContext" crates/star-mcp/tests/`)
- 改写 use `star_context::ActorContext` (per守门 #1 v18 H2 派生)
- 加 1 个新 test 验证 ActorContext 集成

### 3.3 B.3 — 守门 #1 v3 派生规 文字补全

**文件**: `AGENTS.md` §4.1 (line 130-140 附近, 守门 #1 v3 段)

**任务**: 在 `### v3` 段加:
- 文字实证 `--all-targets` 跨 sub-session 收敛 0 err 流程
- 加 5.1-5.5 报告模板 (per守门 #1 v3 派生规)
- 引用 `STAR-P4-UNIMPL-WBS-001.md` §3 B.3 链接

**注意**: 这是 docs-only commit, per守门 #12 饱和约束, 需先有代码改动触发 (本任务有 B.1 + B.2 代码改动, 满足触发)

### 3.4 B.4 — 守门 #1 v3 实证

**任务**:
- 跑 `cargo check --workspace --all-targets -j 4` 0 err 实证
- 跑 `cargo test -p star-context --lib -j 4` 21/21 pass
- 把结果写进 `AGENTS.md` §4.1 v3 段 (per守门 #12 实证)
- 创建 `docs/reports/PHASE-P4-B-T17-IMPL-REPORT.md` (per守门 #3 报告 7 段结构)

---

## 4. 守门硬约束

1. `cargo check --workspace --all-targets -j 4` 必须 0 err (per守门 #1 v19)
2. `cargo test -p star-context --lib -j 4` 必须 21/21 pass
3. `cargo test -p star-mcp --lib -j 4` 必须 134/134 pass (维持 baseline)
4. `cargo fmt + clippy` 必须干净
5. docs 改动 commit message 必须含代码改动引用 (per守门 #12 饱和约束)
6. 禁回溯叙事 + 禁 RGS 引用 + PowerShell only

---

## 5. Commit 形式

**2 个 commit (per守门 #12 触发链)**:

Commit 1 (代码改动 B.1 + B.2):
```bash
git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' commit -m "feat(context/mcp): 完成 Phase B T1.7 B.1 + B.2 (per OPT-WORKER-03)

- B.1: ActorContext::as_local_runtime helper + RuntimeKind enum + 1 unit test
- B.2: star-mcp 2 份 tests 改写 + 1 新 test (ActorContext 集成)

Refs: docs/briefs/OPT-WORKER-03-phase-b.md
Refs: docs/reports/STAR-P4-OPT-WBS-001.md §4.1 #4
Refs: STAR-P4-UNIMPL-WBS-001.md §3 B.1 + B.2
Per守门 #10 author=Ulysses
"
```

Commit 2 (docs B.3 + 实证 B.4):
```bash
git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' commit -m "docs(agents/report): 守门 #1 v3 文字补全 + Phase B 实证 (per B.3 + B.4)

- AGENTS.md §4.1 v3 加 5.1-5.5 报告模板 (per守门 #1 v3 派生规)
- 新建 docs/reports/PHASE-P4-B-T17-IMPL-REPORT.md (7 段结构)
- cargo check --workspace --all-targets -j 4 0 err 实证
- cargo test -p star-context --lib 21/21 pass 实证

Refs: docs/briefs/OPT-WORKER-03-phase-b.md
Refs: STAR-P4-UNIMPL-WBS-001.md §3 B.3 + B.4
Per守门 #12 commit-time docs 同步 (新事件触发 = Commit 1 代码改动)
Per守门 #10 author=Ulysses
"
```

---

## 6. 交付物

返回时报告:
1. worktree 路径: `D:/Star/.worktrees/wt-opt-phase-b`
2. branch 名称: `feat/opt-phase-b-t17`
3. **2 个 commit hash** (7 字符)
4. 修改文件列表 (file:line, 跨 2 commit)
5. `cargo check --workspace --all-targets -j 4` 输出 (0 err)
6. `cargo test -p star-context + star-mcp` 输出 (全 pass)
7. 守门 #1 v3 实证 log 引用

---

## 7. 失败处理

- cargo check 不通过: 修复 + 重试, max 2 次
- 仍失败: 报告具体错误, 不 commit

---

## 8. 时间预算

≤ 400K token
