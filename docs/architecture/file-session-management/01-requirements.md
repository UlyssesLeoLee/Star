# File-Based Session Management System (FSMS)
> **架构集成状态（2026-10-12）**：本文件包作为设计提案并入 `dev`，尚未通过产品架构裁决。其中“FileSession 直接隶属 Worktree、与 Task Card 并列”等层级陈述与当前规范（Project → Cloud Branch → Engineering Run → Worktree；Run 内 App/工作项按 RunContext 协作）不一致。本提案不得作为已批准的实现依据；FileSession 的归属、与 Run/Worktree/Task Card 的关系、存储及 CLI 边界须重新评审，并同步需求、基本设计、详细设计后才能排期。

> **Status**: Draft v0.1
> **Author**: Ulysses (per DEC-008) — Mavis 接手
> **Created**: 2026-10-12
> **Related**: ADR-0049 (Task Card → Worktree + Agent), ADR-0045 (Agent Runtime), ULYS-156 (CLI Session), ADR-0024 (IDE Session Identity)

---

## §0 背景与动机

### 0.1 现状痛点

| 现有模式 | 局限 |
|----------|------|
| **Task Card CLI** (`star task ...`) | 仅绑定 *一个* task_card_id，session 生命周期 = 任务卡生命周期，粒度过粗 |
| **ADR-0049 Auto Worktree** | 1 Task Card → 1 Worktree → 1 AgentSession，但无法针对**单个文件**发起探索/重构/调试会话 |
| **ADR-0024 IDE Session** | 仅记录 IDE 内部状态，不承载 AI agent 业务会话 |
| **CLI Session (ULYS-156)** | 7 态状态机 + SQLite 持久化，但 session 标识 = UUID，缺乏文件级归属 |

> **核心缺口**：用户点击任意代码文件 → 期望**即刻开启**一个独立的 AI agent session（探索、重构、调试、文档生成等），且该 session 具备完整的版本历史、审计、恢复、协作能力。

### 0.2 目标

> **设计一套以文件为单位的 session 管理体系**，在现有 Task Card CLI 模式之外，实现：
- **点击文件 → 开启 session**（零配置、秒级启动）
- **Git 结构化存储**（每文件一个 `.star/sessions/<relative-path>/` 目录，Git 追踪历史）
- **多 AI Agent CLI 集成**（Claude Code / Codex / OpenCode / Hermes / 任意兼容 ACP 的 CLI）
- **跨设备/跨工作区同步**（Git push/pull、GitHub/GitLab MR 评审 session 变更）
- **与现有架构解耦但可互操作**（可选关联 task_card / worktree / agent_session）

### 0.3 产品定位：Worktree 下的**双入口**设计

> **关键设计决策**：FileSession **隶属于 Worktree**，与 Task Card 处于**同级平行**关系，构成 Worktree 下的两大核心入口。

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

### 0.4 动态交互体验标准 (参考 sakura 级水准)

> **基准**：sakura 终端/编辑器菜单交互 —— 菜单打开后移动鼠标，子菜单/上下文菜单能**跟随鼠标位置即时打开**，零感知延迟，无闪烁、无抖动。

| 交互场景 | 行为规范 | 性能指标 |
|----------|----------|----------|
| **文件树右键菜单** | 右键点击文件节点 → 100ms 内弹出上下文菜单，菜单项含 "Start File Session"、"Open Sessions"、"Reveal in Explorer" 等；鼠标移动到菜单项 → 子菜单即时展开 (≤ 50ms) | 首帧渲染 < 100ms；子菜单展开 < 50ms |
| **CodeLens 悬停** | 编辑器标题栏/行内 CodeLens 显示 "🤖 N sessions"；鼠标悬停 → Tooltip 即时展开 (≤ 80ms)，显示会话列表预览 (标题、状态、agent、更新时间)；点击列表项 → 直接跳转/恢复 | Tooltip 首显 < 80ms；列表渲染 < 120ms |
| **Session 列表行悬停** | Session Panel / Worktree 页列表行鼠标悬停 → 行背景高亮 + 操作按钮显示 (Resume/Checkpoint/Archive/Delete)；无布局抖动 | 悬停响应 < 30ms |
| **多级菜单导航** | 一级菜单 → 二级菜单 → 三级菜单；鼠标斜向移动进入子菜单区域时，**不关闭父菜单**，使用「安全三角区」判定 (参考 Amazon/Stripe 菜单算法) | 子菜单展开 < 50ms；误触关闭率 < 0.1% |
| **键盘/鼠标混合操作** | 键盘 ↑↓ 选中菜单项时，鼠标位置不干扰焦点；Esc 关闭所有层级菜单；焦点陷阱正确 | 焦点切换 < 16ms |
| **触发区域容错** | 菜单边缘 8px 内移动不触发关闭；鼠标快速穿越菜单间隙不误关闭 | 容错半径 ≥ 8px |
| **动画缓动** | 菜单展开/关闭使用 `ease-out-cubic` 120ms；Tooltip `ease-out-quad` 80ms；无抖动、无掉帧 | 60fps 全程；无 layout shift |
| **虚拟化长列表** | Session 列表 > 100 项时自动虚拟化渲染；搜索/过滤即时响应 (< 100ms) | 滚动 60fps；筛选 < 100ms |

**实装技术要求** (供前端参考)：
- 使用 **Floating UI / Popper.js** 定位引擎 (自动翻转、防溢出、跟随锚点)
- 菜单状态机：`closed → opening → open → closing → closed`，配合 `requestAnimationFrame` 批量 DOM 操作
- 事件委托 + 被动事件监听器 (`passive: true`) 减少主线程阻塞
- CSS `contain: layout paint` 隔离菜单重绘影响范围
- 避免 `position: fixed` 导致的滚动容器定位错误，改用 `position: absolute` + 容器相对定位

**验收测试用例** (自动化 + 手工)：
1. 连续快速在文件树 20 个节点间右键 → 每次菜单正确定位、无残留
2. 鼠标从一级菜单斜向移动到三级菜单 → 父菜单保持打开、子菜单跟手
3. CodeLens 悬停 100 次 → 平均首显 < 80ms，P99 < 120ms
4. 窗口 resize / 滚动时菜单自动重定位或优雅关闭
5. 键盘 Tab 遍历菜单项 → 焦点环可见、屏幕阅读器可读

---

---

## §1 核心概念

| 概念 | 定义 | 关键属性 |
|------|------|----------|
| **FileSession** | 绑定单个源码文件的 AI agent 会话 | `file_path` (relative to repo root) + `session_id` (UUID) + `agent_type` + `created_at` |
| **Session Workspace** | 文件级会话的持久化根目录 | `<repo-root>/.star/sessions/<relative-file-path>/<session_id>/` |
| **Session Manifest** | 单文件所有 session 的索引 | `<repo-root>/.star/sessions/<relative-file-path>/manifest.json` |
| **Session Registry** | 仓库级会话聚合视图 | `<repo-root>/.star/sessions/registry.json` (Git-tracked) |
| **Agent Adapter** | 抽象 AI CLI 交互的统一接口 | `spawn()`, `attach()`, `checkpoint()`, `resume()`, `export()` |

### 1.1 目录结构约定

```
<repo-root>/
├── .star/
│   ├── sessions/
│   │   ├── registry.json                    # 仓库级聚合索引
│   │   ├── crates/
│   │   │   └── star-cli/
│   │   │       └── src/
│   │   │           └── commands/
│   │   │               └── task.rs          # ← 目标文件
│   │   │                   ├── manifest.json # 该文件的 session 列表
│   │   │                   ├── a1b2c3d4/    # session_id 目录
│   │   │                   │   ├── session.json      # 会话元数据
│   │   │                   │   ├── context/          # 上下文包 (context packets)
│   │   │                   │   │   ├── initial.json  # 启动时上下文
│   │   │                   │   │   └── checkpoints/  # 增量检查点
│   │   │                   │   ├── transcript/       # 完整对话记录
│   │   │                   │   │   ├── 001-user.json
│   │   │                   │   │   ├── 002-assistant.json
│   │   │                   │   │   └── ...
│   │   │                   │   ├── artifacts/        # 产出物
│   │   │                   │   │   ├── patches/      # 生成的 patch 文件
│   │   │                   │   │   ├── diffs/        # diff 输出
│   │   │                   │   │   └── exports/      # 导出格式
│   │   │                   │   ├── git/              # Git 集成数据
│   │   │                   │   │   ├── branch        # 关联分支名
│   │   │                   │   │   ├── commits.json  # 关联提交记录
│   │   │                   │   │   └── mr-links.json # MR/PR 链接
│   │   │                   │   └── state/            # Agent 状态快照
│   │   │                   └── e5f6g7h8/            # 另一个 session
│   │   └── ...
│   └── config.toml                      # FSMS 全局配置
```

---

## §2 功能需求

### 2.1 会话生命周期 (FR-LIFECYCLE)

| ID | 需求 | 优先级 | 验收标准 |
|----|------|--------|----------|
| FR-LIFECYCLE-01 | **创建**: 右键文件/命令行 `star session start <file>` → 新建 FileSession | P0 | 3 秒内返回目录结构 + session_id |
| FR-LIFECYCLE-02 | **列出**: `star session list [--file <path>] [--agent <type>]` | P0 | 显示 manifest + registry 聚合视图 |
| FR-LIFECYCLE-03 | **恢复**: `star session resume <session_id>` → 加载上下文 + 继续对话 | P0 | 完整恢复 transcript + context + agent 状态 |
| FR-LIFECYCLE-04 | **检查点**: 自动/手动 `checkpoint` → 增量持久化上下文 | P1 | 恢复时 token 使用量 < 初始上下文 30% |
| FR-LIFECYCLE-05 | **归档**: `archive` → 标记只读、压缩 transcript | P2 | 归档后不可写、占用空间 < 50% |
| FR-LIFECYCLE-06 | **删除**: `delete` → 物理删除 + registry 更新 | P2 | 支持软删除 (移入 `.trash/`) |

### 2.2 多 Agent CLI 适配 (FR-ADAPTER)

| ID | 需求 | 优先级 | 备注 |
|----|------|--------|------|
| FR-ADAPTER-01 | **Claude Code** (`claude`) 适配器 | P0 | 支持 `--continue` / `--resume` / `--print` 模式 |
| FR-ADAPTER-02 | **OpenAI Codex** (`codex`) 适配器 | P0 | 支持 `exec` / `apply-patch` 模式 |
| FR-ADAPTER-03 | **OpenCode** (`opencode`) 适配器 | P1 | 支持 `--plugin` 扩展 |
| FR-ADAPTER-04 | **Hermes Agent** (`hermes`) 适配器 | P1 | 复用现有 ACP/ACL 基础设施 |
| FR-ADAPTER-05 | **通用 ACP 协议** 适配器 | P2 | 兼容任意实现 ACP 的 CLI |
| FR-ADAPTER-06 | **自定义 Agent** 插件机制 | P2 | WASM/动态库加载，`star session adapter add <path>` |

> **适配器接口契约** (Trait `AgentAdapter`):
```rust
#[async_trait]
pub trait AgentAdapter: Send + Sync {
    fn agent_type(&self) -> AgentType;           // Claude / Codex / OpenCode / Hermes / Custom
    fn version(&self) -> semver::Version;
    async fn spawn(&self, ctx: SpawnContext) -> Result<SessionHandle>;
    async fn attach(&self, handle: SessionHandle) -> Result<AttachResult>;
    async fn checkpoint(&self, handle: SessionHandle) -> Result<Checkpoint>;
    async fn resume(&self, handle: SessionHandle, checkpoint: Checkpoint) -> Result<SessionHandle>;
    async fn export(&self, handle: SessionHandle, format: ExportFormat) -> Result<Vec<u8>>;
}
```

### 2.3 Git 集成 (FR-GIT)

| ID | 需求 | 优先级 | 说明 |
|----|------|--------|------|
| FR-GIT-01 | **自动提交**: 每次 checkpoint 自动 `git add + commit` (message = `chore(session): <file> checkpoint <n>`) | P0 | 可配置 `auto_commit = true/false` |
| FR-GIT-02 | **分支策略**: 每 session 可选绑定 `session/<file-hash>/<session-id>` 分支 | P1 | `git worktree` 隔离并行 session |
| FR-GIT-03 | **远程同步**: `star session sync` → push/pull 所有 session 历史 | P1 | 支持 GitHub/GitLab/Gitea |
| FR-GIT-04 | **MR 评审**: 导出 session 为 MR 描述/评论，支持 AI 互评 | P2 | `star session export --mr <session_id>` |
| FR-GIT-05 | **冲突解决**: 并发编辑同一文件的多 session 合并策略 | P2 | 基于语义 diff + LLM 辅助合并 |

### 2.4 上下文管理 (FR-CONTEXT)

| ID | 需求 | 优先级 | 说明 |
|----|------|--------|------|
| FR-CONTEXT-01 | **初始上下文**: 文件内容 + 符号索引 + 依赖图 + 最近 git log | P0 | 复用 `star-context-tiering` + `treesitter-worktree-graph` |
| FR-CONTEXT-02 | **增量上下文**: checkpoint 仅存增量 (用户输入 + agent 输出 + 文件变更) | P0 | 压缩率 > 80% |
| FR-CONTEXT-03 | **跨文件引用**: `@import` 语法引用同 repo 其他文件的 session 上下文 | P1 | `star session link <source> <target>` |
| FR-CONTEXT-04 | **Token 预算**: 可配置 max_tokens，超限自动摘要/截断 | P1 | 复用 `ContextPacket.token_budget` |
| FR-CONTEXT-05 | **隐私过滤**: 自动脱敏 secrets / PII / 内部路径 | P1 | 复用 `star-credential` 规则引擎 |

### 2.5 协作与审计 (FR-COLLAB)

| ID | 需求 | 优先级 | 说明 |
|----|------|--------|------|
| FR-COLLAB-01 | **多用户会话**: 同一 FileSession 多人协作 (OT/CRDT) | P2 | 实时光标、冲突高亮 |
| FR-COLLAB-02 | **审计日志**: 完整记录谁/何时/做了什么 (复用 `ai-audit` schema) | P1 | 符合 ADR-0043 审计要求 |
| FR-COLLAB-03 | **权限模型**: Owner/Editor/Viewer，基于 Git 仓库权限继承 | P1 | 复用 `permission` 域 |
| FR-COLLAB-04 | **会话分叉**: `fork <session_id>` → 新 session 继承历史 | P2 | 类似 Git fork，保留共同祖先 |

---

## §3 非功能需求

| 类别 | 指标 | 目标 |
|------|------|------|
| **性能** | 会话创建延迟 (冷启动) | < 3s (P95) |
| **性能** | 会话恢复延迟 (含上下文加载) | < 2s (P95) |
| **性能** | checkpoint 写入延迟 | < 500ms (P99) |
| **存储** | 单 session 磁盘占用 (含 transcript) | < 50 MB (典型) |
| **存储** | 仓库级 registry 体积 | < 1 MB / 10k sessions |
| **可靠性** | 进程崩溃后数据零丢失 | WAL + fsync 保证 |
| **可靠性** | 并发 session 数 (单 repo) | ≥ 1000 |
| **兼容性** | 支持的 Git 托管平台 | GitHub, GitLab, Gitea, Bitbucket, 任意 Git remote |
| **兼容性** | 支持的 Shell | bash, zsh, fish, PowerShell, cmd.exe |
| **安全** | 静态加密敏感字段 | AES-256-GCM (可选启用) |
| **安全** | 审计日志不可篡改 | Append-only + 签名链 |

---

## §4 架构设计

### 4.1 模块拓扑

```
┌─────────────────────────────────────────────────────────────────┐
│                        star-cli (FSMS Entry)                     │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐              │
│  │ session     │  │ session     │  │ session     │   ...        │
│  │ start       │  │ list        │  │ resume      │              │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘              │
└─────────┼────────────────┼────────────────┼─────────────────────┘
          │                │                │
          ▼                ▼                ▼
┌─────────────────────────────────────────────────────────────────┐
│                     FileSessionManager (Core)                    │
│  ┌──────────────────┐  ┌──────────────────┐  ┌────────────────┐ │
│  │ SessionRegistry  │  │ SessionFactory   │  │ ContextBuilder │ │
│  │ (registry.json)  │  │ (create/resume)  │  │ (initial ctx)  │ │
│  └──────────────────┘  └──────────────────┘  └────────────────┘ │
└────────────────────────────────┬────────────────────────────────┘
                                 │
         ┌───────────────────────┼───────────────────────┐
         ▼                       ▼                       ▼
┌─────────────────┐   ┌─────────────────┐   ┌─────────────────┐
│  AgentAdapter   │   │  GitManager     │   │  StorageBackend │
│  (Plugin Registry)│  │ (git2-rs / CLI) │   │ (FS / SQLite /  │
│                 │   │                 │   │  S3 / Postgres) │
└─────────────────┘   └─────────────────┘   └─────────────────┘
```

### 4.2 核心数据模型

```rust
// 仓库级注册表
#[derive(Serialize, Deserialize)]
pub struct SessionRegistry {
    pub version: u32,                          // schema version
    pub updated_at: DateTime<Utc>,
    pub sessions: Vec<SessionRef>,             // 扁平索引，便于全局查询
}

#[derive(Serialize, Deserialize)]
pub struct SessionRef {
    pub session_id: Uuid,
    pub file_path: String,                     // relative to repo root
    pub agent_type: AgentType,
    pub state: SessionState,                   // Active / Paused / Archived / Deleted
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<String>,                     // 用户自定义标签
    pub linked_task_card_id: Option<Uuid>,     // 可选关联 task card
    pub linked_worktree_id: Option<Uuid>,      // 可选关联 worktree
    pub linked_agent_session_id: Option<Uuid>, // 可选关联 ADR-0049 AgentSession
}

// 单文件 manifest
#[derive(Serialize, Deserialize)]
pub struct FileSessionManifest {
    pub file_path: String,
    pub version: u32,
    pub updated_at: DateTime<Utc>,
    pub sessions: Vec<SessionSummary>,         // 仅摘要，详情在 session 目录
}

#[derive(Serialize, Deserialize)]
pub struct SessionSummary {
    pub session_id: Uuid,
    pub agent_type: AgentType,
    pub state: SessionState,
    pub title: String,                         // 用户给的标题或首轮意图摘要
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub checkpoint_count: usize,
    pub token_usage: TokenUsage,
    pub git_branch: Option<String>,
}

// 会话详情
#[derive(Serialize, Deserialize)]
pub struct FileSession {
    pub meta: SessionMeta,
    pub context: SessionContext,
    pub transcript: TranscriptIndex,           // 不内联存完整消息，存索引
    pub artifacts: ArtifactIndex,
    pub git: GitIntegrationData,
    pub state_snapshots: Vec<StateSnapshot>,   // agent 状态快照
}

#[derive(Serialize, Deserialize)]
pub struct SessionMeta {
    pub session_id: Uuid,
    pub file_path: String,
    pub agent_type: AgentType,
    pub agent_version: String,
    pub adapter_config: AdapterConfig,         // agent 特定配置
    pub state: SessionState,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<String>,
    pub links: SessionLinks,
}

#[derive(Serialize, Deserialize)]
pub struct SessionLinks {
    pub task_card_id: Option<Uuid>,
    pub worktree_id: Option<Uuid>,
    pub agent_session_id: Option<Uuid>,        // ADR-0049 agent_session
    pub parent_session_id: Option<Uuid>,       // fork 来源
    pub child_session_ids: Vec<Uuid>,          // fork 衍生
    pub related_sessions: Vec<SessionLink>,    // 交叉引用
}
```

### 4.3 状态机

```
Created
  ├─ spawn() ──→ Running
  └─ cancel() ──→ Cancelled
Running
  ├─ pause() ──→ Paused
  ├─ checkpoint() ──→ Running (内部自环)
  ├─ cancel() ──→ Cancelling ──→ Cancelled
  └─ (agent 退出/超时) ──→ Completed
Paused
  ├─ resume() ──→ Running
  └─ cancel() ──→ Cancelling ──→ Cancelled
Completed / Cancelled
  └─ archive() ──→ Archived (终态)
Archived
  └─ (不可迁出)
```

> 复用 `CliSessionState` 但简化为 6 态 (去掉 Restart/Orphaned —— FileSession 由 CLI 直接管理，无 parent runtime 重启问题)

---

## §5 接口规范

### 5.1 CLI 命令 (star-cli 新增)

```bash
# 会话管理
star session start <file-path> [--agent <claude|codex|opencode|hermes|custom>] [--title <str>] [--tag <tag>...] [--link-task-card <id>] [--link-worktree <id>] [--json]
star session list [--file <path>] [--agent <type>] [--state <state>] [--tag <tag>] [--json] [--format table|json|tree]
star session show <session-id> [--json] [--include-transcript] [--include-artifacts]
star session resume <session-id> [--checkpoint <n>] [--json]
star session checkpoint <session-id> [--message <str>] [--auto-commit] [--json]
star session archive <session-id> [--json]
star session delete <session-id> [--force] [--json]
star session fork <session-id> [--title <str>] [--json]
star session link <source-session-id> <target-session-id> [--type <depends|relates|blocks>] [--json]

# Git 集成
star session sync [--remote <name>] [--push] [--pull] [--dry-run] [--json]
star session branch <session-id> [--create] [--list] [--switch] [--json]
star session export <session-id> [--format markdown|json|patch|mr] [--output <path>] [--json]
star session mr <session-id> [--title <str>] [--target-branch <branch>] [--draft] [--json]

# 适配器管理
star session adapter list [--json]
star session adapter add <path-to-wasm-or-so> [--name <str>] [--json]
star session adapter remove <adapter-name> [--json]
star session adapter configure <adapter-name> [--key <k> --value <v>] [--json]

# 配置
star session config show [--json]
star session config set <key> <value> [--global] [--json]
```

### 5.2 REST API (star-api-rest 新增 `/api/v1/file-sessions`)

| Method | Path | 说明 |
|--------|------|------|
| POST | `/api/v1/file-sessions` | 创建 FileSession |
| GET | `/api/v1/file-sessions` | 列出 (支持 `?file_path=&agent_type=&state=&limit=&offset=`) |
| GET | `/api/v1/file-sessions/{session_id}` | 获取详情 |
| PATCH | `/api/v1/file-sessions/{session_id}` | 更新 (state, tags, title) |
| POST | `/api/v1/file-sessions/{session_id}/checkpoint` | 创建检查点 |
| POST | `/api/v1/file-sessions/{session_id}/resume` | 恢复 |
| POST | `/api/v1/file-sessions/{session_id}/archive` | 归档 |
| DELETE | `/api/v1/file-sessions/{session_id}` | 删除 |
| POST | `/api/v1/file-sessions/{session_id}/fork` | 分叉 |
| POST | `/api/v1/file-sessions/{session_id}/link` | 关联其他 session |
| GET | `/api/v1/file-sessions/{session_id}/transcript` | 获取对话记录 (分页) |
| GET | `/api/v1/file-sessions/{session_id}/artifacts` | 获取产出物列表 |
| POST | `/api/v1/file-sessions/{session_id}/sync` | Git 同步 |
| POST | `/api/v1/file-sessions/{session_id}/export` | 导出 |

> **认证**: Bearer token (同现有 Group API)，RLS 强制 `tenant_id`

### 5.3 前端集成 (VS Code / Multica / Web IDE)

- **Code Lens**: 文件标题栏显示 "🤖 3 sessions"，点击展开列表
- **Inline Action**: 选中代码右键 "Start AI Session" → 预填上下文
- **Session Panel**: 侧边栏 TreeView 按文件分组显示所有 session
- **Diff View**: checkpoint 间可视化 diff
- **Live Share**: 实时协作光标 (WebRTC + CRDT)

---

## §6 实装计划

### Phase 1: 核心基础设施 (2 周)

| 任务 | 交付物 | 验收 |
|------|--------|------|
| 1.1 目录结构 + Registry/Manifest schema | `crates/star-file-session/` 新 crate | `cargo test` 通过 |
| 1.2 FileSessionManager (CRUD + 状态机) | `session_manager.rs` | 单元测试覆盖 90%+ |
| 1.3 存储后端抽象 (FS / SQLite 双模式) | `storage/` 模块 | 文件模式 + 内存模式测试通过 |
| 1.4 star-cli `session` 子命令骨架 | `commands/session.rs` | `star session --help` 可用 |
| 1.5 GitManager 基础 (init/add/commit/push/pull) | `git_manager.rs` | 临时 repo 集成测试通过 |

### Phase 2: Agent 适配器框架 (2 周)

| 任务 | 交付物 | 验收 |
|------|--------|------|
| 2.1 AgentAdapter trait + 插件注册表 | `adapters/` 模块 | 动态加载 .wasm/.so 测试通过 |
| 2.2 Claude Code 适配器 | `adapters/claude_code.rs` | 真实 `claude --continue` 集成测试 |
| 2.3 Codex 适配器 | `adapters/codex.rs` | 真实 `codex exec` 集成测试 |
| 2.4 Hermes 适配器 (复用 ACP) | `adapters/hermes.rs` | 复用现有 `agent-bridge` 测试通过 |
| 2.5 适配器配置 / 版本检测 / 健康检查 | `adapter_registry.rs` | `star session adapter list` 显示版本 |

### Phase 3: 上下文与检查点 (2 周)

| 任务 | 交付物 | 验收 |
|------|--------|------|
| 3.1 ContextBuilder (复用 star-context-tiering) | `context_builder.rs` | 初始上下文 token < 8k (典型文件) |
| 3.2 增量 checkpoint 编码/解码 | `checkpoint.rs` | checkpoint 体积 < 全量 20% |
| 3.3 Transcript 索引 + 分页读取 | `transcript.rs` | 10k 消息 < 100ms 定位 |
| 3.4 Token 预算 + 自动摘要 | `token_budget.rs` | 超限自动触发摘要，不丢关键信息 |
| 3.5 隐私过滤集成 | `privacy_filter.rs` | secrets/PII 100% 脱敏 |

### Phase 4: Git 深度集成 (1.5 周)

| 任务 | 交付物 | 验收 |
|------|--------|------|
| 4.1 自动提交 + 语义化 commit message | `git_auto_commit.rs` | `git log --oneline` 可读 |
| 4.2 Worktree 隔离并行 session | `git_worktree.rs` | 10 并发 session 无冲突 |
| 4.3 远程同步 (多 remote、冲突处理) | `git_sync.rs` | GitHub/GitLab 双向同步 |
| 4.4 MR 导出 + 评审集成 | `mr_export.rs` | 生成 MR 描述含完整 session 摘要 |

### Phase 5: REST API + 前端 (2 周)

| 任务 | 交付物 | 验收 |
|------|--------|------|
| 5.1 `/api/v1/file-sessions` REST 端点 | `crates/star-api-rest/src/group_api/file_sessions.rs` | OpenAPI 3.0 spec 生成 |
| 5.2 RLS + 权限模型 | `file_session_policy.sql` | 多租户隔离测试通过 |
| 5.3 VS Code 扩展原型 | `vscode-extension/` | CodeLens + Session Panel 可用 |
| 5.4 Multica Web 集成 | `frontend/src/app/worktree/[id]/file-sessions/` | 点击文件 → 弹窗创建/选择 session |

### Phase 6: 协作与高级特性 (持续)

| 任务 | 备注 |
|------|------|
| 6.1 实时协作 (Yjs/Automerge) | P2，需 WebRTC 信令服务器 |
| 6.2 语义合并冲突解决 | P2，基于 tree-sitter AST diff |
| 6.3 Session 模板/预设 | P3，如 "Code Review", "Refactor", "Debug" |
| 6.4 企业级审计导出 | P3，符合 SOC2/ISO27001 |

---

## §7 与现有系统的集成点

| 现有系统 | 集成方式 | 数据流向 |
|----------|----------|----------|
| **Task Card (ADR-0049)** | `linked_task_card_id` 字段双向绑定 | Task Card 创建 → 可选自动创建入口 FileSession；FileSession 完成 → 回填 Task Card 进度 |
| **Worktree (star-taskgraph)** | **隶属关系**：FileSession 归属 Worktree，与 Task Card 同级并列 | FileSession 在 Worktree 内创建，可共享或 fork worktree；权限继承 Worktree |
| **AgentSession (ADR-0045/ULYS-156)** | `linked_agent_session_id` 关联 | FileSession 可升级为完整 AgentSession (持久化到 PG) |
| **Context Packet (domain-context)** | 复用 `ContextPacket` schema 作为初始上下文 | FileSession 启动时生成 ContextPacket，checkpoint 时增量更新 |
| **Audit (domain-audit)** | 复用 `ai_audit_metadata` 记录所有操作 | 审计日志统一入库，支持跨域追溯 |
| **Credential (star-credential)** | 复用凭证路由规则 | Agent Adapter 调用时自动注入对应 provider credentials |
| **Hook Policy (domain-hook)** | Session 生命周期事件触发 hook | `session.created` / `session.checkpoint` / `session.archived` |

---

## §8 迁移与兼容性

### 8.1 现有数据迁移
- **无需迁移**：FileSession 是全新体系，不依赖现有 task_card/agent_session 数据
- **可选关联**：用户可在现有 Task Card 详情页点击 "Open File Session"，自动建立 `linked_task_card_id`

### 8.2 版本兼容性策略
- `SessionRegistry.version` / `FileSessionManifest.version` / `FileSession.meta.schema_version` 三级版本
- 升级时自动迁移 (类似 `sqlx` migrate)，保留 `v-1` 读取兼容
- 破坏性变更仅在 Major 版本，配套迁移工具 `star session migrate`

### 8.3 回滚策略
- Git 历史即回滚日志：`git checkout <commit> -- .star/sessions/...`
- `star session rollback <session_id> --to-checkpoint <n>`

---

## §9 风险与缓解

| 风险 | 影响 | 可能性 | 缓解措施 |
|------|------|--------|----------|
| Git 仓库膨胀 (大量 session + transcript) | 克隆慢、磁盘占用大 | 高 | 1) 默认不提交 transcript 到 Git (仅本地 + 可选远程 artifact store) 2) Git LFS 存大文件 3) 定期 `git gc` + 浅克隆支持 |
| 并发 session 编辑同一文件冲突 | 数据损坏、用户困惑 | 中 | 1) 乐观锁 (session 级) 2) 语义合并 3) UI 明确提示冲突 |
| Agent CLI 版本不兼容导致 spawn 失败 | 会话无法启动 | 中 | 1) 适配器版本检测 + 兼容性矩阵 2) 降级到基础模式 3. 清晰错误提示 |
| 敏感信息泄露 (transcript 含 secrets) | 安全事件 | 高 | 1) 默认开启隐私过滤 2) transcript 不进 Git 3) 导出前二次确认 |
| 存储后端切换 (FS → SQLite → PG) 数据不一致 | 数据丢失 | 低 | 1) 统一 `StorageBackend` trait 2) 迁移工具验证 checksum 3. 集成测试覆盖 |

---

## §10 验收标准 (Definition of Done)

| 维度 | 标准 |
|------|------|
| **功能** | 所有 P0 FR 通过验收测试 (自动化 + 手工) |
| **性能** | 基准测试达标 (见 §3) |
| **测试** | 单测覆盖 ≥ 85%，集成测试覆盖核心流程 100% |
| **文档** | CLI help / REST API spec / 架构决策记录 (ADR) 全部落档 |
| **安全** | 无高危漏洞 (cargo audit / dependabot 通过) |
| **示例** | README 包含 3 个端到端示例 (探索/重构/调试) |

---

## §11 附录

### 11.1 术语表
| 术语 | 定义 |
|------|------|
| FileSession | 绑定单个源码文件的 AI agent 会话 |
| Session Workspace | 会话的持久化目录 |
| AgentAdapter | 统一 AI CLI 交互的抽象接口 |
| Checkpoint | 会话上下文的增量快照 |
| Transcript | 完整对话记录 (用户/助手/工具调用) |

### 11.2 相关文档
- `docs/architecture/2026-08-26-upgrade/adr/0049-task-card-auto-worktree-agent.md`
- `docs/architecture/2026-08-26-upgrade/adr/0045-agent-runtime.md`
- `docs/architecture/2026-08-26-upgrade/adr/0024-ide-session-identity.md`
- `docs/architecture/2026-08-26-upgrade/spec/resources/04-ide-session-identity.md`
- `crates/domain-local-runtime/src/cli_session.rs` (ULYS-156)
- `crates/star-api-rest/src/group_api/cli_sessions.rs` (Task Card CLI Session REST)

### 11.3 变更历史
| 版本 | 日期 | 作者 | 变更 |
|------|------|------|------|
| v0.1 | 2026-10-12 | Ulysses (Mavis 接手) | 初版建立 |

---