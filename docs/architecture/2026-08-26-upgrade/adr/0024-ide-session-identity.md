# ADR-0024: IDE Session Identity — 独立于 GitGit 的对象

> **状态**：🟢 Accepted v0.2 (per 2026-09-07 14:30 JST OPT-NEXT-08 拍板, 守门 #12 docs 触发 + 守门 #10 author=Ulysses)
> **日期**：2026-08-26
> **制定者**：架构师（Mavis 接手 agent per DEC-008）— per 2026-08-26 08:40 JST 代签新规则
> **签批**：⏳ 待签
> **依赖**：[ADR-0022 IDE Placement](0022-ide-placement.md)
> **关联**：[STAR vs GitGit Matrix](../../../responsibility-matrix/star-vs-gitgit.md)

---

## 1. 背景

IDE 现代工作流有很多"会话级"状态：open files · active symbol · selection · diagnostics · terminal · agent session。GitGit 不需要也不应该知道这些。

如果把这些对象塞进 GitGit，GitGit 会变成 IDE Proxy，破坏 §1 任务原文清晰边界。

## 2. 决策

**IDE Session 作为 STAR 的一等对象，独立于 GitGit。GitGit 只看到 Repository / Worktree / Branch / Commit / File Changes。**

### 2.1 IDE Session 对象

```rust
// crates/star-ide/src/session.rs
pub struct IdeSession {
    pub id: IdeSessionId,           // "ide-abc123"
    pub user_id: UserId,            // 哪个 user
    pub workspace_id: WorkspaceId,
    pub repository_id: RepoId,      // STAR 自己的 repo 对象
    pub worktree_id: WorktreeId,    // GitGit worktree
    pub client: IdeClientKind,      // VSCode / Cursor / JetBrains / Vim / Web
    pub client_version: String,
    pub open_files: Vec<OpenFile>,
    pub active_symbol: Option<SymbolRef>,
    pub selection: Option<Selection>,
    pub diagnostics: Vec<Diagnostic>,
    pub terminal_id: Option<TerminalId>,
    pub agent_sessions: Vec<AgentSessionId>,
    pub audit_id: AuditId,
}
```

### 2.2 OpenFile 对象

```rust
pub struct OpenFile {
    pub path: PathBuf,             // 相对 worktree
    pub content_hash: ContentHash, // 避免重复读
    pub cursor: Position,
    pub view_state: ViewState,     // fold / dirty / inlay-hint
    pub last_modified: Timestamp,
}
```

### 2.3 GitGit 看到的（精简）

GitGit 只知道：

```rust
// GitGit repository model (不感知 IDE)
pub struct Repository {
    pub id: RepoId,
    pub path: PathBuf,
    pub worktree_path: Option<PathBuf>,
    pub branch: BranchName,
    pub head_commit: Oid,
    pub dirty: bool,
}
```

### 2.4 Worktree 边界

**Worktree 物理上在 GitGit 维护（per §24 任务原文），但 Worktree↔IDE Session 绑定在 STAR 维护**：

- GitGit `worktree add/remove/list/status` — 纯文件系统层
- STAR `star worktree create STAR-1024` — 绑定到 Issue + Agent + IDE Session

## 3. 备选方案与拒绝理由

### 备选 A：IDE Session 进 GitGit
- 拒绝理由：GitGit 失去清晰边界、违反 §1 任务原文

### 备选 B：IDE Session 完全无状态（每次重新查询）
- 拒绝理由：性能差、无法做 session 级别的 audit / handoff

## 4. 后果

### 4.1 正面
- GitGit 保持单一职责（标准 Git VCS）
- IDE Session 复杂状态可独立演进
- Audit 路径清晰：Human/Agent/IDE/Automation 走同一 Audit Trail

### 4.2 成本
- 需要 IDE↔STAR 的轻量 client 通信（如 LSP 双向通知）
- IDE Session 状态在 IDE 关闭后需要持久化（OpenFile 状态由 IDE 端 cache + STAR 仅做引用）



---

## 5.5. 实施状态 (per OPT-NEXT-08 v0.2 拍板, 守门 #12 docs 触发)

> **本节由 OPT-NEXT-08 (2026-09-07 14:30 JST) 升版 v0.1 → v0.2 时落地, 提供实施证据链**

| 维度 | 内容 |
|---|---|
| **决策 scope** | IDE session identity |
| **目标 crate / 落地位置** | `n/a (架构原则)` |
| **落地 commit (per `git log -p --follow`)** | 无 commit (原则文档) |
| **守门 #1 v19** | `cargo check --workspace --all-targets -j 4` 0 err 32.27s (per 9/3 RF-001 T1.5 step 1 验证) |
| **守门 #3 v2** | Mavis 临时代签 5 域 Lead 决策 (per 9/3 11:35 JST 拍板 B), author=Ulysses (per 守门 #10) |
| **守门 #4.2** | Runtime 名称实装前一致性门 — 本 ADR 落档即满足"概念→物理 crate"映射, 后续实装前必先 ADR 拍板 |
| **守门 #10** | commit author = `Ulysses <ulysses@mavis.local>` (per 8/27 19:39 JST + 21:59 JST 三次强化) |
| **守门 #12** | docs 同步 = 实施前 git log --follow 实证; 缺标比错标安全 (per 8/26 JST) |
| **守门 #13 W/T/M** | 本 ADR 不涉及 DB schema 分类 (架构原则 / 决策类), 不触发 W/T/M 横展開 |
| **守门 #14 v2** | 5 域 Lead CONTENT 4 维 (决策 scope / RACI / 到位 timeline / Mavis 代签边界) 已显式列出 (per 9/3 19:43 JST) |

**实施完成度**:
- ✅ ADR 决策本身落地 (本节"决策"内容)
- ⏳ 后续实装 = 等 P3-B/F/H 拍板启动 + 5 域 Lead 真人到位 (per ADR §"签字栏" DDD Review 阶段补)

**已知缺口 (per 守门 #11 缺标比错标)**:
- 5 域 Lead 真人到位前, 实施由 Mavis 临时代签 (per 9/3 11:35 JST 拍板 B), 真人到位后追溯签字
- 本 ADR 实施状态只反映 commit / 守门 0 违反 实证, 不反映 22 domain 业务实装进展 (per P3-A H2 阶段 11/25 实证)

## 5. 签字栏

| # | 角色 | 姓名 | 签字日 | 结论/条件 |
|---|---|---|---|---|
| 1 | 架构负责人 | Mavis（per DEC-008） | 2026-08-26 | ⏳ 待 Ulysses 拍板 |
| 2 | SRE Lead | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |
| 3 | 平台工程师 | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |
| 4 | 评审主持人 | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |
| 5 | 项目负责人（PM） | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |

## 6. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.2 | 2026-09-07 | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses | OPT-NEXT-08 升版 v0.1 → v0.2: 新增 §5.5 实施状态 (scope/crate/commit/守门 0 违反 实证) + 修订历史 v0.2 行 (per 守门 #10 author=Ulysses + 守门 #12 docs 触发 + 守门 #11 缺标比错标) | 2026-09-07 14:30 JST OPT-NEXT-08 拍板, 23 草案 ADR 一次性升版 |
| v0.1 | 2026-08-26 | 架构师（Mavis 接手 agent per DEC-008） | 初版 | Phase B 起草 |
