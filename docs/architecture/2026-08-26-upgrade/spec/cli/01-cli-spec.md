# 11. STAR CLI Specification

> **状态**：🟡 草案 v0.2
> **依赖**：[arch/03 STAR AI Compat Arch](../../arch/03-star-ai-compat-arch.md)

## 1. 设计原则

- 表达 Software Engineering Domain Semantics（不只 REST API 映射）
- Machine-readable 优先于 human-readable
- --json 必须稳定（versioned as `agent-api/v1`）
- Fallback 兼容 shell 解析
- **`star` 是 `git` 的 superset**（per P1-H 修复 2026-08-27） — `star` 提供 `git` 不具备的领域操作（issue / mr / workspace / submit），并包装 `git` 子命令（diff / commit / push）以注入 Policy / Audit / Worktree 上下文；`star` **不**替代 `git`，所有 Git 协议能力继续由 GitGit 提供（per [arch/02 §2 IDE Capability Boundary](../../arch/02-ide-capability-boundary.md)）

## 2. 核心命令

### 2.1 MVP 17 核心命令（per 任务原文 §9，MVP 子集边界）

> "schema 引用"列 = `agent-api/v1` spec 内对应 schema 段，详见 [spec/agent-api/01-schema.md §3](../agent-api/01-schema.md)：
> ProjectList / IssueList / Issue / ClaimResult / CurrentTask / Context / CodeSearchResult / SymbolResult / WorkspaceList / WorkspaceSummary / Worktree / WorktreeStatus / MR / TestResult / SubmitResult = 15 个 schema（部分命令输出同 schema 如 `star workspace list` / `current` / `star mr create` / `show`）。

| 命令 | 用途 | schema 引用 | agent-api/01 §3 节 |
|---|---|---|---|
| `star project list` | 列出项目 | `agent-api/v1#ProjectList` | §3.x（待补）|
| `star issue list` | 列出 issue | `agent-api/v1#IssueList` | §3.5 |
| `star issue show <id>` | 显示 issue 详情 | `agent-api/v1#Issue` | §3.4 |
| `star issue claim <id>` | 认领 issue | `agent-api/v1#ClaimResult` | §3.x（待补）|
| `star task current` | 当前任务 | `agent-api/v1#CurrentTask` | §3.6 |
| `star context get <id>` | 获取 context | `agent-api/v1#Context` | §3.8 |
| `star code search <q>` | 搜索代码 | `agent-api/v1#CodeSearchResult` | §3.9 |
| `star code symbol <name>` | 符号定位 | `agent-api/v1#SymbolResult` | §3.10 |
| `star workspace list` | 列出 workspace | `agent-api/v1#WorkspaceList` | §3.x（待补）|
| `star workspace current` | 当前 workspace | `agent-api/v1#WorkspaceSummary` | §3.16（per P1-C 修复，agent 视角逻辑抽象，**不**是 §3.x `Workspace`）|
| `star worktree create <id>` | 创建 worktree | `agent-api/v1#Worktree` | §3.2 |
| `star worktree enter <id>` | 进入 worktree | n/a (cd) | — |
| `star worktree status` | worktree 状态 | `agent-api/v1#WorktreeStatus` | §3.11 |
| `star mr create` | 创建 MR | `agent-api/v1#MR` | §3.7 |
| `star mr show <id>` | MR 详情 | `agent-api/v1#MR` | §3.7 |
| `star test affected` | 跑受影响测试 | `agent-api/v1#TestResult` | §3.12 |
| `star submit` | Universal Submit | `agent-api/v1#SubmitResult` | §3.3 |

> **MVP 退出条件** acceptance/04 §3 第一条 = 这 17 命令全部可调用 + `--json` 稳定 schema。
>
> **数字基线（per F-01 修复 2026-08-27）**：本表 17 行 = MVP 核心；§2.2 11 行 = 扩展（非 MVP）；合计 17+11=**28** 个 CLI 命令。不再出现 17/18/23 三处数字打架（v0.1 23 单表已拆开）。

#### 2.1.1 bash 块示例 17 核心命令（per F-01 修复 2026-08-27）

```bash
# MVP 17 核心命令 (per 任务原文 §9 + acceptance/04 §3 退出条件)
star project list --json
star issue list --json
star issue show STAR-1024 --json
star issue claim STAR-1024 --json
star task current --json
star context get STAR-1024 --json
star code search "auth.rs" --limit 20 --json
star code symbol "verify_token" --json
star workspace list --json
star workspace current --json
star worktree create STAR-1024 --branch feat/auth-fix
star worktree enter wt-STAR-1024
star worktree status --json
star mr create --title "fix: auth" --description "..." --json
star mr show 42 --json
star test affected --json
star submit --json
```

> 这 17 行 bash 命令与上表 17 行一一对应；任何 spec 改动都需同时更新本块 + 上表，避免数字漂移。

### 2.2 扩展命令（非 MVP 子集，共 11 个）

| 命令 | 用途 | 输出 schema | 来源 |
|---|---|---|---|
| `star context current` | 当前 context | `agent-api/v1#Context` | 原 §2 表内已有，arch/03 §2.2 缺 |
| `star code references <name>` | 引用查找 | `agent-api/v1#ReferencesResult` | 原 §2 表内已有，arch/03 §2.2 缺 |
| `star mr review <id>` | Review MR | `agent-api/v1#ReviewResult` | 原 §2 表内已有，arch/03 §2.2 缺（per F-20 修复 2026-08-27）|
| `star test run` | 跑全部测试 | `agent-api/v1#TestResult` | 原 §2 表内已有，arch/03 §2.2 缺（per F-20 修复 2026-08-27）|
| `star pipeline run` | 跑 pipeline | `agent-api/v1#PipelineRun` | 原 §2 表内已有，arch/03 §2.2 缺 |
| `star pipeline status` | pipeline 状态 | `agent-api/v1#PipelineStatus` | 原 §2 表内已有，arch/03 §2.2 缺 |
| `star diff` | Diff 检查（Universal Submit 第 4 步暴露） | `agent-api/v1#DiffResult` | **P1-H 新增**（per 2026-08-27） |
| `star policy check` | Policy 检查（Universal Submit 第 6 步暴露） | `agent-api/v1#PolicyCheckResult` | **P1-H 新增**（per 2026-08-27） |
| `star commit` | Commit（注入 Policy / Audit / Worktree 上下文） | `agent-api/v1#CommitResult` | **P1-H 新增**（per 2026-08-27） |
| `star push` | Push（注入 Audit 上下文） | `agent-api/v1#PushResult` | **P1-H 新增**（per 2026-08-27） |
| `star mr link <id>` | 关联 Issue 到 MR（Universal Submit 第 10 步暴露） | `agent-api/v1#MRLinkResult` | **P1-H 新增**（per 2026-08-27） |

> 6 个原扩展命令（context current / code references / mr review / test run / pipeline run / pipeline status）业务语义完整但不在 MVP 退出条件 17 之列，作为 Phase 2+ 候选。5 个新加命令（diff / policy check / commit / push / mr link）对应 Universal Submit 12 步流程中原本**没有**独立 CLI 命令的 5 步（第 4 / 6 / 7 / 8 / 10 步），per P1-H 修复 2026-08-27。

#### 2.2.1 bash 块示例 11 扩展命令（per F-20 修复 2026-08-27，补全 arch/03 §2.2 漏 3 个）

```bash
# 6 个原扩展（arch/03 §2.2 bash 块漏：mr review / test run / context current / 已补）
star context current --json                    # 之前 arch/03 §2.2 漏
star code references "verify_token" --json
star mr review 42 --approve                    # 之前 arch/03 §2.2 漏
star test run --json                           # 之前 arch/03 §2.2 漏
star pipeline run --json
star pipeline status --json

# 5 个 P1-H 新增 (Universal Submit 第 4/6/7/8/10 步)
star diff HEAD~1 --json                        # Universal Submit step 4
star policy check --json                       # Universal Submit step 6
star commit -m "fix: auth" --json              # Universal Submit step 7
star push origin feat/auth-fix --json          # Universal Submit step 8
star mr link 42 --issue STAR-1024 --json       # Universal Submit step 10
```

### 2.3 Capabilities 范围声明（per F-15 修复 2026-08-27）

per [arch/03 §4 Capability Discovery](../../arch/03-star-ai-compat-arch.md) 的 capability 数组：
`["projects", "issues", "tasks", "workspaces", "worktrees", "repositories", "code_search", "code_navigation", "code_context", "merge_requests", "context", "tests", "pipelines", "reviews", "deployments"]` = 15 个。

**CLI MVP 17 核心 + 11 扩展 = 28 命令的 capability 覆盖**：

| capability | CLI 命令 | 覆盖状态 |
|---|---|---|
| `projects` | `star project list` | ✅ MVP |
| `issues` | `star issue list/show/claim` | ✅ MVP |
| `tasks` | `star task current` | ✅ MVP |
| `workspaces` | `star workspace list/current` | ✅ MVP |
| `worktrees` | `star worktree create/enter/status` | ✅ MVP |
| `repositories` | （无）| ⚠️ **CLI MVP 不覆盖**（隐式通过 `workspace.repository` 字段拿到，per arch/03 §4 抽象） |
| `code_search` | `star code search` | ✅ MVP |
| `code_navigation` | `star code symbol/references` | ✅ MVP |
| `code_context` | `star context get/current` | ✅ MVP + 扩展 |
| `merge_requests` | `star mr create/show/review` | ✅ MVP + 扩展 |
| `context` | `star context get/current` | ✅ MVP + 扩展（与 `code_context` 字段重叠，per F-15 note） |
| `tests` | `star test affected/run` | ✅ MVP + 扩展 |
| `pipelines` | `star pipeline run/status` | ✅ 扩展（非 MVP）|
| `reviews` | `star mr review` | ✅ 扩展（非 MVP）|
| `deployments` | （无）| ⚠️ **CLI MVP 不覆盖**（用 `star pipeline run` 表达部署，per arch/03 §4 抽象） |

> **修复结论（per F-15）**：`repositories` 和 `deployments` 两个 capability 在 CLI MVP 28 命令中无独立命令。CLI `star agent capabilities` 输出这 15 个 capability，但其中 2 个标"CLI 间接覆盖"（不直接暴露命令，依赖 `workspace.repository` / `pipeline run` 派生）。arch/03 §4 capability 数组**不删除**这两项（向后兼容 arch/03），仅在本 spec 标注覆盖关系。
>
> **不修 arch/03 §4**（per 任务边界：仅改 cli/01 + mcp/01）—— arch/03 §4 capability 数组保留 15 项，本 spec 负责"声明 CLI MVP 不直接覆盖哪几个"，承担"能力存在但 CLI 不暴露"的语义。

### 2.4 命名风格约定（per F-08 修复 2026-08-27）

CLI 命令命名风格（与 [mcp/01 §2.1 命名约定](../mcp/01-mcp-spec.md) 对齐）：

- **查询**（多对象）：`list` 动词（CLI 习惯）→ MCP `search_*` 表达（per F-08 #8 命名差异，CLI 用 `list`，MCP 用 `search_*`）
- **查询**（单对象）：`show` 动词（CLI）→ MCP `get_*` 一致
- **操作**：`create` / `claim` / `link` 动词（CLI）→ MCP `create_*` / `claim_*` / 等（保持动词一致）
- **缩写**：`mr` 缩写（CLI shell 习惯，per F-08） → MCP `merge_request` 全名（machine 协议）
- **测试**：`test` 表达（CLI 业务语义）→ MCP `validation` 表达（per F-08 命名差异，CLI 视角 vs STAR 内部）
- **状态查询**：`current` 修饰（CLI，空入参表示当前）→ MCP 单 tool + 空对象入参（per `get_current_task` / `get_workspace`）

### 2.5 CLI ↔ MCP 命名映射表（per F-08 修复 2026-08-27）

| CLI 命令 | MCP tool | 命名差异 | 备注 |
|---|---|---|---|
| `star project list` | （待定，可能 `list_projects`） | `list` vs `list_*` | Phase 2 评估是否加 list_* |
| `star issue list` | `search_issues` (empty query) | `list` vs `search_*` | MCP 统一用 `search_*` 表达列表 |
| `star issue show <id>` | `get_issue` | 一致 | — |
| `star issue claim <id>` | （待定，可能 `claim_issue`） | — | 待 MCP Phase 2 加 |
| `star task current` | `get_current_task` | 一致（CLI 用 current，MCP 用 `get_*_current_*`） | — |
| `star context get <id>` | `get_context` | 一致 | — |
| `star context current` | （无对应） | — | ⚠️ MCP 缺，per F-08 跨层缺口 |
| `star code search <q>` | `search_code` | 一致（CLI / MCP 主体一致，只是宾语位置不同） | — |
| `star code symbol <name>` | `get_symbol` | `symbol` vs `get_*` | MCP 把 symbol 视为 noun，CLI 视为 verb |
| `star code references <name>` | `find_references` | `references` vs `find_*` | MCP 动词不同 |
| `star workspace list` | （无对应） | — | ⚠️ MCP 缺，per F-08 跨层缺口 |
| `star workspace current` | `get_workspace` (空入参) | `current` vs `get_*` | 单 tool 覆盖 + agent 视角 |
| `star worktree create` | `create_worktree` | 一致 | — |
| `star worktree status` | `get_worktree` | `status` vs `get_*` | CLI 状态视角，MCP 单 tool 覆盖 |
| `star mr create` | `create_merge_request` | `mr` vs `merge_request` | CLI 缩写 shell 习惯 |
| `star mr show <id>` | （无对应） | — | ⚠️ MCP 缺 `get_mr`，per F-08 跨层缺口 |
| `star mr review <id>` | `request_review` | `review` vs `request_*` | MCP 用动作化命名 |
| `star test affected` | `run_validation` (scope=affected) | `test` vs `validation` | MCP 统一 `validation` 表达测试+检查 |
| `star test run` | `run_validation` | 同上 | — |
| `star pipeline run` | （无对应） | — | ⚠️ MCP 缺，per F-08 跨层缺口 |
| `star pipeline status` | `get_pipeline_status` | 一致 | — |
| `star submit` | `submit` | 一致 | ✅ Universal Submit 双层一致 |

> **跨层缺口（per F-08）**：5 个 CLI 命令无对应 MCP tool — `star context current` / `star workspace list` / `star mr show` / `star pipeline run` / `star issue claim`（待 MCP 加）。这些缺口**不阻塞 MVP**，但 Phase 2 补齐。
> **CLI / MCP 缩写差异是设计选择**（per F-08 修复建议）：CLI 走 shell 短名（`mr`），MCP 走 machine 全名（`merge_request`），不强制统一。

## 3. 通用 flags

| flag | 说明 | F- 修复 |
|---|---|---|
| `--json` | 强制 JSON 输出 | — |
| `--quiet` | 只输出 ID / 摘要 | — |
| `--fields k1,k2` | 限制输出字段 | — |
| `--limit N` | 限制行数 | — |
| `--cursor <c>` | 分页游标 | — |
| `--no-color` | 关闭 ANSI | — |
| `--schema-version <v>` | 显式 schema 版本，**默认 `v1`（= `agent-api/v1`）**，与 `star agent capabilities` 输出一致 | F-27 |
| `--help` / `-h` | 显示帮助（基础 shell 习惯，per F-25 修复 2026-08-27） | F-25 |
| `--no-header` | 关闭 banner / header 行（与 `--no-color` 类似，per F-25 修复 2026-08-27） | F-25 |
| `--version` / `-V` | 显示 `star` 版本（基础 shell 习惯，per F-25 修复 2026-08-27） | F-25 |

> **F-27 修复（per 2026-08-27）**：`--schema-version <v>` 默认值 = `v1`（= `agent-api/v1`）。`v1` 是当前 `star agent capabilities` 输出的 schema version，与 spec/agent-api/01-schema.md §1 OpenAPI `info.version` 同步演化（per INTERFACE-REVIEW-A 🟡 #11）。如果调用方传 `--schema-version v2` 但 server 未实现 v2，server 报 `SCHEMA_VERSION_UNSUPPORTED` 错误（per agent-api/v1#Error）。

## 4. 子命令

```bash
star agent capabilities       # Capability Discovery
star agent describe <cmd>     # 单命令详细 schema
star agent instructions      # 当前环境的 AI 操作说明
star agent permissions       # 权限查询

star ide capabilities
star ide describe <cmd>
star ide instructions
star ide permissions
```

## 5. 错误模型（per §11，统一权威 schema）

```json
{
  "error": "WORKTREE_CONFLICT",
  "recoverable": true,
  "suggested_actions": ["inspect_conflict", "request_rebase"],
  "message": "Worktree STAR-1024 has uncommitted changes conflicting with main",
  "trace_id": "...",
  "details": {"worktree_id": "wt-STAR-1024", "conflicting_files": ["src/auth.rs"]}
}
```

> 字段：`error` / `recoverable` / `suggested_actions` / `message` / `trace_id` / `details`（6 字段，per F-06 修复 2026-08-27）— 5 字段原版基础上 + `details` 与 `agent-api/v1#Error`（per [spec/agent-api/01-schema.md §3.15 Error](../agent-api/01-schema.md)，W4 子代理定义，per P1-G 修复 2026-08-27）完全对齐。CLI / MCP / REST / Universal Submit **全部**引用同一份 `Error` schema。
>
> **F-06 引用约定（per 2026-08-27）**：本 spec 引用 `agent-api/01-schema.md §3.15 Error`（**不**重新定义 6 字段）。任务原始描述 "§3.14" 是 W4 子代理修复时的初稿编号，正式落盘后 `Capabilities` 占 §3.14，`Error` 落 §3.15。统一以落盘节号为准，**避免改一处破全部 spec**。

## 6. 实施位置

- `crates/star-cli/` — 主 binary
- `crates/star-cli/src/commands/` — 子命令模块
- `crates/star-cli/src/output.rs` — JSON schema 输出

## 7. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-26 | Mavis（per DEC-008）| 初版：23 命令单表 | Phase C 54 份 spec 草案 |
| v0.2 | 2026-08-27 | Ulysses（一人公司 12 角色 per DEC-008）| P1-A：§2 拆 17 核心 + 11 扩展（标 MVP 子集边界） · P1-C：`star workspace current` 引用改 `WorkspaceSummary` · P1-G：§5 错误模型 6 字段 + 引用 `agent-api/v1#Error` · P1-H：§1 加 "`star` 是 `git` superset" 原则 + §2.2 增 5 个新命令（diff / policy check / commit / push / mr link） | 8 子代理 INTERFACE-REVIEW-A 🔴 #1/#5/#6/#7 + P1-BLOCKERS-SUMMARY v0.2 |
| v0.2 fix | 2026-08-27 | Mavis（接手 agent per DEC-008）| **F-01**：§2.1.1 加 17 核心命令 bash 块（数字基线 17+11=28） · **F-04**：§2.1 表格加 "agent-api/01 §3 节" 列（17 命令对应 schema 节号） · **F-06**：§5 错误模型明确引用 `§3.15 Error`（注：W4 初稿编号 §3.14，落盘后 Error 在 §3.15） · **F-08**：§2.4 + §2.5 加 CLI 命名风格约定 + CLI ↔ MCP 命名映射表（标 5 跨层缺口） · **F-15**：§2.3 加 Capabilities 范围声明（`repositories` / `deployments` 不直接覆盖） · **F-20**：§2.2.1 加 11 扩展命令 bash 块（补 arch/03 §2.2 漏的 mr review / test run / context current） · **F-25**：§3 加 `--help` / `--no-header` / `--version` 3 个基础 flag · **F-27**：§3 `--schema-version <v>` 默认 `v1`（=`agent-api/v1`），与 `star agent capabilities` 输出一致 | 8 子代理 INTERFACE-REVIEW-A 🟡 #8/#15/#20 + 🟢 #25/#27 |
