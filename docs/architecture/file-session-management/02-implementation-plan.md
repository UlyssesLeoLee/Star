# FSMS 实装计划 (Implementation Plan)
> **架构集成状态（2026-10-12）**：本文件包作为设计提案并入 `dev`，尚未通过产品架构裁决。其中“FileSession 直接隶属 Worktree、与 Task Card 并列”等层级陈述与当前规范（Project → Cloud Branch → Engineering Run → Worktree；Run 内 App/工作项按 RunContext 协作）不一致。本提案不得作为已批准的实现依据；FileSession 的归属、与 Run/Worktree/Task Card 的关系、存储及 CLI 边界须重新评审，并同步需求、基本设计、详细设计后才能排期。

> **Status**: Draft v0.1
> **Based on**: ADR-0052 / requirements.md
> **Target**: Phase 1-3 完成核心闭环，Phase 4-5 交付生产可用

---

## Phase 1: 核心基础设施 (2 周)

### Week 1: Crate 创建 + 数据模型 + 存储后端

#### Day 1-2: Crate 脚手架
- [ ] `cargo new --lib crates/star-file-session`
- [ ] 添加依赖: `serde`, `serde_json`, `uuid`, `chrono`, `tokio`, `rusqlite`, `git2`, `anyhow`, `thiserror`, `clap`, `async-trait`
- [ ] 定义核心 Schema: `SessionRegistry`, `FileSessionManifest`, `FileSession`, `SessionMeta`, `SessionState`, `AgentType`
- [ ] 单元测试: JSON 序列化往返、UUID 生成、状态机合法迁移

#### Day 3-4: 存储后端抽象
```rust
// crates/star-file-session/src/storage/mod.rs
#[async_trait]
pub trait StorageBackend: Send + Sync {
    async fn create_session_workspace(&self, session: &FileSession) -> Result<()>;
    async fn read_session(&self, session_id: Uuid) -> Result<FileSession>;
    async fn update_session(&self, session: &FileSession) -> Result<()>;
    async fn delete_session(&self, session_id: Uuid) -> Result<()>;
    async fn list_sessions(&self, filter: SessionFilter) -> Result<Vec<SessionRef>>;
    async fn read_registry(&self) -> Result<SessionRegistry>;
    async fn write_registry(&self, registry: &SessionRegistry) -> Result<()>;
    async fn read_manifest(&self, file_path: &str) -> Result<FileSessionManifest>;
    async fn write_manifest(&self, manifest: &FileSessionManifest) -> Result<()>;
}

// 两个实现:
// - FileStorage: 直接读写 .star/sessions/ 目录 (生产默认)
// - SqliteIndexStorage: SQLite 索引加速查询 (可选，大仓库用)
```

#### Day 5: FileSessionManager 核心逻辑
- [ ] `create_session(file_path, agent_type, options) -> FileSession`
- [ ] `get_session(session_id) -> FileSession`
- [ ] `update_session_state(session_id, new_state) -> Result`
- [ ] `list_sessions(filter) -> Vec<SessionRef>`
- [ ] `archive_session(session_id)`
- [ ] `delete_session(session_id, force)`

### Week 2: GitManager + CLI 集成

#### Day 1-2: GitManager
```rust
// crates/star-file-session/src/git_manager.rs
pub struct GitManager {
    repo: git2::Repository,
    config: GitConfig,
}

impl GitManager {
    pub fn init_session(&self, session: &FileSession) -> Result<()>;
    pub fn commit_checkpoint(&self, session: &FileSession, checkpoint_num: usize) -> Result<()>;
    pub fn commit_archive(&self, session: &FileSession) -> Result<()>;
    pub fn create_worktree(&self, session: &FileSession) -> Result<PathBuf>;
    pub fn sync(&self, remote: Option<&str>, push: bool, pull: bool) -> Result<SyncResult>;
    pub fn export_mr(&self, session: &FileSession) -> Result<String>;
}
```

#### Day 3-4: star-cli `session` 子命令
```rust
// crates/star-cli/src/commands/session.rs
#[derive(Subcommand)]
pub enum SessionCommand {
    Start(StartArgs),
    List(ListArgs),
    Show(ShowArgs),
    Resume(ResumeArgs),
    Checkpoint(CheckpointArgs),
    Archive(ArchiveArgs),
    Delete(DeleteArgs),
    Fork(ForkArgs),
    Link(LinkArgs),
    Sync(SyncArgs),
    Branch(BranchArgs),
    Export(ExportArgs),
    Mr(MrArgs),
    Adapter(AdapterArgs),
    Config(ConfigArgs),
}
```

#### Day 5: 端到端集成测试
- [ ] 临时 git repo 初始化
- [ ] `star session start <file>` → 创建目录结构 + registry + manifest
- [ ] `star session list` → 显示会话
- [ ] `star session show <id>` → 显示详情
- [ ] `star session checkpoint <id>` → 写 checkpoint + git commit
- [ ] `star session resume <id>` → 读取上下文 (Mock adapter)
- [ ] `star session archive <id>` → 标记归档 + git commit

---

## Phase 2: Agent 适配器框架 (2 周)

### Week 1: 插件架构 + 基础适配器

#### Day 1-2: AgentAdapter Trait + 注册表
```rust
// crates/star-file-session/src/adapters/mod.rs
#[async_trait]
pub trait AgentAdapter: Send + Sync {
    fn agent_type(&self) -> AgentType;
    fn version(&self) -> semver::Version;
    fn capabilities(&self) -> AdapterCapabilities;
    
    async fn spawn(&self, ctx: SpawnContext) -> Result<SessionHandle>;
    async fn attach(&self, handle: SessionHandle) -> Result<AttachResult>;
    async fn checkpoint(&self, handle: &SessionHandle) -> Result<Checkpoint>;
    async fn resume(&self, handle: SessionHandle, checkpoint: Checkpoint) -> Result<SessionHandle>;
    async fn shutdown(&self, handle: SessionHandle, graceful: bool) -> Result<ShutdownResult>;
    async fn export(&self, handle: &SessionHandle, format: ExportFormat) -> Result<Vec<u8>>;
    async fn health_check(&self) -> Result<HealthStatus>;
}

pub struct AdapterRegistry {
    adapters: HashMap<AgentType, Box<dyn AgentAdapter>>,
    custom_adapters: HashMap<String, Box<dyn AgentAdapter>>, // WASM/动态库
}
```

#### Day 3: CLI 适配器基类 (复用 `std::process::Command` + PTY)
```rust
// crates/star-file-session/src/adapters/cli_base.rs
pub struct CliAdapterBase {
    pub command: String,
    pub args_template: Vec<String>,      // 模板，含 {session_id}, {prompt}, {context_file}
    pub resume_args: Vec<String>,        // --continue/--resume 参数
    pub checkpoint_args: Vec<String>,    // --checkpoint 参数
    pub export_args: Vec<String>,        // --export 参数
}
```

#### Day 4-5: Claude Code 适配器 (最高优先级)
- [ ] 检测 `claude` 版本、支持的模型
- [ ] `spawn`: `claude --print "<prompt>" --model <model> --max-tokens <n>`
- [ ] `attach`: `claude --continue --resume <session_id>`
- [ ] `checkpoint`: 解析输出提取 checkpoint 数据 (或调用 `--checkpoint` flag)
- [ ] `export`: `--export json`

### Week 2: Codex / OpenCode / Hermes 适配器

#### Day 1-2: Codex 适配器
- [ ] `codex exec "<prompt>"` / `codex exec --continue <session_id>`
- [ ] 解析 Codex session 文件格式 (`~/.codex/sessions/`)

#### Day 3: OpenCode 适配器
- [ ] `opencode run "<prompt>"` / `opencode run --continue <session_id>`
- [ ] 插件机制探索

#### Day 4: Hermes 适配器 (复用 ACP)
- [ ] 复用 `agent-bridge` / `domain-local-runtime` 现有 ACP 客户端
- [ ] `hermes acp --session <id>` / `hermes acp --attach <id>`

#### Day 5: 适配器配置 + 健康检查 + CLI 暴露
- [ ] `star session adapter list` 显示版本、能力
- [ ] `star session adapter configure <name> --key <k> --value <v>`
- [ ] 启动时自动健康检查，不可用标记 degraded

---

## Phase 3: 上下文与检查点 (2 周)

### Week 1: ContextBuilder + 初始上下文

#### Day 1-2: ContextBuilder (复用现有基础设施)
```rust
// crates/star-file-session/src/context_builder.rs
pub struct ContextBuilder {
    tiering: Arc<star_context_tiering::ContextTiering>,
    graph: Arc<treesitter_worktree_graph::GraphService>,
    credential_filter: Arc<star_credential::PrivacyFilter>,
}

impl ContextBuilder {
    pub async fn build_initial(&self, file_path: &str, repo_root: &str) -> Result<InitialContext> {
        // 1. 读取文件内容
        // 2. treesitter 提取符号 (functions, structs, imports)
        // 3. 依赖图: 谁依赖该文件、该文件依赖谁
        // 4. 最近 git log (--oneline -20 -- <file>)
        // 5. 隐私过滤: 脱敏 secrets/PII/内部路径
        // 6. 组装 InitialContext，估算 token
    }
}
```

#### Day 3-4: 增量 Checkpoint 编码
```rust
// crates/star-file-session/src/checkpoint.rs
#[derive(Serialize, Deserialize)]
pub struct Checkpoint {
    pub number: usize,
    pub context_delta: ContextDelta,      // 仅增量: 新增/修改/删除的上下文片段
    pub transcript_range: (usize, usize), // 本 checkpoint 涉及的消息范围
    pub artifacts: Vec<ArtifactRef>,      // 新产出的 patch/diff/export
    pub agent_state: AgentStateSnapshot,  // adapter 导出的内部状态
    pub token_delta: TokenUsage,          // 本轮 token 消耗
    pub created_at: DateTime<Utc>,
}

pub struct ContextDelta {
    pub added: Vec<ContextChunk>,
    pub modified: Vec<ContextChunk>,
    pub removed: Vec<String>,             // chunk IDs
}
```

#### Day 5: Transcript 索引 + 分块存储
- [ ] `transcript/index.json`: 消息索引 (id, offset, len, type, timestamp)
- [ ] `transcript/XXXX-YYYY.msgpack`: MessagePack 分块 (每块 100 条)
- [ ] 随机访问 O(1) 定位任意消息

### Week 2: Token 预算 + 隐私过滤 + 恢复逻辑

#### Day 1-2: Token 预算管理
```rust
// crates/star-file-session/src/token_budget.rs
pub struct TokenBudgetManager {
    max_tokens: usize,
    summarizer: Arc<dyn Summarizer>,  // 可插拔: LLM 摘要 / 规则截断
}

impl TokenBudgetManager {
    pub fn check_and_maybe_summarize(&mut self, context: &mut SessionContext) -> Result<()> {
        if context.token_used > self.max_tokens * 0.8 {
            // 触发摘要: 保留最近 N 轮 + 关键决策 + 文件当前状态
            let summary = self.summarizer.summarize(&context.transcript)?;
            context.context.checkpoints.push(Checkpoint {
                context_delta: ContextDelta::summary(summary),
                ...
            });
        }
    }
}
```

#### Day 3: 隐私过滤集成
- [ ] 复用 `star-credential` 的规则引擎
- [ ] 预置规则: API Keys, JWT, 数据库连接串, 内网 IP, 邮箱, 手机号
- [ ] 用户可扩展自定义规则 (`.star/privacy-rules.yaml`)

#### Day 4-5: 恢复逻辑 + 端到端测试
- [ ] `resume` 从最新 checkpoint 重建上下文
- [ ] 验证: 恢复后 token < 初始 30%
- [ ] 压测: 100 轮对话 checkpoint 恢复正确性

---

## Phase 4: Git 深度集成 (1.5 周)

### Week 1: 自动提交 + Worktree 隔离

#### Day 1-2: 语义化 Commit Message
- [ ] Template: `chore(session): {file} {action} {num} [{session_id:8}]`
- [ ] Action: `init` / `checkpoint` / `archive` / `fork` / `promote`
- [ ] 可选 GPG 签名

#### Day 3-4: Worktree 隔离
```rust
// 每 session 可选独立 worktree
git worktree add .worktrees/wt-session-{hash} -b session/{file-hash}/{session_id}

// Session 内文件操作发生在 worktree
// 结束时: merge 回主分支 / 保留 worktree / 删除 worktree
```

#### Day 5: 并发安全 (多 session 同文件)
- [ ] 乐观锁: session 级 `version` 字段，更新时 CAS
- [ ] 文件级锁: `.star/sessions/<file>/.lock` (fcntl/flock)

### Week 1.5: 远程同步 + MR 导出

#### Day 1-3: 远程同步
- [ ] `star session sync --remote origin --push --pull`
- [ ] 冲突解决: registry/manifest 合并 (last-writer-wins + 用户确认)
- [ ] 大文件: Git LFS / S3 artifact store

#### Day 4: MR 导出
- [ ] `star session export <id> --format mr`
- [ ] 生成 Markdown: 标题、摘要、关键决策、patch 列表、transcript 链接
- [ ] `star session mr <id> --title "..." --target-branch main --draft`

---

## Phase 5: REST API + 前端集成 (2 周)

### Week 1: REST API (star-api-rest)

#### Day 1-2: Router + Handlers
```rust
// crates/star-api-rest/src/group_api/file_sessions.rs
router()
    .route("/api/v1/file-sessions", post(create).get(list))
    .route("/api/v1/file-sessions/{session_id}", get(get).patch(update).delete(delete))
    .route("/api/v1/file-sessions/{session_id}/checkpoint", post(checkpoint))
    .route("/api/v1/file-sessions/{session_id}/resume", post(resume))
    .route("/api/v1/file-sessions/{session_id}/archive", post(archive))
    .route("/api/v1/file-sessions/{session_id}/fork", post(fork))
    .route("/api/v1/file-sessions/{session_id}/link", post(link))
    .route("/api/v1/file-sessions/{session_id}/transcript", get(transcript))
    .route("/api/v1/file-sessions/{session_id}/artifacts", get(artifacts))
    .route("/api/v1/file-sessions/{session_id}/sync", post(sync))
    .route("/api/v1/file-sessions/{session_id}/export", post(export))
    .route("/api/v1/file-sessions/{session_id}/mr", post(create_mr))
```

#### Day 3: RLS + 权限
- [ ] PostgreSQL RLS Policy: `tenant_id` 强制匹配
- [ ] 权限: `file_session:read` / `file_session:write` / `file_session:admin`
- [ ] 继承: 仓库 Owner/Editor/Viewer → FileSession 权限

#### Day 4-5: OpenAPI Spec + 集成测试
- [ ] `utoipa` 生成 OpenAPI 3.0
- [ ] 契约测试: CLI ↔ REST 行为一致

### Week 2: 前端集成

#### Day 1-2: VS Code 扩展原型
- [ ] CodeLens: 文件标题栏显示 "🤖 N sessions"
- [ ] TreeView: 侧边栏 `.star/sessions` 按文件分组
- [ ] 右键菜单: "Start AI Session" / "Resume Session" / "View Transcript"
- [ ] Diff View: checkpoint 间可视化 diff

#### Day 3-4: Multica Web 集成
- [ ] `frontend/src/app/worktree/[id]/file-sessions/page.tsx`
- [ ] 文件树点击 → 弹窗显示 session 列表 → 创建/恢复
- [ ] 实时同步: WebSocket 监听 Git 变更 (或轮询 REST)

#### Day 5: 端到端验收
- [ ] CLI 创建 → Web 可见 → VS Code 可见 → CLI resume → 循环

---

## 依赖关系图

```
Phase 1 (核心基建)
    │
    ├── Phase 2 (适配器) ◄── 依赖 Phase 1 SessionHandle/Storage
    │
    ├── Phase 3 (上下文) ◄── 依赖 Phase 1 ContextBuilder 接口
    │       └── 复用 star-context-tiering / treesitter-worktree-graph (现有)
    │
    ├── Phase 4 (Git深度) ◄── 依赖 Phase 1 GitManager
    │
    └── Phase 5 (API+前端) ◄── 依赖 Phase 1-3 核心功能稳定
            ├── REST API ◄── 复用 star-api-rest 现有认证/RLS
            ├── VS Code ◄── 独立扩展，调用 CLI 或 REST
            └── Multica Web ◄── 复用现有 frontend 架构
```

---

## 里程碑交付物清单

| 里程碑 | 交付物 | 验收命令 |
|--------|--------|----------|
| **M1: Phase 1 完工** | `star-file-session` crate + `star session` 基础命令 | `star session start test.rs && star session list` |
| **M2: Phase 2 完工** | 4 个内置适配器 + 插件框架 | `star session start test.rs --agent claude && star session checkpoint <id>` |
| **M3: Phase 3 完工** | 增量 checkpoint + 恢复 + 隐私过滤 | `star session resume <id> --verify-tokens` |
| **M4: Phase 4 完工** | Git 自动提交 + worktree + sync + MR | `star session sync && star session mr <id>` |
| **M5: Phase 5 完工** | REST API + VS Code + Multica | `curl /api/v1/file-sessions` + VS Code CodeLens 可见 |

---

## 资源需求

| 角色 | Phase 1 | Phase 2 | Phase 3 | Phase 4 | Phase 5 |
|------|---------|---------|---------|---------|---------|
| Rust 后端 | 1.0 FTE | 1.0 FTE | 1.0 FTE | 0.5 FTE | 0.5 FTE |
| CLI/工具链 | 0.5 FTE | 0.5 FTE | - | 0.5 FTE | 0.5 FTE |
| 前端 | - | - | - | - | 1.0 FTE |
| QA/测试 | 0.3 FTE | 0.3 FTE | 0.5 FTE | 0.3 FTE | 0.5 FTE |

---

## 风险缓冲

| 风险 | 缓冲时间 | 备选方案 |
|------|----------|----------|
| Agent CLI 版本不兼容 | +3 天/适配器 | 降级到基础模式 (仅 spawn/attach，无 checkpoint) |
| Git 仓库膨胀超预期 | +1 周 | 强制 transcript 不进 Git，仅本地 + S3 |
| 并发冲突复杂度高 | +1 周 | 先单 session 单文件，后续迭代加并发 |
| 前端集成 API 变更 | +3 天 | 先定 OpenAPI 契约，前后端并行 |

---

## 当前可执行的 Next Actions

1. **立即可做**: `cargo new --lib crates/star-file-session` + 定义核心 Schema
2. **今日可做**: 实现 `FileStorage` + `SessionRegistry` CRUD + 单测
3. **本周可做**: `star session start/list/show` CLI 命令跑通
4. **下周**: 引入 `git2` 实现 `GitManager.init_session/commit_checkpoint`