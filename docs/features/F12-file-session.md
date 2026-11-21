# F12 文件会话管理（File Session）

> 状态摘要: FileSession 在代码基线 637ed19d 中没有实现：不存在 `star-file-session` crate、`/api/v1/file-sessions` 路由、`star session` CLI、`.star/sessions/` 存储与 AgentAdapter 契约。相邻能力（IDE1.0 文件编辑工作区、ULYS-156 CLI Session、Task Card 绑定的 CLI 会话、编辑器实例钩子、侧栏层级变量）已存在，但不承载 FileSession 语义，且挂载层级尚未裁决。

> 代码基线: 637ed19d; 核对方法: git reads at the pin (paths and grep); 核对日期 2026-11-19.

## 1. 目标与边界

### 1.1 目标

以文件为单位管理 AI agent 会话：

- 点击代码文件即开启一个绑定该文件的独立会话，用于探索、重构、调试与文档生成。会话以 Git 结构化存储于 `<repo>/.star/sessions/<相对文件路径>/<session_id>/`（01 §0.2、§1.1；ADR-0052 §0）。
- 会话生命周期：创建、列出、恢复、检查点、归档、删除、分叉（01 §2.1）。
- 多 agent CLI 适配：Claude Code、Codex、OpenCode、Hermes，以及通用 ACP 与插件机制（01 §2.2）。
- Git 集成：检查点自动提交、可选会话分支与 worktree 隔离、远程同步、MR 导出（01 §2.3）。
- 与 Task Card、Worktree、AgentSession 可选关联；权限使用 `file_session:*` 键并受租户 RLS 约束（03 权限矩阵）。
- UI 入口为文件右键菜单、编辑器 CodeLens 与底部终端 dock，并遵守侧栏层级契约（01 §0.3–§0.5）。

### 1.2 范围内 / 范围外

范围内：

- 会话实体、存储布局、状态机、CLI、REST 与适配器端口的形态。
- Git、上下文、隐私、权限、审计等横切要求。
- Worktree 页面的入口位置与布局层级契约。

范围外：

- IDE1.0 编辑内核（`crates/ide-kernel`、`/api/v1/ide`）。本文件只记录与 FileSession 的边界（§4.6），不定义其契约。
- ULYS-156 CLI Session（`crates/domain-local-runtime/src/cli_session.rs`）。FileSession 不复用其实体（§4.6）。
- 平台级 AgentSession 持久化（ADR-0045、ADR-0049）。仅以 `promote` 单向衔接，契约不在本文件定义（ADR-0052 §5.2）。
- 挂载层级。FileSession 挂在 Worktree 下、作为 Run/Lane 能力，或作为独立 App，尚未裁决（§4.1）。

## 2. 用户与场景

| 角色 | 主要场景 | 来源 |
|---|---|---|
| 开发者 | 编辑某文件时开启会话，做代码探索、重构实验、调试复现、单测编写 | 01 §0.3；ADR-0052 §0.1 |
| 开发者（多会话） | 同一文件并行多个独立会话，可 fork 与关联 | 01 §0.3（1 File → N FileSessions）；01 FR-COLLAB-04 |
| 代码评审者 | 将会话导出为 MR 描述，或评审会话产生的变更 | 01 FR-GIT-04；03 端点 15、16 |
| 团队成员（跨设备） | 通过 Git push/pull 同步会话历史 | 01 FR-GIT-03；04 `sync` |
| 平台管理员 | 配置 adapter、隐私规则、默认 agent 与 token 预算 | 03 权限矩阵（`file_session:admin`）；04 `config` |

## 3. 需求

优先级取自 01（P0/P1/P2）；无源优先级的 UI 设计项标 `—`。状态按代码基线判定。

| 编号 | 需求 | 优先级 | 状态 | 证据 |
|---|---|---|---|---|
| FR-12-01 | 创建 FileSession：`star session start` 与 `POST /api/v1/file-sessions`，返回 session_id 与工作区路径（FR-LIFECYCLE-01） | P0 | ⬜ | `crates/star-cli/src/commands/` 无 session 模块（现有模块：agent、code、context、issue、mr、pipeline、project、submit、task、test、workspace、worktree）；`crates/star-api-rest/src/lib.rs:121-133`、`:168-174` 的挂载列表无 file-sessions；crates 与 frontend/src 无 `file-sessions` 字符串 |
| FR-12-02 | 列出会话，支持按 file、agent、state、tag 过滤（FR-LIFECYCLE-02） | P0 | ⬜ | 同 FR-12-01 |
| FR-12-03 | 恢复会话，从最新或指定 checkpoint 恢复 transcript、context 与 agent 状态（FR-LIFECYCLE-03） | P0 | ⬜ | 无 FileSession 恢复实现。`CliSessionState::Restart`（`crates/domain-local-runtime/src/cli_session.rs:69`）属 ULYS-156，不对应 FileSession |
| FR-12-04 | 检查点：增量持久化上下文，可选自动 commit（FR-LIFECYCLE-04） | P1 | ⬜ | 无 FileSession checkpoint 结构或存储代码。crates 中同名 `CheckpointStore`（`crates/star-dispatcher/src/lib.rs:901`）职责不同 |
| FR-12-05 | 归档：只读化并压缩 transcript（FR-LIFECYCLE-05） | P2 | ⬜ | 无 FileSession 归档实现 |
| FR-12-06 | 删除：默认软删除到 `.star/sessions/.trash/`，`--force` 物理删除（FR-LIFECYCLE-06；03 端点 8） | P2 | ⬜ | 无 FileSession 删除实现 |
| FR-12-07 | Claude Code 适配器：spawn、attach、checkpoint、resume、export（FR-ADAPTER-01） | P0 | 🟡 | `crates/argo-core/src/model.rs:248-250` 构造 `claude -p … --output-format stream-json` 调用参数；缺 `trait AgentAdapter`（crates 与 frontend/src 无命中）；attach、checkpoint、resume、export 未实现；`invocation` 无生产调用方（仅 `model.rs:387` 测试调用）|
| FR-12-08 | Codex 适配器：`exec` 与 apply-patch 模式（FR-ADAPTER-02） | P0 | 🟡 | `crates/argo-core/src/model.rs:254-256` 构造 `codex exec --json …` 调用参数；缺同 FR-12-07 的生命周期方法；无生产调用方（仅 `model.rs:387` 测试）|
| FR-12-09 | OpenCode 适配器（FR-ADAPTER-03） | P1 | ⬜ | `crates/argo-core/src/model.rs:11-12` 的 `SUPPORTED` 列表不含 opencode；无适配器代码 |
| FR-12-10 | Hermes 适配器，复用 ACP/ACL 基础设施（FR-ADAPTER-04） | P1 | ⬜ | `crates/argo-core/src/model.rs:261-263` 的 hermes 分支为 `hermes chat --query-file …`，未使用 ACP；`git grep -w ACP` 仅命中 `crates/star-registry/src/lib.rs:48` 注释；`AgentRuntime` trait（`crates/agent-bridge/src/runtime.rs:227`）不含 ACP 实现 |
| FR-12-11 | 通用 ACP 协议适配器（FR-ADAPTER-05） | P2 | ⬜ | 同 FR-12-10，无 ACP 实现 |
| FR-12-12 | 自定义 agent 插件机制，WASM 或动态库加载（FR-ADAPTER-06） | P2 | ⬜ | 未发现 AgentAdapter 注册或动态加载代码（检索范围同 FR-12-07） |
| FR-12-13 | checkpoint 自动 `git add` 与 commit，message 为 `chore(session): <file> checkpoint <n>`（FR-GIT-01） | P0 | ⬜ | `chore(session)` 字符串在 crates 与 frontend/src 无命中 |
| FR-12-14 | 每 session 可选绑定 `session/<file-hash>/<session-id>` 分支，并以 worktree 隔离（FR-GIT-02） | P1 | ⬜ | 无 session 分支逻辑。`Worktree` 实体存在（`crates/domain-worktree/src/lib.rs:189`；`crates/star-taskgraph/src/lib.rs:112`），无 session 关联字段 |
| FR-12-15 | `star session sync`：push 与 pull 全部 session 历史（FR-GIT-03） | P1 | ⬜ | 无 FileSession 同步实现 |
| FR-12-16 | MR 导出与创建：`export --format mr`、`mr` 命令（FR-GIT-04；03 端点 15、16） | P2 | ⬜ | `crates/star-cli/src/commands/mr.rs` 存在但不引用 session（grep 无命中） |
| FR-12-17 | 同一文件多 session 并发编辑的冲突解决（FR-GIT-05） | P2 | ⬜ | 无 FileSession 版本号或文件锁代码 |
| FR-12-18 | 初始上下文：文件内容、符号、依赖图、最近 git log（FR-CONTEXT-01） | P0 | ⬜ | 可复用 `parse_rust`、`parse_typescript`（`crates/star-treesitter/src/lib.rs:248`、`:253`）；无 ContextBuilder 实现（`crates/domain-llm/src/context.rs:198` 的同名 trait 为 RAG 检索，职责不同）；01 指定的 `treesitter-worktree-graph` crate 不存在 |
| FR-12-19 | 增量 checkpoint：仅存增量，压缩率大于 80%（FR-CONTEXT-02） | P0 | ⬜ | 无 checkpoint 编码或 delta 代码 |
| FR-12-20 | `@import` 语法引用同仓库其他文件的会话上下文（FR-CONTEXT-03） | P1 | ⬜ | 无 |
| FR-12-21 | token 预算：可配置 max_tokens，超限自动摘要或截断（FR-CONTEXT-04） | P1 | ⬜ | `ContextPacket.token_budget`（`crates/domain-context/src/lib.rs:182`、`:230`）存在，FileSession 未使用；未发现 token 计数依赖 |
| FR-12-22 | 隐私过滤：脱敏 secrets、PII、内部路径（FR-CONTEXT-05） | P1 | ⬜ | `crates/star-credential/src/lib.rs:1-15` 仅含 KMS 凭证管理，无规则引擎；crates 中 privacy、redact 命中均为无关上下文 |
| FR-12-23 | 多用户协作，OT 或 CRDT（FR-COLLAB-01） | P2 | ⬜ | 无 |
| FR-12-24 | 审计日志：谁、何时、做了什么，复用 ai-audit schema（FR-COLLAB-02） | P1 | ⬜ | `crates/domain-audit/src/lib.rs:124` 的 `AuditAction` 无 FileSession 变体；`Actor` 枚举（`:183`）的 `Agent` 变体携带 `AgentSessionId`（`:189-195`）；无 `ai_audit_metadata` |
| FR-12-25 | 权限键 `file_session:*`，继承 Worktree 的 Owner、Editor、Viewer（FR-COLLAB-03；03 权限矩阵） | P1 | ⬜ | `crates/domain-permission` 为 workspace 成员（`Cargo.toml:24`）；crates 无 `file_session:` 字符串 |
| FR-12-26 | 会话分叉 fork，继承历史与共同祖先（FR-COLLAB-04；03 端点 9） | P2 | ⬜ | 无 |
| FR-12-27 | 与 Task Card 可选双向关联（`linked_task_card_id`）（ADR-0052 §1 决策 6；01 §4.2） | — | ⬜ | crates 与 frontend/src 无 `linked_task_card` 字段。Task Card 绑定的 CLI 会话已存在（`crates/star-api-rest/src/group_api/cli_sessions.rs:459-467`），但不是 FileSession 关联 |
| FR-12-28 | 与 Worktree、AgentSession 可选关联（`linked_worktree_id`、`linked_agent_session_id`）（01 §4.2） | — | ⬜ | 无对应字段 |
| FR-12-29 | Worktree 详情页并列展示 Task Cards 与 File Sessions 两个入口（01 §0.3；ADR-0052 §0.1） | — | ⬜ | `frontend/src/app/worktree/[id]/` 下仅 `group` 目录；`frontend/src/app/worktree/[id]/group/page.tsx` 无 FileSession 字样；`frontend/src/app/worktree/page.tsx:962` 文案为「历史 Agent Sessions 待接入」 |
| FR-12-30 | 文件树右键菜单含 “Start File Session” 等项（01 §0.4） | — | ⬜ | `frontend/src/` 无 “Start File Session” 字符串，无 ContextMenu 组件 |
| FR-12-31 | 编辑器 CodeLens 显示 “N sessions”，悬停展开会话列表（01 §0.4） | — | 🟡 | `frontend/src/components/editor/MonacoEditor.tsx:105`（`onEditorReady` 属性）与 `:380`（调用）已暴露底层实例；未注册任何 CodeLens provider，`CodeLens` 仅出现在注释（`MonacoEditor.tsx:104`、`:242`、`:379`） |
| FR-12-32 | 菜单交互：跟随鼠标定位、子菜单安全三角区、Floating UI 定位、会话列表超 100 项虚拟化（01 §0.4） | — | ⬜ | `frontend/package.json` 无 `@floating-ui` 或 popper 依赖；无 FileSession 列表虚拟化 |
| FR-12-33 | 底部 dock 终端面板：`z-[35]`，`left: var(--sidebar-w)`，被侧栏覆盖（01 §0.5.2、§0.5.3；INV-LAYOUT-01、02） | — | ⬜ | 无 `z-[35]` 元素；`frontend/src/components/Sidebar.tsx:115` 仅注释提及 |
| FR-12-34 | Sidebar 为结构层，`z-40` 高于内容层与 dock（INV-LAYOUT-04） | — | 🟡 | `frontend/src/components/Sidebar.tsx:118`（class 含 `sticky top-0 h-screen … z-40`）；`frontend/src/components/chat/RightChatRail.tsx:96` 同为 `z-40`，未低于侧栏；dock 不存在（见 FR-12-33），"高于 dock" 无法验证 |
| FR-12-35 | 侧栏宽度为单一事实源 CSS 变量，所有 fixed 元素引用（INV-LAYOUT-03） | — | 🟡 | 变量定义于 `frontend/src/app/globals.css:56`、`:60`、`:66`；群组聊天栏读取变量（`frontend/src/app/worktree/[id]/group/page.tsx:3399`），但 232px 偏移仍硬编码 |
| FR-12-36 | 布局与交互验收用例（01 §0.4 五条、§0.5.4 六条）的自动化断言 | — | ⬜ | 检索 frontend/src 与 frontend/e2e 的测试文件，未发现 z-index 或 dock 层级断言；唯一 computed 断言为 `frontend/e2e/editor-sakura-parity.spec.ts:118-139`，覆盖编辑器主题 |
| FR-12-37 | 与 VS Code 或 Web IDE 集成：CodeLens、Session Panel、Diff View（01 §5.3；02 Phase 5.3） | — | ⬜ | 仓内无 vscode 扩展目录（`git ls-tree -r` 无命中）；无 FileSession 页面路由 |

## 4. 设计

### 4.1 概念模型

挂载层级（未裁决）：ADR-0052 §0.1 与 01 §0.3 将 FileSession 置于 Worktree 之下，与 Task Card 同级。INTEGRATION-STATUS.md 将该陈述列为实施前须裁决的事项，并指出它与 `AGENTS.md:22` 的当前规范 Project → Lane → Worktree → Branch 不一致。挂载候选为三者之一：Run 或 Lane 的能力、Worktree 文件视图中的操作、或独立可插拔 App。在裁决前，下表的概念定义与挂载位置无关。

| 概念 | 定义 | 来源 | 状态 |
|---|---|---|---|
| FileSession | 绑定单个仓库相对文件路径的 AI agent 会话 | 01 §1 | ⬜ |
| Session Workspace | 会话持久化目录 `<repo>/.star/sessions/<file>/<session_id>/` | 01 §1.1；ADR-0052 §3.1 | ⬜ |
| Session Manifest | 单文件所有会话的摘要索引 `manifest.json` | 01 §1；ADR-0052 §3.2 | ⬜ |
| Session Registry | 仓库级聚合索引 `registry.json`，Git 跟踪，SQLite 为辅助索引 | ADR-0052 §1 决策 1 | ⬜ |
| Checkpoint | 增量上下文快照，含 context delta、transcript 范围与 artifact 引用 | 01 §2.4；02 Phase 3 | ⬜ |
| Transcript | 对话记录，索引加分块存储，支持随机访问 | 01 §4.2；02 Phase 3 Day 5 | ⬜ |
| Artifact | patch、diff、export 等产出物 | 01 §1.1 | ⬜ |
| Agent Adapter | 统一 AI CLI 交互的抽象端口 | 01 §2.2；ADR-0052 §3.4 | ⬜ |
| Session Links | task_card、worktree、agent_session、parent、child 与 related 关联 | 01 §4.2；03 资源模型 | ⬜ |

### 4.2 数据模型

存储布局（采用 ADR-0052 §3.1，与 01 §1.1 一致）：

```
<repo>/.star/sessions/
  registry.json                  仓库级索引
  config.toml                    FSMS 配置
  <相对文件路径>/
    manifest.json                单文件会话摘要
    <session_id>/
      session.json               元数据与索引
      context/initial.json
      context/checkpoints/NNN.json
      transcript/index.json
      transcript/NNNN-NNNN.msgpack   默认不入 Git
      artifacts/patches、diffs、exports
      git/                       branch、commits、mr-links
      state/                     adapter 状态快照
```

实体与字段：

| 实体 | 字段 | 来源 | 状态 |
|---|---|---|---|
| SessionRegistry | version、updated_at、sessions（SessionRef 列表） | 01 §4.2 | ⬜ |
| SessionRef | session_id、file_path、agent_type、state、created_at、updated_at、tags、linked_task_card_id、linked_worktree_id、linked_agent_session_id | 01 §4.2 | ⬜ |
| FileSessionManifest | file_path、version、updated_at、sessions（SessionSummary 列表） | 01 §4.2；ADR-0052 §3.2 | ⬜ |
| SessionSummary | session_id、agent_type、state、title、created_at、updated_at、checkpoint_count、token_usage（input、output、total）、git_branch | 01 §4.2；ADR-0052 §3.2 | ⬜ |
| SessionMeta | session_id、file_path、agent_type、agent_version、adapter_config、state、created_at、updated_at、tags、links | 01 §4.2；03 资源模型 | ⬜ |
| SessionLinks | task_card_id、worktree_id、agent_session_id、parent_session_id、child_session_ids、related_sessions（类型为 depends、relates、blocks） | 01 §4.2；03 资源模型 | ⬜ |
| FileSession 详情 | meta、context（initial_ref、checkpoints、current_checkpoint、token_budget、token_used）、transcript（index_ref、chunks、total_messages）、artifacts、git（branch、worktree_path、commits、mr_links）、state_snapshots | 03 资源模型 | ⬜ |
| Checkpoint | number、context_delta（added、modified、removed）、transcript_range、artifacts、agent_state、token_delta、created_at | 02 Phase 3 Day 3-4 | ⬜ |

持久化规则：

- `state` 取值采用 03 资源模型的六态：created、running、paused、completed、cancelled、archived（§4.4）。
- transcript 默认不入 Git，仅本地保存，或在配置的 artifact store 中保存（01 §9；ADR-0052 §8）。
- 目录名按 ADR-0052 §3.1 使用相对文件路径。Windows 路径长度的缩短策略尚未写入正式决策（§6）。
- `.star/` 下已有提交流程使用的文件（`workspace.json`、`issue-link.json`、`agent-state.json`、`ide-session-state.json`，`crates/star-cli/src/commands/submit.rs:12-22`），FileSession 的目录与文件名须与之区分。

### 4.3 接口

CLI（命令清单取自 01 §5.1 并以 04 补充；形态以 04 为准）：

| 命令组 | 命令 | 状态 |
|---|---|---|
| 会话 | `start`、`list`、`show`、`resume`、`checkpoint`、`archive`、`delete`、`fork`、`link` | ⬜ |
| 会话扩展（仅 04） | `pause`、`rollback`、`promote`、`migrate`、`doctor` | ⬜ |
| Git | `sync`、`branch`（子命令 list、create、switch、delete、merge）、`export`、`mr` | ⬜ |
| 适配器 | `adapter list`、`adapter add`、`adapter remove`、`adapter configure` | ⬜ |
| 配置 | `config show`、`config set`、`config reset` | ⬜ |
| 全局选项与环境变量 | `--json`、`--repo`、`STAR_SESSION_REPO`、`STAR_SESSION_DEFAULT_AGENT` 等（04） | ⬜ |
| 退出码 | 0 至 10（04 退出码） | ⬜ |

REST（基路径 `/api/v1/file-sessions`，端点编号取自 03）：

| 路径 | 方法与端点编号 | 状态 |
|---|---|---|
| `/` | POST 创建（1）、GET 列表（2） | ⬜ |
| `/{session_id}` | GET 详情（3）、PATCH 更新（4）、DELETE 删除（8） | ⬜ |
| `/{session_id}/checkpoint`、`/resume`、`/archive`、`/fork`、`/link` | POST（5、6、7、9、10） | ⬜ |
| `/{session_id}/transcript`、`/artifacts` | GET（11、12） | ⬜ |
| `/{session_id}/branch` | GET 列出、POST 创建或切换（14） | ⬜ |
| `/{session_id}/sync`、`/export`、`/mr` | POST（13、15、16） | ⬜ |
| `/adapters`、`/adapters/{adapter_name}` | GET、PATCH | ⬜ |
| `/config` | GET、PATCH | ⬜ |
| `/{session_id}/ws` | GET，WebSocket，可选 | ⬜ |

端口（方法集取自 ADR-0052 §3.4，与 02 Phase 1–3 的端口签名一致）：

| 端口 | 方法 | 同名符号（637ed19d） | 状态 |
|---|---|---|---|
| AgentAdapter | agent_type、version、supported_models、capabilities、spawn、attach、checkpoint、resume、shutdown、export、health_check | 无 `trait AgentAdapter`。`infrastructure` 的 `AdapterRegistry`（`crates/infrastructure/src/lib.rs:79`）为平台侧适配器注册，语义不同 | ⬜ |
| StorageBackend | 会话、registry、manifest 的读写 | 无 | ⬜ |
| GitManager | init_session、commit_checkpoint、commit_archive、create_worktree、sync、export_mr | 无 | ⬜ |
| ContextBuilder | build_initial | 无 FileSession 实现；`crates/domain-llm/src/context.rs:198` 为同名 RAG 检索 trait，职责不同 | ⬜ |
| TokenBudgetManager | check_and_maybe_summarize | 无 | ⬜ |

相邻已实现接口（非 FileSession）：

| 接口 | 证据 | 状态 |
|---|---|---|
| IDE1.0 内核 `/api/v1/ide/*` | 路由 `crates/star-api-rest/src/routes/ide.rs:126-148`；挂载 `crates/star-api-rest/src/lib.rs:129`、`:172` | ✅ |
| Task Card 绑定的 CLI 会话 `/api/v1/worktrees/{worktree_id}/work-items/{work_item_id}/cli-sessions` | `crates/star-api-rest/src/group_api/cli_sessions.rs:459-467`；handler `:471`、`:514` | ✅ |
| Agent 运行时 trait | `crates/agent-bridge/src/runtime.rs:227` | ✅ |

### 4.4 状态机与关键流程

状态机采用六态（03 资源模型 `state` 枚举；01 §4.3 与 ADR-0052 §3.3 图示与之一致）：

```
created   --spawn-->               running
created   --cancel-->              cancelled
running   --checkpoint-->          running      (自环)
running   --pause-->               paused
paused    --resume-->              running
running   --cancel-->              cancelled
paused    --cancel-->              cancelled
running   --agent 正常退出-->      completed
completed / cancelled --archive--> archived     (终态，只读)
```

01 §4.3 与 ADR-0052 §3.3 的图示含 `cancelling` 中间态；03 枚举不含该值。是否持久化该中间态见 §6。

| 迁移 | 前置 | 状态 |
|---|---|---|
| created → running | AgentAdapter.spawn 成功（01 §4.3） | ⬜ |
| running ↔ paused | pause 与 resume（04 `pause` 等同于 state 更新为 paused） | ⬜ |
| → archived | 处于 completed 或 cancelled；压缩 transcript（04 `archive`） | ⬜ |

关键流程：

- 创建（ADR-0052 §2.2；02 Phase 1 Week 2 的 `init_session`）：分配 session_id；ContextBuilder 构建初始上下文（文件内容、符号、依赖、最近 git log、隐私过滤）；StorageBackend 创建工作区与 manifest；GitManager 初始化提交；AgentAdapter.spawn。全部 ⬜。
- 检查点（01 FR-LIFECYCLE-04、FR-GIT-01；ADR-0052 §4.1）：写入增量 context delta、transcript 范围与 artifact 引用，可选自动 commit，message 模板为 `chore(session): {file} checkpoint {num} [{session_id:8}]`。⬜。
- 恢复（01 FR-LIFECYCLE-03）：从最新或指定 checkpoint 重建上下文；恢复后的 token 使用量应低于初始上下文的 30%（01 FR-LIFECYCLE-04 验收）。⬜。
- 并发（02 Phase 4 Day 5）：session 级乐观锁（version CAS）加文件锁 `.star/sessions/<file>/.lock`。⬜。

相邻状态机（不属于 FileSession）：ULYS-156 的 `CliSessionState` 为 7 态，含 Restart 与 Orphaned（`crates/domain-local-runtime/src/cli_session.rs:61-75`），`CliSession` 绑定 tenant_id 与 worktree_id，不含文件路径（`:104-125`）。

### 4.5 权限与安全

| 项 | 设计要求 | 现状 | 证据 |
|---|---|---|---|
| 权限键 | `file_session:create`、`read`、`write`、`delete`、`archive`、`fork`、`link`、`sync`、`export`、`mr`、`admin`，默认角色见 03 权限矩阵 | ⬜ | crates 无 `file_session:` 字符串；`domain-permission` 为 workspace 成员（`Cargo.toml:24`） |
| 租户隔离 | PostgreSQL RLS，`WHERE tenant_id = current_setting('app.tenant_id')` | ⬜ | 同一会话变量模式已在 `crates/bi-owner-runtime/src/task_event_query.rs:247` 使用；FileSession 表与策略未定义 |
| 认证 | Bearer JWT（03 通用规范） | ⬜ | `crates/star-api-rest/src/middleware/auth.rs:23-32` 为 pass-through stub；挂于 `/api/v1` 外层（`crates/star-api-rest/src/lib.rs:137-138`） |
| 隐私过滤 | 默认开启；transcript 默认不入 Git；导出默认脱敏（01 FR-CONTEXT-05；03 端点 15） | ⬜ | `crates/star-credential/src/lib.rs:1-15` 仅 KMS 凭证管理，无规则引擎 |
| 凭证注入 | `SpawnContext.env` 注入 provider 凭证（ADR-0052 §3.4） | ⬜ | 无 SpawnContext 类型，无注入路径 |
| 审计 | 复用 ai-audit 记录操作（01 FR-COLLAB-02） | ⬜ | `crates/domain-audit/src/lib.rs:124`、`:189-192`，无 FileSession 事件 |

相邻安全边界：IDE 的磁盘读仅在显式设置 `IDE_PROJECT_ROOT` 时启用（`crates/star-api-rest/src/routes/ide.rs:40`、`:178`）。该门控未被 FileSession 继承。

### 4.6 依赖关系

| 依赖 | 设计用途 | 代码状态 | 证据 |
|---|---|---|---|
| star-context-tiering | 上下文分层（FR-12-18） | 🟡 crate 存在，描述 L0/L1/L2 agent 上下文；除自身测试外无调用方 | `crates/star-context-tiering/src/lib.rs:2-7`；`crates/star-context-tiering/tests/tiering_3layers_test.rs:4` |
| treesitter-worktree-graph | 符号索引与依赖图 | ⬜ crate 不存在；可用替代为 star-treesitter | `crates/star-treesitter/src/lib.rs:248`、`:253` |
| star-credential | 隐私规则与凭证注入（FR-12-22） | ⬜ 无规则引擎；KMS 凭证管理存在 | `crates/star-credential/src/lib.rs:1-15` |
| domain-context（ContextPacket） | 初始上下文与 token 预算（FR-12-21） | 🟡 类型存在，FileSession 未使用 | `crates/domain-context/src/lib.rs:182`、`:230` |
| domain-hook | 生命周期事件 `session.created`、`session.checkpoint`、`session.archived` | ⬜ crate 存在（`Cargo.toml:38`），但 `crates/domain-hook/src/lib.rs` 无 session 字样 | `crates/domain-hook/src/lib.rs` |
| domain-audit | 审计（FR-12-24） | 🟡 AuditAction 框架存在，无 FileSession 事件 | `crates/domain-audit/src/lib.rs:124`、`:189-192` |
| domain-permission | 权限键与继承（FR-12-25） | ⬜ crate 存在（`Cargo.toml:24`），无 `file_session:` 键 | `Cargo.toml:24` |
| agent-bridge（AgentRuntime） | Hermes 与 adapter 参考（FR-12-10） | 🟡 trait 存在，ACP 未实现 | `crates/agent-bridge/src/runtime.rs:227` |
| argo-core（invocation） | 各 CLI 的调用参数构造（FR-12-07、FR-12-08） | 🟡 构造器存在，`invocation` 无生产调用方（仅 `model.rs:387` 测试）；`argo_core::SUPPORTED` 被 `routes/integrations.rs:143` 使用；文件头记录复用许可决定 | `crates/argo-core/src/model.rs:2-4`、`:11-13`、`:242-263`；`crates/star-api-rest/src/routes/integrations.rs:143` |
| domain-local-runtime（ULYS-156） | 状态机与进程管理参考 | 🟡 7 态状态机存在；子进程 kill 留待 P1 | `crates/domain-local-runtime/src/cli_session.rs:61-75`、`:104-125`；`crates/domain-local-runtime/src/graceful_shutdown.rs:23` |
| Worktree 实体 | 归属与分支管理（FR-12-14） | 🟡 两处实体并存（star-taskgraph 与 domain-worktree），归属未裁决 | `crates/star-taskgraph/src/lib.rs:112`；`crates/domain-worktree/src/lib.rs:189` |
| infrastructure（AdapterRegistry） | 平台侧适配器注册 | 🟡 与 FileSession 适配器无关 | `crates/infrastructure/src/lib.rs:79`、`:273-289` |
| API 挂载点 | `/api/v1` 下的 FileSession 路由 | 🟡 挂载点存在，无 FileSession 模块 | `crates/star-api-rest/src/lib.rs:137` |
| 当前层级规范 | Lane 与 Worktree 的关系 | ✅ 文本存在 | `AGENTS.md:22` |

4.7 页面设计: 本功能无独立页面，入口见 Worktree 页面（`frontend/src/app/worktree/page.tsx`）与看板「按文件开卡」页签中的 IDE 文件编辑工作区（`frontend/src/components/ide/IdeFileWorkspace.tsx`）；两处 FileSession 入口均未实现。

## 5. 当前实现

| 模块 | 路径 | 状态 | 证据 |
|---|---|---|---|
| FileSession 核心 crate | `crates/star-file-session/` | ⬜ | 路径不存在（`git ls-tree 637ed19d crates/`） |
| FileSession REST 模块 | `crates/star-api-rest/src/group_api/file_sessions.rs` | ⬜ | 文件不存在；`crates/star-api-rest/src/group_api.rs:208` 仅挂载既有的 `cli_sessions` 等模块 |
| `/api/v1/file-sessions` 路由 | `crates/star-api-rest/src/lib.rs:137` | ⬜ | `/api/v1` 下的挂载列表（`lib.rs:121-133`、`:168-174`）无 file-sessions |
| `star session` CLI | `crates/star-cli/src/commands/` | ⬜ | 目录无 session 模块 |
| `.star/sessions/` 存储 | 无 | ⬜ | `crates/star-cli/src/commands/submit.rs:12-22` 使用 `.star/` 下其他文件（见 §4.2） |
| AgentAdapter 与插件注册 | 无 | ⬜ | `trait AgentAdapter` 无命中 |
| Claude、Codex、Hermes 调用参数构造 | `crates/argo-core/src/model.rs:242-263` | 🟡 | `claude -p … --output-format stream-json`（`:250`）、`codex exec --json …`（`:256`）、`hermes chat --query-file …`（`:263`）；无 spawn、attach、checkpoint、resume、export 生命周期；无生产调用方（`invocation` 仅 `model.rs:387` 测试引用） |
| 平台侧适配器注册 | `crates/infrastructure/src/lib.rs:79`、`:273-289` | ✅ | `AdapterRegistry` trait 与 `RegisterAgentAdapterCmd`，属平台侧适配器，与 CLI agent 适配器无关 |
| ULYS-156 CLI 会话状态机 | `crates/domain-local-runtime/src/cli_session.rs:61-75`、`:104-125` | ✅ | 7 态 `CliSessionState`；`CliSession` 含 tenant_id（`:108`）、worktree_id（`:110`）、state_history（`:125`），无文件路径与 task_card 字段 |
| Task Card 绑定的 CLI 会话 REST | `crates/star-api-rest/src/group_api/cli_sessions.rs:459-467`、`:471`、`:514` | ✅ | 路由按 worktree_id 与 work_item_id 绑定，与 FileSession 无关联 |
| IDE1.0 内核 HTTP | `crates/star-api-rest/src/routes/ide.rs:126-148` | ✅ | 文档、大纲、SCM 路由，挂载于 `/api/v1/ide`（`lib.rs:129`、`:172`）；内核状态为内存态 |
| IDE1.0 内核会话类型 | `crates/ide-kernel/src/session.rs:13`、`:24-26` | ✅ | `TabState` 与 `Session`；`docs` 为 `HashMap`（内存） |
| 文件编辑工作区 | `frontend/src/components/ide/IdeFileWorkspace.tsx:170`、`:232`、`:25-31` | ✅ | 未接入内核时降级（`:25-31` 注释）；选中文件调用 `kernel.openFile`（`:232`） |
| 内核 hook | `frontend/src/hooks/useIdeKernel.ts:183` | ✅ | `useIdeKernel` |
| 看板「按文件开卡」 | `frontend/src/app/(app)/kanban/page.tsx:68`、`:448`、`:452` | ✅ | 视图页签与 IdeFileWorkspace 挂载，数据源为 `nemaTasks` |
| `.nema` 文件即任务 | `frontend/src/lib/nema/nemaStore.ts:188`、`:250` | ✅ | `nemaTasks` 与 `useNemaStore`，不创建 agent 会话 |
| 编辑器实例钩子 | `frontend/src/components/editor/MonacoEditor.tsx:104-105`、`:380` | 🟡 | `onEditorReady` 暴露实例；未注册 CodeLens provider |
| Worktree 文件预览 | `frontend/src/app/worktree/page.tsx:962`、`:974` | 🟡 | 只读预览；`:962` 为「历史 Agent Sessions 待接入」；`:974` 注释称真实编辑走 FileSession，入口未实现 |
| Worktree 详情页入口 | `frontend/src/app/worktree/[id]/group/page.tsx` | ⬜ | `frontend/src/app/worktree/[id]/` 下仅 `group`；该页无 FileSession 入口 |
| 文件右键菜单 | 无 | ⬜ | `frontend/src/` 无 ContextMenu 组件与 “Start File Session” 字符串 |
| 侧栏 | `frontend/src/components/Sidebar.tsx:118` | ✅ | `sticky top-0 h-screen … z-40` |
| 右侧聊天栏 | `frontend/src/components/chat/RightChatRail.tsx:96` | ✅ | `fixed top-0 right-0 z-40 h-screen` |
| 布局根与内容列 | `frontend/src/app/layout.tsx:76-85` | ✅ | `flex min-h-screen`，Sidebar 与内容列为兄弟节点（`:76-78`）；内容列 `relative` 无 z-index（`:81`）；高度与滚动（`:84-85`） |
| 侧栏宽度变量 | `frontend/src/app/globals.css:56`、`:60`、`:66` | ✅ | 展开 256px、折叠 64px、移动端 0px |
| 群组聊天栏 | `frontend/src/app/worktree/[id]/group/page.tsx:3399` | 🟡 | 左偏移读取 `--sidebar-w`，232px 偏移硬编码 |
| 底部 dock 终端 | 无 | ⬜ | 无 `z-[35]` 元素（`Sidebar.tsx:115` 仅注释） |
| Worktree 实体 | `crates/domain-worktree/src/lib.rs:189`；`crates/star-taskgraph/src/lib.rs:112` | ✅ | 两处 Worktree 结构并存，均无 FileSession 关联 |
| 上下文类型 | `crates/domain-context/src/lib.rs:182`、`:230` | ✅ | `ContextPacket` 与 `token_budget`，FileSession 未使用 |
| 符号解析 | `crates/star-treesitter/src/lib.rs:248`、`:253` | ✅ | `parse_rust`、`parse_typescript` |
| 审计类型 | `crates/domain-audit/src/lib.rs:124`、`:189-192` | ✅ | `AuditAction` 与 `Actor::Agent` 变体（携带 `AgentSessionId`，`:183-195`）；无 FileSession 事件 |
| Agent 运行时 trait | `crates/agent-bridge/src/runtime.rs:227` | ✅ | `AgentRuntime` trait；未含 ACP 实现 |
| 同名 RAG 检索 trait | `crates/domain-llm/src/context.rs:198` | ✅ | 与 FileSession 的 ContextBuilder 同名、职责不同 |

## 6. 差距与后续

| 差距 | 影响 | 闭合它的下一步 |
|---|---|---|
| FileSession 无实现（crate、路由、CLI、存储、适配器、UI 均缺） | FR-12-01 至 FR-12-26 无法验收 | 完成挂载裁决后，按 02 Phase 1 建立 `crates/star-file-session` 与存储端口 |
| 挂载层级未裁决：Worktree 双入口与 `AGENTS.md:22` 的 Project → Lane → Worktree → Branch 不一致，Run 与 Lane 能力未定 | UI 入口、API 位置与权限继承无法定型 | 按 INTEGRATION-STATUS.md 的五项裁决，同步更新 01、基本设计与详细设计 |
| 状态集合不一致：01 §4.2 为 Active、Paused、Archived、Deleted，ADR-0052 §3.2 示例为 `active`，03 为六态 | 客户端与服务端状态解析不一致 | 统一为 03 六态，修订 01 §4.2 与 ADR-0052 §3.2 示例 |
| `cancelling` 中间态在 01 §4.3 与 ADR-0052 §3.3 图示中出现，未列入 03 枚举 | 取消在持久化与恢复时语义不确定 | 裁决中间态是否持久化；若持久化则补入枚举 |
| transcript 默认入 Git 的示例与风险缓解矛盾（ADR-0052 §4.1 示例提交 transcript；§8 缓解为不入 Git） | 仓库膨胀与敏感信息入库 | 以默认不入 Git 修订 ADR-0052 §4.1 示例 |
| Windows 路径长度（260）无正式决策；SELF-REVIEW.md:106 提出 blake3 短名，ADR-0052 与其他 folder 文档均无 | 深层文件路径下目录创建失败，主要运行平台为 Windows | 在 ADR-0052 §3.1 写入路径缩短决策 |
| ADR 号 0052 与 `0052-subtask-binding.md` 重号；01 §11.2 引用的 `0045-agent-runtime.md` 不存在 | 交叉引用歧义与断链 | 重新编号；引用改为 `0045-star-agent-runtime-design.md` |
| Hermes ACP 复用前提未满足：agent-bridge 无 ACP；argo-core 的 hermes 分支使用 `hermes chat --query-file`（`model.rs:263`） | FR-12-10 工作量不确定 | 核实 ACP 客户端所在模块，或记为新增工作 |
| 适配器无契约代码；argo-core 调用参数构造无调用方 | FR-12-07、FR-12-08 无法执行生命周期方法 | 定义 AgentAdapter 端口；决定 argo-core 构造器是否纳入，并按其文件头许可决定处理 |
| ULYS-156 CliSession 无文件路径与 task_card 绑定（`cli_session.rs:104-125`） | 无法直接复用该实体 | 决定扩展该实体或另建 FileSession 实体，与挂载裁决同步 |
| 隐私规则引擎无承载（FR-12-22） | P1 无法满足 | 指定规则引擎归属并写入详细设计 |
| 上下文构建依赖的 `treesitter-worktree-graph` 不存在；`star-context-tiering` 无调用方 | FR-12-18 无法按 01 复用 | 以 star-treesitter 为符号来源，补齐依赖图来源，并决定是否接入 star-context-tiering |
| 权限键与 RLS 策略缺失（domain-permission 无 `file_session:*`） | 多租户隔离与 FR-12-25 无法验证 | 在 domain-permission 定义键，并写入 RLS 策略 |
| 审计与 hook 事件缺失（domain-hook 无 session 事件；AuditAction 无 FileSession 变体） | FR-12-24 与生命周期事件不可用 | 在 domain-audit、domain-hook 中定义事件类型 |
| 认证为 pass-through stub（`middleware/auth.rs:23-32`；`lib.rs:138`） | FileSession 路由上线前不具备真实身份 | 替换 stub 为真实认证后再挂载 FileSession 路由 |
| 底部 dock 缺失；群组聊天栏 232px 偏移硬编码（`group/page.tsx:3399`） | INV-LAYOUT-02、03 仅部分满足 | 新建 dock 使用 `left: var(--sidebar-w)`；群组聊天栏偏移改为由变量派生 |
| 无 CodeLens provider、无定位库依赖、无文件树右键菜单组件 | FR-12-30 至 FR-12-32 不可用 | 注册 CodeLens provider；选定定位库并加入依赖；建立文件树上下文菜单 |
| 布局验收断言缺失（FR-12-36） | 层级回归无法被自动发现 | 增加 z-index 与 computed-style 的 e2e 断言 |
| 页面路由与 VS Code 扩展不存在（`frontend/src/app/worktree/[id]/file-sessions/`；仓内无 vscode 目录） | FR-12-29、FR-12-37 无法验收 | 随挂载裁决建立路由；VS Code 扩展的仓库位置待定 |
| 文件级入口职责重叠（IDE 文件工作区、`.nema` 文件即任务、FileSession） | 同一文件可能有三类入口，数据边界不清 | 在挂载裁决中界定三者职责与数据边界 |
| `.star/` 命名空间已被提交流程使用（`submit.rs:12-22`） | 目录与文件名冲突风险 | 在存储设计中列出 `.star/` 下保留的文件名与子目录规则 |
| adapter 子进程 kill 未实现（`graceful_shutdown.rs:23`，留待 P1） | agent 进程孤儿风险 | 完成 P1 后再接入 adapter 生命周期 |
| SELF-REVIEW 的完成声明无对应文档支撑：称 adapter 进程管理已复用 `process_supervisor`，02 中未出现该复用 | 评分与“已补充”声明不可作为状态依据 | 以本文件与代码检查为准；声明按文档实际内容更新 |

## 7. 验收与测试

### 7.1 现有测试

| 测试 | 路径 | 覆盖 | 数量 |
|---|---|---|---|
| IdeFileWorkspace | `frontend/src/components/ide/IdeFileWorkspace.test.tsx` | 未接入降级、文件树、语义节点、Patch 空态、内核探测后的 open、sync、undo、outline（IDE 相邻能力） | 19 个 `it(` |
| ide_endpoints | `crates/star-api-rest/tests/ide_endpoints.rs` | IDE 内核端点：version、open、insert、undo、redo、tabs、sync、export、outline 等（IDE 相邻能力） | 9 个测试函数 |
| MonacoEditor | `frontend/src/components/editor/MonacoEditor.test.tsx:215` | `onEditorReady` 暴露底层实例；用例标题提及 FileSession CodeLens，但覆盖的是实例暴露，不是 CodeLens | 21 个 `it(` |
| 编辑器主题 e2e | `frontend/e2e/editor-sakura-parity.spec.ts:118-139` | 编辑器 star-dark 主题的 computed 背景色（与 FileSession 无关） | 1 个用例（同文件共 7 个用例） |

### 7.2 验证范围

- 静态读取，未执行上述测试；上表仅确认测试文件与用例数量存在。
- 已确认存在但与 FileSession 无关：IDE 工作区降级与内核连接路径、`/api/v1/ide` 端点的测试文件。
- 未验证或不存在：FR-12-01 至 FR-12-28 无对应测试；01 §3 的性能指标（创建 P95 小于 3 秒、恢复 P95 小于 2 秒、checkpoint P99 小于 500 毫秒）无测量；01 §0.4、§0.5.4 的交互与布局验收用例无自动化断言；02 各 Phase 的验收命令（如 `star session start`、`star session list`）无法执行；03 的契约测试（CLI 与 REST 行为一致）不存在。

### 7.3 完成定义映射（01 §10）

| 维度 | 标准 | 现状 |
|---|---|---|
| 功能 | 所有 P0 FR 通过验收测试 | ⬜ |
| 性能 | 达到 01 §3 基准 | ⬜ |
| 测试 | 单元覆盖 85% 以上，核心流程集成测试 100% | ⬜ |
| 文档 | CLI help、REST 规范、ADR 落档 | 🟡 设计文档与 ADR 存在（ADR-0052 为 Proposed）；CLI help 与 REST 实现缺失 |
| 安全 | 无高危漏洞；认证为真实实现 | ⬜ 认证为 pass-through stub（`middleware/auth.rs:23-32`） |
| 示例 | README 含三个端到端示例（探索、重构、调试） | ⬜ |

## 8. 来源与裁决

| 源文档 | 吸收内容 | 与代码或其他源的差异 | 裁决 |
|---|---|---|---|
| `docs/architecture/file-session-management/01-requirements.md` | FR-LIFECYCLE、FR-ADAPTER、FR-GIT、FR-CONTEXT、FR-COLLAB 需求与优先级；§0 UI 与布局；§1 概念与目录；§2.2 AgentAdapter 契约；§4 数据模型与状态机；§5 CLI 与 REST；§6 实装计划；§7 集成；§9 风险；§10 完成定义；§11 参考 | §0.3 的 Worktree 双入口与 `AGENTS.md:22` 不一致；§4.2 SessionRef.state 为 Active、Paused、Archived、Deleted，与 03 六态不一致；§9 transcript 默认不入 Git，与 ADR-0052 §4.1 示例不一致；§11.2 引用的 `0045-agent-runtime.md` 不存在 | 需求编号、优先级、概念、目录结构与 §4.3 状态机采用；状态集合采用 03 六态；transcript 默认不入 Git；挂载层级不采纳，待裁决 |
| `docs/architecture/file-session-management/02-implementation-plan.md` | Phase 1 至 5 的任务与验收；`crates/star-file-session` 布局；AgentAdapter、StorageBackend、GitManager、ContextBuilder、TokenBudgetManager 的端口签名；Schema；里程碑 | Phase 5 的目标文件 `group_api/file_sessions.rs` 与 `frontend/src/app/worktree/[id]/file-sessions/` 当前不存在（代码事实）；Phase 4 Day 5 的文件锁 `.lock` 未在 01 或 ADR-0052 中出现 | 端口方法集采用 ADR-0052 §3.4，因为该 ADR 为决策记录且包含 supported_models，与 03 适配器端点一致；排期不采纳（待挂载裁决）；文件锁与乐观锁采用 |
| `docs/architecture/file-session-management/03-api-spec.md` | REST 资源模型；端点 1 至 16；适配器与配置端点；WebSocket 端点；RFC 9457 错误格式；权限矩阵；速率限制；版本控制 | 03 无数字化章节号，其他文档引用的 “03 §N” 实为端点编号；01 §5.2 的 14 行是其子集 | 采用为 REST 形态的权威来源；端点按 03 编号引用 |
| `docs/architecture/file-session-management/04-cli-reference.md` | CLI 选项、输出、退出码、环境变量、补全、别名、高级命令 | 比 01 §5.1 多 pause、rollback、promote、migrate、doctor；branch 为子命令形式，01 为 flag 形式 | 采用 04 为 CLI 形态权威；01 的 branch flag 形式不采用 |
| `docs/architecture/file-session-management/INTEGRATION-STATUS.md` | 未裁决声明；实施前须裁决的五项；API、CLI、计划为草案 | 层级描述写作 Cloud Branch，与 `AGENTS.md:22` 的 Project → Lane → Worktree → Branch 不一致 | 采用其状态结论；挂载层级以 `AGENTS.md:22` 为当前规范，裁决前不设定挂载位置 |
| `docs/architecture/file-session-management/README.md` | 文档导航；核心价值；Worktree 双入口概述；交互标准摘要；路线图；当前状态 | 与 01 与 ADR-0052 一致；当前状态自述 Phase 1 代码未开始 | 仅作导航与状态参照，不新增需求 |
| `docs/architecture/file-session-management/SELF-REVIEW.md` | 缺口清单：Windows 路径、会话锁、adapter 进程、transcript 格式、token 计数、隐私热重载、MR 模板、VS Code 发布、OpenAPI 版本；评分 4.65 | 称 Windows 路径缺口“已在 ADR-0052 补充”，但 ADR-0052 无 hash 策略，仅 SELF-REVIEW.md:106 出现 blake3；称 adapter 进程管理复用 `process_supervisor`，02 未出现该复用；session 锁与乐观锁确在 02 Phase 4 Day 5 出现 | 缺口清单列入 §6；评分与“已补充”声明不作为状态依据 |
| `docs/architecture/2026-08-26-upgrade/adr/0052-file-session-management.md` | 决策矩阵 §1（8 项）；模块拓扑与数据流 §2；目录与 Schema §3.1–3.2；状态机 §3.3；AgentAdapter 契约 §3.4；Git 集成 §4；与现有系统集成 §5；里程碑 §6；NFR §7；风险 §8；回滚 §9 | ADR 头部状态为 🟡 Proposed；§0.3.2 引用 `Sidebar.tsx:135` 的 z-20 与 `group/page.tsx:3002` 的 `md:left-[17rem]`，在 637ed19d 中均不存在（Sidebar 为 z-40 于 `:118`；群组页无 17rem 与 SIDEBAR_WIDTH）；§4.1 示例提交 transcript，与 §8 缓解冲突；§3.2 registry 示例使用 `active`；文件号 0052 与 `0052-subtask-binding.md` 重号 | 决策矩阵作为设计意图采用；布局现状以代码为准；transcript 默认不入 Git 采用 §8；状态枚举采用六态 |
| `AGENTS.md`（637ed19d） | §0 一句话硬约束（产品导航条目）：Project → Lane → Worktree → Branch；Lane 管理本地 Worktree；Run 为内部契约命名 | 与 INTEGRATION-STATUS.md 的 Cloud Branch 描述不一致；未出现 Cloud Branch 字样 | 作为当前层级权威（已接受的规范） |
| `docs/architecture/2026-08-26-upgrade/adr/0049-task-card-auto-worktree-agent.md` | Task Card、Worktree、Agent 的关联 | 正文未纳入判定 | 仅作关联引用 |
| `docs/architecture/2026-08-26-upgrade/adr/0045-star-agent-runtime-design.md` | Agent Runtime 的关联 | 01 §11.2 引用的 `0045-agent-runtime.md` 不存在；实际文件为本文件 | 仅作关联引用，引用以实际文件名为准 |
| `docs/architecture/2026-08-26-upgrade/adr/0024-ide-session-identity.md`、`adr/0043-audit-onboarding-failed.md` | IDE 会话身份与审计相邻能力 | 正文未纳入判定 | 仅作相邻引用；IDE 会话身份不等同 FileSession 身份 |
| `docs/architecture/2026-08-26-upgrade/adr/0052-subtask-binding.md` | 无 | 与 FileSession 的 ADR 同号 | 不采纳；编号冲突记入 §6 |
| `docs/features/F12-file-session.md`（637ed19d 上的同名文件） | 需求编号、检索关键词与代码行号线索 | 其 header 记录代码基线为 1643559f；其中 “03 §N” 引用（如 03 §8、§15、§16）实为端点编号 | 不采用其基线与行号；事实以 637ed19d 为准 |
| 代码证据（637ed19d） | `crates/star-cli/src/commands/`、`crates/star-api-rest/src/{lib.rs,group_api.rs,group_api/cli_sessions.rs,routes/ide.rs,middleware/auth.rs}`、`crates/domain-local-runtime/src/{cli_session.rs,graceful_shutdown.rs}`、`crates/argo-core/src/model.rs`、`crates/agent-bridge/src/runtime.rs`、`crates/ide-kernel/src/session.rs`、`crates/infrastructure/src/lib.rs`、`crates/domain-{audit,context,permission,hook,worktree,llm,local-runtime}`、`crates/star-{credential,taskgraph,treesitter,context-tiering,registry,dispatcher}`、`crates/bi-owner-runtime/src/task_event_query.rs`、`frontend/src/components/{Sidebar.tsx,CommandBar.tsx,ide/IdeFileWorkspace.tsx,editor/MonacoEditor.tsx,chat/RightChatRail.tsx,agent/SessionCard.tsx}`、`frontend/src/hooks/useIdeKernel.ts`、`frontend/src/lib/nema/nemaStore.ts`、`frontend/src/app/{layout.tsx,globals.css,worktree/page.tsx,worktree/[id]/group/page.tsx,(app)/kanban/page.tsx}`、`frontend/package.json`、`frontend/e2e/editor-sakura-parity.spec.ts`、`Cargo.toml` | 作为现状判定依据 | 作为状态裁决依据 |
| `frontend/src/components/agent/SessionCard.tsx`、`frontend/src/lib/run/runDirectorySession.tsx`、`crates/star-mcp/src/d6_session.rs`、`crates/sandboxd/migrations/001_sandbox_session.sql` | 文件头注释所述用途 | 分别为 agent 会话卡片、Run 目录上下文、MCP 会话重连、沙箱会话表，与 FileSession 无引用关系 | 排除，不作为依据 |
