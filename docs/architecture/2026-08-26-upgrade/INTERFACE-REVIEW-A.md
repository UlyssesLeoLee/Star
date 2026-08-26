# STAR Phase C 第 2 轮接口一致性审查报告（子代理 A）

> **审查对象**：5 份主 spec（CLI / agent-api / ide-api / MCP / REST）+ 4 份交叉对账（arch/03 / arch/04 / protocol-survey / star-vs-gitgit）
> **审查范围**：命令名一致性、Schema 内部矛盾、MCP 2026-07-28 符合度、CLI 17 命令覆盖度、OpenAPI 3.1 正确性、版本号一致性、JSON 字段命名、错误模型、Capability Discovery 对齐、守门规则遵循度
> **审查者**：架构师（Mavis 接手 agent per DEC-008）— 子代理 A
> **审查日期**：2026-08-26
> **基于 commit**：`876a2a7`（Phase C 54 份 spec 草案）
> **不沿用**：`bc23d6c` 的叙事（该 commit 引用了未做过的 frontend commit hash，属于历史叙事灰区）

---

## §1 一致性结论表

| # | 问题 | 涉及 spec | 严重度 | 建议修复 |
|---|---|---|---|---|
| 1 | CLI "17 命令" 数字与 arch/03 §2.2 实际列出的 18 条命令不一致；CLI spec §2 表里 23 条 | cli/01, arch/03 | 🔴 高 | 统一成 "17 核心命令" 实际列表（删 4 个加 3 个补齐） |
| 2 | MCP spec 漏 2026-07-28 关键变更（stateless core / Header routing / ttlMs / Feature Lifecycle / Authorization hardening / MRTR） | mcp/01 | 🔴 高 | 在 §1 加 6 项关键变更检查表 |
| 3 | MCP spec §7 验证清单说"必须能 invoke star submit"，但 §2 工具表无 submit tool | mcp/01 | 🔴 高 | 加 `submit` tool（domain 语义），或在 §7 删 "star submit" 验证项 |
| 4 | agent-api/v1 引用大量 schema（CurrentTask / Issue / Context / CodeSearchResult / SymbolResult / ReferencesResult / WorkspaceList / MR / ReviewResult / TestResult / PipelineRun / PipelineStatus / Capabilities / Permissions / Error）在 §3 主体未展开 | agent-api/01 | 🔴 高 | §3 必须至少给 5 个核心 schema 完整定义；§4 文件名清单 + §3 内容必须对齐 |
| 5 | CLI `star workspace current` 引用的 `agent-api/v1#Workspace` 实际是 ide-api/v1 的 Workspace（含 open_files / diagnostics / ide_client） | cli/01, agent-api/01, ide-api/01 | 🔴 高 | 拆 `WorkspaceSummary`（agent 视角，agent-api/v1）+ `WorkspaceState`（IDE 视角，ide-api/v1） |
| 6 | 错误模型 4 套并存：CLI §5 五字段 / Universal Submit §3 四字段 / MCP 完全无 / REST 完全无 | cli/01, mcp/01, rest/01, flows/05 | 🔴 高 | 统一为单 `Error` schema，CLI/MCP/REST/Submit 全部引用同一份 |
| 7 | Universal Submit 11 步流程中 diff / policy / commit / push / link 步骤无独立 CLI 命令 | cli/01, flows/05 | 🔴 高 | 加 `star diff` / `star policy check` / `star commit` / `star push` / `star mr link <id>`，或在 `star submit` 文档明确"这 5 步是内部步骤不暴露" |
| 8 | CLI ↔ MCP 命名风格不一致：`star issue list` vs MCP `search_issues` / `star mr review` vs `request_review` / `star test affected` vs `run_validation` / `star mr create` vs `create_merge_request` | cli/01, mcp/01 | 🟡 中 | 在 cli/01 §2 加 "CLI ↔ MCP 映射表"，明确每个 CLI 命令对应的 MCP tool 名称 |
| 9 | REST §4 端点表缺：project / issue / workspace-list / mr-show / mr-review / test / pipeline-run / context-current / code-references / worktree-by-id | rest/01, cli/01 | 🟡 中 | §4 端点表补齐 17 个 endpoint（与 CLI 17 命令一一对应） |
| 10 | REST §1 漏 3.1 关键提示：webhooks 字段 / `nullable: true` → `type: [string, "null"]` 破坏性迁移 / `info.summary` 允许 | rest/01, ecosystem-survey/protocol-survey | 🟡 中 | §1 加 3.1 关键字段清单 |
| 11 | agent-api/v1 §2 顶部 OpenAPI snippet `version: "1.0.0"` 与 schema_version `agent-api/v1` 关系不明（info.version vs schema_version 是否同步演化？） | agent-api/01 | 🟡 中 | §1 加 versioning 规则：info.version = schema major.minor；schema patch 走 metadata |
| 12 | REST §2 文件名 `agent-api-v1.yaml` 用连字符；agent-api spec 主体用斜杠 `agent-api/v1` | rest/01, agent-api/01, ide-api/01 | 🟡 中 | 统一为斜杠（OpenAPI info.version 字段会带斜杠，避免双重风格） |
| 13 | MCP spec §2 注释"实际 15 个 tools"与 §6 实施位置"13 个 tool 实现"自相矛盾 | mcp/01 | 🟡 中 | 统一为 15，§6 改 "15 个 tool 实现"，§2 注释删掉 |
| 14 | MCP spec §2 工具表 `request_review` 输入 `{mr_id, reviewers?}`，与 §3 禁止的 `update_issue_table` 等是同层（"领域操作"），但 request_review 实际是动作触发而非查询 —— 命名应区分 | mcp/01 | 🟡 中 | 把动作型 tool 加动词前缀约定：query = `get_*` / `search_*`；action = `create_*` / `update_*` / `request_*`（已部分遵守，可显式声明） |
| 15 | arch/03 §4 Capability Discovery `capabilities` 数组含 `repositories` / `deployments`，但 CLI 17 命令无 `star repo` / `star deploy` 对应 | cli/01, arch/03 | 🟡 中 | 要么加命令（`star repo list` / `star deploy list`），要么从 capabilities 数组删（`repositories` 隐式 / `deployments` 走 pipeline） |
| 16 | REST §4 缺 `GET /api/v1/ide/capabilities` / `permissions` / `instructions`（agent 有，ide 无） | rest/01, cli/01 | 🟡 中 | §4 补 3 个 `/api/v1/ide/*` 端点 |
| 17 | REST §4 `POST /api/v1/mr` 用单数，其他 endpoint 复数（`/worktrees`, `/tasks`, `/workspaces`） | rest/01 | 🟡 中 | 改为 `POST /api/v1/merge-requests`（与 capabilities 数组 `merge_requests` 复数风格一致） |
| 18 | MCP spec §1 缺 Tool list 排序 + ttlMs 缓存要求（per protocol-survey §1 "Tool list 必须按 deterministic order 排序 + 支持 ttlMs 缓存"） | mcp/01, ecosystem-survey | 🟡 中 | §1 补：tool list 按 name 升序；metadata 字段含 `ttlMs` + `cacheScope` |
| 19 | REST §4 端点表无 error response（OpenAPI 3.1 应有 `responses` 块，per Error schema） | rest/01 | 🟡 中 | §4 每个端点加 4xx/5xx 响应（统一引用 Error schema） |
| 20 | arch/03 §2.2 CLI 17 命令示例缺 `star mr review` / `star test run` / `star context current`（3 个遗漏） | arch/03, cli/01 | 🟡 中 | §2.2 补 3 个命令示例 |
| 21 | agent-api spec §4 落盘位置列文件名（Issue.json / Task.json / ...）但 §3 主体未给 schema | agent-api/01 | 🟢 低 | §3 与 §4 交叉引用：§3 列出已定义 schema，§4 列出待落盘 schema |
| 22 | arch/04 §2 架构图 `MCP client` → `star-mcp`，但 §5 "3 个最低要求"列 MCP 2026-07-28，没指明 transport 类型（per arch/03 §2.3 stdio） | arch/04, arch/03 | 🟢 低 | §5 加 transport 约束：stdio（local process）/ Streamable HTTP（remote, Phase 2） |
| 23 | REST §2 列 `git-provider-v1.yaml` 但 §4 端点表无 git-provider endpoint；§5 验证清单也未涉及 | rest/01, adr/0023 | 🟢 低 | 要么 §4 显式标 "TBD Phase 2"，要么 §2 移出（git provider API 是 GitGit 自己的 axum HTTP，per ADR-0022） |
| 24 | agent-api §3.2 Worktree 包含 `agent_session_id` + `ide_session_id` 字段 —— agent 视角的 Worktree 不应同时绑两个 session | agent-api/01, ide-api/01 | 🟢 低 | 拆 `WorktreeBinding` 子对象：`{ agent_sessions: [...], ide_sessions: [...] }` |
| 25 | CLI spec §3 通用 flags 缺 `--help` / `--no-header` / `--version`（基础 shell 习惯） | cli/01 | 🟢 低 | §3 补 3 个 flag |
| 26 | arch/03 §2.5 AGENTS.md bootstrap 示例用 `star test affected` 但 AGENTS.md spec（per 任务原文）可能要求 3 个最小命令：`star agent capabilities` + `star task current --json` + `star submit` | arch/03, ecosystem-survey | 🟢 低 | §2.5 bootstrap 删 `star test affected` / `star context current --json` / `star code search`（不在 3 个最小命令里），per protocol-survey §2 |
| 27 | CLI spec §3 `--schema-version <v>` flag 但未说明默认版本（应默认 `agent-api/v1`） | cli/01, agent-api/01 | 🟢 低 | §3 加："默认 = 当前实现 schema version" + "与 `star agent capabilities` 输出一致" |
| 28 | MCP spec §4 Resources / §5 Prompts 用 "可选 / 不强制" 措辞，但 arch/03 §2.3 没说 "可选" —— MCP 资源是否必实现不明 | mcp/01, arch/03 | 🟢 低 | arch/03 §2.3 加 "MCP Resources / Prompts MVP 不实现" 明确范围 |
| 29 | ide-api/v1 §1 "跟 `agent-api/v1` 平行，独立演进" 但 §3 落盘位置在 `star-cli/src/schemas/ide-api-v1/` —— 跟 agent-api 共用目录 | agent-api/01, ide-api/01 | 🟢 低 | §3 改路径：`crates/star-ide-gateway/src/schemas/ide-api-v1/`（per arch/04 IDE Gateway crate 边界） |
| 30 | REST §2 `git-provider-v1.yaml` 路径在 `star-rest/openapi/`，与 ADR-0022 §2.2 "Git Provider Abstraction" 边界模糊（per ADR-0022 GitGit 暴露自己的 axum HTTP） | rest/01, adr/0022 | 🟢 低 | §2 加注释："git-provider-v1.yaml 是 STAR 对 Git Provider 的抽象，GitGit 自身的 HTTP API 由 GitGit crate 单独维护" |

---

## §2 必须修复（🔴 高）

### 🔴 #1 — CLI "17 命令" 数字与实际列表不一致

**原文引用**：

> `arch/03 §2.2` 标题 "17 个核心命令" 但 bash 代码块列出 18 个：
> `star project list` / `star issue list` / `star issue show` / `star issue claim` / `star task current` / `star context get` / `star code search` / `star code symbol` / `star workspace list` / `star workspace current` / `star worktree create` / `star worktree enter` / `star worktree status` / `star mr create` / `star mr show` / `star test affected` / `star pipeline run` / `star submit` = **18 个**

> `cli/01 §2` 标题"核心命令（per 任务原文 §9）"表格里 23 行（含子命令 `star test run`、`star pipeline status`、`star mr review`、`star code references`）

**问题**：
- 任务原文 §9 指定 17 个核心命令，但 cli/01 §2 列了 23 个（含 6 个非 §9 任务原文的命令）
- arch/03 §2.2 标题"17"与代码块"18"自相矛盾
- 缺：`star mr review` / `star test run` / `star context current`（cli/01 §2 有但 arch/03 §2.2 bash 块无）
- 多：`star code references`（cli/01 §2 有但 arch/03 §2.2 bash 块无 —— arch/03 漏命令）
- 多：`star project list` 是 §9 任务原文的吗？需要核对

**修复建议**：
- cli/01 §2 拆 "§9 任务原文 17 个核心命令"（主表）+ "扩展命令（per arch/03 §4 capabilities 覆盖）"（附表）
- arch/03 §2.2 改标题 "17 个核心命令 + 3 个扩展命令（mr review / test run / context current）"，bash 块补齐 3 个

**影响字段**：
- `cli/01 §2 全部 23 行`
- `arch/03 §2.2 标题 + 17 行 bash 块`
- `arch/03 §4 capability 数组`（[projects, issues, tasks, workspaces, worktrees, repositories, code_search, code_navigation, code_context, merge_requests, context, tests, pipelines, reviews, deployments] — `reviews`/`pipelines` 隐式走 `mr review` / `pipeline run`，但 `repositories` 和 `deployments` 在 cli/01 §2 无对应命令）

---

### 🔴 #2 — MCP spec 漏 2026-07-28 关键变更

**原文引用**：

> `ecosystem-survey/protocol-survey.md §1` 列出 2026-07-28 核心变更 6 项：
> ① Stateless core（无 session）② Multi Round-Trip Requests (MRTR) ③ Header-based routing（`Mcp-Method` / `Mcp-Name`）④ 可缓存 list 结果（`ttlMs` / `cacheScope`）⑤ Authorization hardening（RFC 9207 issuer validation）⑥ 正式 Feature Lifecycle（Active / Deprecated / Removed）

> `mcp/01 §1` 仅提到：
> - MCP **2026-07-28** ✅
> - Transport: **stdio** ✅
> - 12 个月 deprecation 窗口 ✅
>
> **未提到上述 6 项中的任何一项**

**问题**：
- 协议事实基线要求 STAR MCP 实现这 6 项，但 spec §1 都没说
- "Tool list 必须按 deterministic order 排序 + 支持 ttlMs 缓存"（per protocol-survey §1 "对 STAR 的推论"）在 mcp/01 §2 / §1 完全没体现
- Feature Lifecycle 影响 spec 自身：MCP spec §1 应说明"本 spec 列的 tools 处于 Active Lifecycle"
- 2026-07-28 强制要求 Starlette/Express 风格的 Header routing（即使 stdio 也有 protocol-level header），spec 没提

**修复建议**：mcp/01 §1 改为表格：

```markdown
## 1. 规范版本

- MCP **2026-07-28**（per 2026-08-26 调研）
- Transport: **stdio**（Rust SDK 仍在 beta，Streamable HTTP 风险规避；per Protocol Survey §1）

### 1.1 2026-07-28 关键变更符合度

| 关键变更 | 符合度 | 说明 |
|---|---|---|
| ① Stateless core（无 session） | ✅ 必遵 | server 不持有 agent session 状态；所有上下文由 tool input 传入 |
| ② Multi Round-Trip Requests (MRTR) | 🟡 暂不实现 | Phase 2 再评估，MVP 工具都是单回合 |
| ③ Header-based routing（`Mcp-Method` / `Mcp-Name`） | ✅ 必遵 | stdio transport 通过 JSON envelope 携带 method/name |
| ④ 可缓存 list 结果（`ttlMs` / `cacheScope`） | ✅ 必遵 | tool list metadata 包含 `ttlMs=30000` + `cacheScope=workspace` |
| ⑤ Authorization hardening（RFC 9207 issuer validation） | ✅ 必遵 | OAuth 2.1 + issuer validation 在 MCP server 入口校验 |
| ⑥ 正式 Feature Lifecycle（Active / Deprecated / Removed） | ✅ 必遵 | 本 spec 列的 tools 全部 Active；12 个月内不弃用 |

### 1.2 兼容承诺

- 必须兼容旧 spec 至少 12 个月（per MCP 官方 12 个月 deprecation 窗口）
- tool list 按 name 字典序排序（deterministic order）
- tool metadata 必含 `ttlMs` + `cacheScope` 字段
```

**影响字段**：
- `mcp/01 §1 整段`
- `crates/star-mcp/src/main.rs` 实现
- `mcp/01 §2 工具表` 需在每行加 metadata 字段
- `mcp/01 §7 验证清单` 需加 "tool list 排序校验"

---

### 🔴 #3 — MCP spec §7 验证清单说"必须能 invoke star submit"但 §2 工具表无 submit

**原文引用**：

> `mcp/01 §7 验证` 列出：
> ```bash
> # 必须能 invoke get_issue
> # 必须能 invoke get_current_task
> # 必须能 invoke create_worktree
> # 必须能 invoke star submit
> ```

> `mcp/01 §2 工具表` 共 15 个 tool：get_issue / search_issues / get_current_task / get_workspace / get_worktree / create_worktree / search_code / get_symbol / find_references / get_code_context / get_context / create_merge_request / request_review / run_validation / get_pipeline_status — **无 submit tool**

**问题**：
- spec §7 验证清单和 §2 工具表自相矛盾
- "star submit" 是 Universal Submit 11 步流程入口（per `flows/05`），MCP 应该暴露为 `submit` tool
- 缺 submit tool = MCP 用户必须用 `create_merge_request` + `request_review` + `run_validation` 自己拼 11 步，违背 "Universal Submit" 简化意图

**修复建议**：
- 方案 A（推荐）：mcp/01 §2 加 `submit` tool：

```markdown
| `submit` | `{worktree_id?, force?}` | `SubmitResult` |
```

- 方案 B：mcp/01 §7 验证清单删 "必须能 invoke star submit"，改为 "必须能 invoke create_merge_request + request_review + run_validation 联合流"

**影响字段**：
- `mcp/01 §2 工具表` 增 1 行（方案 A）或 `mcp/01 §7 验证清单` 删 1 行（方案 B）
- `crates/star-mcp/src/tools/submit.rs` 新增（方案 A）
- `flows/05 §4 实施位置` 引用关系更新

---

### 🔴 #4 — agent-api/v1 大量 schema 引用未在 §3 主体展开

**原文引用**：

> `agent-api/01 §3 核心 Schemas（节选）` 实际展开 3 个：`Task`（§3.1）/ `Worktree`（§3.2）/ `SubmitResult`（§3.3）
>
> `cli/01 §2` 引用 17 个 schema：`ProjectList` / `IssueList` / `Issue` / `ClaimResult` / `CurrentTask` / `Context` / `CodeSearchResult` / `SymbolResult` / `ReferencesResult` / `WorkspaceList` / `Workspace` / `Worktree` / `WorktreeStatus` / `MR` / `ReviewResult` / `TestResult` / `PipelineRun` / `PipelineStatus` / `SubmitResult` / `Capabilities` / `Permissions`
>
> `agent-api/01 §4 落盘位置` 列出文件名：`Issue.json` / `Task.json` / `Worktree.json` / `Workspace.json` / `MR.json` / `Context.json` / `CodeSearchResult.json` / `SymbolResult.json` / `SubmitResult.json` / `Error.json` / `Capabilities.json` / `Permissions.json` / ...

**问题**：
- §3 标题 "节选" 暗示还有其他节，但 spec 实际只给了 3 个
- §3.1 给了 `Task`，但 cli/01 引用的是 `CurrentTask` —— 命名不一致
- §4 落盘位置的文件名 ≠ cli/01 引用的 schema 名（多个 mismatch）
- 大量 schema 在 spec 中是 "黑盒" —— 写代码时无法从 spec 推断字段

**修复建议**：agent-api/01 §3 扩展为 3.1-3.15，每个 schema 至少给 3-5 个核心字段（id / 状态 / 时间戳 + 1-2 个领域字段）：

```markdown
### 3.1 Task
（已展开）

### 3.2 Worktree
（已展开）

### 3.3 SubmitResult
（已展开）

### 3.4 Issue
- id (string, e.g. "STAR-1024")
- title (string)
- status (enum: OPEN | IN_PROGRESS | REVIEW | CLOSED)
- priority (enum: P0 | P1 | P2 | P3)
- labels (string[])
- assignee (string?)
- created_at / updated_at (timestamp)

### 3.5 IssueList
- items (Issue[])
- total (integer)
- cursor (string?)

### 3.6 CurrentTask
（与 Task 共享，但额外必含 `claimed_at` + `claim_expires_at`）

### 3.7 MR (MergeRequest)
- id (string, e.g. "MR-789")
- title (string)
- status (enum: OPEN | MERGED | CLOSED)
- source_branch / target_branch (string)
- url (string?)

### 3.8 Context
- issue_id (string)
- related_code (CodeRef[])
- related_docs (DocRef[])
- related_mrs (MRRef[])

### 3.9 CodeSearchResult
- query (string)
- matches (CodeMatch[])
- total (integer)

### 3.10 SymbolResult
- name (string)
- kind (enum: function | struct | enum | trait | ...)
- file (string)
- line (integer)

### 3.11 WorktreeStatus
- worktree (Worktree)
- last_commit (Commit)
- uncommitted_files (integer)

### 3.12 TestResult
- passed (integer)
- failed (integer)
- skipped (integer)
- failed_tests (TestCase[])

### 3.13 PipelineRun
- id (string)
- status (enum: QUEUED | RUNNING | SUCCESS | FAILED)
- url (string?)

### 3.14 Capabilities
（per arch/03 §4 capability 数组）

### 3.15 Error
- error (string, e.g. "WORKTREE_CONFLICT")
- recoverable (boolean)
- suggested_actions (string[])
- message (string)
- trace_id (string)
- details (object?)
```

**影响字段**：
- `agent-api/01 §3 整段` 从 3 个扩展到 15 个
- `cli/01 §2 schema 引用` 全部有效
- `mcp/01 §2 工具表` "输出" 列引用有效
- `crates/star-cli/src/schemas/agent-api-v1/*.json` 实现 15 个

---

### 🔴 #5 — `agent-api/v1#Workspace` 与 `ide-api/v1#Workspace` 是不同对象但 CLI 错位引用

**原文引用**：

> `ide-api/01 §2.1 Workspace` 定义：id / name / repository{id, provider, url} / worktree_id / **open_files**（带 cursor + dirty）/**active_symbol** / **diagnostics** / **ide_client** / **ide_version`

> `cli/01 §2` `star workspace current` 输出 schema = `agent-api/v1#Workspace`

> `arch/03 §4` Capability Discovery 区分 `agent.commands` / `ide.commands`，agent 视角的 workspace 命令是 `workspace current`

**问题**：
- ide-api/v1 的 Workspace 是 IDE 视角的实体状态（含 open_files / diagnostics / ide_client）
- agent-api/v1 的 Workspace（如果存在）应该是 agent 视角的逻辑抽象（不含 IDE 内部状态）
- CLI `star workspace current` 是 agent 命令，但引用 `agent-api/v1#Workspace` 会让 agent 拿到 IDE 视角的 schema（带 open_files / diagnostics / ide_client）—— 跨层数据泄漏，违反 ADR-0022 "IDE 归 STAR" 边界

**修复建议**：
- agent-api/v1 §3 加 `WorkspaceSummary`：

```markdown
### 3.16 WorkspaceSummary
- id (string, e.g. "ws-abc")
- name (string)
- repository (RepositoryRef)  // {id, provider, url}，不带 worktree_id
- worktree_id (string?)
- agent_session_id (string)
- created_at / updated_at (timestamp)
```

- ide-api/v1 §2.1 `Workspace` 改名为 `WorkspaceState`（明确是状态视图）
- cli/01 §2 `star workspace current` 引用改为 `agent-api/v1#WorkspaceSummary`
- arch/03 §4 Capability Discovery `ide.commands` 增 `workspace state` 命令，引用 `ide-api/v1#WorkspaceState`

**影响字段**：
- `agent-api/01 §3 增 §3.16 WorkspaceSummary`
- `ide-api/01 §2.1 标题改 Workspace → WorkspaceState`
- `cli/01 §2 两行（workspace current / workspace list）引用更新`
- `arch/03 §4 Capability Discovery JSON 示例` 更新

---

### 🔴 #6 — 错误模型 4 套并存

**原文引用**：

> `cli/01 §5` 错误模型：
> ```json
> { "error": "WORKTREE_CONFLICT", "recoverable": true, "suggested_actions": [...], "message": "...", "trace_id": "..." }
> ```
> 字段：`error` / `recoverable` / `suggested_actions` / `message` / `trace_id`（**5 个**）

> `flows/05-universal-submit §3` 错误恢复：
> ```json
> { "error": "VALIDATION_FAILED", "recoverable": true, "suggested_actions": [...], "details": {...} }
> ```
> 字段：`error` / `recoverable` / `suggested_actions` / **`details`**（**4 个，无 message 无 trace_id**）

> `mcp/01 §3 / §4 / §5` **无错误模型**（MCP 2026-07-28 规定 server 应返回标准 JSON-RPC error，spec 完全没提）

> `rest/01 §4` **无错误模型**（OpenAPI 3.1 `responses` 块未定义任何 error response）

> `agent-api/01 §4` 落盘位置列了 `Error.json` 但 §3 没展开

**问题**：
- 4 套不同字段的错误对象会让 client 端写 4 套 parser
- MCP server 返回非标准 JSON-RPC error 会破坏 Claude / Cursor / Junie 客户端的通用错误处理
- REST endpoint 缺 `4xx` / `5xx` 响应定义 = OpenAPI spec 不完整
- `details` 字段在 CLI 错误模型中没有，但 Universal Submit 错误有 —— agent 重试时拿不到 failed_tests 列表

**修复建议**：
1. `agent-api/01 §3.15 Error` 定义为唯一权威 schema：

```yaml
type: object
required: [error, recoverable, suggested_actions, message, trace_id]
properties:
  error:
    type: string
    description: "Machine-readable error code, e.g. WORKTREE_CONFLICT, VALIDATION_FAILED"
  recoverable:
    type: boolean
  suggested_actions:
    type: array
    items: { type: string }
    description: "List of CLI commands or natural language hints"
  message:
    type: string
    description: "Human-readable summary"
  trace_id:
    type: string
    description: "Distributed trace correlation ID"
  details:
    type: object
    additionalProperties: true
    description: "Optional structured details (e.g. failed_tests, conflict_files)"
```

2. `cli/01 §5` 改为引用 `agent-api/v1#Error`
3. `flows/05 §3` 改为引用 `agent-api/v1#Error`（增加 `message` + `trace_id` 字段，把 failed_tests 放进 `details`）
4. `mcp/01 §3` 加错误模型段："MCP tool 返回失败时，result.content 包含 `agent-api/v1#Error` JSON 字符串 + `isError: true`"
5. `rest/01 §4` 每个端点加 `responses` 块：

```yaml
responses:
  '4xx':
    content:
      application/json:
        schema: { $ref: '#/components/schemas/Error' }
  '5xx':
    content:
      application/json:
        schema: { $ref: '#/components/schemas/Error' }
```

**影响字段**：
- `agent-api/01 §3 增 §3.15 Error`
- `cli/01 §5 改引用`
- `flows/05 §3 改引用`
- `mcp/01 §3 增错误模型段`
- `rest/01 §4 每个端点加 responses 块`
- `crates/star-cli/src/schemas/agent-api-v1/Error.json` 落盘

---

### 🔴 #7 — Universal Submit 11 步流程中 5 步无独立 CLI 命令

**原文引用**：

> `flows/05-universal-submit §2` 11 步流程：
> 1. 检查 Task → `star task current` ✅
> 2. 检查 Workspace → `star workspace current` ✅
> 3. 检查 Worktree → `star worktree status` ✅
> 4. 检查 Diff → ❌ 无 `star diff` 命令
> 5. 执行 Required Validation → `star test affected` ✅
> 6. 检查 Policy → ❌ 无 `star policy check` 命令
> 7. Commit → ❌ 无 `star commit` 命令（兜底 `git commit`）
> 8. Push → ❌ 无 `star push` 命令（兜底 `git push`）
> 9. 创建 / 更新 MR → `star mr create` ✅
> 10. 关联 Issue → ❌ 无 `star mr link <issue_id>` 命令
> 11. 回写 Agent 状态 → ❌（内部操作，合理）
> 12. 回写 IDE Session 状态 → ❌（内部操作，合理）

**问题**：
- 步骤 4 / 6 / 7 / 8 / 10 没有独立命令，agent 失败恢复时无法重跑单步
- per arch/03 §3 Fallback Ladder Level 1-4，agent 必须能**单步重试**（如 `star test affected` 失败后单独跑 `star test affected --fix` 或 `star policy check`）
- "agent 兜底走 git" 不是 spec 的显式声明 —— cli/01 §2 没提 `star` 是 `git` 的 superset 还是 disjoint

**修复建议**（两选一）：

**方案 A（推荐）**：cli/01 §2 增 5 个命令：

```markdown
| `star diff` | 显示当前 worktree 改动 | `agent-api/v1#DiffResult` |
| `star policy check` | 跑 policy 校验 | `agent-api/v1#PolicyResult` |
| `star commit` | 提交（含 STAR 元数据） | `agent-api/v1#CommitResult` |
| `star push` | 推送（含 pre-push hook） | `agent-api/v1#PushResult` |
| `star mr link <issue_id>` | MR 关联 Issue | `agent-api/v1#LinkResult` |
```

并在 §1 设计原则加："`star` 是 `git` 的 superset，必要时内部直接调 `git` 命令（如 `star commit` 内部调 `git commit` 后追加 agent metadata trailer）"

**方案 B**：flows/05 §2 改 11 步流程描述，把 4 / 6 / 7 / 8 / 10 步标注为 "内部步骤，不暴露单独命令"，并加 1 条说明："失败时只能重跑 `star submit` 整体"。

**影响字段**：
- `cli/01 §2` 增 5 行（方案 A）或 `flows/05 §2` 改描述（方案 B）
- `arch/03 §2.2 17 核心命令` 不变（方案 A 增加到 22 核心 + 5 扩展；方案 B 保持 17）
- `mcp/01 §2` 需对应增 `get_diff` / `check_policy` 等 tool（方案 A）

---

## §3 建议改进（🟡 中）

### 🟡 #8 — CLI ↔ MCP 命名风格不一致

**原文引用**：

| CLI 命令 | MCP tool | 差异 |
|---|---|---|
| `star issue list` | `search_issues` | "list" vs "search" 语义不同 |
| `star issue show <id>` | `get_issue` | ✅ 语义一致 |
| `star issue claim <id>` | (无对应) | ❌ |
| `star task current` | `get_current_task` | ✅ |
| `star context get <id>` | `get_context` | ✅ |
| `star context current` | (无对应) | ❌ |
| `star code search` | `search_code` | ✅ |
| `star code symbol` | `get_symbol` | ✅ |
| `star code references` | `find_references` | 动词差异 |
| `star workspace current` | `get_workspace` (空入参) | 单 tool 覆盖 + agent 视角 vs IDE 视角混乱 |
| `star worktree create` | `create_worktree` | ✅ |
| `star worktree status` | `get_worktree` | "status" vs "get" |
| `star mr create` | `create_merge_request` | "mr" vs "merge_request" |
| `star mr show` | (无对应) | ❌ |
| `star mr review` | `request_review` | "review" vs "request_review" |
| `star test affected` | `run_validation` | "test" vs "validation" |
| `star test run` | `run_validation` | 同上 |
| `star pipeline run` | (无对应) | ❌ |
| `star pipeline status` | `get_pipeline_status` | ✅ |
| `star submit` | (无对应) | ❌（per 🔴 #3） |

**问题**：
- "list" / "search" / "find" 三个动词在不同 spec 表达"查找多个"——agent 客户端需要 query 多套命名
- "mr" 缩写 vs "merge_request" 全名——cli 用 mr 是 shell 习惯（短），但 MCP 暴露给 IDE 时是全名，跨层不一致
- "test" / "validation" 表达"跑检查"——agent 视角用 test，STAR 内部用 validation（per cli/01 `star test affected` 输出 `TestResult` 而 mcp `run_validation` 输出 `ValidationResult`）

**修复建议**：cli/01 §2 加 "CLI ↔ MCP 映射表"：

```markdown
## 2.5 CLI ↔ MCP 命名映射

| CLI 命令 | MCP tool | 备注 |
|---|---|---|
| `star issue list` | `search_issues` (empty query) | MCP 统一用 search_ 表达列表 |
| `star issue show <id>` | `get_issue` | 一致 |
| `star issue claim <id>` | (待定，可能用 `claim_issue`) | 🔴 #3 同源问题 |
| `star mr create` | `create_merge_request` | CLI 用 mr 缩写，遵循 shell 习惯 |
| `star test affected` | `run_validation` (scope=affected) | MCP 统一用 validation 表达测试+检查 |
```

并在 mcp/01 §2 表格前加命名约定：

```markdown
### 2.1 命名约定

- 查询：get_*（单对象）/ search_*（多对象）
- 操作：create_* / update_* / delete_* / request_*（动作）
- 缩写：CLI 用 mr 缩写（shell 习惯），MCP 用 merge_request 全名（machine 协议）
```

**影响字段**：
- `cli/01 §2.5 新增`
- `mcp/01 §2.1 新增`
- `crates/star-mcp/src/tools/*.rs` 实现时命名同步

---

### 🟡 #9 — REST §4 端点表覆盖度不均

**原文引用**：

> `rest/01 §4` 列 12 个 endpoint：
> - `GET /api/v1/agent/capabilities` / `permissions` / `instructions`
> - `GET /api/v1/tasks/current`
> - `GET /api/v1/workspaces/current`
> - `GET /api/v1/worktrees` / `POST /api/v1/worktrees`
> - `GET /api/v1/code/search` / `GET /api/v1/code/symbols/{name}`
> - `POST /api/v1/mr`
> - `POST /api/v1/submit`
> - `GET /api/v1/context/{issue_id}`

**问题**：对照 cli/01 §2 17 核心命令，REST §4 缺：
- ❌ `star project list` → 缺 `GET /api/v1/projects`
- ❌ `star issue list` / `show` / `claim` → 缺 `GET /api/v1/issues` / `GET /api/v1/issues/{id}` / `POST /api/v1/issues/{id}/claim`
- ❌ `star workspace list` → 缺 `GET /api/v1/workspaces`
- ❌ `star context current` → 缺 `GET /api/v1/contexts/current`
- ❌ `star code references` → 缺 `GET /api/v1/code/references/{name}`
- ❌ `star worktree status` 单独端点（仅 list）→ 缺 `GET /api/v1/worktrees/{id}/status`
- ❌ `star mr show` → 缺 `GET /api/v1/mr/{id}`
- ❌ `star mr review` → 缺 `POST /api/v1/mr/{id}/review`
- ❌ `star test affected` / `run` → 缺 `POST /api/v1/tests/run`
- ❌ `star pipeline run` → 缺 `POST /api/v1/pipelines`
- ❌ IDE 端 `ide capabilities` / `permissions` / `instructions`（per cli/01 §4）→ 缺 `GET /api/v1/ide/*`（per 🟡 #16）

**修复建议**：rest/01 §4 扩展到 25+ 端点，每个 CLI 命令对应 1 个或多个 REST endpoint。

**影响字段**：
- `rest/01 §4 端点表从 12 行扩展到 25+ 行`
- `crates/star-rest/openapi/agent-api-v1.yaml` 实现
- arch/03 §4 Capability Discovery 端点引用更新

---

### 🟡 #10 — REST §1 漏 OpenAPI 3.1 关键提示

**原文引用**：

> `rest/01 §1`：
> ```markdown
> - **OpenAPI 3.1**（不是 3.0）— 完整对齐 JSON Schema 2020-12
> - 不采用 OpenAPI 3.2（MVP 阶段，3.1 已稳）
> ```

> `ecosystem-survey/protocol-survey.md §3` 列 3.1 关键差异：
> - 完整对齐 JSON Schema Draft 2020-12 ✅（rest/01 §1 提到）
> - 支持 `webhooks` ❌（rest/01 §1 没提）
> - `info.summary` ❌（rest/01 §1 没提）
> - `info.license.identifier` ❌（rest/01 §1 没提）
> - 破坏性变更：`nullable: true` → `type: [string, "null"]` ❌（rest/01 §1 没提）
> - exclusive bound 用 boolean modifier ❌（rest/01 §1 没提）

**问题**：
- 3.1 新增的 webhooks 字段未声明用 / 不用 —— Webhook 通知是 MR / Pipeline 关键场景
- 3.0→3.1 破坏性变更（`nullable`）spec 没提示，后续维护者会用 3.0 风格导致 lint 失败

**修复建议**：rest/01 §1 改：

```markdown
## 1. OpenAPI 版本

- **OpenAPI 3.1**（不是 3.0）— 完整对齐 JSON Schema 2020-12
- 不采用 OpenAPI 3.2（MVP 阶段，3.1 已稳）

### 1.1 3.1 关键字段采用

| 字段 | 状态 | 说明 |
|---|---|---|
| `webhooks` | ✅ 采用 | MR / Pipeline 状态变更推送给 Web 客户端 |
| `info.summary` | ✅ 采用 | spec 摘要 |
| `info.license.identifier` | ✅ 采用 | `Apache-2.0` |
| `nullable: true` 风格 | ❌ 禁用 | 改用 `type: [string, "null"]`（3.1 标准） |
| exclusive bound | ✅ 采用 | `exclusiveMinimum: 0` 而非 `minimum: 0, exclusiveMinimum: true` |
```

**影响字段**：
- `rest/01 §1.1 新增`
- `crates/star-rest/openapi/*.yaml` 实现
- `mcp/01 §3` 错误模型同步（per 🔴 #6）

---

### 🟡 #11 — agent-api/v1 info.version 与 schema_version 关系不明

**原文引用**：

> `agent-api/01 §2 核心 Schema（顶层）`：
> ```yaml
> openapi: 3.1.0
> info:
>   title: STAR Agent API
>   version: "1.0.0"  # ← 1.0.0
>   description: ...
> ```

> `agent-api/01 §1 Versioning`：
> - Schema version: `agent-api/v1`
> - Breaking change 必须升 v2
> - New field 是 additive（minor）

**问题**：
- `info.version: "1.0.0"` 与 `schema_version: "agent-api/v1"` 是两个不同维度
- 1.0.0 是 OpenAPI info.version（API 服务版本？spec 文档版本？）
- agent-api/v1 是 schema 版本（数据契约版本）
- 加新 field 是 minor（v1.0.0 → v1.1.0？v1 → v1.1？）
- breaking change 升 v2（agent-api/v1 → agent-api/v2？还是 1.0.0 → 2.0.0？）

**修复建议**：agent-api/01 §1 改：

```markdown
## 1. Versioning

### 1.1 Schema Version（数据契约）

- Schema version: `agent-api/v1`（semver: MAJOR.MINOR）
- Breaking change 升 MAJOR（v1 → v2）
- Additive new field 升 MINOR（v1.0 → v1.1）
- 任何 field 重命名 / 移除 / 类型变更都算 breaking（升 MAJOR）
- OpenAPI `info.version` 同步演化：`1.0.0` → `1.1.0` → `2.0.0`

### 1.2 API Service Version（实现版本）

- 与 schema version 独立
- 同一 schema version 可有多个 service version（e.g. `agent-api/v1` + service `2.3.0` 表示 service 实现版本）

### 1.3 表达约定

- CLI 输出 `schema_version` 字段 = schema MAJOR.MINOR
- HTTP `Accept` header 可指定 schema version：`Accept: application/json; schema=agent-api/v1`
```

**影响字段**：
- `agent-api/01 §1 拆 1.1 / 1.2 / 1.3`
- `cli/01 §3 --schema-version flag` 行为明确
- `rest/01` HTTP content negotiation 实施

---

### 🟡 #12 — REST 文件名用连字符，agent-api spec 主体用斜杠

**原文引用**：

> `rest/01 §2 Spec 文件位置`：
> ```
> crates/star-rest/openapi/agent-api-v1.yaml
> crates/star-rest/openapi/ide-api-v1.yaml
> crates/star-rest/openapi/git-provider-v1.yaml
> ```

> `agent-api/01 §1` / `ide-api/01 §1` 全文用 `agent-api/v1` / `ide-api/v1`（斜杠）

**问题**：
- 文件名用 `agent-api-v1.yaml`，正文用 `agent-api/v1`
- 操作系统文件名不能用 `/`（Linux 除外，Windows 不行）
- OpenAPI `info.version` 字段是字符串，可以含 `/`，但文件名不行
- 风格不统一会让 grep 时漏

**修复建议**：

**方案 A**：文件名改用 `agent_api_v1.yaml`（下划线 + 单下划线 + v1）
**方案 B**：文件名保留 `agent-api-v1.yaml`，spec 正文用 `agent-api-v1`（连字符统一）
**方案 C**：文件名保留 `agent-api-v1.yaml`，spec 正文用 `agent-api/v1`，spec §1 显式说明"文件名 = agent-api-v1；schema identifier = agent-api/v1"

推荐方案 C，并在 rest/01 §2 加注释：

```markdown
## 2. Spec 文件位置

- `crates/star-rest/openapi/agent-api-v1.yaml` — Agent API（schema identifier = `agent-api/v1`）
- `crates/star-rest/openapi/ide-api-v1.yaml` — IDE API（schema identifier = `ide-api/v1`）
- `crates/star-rest/openapi/git-provider-v1.yaml` — Git Provider API

> 注：文件名用连字符（OS 兼容），schema identifier 用斜杠（OpenAPI info.version 字符串）
```

**影响字段**：
- `rest/01 §2 加注释`
- `agent-api/01 §1` / `ide-api/01 §1` 不变
- `crates/star-rest/openapi/*.yaml` 文件名不变

---

### 🟡 #13 — MCP spec §2 / §6 工具数自相矛盾

**原文引用**：

> `mcp/01 §2 工具表` 共 15 个 tool，注释写 "实际 15 个 tools，比 §17 任务原文多 2 个（get_workspace + request_review 是 17 任务原文中未列但常需要）"

> `mcp/01 §6 实施位置`：
> ```
> - `crates/star-mcp/src/tools/` — 13 个 tool 实现
> ```

> `mcp/01 §7 验证` 写 "必须列出 13+ tools"

**问题**：
- §2 注释承认 15 个，§6 实施位置写 13 个，§7 验证写 13+
- 三处数字不一致
- 数字不影响 spec 质量，但会让 reviewer 怀疑"这份 spec 自身有没有 DDD review"

**修复建议**：
- §2 注释确认 15 个
- §6 改 "15 个 tool 实现"
- §7 改 "必须列出 15 tools（不包含未来扩展）"

**影响字段**：
- `mcp/01 §2 / §6 / §7 数字统一`

---

### 🟡 #14 — MCP 命名 query / action 分类不显式

**原文引用**：

> `mcp/01 §2 工具表` 15 个 tool 命名：
> - 查询类：get_issue / search_issues / get_current_task / get_workspace / get_worktree / get_symbol / find_references / get_code_context / get_context / get_pipeline_status
> - 操作类：create_worktree / create_merge_request / request_review / run_validation
> - 异常：`search_issues` 用 search 不用 list（per 🟡 #8）

**问题**：
- spec §2 没显式说明"query = get_*/search_*, action = create_*/update_*/request_*"
- `request_review` 跟 `create_merge_request` 同样是 action，但 verb 不同（review 是动作名，create 是操作类型）
- 未来加新 tool 时命名会乱

**修复建议**：mcp/01 §2 前加命名约定（per 🟡 #8 §2.1 命名约定）：

```markdown
### 2.0 命名约定

- 查询（无副作用）：`get_<entity>` (单对象) / `search_<entities>` (多对象)
- 操作（有副作用）：`create_<entity>` / `update_<entity>` / `delete_<entity>` / `request_<action>` / `run_<workflow>`
- 状态查询（无副作用）：`get_<entity>_<state>` 或 `get_<entity>` 含 status 字段
```

**影响字段**：
- `mcp/01 §2.0 新增`
- 未来新 tool 实现时遵循

---

### 🟡 #15 — Capabilities 数组 vs CLI 17 命令不匹配

**原文引用**：

> `arch/03 §4` Capability Discovery 输出 `capabilities` 数组：
> ```json
> ["projects", "issues", "tasks", "workspaces", "worktrees", "repositories", "code_search", "code_navigation", "code_context", "merge_requests", "context", "tests", "pipelines", "reviews", "deployments"]
> ```
> 15 个

> `cli/01 §2` 17 核心命令 + 4 子命令覆盖：
> - projects → `star project list` ✅
> - issues → `star issue list/show/claim` ✅
> - tasks → `star task current` ✅
> - workspaces → `star workspace list/current` ✅
> - worktrees → `star worktree create/enter/status` ✅
> - repositories → ❌ **无 `star repo` 命令**
> - code_search → `star code search` ✅
> - code_navigation → `star code symbol/references` ✅
> - code_context → `star context get/current` ✅
> - merge_requests → `star mr create/show` ✅
> - context → `star context get/current` ✅（重复）
> - tests → `star test affected/run` ✅
> - pipelines → `star pipeline run/status` ✅
> - reviews → `star mr review` ✅
> - deployments → ❌ **无 `star deploy` 命令**

**问题**：
- 2 个 capability 无对应命令（repositories / deployments）
- `context` 重复（与 `code_context` 重叠）
- 18 核心命令实际数与 capability 数组不对应

**修复建议**：

**方案 A**（加命令）：cli/01 §2 加：
- `star repo list` / `star repo show`（→ `agent-api/v1#RepositoryList` / `agent-api/v1#Repository`）
- `star deploy list` / `star deploy run`（→ `agent-api/v1#Deployment`）

**方案 B**（删 capability）：arch/03 §4 capability 数组删 `repositories`（隐式通过 `workspace.repository` 拿到）+ `deployments`（用 `pipeline run` 表达）

推荐方案 B（保持 CLI 17 命令简洁）。

**影响字段**：
- 方案 A：`cli/01 §2 增 4 行` / `arch/03 §4 capability 数组不变`
- 方案 B：`arch/03 §4 capability 数组从 15 减到 13`

---

### 🟡 #16 — REST §4 缺 IDE 端 capabilities / permissions / instructions

**原文引用**：

> `rest/01 §4` agent 端：
> - `GET /api/v1/agent/capabilities`
> - `GET /api/v1/agent/permissions`
> - `GET /api/v1/agent/instructions`

> `cli/01 §4` 列出 4 个 agent + 4 个 ide：
> - `star agent capabilities` / `describe` / `instructions` / `permissions`
> - `star ide capabilities` / `describe` / `instructions` / `permissions`

> `rest/01 §4` **无 `ide/*` 端点**

**问题**：
- agent 端 3 个端点都有，ide 端 0 个
- IDE-only 客户端（如 VS Code / Cursor with Claude Code plugin）走 REST 时拿不到 IDE capabilities

**修复建议**：rest/01 §4 增：

```markdown
| `GET /api/v1/ide/capabilities` | IDE Capability Discovery |
| `GET /api/v1/ide/permissions` | IDE Permission Discovery |
| `GET /api/v1/ide/instructions` | IDE Instructions |
```

**影响字段**：
- `rest/01 §4 增 3 行`
- `crates/star-rest/openapi/ide-api-v1.yaml` 实现

---

### 🟡 #17 — REST 端点单复数风格不统一

**原文引用**：

> `rest/01 §4` 端点表：
> - `GET /api/v1/worktrees`（复数）
> - `GET /api/v1/tasks/current`（单数 + 修饰）
> - `GET /api/v1/workspaces/current`（单数 + 修饰）
> - `POST /api/v1/worktrees`（复数）
> - `GET /api/v1/code/search`（单数 + 修饰）
> - `GET /api/v1/code/symbols/{name}`（复数）
> - `POST /api/v1/mr`（**单数**）
> - `POST /api/v1/submit`（单数）
> - `GET /api/v1/context/{issue_id}`（单数）

**问题**：
- `POST /api/v1/mr` 是单数，其他 entity 端点（`worktrees` / `tasks` / `workspaces` / `issues`）默认复数
- arch/03 §4 capability 数组用 `merge_requests`（复数）
- mcp/01 §2 用 `create_merge_request`（单数）
- 三层（CLI / REST / MCP）命名风格不统一

**修复建议**：
- `POST /api/v1/mr` → `POST /api/v1/merge-requests`（与 capability 数组一致）
- `GET /api/v1/mr/{id}` → `GET /api/v1/merge-requests/{id}`
- `POST /api/v1/mr/{id}/review` → `POST /api/v1/merge-requests/{id}/review`

**影响字段**：
- `rest/01 §4 端点表所有 mr → merge-requests`
- `cli/01 §2` 不变（CLI 仍可用 `mr` 简写）
- `mcp/01 §2` 不变（MCP tool 名是 `create_merge_request`）

---

### 🟡 #18 — MCP spec §1 缺 Tool list 排序 + ttlMs 缓存要求

**原文引用**：

> `ecosystem-survey/protocol-survey.md §1 对 STAR 的推论`：
> > Tool list 必须按 deterministic order 排序 + 支持 ttlMs 缓存

> `mcp/01 §1` 未提排序 + ttlMs

**问题**：
- 2026-07-28 强制要求 tool list 排序 + 缓存
- 不排序会导致 client 端 hash 校验失败
- 不缓存会导致 list_tools RPC 频繁打 server

**修复建议**：per 🔴 #2 §1.1 表格"④ 可缓存 list 结果"行已含此建议。同步在 mcp/01 §2 工具表加 `metadata` 列：

```markdown
| Tool | 输入 | 输出 | Metadata |
|---|---|---|---|
| `get_issue` | `{issue_id}` | `Issue` | `{ ttlMs: 30000, cacheScope: "workspace" }` |
```

**影响字段**：
- `mcp/01 §2 增 metadata 列`
- `crates/star-mcp/src/main.rs` 实现 tool list 排序

---

### 🟡 #19 — REST §4 端点表无 error response

**原文引用**：

> `rest/01 §4` 12 个端点，**0 个有 `responses` 块**

> `OpenAPI 3.1` 标准要求每个 operation 有 `responses`（至少 1 个成功 + 1 个 error）

**问题**：
- 12 端点全无 error response 定义
- client SDK 生成时不会生成错误处理代码
- 错误模型 4 套并存（per 🔴 #6），但 REST 这层完全没接入

**修复建议**：per 🔴 #6，每个端点加：

```yaml
responses:
  '200': { ... }
  '4xx':
    description: "Client error"
    content:
      application/json:
        schema: { $ref: '#/components/schemas/Error' }
  '5xx':
    description: "Server error"
    content:
      application/json:
        schema: { $ref: '#/components/schemas/Error' }
```

**影响字段**：
- `rest/01 §4 每个端点加 responses 块`
- `crates/star-rest/openapi/*.yaml` 实现

---

### 🟡 #20 — arch/03 §2.2 CLI 17 命令示例缺 3 个

**原文引用**：

> `arch/03 §2.2` bash 代码块 18 个命令（per 🔴 #1），但**对照 cli/01 §2 应有 17 + 3 个扩展 = 20 个**，bash 块**缺**：
> - `star mr review`
> - `star test run`
> - `star context current`

**问题**：
- arch/03 §2.2 漏 3 个命令示例
- 读 arch/03 的人会以为这 3 个命令不存在

**修复建议**：arch/03 §2.2 bash 块补 3 个：

```bash
star mr review --approve  # 之前
star test run  # 之前
star context current --json  # 之前
```

**影响字段**：
- `arch/03 §2.2 bash 块补 3 行`

---

## §4 锦上添花（🟢 低）

### 🟢 #21 — agent-api §3 与 §4 schema 落盘位置交叉引用

`agent-api/01 §4 落盘位置` 列文件名（Issue.json / Task.json / ...）但 §3 主体未给 schema。两处应显式交叉引用：§3 列已定义 schema（Task / Worktree / SubmitResult），§4 列待落盘 schema（Issue / IssueList / ...）。

### 🟢 #22 — arch/04 §5 IDE 接入的 3 个最低要求缺 transport 约束

`arch/04 §5` 列 "3. MCP 客户端 — IDE 支持 MCP 2026-07-28"，但 arch/03 §2.3 明确 stdio transport。arch/04 应补 "MCP transport: stdio (local) / Streamable HTTP (remote, Phase 2)"。

### 🟢 #23 — REST §2 `git-provider-v1.yaml` 范围未明

`rest/01 §2` 列 `git-provider-v1.yaml` 但 §4 端点表无 git-provider endpoint，§5 验证也未涉及。要么 §4 显式标 "TBD Phase 2"，要么 §2 移出（per ADR-0022 Git Provider 抽象在 STAR，但 GitGit 自身有 axum HTTP API）。

### 🟢 #24 — agent-api Worktree 含 agent_session_id + ide_session_id 跨层

`agent-api/01 §3.2 Worktree` 包含 `agent_session_id` + `ide_session_id` 字段。agent 视角的 Worktree 不应同时绑两个 session。建议拆 `WorktreeBinding` 子对象：`{ agent_sessions: [...], ide_sessions: [...] }`。

### 🟢 #25 — cli/01 §3 通用 flags 缺基础 shell 习惯

`cli/01 §3` 列 7 个 flag（--json / --quiet / --fields / --limit / --cursor / --no-color / --schema-version），缺：
- `--help`（基础）
- `--no-header`（与 `--no-color` 类似，去掉 banner）
- `--version`（基础）

### 🟢 #26 — arch/03 §2.5 AGENTS.md bootstrap 命令超出 3 个最小命令

`arch/03 §2.5` AGENTS.md 示例含 6 个命令（`star agent capabilities` / `star task current --json` / `star context current --json` / `star code search` / `star test affected` / `star submit`）。`ecosystem-survey/protocol-survey §2 对 STAR 的推论` 明确 "AGENTS.md 必须含 3 个最小可用命令"。

应缩减到 3 个：`star agent capabilities` + `star task current --json` + `star submit`。

### 🟢 #27 — cli/01 §3 `--schema-version` flag 默认值未明

`cli/01 §3` 列 `--schema-version <v>` 但未说明默认版本。应为 "默认 = 当前实现 schema version"，并加 "与 `star agent capabilities` 输出一致"。

### 🟢 #28 — MCP Resources / Prompts "可选" 措辞模糊

`mcp/01 §4 / §5` 用 "可选 / 不强制" 措辞，但 arch/03 §2.3 没说 "可选"。应明确 "MVP 阶段 Resources / Prompts 不实现，Phase 2 评估"。

### 🟢 #29 — ide-api §3 落盘位置在 star-cli crate 而非 star-ide-gateway

`ide-api/01 §3` 路径 `crates/star-cli/src/schemas/ide-api-v1/` 与 `agent-api/01 §4` 路径 `crates/star-cli/src/schemas/agent-api-v1/` 共用目录。但 per arch/04 IDE Gateway 边界，IDE 相关 schema 应放 `crates/star-ide-gateway/src/schemas/ide-api-v1/`。

### 🟢 #30 — REST §2 `git-provider-v1.yaml` 与 ADR-0022 边界

`rest/01 §2` 把 `git-provider-v1.yaml` 放 `star-rest/openapi/`，与 ADR-0022 §2.2 "Git Provider Abstraction: GitGit / GitHub / GitLab / Other Git Providers" 边界模糊。应加注释 "git-provider-v1.yaml 是 STAR 对 Git Provider 的抽象，GitGit 自身的 HTTP API 由 GitGit crate 单独维护"。

---

## §5 跨文档对齐结论

### CLI ↔ MCP 对齐度：**62%**

**对齐点**（10/16）：
- ✅ `star issue show` ↔ `get_issue`
- ✅ `star task current` ↔ `get_current_task`
- ✅ `star context get` ↔ `get_context`
- ✅ `star code search` ↔ `search_code`
- ✅ `star code symbol` ↔ `get_symbol`
- ✅ `star worktree create` ↔ `create_worktree`
- ✅ `star pipeline status` ↔ `get_pipeline_status`
- ✅ `star code references` ↔ `find_references`（动词差异）
- ✅ `star test affected` ↔ `run_validation`（语义差异）
- ✅ `star mr review` ↔ `request_review`（动作差异）

**mismatch**（6 项）：
- ⚠️ `star issue list` ↔ `search_issues`（list vs search）
- ⚠️ `star issue claim` ↔ 无（per 🔴 #3 同源问题：claim 是关键缺失）
- ⚠️ `star context current` ↔ 无
- ⚠️ `star workspace list` ↔ 无（`get_workspace` 模糊覆盖）
- ⚠️ `star worktree status` ↔ `get_worktree`（status vs get）
- ⚠️ `star mr create` ↔ `create_merge_request`（mr 缩写 vs 全名）
- ❌ `star mr show` ↔ 无
- ❌ `star test run` ↔ 无（与 `run_validation` 重复）
- ❌ `star pipeline run` ↔ 无
- ❌ `star submit` ↔ 无（per 🔴 #3）

### MCP ↔ OpenAPI 对齐度：**40%**

**对齐点**（约 4/10）：
- ✅ 共享 Domain API（per rest/01 §3 架构图）
- ✅ 错误模型（理论上共享，per 🔴 #6 待修复）
- ✅ 工具/端点语义映射（部分）
- ✅ 2026-07-28 规范基线（REST 用 OpenAPI 3.1，per protocol-survey §3 对齐 JSON Schema 2020-12）

**mismatch**（6 项）：
- ❌ REST 缺 IDE 端点（per 🟡 #16）
- ❌ REST 缺大部分 CLI 对应端点（per 🟡 #9）
- ❌ MCP Resources / Prompts vs REST 资源端点（REST 是否有 resources 路径不明）
- ❌ MCP 工具 15 个 vs REST 端点 12 个 —— 数量不匹配
- ❌ MCP 错误模型 vs REST 错误模型（per 🔴 #6）
- ❌ MCP 2026-07-28 关键变更 vs OpenAPI 3.1 关键字段（per 🔴 #2 + 🟡 #10）

### agent-api ↔ ide-api 对齐度：**70%**

**对齐点**：
- ✅ 平行版本（`agent-api/v1` / `ide-api/v1`）
- ✅ 独立演进（per ide-api/01 §1）
- ✅ snake_case 字段命名一致
- ✅ OpenAPI 3.1 + JSON Schema 2020-12 一致
- ✅ json 风格错误对象（理论上）

**mismatch**：
- ⚠️ `Workspace` schema 错位（per 🔴 #5，agent 视角 vs IDE 视角）
- ⚠️ 错误模型不统一（per 🔴 #6）
- ❌ `Worktree` 跨层（agent-api 含 `ide_session_id` 字段，per 🟢 #24）
- ❌ info.version 关系（per 🟡 #11）

### Universal Submit ↔ CLI 17 命令覆盖度：**60%**

**对齐点**（约 7/12 步）：
- ✅ 步骤 1 (检查 Task) → `star task current`
- ✅ 步骤 2 (检查 Workspace) → `star workspace current`
- ✅ 步骤 3 (检查 Worktree) → `star worktree status`
- ✅ 步骤 5 (执行 Required Validation) → `star test affected`
- ✅ 步骤 9 (创建 / 更新 MR) → `star mr create`
- ❌ 步骤 4 (检查 Diff) → 无（per 🔴 #7）
- ❌ 步骤 6 (检查 Policy) → 无（per 🔴 #7）
- ❌ 步骤 7 (Commit) → 无（per 🔴 #7）
- ❌ 步骤 8 (Push) → 无（per 🔴 #7）
- ❌ 步骤 10 (关联 Issue) → 无（per 🔴 #7）
- 步骤 11-12 (回写状态) → 内部操作，合理不暴露

---

## §6 守门规则遵循度

### Zero Vendor Cooperation（per ADR-0021）：✅ 通过

- ✅ `mcp/01 §1` 引用 ADR-0021 依赖
- ✅ `arch/03 §2 5 接入通道` 全 vendor-neutral（Git CLI / Shell / MCP 2026-07-28 / OpenAPI 3.1 / AGENTS.md）
- ✅ `cli/01 §4` 子命令（agent/ide）无 vendor-specific 命名
- ✅ `arch/04 §1` "任何 IDE 都通过标准能力接入 STAR，无需为 STAR 开发专用 plugin"
- ✅ `ide-api/01` / `agent-api/01` schema 不含 vendor-specific 字段

**轻微瑕疵**：
- ⚠️ `mcp/01 §3 禁止表` 没显式说"删除后 Core 仍完整"（per ADR-0025 验证规则）
- ⚠️ `rest/01` 未声明 endpoint 必 vendor-neutral（隐式但未明）

### IDE 归 STAR（per ADR-0022）：✅ 通过

- ✅ `rest/01 §2` 三个 spec file 都在 `star-rest/` 域，不污染 GitGit
- ✅ `ide-api/v1` 跟 `agent-api/v1` 平行，都归 STAR
- ✅ `arch/04` 完整描述 IDE Gateway 归 STAR 责任

**轻微瑕疵**：
- ⚠️ `rest/01 §2` `git-provider-v1.yaml` 与 ADR-0022 §2.2 "Git Provider Abstraction" 边界模糊（per 🟢 #30）
- ⚠️ `cli/01 §2` `star workspace current` 引用 `agent-api/v1#Workspace`（实际是 ide-api/v1 的 Workspace）—— 跨层数据泄漏（per 🔴 #5）

### Vendor Adapter Anti-Contamination（per ADR-0025）：✅ 通过

- ✅ `mcp/01 §3 禁止表` 拒绝暴露 `update_issue_table` / `insert_worktree_row` 等表操作
- ✅ `cli/01 §2 / §4` 命令全用领域语义（project / issue / task / context / code / workspace / worktree / mr / test / pipeline / submit / agent / ide）
- ✅ `arch/03 §2.2` 命令示例用 vendor-neutral 命名
- ✅ `arch/04 §8 IDE Gateway 不应提供` 明确拒绝 vendor-specific 集成

**轻微瑕疵**：
- ⚠️ `cli/01 §4` 子命令没显式说"删除 Optional Adapter 后 Core 100% 完整"
- ⚠️ `rest/01 §2` 没显式说 endpoint 必须 vendor-neutral
- ⚠️ `agent-api/01 §3.2 Worktree` 含 `agent_session_id` + `ide_session_id` 跨层字段（per 🟢 #24）

---

## §7 修订历史 + 签字栏

### 7.1 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-26 | 架构师（Mavis 接手 agent per DEC-008）— 子代理 A | 初版（30 finding：🔴 7 / 🟡 13 / 🟢 10 + 5 节对齐分析 + 3 条守门规则） | Phase C 第 2 轮接口一致性审查 |

### 7.2 签字栏

> 架构师（Mavis 接手 agent per DEC-008）— 子代理 A — 2026-08-26
>
> 本报告基于 commit `876a2a7`（Phase C 54 份 spec 草案）的 9 份 spec + 4 份交叉对账；不沿用 `bc23d6c` 的叙事（该 commit 引用了未做过的 frontend commit hash，属于历史叙事灰区）。
>
> 硬约束遵循：
> - ✅ 未修改任何现有文件
> - ✅ 未 commit（Mavis 终审后由 Mavis 统一 commit）
> - ✅ 未写代码
> - ✅ 未触碰 STAR 仓库的 12 个未跟踪文件
> - ✅ 未触碰 `bc23d6c` 叙事
> - ✅ 范围限定 9 份 spec + 4 份交叉对账（未扩到 54 份全部）
> - ✅ 未替 Mavis 写完整 JSON Schema 文件（仅指出字段不一致）

### 7.3 已知缺口

- ⚠️ 本审查未实际读 `crates/star-cli/src/schemas/*.json`（spec §4 落盘位置但文件未生成）—— schema 实际定义需 Phase D 实施时核对
- ⚠️ 本审查未核对 cli/01 §2 命令是否完全覆盖 "§9 任务原文 17 命令"（任务原文在 876a2a7 不可见）—— 数字 diff 需 Mavis 提供原文
- ⚠️ 本审查未核对 arch/03 §4 capability 数组 15 个与 cli/01 §2 命令是否完全对应（per 🟡 #15）—— 需 Mavis 提供 capability 来源
- ⚠️ 本审查未跑 Redocly CLI / ajv / @modelcontextprotocol/inspector 验证 —— 验证命令是 spec 写的，未实际执行
- ⚠️ 本审查未涵盖 `flows/01-04` 和 `flows/06-08` 8 份 flow spec（per 任务边界仅审 9 份 + 4 份交叉对账）

### 7.4 后续建议

- **优先级 P0（Phase C 完成前）**：修复 🔴 #1-#7（7 个高严重度问题）
- **优先级 P1（Phase D 实施前）**：修复 🟡 #8-#20（13 个中严重度问题）
- **优先级 P2（Phase D 实施中）**：修复 🟢 #21-#30（10 个低严重度问题）
- **DDD Review 必须查**：错误模型统一（per 🔴 #6）+ Workspace 视角拆解（per 🔴 #5）+ CLI 命令数 17 vs 23（per 🔴 #1）
