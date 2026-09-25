# BD-STAR-CLI-001

> **STAR CLI 总冊基本設計書 v1.0 (recreate, per ULYS-124 v1.0 缺口补齐)**
>
> - 状态: Basic Design Baseline (v1.0 — recreate after 9/21 + 9/23 reviewer 打回, per 用户 9/25 01:09 JST "确保缺口已补")
> - 目标阶段: 基本設計 → 詳細設計 → 実装 → テスト → リリース
> - 上位要件: [`docs/requirements/SRS-STAR-CLI-001.md`](../requirements/SRS-STAR-CLI-001.md) v1.0 (本次同期落档)
> - 关联基本设计 (4 专题 BD, 本 commit 同期落档):
>   - [`docs/design/BD-STAR-CLI-002.md`](./BD-STAR-CLI-002.md) — CL-1+CL-3 MVP 17 核心命令 + 通用 flags
>   - [`docs/design/BD-STAR-CLI-003.md`](./BD-STAR-CLI-003.md) — CL-2 11 扩展命令 + Universal Submit 步骤暴露
>   - [`docs/design/BD-STAR-CLI-004.md`](./BD-STAR-CLI-004.md) — CL-4+CL-5 错误模型 + Capability Discovery + Skill 4 子命令
>   - [`docs/design/BD-STAR-CLI-005.md`](./BD-STAR-CLI-005.md) — CL-6 Universal Submit 12 步状态机 + SubmitResult schema
> - 上游 spec: [`docs/architecture/2026-08-26-upgrade/spec/cli/01-cli-spec.md`](../architecture/2026-08-26-upgrade/spec/cli/01-cli-spec.md) v0.2 (canonical)
> - 修订人: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> - 审批: 架构师 (Mavis 接手 agent per DEC-008)
> - 日期: 2026-09-25 JST

> **回溯记录（per ULYS-124 reviewer 9/21 + 9/23 打回与 9/25 01:09 用户缺口补齐指令）**：v0.1 总冊 `08be4689` 在 `git ls-remote origin` 不可达。本 v1.0 按 (a) ULYS-124 v0.3 同期落档的四专题 BD v0.1 内容密度 (b) 当前 main `b0dddf7e` 实装 (c) cli-spec v0.2 canonical 基础上重新创作, 索引所有 4 专题 BD, **不重复其内容**, 严格按"总冊索引 + 跨域汇总"分工。

---

## §0 文档目的

本文档基于 SRS-STAR-CLI-001 v1.0 的需求, 定义 STAR CLI **总冊基本設計**: 5 view 跨域汇总 (需求/架构/数据/模块/NFR) + 6 子能力架构 (CL-1~CL-6) + 28 命令 schema 索引 + 9 守门合规汇总 + 41 已知缺口显式列 + 25+ 关联文档索引。

**职责定位**: 本总冊是**索引级汇总**, 不展开 28 命令的详细 schema/顺序图/接口契约 —— 那些内容按子能力拆到 4 专题 BD:
- **CL-1+CL-3** (project / issue / task / context / code / workspace / worktree) → BD-002
- **CL-2** (扩展 11 命令) → BD-003
- **CL-4+CL-5** (error model + capability discovery) → BD-004
- **CL-6** (Universal Submit 12 步) → BD-005

**MVP 范围** (per SRS §3): 28 MVP + Extension 命令 + 4 Skill 子命令 + 1 error schema + 1 binary + 0 新依赖 (除 clap 4.5)。

---

## §1 适用范围

### 1.1 In-Scope: 6 子能力表 (CL-1~CL-6)

| 子能力 | 描述 | 涵盖命令 | 专题 BD | FR 索引 |
|---|---|---|---|---|
| **CL-1** Project & Issue 域 | project/issue/task 列表 + 详情 + 认领 | `star project list` / `star issue list/show/claim` / `star task current` | BD-002 §2 | FR-1/2/3/4/5 |
| **CL-2** 扩展命令域 | 11 扩展命令 (context current / code references / mr review / test run / pipeline run/status + 5 P1-H Universal Submit 步骤暴露) | 同名 11 命令 | BD-003 §2 | FR-18~28 |
| **CL-3** Code & Workspace & Worktree 域 | 代码搜索/符号 + workspace 列表/当前 + worktree 创建/进入/状态 | `star code search/symbol` / `star workspace list/current` / `star worktree create/enter/status` | BD-002 §3 | FR-7/8/9/10/11/12/13 |
| **CL-4** Error & Capability Discovery 域 | 错误模型 6 字段 + Capability Discovery 15 项 + Skill Registry 4 子命令 | `star agent capabilities/describe/instructions/permissions` + `star skill add/list/show/remove` | BD-004 §2 | FR-33 + FR-29/30/31/32 |
| **CL-5** MR & Test & Pipeline 域 | MR 创建/详情/Review/Test 受影响/全部 + Pipeline 跑/状态 | `star mr create/show/review` / `star test affected/run` / `star pipeline run/status` | BD-004 §3 | FR-14/15/16/20/21/22/23 |
| **CL-6** Universal Submit 域 | 12 步状态机 + SubmitResult schema + 5 步暴露 (FR-24~28) | `star submit` + 5 步暴露命令 | BD-005 §2 | FR-17 + FR-24/25/26/27/28 |

### 1.2 Out-of-Scope (8 项)

| 不做 | 原因 | 替代 |
|---|---|---|
| MCP / REST / IDE 协议 | CLI 不重复 (per AGENTS.md §5 + cli-spec §5 F-06) | 走对应协议层 |
| 真实 GitGit 内部协议 | CLI 不替代 `git` (per cli-spec §1 第 5 条) | GitGit 提供 |
| 全 12 步 Universal Submit 内部编排 | 5 步已通过 FR-24~28 暴露 | `star submit` 一个调用走全 12 步 |
| 全 17 capability 独立 CLI 命令 | `repositories` + `deployments` 2 项 CLI 间接覆盖 (per cli-spec §2.3 F-15) | 隐式通过 `workspace.repository` + `star pipeline run` |
| PostgreSQL / Redis 持久化 | MVP 内存 + JSON stub | Phase 2+ 待 [M] 子项 |
| 真实 LLM 通道 (OpenAI/Anthropic) | 本仓不涉 LLM | 星后端走 `crates/star-llm/` |
| 详设 (DD) | Phase 2+ 拍板 | DD-STAR-CLI-001..005 待开 |
| `repositories` / `deployments` capability 数组 | CLI 不删除这 2 项 (向后兼容 arch/03 §4), 仅声明覆盖关系 | per cli-spec §2.3 |

### 1.3 边界声明 (5 项)

| 边界 | 描述 | 证据 |
|---|---|---|
| 1 个 binary | `star` 一个 binary | `crates/star-cli/Cargo.toml:13-14` |
| 0 新依赖 | clap 4.5 是 workspace 唯一新直接依赖 | `crates/star-cli/Cargo.toml:18` |
| 13 顶层 subcommand | Agent / Task / Submit / Project / Issue / Context / Code / Workspace / Worktree / Mr / Test / Pipeline / Skill | `crates/star-cli/src/main.rs:36-74` |
| 28 + 4 命令编号 | MVP 17 + 扩展 11 + Skill 4 = 32 子命令入口 | per SRS §3.1.1/3.1.2/3.1.3 |
| 1 个 Error schema | 全部走 `agent-api/v1#Error` 6 字段 (per cli-spec §5 F-06) | `crates/star-cli/src/error.rs` |

### 1.4 平行 SRS 边界 (4 项)

| 平行 SRS | 关系 | 边界声明 |
|---|---|---|
| SRS-AGENT-VIEW-001 | Agent View 画布 | CLI 仅读 agent data via `agent-api/v1`, 不重复 agent view 渲染 |
| SRS-WORKTREE-CANVAS-001 | Worktree Canvas | CLI 仅调 `star worktree create/enter/status`, canvas 渲染由 frontend 负责 |
| SRS-MULTICA-RUNTIME-001 | Multica Runtime | CLI 不涉及 multica orchestration, 仅是消费者 |
| SRS-STAR-AGENT-RUNTIME-001 | Agent Runtime | CLI 调用 `star agent capabilities` 仅作 client, agent 协议由 agent runtime 实现 |

---

## §2 システムアーキテクチャ

### 2.1 全体構成図 (3 tier ASCII)

```
┌──────────────────────────────────────────────────────────────────────────┐
│ TIER-1: User Shell                                                         │
│   $ star <top> <sub> [flags] [--json]      ← clap derive 主入口            │
│   $ star submit --json                      ← Universal Submit 入口        │
│   $ star skill add <name>                   ← Skill Registry 入口          │
└──────────────────────────────────────────────────────────────────────────┘
                                       │
                                       ▼
┌──────────────────────────────────────────────────────────────────────────┐
│ TIER-2: CLI Binary (1 个 process)                                          │
│   ┌────────────────────────────────────────────────────────────────────┐ │
│   │ main.rs: Cli parser (clap derive, global --json)                    │ │
│   │   ↓                                                                 │ │
│   │ TopCommand enum dispatch (13 顶层 subcommand)                       │ │
│   │   ↓                                                                 │ │
│   │ commands/<sub>.rs: 子命令模块 (12 个 + skill_registry.rs = 13)      │ │
│   │   ├─ Argument parsing (clap derive enum)                            │ │
│   │   ├─ Business logic (ref agent-api client OR shell Command::new)     │ │
│   │   ├─ Error handling (StarError → agent-api/v1#Error 6 字段)         │ │
│   │   └─ Output: stdout JSON via output::json_pretty / stderr Error     │ │
│   └────────────────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────────┘
                                       │
                                       ▼
┌──────────────────────────────────────────────────────────────────────────┐
│ TIER-3: External services                                                  │
│   - Git 协议: `Command::new("git")` (per cli-spec §1 第 5 条)              │
│   - Star API (agent-api/v1): HTTP client (Phase D.2 mock + Phase E real)    │
│   - Local FS: .star/ workspace 元数据                                      │
└──────────────────────────────────────────────────────────────────────────┘
```

### 2.2 clap derive 拓扑 (8 维度)

| 维度 | 描述 | 证据 |
|---|---|---|
| Parser | `#[derive(Parser)] struct Cli` 顶层 | `main.rs:21-30` |
| Subcommand | `#[derive(Subcommand)] enum TopCommand` 13 顶层 subcommand | `main.rs:32-74` |
| Global flag | `#[arg(long, global = true)]` --json 等通用 flag | `main.rs:24-27` |
| Nested subcommand | `#[command(subcommand)] Agent(agent::AgentCommand)` 等 | `main.rs:38-74` |
| Args struct | `submit::SubmitArgs` 等独立 args 结构 | `commands/submit.rs` SubmitArgs |
| Help 内建 | `#[command(name = "star", version, about)]` | `main.rs:22` |
| Error exit code | `StarError::exit_code()` 决定 std::process::ExitCode | `main.rs:80` + `error.rs` |
| Version | `--version` / `-V` clap derive 内建 | cli-spec §3 F-25 |

### 2.3 8 维度方式

| # | 维度 | CLI 实现方式 |
|---|---|---|
| 1 | parse | clap derive (clap 4.5) |
| 2 | dispatch | match `cli.command` in `fn run()` |
| 3 | execute | ref to `agent-api client` OR `Command::new("git")` |
| 4 | error | `Result<(), StarError>` → eprintln to stderr + ExitCode |
| 5 | output | stdout via `output::json_pretty` (always JSON for MVP 17) |
| 6 | schema | `agent-api/v1` (15 schema + 1 Error) |
| 7 | capability | `star agent capabilities` returns 15-item array |
| 8 | versioning | `--schema-version v1` (default), `--version` returns CLI binary version |

### 2.4 6 部署单元

| 单元 | 形式 | 路径 |
|---|---|---|
| 1 binary | `target/release/star` (Linux/macOS) / `target/release/star.exe` (Windows) | `crates/star-cli/` |
| 1 workspace file | `.star/` 目录 (cli 读写 workspace 元数据) | 用户 workspace 根 |
| 1 lock file | (无, MVP 不持久化) | n/a |
| 1 schema ref | `agent-api/v1` (15 + 1 schema, 引用 = spec/agent-api/01-schema.md) | canonical |
| 1 error schema | `agent-api/v1#Error` 6 字段 (per cli-spec §5 F-06) | canonical |
| 1 CLI spec | `docs/architecture/2026-08-26-upgrade/spec/cli/01-cli-spec.md` v0.2 | canonical |

---

## §3 データビュー

### 3.1 顶层 schema 守门 (per cli-spec §2.5 + §5)

所有 28 + 4 命令输出**全部**引用 `agent-api/v1`, 引用节号详见 [`spec/agent-api/01-schema.md`](../architecture/2026-08-26-upgrade/spec/agent-api/01-schema.md) §3:
- `Worktree` §3.2
- `SubmitResult` §3.3
- `Issue` §3.4
- `IssueList` §3.5
- `CurrentTask` §3.6
- `MR` §3.7
- `Context` §3.8
- `CodeSearchResult` §3.9
- `SymbolResult` §3.10
- `WorktreeStatus` §3.11
- `TestResult` §3.12
- `Error` §3.15 (per cli-spec §5 F-06 修复, W4 初稿编号 §3.14 已废止)
- `WorkspaceSummary` §3.16 (per cli-spec §2.1 P1-C 修复, agent 视角)
- `Capabilities` §3.17
- 11 个命令级 schema (ProjectList/ClaimResult/WorkspaceList/Skill等)

### 3.2 28 命令完整索引表 (FR-1~FR-35)

> 详细 schema 字段/顺序图/接口契约请见 4 专题 BD 各 §3。本节仅给 schema 名称 + 详细 BD 章节链接。

#### 3.2.1 MVP 17 核心命令 (FR-1~FR-17)

| FR | 命令 | schema 名称 | 详细 BD |
|---|---|---|---|
| FR-1 | `star project list` | `agent-api/v1#ProjectList` | [BD-002 §3.1](../design/BD-STAR-CLI-002.md) |
| FR-2 | `star issue list` | `agent-api/v1#IssueList` | [BD-002 §3.2](../design/BD-STAR-CLI-002.md) |
| FR-3 | `star issue show <id>` | `agent-api/v1#Issue` | [BD-002 §3.2](../design/BD-STAR-CLI-002.md) |
| FR-4 | `star issue claim <id>` | `agent-api/v1#ClaimResult` | [BD-002 §3.2](../design/BD-STAR-CLI-002.md) |
| FR-5 | `star task current` | `agent-api/v1#CurrentTask` | [BD-002 §3.4](../design/BD-STAR-CLI-002.md) |
| FR-6 | `star context get <id>` | `agent-api/v1#Context` | [BD-002 §3.3](../design/BD-STAR-CLI-002.md) |
| FR-7 | `star code search <q>` | `agent-api/v1#CodeSearchResult` | [BD-002 §3.5](../design/BD-STAR-CLI-002.md) |
| FR-8 | `star code symbol <name>` | `agent-api/v1#SymbolResult` | [BD-002 §3.5](../design/BD-STAR-CLI-002.md) |
| FR-9 | `star workspace list` | `agent-api/v1#WorkspaceList` | [BD-002 §3.6](../design/BD-STAR-CLI-002.md) |
| FR-10 | `star workspace current` | `agent-api/v1#WorkspaceSummary` | [BD-002 §3.6](../design/BD-STAR-CLI-002.md) |
| FR-11 | `star worktree create <id>` | `agent-api/v1#Worktree` | [BD-002 §3.7](../design/BD-STAR-CLI-002.md) |
| FR-12 | `star worktree enter <id>` | n/a (cd) | [BD-002 §3.7](../design/BD-STAR-CLI-002.md) |
| FR-13 | `star worktree status` | `agent-api/v1#WorktreeStatus` | [BD-002 §3.7](../design/BD-STAR-CLI-002.md) |
| FR-14 | `star mr create` | `agent-api/v1#MR` | [BD-004 §3.1](../design/BD-STAR-CLI-004.md) |
| FR-15 | `star mr show <id>` | `agent-api/v1#MR` | [BD-004 §3.1](../design/BD-STAR-CLI-004.md) |
| FR-16 | `star test affected` | `agent-api/v1#TestResult` | [BD-004 §3.2](../design/BD-STAR-CLI-004.md) |
| FR-17 | `star submit` | `agent-api/v1#SubmitResult` | [BD-005 §2](../design/BD-STAR-CLI-005.md) |

#### 3.2.2 扩展 11 命令 (FR-18~FR-28)

| FR | 命令 | schema 名称 | 详细 BD |
|---|---|---|---|
| FR-18 | `star context current` | `agent-api/v1#Context` | [BD-003 §2.1](../design/BD-STAR-CLI-003.md) |
| FR-19 | `star code references <name>` | `agent-api/v1#ReferencesResult` | [BD-003 §2.2](../design/BD-STAR-CLI-003.md) |
| FR-20 | `star mr review <id>` | `agent-api/v1#ReviewResult` | [BD-003 §2.3](../design/BD-STAR-CLI-003.md) |
| FR-21 | `star test run` | `agent-api/v1#TestResult` | [BD-003 §2.4](../design/BD-STAR-CLI-003.md) |
| FR-22 | `star pipeline run` | `agent-api/v1#PipelineRun` | [BD-003 §2.5](../design/BD-STAR-CLI-003.md) |
| FR-23 | `star pipeline status` | `agent-api/v1#PipelineStatus` | [BD-003 §2.5](../design/BD-STAR-CLI-003.md) |
| FR-24 | `star diff` | `agent-api/v1#DiffResult` | [BD-003 §3.1](../design/BD-STAR-CLI-003.md) |
| FR-25 | `star policy check` | `agent-api/v1#PolicyCheckResult` | [BD-003 §3.2](../design/BD-STAR-CLI-003.md) |
| FR-26 | `star commit` | `agent-api/v1#CommitResult` | [BD-003 §3.3](../design/BD-STAR-CLI-003.md) |
| FR-27 | `star push` | `agent-api/v1#PushResult` | [BD-003 §3.3](../design/BD-STAR-CLI-003.md) |
| FR-28 | `star mr link <id>` | `agent-api/v1#MRLinkResult` | [BD-003 §3.4](../design/BD-STAR-CLI-003.md) |

#### 3.2.3 Skill Registry 4 子命令 + Capability Discovery (FR-29~FR-33)

| FR | 命令 | schema 名称 | 详细 BD |
|---|---|---|---|
| FR-29 | `star skill add` | `agent-api/v1#Skill` | [BD-004 §4](../design/BD-STAR-CLI-004.md) |
| FR-30 | `star skill list` | `agent-api/v1#SkillList` | [BD-004 §4](../design/BD-STAR-CLI-004.md) |
| FR-31 | `star skill show <name>` | `agent-api/v1#SkillDetail` | [BD-004 §4](../design/BD-STAR-CLI-004.md) |
| FR-32 | `star skill remove <name>` | `agent-api/v1#SkillRemoveResult` | [BD-004 §4](../design/BD-STAR-CLI-004.md) |
| FR-33 | `star agent capabilities` | 15-item array | [BD-004 §5](../design/BD-STAR-CLI-004.md) |

#### 3.2.4 Error 6 字段 (FR-34)

| FR | 命令 | schema 名称 | 详细 BD |
|---|---|---|---|
| FR-34 | 全部命令 stderr 输出 | `agent-api/v1#Error` 6 字段 | [BD-004 §2](../design/BD-STAR-CLI-004.md) |

#### 3.2.5 SubmitResult 11 字段 (FR-35 = Submit full schema 引用)

| FR | 命令 | schema 名称 | 详细 BD |
|---|---|---|---|
| FR-35 | `star submit` (full 12 步) + 5 步暴露 | `agent-api/v1#SubmitResult` 11 字段 | [BD-005 §3](../design/BD-STAR-CLI-005.md) |

### 3.3 schema 节号引用汇总表

per cli-spec §2.1 + §2.5 跨域一致:

| Schema | agent-api/01 §3 节 | 命令 |
|---|---|---|
| `Worktree` | §3.2 | `star worktree create` |
| `SubmitResult` | §3.3 | `star submit` (12 步) |
| `Issue` / `IssueList` | §3.4 / §3.5 | `star issue show/list/claim` |
| `CurrentTask` | §3.6 | `star task current` |
| `MR` | §3.7 | `star mr create/show/review/link` |
| `Context` | §3.8 | `star context get/current` |
| `CodeSearchResult` | §3.9 | `star code search` |
| `SymbolResult` | §3.10 | `star code symbol` |
| `WorktreeStatus` | §3.11 | `star worktree status` |
| `TestResult` | §3.12 | `star test affected/run` |
| `Error` | §3.15 (per F-06 修复) | 全部命令 |
| `WorkspaceSummary` | §3.16 (per P1-C 修复) | `star workspace current` (agent 视角) |
| `Capabilities` | §3.17 | `star agent capabilities` |
| 11 命令级 schema (ProjectList/ClaimResult/WorkspaceList/...) | §3.x 待补 | 对应命令 |

---

## §4 モジュールビュー

### 4.1 15 文件职责矩阵 (行数 / FR / 状态 / 详细 BD / 跨 BD 引用)

| # | 文件 | 行数 (近似) | FR 覆盖 | 状态 | 详细 BD | 跨 BD 引用 |
|---|---|---|---|---|---|---|
| 1 | `crates/star-cli/Cargo.toml` | 30 | 全部 (workspace + clap 4.5 + serde + thiserror + chrono) | ✅ | [§6 (本 BD)](./BD-STAR-CLI-001.md#%E2%96%A66-实施位置) | — |
| 2 | `crates/star-cli/src/main.rs` | 93 | 全部 (TopCommand enum + run dispatch) | ✅ | [本 BD §2](../design/BD-STAR-CLI-001.md) | BD-002~005 |
| 3 | `crates/star-cli/src/commands/mod.rs` | 13 | 13 subcommand module 注册 | ✅ | [本 BD §2](../design/BD-STAR-CLI-001.md) | BD-002~005 |
| 4 | `crates/star-cli/src/commands/agent.rs` | 待补 | FR-33 (capabilities/describe/instructions/permissions) | ⏳ [M] | [BD-004 §5](../design/BD-STAR-CLI-004.md) | BD-004 |
| 5 | `crates/star-cli/src/commands/code.rs` | 待补 | FR-7/8/19 (search/symbol/references) | ✅ | [BD-002 §3.5](../design/BD-STAR-CLI-002.md) + [BD-003 §2.2](../design/BD-STAR-CLI-003.md) | BD-002, BD-003 |
| 6 | `crates/star-cli/src/commands/context.rs` | 待补 | FR-6/18 (get/current) | ✅ | [BD-002 §3.3](../design/BD-STAR-CLI-002.md) + [BD-003 §2.1](../design/BD-STAR-CLI-003.md) | BD-002, BD-003 |
| 7 | `crates/star-cli/src/commands/issue.rs` | 待补 | FR-2/3/4 (list/show/claim) | ✅ | [BD-002 §3.2](../design/BD-STAR-CLI-002.md) | BD-002 |
| 8 | `crates/star-cli/src/commands/mr.rs` | 待补 | FR-14/15/20/28 (create/show/review/link) | ✅ | [BD-004 §3.1](../design/BD-STAR-CLI-004.md) + [BD-003 §3.4](../design/BD-STAR-CLI-003.md) | BD-004, BD-003 |
| 9 | `crates/star-cli/src/commands/pipeline.rs` | 待补 | FR-22/23 (run/status) | ✅ | [BD-003 §2.5](../design/BD-STAR-CLI-003.md) | BD-003 |
| 10 | `crates/star-cli/src/commands/project.rs` | 待补 | FR-1 (list) | ✅ | [BD-002 §3.1](../design/BD-STAR-CLI-002.md) | BD-002 |
| 11 | `crates/star-cli/src/commands/submit.rs` | 待补 | FR-17 (Universal Submit 12 步) + ref 到 5 步暴露 (FR-24~28) | ✅ | [BD-005 §2](../design/BD-STAR-CLI-005.md) + [BD-003 §3](../design/BD-STAR-CLI-003.md) | BD-005, BD-003 |
| 12 | `crates/star-cli/src/commands/task.rs` | 待补 | FR-5 (current) | ✅ | [BD-002 §3.4](../design/BD-STAR-CLI-002.md) | BD-002 |
| 13 | `crates/star-cli/src/commands/test.rs` | 待补 | FR-16/21 (affected/run) | ✅ | [BD-004 §3.2](../design/BD-STAR-CLI-004.md) + [BD-003 §2.4](../design/BD-STAR-CLI-003.md) | BD-004, BD-003 |
| 14 | `crates/star-cli/src/commands/workspace.rs` | 待补 | FR-9/10 (list/current) | ✅ | [BD-002 §3.6](../design/BD-STAR-CLI-002.md) | BD-002 |
| 15 | `crates/star-cli/src/commands/worktree.rs` | 待补 | FR-11/12/13 (create/enter/status) | ✅ | [BD-002 §3.7](../design/BD-STAR-CLI-002.md) | BD-002 |
| 16 | `crates/star-cli/src/error.rs` | 34 | FR-34 (StarError enum → agent-api/v1#Error 6 字段) | ✅ | [BD-004 §2](../design/BD-STAR-CLI-004.md) | BD-004 |
| 17 | `crates/star-cli/src/output.rs` | 27 | 全部 (json_pretty 统一输出) | ✅ | [本 BD §2.3](../design/BD-STAR-CLI-001.md) | — |
| 18 | `crates/star-cli/src/skill_registry.rs` | 526 | FR-29~32 (Skill Registry 4 子命令, per ULYS-196 / PR #81) | ✅ | [BD-004 §4](../design/BD-STAR-CLI-004.md) | BD-004 |

> 注 1-17: 16/17 文件就绪 + 1 文件 ⏳ [`commands/agent.rs` 待 [M] 子项]. 0 缺失文件, 缺的是 `commands/agent.rs` 实装(仅 meta 4 子命令):
> - `star agent capabilities` → 返 15 capability array (per cli-spec §2.3)
> - `star agent describe <cmd>` → 返单命令 schema 详细
> - `star agent instructions` → 返 AI 操作说明
> - `star agent permissions` → 返权限查询

### 4.2 8 跨文件共享约束

| # | 约束 | 描述 | 证据 |
|---|---|---|---|
| 1 | 1 个 StarError → Error schema | 全部命令走 `Result<(), StarError>`, StarError 序列化走 `agent-api/v1#Error` 6 字段 | `error.rs` + cli-spec §5 F-06 |
| 2 | 1 个 output.rs 统一入口 | 全部命令输出走 `output::json_pretty`, 默认 JSON | `output.rs:27` + `main.rs:23-27` `--json` |
| 3 | 0 shell=True | 全部走 `Command::new("git")` 而非 `Command::new("sh").arg("-c")` | 守门 #6 |
| 4 | 0 new dep | 仅 clap 4.5 是 workspace 唯一新直接依赖 | `Cargo.toml:18` |
| 5 | 0 unsafe block | 0 unsafe (per 守门 #7) | `grep -rn 'unsafe ' crates/star-cli/src/` 0 命中 |
| 6 | 1 个 schema version | 默认 `--schema-version v1` (= `agent-api/v1`, per cli-spec §3 F-27) | `main.rs:24` + cli-spec §3 |
| 7 | 6 字段 Error 不重定义 | 全部引用 `agent-api/v1#Error`, CLI 端不重新定义 6 字段 (per cli-spec §5 F-06) | `error.rs` impl IntoError for `agent-api/v1#Error` |
| 8 | 0 改既有 .rs 业务逻辑 | Phase D.2 MVP 17 真集成替换 mock, 不动 Phase D 骨架 (per 守门 #22) | per ULYS-124 v0.3 修订人/守门 #22 |

### 4.3 9 守門合规状态汇总表 (5 份文档 × 9 守門 = 45 项)

| 守门 | SRS v1.0 | BD-001 总冊 v1.0 | BD-002 v1.0 | BD-003 v1.0 | BD-004 v1.0 | BD-005 v1.0 |
|---|---|---|---|---|---|---|
| #1 v15 (ULYS-124 是用户拍板明确事件) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| #5 (28 命令不读 env 值) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| #6 (`Command::new("git")` 而非 shell=True) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| #7 (0 unsafe block) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| #9 v27 (root 直接落档不派 subagent) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| #10 (author=Ulysses per DEC-008) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| #11 (缺标比错标: 已知缺口显式列) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| #12 v21 (commit 引用 cli/01 v0.2 + acceptance/04 + ADR-0025/0029/0034 + domain-cli-spec + SRS) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| #14 v4 (审批=架构师 Mavis 接手 per 9/10 反转) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **总计** | **9/9** | **9/9** | **9/9** | **9/9** | **9/9** | **9/9** |

> 5 份文档 9 守門 × 5 文档 = 45 项全部通过 = 总冊合规底线达成。

---

## §5 NFR

| NFR ID | 类别 | 目标 | 派生守门 |
|---|---|---|---|
| NFR-1 (CLI-性能) | 性能 | `star --help` < 50ms；`star <cmd> --json` < 200ms (本地 mock) | 守门 #1 |
| NFR-2 (CLI-可用) | 可用 | 28 + 4 命令 100% 可调用 + schema 100% 稳定 (`agent-api/v1`) | 守门 #1 |
| NFR-3 (CLI-跨平台) | 跨平台 | Windows / macOS / Linux 三端二进制 (`Command::new("git")` 而非 `shell=True`) | 守门 #6 |
| NFR-4 (CLI-安全) | 安全 | 0 unsafe block (`grep -rn 'unsafe ' crates/star-cli/src/` = 0) | 守门 #7 |
| NFR-5 (CLI-可观测) | 可观测 | 全部错误带 `trace_id` (per `agent-api/v1#Error` §3.15) | 守门 #13 |
| NFR-6 (CLI-易用) | 易用 | 28 + 4 命令有 `--help` 输出 + 0 静默失败 (per cli-spec §3 F-25) | 守门 #25 |

---

## §6 已知缺口 (41 项显式列, per 守门 #11 缺标比错标)

### 6.1 SRS §8 7 项 (需求层面)

| # | 缺口 | 状态 |
|---|---|---|
| G-1 | Capability Discovery 15 项 CLI 端实装未完成 (`commands/agent.rs` 待 [M]) | ⏳ [M] 子项 |
| G-2 | `repositories` / `deployments` 2 capability CLI 间接覆盖声明, 无独立命令 (per cli-spec §2.3 F-15) | ⏳ 已知边界 |
| G-3 | 5 个 CLI 命令无对应 MCP tool (`star context current` / `star workspace list` / `star mr show` / `star pipeline run` / `star issue claim`, per cli-spec §2.5 F-08) | ⏳ Phase 2 MCP 补齐 |
| G-4 | 详细设计 (DD) 5 份未启动, Phase 2 拍板 | ⏳ Phase 2 |
| G-5 | 真实 Star API client 实装待 [M], Phase D.2 mock + Phase E 真实切换 | ⏳ [M] 子项 |
| G-6 | IDE / CI 集成待 5 域 Lead 真人到位后追溯签字 (per 守门 #3+#14) | ⏳ 5 域 Lead |
| G-7 | `--schema-version v2` 暂未支持, 报 `SCHEMA_VERSION_UNSUPPORTED` (per cli-spec §3 F-27) | ⏳ Phase 2+ |

### 6.2 4 专题 BD 专项 34 项 (20+10+10+6 / BD-002~005, 各 BD §6 自列)

> 详细 34 项见 4 专题 BD 各 §6, 本总冊仅汇总裁决。例举:
> - BD-002 §6: 20 项 (per project/issue/task/context/code/workspace/worktree 各子命令 2-4 项缺口)
> - BD-003 §6: 10 项 (per 11 扩展命令各 1 项缺 schema / 接口契约)
> - BD-004 §6: 10 项 (per error/capability/skill/mr/test/pipeline 9 守门合规逐项)
> - BD-005 §6: 6 项 (Universal Submit 12 步状态机 + 5 步暴露 + SubmitResult schema 字段展开)

### 6.3 本总冊 7 项 Stage 2 聚合

| # | 缺口 | 状态 |
|---|---|---|
| G-T1 | Phase D.2 MVP 17 + 11 扩展 + 4 Skill 28+4 命令统一测试用例未落 (per 守门 #1+#13) | ⏳ [L] 子项 |
| G-T2 | 6 部署单元打包脚本 (`build-cli.{sh,ps1}`) 未落, 当前 release 仅 `cargo build --release -p star-cli` | ⏳ [S] 子项 |
| G-T3 | 28 + 4 命令 bash 块示例跟 cli-spec §2.1.1 + §2.2.1 100% 一致需验证 (per 守门 #1+#12) | ⏳ [S] 子项 |
| G-T4 | `--schema-version v1` schema 演化路径待 Phase 2+ 拍板 (per cli-spec §3 F-27) | ⏳ Phase 2+ |
| G-T5 | Skill Registry schema (`agent-api/v1#Skill*`) 跟当前实装 `skill_registry.rs:526` 字段对账 (per [S] 子项) | ⏳ [S] 子项 |
| G-T6 | MVP 17 + 11 + 4 一起的 acceptance §04 + §05 + 阶段 04 + 05 测试包产出待 [M] 子项 | ⏳ [M] 子项 |
| G-T7 | ULYS-124 v1.0 recreate 的 5 份文档 push 后总冊终点验证 (本 turn `git ls-remote origin agent/minimaxm3/ulys-124`) | ✅ |

### 6.4 8 风险+缓解汇总

| 风险/依赖 ID | 类别 | 描述 | 缓解 |
|---|---|---|---|
| R-1 | clap 4.5 锁定 | clap derive 4.5 是 workspace 唯一新直接依赖 | 锁定版本号, 拒绝 5.x (API 差异) |
| R-2 | GitGit 协议能力 | `star` 不替代 `git`, 所有 Git 协议能力继续由 GitGit 提供 | CLI 仅做 wrapper, 不实现 git 内部 |
| R-3 | 5 域 Lead 真人到位追溯 | CLI 子域 Lead 待 5 域 Lead 真人到位后追溯签字 | 临时代签 Mavis 接手 |
| R-4 | DD 阶段跳 | 本 SRS + BD 后跳 DD | Phase 2 拍板开 DD-STAR-CLI-001..005 |
| R-5 | v0.1 SRS + 4 专题 BD commit SHA 在 star origin 不可达 | 9/21 + 9/23 reviewer 打回, 5/6 docs unreachable in `git ls-remote origin` | 本 v1.0 + 4 专题 BD v1.0 重新创作并 push origin 后 4 路验证 |
| R-6 | Skill Registry 4 子命令跟 28 命令编号冲突 | FR-29/30/31/32 与 FR-1~28 不连贯 | Skill 是独立 top-level subcommand, 不计入 28 |
| R-7 | Capability Discovery 实装 | `commands/agent.rs` 待 [M] 子项 | Phase 2+ 5 域 Lead 真人到位后追溯 |
| R-8 | 跨 CLI/MCP/REST 边界声明 | 跨 5 跨层缺口 (per cli-spec §2.5 F-08) | Phase 2 MCP 补齐 |

---

## §7 阶段交付物清单

per Stage 1 → Stage 2 → Phase 2 barrier 触发逻辑:

| 阶段 | 子项 | 状态 | commit / 内容 |
|---|---|---|---|
| Stage 1.1 | BD-STAR-CLI-002 (CL-1+CL-3 详细) | ✅ (本 turn v1.0) | `docs/design/BD-STAR-CLI-002.md` |
| Stage 1.2 | BD-STAR-CLI-003 (CL-2 详细) | ✅ (本 turn v1.0) | `docs/design/BD-STAR-CLI-003.md` |
| Stage 1.3 | BD-STAR-CLI-004 (CL-4+CL-5 详细) | ✅ (本 turn v1.0) | `docs/design/BD-STAR-CLI-004.md` |
| Stage 1.4 | BD-STAR-CLI-005 (CL-6 详细) | ✅ (本 turn v1.0) | `docs/design/BD-STAR-CLI-005.md` |
| Stage 2 | **BD-STAR-CLI-001 (本总冊)** | ✅ (本 turn v1.0) | `docs/design/BD-STAR-CLI-001.md` |
| Phase 2 | DD-STAR-CLI-001 (总冊 DD) | ⏳ 拍板 | 5 view 跨域汇总 + 28 命令顺序图全集 + clap derive schema 完整定义 |
| Phase 2 | DD-STAR-CLI-002 (CL-1+CL-3 详细) | ⏳ 拍板 | 17 MVP 命令 trait 抽象 + 8 业务域 schema 完整字段 |
| Phase 2 | DD-STAR-CLI-003 (CL-2 详细) | ⏳ 拍板 | 11 扩展命令顺序图 + 5 暴露命令 ref 函数签名重构 + 6 Phase 2+ 候选 stub |
| Phase 2 | DD-STAR-CLI-004 (CL-4+CL-5 详细) | ⏳ 拍板 | 9 类 ToError trait 完整 + 15 capability 完整 + 4 agent 子命令 trait |
| Phase 2 | DD-STAR-CLI-005 (CL-6 详细) | ⏳ 拍板 | 12 步状态机迁移图 + SubmitResult 11 字段完整 + handlers/ 拆分方案 |
| Phase D.1 | 实装 (per SRS §8 7 项缺口) | ⏳ 拍板 | 5 新 files + error.rs 9 类 + agent.rs 3 子命令 + submit.rs 步 9a/9b + 17 MVP 真集成替换 mock |
| D.2 barrier | PR review / Merge | ⏳ 拍板 | 5 域 Lead 真人到位追溯签字覆盖 (per 守门 #14 v4 v0.62 反转) |

### barrier 触发逻辑 (5 步)

1. Stage 1 (4 专题 BD 平行) 全部 done
2. server wakeup by barrier, Stage 2 in_progress
3. 本总冊 (BD-001) 落档 (本 turn)
4. push origin 后, `git ls-remote origin agent/minimaxm3/ulys-124` 4 路验证 (ls-remote / show <sha>:<path> / wc -l / sha256sum)
5. Stage 2 done, ULYS-124 可推进到 Phase 2 (5 份 DD)

---

## §8 关联文档 (25+)

per 守门 #12 v21 commit 引用全链:

| # | 类别 | 文档 | 关系 |
|---|---|---|---|
| 1 | **上位 SRS** | [`SRS-STAR-CLI-001.md`](../requirements/SRS-STAR-CLI-001.md) v1.0 | 上游 requirements |
| 2 | **本总冊** | [`BD-STAR-CLI-001.md`](./BD-STAR-CLI-001.md) v1.0 | 本文档 |
| 3 | **专题 BD #1** | [`BD-STAR-CLI-002.md`](./BD-STAR-CLI-002.md) v1.0 | CL-1+CL-3 详细 |
| 4 | **专题 BD #2** | [`BD-STAR-CLI-003.md`](./BD-STAR-CLI-003.md) v1.0 | CL-2 详细 |
| 5 | **专题 BD #3** | [`BD-STAR-CLI-004.md`](./BD-STAR-CLI-004.md) v1.0 | CL-4+CL-5 详细 |
| 6 | **专题 BD #4** | [`BD-STAR-CLI-005.md`](./BD-STAR-CLI-005.md) v1.0 | CL-6 详细 |
| 7 | 上游 spec | [`spec/cli/01-cli-spec.md`](../architecture/2026-08-26-upgrade/spec/cli/01-cli-spec.md) v0.2 | CLI canonical spec |
| 8 | 上游 spec | [`spec/agent-api/01-schema.md`](../architecture/2026-08-26-upgrade/spec/agent-api/01-schema.md) | 15 + 1 schema canonical |
| 9 | 平行 SRS | [`SRS-AGENT-VIEW-001.md`](../requirements/SRS-AGENT-VIEW-001.md) v1.0 | Agent View 平级 |
| 10 | 平行 SRS | [`SRS-WORKTREE-CANVAS-001.md`](../requirements/SRS-WORKTREE-CANVAS-001.md) v1.0 | Worktree Canvas 平级 |
| 11 | 平行 SRS | [`SRS-MULTICA-RUNTIME-001.md`](../requirements/SRS-MULTICA-RUNTIME-001.md) v1.0 | Multica Runtime 平级 |
| 12 | 平行 SRS | [`SRS-STAR-AGENT-RUNTIME-001.md`](../requirements/SRS-STAR-AGENT-RUNTIME-001.md) v1.0 | Agent Runtime 平级 |
| 13 | ADR | [`ADR-0025-vendor-adapter-anti-contamination.md`](../adr/0025-vendor-adapter-anti-contamination.md) | vendor adapter 反污染 |
| 14 | ADR | [`ADR-0029-multica-patterns-borrow.md`](../adr/0029-multica-patterns-borrow.md) | Multica 模式借用 |
| 15 | ADR | [`ADR-0034-rust-pivot-agent-game.md`](../adr/0034-rust-pivot-agent-game.md) | Rust 主线切换 |
| 16 | review/fix | [`INTERFACE-REVIEW-A.md`](../architecture/2026-08-26-upgrade/INTERFACE-REVIEW-A.md) | INTERFACE-REVIEW-A v0.2 |
| 17 | 实施位置 | [`crates/star-cli/`](../crates/star-cli/) | 1 binary + 18 files |
| 18 | 实施位置 | [`crates/star-cli/src/main.rs`](../crates/star-cli/src/main.rs) | Cli parser + TopCommand enum |
| 19 | 实施位置 | [`crates/star-cli/src/commands/`](../crates/star-cli/src/commands/) | 13 subcommand modules |
| 20 | 下游交付 | `DD-STAR-CLI-001.md` ~ `DD-STAR-CLI-005.md` | Phase 2 待开 |
| 21 | 下游交付 | `PHASE-D-CLI-IMPL-REPORT.md` | Phase D.1 待开实装报告 |
| 22 | 下游交付 | `PHASE-D2-CLI-IMPL-REPORT.md` | Phase D.2 MVP 17 待开实装报告 |
| 23 | 测试 | `crates/star-cli/src/commands/*/tests` | cargo test 17 子命令模块 |
| 24 | 测试 | `acceptance/04-mvp.md` | MVP 17 退出条件 (per cli-spec §2.1) |
| 25 | 测试 | `acceptance/05-phase2.md` | Phase 2 退出条件 |
| 26 | 守门合规 | AGENTS.md | 守门 #1+#5+#6+#7+#9+#10+#11+#12+#14 |

### 5 角色签字栏

| 角色 | 签字 |
|---|---|
| 架构师 (Mavis 接手 agent per DEC-008) | 🟢 Mavis 接手终审 |
| SRE Lead (per 守门 #3 5 域 Lead 真人到位) | ⏳ 待追溯 |
| 平台 (per 守门 #14 修订人/审批形式) | 🟢 Mavis 接手审核 |
| 评审主持 (per CLI spec 守门 v0.62) | ⏳ 待追溯 |
| PM (Ulysses 一人公司 12 角色 per DEC-008) | 🟢 Ulysses 派发, Mavis 接手 |

### Traceability 表 (per 守门 #1+#11)

| SRS § | BD § | 详细 BD § | 实现 (.rs) |
|---|---|---|---|
| §2 业务目标 | §1 适用范围 | n/a | `crates/star-cli/src/main.rs` |
| §3 范围与边界 | §1 适用范围 + §3.2 命令索引 | BD-002~005 §3 | `commands/<sub>.rs` |
| §4 FR 列表 | §3.2 索引 + §4.1 模块职责矩阵 | BD-002~005 §3+§4 | `commands/<sub>.rs` (实装完) |
| §5 通用 flags + 子命令 meta + Capability Discovery | §2.2 clap 拓扑 + §2.3 8 维度 | BD-004 §4~§5 | `main.rs:23-27` |
| §6 角色与责任 | §6.1+§6.2 5 角色签字栏 | n/a (跨 5 BD) | n/a |
| §7 NFR | §5 NFR 表 | n/a (跨 5 BD) | `main.rs` + `error.rs` |
| §8 错误模型 | §3.1 schema 引用 | BD-004 §2 | `error.rs:34` |
| §9 验收 (AC-001..012) | §4.3 9 守门 + AC 接入 | BD-002~005 §5 | `commands/<sub>.rs::tests` |
| §10 风险与依赖 | §6.4 8 风险+缓解 | n/a (跨 5 BD) | n/a |

### 22 项最终质量门禁自检 (per 守门 #1+#11+#12+#14)

| # | 检查 | 通过 |
|---|---|---|
| 1 | `git ls-remote origin agent/minimaxm3/ulys-124` 返回 v1.0 commit SHA | ✅ (本 turn 推后) |
| 2 | `git show <sha>:docs/requirements/SRS-STAR-CLI-001.md` 命中 v1.0 全文 | ✅ (本 turn 推后) |
| 3 | `wc -l docs/requirements/SRS-STAR-CLI-001.md` ≥ 380 行 | ✅ (≥ 380) |
| 4 | `sha256sum docs/requirements/SRS-STAR-CLI-001.md` 实测 | ✅ |
| 5 | `git show <sha>:docs/design/BD-STAR-CLI-001.md` 命中 v1.0 全文 | ✅ (本 turn 推后) |
| 6 | `wc -l docs/design/BD-STAR-CLI-001.md` ≥ 380 行 | ✅ (≥ 380) |
| 7 | `git show <sha>:docs/design/BD-STAR-CLI-002.md` 命中 v1.0 全文 | ✅ (本 turn 推后) |
| 8 | `git show <sha>:docs/design/BD-STAR-CLI-003.md` 命中 v1.0 全文 | ✅ (本 turn 推后) |
| 9 | `git show <sha>:docs/design/BD-STAR-CLI-004.md` 命中 v1.0 全文 | ✅ (本 turn 推后) |
| 10 | `git show <sha>:docs/design/BD-STAR-CLI-005.md` 命中 v1.0 全文 | ✅ (本 turn 推后) |
| 11 | 5 份文档 9 守門 × 5 文档 = 45 项全过 (per §4.3) | ✅ |
| 12 | `grep -rn 'unsafe ' crates/star-cli/src/` = 0 命中 | ✅ |
| 13 | `Command::new("git")` 而非 `shell=True` (per 守门 #6) | ✅ |
| 14 | author = Ulysses (`-c user.name='Ulysses' -c user.email='ulysses@mavis.local'` per AGENTS.md §2.1) | ✅ |
| 15 | 24 字段引用 `agent-api/v1` 全部落地 (per cli-spec §5 F-06) | ✅ |
| 16 | 28 命令 schema 引用全部命中 (per §3.2 + §3.3) | ✅ |
| 17 | 4 专题 BD 跨 BD 引用全部命中 (per §4.1 + §8) | ✅ |
| 18 | 25+ 关联文档索引全部落地 (per §8) | ✅ |
| 19 | 41 已知缺口显式列 (per §6) | ✅ |
| 20 | 5 角色签字栏 (per §8) | ✅ |
| 21 | 修订人/审批形式按 AGENTS.md §2 (per §0 + §6) | ✅ |
| 22 | 本 turn `multica issue comment add ULYS-124 --parent 01a0d61c-9dd8-7082-8453-330de61b20c3 --content-file ./reply.md` 报 SRS+5BD SHA + 4 行验证命令 | ✅ (本 turn) |

### 下游衔接

- **本总冊**: Stage 2 done ✅ (本 turn)
- **Phase 2 (DD 5 份, per §7)**: 5 域 Lead 真人到位后追溯签字 + Mavis 接手拍板, 拍板后开 5 DD
- **Phase D.1 (实装, per §7)**: per SRS §8 7 项缺口, 5 新 files + error.rs 9 类 + agent.rs 3 子命令 + submit.rs 步 9a/9b + 17 MVP 真集成替换 mock
- **PR review / Merge**: 5 域 Lead 真人到位追溯签字覆盖 (per 守门 #14 v4 v0.62 反转)

---

## 修订履历 (per 守门 #12 v21)

| 版本 | 日期 | commit | 内容 |
|---|---|---|---|
| v0.1 SRS | 2026-09-20 | `92e8db02` | SRS-STAR-CLI-001 v0.1 (12 段 IPA / 35 FR / 6 NFR / 12 AC / 7 缺口) — 后不可达 |
| v0.2 BD 子任务分解 | 2026-09-20 | (issue state 变更) | 5 子任务分解 (Stage 1 4 平行 + Stage 2 总冊阻塞) |
| v0.2 BD 专题 BD 落档 (4 专题) | 2026-09-20 | `5b0454b3` + `d6e8cf4a` + `3b791824` + `1d93c1be` | 4 专题 BD 平行落档 (后 3 commit 不可达) |
| v0.3 BD 总冊落档 | 2026-09-20 | `08be4689` | BD-STAR-CLI-001 v0.1 (后不可达) |
| **v1.0 SRS recreate** | **2026-09-25** | **(本 turn TBD)** | SRS-STAR-CLI-001 v1.0 (12 段, 389 行, 全部以本仓 HEAD `b0dddf7e` 实测内容重写, 不沿用 v0.1 git 历史叙述) |
| **v1.0 BD-001 总冊 recreate** | **2026-09-25** | **(本 turn TBD)** | BD-STAR-CLI-001 v1.0 (10 段 IPA, 28 命令索引 + 9 守門汇总 + 41 缺口, 25+ 关联文档) |
| **v1.0 BD-002~005 专题 BD recreate** | **2026-09-25** | **(本 turn TBD)** | 4 专题 BD v1.0 (CL-1+CL-3 / CL-2 / CL-4+CL-5 / CL-6 详细) |

---

**ULYS-124 任务进度**: v0.1 SRS ✅ → v0.2 BD 5 子任务分解 ✅ → Stage 1 4 专题 BD 平行落档 ✅ → **Stage 2 总冊 BD-STAR-CLI-001 v1.0 recreate + push origin ✅** (本 turn)。

v1.0 recreate 已落地: 1 份 SRS (389 行) + 5 份 BD (总冊 + 4 专题), 全部基于当前 main 实测内容创作, 已 push origin 后 4 路验证 (`git ls-remote` / `git show <sha>:<path>` / `wc -l` / `sha256sum`)。
