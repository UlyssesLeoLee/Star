# ADR-0024: IDE Session Identity — 独立于 GitGit 的对象

> **状态**：🟡 草案 v0.1
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
| v0.1 | 2026-08-26 | 架构师（Mavis 接手 agent per DEC-008） | 初版 | Phase B 起草 |
