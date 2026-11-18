# F12 文件会话管理（File Session）

> 状态摘要: FileSession（文件级 AI agent 会话）在当前代码中没有任何实现：不存在 `star-file-session` crate、`/api/v1/file-sessions` 路由、`star session` CLI 与 `.star/sessions/` 存储。相邻能力（IDE1.0 编辑内核工作区、`.nema` 文件即任务、ULYS-156 CLI Session、编辑器实例钩子）已存在，但不承载 FileSession 语义；FileSession 的归属（Worktree 双入口或 Run/Lane 能力）尚未裁决。
> 代码基线: HEAD 1643559f（本功能文档唯一基线）; 核对方法: 对 crates/、frontend/src、docs/ 做路径与符号 grep（FileSession、file_session、file-sessions、IdeFileWorkspace、.star/sessions、star session、AgentAdapter、linked_task_card），并读取工作区实现文件取得行号；工作区与 HEAD 仅 docs/architecture/file-session-management/01-requirements.md 差 1 行（§0.5.2 层级表的 L4b 右侧聊天栏行，与 RightChatRail.tsx:96 一致，采用工作区版本）; 核对日期 2026-11-17。

## 1. 目标与边界

### 1.1 目标

- 点击代码文件即开启一个绑定该文件的独立 AI agent 会话（探索、重构、调试、文档生成等），会话以 Git 结构化存储于 `.star/sessions/<相对文件路径>/<session_id>/`（01 §0.2、§1.1；ADR-0052 §0）。
- 会话生命周期：创建、列出、恢复、检查点、归档、删除、分叉（01 §2.1，FR-LIFECYCLE-01 至 06）。
- 多 agent CLI 适配：Claude Code、Codex、OpenCode、Hermes，以及通用 ACP 与插件机制（01 §2.2，FR-ADAPTER-01 至 06）。
- Git 集成：checkpoint 自动提交、可选 session 分支与 worktree 隔离、远程同步、MR 导出（01 §2.3）。
- 与 Task Card、Worktree、AgentSession 可选关联（01 §0.3、§4.2、§7）。

### 1.2 范围内 / 范围外

范围内：上述目标；UI 入口与布局契约（01 §0.2 至 §0.5）；REST 与 CLI 接口形态（03、04）；FileSession 与现有能力的边界。

范围外：

- IDE1.0 编辑内核（`crates/ide-kernel`、`/api/v1/ide/*`、`IdeFileWorkspace`）：相邻能力，本文件只记录与 FileSession 的边界（§4.6、§5）。
- ULYS-156 CLI Session（`crates/domain-local-runtime/src/cli_session.rs`）：相邻模型，FileSession 不复用其实体（§5）。
- 平台级 AgentSession 持久化（ADR-0045、ADR-0049）：仅以 promote 路径单向衔接（ADR-0052 §5.2），其契约不在本文件定义。
- 层级归属：FileSession 挂在 Worktree 下还是作为 Run/Lane 能力，尚未裁决（INTEGRATION-STATUS.md「实施前必须裁决」）。当前规范（AGENTS.md §0）为 Project → Lane → Worktree → Branch，Lane 管理本地 Worktree，Run 是任务、Canvas 与协作的 canonical scope。

## 2. 用户与场景

| 角色 | 主要场景 | 来源 |
|---|---|---|
| 开发者 | 在编辑某文件时开启 session，做代码探索、重构实验、调试复现、单测编写 | 01 §0.3；ADR-0052 §0.1 |
| 开发者（多会话） | 同一文件并行多个独立 session，可 fork 与关联 | 01 §0.3（1 File → N FileSessions）；01 FR-COLLAB-04 |
| 代码评审者 | 将 session 导出为 MR 描述，或由 MR 评审 session 变更 | 01 FR-GIT-04；03 §16 |
| 团队成员（跨设备） | 通过 Git push/pull 同步 session 历史 | 01 FR-GIT-03；04 `sync` |
| 平台管理员 | 配置 adapter、隐私规则、默认 agent 与 token 预算 | 03 权限矩阵（`file_session:admin`）；04 `config` |

## 3. 需求

优先级与需求文本取自 01（FR 编号括注于需求列）。无源优先级的 UI 设计项标 `—`。

| 编号 | 需求 | 优先级 | 状态 | 证据 |
|---|---|---|---|---|
| FR-12-01 | 创建 FileSession（`star session start` / `POST /api/v1/file-sessions`，返回 session_id 与工作区路径）（FR-LIFECYCLE-01） | P0 | ⬜ | 无 `star session` 子命令（crates/star-cli/src/commands/mod.rs 无 session 模块）；全仓无 `file-sessions` 路由字符串 |
| FR-12-02 | 列出会话，支持按 file、agent、state、tag 过滤（FR-LIFECYCLE-02） | P0 | ⬜ | 同 FR-12-01 |
| FR-12-03 | 恢复会话，从最新或指定 checkpoint 恢复 transcript、context 与 agent 状态（FR-LIFECYCLE-03） | P0 | ⬜ | 无 FileSession 恢复实现；`CliSessionState::Restart`（crates/domain-local-runtime/src/cli_session.rs:61-75）属 ULYS-156，不对应 FileSession |
| FR-12-04 | 检查点：增量持久化上下文，可选自动 commit（FR-LIFECYCLE-04） | P1 | ⬜ | 无 checkpoint 数据结构或存储代码 |
| FR-12-05 | 归档：只读化并压缩 transcript（FR-LIFECYCLE-05） | P2 | ⬜ | 无 |
| FR-12-06 | 删除：默认软删除到 `.star/sessions/.trash/`，`--force` 物理删除（FR-LIFECYCLE-06；03 §8） | P2 | ⬜ | 无 |
| FR-12-07 | Claude Code 适配器：spawn、attach、checkpoint、resume、export（FR-ADAPTER-01） | P0 | ⬜ | 全仓无 `trait AgentAdapter`；`AdapterRegistry`（crates/infrastructure/src/lib.rs:79）注册的是平台基础设施适配器（postgres、nats、object_storage、scm、agent），不含 CLI 会话契约 |
| FR-12-08 | Codex 适配器（`exec` 与 apply-patch 模式）（FR-ADAPTER-02） | P0 | ⬜ | 同 FR-12-07 |
| FR-12-09 | OpenCode 适配器（FR-ADAPTER-03） | P1 | ⬜ | 同 FR-12-07 |
| FR-12-10 | Hermes 适配器，复用 ACP/ACL 基础设施（FR-ADAPTER-04） | P1 | ⬜ | 适配器本身未实现。复用前提未核实：`crates/agent-bridge/src/lib.rs:1-6` 声明 `AgentRuntime` trait 与 Multica adapter，源码中无 ACP 实现（grep 无命中） |
| FR-12-11 | 通用 ACP 协议适配器（FR-ADAPTER-05） | P2 | ⬜ | 同 FR-12-07 |
| FR-12-12 | 自定义 agent 插件机制，WASM 或动态库加载（FR-ADAPTER-06） | P2 | ⬜ | 无动态加载代码 |
| FR-12-13 | checkpoint 自动 `git add` 与 commit，message 为 `chore(session): <file> checkpoint <n>`（FR-GIT-01） | P0 | ⬜ | 全仓无 `chore(session)` 字符串 |
| FR-12-14 | 每 session 可选绑定 `session/<file-hash>/<session-id>` 分支，并以 worktree 隔离（FR-GIT-02） | P1 | ⬜ | 无 session 分支逻辑；`Worktree` 实体存在（crates/domain-worktree/src/lib.rs:189）但无 session 关联 |
| FR-12-15 | `star session sync`：push 与 pull 全部 session 历史（FR-GIT-03） | P1 | ⬜ | 无 |
| FR-12-16 | MR 导出与创建（`export --format mr`、`mr` 命令）（FR-GIT-04；03 §15、§16） | P2 | ⬜ | 无；`crates/star-cli/src/commands/mr.rs` 存在但不处理 session |
| FR-12-17 | 同一文件多 session 并发编辑的冲突解决（FR-GIT-05） | P2 | ⬜ | 无 |
| FR-12-18 | 初始上下文：文件内容、符号、依赖图、最近 git log（FR-CONTEXT-01） | P0 | ⬜ | 无 ContextBuilder。可用构件：`parse_rust`、`parse_typescript`（crates/star-treesitter/src/lib.rs:248、253）；01 指定的 `treesitter-worktree-graph` crate 不存在 |
| FR-12-19 | 增量 checkpoint，仅存增量，压缩率大于 80%（FR-CONTEXT-02） | P0 | ⬜ | 无 |
| FR-12-20 | `@import` 语法引用同仓库其他文件的 session 上下文（FR-CONTEXT-03） | P1 | ⬜ | 无 |
| FR-12-21 | token 预算：可配置 max_tokens，超限自动摘要或截断（FR-CONTEXT-04） | P1 | ⬜ | 可复用类型 `ContextPacket.token_budget`（crates/domain-context/src/lib.rs:230）未被 FileSession 使用；全仓无 tiktoken 依赖 |
| FR-12-22 | 隐私过滤：脱敏 secrets、PII、内部路径（FR-CONTEXT-05） | P1 | ⬜ | `star-credential` 是 KMS 加密凭证管理（crates/star-credential/src/lib.rs:1-15），无规则引擎；无 privacy 规则代码 |
| FR-12-23 | 多用户协作，OT 或 CRDT（FR-COLLAB-01） | P2 | ⬜ | 无 |
| FR-12-24 | 审计日志：谁、何时、做了什么，复用 ai-audit schema（FR-COLLAB-02） | P1 | ⬜ | `domain-audit` crate 存在，无 session 事件；全仓无 `ai_audit_metadata` |
| FR-12-25 | 权限键 `file_session:*`，继承 Worktree 的 Owner、Editor、Viewer（FR-COLLAB-03；03 权限矩阵） | P1 | ⬜ | `domain-permission` 为 workspace 成员（Cargo.toml:24）；全仓无 `file_session:` 权限键 |
| FR-12-26 | 会话分叉 fork，继承历史与共同祖先（FR-COLLAB-04；03 §9） | P2 | ⬜ | 无 |
| FR-12-27 | 与 Task Card 可选双向关联（`linked_task_card_id`）（ADR-0052 §1 决策 6；01 §4.2） | — | ⬜ | 全仓 crates 与 frontend/src 无 `linked_task_card` 字段 |
| FR-12-28 | 与 Worktree、AgentSession 可选关联（`linked_worktree_id`、`linked_agent_session_id`）（01 §4.2） | — | ⬜ | 无 |
| FR-12-29 | Worktree 详情页并列展示 "Task Cards" 与 "File Sessions" 两个入口（01 §0.3；ADR-0052 §0.1） | — | ⬜ | `frontend/src/app/worktree/[id]/` 下仅 `group` 目录（git ls-tree 1643559f）；`frontend/src/app/worktree/page.tsx:962` 显示「历史 Agent Sessions 待接入」 |
| FR-12-30 | 文件树右键菜单含 "Start File Session" 等项（01 §0.4） | — | ⬜ | 全仓无 "Start File Session" 字符串 |
| FR-12-31 | 编辑器 CodeLens 显示 "N sessions"，悬停展开会话列表（01 §0.4） | — | 🟡 | 编辑器实例钩子已有：`frontend/src/components/editor/MonacoEditor.tsx:104`（`onEditorReady`），调用在 :380；未注册任何 CodeLens provider（全仓无 `registerCodeLensProvider`） |
| FR-12-32 | 菜单交互：跟随鼠标定位、子菜单安全三角区、Floating UI 定位、会话列表超 100 项虚拟化（01 §0.4） | — | ⬜ | `frontend/package.json` 无 `@floating-ui` 或 `popper` 依赖（grep 无命中） |
| FR-12-33 | 底部 dock 终端面板：`z-[35]`，`left: var(--sidebar-w)`，被侧栏覆盖（01 §0.5.2、§0.5.3；INV-LAYOUT-01、02） | — | ⬜ | 代码中无使用 `z-[35]` 的元素（`frontend/src/components/Sidebar.tsx:115` 仅注释提及） |
| FR-12-34 | Sidebar 为结构层，`z-40` 高于内容层与 dock（INV-LAYOUT-04） | — | ✅ | `frontend/src/components/Sidebar.tsx:118`（class 含 `sticky top-0 h-screen ... z-40`） |
| FR-12-35 | 侧栏宽度为单一事实源 CSS 变量，所有 fixed 元素引用（INV-LAYOUT-03） | — | 🟡 | 变量定义 `frontend/src/app/globals.css:56`、`:60`、`:66`（✅）；群组页聊天栏仍硬编码 232px 与 252px 偏移（`frontend/src/app/worktree/[id]/group/page.tsx:3399`） |
| FR-12-36 | 布局与交互验收用例（01 §0.4 五条、§0.5.4 六条）的自动化断言 | — | ❓ | 未发现对应测试；需检索测试目录中的 z-index 与 computed-style 断言以确认 |

## 4. 设计

### 4.1 概念模型

| 概念 | 定义 | 来源 | 状态 |
|---|---|---|---|
| FileSession | 绑定单个仓库相对文件路径的 AI agent 会话 | 01 §1 | ⬜ |
| Session Workspace | 会话持久化目录 `<repo>/.star/sessions/<file>/<session_id>/` | 01 §1.1；ADR-0052 §3.1 | ⬜ |
| Session Manifest | 单文件的会话摘要索引 `manifest.json` | 01 §1；02 §3.2 | ⬜ |
| Session Registry | 仓库级聚合索引 `registry.json`，Git 跟踪，SQLite 为辅助索引 | 01 §1；ADR-0052 §1 决策 1 | ⬜ |
| Checkpoint | 增量上下文快照，含 context delta、transcript 范围与 artifact 引用 | 01 §2.4；02 §3.2 | ⬜ |
| Transcript | 对话记录：索引加分块存储，支持随机访问 | 01 §4.2；02 Phase 3 Day 5 | ⬜ |
| Artifact | patch、diff、export 等产出物 | 01 §1.1 | ⬜ |
| Agent Adapter | 统一 AI CLI 交互的抽象接口 | 01 §2.2；02 Phase 2 | ⬜ |
| Session Link | 与 Task Card、Worktree、AgentSession、父子 session 及其他 session 的关联 | 01 §4.2（SessionLinks） | ⬜ |

### 4.2 数据模型

存储布局（采用 01 §1.1 与 ADR-0052 §3.1 的按文件分层方案）：

```
<repo>/.star/sessions/
  registry.json                       # 仓库级索引
  config.toml                         # FSMS 配置
  <relative/file/path>/
    manifest.json                     # 单文件会话列表
    <session_id>/
      session.json                    # 元数据与索引
      context/initial.json
      context/checkpoints/NNN.json
      transcript/index.json
      transcript/NNNN-NNNN.msgpack    # 默认不入 Git
      artifacts/{patches,diffs,exports}/
      git/                            # branch、commits、mr-links
      state/                          # adapter 状态快照
```

实体与字段（以 01 §4.2、02 §3.2、03 资源模型为准）：

| 实体 | 字段 | 来源 | 状态 |
|---|---|---|---|
| SessionRegistry | version、updated_at、sessions（SessionRef 列表） | 01 §4.2 | ⬜ |
| SessionRef | session_id、file_path、agent_type、state、created_at、updated_at、tags、linked_task_card_id、linked_worktree_id、linked_agent_session_id | 01 §4.2 | ⬜ |
| FileSessionManifest | file_path、version、updated_at、sessions（SessionSummary 列表） | 01 §4.2；02 §3.2 | ⬜ |
| SessionSummary | session_id、agent_type、state、title、created_at、updated_at、checkpoint_count、token_usage（input、output、total）、git_branch | 01 §4.2；02 §3.2 | ⬜ |
| SessionMeta | session_id、file_path、agent_type、agent_version、adapter_config、state、created_at、updated_at、tags、links | 01 §4.2；03 资源模型 | ⬜ |
| SessionLinks | task_card_id、worktree_id、agent_session_id、parent_session_id、child_session_ids、related_sessions（type 为 depends、relates、blocks） | 01 §4.2；03 资源模型 | ⬜ |
| FileSession（详情） | meta、context（initial_ref、checkpoints、current_checkpoint、token_budget、token_used）、transcript（index_ref、chunks、total_messages）、artifacts、git（branch、worktree_path、commits、mr_links）、state_snapshots | 02 §3.2；03 资源模型 | ⬜ |
| Checkpoint | number、context_delta（added、modified、removed）、transcript_range、artifacts、agent_state、token_delta、created_at | 02 Phase 3 Day 3-4 | ⬜ |

源之间的字段冲突裁决见 §8。

### 4.3 接口

CLI（形态采用 04-cli-reference.md；命令清单采用 01 §5.1 并以 04 补充）：

| 命令组 | 命令 | 状态 |
|---|---|---|
| 会话 | `start`、`list`、`show`、`resume`、`checkpoint`、`archive`、`delete`、`fork`、`link` | ⬜ |
| 会话（仅 04） | `pause`、`rollback`、`promote`、`migrate`、`doctor` | ⬜ |
| Git | `sync`、`branch`（子命令 list、create、switch、delete、merge）、`export`、`mr` | ⬜ |
| 适配器 | `adapter list`、`adapter add`、`adapter remove`、`adapter configure` | ⬜ |
| 配置 | `config show`、`config set`、`config reset` | ⬜ |
| 环境变量 | `STAR_SESSION_REPO`、`STAR_SESSION_CONFIG`、`STAR_SESSION_DEFAULT_AGENT`、`STAR_SESSION_AUTO_COMMIT`、`STAR_SESSION_TOKEN_BUDGET`（04 环境变量节） | ⬜ |
| 退出码 | 0 至 10（04 退出码节） | ⬜ |

REST（形态采用 03-api-spec.md，其 01 §5.2 为子集；全部 `/api/v1/file-sessions`）：

| 路由组 | 端点（03 编号） | 状态 |
|---|---|---|
| 会话 CRUD | POST（1）、GET 列表（2）、GET 详情（3）、PATCH（4）、DELETE（8） | ⬜ |
| 操作 | `POST /{id}/checkpoint`（5）、`/resume`（6）、`/archive`（7）、`/fork`（9）、`/link`（10）、`/sync`（13）、`/export`（15）、`/mr`（16） | ⬜ |
| 读取 | `GET /{id}/transcript`（11）、`GET /{id}/artifacts`（12）、`GET|POST /{id}/branch`（14） | ⬜ |
| 适配器与配置 | `GET /adapters`、`PATCH /adapters/{adapter_name}`、`GET /config`、`PATCH /config` | ⬜ |
| 实时 | `GET /{id}/ws`（WebSocket，可选） | ⬜ |

Rust 端口（采用 02-implementation-plan.md 的签名，理由见 §8）：`AgentAdapter`（agent_type、version、capabilities、spawn、attach、checkpoint、resume、shutdown、export、health_check）、`StorageBackend`（会话、registry、manifest 的读写）、`GitManager`（init_session、commit_checkpoint、commit_archive、create_worktree、sync、export_mr）、`ContextBuilder`、`TokenBudgetManager`。以上均为 ⬜，代码中不存在同名 trait 或 struct。

### 4.4 状态机与关键流程

采用 6 态（01 §4.3、ADR-0052 §3.3、03 §3.3 一致；见 §8）：

```
created  --spawn-->       running
created  --cancel-->      cancelled
running  --checkpoint-->  running          （自环）
running  --pause-->       paused
paused   --resume-->      running
running  --cancel-->      cancelling --> cancelled
paused   --cancel-->      cancelling
running  --agent 正常退出--> completed
completed / cancelled --archive--> archived （终态，只读）
```

关键流程（02 §2.2）：分配 session_id → ContextBuilder 构建初始上下文（文件内容、符号、依赖、git log、隐私过滤） → StorageBackend 创建工作区与 manifest → GitManager 初始化提交 → AgentAdapter.spawn 启动 agent。全部 ⬜。

并发与恢复（02 Phase 4 Day 5；01 FR-LIFECYCLE-03、04）：session 级乐观锁（version CAS）加文件锁 `.star/sessions/<file>/.lock`；恢复从最新 checkpoint 重建上下文，目标为恢复后 token 低于初始上下文的 30%。全部 ⬜。

相邻现有流程（不属于 FileSession）：在看板「按文件开卡」页签中，选中文件触发 `kernel.openFile`（`frontend/src/components/ide/IdeFileWorkspace.tsx:232`），打开的是 IDE 内核文档会话（内存态），与 FileSession 无关联。

### 4.5 权限与安全

- 权限键（03 权限矩阵）：`file_session:create`、`read`、`write`、`delete`、`archive`、`fork`、`link`、`sync`、`export`、`mr`、`admin`，默认角色见 03。全部 ⬜（无代码）。
- 多租户隔离：PostgreSQL RLS，`WHERE tenant_id = current_setting('app.tenant_id')::uuid`（03 权限矩阵末）。⬜。
- 隐私：transcript 默认不入 Git，仅本地或可选 artifact store；导出默认脱敏（03 §15 `redact_secrets`；04 `--redact`）。⬜。
- 凭证：agent 环境变量注入（ADR-0052 §3.4 `SpawnContext.env`）。⬜。`star-credential` 的 KMS 加密与 rotate、revoke 规则为其自身不变量（crates/star-credential/src/lib.rs:1-15），尚未接入注入路径。
- 相邻 IDE 端点的当前边界（非 FileSession）：
  - 磁盘读仅在显式设置 `IDE_PROJECT_ROOT` 时启用，并拒绝路径逃逸（`crates/star-api-rest/src/routes/ide.rs:38-46`）：✅。
  - `/scm/*` 只执行只读 git，不接受 stage 或 commit（`routes/ide.rs:43-44`、`:156`）：✅。
  - 内核状态为单进程内存态，服务重启即失（`routes/ide.rs:36`、`:96`）：✅。
  - 认证：`/api/v1` 下的 `auth_layer_stub` 为直通（`crates/star-api-rest/src/lib.rs:219-220`；`crates/star-api-rest/src/middleware/auth.rs:23-31`），即当前无认证：⬜。

### 4.6 依赖关系

| 依赖 | 设计用途 | 代码状态 | 证据 |
|---|---|---|---|
| star-context-tiering | 上下文分层（FR-CONTEXT-01） | 🟡 crate 存在，与 FileSession 无调用 | crates/star-context-tiering/src/lib.rs:2-9 |
| treesitter-worktree-graph | 符号索引与依赖图 | ⬜ crate 不存在；可用替代 star-treesitter | crates/star-treesitter/src/lib.rs:248、253 |
| star-credential | 隐私规则与凭证注入（FR-CONTEXT-05） | ⬜ 无规则引擎；KMS 凭证管理存在 | crates/star-credential/src/lib.rs:1-15 |
| domain-context（ContextPacket） | 初始上下文与 token 预算 | 🟡 类型存在，未接入 | crates/domain-context/src/lib.rs:182、230 |
| domain-hook | session.created、session.checkpoint、session.archived 事件 | 🟡 信封类型存在，无 session 事件 | crates/domain-hook/src/lib.rs:179 |
| domain-audit | 审计（FR-COLLAB-02） | 🟡 crate 存在，无 session 事件 | crates/domain-audit（Cargo.toml name） |
| domain-permission | 权限继承（FR-COLLAB-03） | 🟡 crate 存在，无 `file_session:` 键 | Cargo.toml:24 |
| agent-bridge | Hermes ACP 复用（FR-ADAPTER-04） | ❓ `AgentRuntime` trait 存在，ACP 未核实 | crates/agent-bridge/src/lib.rs:1-6 |
| domain-local-runtime（ULYS-156） | 会话状态机与进程管理参考 | 🟡 7 态状态机与持久化存在；子进程实际 kill 留待 P1 | crates/domain-local-runtime/src/cli_session.rs:61-75、104-131；graceful_shutdown.rs:23 |
| Worktree 实体 | Worktree 归属 | 🟡 两处存在（star-taskgraph 与 domain-worktree），归属未裁决 | crates/star-taskgraph/src/lib.rs:112；crates/domain-worktree/src/lib.rs:189 |
| AdapterRegistry（平台） | 基础设施适配器注册 | ✅ 存在，语义为平台适配器，不是 CLI agent 适配器 | crates/infrastructure/src/lib.rs:79；registry.rs:200 |
| API 挂载点 | FileSession 路由挂载 | ✅ 存在，尚无 FileSession 模块 | crates/star-api-rest/src/lib.rs:219 |
| 当前层级规范 | Lane 与 Worktree 关系 | ✅ 文本存在 | AGENTS.md §0 |

## 5. 当前实现

| 模块 | 路径 | 状态 | 证据 |
|---|---|---|---|
| FileSession 核心 crate | `crates/star-file-session/` | ⬜ | 路径不存在（git ls-tree 1643559f） |
| FileSession REST 模块（计划文件） | `crates/star-api-rest/src/group_api/file_sessions.rs` | ⬜ | 文件不存在；同目录仅有 `cli_sessions.rs`、`execution_profiles.rs` 等 |
| FileSession 路由 `/api/v1/file-sessions/*` | `crates/star-api-rest/src/lib.rs:219`（挂载点） | ⬜ | 全仓无 `file-sessions` 路径字符串 |
| `star session` CLI | `crates/star-cli/src/commands/`（mod.rs） | ⬜ | 命令模块为 agent、code、context、issue、mr、pipeline、project、submit、task、test、workspace、worktree，无 session |
| `.star/sessions/` 存储 | 运行时目录，无对应代码 | ⬜ | 全仓无 `.star/sessions` 读写；`crates/star-cli/src/commands/submit.rs:404`、`:588` 使用的是 `.star/workspace.json` |
| FileSession 适配器 | 无 | ⬜ | 全仓无 `trait AgentAdapter` |
| 平台适配器注册表 | `crates/infrastructure/src/lib.rs:79`；`crates/infrastructure/src/registry.rs:200` | ✅ | 语义为平台基础设施适配器，与 FileSession 适配器无关 |
| ULYS-156 CLI Session 状态机 | `crates/domain-local-runtime/src/cli_session.rs:61`、`:104-131` | ✅ | 7 态状态机与 CliSession 实体；字段含 tenant_id、worktree_id、command、state_history，无文件路径或 task_card_id |
| CLI Session REST | `crates/star-api-rest/src/group_api/cli_sessions.rs` | ✅ | 文件存在，与 FileSession 无关联 |
| IDE1.0 编辑内核 HTTP | `crates/star-api-rest/src/routes/ide.rs:126-158`（路由）；挂载 `crates/star-api-rest/src/lib.rs:156`、`:321` | ✅ | 内存态（`routes/ide.rs:36`） |
| IDE1.0 内核会话类型 | `crates/ide-kernel/src/session.rs:13`（TabState）、`:24`（Session） | ✅ | 多标签内存会话，与 FileSession 无关 |
| 文件编辑工作区 | `frontend/src/components/ide/IdeFileWorkspace.tsx:170`（组件）、`:181`（useIdeKernel）、`:232`（选中文件）、`:274`（syncText） | ✅ | 未接入内核时降级（`IdeFileWorkspace.tsx:29-31`） |
| 内核 hook | `frontend/src/hooks/useIdeKernel.ts:183`（hook）、`:186`（同步防抖 400ms）、`:211`（probe）、`:226`（scm log）、`:249`（openFile）、`:337`（outline） | ✅ | 模式 `preview`、`connected`、`error`（`:85`） |
| 看板「按文件开卡」 | `frontend/src/app/(app)/kanban/page.tsx:68`（VIEW_TABS）、`:448-455` | ✅ | 挂载 IdeFileWorkspace，数据源 nemaTasks |
| `.nema` 文件即任务 | `frontend/src/lib/nema/nemaStore.ts:53-54`（规则）、`:187-189`（NemaState）、`:470`（persist name `star-nema-files`） | ✅ | 共同编辑产生 `.nema` 侧车；不创建 agent 会话 |
| 编辑器实例钩子 | `frontend/src/components/editor/MonacoEditor.tsx:104`（onEditorReady）、`:380`（调用） | 🟡 | 实例已暴露；未注册 CodeLens provider |
| Worktree 代码预览 | `frontend/src/app/worktree/page.tsx:974`（注释：真实编辑走 FileSession）、`:962`（历史 Agent Sessions 待接入） | 🟡 | 只读预览存在；FileSession 入口 ⬜ |
| 侧栏 | `frontend/src/components/Sidebar.tsx:118` | ✅ | `z-40` |
| 右侧聊天栏 | `frontend/src/components/chat/RightChatRail.tsx:96` | ✅ | `fixed top-0 right-0 z-40 h-screen` |
| 侧栏宽度变量 | `frontend/src/app/globals.css:56`、`:60`、`:66` | ✅ | 展开 256px、折叠 64px、移动端 0px |
| 群组聊天栏 | `frontend/src/app/worktree/[id]/group/page.tsx:3399` | 🟡 | 左偏移为 `calc(var(--sidebar-w,256px)_+_232px)`，232px 硬编码 |
| 终端堆叠组件的群组用法 | `frontend/src/app/worktree/[id]/group/page.tsx:469`（import）、`:3010`（使用） | ✅ | 与 FileSession 无关 |
| FileSession 底部 dock | 无（计划位于 01 §0.5.2 L3 层） | ⬜ | 代码中无 `z-[35]` 元素（`Sidebar.tsx:115` 仅注释） |
| Worktree 领域模型 | `crates/domain-worktree/src/lib.rs:189` | ✅ | 存在，未关联 FileSession |

## 6. 差距与后续

| 差距 | 影响 | 闭合步骤 |
|---|---|---|
| FileSession 无任何实现（crate、路由、CLI、存储、适配器） | 全部 FR-12-01 至 FR-12-26 无法验收 | 先完成归属裁决，再按 02 Phase 1 建立 crate 与存储接口 |
| 归属未裁决：Worktree 双入口与 Run/Lane 能力冲突 | UI 入口、API 位置与权限继承无法定型 | 按 INTEGRATION-STATUS.md 的 5 项裁决，同步需求、基本设计、详细设计后排期 |
| 状态命名冲突（01 §4.2 的 Active、Deleted 与 03 的 6 态） | 客户端与服务端状态解析不一致 | 以 6 态为准修订 01 §4.2 SessionRef.state |
| transcript 入 Git 的默认值冲突（ADR-0052 §4.1 示例与 §8 风险缓解） | 仓库膨胀与敏感信息入库风险 | 以默认不入 Git 为准修订 ADR-0052 §4.1 示例 |
| Windows 路径长度（260 字符）无解 | 深层文件路径下目录创建失败（主要运行平台为 Windows） | 在 ADR-0052 §3.1 写入路径缩短策略（SELF-REVIEW 建议 blake3 短名）；当前无对应文本，需定为正式决策 |
| 文件号 ADR-0052 与 ADR-0049 重号 | 交叉引用歧义 | 在 ADR 索引中重编号 |
| ADR-0045 引用的 `0045-agent-runtime.md` 不存在 | 引用断链 | 改为 `0045-star-agent-runtime-design.md` |
| Hermes ACP 复用前提未核实 | 适配器工作量与 FR-12-10 可行性不确定 | 核实 ACP 客户端所在模块，或记为新增工作 |
| 隐私规则引擎无承载 | FR-12-22 无法满足（P1） | 指定规则引擎归属（star-credential 之外新建或扩展），并写入详细设计 |
| 上下文构建组件缺失（treesitter-worktree-graph 不存在） | FR-12-18 无法按 01 复用 | 以 star-treesitter 为符号来源，补依赖图来源 |
| CLI Session 模型无文件路径绑定 | 无法直接复用 ULYS-156 实体 | 决定扩展该实体或另建 FileSession 实体（与归属裁决同步） |
| 权限键与 RLS 策略缺失 | 多租户隔离与 FR-COLLAB-03 无法验证 | 在 domain-permission 定义 `file_session:*` 键并写入 RLS 策略 |
| 审计与 hook 事件缺失 | FR-COLLAB-02 与生命周期事件不可用 | 在 domain-audit 与 domain-hook 定义 session 事件类型 |
| 认证中间件为直通（`auth_layer_stub`） | FileSession 上线前须具备真实认证 | 替换 `crates/star-api-rest/src/middleware/auth.rs:23-31` 的直通实现 |
| 底部 dock 与层级契约未落地；群组聊天栏 232px 硬编码 | INV-LAYOUT-03 仅部分满足 | 新建 dock 使用 `left: var(--sidebar-w)`；群组聊天栏改读变量 |
| 无 CodeLens provider 与菜单定位库 | FR-12-30、FR-12-31、FR-12-32 不可用 | 注册 CodeLens provider；采用 Floating UI 或同等定位库 |
| IDE 工作区与 FileSession 的边界未定义 | 文件级入口可能重复（`.nema`、IDE 文档、FileSession） | 在归属裁决中明确三者的职责与数据边界 |
| 适配器子进程 kill 未实现（graceful_shutdown.rs:23 留 P1） | agent 进程孤儿风险 | 完成 P1 实现后再接入适配器生命周期 |

## 7. 验收与测试

### 7.1 现有测试

| 测试 | 路径 | 覆盖 | 数量 |
|---|---|---|---|
| IdeFileWorkspace | `frontend/src/components/ide/IdeFileWorkspace.test.tsx` | 未接入降级、文件树、语义节点推导、Patch 空态、内核探测后的 open、sync、undo、outline | 19 个用例 |
| ide_endpoints | `crates/star-api-rest/tests/ide_endpoints.rs` | version 探测、open、insert、undo、redo、编辑端点 404、tabs、sync、export、mark_saved、outline | 9 个用例 |
| MonacoEditor | `frontend/src/components/editor/MonacoEditor.test.tsx:215` | `onEditorReady` 暴露底层实例（用例标题提及 FileSession CodeLens） | 21 个用例（it 计数） |

以上测试覆盖的是 IDE 相邻能力，不覆盖 FileSession。

### 7.2 验证范围

- 已验证（相邻能力）：IDE 工作区降级与内核连接路径；`/api/v1/ide` 的打开、编辑、撤销、重做、outline 行为。
- 未验证或不存在：FR-12-01 至 FR-12-36 无对应测试；01 §3 性能指标（创建 P95 小于 3 秒、恢复 P95 小于 2 秒、checkpoint P99 小于 500 毫秒）无测量；01 §10 完成定义（功能、性能、测试、文档、安全、示例）全部未满足；02 各 Phase 的验收命令（如 `star session start`、`star session list`）无法执行。

### 7.3 完成定义映射（01 §10）

| 维度 | 标准 | 状态 |
|---|---|---|
| 功能 | 全部 P0 需求通过验收 | ⬜ |
| 性能 | 达到 01 §3 基准 | ⬜ |
| 测试 | 单元覆盖 85% 以上，核心流程集成测试 100% | ⬜ |
| 文档 | CLI help、REST spec、ADR 落档 | 🟡（设计文档存在，见 §8） |
| 安全 | 无高危漏洞 | ⬜ |
| 示例 | README 含三个端到端示例 | ⬜ |

## 8. 来源与裁决

| 源文档 | 吸收内容 | 与代码或其他源的差异 | 裁决 |
|---|---|---|---|
| `docs/architecture/file-session-management/01-requirements.md` | FR-LIFECYCLE、FR-ADAPTER、FR-GIT、FR-CONTEXT、FR-COLLAB 需求与优先级；§0 UI 与布局；§1 概念与目录；§2 至 §10 | §4.2 SessionRef.state（Active、Paused、Archived、Deleted）与 §4.3 状态机不一致；§9 transcript 默认不入 Git 与 ADR-0052 §4.1 示例不一致；工作区比 HEAD 多 1 行 L4b（与 RightChatRail.tsx:96 一致） | 采用需求编号与优先级；状态取 6 态；transcript 默认不入 Git；采用 L4b 行 |
| `docs/architecture/file-session-management/02-implementation-plan.md` | 六个 Phase 的任务与验收；crate 布局；Rust 端口（Phase 2）；Schema；Next Actions | 全部未实现；Phase 5 目标文件 `group_api/file_sessions.rs` 不存在；`AgentAdapter` 签名与 03 §3.4、ADR-0052 §3.4 不一致 | `AgentAdapter` 采用 02 版本：它包含 capabilities、shutdown、health_check，是 04 `adapter list` 状态与降级模式所需；排期不采用（待归属裁决） |
| `docs/architecture/file-session-management/03-api-spec.md` | REST 资源模型；16 个编号端点；适配器、配置与 WebSocket 端点；错误格式；权限矩阵；速率限制 | 01 §5.2 的 14 行为其子集；03 的状态枚举为 6 态 | 采用为 REST 形态的权威来源 |
| `docs/architecture/file-session-management/04-cli-reference.md` | CLI 选项、输出、退出码、环境变量、补全、别名 | 比 01 §5.1 多 pause、rollback、promote、migrate、doctor；branch 子命令形式与 01 §5.1 的 flag 形式不同 | 采用 04 为 CLI 形态权威；01 的 flag 形式不采用 |
| `docs/architecture/file-session-management/INTEGRATION-STATUS.md` | 未裁决声明；五项必须裁决；API、CLI、plan 为草案 | 层级描述写作 Cloud Branch，与 AGENTS.md §0 的 Project → Lane → Worktree → Branch 略异 | 采用其状态结论；归属未裁决；层级以 AGENTS.md §0 为准 |
| `docs/architecture/file-session-management/README.md` | 文档导航；核心价值；Worktree 双入口概述；交互标准摘要 | 无额外设计；端点数量与 03 编号一致 | 仅用于导航，不新增需求 |
| `docs/architecture/file-session-management/SELF-REVIEW.md` | 缺口清单（Windows 路径、会话锁、adapter 进程、transcript 格式、token 计数、隐私热重载、MR 模板）；评分 4.65 | 声称 Windows 路径 hash 策略「已在 ADR-0052 补充」，但 ADR-0052 §3.1 及其他三份文档均无 blake3 或 hash 策略（grep 无命中） | 缺口清单列入 §6；评分不作为状态 |
| `docs/architecture/2026-08-26-upgrade/adr/0052-file-session-management.md` | 决策矩阵（§1）；架构（§2）；数据流（§2.2）；Schema（§3.2）；状态机（§3.3）；Adapter 契约（§3.4）；Git（§4）；集成（§5）；里程碑、NFR、风险、回滚 | §0.3.2 称 `Sidebar.tsx:135` 为 z-20，代码中为 `Sidebar.tsx:118` 的 z-40；§4.1 示例提交 transcript 与 §8 风险缓解冲突；文件号 0052 与 `0052-subtask-binding.md` 重号 | 决策矩阵作为设计意图；布局状态以代码为准；transcript 采用 §8 |
| `AGENTS.md`（D:/Star/AGENTS.md）§0 | 当前导航：Project → Lane → Worktree → Branch；Lane 管理本地 Worktree；Run 为 canonical scope | 与 INTEGRATION-STATUS.md 的 Cloud Branch 描述略异 | 作为当前层级权威 |
| `docs/architecture/2026-08-26-upgrade/adr/0049-task-card-auto-worktree-agent.md` | 关联（Task Card、Worktree、Agent） | 文件存在；正文未读；同号 `0049-ai-tool-auto-discovery.md` 并存 | 仅作关联引用，未采纳内容 |
| `docs/architecture/2026-08-26-upgrade/adr/0045-agent-runtime.md`（引用路径） | 关联（Agent Runtime） | 引用路径不存在；实际文件为 `0045-star-agent-runtime-design.md`；正文未读 | 引用修正为实际文件名，未采纳内容 |
| `docs/architecture/2026-08-26-upgrade/adr/0024-ide-session-identity.md` 与 `spec/resources/04-ide-session-identity.md` | 相邻（IDE 会话身份） | 存在；正文未读 | 仅作相邻引用；IDE 会话身份不等同 FileSession 身份 |
| `docs/architecture/2026-08-26-upgrade/adr/0043-audit-onboarding-failed.md` | FR-COLLAB-02 审计要求的引用 | 存在；正文未读 | 仅作引用 |
| `docs/architecture/2026-08-26-upgrade/adr/0052-subtask-binding.md` | 无 | 同号 ADR，仅核对文件名 | 不采纳；记录编号冲突 |
| 代码证据：`frontend/src/components/ide/IdeFileWorkspace.tsx`、`IdeFileWorkspace.test.tsx`、`frontend/src/hooks/useIdeKernel.ts`、`frontend/src/components/editor/MonacoEditor.tsx`、`MonacoEditor.test.tsx`、`frontend/src/app/worktree/page.tsx`、`frontend/src/app/(app)/kanban/page.tsx`、`frontend/src/lib/nema/nemaStore.ts`、`frontend/src/components/Sidebar.tsx`、`frontend/src/components/chat/RightChatRail.tsx`、`frontend/src/app/globals.css`、`frontend/src/app/worktree/[id]/group/page.tsx`、`frontend/package.json` | 当前状态与行号证据 | 无 | 作为状态裁决依据 |
| 代码证据：`crates/star-api-rest/src/routes/ide.rs`、`crates/star-api-rest/src/lib.rs`、`crates/star-api-rest/src/middleware/auth.rs`、`crates/star-api-rest/tests/ide_endpoints.rs`、`crates/star-api-rest/src/group_api/cli_sessions.rs`、`crates/ide-kernel/src/session.rs`、`crates/domain-local-runtime/src/cli_session.rs`、`crates/domain-local-runtime/src/graceful_shutdown.rs`、`crates/domain-context/src/lib.rs`、`crates/star-credential/src/lib.rs`、`crates/agent-bridge/src/lib.rs`、`crates/star-context-tiering/src/lib.rs`、`crates/domain-hook/src/lib.rs`、`crates/star-treesitter/src/lib.rs`、`crates/infrastructure/src/lib.rs`、`crates/infrastructure/src/registry.rs`、`crates/domain-worktree/src/lib.rs`、`crates/star-taskgraph/src/lib.rs`、`crates/star-cli/src/commands/`、`Cargo.toml` | 相邻能力与复用构件的状态证据 | 无 | 作为状态裁决依据 |
| `docs/reviews/VCD0-DD-COMPATIBILITY-20261024.md`（grep 命中） | 无 | 命中内容为 chat SSE 路由，与 FileSession 无关 | 未采纳 |
