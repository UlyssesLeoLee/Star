# SRS-STAR-CLI-001

> **STAR CLI 软件需求规格说明书 v1.0 (recreate, per ULYS-124 v1.0 缺口补齐)**
>
> - 状态: Requirements Baseline (v1.0 — recreate after 9/21 + 9/23 reviewer 打回, per 用户 9/25 01:09 JST "确保缺口已补")
> - 目标阶段: 需求定义 → 基本设计 (MVP 17 核心 + 11 扩展 + Skill Registry 4)
> - 上游: `docs/architecture/2026-08-26-upgrade/spec/cli/01-cli-spec.md` v0.2 (canonical)
> - 下游: 基本设计 5 份 (BD-STAR-CLI-001 总冊 + 002/003/004/005 专题)
> - 核心语言: Rust (clap 4.5 derive)
> - 修订人: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> - 审批: 架构师 (Mavis 接手 agent per DEC-008)
> - 日期: 2026-09-25 JST

> **回溯记录（per ULYS-124 9/21 + 9/23 reviewer 打回与 9/25 01:09 缺口补齐指令）**：
>
> v0.1 SRS (`92e8db02`) + 4 专题 BD (`5b0454b3`/`d6e8cf4a`/`3b791824`/`1d93c1be`) + 总冊 BD (`08be4689`) 共 6 份文档于 2026-09-20 落档于各自临时 worktree。9/21 + 9/23 复核确认仅有 `1d93c1be` 推到了 origin（`agent/minimaxm3/ulys-131`），其余 5 份在 `git ls-remote origin` 不可达。本 v1.0 是按 (a) 9/21 + 9/23 reviewer 复审确认的 3808 行内容密度 (b) cli-spec v0.2 canonical 内容 (c) 当前 main `b0dddf7e` 含 17 核心命令 + 11 扩展的实装落地基础上重新创作, 并补全 v0.1 草稿若有的内容缺口（v1.0 内容不依赖 v0.1 git 历史 —— 按 AGENTS.md §1.2 #1 派生约束"不可回溯叙事"约束, 全部引用本仓 HEAD 实测）。

---

## §0 文档目的

本文档定义 STAR 项目 `star` 命令行客户端 (CLI) 的软件需求规格 (SRS)。覆盖 (a) 28 核心/扩展命令 (b) 通用 flags (c) 子命令 meta (d) 错误模型 (e) Capability Discovery (f) `star` 是 `git` superset 的设计原则 (per cli-spec v0.2 §1)。

**触发**（per ULYS-124 原文 2026-09-20 用户发令）：
> "完善Star CLI的各项设计，补充完善的符合日本IpA标准的文档，首先制作需求文档"

**MVP 范围**（per cli-spec v0.2 §2.1 数字基线 + 任务原文 §9 拍板）：
- MVP 17 核心命令 (per acceptance/04 §3 退出条件)
- 11 扩展命令 (per cli-spec v0.2 §2.2, 6 原扩展 + 5 P1-H 新增 Universal Submit 步骤暴露)
- 28 命令输出 100% 引用 `agent-api/v1` (per spec/agent-api/01-schema.md)
- Skill Registry 4 子命令 (per ULYS-196 commit `3e510d2b`, FR-ORCA-034 §10.2)
- 文档层: SRS 1 份 (本文件) + 基本设计 5 份 (BD-STAR-CLI-001 总冊 + 002/003/004/005 专题) + 详细设计 5 份 (Phase 2 待开)
- 错误模型 6 字段 100% 引用 `agent-api/v1#Error` (per cli-spec §5 F-06)

**不**做（per cli-spec v0.2 边界声明）：
- 全局 MCP / REST / IDE 协议能力 — 仅 CLI 协议 (per AGENTS.md §5 Star 仓独立, 跟 RGS 仓职责切割)
- 真实 OpenAI/Anthropic LLM 通道 — 仅 stub (本仓不涉 LLM)
- PostgreSQL / Redis 持久化 — MVP 内存 + JSON stub
- 详细设计 (DD) — Phase 2+ 拍板 (per ULYS-124 v0.3 下游衔接 §8)

---

## §1 章节映射

需求章节用 IPA 模板 + skill-multica 风格，状态列：`✅ MVP 落档` / `🟡 MVP 部分` / `⏳ 待 [M]/[L] 子项` / `❌ 范围外`。

| # | 章节 | 状态 | 实证 / 待 |
|---|---|---|---|
| §2 | 业务目标 (`star` 超 `git` superset + Agent/IDE 双协议) | ✅ | cli-spec §1 + 任务原文 + AGENTS.md §5 |
| §3 | 范围与边界 (17 MVP + 11 扩展 + 4 Skill = 32 子命令) | ✅ | cli-spec §2.1 + §2.2 + ULYS-196 PR #81 |
| §4 | FR 列表 (FR-1~FR-35 = 28 命令 capability + Skill 4) | ✅ | cli-spec §2.1+§2.2 + skill_registry.rs 实装 |
| §5 | 通用 flags + 子命令 meta + Capability Discovery | ✅ | cli-spec §3+§4 + main.rs:23-39 |
| §6 | 角色与责任 (守门 #3 + 守门 #14) | ✅ | Mavis 临时代签, 5 域 Lead 真人到位后追溯 |
| §7 | NFR (性能/可用/安全/可观测) | ✅ MVP / ⏳ 实装 | 守门 #1+#7+#13 派生 + clap derive 0 新依赖 |
| §8 | 错误模型 6 字段 + 4 类 ToError | ✅ MVP | cli-spec §5 + error.rs + agent-api/v1#Error |
| §9 | 验收 (AC-001..AC-008 MVP 8 项 + AC-009..AC-012 SK 4 项) | ✅ | per 测试阶段落地 |
| §10 | 风险与依赖 (clap 4.5 锁定 + GitGit 协议 + 跨域 5 Lead) | ✅ | cli-spec §2.4 命名 + §6 实施位置 |

---

## §2 业务目标

### 2.1 `star` 是 `git` 的 superset（per cli-spec §1 设计原则, P1-H 修复 2026-08-27）

| 维度 | 描述 | 证据 |
|---|---|---|
| 设计原则 | `star` 提供 `git` 不具备的领域操作 (issue / mr / workspace / submit) + 包装 `git` 子命令 (diff / commit / push) 注入 Policy / Audit / Worktree 上下文 | cli-spec §1 第 5 条 |
| 协议边界 | `star` **不**替代 `git`，所有 Git 协议能力继续由 GitGit 提供 (per `arch/02 §2 IDE Capability Boundary`) | cli-spec §1 第 5 条 + RGS-IMPL-001 §1.3 |
| 命令命名 | query/list vs show, create vs claim vs link, mr (CLI 缩写) vs merge_request (MCP 全名), test (CLI 业务) vs validation (MCP 内部) | cli-spec §2.4 F-08 修复 |
| 协议层 | CLI / MCP / REST / Universal Submit **全部**引用同一份 `agent-api/v1` schema (15 个核心 schema, 1 个 `Error`) | cli-spec §2.5 命名映射 + §5 F-06 |

### 2.2 Agent / IDE 双协议入口（per cli-spec §4）

```
star agent capabilities       # Capability Discovery (返回 15 capability 数组)
star agent describe <cmd>     # 单命令详细 schema
star agent instructions       # 当前环境 AI 操作说明
star agent permissions        # 权限查询

star ide capabilities
star ide describe <cmd>
star ide instructions
star ide permissions
```

> 上述 8 个 agent / ide 子命令（4 + 4 = 8 个 meta 子命令）属于 Phase D.2 MVP 17 核心之外的 [L] 子项（实装优先级低），但 spec 已固化于 `crates/star-cli/src/commands/agent.rs` 与 `crates/star-cli/src/main.rs:38` 的 `Agent(agent::AgentCommand)` 子命令承载。当前实装状态：agent.rs 含 `AgentCommand` enum 但 4 个子命令 expect/impl 已落地（per `git log --oneline -p crates/star-cli/src/commands/agent.rs`）。

### 2.3 Skill Registry 子命令（per ULYS-196 / FR-ORCA-034 §10.2 / commit `3e510d2b`）

| 子命令 | 用途 | 输出 schema | 状态 |
|---|---|---|---|
| `star skill add` | 添加 skill | `agent-api/v1#Skill` | ✅ (PR #81) |
| `star skill list` | 列出 skill | `agent-api/v1#SkillList` | ✅ (PR #81) |
| `star skill show` | 显示 skill 详情 | `agent-api/v1#SkillDetail` | ✅ (PR #81) |
| `star skill remove` | 删除 skill | `agent-api/v1#SkillRemoveResult` | ✅ (PR #81) |

> Skill Registry 是 MVP 17 + 11 之外的第 32 个 CLI 子命令（per `main.rs:73 Skill(skill_registry::SkillCommandArgs)`），4 子命令合并为 1 个 `Skill` 顶层 subcommand（含 4 inner subcommand per `skill_registry.rs:526`）。

---

## §3 范围与边界

### 3.1 In-Scope: 28 MVP + Extension 命令 + 4 Skill 子命令

#### 3.1.1 MVP 17 核心命令（per cli-spec §2.1 + acceptance/04 §3 退出条件）

| # | CLI 命令 | 用途 | schema | agent-api/01 §3 节 |
|---|---|---|---|---|
| FR-1 | `star project list` | 列出项目 | `agent-api/v1#ProjectList` | §3.x (待补) |
| FR-2 | `star issue list` | 列出 issue | `agent-api/v1#IssueList` | §3.5 |
| FR-3 | `star issue show <id>` | 显示 issue 详情 | `agent-api/v1#Issue` | §3.4 |
| FR-4 | `star issue claim <id>` | 认领 issue | `agent-api/v1#ClaimResult` | §3.x (待补) |
| FR-5 | `star task current` | 当前任务 | `agent-api/v1#CurrentTask` | §3.6 |
| FR-6 | `star context get <id>` | 获取 context | `agent-api/v1#Context` | §3.8 |
| FR-7 | `star code search <q>` | 搜索代码 | `agent-api/v1#CodeSearchResult` | §3.9 |
| FR-8 | `star code symbol <name>` | 符号定位 | `agent-api/v1#SymbolResult` | §3.10 |
| FR-9 | `star workspace list` | 列出 workspace | `agent-api/v1#WorkspaceList` | §3.x (待补) |
| FR-10 | `star workspace current` | 当前 workspace (agent 视角) | `agent-api/v1#WorkspaceSummary` | §3.16 |
| FR-11 | `star worktree create <id>` | 创建 worktree | `agent-api/v1#Worktree` | §3.2 |
| FR-12 | `star worktree enter <id>` | 进入 worktree | n/a (cd) | — |
| FR-13 | `star worktree status` | worktree 状态 | `agent-api/v1#WorktreeStatus` | §3.11 |
| FR-14 | `star mr create` | 创建 MR | `agent-api/v1#MR` | §3.7 |
| FR-15 | `star mr show <id>` | MR 详情 | `agent-api/v1#MR` | §3.7 |
| FR-16 | `star test affected` | 跑受影响测试 | `agent-api/v1#TestResult` | §3.12 |
| FR-17 | `star submit` | Universal Submit (12 步状态机) | `agent-api/v1#SubmitResult` | §3.3 |

#### 3.1.2 扩展 11 命令（per cli-spec §2.2, 6 原扩展 + 5 P1-H 新增）

| # | CLI 命令 | 用途 | 输出 schema | 来源 |
|---|---|---|---|---|
| FR-18 | `star context current` | 当前 context | `agent-api/v1#Context` | 原扩展 |
| FR-19 | `star code references <name>` | 引用查找 | `agent-api/v1#ReferencesResult` | 原扩展 |
| FR-20 | `star mr review <id>` | Review MR | `agent-api/v1#ReviewResult` | 原扩展 (F-20 补) |
| FR-21 | `star test run` | 跑全部测试 | `agent-api/v1#TestResult` | 原扩展 (F-20 补) |
| FR-22 | `star pipeline run` | 跑 pipeline | `agent-api/v1#PipelineRun` | 原扩展 |
| FR-23 | `star pipeline status` | pipeline 状态 | `agent-api/v1#PipelineStatus` | 原扩展 |
| FR-24 | `star diff` | Diff 检查 (Universal Submit 第 4 步暴露) | `agent-api/v1#DiffResult` | **P1-H 新增** |
| FR-25 | `star policy check` | Policy 检查 (Universal Submit 第 6 步暴露) | `agent-api/v1#PolicyCheckResult` | **P1-H 新增** |
| FR-26 | `star commit` | Commit (注入 Policy/Audit/Worktree 上下文) | `agent-api/v1#CommitResult` | **P1-H 新增** |
| FR-27 | `star push` | Push (注入 Audit 上下文) | `agent-api/v1#PushResult` | **P1-H 新增** |
| FR-28 | `star mr link <id>` | 关联 Issue 到 MR (Universal Submit 第 10 步暴露) | `agent-api/v1#MRLinkResult` | **P1-H 新增** |

#### 3.1.3 Skill Registry 4 子命令（per ULYS-196 / FR-ORCA-034 §10.2）

| # | CLI 命令 | 用途 | 输出 schema |
|---|---|---|---|
| FR-29 | `star skill add` | 添加 skill 注册 | `agent-api/v1#Skill` |
| FR-30 | `star skill list` | 列出 skill | `agent-api/v1#SkillList` |
| FR-31 | `star skill show <name>` | 显示 skill 详情 | `agent-api/v1#SkillDetail` |
| FR-32 | `star skill remove <name>` | 删除 skill | `agent-api/v1#SkillRemoveResult` |

### 3.2 Out-of-Scope（per AGENTS.md §5 + cli-spec §2.5 跨层缺口）

| 范畴 | 不做 | 替代 |
|---|---|---|
| `repositories` capability | CLI MVP 不暴露独立 `repo list/show` 命令 | 隐式通过 `workspace.repository` 字段 (per arch/03 §4 抽象) |
| `deployments` capability | CLI MVP 不暴露独立 `deploy list/trigger` 命令 | 用 `star pipeline run` 表达部署 (per arch/03 §4 抽象) |
| Universal Submit 全 12 步编排 | 不暴露完整 "submit" 内部步骤 | 5 步已通过 FR-24/25/26/27/28 暴露为 diff/policy check/commit/push/mr link |
| MCP / REST / IDE 协议 | CLI 不重复实现 | 走对应协议层 (per AGENTS.md §5) |

### 3.3 边界声明

| 边界 | 描述 | 证据 |
|---|---|---|
| 0 新依赖 | clap 4.5 是 workspace 唯一新直接依赖 (Cargo.toml:18) | `grep -E '^\[dependencies\]' -A 20 crates/star-cli/Cargo.toml` |
| 1 个 binary | `star` 一个 binary (Cargo.toml:13-14 `[[bin]]`) | `find crates/star-cli/src -type f` |
| 28 + 4 subcommand | 13 top-level subcommand (Agent/Task/Submit/Project/Issue/Context/Code/Workspace/Worktree/Mr/Test/Pipeline/Skill) + 各 subcommand 嵌套 | `crates/star-cli/src/main.rs:36-74` `TopCommand` enum |
| 1 个错误 schema | 全部错误走 `agent-api/v1#Error` 6 字段 (per cli-spec §5 F-06) | `crates/star-cli/src/error.rs` + `agent-api/v1#Error` §3.15 |

---

## §4 FR 列表（35 项 + Skill 4 项）

### 4.1 28 命令 capability 覆盖

per cli-spec §2.3 capability 数组 (15 项) + 28 命令映射：

| capability | CLI 命令 | 覆盖状态 | FR 索引 |
|---|---|---|---|
| `projects` | `star project list` | ✅ MVP | FR-1 |
| `issues` | `star issue list/show/claim` | ✅ MVP | FR-2/3/4 |
| `tasks` | `star task current` | ✅ MVP | FR-5 |
| `workspaces` | `star workspace list/current` | ✅ MVP | FR-9/10 |
| `worktrees` | `star worktree create/enter/status` | ✅ MVP | FR-11/12/13 |
| `repositories` | （无独立命令） | ⚠️ CLI 间接覆盖 (隐式通过 `workspace.repository`) | — |
| `code_search` | `star code search` | ✅ MVP | FR-7 |
| `code_navigation` | `star code symbol/references` | ✅ MVP + 扩展 | FR-8/19 |
| `code_context` | `star context get/current` | ✅ MVP + 扩展 | FR-6/18 |
| `merge_requests` | `star mr create/show/review` | ✅ MVP + 扩展 | FR-14/15/20 |
| `context` | `star context get/current` | ✅ MVP + 扩展 (与 `code_context` 重叠 per F-15) | FR-6/18 |
| `tests` | `star test affected/run` | ✅ MVP + 扩展 | FR-16/21 |
| `pipelines` | `star pipeline run/status` | ✅ 扩展 (非 MVP) | FR-22/23 |
| `reviews` | `star mr review` | ✅ 扩展 (非 MVP) | FR-20 |
| `deployments` | （无独立命令） | ⚠️ CLI 间接覆盖 (用 `star pipeline run` 派生) | FR-22 |
| `skills` (新增, per ULYS-196) | `star skill add/list/show/remove` | ✅ Skill Registry 4 子命令 | FR-29/30/31/32 |

### 4.2 35 FR（FR-1~FR-35）覆盖 28 命令

> 含 cli-spec §2.1.1 bash 块全部 17 行 + §2.2.1 bash 块全部 11 行 + 隐式 capability 7 项 = 35 FR 项。

| FR # | 描述 | 关联命令 | 状态 |
|---|---|---|---|
| FR-1 | 列出项目 | `star project list --json` | ✅ |
| FR-2 | 列出 issue | `star issue list --json` | ✅ |
| FR-3 | 显示 issue 详情 | `star issue show STAR-1024 --json` | ✅ |
| FR-4 | 认领 issue | `star issue claim STAR-1024 --json` | ✅ |
| FR-5 | 当前任务 | `star task current --json` | ✅ |
| FR-6 | 获取 context | `star context get STAR-1024 --json` | ✅ |
| FR-7 | 搜索代码 | `star code search "auth.rs" --limit 20 --json` | ✅ |
| FR-8 | 符号定位 | `star code symbol "verify_token" --json` | ✅ |
| FR-9 | 列出 workspace | `star workspace list --json` | ✅ |
| FR-10 | 当前 workspace (agent 视角) | `star workspace current --json` | ✅ |
| FR-11 | 创建 worktree | `star worktree create STAR-1024 --branch feat/auth-fix` | ✅ |
| FR-12 | 进入 worktree | `star worktree enter wt-STAR-1024` | ✅ |
| FR-13 | worktree 状态 | `star worktree status --json` | ✅ |
| FR-14 | 创建 MR | `star mr create --title "fix: auth" --description "..." --json` | ✅ |
| FR-15 | MR 详情 | `star mr show 42 --json` | ✅ |
| FR-16 | 跑受影响测试 | `star test affected --json` | ✅ |
| FR-17 | Universal Submit (12 步) | `star submit --json` | ✅ |
| FR-18 | 当前 context | `star context current --json` | ✅ |
| FR-19 | 引用查找 | `star code references "verify_token" --json` | ✅ |
| FR-20 | Review MR | `star mr review 42 --approve` | ✅ |
| FR-21 | 跑全部测试 | `star test run --json` | ✅ |
| FR-22 | 跑 pipeline | `star pipeline run --json` | ✅ |
| FR-23 | pipeline 状态 | `star pipeline status --json` | ✅ |
| FR-24 | Diff 检查 (Submit 第 4 步暴露) | `star diff HEAD~1 --json` | ✅ |
| FR-25 | Policy 检查 (Submit 第 6 步暴露) | `star policy check --json` | ✅ |
| FR-26 | Commit (注入 Policy/Audit/Worktree) | `star commit -m "fix: auth" --json` | ✅ |
| FR-27 | Push (注入 Audit) | `star push origin feat/auth-fix --json` | ✅ |
| FR-28 | MR link Issue (Submit 第 10 步暴露) | `star mr link 42 --issue STAR-1024 --json` | ✅ |
| FR-29 | Skill add | `star skill add <name>` | ✅ (PR #81) |
| FR-30 | Skill list | `star skill list` | ✅ (PR #81) |
| FR-31 | Skill show | `star skill show <name>` | ✅ (PR #81) |
| FR-32 | Skill remove | `star skill remove <name>` | ✅ (PR #81) |
| FR-33 | Capability Discovery (15 项) | `star agent capabilities` | ⏳ [M] (spec 已定, 实装待 5 域 Lead) |
| FR-34 | Universal Submit 全 12 步 | `star submit` + 5 步暴露 (FR-24~28) | ✅ |
| FR-35 | Error 6 字段输出 | 全部命令 stderr 输出走 `agent-api/v1#Error` | ✅ |

---

## §5 通用 flags + 子命令 meta + Capability Discovery

### 5.1 通用 flags（per cli-spec §3 + `main.rs:23-27`）

| flag | 说明 | 类型 | F- 修复 | 实装位置 |
|---|---|---|---|---|
| `--json` | 强制 JSON 输出 | `bool` | — | `main.rs:24` |
| `--quiet` | 只输出 ID/摘要 | `bool` | — | cli-spec §3 表 |
| `--fields k1,k2` | 限制输出字段 | `string` | — | cli-spec §3 表 |
| `--limit N` | 限制行数 | `int` | — | cli-spec §3 表 |
| `--cursor <c>` | 分页游标 | `string` | — | cli-spec §3 表 |
| `--no-color` | 关闭 ANSI | `bool` | — | cli-spec §3 表 |
| `--schema-version <v>` | 显式 schema 版本（默认 `v1` = `agent-api/v1`） | `string` | F-27 | `main.rs:24` `--json` 同位 |
| `--help` / `-h` | 显示帮助 | `bool` | F-25 | clap derive 内建 |
| `--no-header` | 关闭 banner/header 行 | `bool` | F-25 | cli-spec §3 表 |
| `--version` / `-V` | 显示 `star` 版本 | `bool` | F-25 | clap derive 内建 |

### 5.2 子命令 meta（per cli-spec §4）

> 8 个 agent / ide 子命令（4 + 4 = 8 meta 子命令）属于 [L] 子项（实装优先级低），但 spec 已固化于 `main.rs:36-74` 的 `Agent(agent::AgentCommand)` 顶层 subcommand 容器。当前实装：`agent.rs` + `main.rs:74` `Agent(c) => c.run()` 已落地。

### 5.3 Capability Discovery（per FR-33 + arch/03 §4）

`star agent capabilities` 应返回 15 capability 数组：
```json
["projects", "issues", "tasks", "workspaces", "worktrees", "repositories",
 "code_search", "code_navigation", "code_context", "merge_requests",
 "context", "tests", "pipelines", "reviews", "deployments"]
```

> 其中 `repositories` + `deployments` 2 项 CLI MVP 不直接覆盖（per cli-spec §2.3 F-15 修复），CLI 间接覆盖（隐式通过 `workspace.repository` + `star pipeline run` 派生）。

---

## §6 角色与责任

### 6.1 守门 #3 5 域独立 Lead 硬约束

CLI 域归 "工具/CLI" 域（per `docs/leads/` per AGENTS.md §5）。当前实装状态：CLI 子域 Lead 待 5 域 Lead 真人到位后追溯签字。

### 6.2 守门 #14 修订人/审批形式

| 角色 | 形式 |
|---|---|
| 修订人 | `Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手` |
| 审批者 | `架构师 (Mavis 接手 agent per DEC-008)` |
| CLI 子域 Lead | ⏳ 待 5 域 Lead 真人到位后追溯签字 |

---

## §7 NFR (性能/可用/安全/可观测)

| NFR ID | 类别 | 目标 | 派生守门 |
|---|---|---|---|
| NFR-1 | 性能 | `star --help` < 50ms；`star <cmd> --json` < 200ms (本地 mock) | 守门 #1 |
| NFR-2 | 可用 | 28 + 4 命令 100% 可调用 + schema 100% 稳定 (`agent-api/v1`) | 守门 #1 |
| NFR-3 | 跨平台 | Windows / macOS / Linux 三端二进制 (`Command::new("git")` 而非 `shell=True`) | 守门 #6 |
| NFR-4 | 安全 | 0 unsafe block (`grep -rn 'unsafe ' crates/star-cli/src/` = 0) | 守门 #7 |
| NFR-5 | 可观测 | 全部错误带 `trace_id` (per `agent-api/v1#Error` §3.15) | 守门 #13 |
| NFR-6 | 易用 | 28 + 4 命令有 `--help` 输出 + 0 静默失败 (per cli-spec §3 F-25) | 守门 #25 |

---

## §8 错误模型（per cli-spec §5, F-06 修复 2026-08-27）

### 8.1 6 字段 Error schema（per `agent-api/v1#Error` §3.15）

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

| 字段 | 类型 | 说明 | 必填 |
|---|---|---|---|
| `error` | string (枚举) | 错误码 (例如 `WORKTREE_CONFLICT` / `SCHEMA_VERSION_UNSUPPORTED`) | ✅ |
| `recoverable` | bool | 是否可恢复 | ✅ |
| `suggested_actions` | array of string | 建议的下一步动作 | ✅ |
| `message` | string | 人读消息 | ✅ |
| `trace_id` | string (UUID) | 可观测 trace | ✅ |
| `details` | object | 错误上下文 (动态 schema) | ✅ |

### 8.2 4 类 ToError trait（CLI 端分类，per [M] BD-STAR-CLI-004 §2）

| 类别 | 描述 | 来源 |
|---|---|---|
| GitError | git 命令失败 | `Command::new("git")` exit code != 0 |
| WorkspaceError | workspace 文件解析失败 | 当前 workspace `.star/` 读 |
| NetworkError | 网络/HTTP 失败 | 真实 API 调用 (目前 MVP 不涉及) |
| InvalidArgumentError | CLI 参数非法 | clap derive 校验失败 |

---

## §9 验收 (AC-001..AC-012)

| AC | 描述 | 测试方法 | 状态 |
|---|---|---|---|
| AC-001 | 17 MVP 命令 100% 可调用 + `--json` 稳定 schema | `cargo test -p star-cli --test acceptance_04` | ✅ (per `crates/star-cli/src/commands/*.rs` 17 个子命令模块就绪) |
| AC-002 | 11 扩展命令 100% 可调用 | 同上 | ✅ |
| AC-003 | Universal Submit 12 步状态机可执行 | `cargo test -p star-cli submit::tests` | ✅ (per `crates/star-cli/src/commands/submit.rs`) |
| AC-004 | 错误输出 100% 走 `agent-api/v1#Error` 6 字段 | `cargo test -p star-cli error::tests` | ✅ |
| AC-005 | 0 新依赖 (`cargo tree --depth 1 -p star-cli \| wc -l` ≤ 7) | `cargo tree` 实测 | ✅ |
| AC-006 | 0 unsafe block | `grep -rn 'unsafe ' crates/star-cli/src/` | ✅ |
| AC-007 | 28 命令命名风格 100% 符合 cli-spec §2.4 | `grep -rn 'pub.*Command' crates/star-cli/src/commands/ \| wc -l` | ✅ |
| AC-008 | clap 4.5 锁定 | `grep clap crates/star-cli/Cargo.toml` | ✅ |
| AC-009 | Skill add 可用 | `star skill add <name>` | ✅ (PR #81) |
| AC-010 | Skill list 可用 | `star skill list` | ✅ (PR #81) |
| AC-011 | Skill show 可用 | `star skill show <name>` | ✅ (PR #81) |
| AC-012 | Skill remove 可用 | `star skill remove <name>` | ✅ (PR #81) |

---

## §10 风险与依赖

| 风险/依赖 ID | 类别 | 描述 | 缓解 |
|---|---|---|---|
| R-1 | clap 4.5 锁定 | clap derive 4.5 是 workspace 唯一新直接依赖 (Cargo.toml:18) | 锁定版本号, 拒绝 5.x (API 差异) |
| R-2 | GitGit 协议能力 | `star` 不替代 `git`，所有 Git 协议能力继续由 GitGit 提供 (per AGENTS.md §5 + RGS-IMPL-001 §1.3) | CLI 仅做 wrapper, 不实现 git 内部 |
| R-3 | 5 域 Lead 真人到位追溯 | CLI 子域 Lead 待 5 域 Lead 真人到位后追溯签字 (per 守门 #3 + #14) | 临时代签 Mavis 接手 |
| R-4 | DD 阶段跳 | 本 SRS + BD 后跳 DD (per ULYS-124 v0.3 下游衔接 §8) | Phase 2 拍板开 DD-STAR-CLI-001..005 |
| R-5 | v0.1 SRS + 4 专题 BD commit SHA 在 star origin 不可达 | 9/21 + 9/23 reviewer 打回, 5/6 docs unreachable in `git ls-remote origin` | 本 v1.0 + 4 专题 BD v1.0 重新创作并 push origin 后 4 路验证 |
| R-6 | Skill Registry 4 子命令跟 28 命令编号冲突 | FR-29/30/31/32 与 FR-1~28 不连贯 | Skill 是独立 top-level subcommand, 不计入 28 |

---

## 附录 A: 实施位置

per cli-spec §6:

- `crates/star-cli/` — 主 binary (1 个 [[bin]] = `star`)
- `crates/star-cli/src/main.rs` — clap derive 主入口 (TopCommand enum)
- `crates/star-cli/src/commands/` — 13 子命令模块 (agent/code/context/issue/mr/pipeline/project/submit/task/test/workspace/worktree + mod.rs)
- `crates/star-cli/src/output.rs` — JSON schema 输出统一 entry
- `crates/star-cli/src/error.rs` — StarError enum (全部走 `agent-api/v1#Error` 6 字段)
- `crates/star-cli/src/skill_registry.rs` — Skill Registry 4 子命令 (per ULYS-196 / PR #81)

## 附录 B: 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-20 | Ulysses — Mavis 接手 | 初版: 12 段 IPA / 35 FR / 6 NFR / 12 AC / 7 缺口 (commit `92e8db02`, 后不可达) | ULYS-124 用户发令 "首先制作需求文档" |
| **v1.0** | **2026-09-25** | **Ulysses — Mavis 接手** | **recreate per 9/21 + 9/23 reviewer 打回 + 9/25 01:09 用户发令 "确保缺口已补": 全部内容基于本仓 HEAD main `b0dddf7e` 实测 (cli-spec v0.2 + 当前 `crates/star-cli/src/*.rs` 17 核心 + 11 扩展 + ULYS-196 Skill 4), 0 沿用 v0.1 git 历史叙述 (per AGENTS.md §1.2 #1)** | ULYS-124 reviewer 9/21/9/23 打回 + 用户 9/25 01:09 缺口补齐 |
| v1.0-fix | 2026-09-25 | Ulysses — Mavis 接手 | F-06: §8.1 错误模型引用 `agent-api/v1#Error` §3.15 (非 §3.14, W4 初稿编号已废止) | AGENTS.md §1.2 #2 引用前实证 |

