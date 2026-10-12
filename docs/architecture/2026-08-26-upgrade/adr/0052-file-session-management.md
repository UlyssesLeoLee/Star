# ADR-XXX: File-Based Session Management System (FSMS)
> **架构集成状态（2026-10-12）**：本文件包作为设计提案并入 `dev`，尚未通过产品架构裁决。其中“FileSession 直接隶属 Worktree、与 Task Card 并列”等层级陈述与当前规范（Project → Cloud Branch → Engineering Run → Worktree；Run 内 App/工作项按 RunContext 协作）不一致。本提案不得作为已批准的实现依据；FileSession 的归属、与 Run/Worktree/Task Card 的关系、存储及 CLI 边界须重新评审，并同步需求、基本设计、详细设计后才能排期。

> **Status**: 🟡 Proposed
> **Author**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **Created**: 2026-10-12
> **Supersedes**: 无 (新引入)
> **Related**: ADR-0049 (Task Card → Worktree + Agent), ADR-0045 (Agent Runtime), ADR-0024 (IDE Session Identity), ULYS-156 (CLI Session)
> **Source**: 用户需求: "在现有任务卡启动cli的模式以外，再设计一套以文件为单位管理session的管理体系，要求点击每个代码文件时可以开启一个session。管理这个文件库可以用git的结构，借助github/gitlab和其他ai agent的cli实现"

---

## §0 目的

设计并落地 **File-Based Session Management System (FSMS)**，在现有 Task Card CLI 模式 (`star task ...` / ADR-0049) 之外，提供：

1. **文件级会话粒度**：点击任意代码文件 → 秒级开启独立 AI agent session
2. **Git 原生持久化**：每文件 `.star/sessions/<path>/<session_id>/` 目录，Git 追踪完整历史
3. **多 Agent CLI 生态**：Claude Code / Codex / OpenCode / Hermes / 任意 ACP 兼容 CLI 统一适配
4. **跨设备/跨工作区同步**：Git push/pull + GitHub/GitLab MR 评审 session 变更
5. **与现有架构解耦但可互操作**：可选关联 `task_card_id` / `worktree_id` / `agent_session_id`

---

## §0.1 产品定位：Worktree 下的**双入口**设计

> **关键架构决策**：FileSession **隶属于 Worktree**，与 Task Card 处于**同级平行**关系，构成 Worktree 下的两大核心入口。

```
Worktree (git worktree)
├── Entry 1: Task Card 维度 (任务导向)
│   ├── 业务意图驱动
│   ├── 1 Task Card → 1 Worktree → 1 AgentSession (ADR-0049)
│   ├── 适合: 需求实装、Bug 修复、Feature 开发 (有明确任务边界)
│   └── 生命周期 = 任务卡生命周期
│
└── Entry 2: FileSession 维度 (文件/探索导向)  ← 本设计
    ├── 代码文件驱动
    ├── 1 File → N FileSessions (并行、独立)
    ├── 适合: 代码探索、重构实验、调试复现、文档生成、单测编写、Code Review 辅助
    └── 生命周期 = 文件编辑会话级 (可随时创建/销毁/归档)
```

**UI/UX 设计要求** (文档记录，不实装)：
- Worktree 详情页 / 文件树视图中，**并列展示** "Task Cards" 与 "File Sessions" 两个 Tab/Section
- 文件右键菜单 / CodeLens 直接入口 "Start File Session"
- 两个入口数据正交：Task Card 可选关联 FileSession (`linked_file_sessions`)，FileSession 可选关联 Task Card (`linked_task_card_id`)
- 权限模型复用 Worktree 级权限 (Owner/Editor/Viewer)

---

## §1 决策矩阵

| # | 决策点 | 选项 | 拍板 | 理由 |
|---|--------|------|------|------|
| 1 | **持久化层** | (a) Git 文件系统 (b) PostgreSQL (c) SQLite (d) 混合 | **(a) Git FS 为主，SQLite 为辅索引** | Git 是原生版本控制，天然支持分支/合并/远程同步/审计；SQLite 仅作 registry/manifest 加速查询 |
| 2 | **目录结构** | (a) 扁平 `<repo>/.star/sessions/<session_id>/` (b) 按文件分层 `<repo>/.star/sessions/<file-path>/<session_id>/` | **(b) 按文件分层** | 符合 "文件为单位" 语义；便于 CodeLens/TreeView 按文件聚合；Git diff 语义清晰 |
| 3 | **Agent 适配** | (a) 硬编码 3-4 个主流 CLI (b) 插件机制 + 内置主流 (c) 仅 ACP 协议 | **(b) 插件机制 + 内置主流** | 兼容开放生态；内置 Claude/Codex/OpenCode/Hermes 覆盖 95% 场景；WASM/动态库扩展自定义 |
| 4 | **上下文策略** | (a) 全量 transcript 存 Git (b) 增量 checkpoint + 可选 artifact store (c) 仅存摘要 | **(b) 增量 checkpoint + 可选 artifact store** | 平衡存储成本与恢复速度；大 transcript 走 Git LFS / S3；默认不进 Git 避免仓库膨胀 |
| 5 | **分支策略** | (a) 所有 session 共享主分支 (b) 每 session 独立 worktree 分支 (c) 仅 tag 标记 | **(b) 每 session 可选独立 worktree 分支** | 并行 session 隔离；`git worktree` 原生支持；可配置 `auto_worktree = true/false` |
| 6 | **与 Task Card 关系** | (a) 完全独立 (b) 强制关联 (c) 可选双向链接 | **(c) 可选双向链接** | 保持正交性；用户可从 Task Card 入口跳转 FileSession，反之亦然；不强制绑定 |
| 7 | **REST API** | (a) 复用 `/api/v1/agent-sessions` (b) 新建 `/api/v1/file-sessions` (c) GraphQL 统一 | **(b) 新建 `/api/v1/file-sessions`** | 资源模型差异大 (文件级 vs 任务级)；REST 语义清晰；便于权限/RLS 独立演进 |
| 8 | **前端入口** | (a) 仅 CLI (b) VS Code 扩展 (c) Web IDE (Multica) 集成 (d) 全部 | **(d) 全部** | CLI 为基础；VS Code 覆盖本地开发；Multica 覆盖云端协作；三端数据同步靠 Git |

---

## §2 架构概览

### 2.1 模块拓扑

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              star-cli (FSMS Entry)                           │
│  session {start,list,show,resume,checkpoint,archive,delete,fork,link,       │
│           sync,branch,export,mr, adapter, config}                            │
└────────────────────────────────────┬────────────────────────────────────────┘
                                     │
                                     ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                         FileSessionManager (Core Crate)                      │
│  ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐               │
│  │ SessionRegistry │ │ SessionFactory  │ │ ContextBuilder  │               │
│  │ (registry.json) │ │ (create/resume) │ │ (initial ctx)   │               │
│  └─────────────────┘ └─────────────────┘ └─────────────────┘               │
└────────────────────────────────────┬────────────────────────────────────────┘
                                     │
         ┌───────────────────────────┼───────────────────────────┐
         ▼                           ▼                           ▼
┌─────────────────────┐   ┌─────────────────────┐   ┌─────────────────────┐
│  AgentAdapter       │   │  GitManager         │   │  StorageBackend     │
│  (Plugin Registry)  │   │ (git2-rs / CLI)     │   │ (FS / SQLite / S3)  │
│                     │   │                     │   │                     │
│ • ClaudeCodeAdapter │   │ • auto_commit       │   │ • FileStorage       │
│ • CodexAdapter      │   │ • worktree_isolate  │   │ • SqliteIndex       │
│ • OpenCodeAdapter   │   │ • remote_sync       │   │ • S3ArtifactStore   │
│ • HermesAdapter     │   │ • mr_export         │   │ • PostgresArchive   │
│ • Custom (WASM)     │   │ • conflict_resolve  │   │                     │
└─────────────────────┘   └─────────────────────┘   └─────────────────────┘
```

### 2.2 核心数据流

```
用户点击文件 → star session start <file>
                    │
                    ▼
         FileSessionManager.create()
                    │
                    ├─→ SessionRegistry.alloc_id() → UUID
                    ├─→ ContextBuilder.build_initial(file_path)
                    │        ├─ 文件内容 + 符号索引 (treesitter-worktree-graph)
                    │        ├─ 依赖图 (star-context-tiering)
                    │        ├─ 最近 git log (--oneline -20)
                    │        └─ 隐私过滤 (star-credential rules)
                    ├─→ StorageBackend.create_workspace(session_id)
                    │        ├─ .star/sessions/<file>/<session_id>/
                    │        ├─ session.json (meta)
                    │        ├─ context/initial.json
                    │        └─ manifest.json 更新
                    ├─→ GitManager.init_session(session_id)
                    │        ├─ git add .star/sessions/<file>/<session_id>/
                    │        ├─ git commit "chore(session): <file> init <session_id>"
                    │        └─ (可选) git worktree add session/<hash>/<id>
                    └─→ AgentAdapter.spawn(SpawnContext)
                             ├─ 选择 adapter (--agent 或默认)
                             ├─ 注入初始上下文
                             └─ 返回 SessionHandle
```

---

## §3 详细设计

### 3.1 目录结构规范

```
<repo-root>/
├── .star/
│   ├── sessions/
│   │   ├── registry.json                    # 仓库级聚合索引 (SQLite 索引同步)
│   │   ├── config.toml                      # FSMS 配置
│   │   ├── crates/
│   │   │   └── star-cli/
│   │   │       └── src/
│   │   │           └── commands/
│   │   │               └── task.rs          # ← 目标文件
│   │   │                   ├── manifest.json # 该文件的 session 列表
│   │   │                   ├── a1b2c3d4/    # session_id 目录
│   │   │                   │   ├── session.json      # 会话元数据
│   │   │                   │   ├── context/
│   │   │                   │   │   ├── initial.json  # 启动时上下文
│   │   │                   │   │   └── checkpoints/
│   │   │                   │   │       ├── 001.json  # checkpoint 1
│   │   │                   │   │       └── 002.json
│   │   │                   │   ├── transcript/
│   │   │                   │   │   ├── index.json    # 消息索引 (offset/len/type)
│   │   │                   │   │   ├── 0001-0100.msgpack  # 分块存储
│   │   │                   │   │   └── 0101-0200.msgpack
│   │   │                   │   ├── artifacts/
│   │   │                   │   │   ├── patches/
│   │   │                   │   │   ├── diffs/
│   │   │                   │   │   └── exports/
│   │   │                   │   ├── git/
│   │   │                   │   │   ├── branch        # 关联分支名
│   │   │                   │   │   ├── commits.json  # 关联提交
│   │   │                   │   │   └── mr-links.json # MR/PR 链接
│   │   │                   │   └── state/
│   │   │                   │       ├── adapter.json  # agent 状态快照
│   │   │                   │       └── checkpoints/  # agent 自身 checkpoint
│   │   │                   └── e5f6g7h8/            # 另一个 session
│   │   └── ...
│   └── config.toml                      # FSMS 全局配置
```

### 3.2 核心 Schema

#### registry.json (仓库级)
```json
{
  "version": 1,
  "updated_at": "2026-10-12T10:30:00Z",
  "sessions": [
    {
      "session_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
      "file_path": "crates/star-cli/src/commands/task.rs",
      "agent_type": "claude",
      "state": "active",
      "created_at": "2026-10-12T10:00:00Z",
      "updated_at": "2026-10-12T10:25:00Z",
      "tags": ["refactor", "cli"],
      "linked_task_card_id": "01a0bf6b-486c-71b8-9bf4-92bde909c7f8",
      "linked_worktree_id": "wt-sub-session-001",
      "linked_agent_session_id": "agt-12345678"
    }
  ]
}
```

#### manifest.json (单文件)
```json
{
  "file_path": "crates/star-cli/src/commands/task.rs",
  "version": 1,
  "updated_at": "2026-10-12T10:30:00Z",
  "sessions": [
    {
      "session_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
      "agent_type": "claude",
      "state": "active",
      "title": "重构 task.rs: 提取 CurrentArgs 解析逻辑",
      "created_at": "2026-10-12T10:00:00Z",
      "updated_at": "2026-10-12T10:25:00Z",
      "checkpoint_count": 3,
      "token_usage": { "input": 12500, "output": 8200, "total": 20700 },
      "git_branch": "session/crates-star-cli-src-commands-task-rs/a1b2c3d4"
    }
  ]
}
```

#### session.json (会话详情)
```json
{
  "meta": {
    "session_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
    "file_path": "crates/star-cli/src/commands/task.rs",
    "agent_type": "claude",
    "agent_version": "1.0.45",
    "adapter_config": { "model": "claude-3-5-sonnet", "max_tokens": 8192 },
    "state": "active",
    "created_at": "2026-10-12T10:00:00Z",
    "updated_at": "2026-10-12T10:25:00Z",
    "tags": ["refactor", "cli"],
    "links": {
      "task_card_id": "01a0bf6b-486c-71b8-9bf4-92bde909c7f8",
      "worktree_id": "wt-sub-session-001",
      "agent_session_id": "agt-12345678",
      "parent_session_id": null,
      "child_session_ids": [],
      "related_sessions": []
    }
  },
  "context": {
    "initial_ref": "context/initial.json",
    "checkpoints": [
      { "num": 1, "ref": "context/checkpoints/001.json", "token_delta": 2100, "at": "2026-10-12T10:05:00Z" },
      { "num": 2, "ref": "context/checkpoints/002.json", "token_delta": 1800, "at": "2026-10-12T10:15:00Z" },
      { "num": 3, "ref": "context/checkpoints/003.json", "token_delta": 3200, "at": "2026-10-12T10:25:00Z" }
    ],
    "current_checkpoint": 3,
    "token_budget": 50000,
    "token_used": 20700
  },
  "transcript": {
    "index_ref": "transcript/index.json",
    "chunks": [
      { "file": "transcript/0001-0100.msgpack", "range": [1, 100] },
      { "file": "transcript/0101-0200.msgpack", "range": [101, 150] }
    ],
    "total_messages": 150
  },
  "artifacts": {
    "patches": ["artifacts/patches/001-refactor-extract-args.patch"],
    "diffs": ["artifacts/diffs/001-vs-002.diff"],
    "exports": []
  },
  "git": {
    "branch": "session/crates-star-cli-src-commands-task-rs/a1b2c3d4",
    "worktree_path": "../.worktrees/wt-session-a1b2c3d4",
    "commits": [
      { "sha": "abc123", "message": "chore(session): task.rs init a1b2c3d4", "at": "2026-10-12T10:00:00Z" },
      { "sha": "def456", "message": "chore(session): task.rs checkpoint 1", "at": "2026-10-12T10:05:00Z" }
    ],
    "mr_links": []
  },
  "state_snapshots": [
    { "checkpoint": 1, "ref": "state/checkpoints/001.json", "at": "2026-10-12T10:05:00Z" },
    { "checkpoint": 2, "ref": "state/checkpoints/002.json", "at": "2026-10-12T10:15:00Z" },
    { "checkpoint": 3, "ref": "state/checkpoints/003.json", "at": "2026-10-12T10:25:00Z" }
  ]
}
```

### 3.3 状态机

```
Created (session.json 写入)
    │
    ├─ spawn() ──→ Running (AgentAdapter 启动成功)
    │
    └─ cancel() ──→ Cancelled
Running
    │
    ├─ pause() ──→ Paused
    │
    ├─ checkpoint() ──→ Running (内部自环，写 checkpoint + 可选 git commit)
    │
    ├─ cancel() ──→ Cancelling ──→ Cancelled
    │
    └─ (agent 正常退出) ──→ Completed
Paused
    │
    ├─ resume() ──→ Running
    │
    └─ cancel() ──→ Cancelling ──→ Cancelled
Completed / Cancelled
    │
    └─ archive() ──→ Archived (终态，只读)
Archived
    └─ (不可迁出)
```

> 复用 `CliSessionState` 设计模式，简化为 6 态 (无 Restart/Orphaned —— FileSession 无 parent runtime 重启问题)

### 3.4 AgentAdapter 接口契约

```rust
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentType {
    ClaudeCode,
    Codex,
    OpenCode,
    Hermes,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnContext {
    pub session_id: Uuid,
    pub file_path: String,
    pub repo_root: String,
    pub initial_context: InitialContext,
    pub adapter_config: AdapterConfig,
    pub working_dir: String,          // session workspace 或 worktree path
    pub env: Vec<(String, String)>,   // 环境变量 (含 credentials)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitialContext {
    pub file_content: String,
    pub symbols: Vec<SymbolInfo>,
    pub dependencies: Vec<DependencyEdge>,
    pub git_log: Vec<GitLogEntry>,
    pub token_budget: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionHandle {
    pub process_id: u32,
    pub control_channel: ControlChannel,  // stdin/stdout/pty/WebSocket
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub number: usize,
    pub context_delta: ContextDelta,
    pub transcript_range: (usize, usize),
    pub artifacts: Vec<ArtifactRef>,
    pub agent_state: AgentStateSnapshot,
    pub created_at: DateTime<Utc>,
}

#[async_trait]
pub trait AgentAdapter: Send + Sync {
    fn agent_type(&self) -> AgentType;
    fn version(&self) -> semver::Version;
    fn supported_models(&self) -> Vec<String>;
    fn capabilities(&self) -> AdapterCapabilities;

    /// 启动新会话
    async fn spawn(&self, ctx: SpawnContext) -> Result<SessionHandle, AdapterError>;

    /// 恢复现有会话 (attach 到运行中进程或重新 spawn --continue)
    async fn attach(&self, handle: SessionHandle) -> Result<AttachResult, AdapterError>;

    /// 创建检查点 (触发 agent 内部 checkpoint + 导出状态)
    async fn checkpoint(&self, handle: &SessionHandle) -> Result<Checkpoint, AdapterError>;

    /// 从检查点恢复
    async fn resume(&self, handle: SessionHandle, checkpoint: Checkpoint) -> Result<SessionHandle, AdapterError>;

    /// 优雅关闭
    async fn shutdown(&self, handle: SessionHandle, graceful: bool) -> Result<ShutdownResult, AdapterError>;

    /// 导出会话 (用于 MR/分享/归档)
    async fn export(&self, handle: &SessionHandle, format: ExportFormat) -> Result<Vec<u8>, AdapterError>;

    /// 健康检查
    async fn health_check(&self) -> Result<HealthStatus, AdapterError>;
}
```

#### 内置适配器实现策略

| Adapter | 启动命令模式 | 恢复模式 | Checkpoint 支持 | 状态导出 |
|---------|-------------|----------|-----------------|----------|
| **Claude Code** | `claude --print "<prompt>"` | `claude --continue --resume <session_id>` | `--checkpoint` (v1.0.40+) | JSON 导出 |
| **Codex** | `codex exec "<prompt>"` | `codex exec --continue <session_id>` | 内置 session 文件 | `--export json` |
| **OpenCode** | `opencode run "<prompt>"` | `opencode run --continue <session_id>` | 插件机制 | 插件导出 |
| **Hermes** | `hermes acp --session <id>` | `hermes acp --attach <id>` | ACP 原生 | ACP 协议 |

> 所有适配器统一通过 `AgentAdapter` trait 暴露，CLI 层无感知具体实现

---

## §4 Git 集成设计

### 4.1 自动提交策略

```toml
# .star/config.toml
[git]
auto_commit = true                    # 每次 checkpoint 自动 commit
commit_message_template = "chore(session): {file} checkpoint {num} [{session_id:8}]"
sign_commits = false                  # 可选 GPG 签名
worktree_isolation = true             # 每 session 独立 worktree
worktree_base = ".worktrees"          # worktree 存放目录
remote_sync_interval = 300            # 秒，自动 push/pull
```

**Commit 示例**：
```bash
# 初始化
git add .star/sessions/crates/star-cli/src/commands/task.rs/a1b2c3d4/
git commit -m "chore(session): crates/star-cli/src/commands/task.rs init a1b2c3d4"

# Checkpoint 1
git add .star/sessions/crates/star-cli/src/commands/task.rs/a1b2c3d4/context/checkpoints/001.json
git add .star/sessions/crates/star-cli/src/commands/task.rs/a1b2c3d4/transcript/0001-0100.msgpack
git commit -m "chore(session): crates/star-cli/src/commands/task.rs checkpoint 1 [a1b2c3d4]"

# 归档
git commit -m "chore(session): crates/star-cli/src/commands/task.rs archive a1b2c3d4"
```

### 4.2 Worktree 隔离

```bash
# 启用 worktree 隔离时，每个 session 获得独立工作目录
git worktree add .worktrees/wt-session-a1b2c3d4 -b session/crates-star-cli-src-commands-task-rs/a1b2c3d4

# Session 内所有文件操作发生在 worktree 中
# 结束时可选：merge 回主分支、或保留 worktree 供后续 resume
```

### 4.3 远程同步

```bash
star session sync [--remote origin] [--push] [--pull] [--dry-run]
```

- **Push**: 将本地 `.star/sessions/` 新增 commit 推送到 remote
- **Pull**: 获取远程新增 session，合并 registry/manifest
- **冲突**: 同一文件并发 session → 基于语义 diff 自动合并，失败回退手工

### 4.4 MR 导出

```bash
star session export a1b2c3d4 --format mr --output /tmp/mr.md
```

生成的 MR 描述包含：
- Session 标题 + 标签 + 时长 + token 使用
- 关键决策摘要 (LLM 生成)
- 产出 patch/diff 列表
- 完整 transcript 链接 (可选)

---

## §5 与现有系统集成

### 5.1 Task Card (ADR-0049) 双向链接

```
Task Card (in_progress)
    │ linked_file_sessions: [session_id_1, session_id_2]
    ▼
FileSession A (探索代码结构)
    │ linked_task_card_id: task_card_id
    ▼
FileSession B (实装重构)
    │ linked_task_card_id: task_card_id
    ▼
Task Card 进度自动更新 (基于 session 产出)
```

### 5.2 AgentSession (ULYS-156 / ADR-0045) 升级路径

```
FileSession (轻量、文件级、Git 本地)
    │ 用户决定需要持久化到平台、跨设备、团队协作
    ▼
star session promote <session_id> --to agent-session
    │
    ├─ 创建 ADR-0049 AgentSession (PG 持久化)
    ├─ 迁移 transcript + context + artifacts
    ├─ 绑定 worktree + agent_session_id
    └─ FileSession 标记 promoted, 保留只读副本
```

### 5.3 复用现有基础设施

| 基础设施 | 复用方式 |
|----------|----------|
| `star-context-tiering` | ContextBuilder 初始上下文构建 |
| `treesitter-worktree-graph` | 符号索引 + 依赖图 |
| `star-credential` | 隐私过滤规则 + 凭证注入 |
| `domain-hook` | Session 生命周期事件触发 hook |
| `domain-audit` | 审计日志统一入库 |
| `permission` | **权限模型继承 Worktree (Owner/Editor/Viewer)** |
| `star-taskgraph` | **Worktree 归属 + 分支管理** (FileSession 隶属 Worktree，与 Task Card 同级) |

---

## §6 实装里程碑

| 阶段 | 周期 | 交付物 | 关键验收 |
|------|------|--------|----------|
| **Phase 1: 核心基建** | 2w | `star-file-session` crate + CLI 骨架 + FS/SQLite 存储 + GitManager 基础 | `star session start/list/show/resume` 端到端跑通 |
| **Phase 2: 适配器框架** | 2w | 插件注册表 + Claude/Codex/OpenCode/Hermes 4 个内置适配器 | 4 个 CLI 均能 spawn/attach/checkpoint |
| **Phase 3: 上下文/检查点** | 2w | 增量 checkpoint + transcript 索引 + token 预算 + 隐私过滤 | 恢复 token < 初始 30%，过滤 100% secrets |
| **Phase 4: Git 深度集成** | 1.5w | 自动提交 + worktree 隔离 + 远程同步 + MR 导出 | GitHub/GitLab 双向同步，MR 描述自动生成 |
| **Phase 5: API + 前端** | 2w | REST API + VS Code 扩展 + Multica 集成 | CodeLens + Session Panel + Web IDE 点击文件开启 |
| **Phase 6: 高级特性** | 持续 | 实时协作 + 语义合并 + 模板 + 企业审计 | P2/P3 按需交付 |

---

## §7 非功能指标

| 指标 | 目标 | 测量方式 |
|------|------|----------|
| 会话创建冷启动延迟 | P95 < 3s | 基准测试 (100 次) |
| 会话恢复延迟 | P95 < 2s | 基准测试 (100 次) |
| Checkpoint 写入延迟 | P99 < 500ms | 压测 (并发 50) |
| 单会话磁盘占用 | 典型 < 50 MB | 真实场景采样 |
| Registry 体积 | 10k sessions < 1 MB | 容量规划测试 |
| 并发会话数 | 单 repo ≥ 1000 | 压测 |
| 静态加密 | AES-256-GCM 可选启用 | 安全审计 |
| 审计不可篡改 | Append-only + 签名链 | 合规检查 |

---

## §8 风险与缓解

| 风险 | 影响 | 可能性 | 缓解 |
|------|------|--------|------|
| Git 仓库膨胀 | Clone 慢、磁盘满 | 高 | 1) Transcript 默认不进 Git (本地 + 可选 S3) 2) Git LFS 大文件 3) 定期 `git gc` + 浅克隆 |
| 并发编辑冲突 | 数据损坏 | 中 | 1) 乐观锁 (session 级) 2) 语义合并 (tree-sitter AST) 3) UI 冲突可视化 |
| Agent CLI 版本不兼容 | 无法启动 | 中 | 1) 版本检测矩阵 2) 降级基础模式 3) 清晰错误码 |
| 敏感信息泄露 | 安全事件 | 高 | 1) 默认开启隐私过滤 2) Transcript 不进 Git 3) 导出前二次确认 |
| 存储后端切换不一致 | 数据丢失 | 低 | 1) 统一 StorageBackend trait 2) 迁移工具 checksum 3. 集成测试覆盖 |

---

## §9 回滚策略

- **Git 历史即回滚日志**：`git checkout <commit> -- .star/sessions/...`
- **CLI 命令**：`star session rollback <session_id> --to-checkpoint <n>`
- **数据迁移**：`star session migrate --from-version <v> --to-version <v+1>`

---

## §10 签字栏

| 角色 | 审批者 | 日期 | 备注 |
|------|--------|------|------|
| 架构师 | | | |
| SRE Lead | | | |
| 平台工程师 | | | |
| 评审主持人 | | | |
| 项目负责人 (PM) | | | |

---

## §11 修订历史

| 版本 | 日期 | 作者 | 变更 |
|------|------|------|------|
| v0.1 | 2026-10-12 | Ulysses (Mavis 接手) | 初版建立 |