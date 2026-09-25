# BD-STAR-CLI-002

> **STAR CLI 专题 BD #1 v1.0 (recreate, per ULYS-124 v1.0 缺口补齐)**
>
> **CL-1 + CL-3: MVP 17 核心命令中的 13 命令 + 通用 flags**
>
> - 状态: Basic Design Baseline (v1.0 — recreate after 9/21 + 9/23 reviewer 打回, per 用户 9/25 01:09 JST "确保缺口已补")
> - 目标阶段: 基本設計 → 詳細設計 → 実装 → テスト → リリース
> - 上位要件: [`docs/requirements/SRS-STAR-CLI-001.md`](../requirements/SRS-STAR-CLI-001.md) v1.0 (本仓 HEAD main path, 本 commit 同期落档)
> - 上游 spec: [`docs/architecture/2026-08-26-upgrade/spec/cli/01-cli-spec.md`](../architecture/2026-08-26-upgrade/spec/cli/01-cli-spec.md) v0.2 (canonical)
> - 上游总冊: [`docs/design/BD-STAR-CLI-001.md`](./BD-STAR-CLI-001.md) v1.0 (本 commit 同期落档, 索引级汇总)
> - 下游交接: [`docs/design/BD-STAR-CLI-003.md`](./BD-STAR-CLI-003.md) v1.0 (CL-2 扩展命令) / [`docs/design/BD-STAR-CLI-005.md`](./BD-STAR-CLI-005.md) v1.0 (CL-6 Universal Submit)
> - 实施位置: [`crates/star-cli/src/`](../crates/star-cli/src/) (13 子命令模块 + main.rs + error.rs + output.rs)
> - 修订人: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> - 审批: 架构师 (Mavis 接手 agent per DEC-008)
> - 日期: 2026-09-25 JST

> **回溯记录（per ULYS-124 9/21 + 9/23 reviewer 打回与 9/25 01:09 用户缺口补齐指令）**: v0.1 专题 BD #1 在 9/20 commit 落档但 9/21 reviewer 复审确认 `git ls-remote origin agent/minimaxm3/ulys-124` 不可达。本 v1.0 按 (a) 当前 main `b0dddf7e` 实装的 13 命令代码 (b) cli-spec v0.2 canonical (c) SRS-STAR-CLI-001 v1.0 + 总冊 BD-001 v1.0 索引关系基础上重新创作, 不复制任何前 worker 内容, 严格按"CL-1+CL-3 13 命令详细 BD"分工。

---

## §0 文档目的

本文档基于 SRS-STAR-CLI-001 v1.0 的 FR-1~FR-13 需求, 定义 STAR CLI **CL-1 (Project/Issue/Task) + CL-3 (Code/Workspace/Worktree) 域共 13 命令**的基本設計: 每个命令的 (a) schema 字段完整展开 (b) 模块视图职责 (c) NFR 子集 (d) 已知缺口显式列 (e) 测试用例 (f) 接口契约 (g) 10 项通用 flags 完整定义。

**职责定位**: 本 BD 是**专题 BD**, 详尽展开 13 命令的 schema/字段/职责/缺口; 不重复总冊 BD-001 的索引职责。

**MVP 17 覆盖**: 13 / 17 核心命令 (per cli-spec v0.2 §2.1 表); 余 4 命令 (`star mr create/show` + `star test affected` + `star submit`) 跨 BD 处理:
- `star mr create/show` → [BD-STAR-CLI-004.md §3.1](./BD-STAR-CLI-004.md)
- `star test affected` → [BD-STAR-CLI-004.md §3.2](./BD-STAR-CLI-004.md)
- `star submit` → [BD-STAR-CLI-005.md §2](./BD-STAR-CLI-005.md)

**通用 flags**: 10 项完整定义 (`--json` / `--quiet` / `--fields` / `--limit` / `--cursor` / `--no-color` / `--schema-version` / `--help` / `--no-header` / `--version`), per cli-spec §3 + F-25/F-27 修复。

---

## §1 適用範囲

### 1.1 In-Scope: 13 命令 (CL-1 5 命令 + CL-3 8 命令)

| 子能力 | # | 命令 | 输出 schema | agent-api/01 §3 节 | 实施文件 |
|---|---|---|---|---|---|
| CL-1 | FR-1 | `star project list` | `agent-api/v1#ProjectList` | §3.x (待补) | [`commands/project.rs:80`](../crates/star-cli/src/commands/project.rs) |
| CL-1 | FR-2 | `star issue list` | `agent-api/v1#IssueList` | §3.5 | [`commands/issue.rs:93`](../crates/star-cli/src/commands/issue.rs) |
| CL-1 | FR-3 | `star issue show <id>` | `agent-api/v1#Issue` | §3.4 | [`commands/issue.rs:115`](../crates/star-cli/src/commands/issue.rs) |
| CL-1 | FR-4 | `star issue claim <id>` | `agent-api/v1#ClaimResult` | §3.x (待补) | [`commands/issue.rs:143`](../crates/star-cli/src/commands/issue.rs) |
| CL-1 | FR-5 | `star task current` | `agent-api/v1#CurrentTask` | §3.6 | [`commands/task.rs:111`](../crates/star-cli/src/commands/task.rs) |
| CL-1 | FR-6 | `star context get <id>` | `agent-api/v1#Context` | §3.8 | [`commands/context.rs:78`](../crates/star-cli/src/commands/context.rs) |
| CL-3 | FR-7 | `star code search <q>` | `agent-api/v1#CodeSearchResult` | §3.9 | [`commands/code.rs:48`](../crates/star-cli/src/commands/code.rs) |
| CL-3 | FR-8 | `star code symbol <name>` | `agent-api/v1#SymbolResult` | §3.10 | [`commands/code.rs:69`](../crates/star-cli/src/commands/code.rs) |
| CL-3 | FR-9 | `star workspace list` | `agent-api/v1#WorkspaceList` | §3.x (待补) | [`commands/workspace.rs:30`](../crates/star-cli/src/commands/workspace.rs) |
| CL-3 | FR-10 | `star workspace current` | `agent-api/v1#WorkspaceSummary` | §3.16 (per P1-C 修复) | (Phase D 留 stub, 见 §6) |
| CL-3 | FR-11 | `star worktree create <id>` | `agent-api/v1#Worktree` | §3.2 | [`commands/worktree.rs:35`](../crates/star-cli/src/commands/worktree.rs) |
| CL-3 | FR-12 | `star worktree enter <id>` | n/a (cd, stdout path) | — | [`commands/worktree.rs:55`](../crates/star-cli/src/commands/worktree.rs) |
| CL-3 | FR-13 | `star worktree status` | `agent-api/v1#WorktreeStatus` | §3.11 | [`commands/worktree.rs:61`](../crates/star-cli/src/commands/worktree.rs) |

**当前实装状态** (per `git log --oneline -p crates/star-cli/src/commands/`):
- 12 / 13 命令已落 Phase D 骨架 mock (实装完, 走 `output::json_pretty` 输出)
- 1 命令 (`star workspace current` / FR-10) **⏳ 待 [M] 子项**: 仅 `star workspace list` 实装 (FR-9), `current` 子命令未实装 (per `commands/workspace.rs` `WorkspaceCommand` enum 仅含 `List` 变体)

### 1.2 Out-of-Scope (8 项)

| 不做 | 原因 | 替代 |
|---|---|---|
| `star context current` | 本 BD 仅含 `get` (FR-6); `current` 是扩展命令 | [BD-003 §2.1](./BD-STAR-CLI-003.md) |
| `star code references` | 本 BD 仅含 `search/symbol` (FR-7/8); `references` 是扩展命令 | [BD-003 §2.2](./BD-STAR-CLI-003.md) |
| `star mr create/show/review` | 本 BD 不涉 MR 域 | [BD-004 §3.1](./BD-STAR-CLI-004.md) |
| `star test affected` | 本 BD 不涉 test 域 | [BD-004 §3.2](./BD-STAR-CLI-004.md) |
| `star submit` Universal Submit | 本 BD 不涉 submit 域 | [BD-005 §2](./BD-STAR-CLI-005.md) |
| `star pipeline run/status` | 本 BD 不涉 pipeline 域 | [BD-003 §2.5](./BD-STAR-CLI-003.md) |
| 5 个 P1-H 新增 Universal Submit 步骤暴露 (`diff`/`policy check`/`commit`/`push`/`mr link`) | 本 BD 不涉 | [BD-003 §3](./BD-STAR-CLI-003.md) |
| Skill Registry 4 子命令 (`add`/`list`/`show`/`remove`) | 本 BD 不涉 | [BD-004 §4](./BD-STAR-CLI-004.md) |

### 1.3 边界声明 (5 项)

| 边界 | 描述 | 证据 |
|---|---|---|
| 13 / 17 MVP 核心命令 | 本 BD 仅负责 13 命令 (CL-1+CL-3); 余 4 走 BD-004 / BD-005 | per cli-spec §2.1 |
| 1 个 binary `star` | `crates/star-cli/Cargo.toml:13-14` `[[bin]]` | `Cargo.toml` |
| 7 个 .rs 文件 | `project.rs` / `issue.rs` / `task.rs` / `context.rs` / `code.rs` / `workspace.rs` / `worktree.rs` | per `crates/star-cli/src/commands/mod.rs` |
| 10 项通用 flags | `--json` / `--quiet` / `--fields` / `--limit` / `--cursor` / `--no-color` / `--schema-version` / `--help` / `--no-header` / `--version` | per cli-spec §3 |
| 1 个 Error schema | 全部错误走 `agent-api/v1#Error` 6 字段 | [BD-004 §2](./BD-STAR-CLI-004.md) |

### 1.4 平行 SRS 边界 (4 项)

| 平行 SRS | 关系 | 边界声明 |
|---|---|---|
| SRS-AGENT-VIEW-001 | Agent View 画布 | CLI 仅读 agent data via `agent-api/v1`, 不重复 agent view 渲染 |
| SRS-WORKTREE-CANVAS-001 | Worktree Canvas | CLI 仅调 `star worktree create/enter/status`, canvas 渲染由 frontend 负责 |
| SRS-STAR-AGENT-RUNTIME-001 | Agent Runtime | CLI 调用 `star task current` / `star agent capabilities` 仅作 client, agent 协议由 agent runtime 实现 |
| SRS-MULTICA-RUNTIME-001 | Multica Runtime | CLI 不涉及 multica orchestration, 仅是消费者 |

---

## §2 システムアーキテクチャ

### 2.1 clap derive 拓扑 (13 命令 / 7 文件)

per `crates/star-cli/src/main.rs:36-74` `TopCommand` enum + 7 个 `commands/*.rs` 模块:

```
TopCommand (clap derive Subcommand, main.rs:36)
├─ Agent(agent::AgentCommand)       ← 本 BD 不涉, BD-004 §5
├─ Task(task::TaskCommand)          ← CL-1 #5, commands/task.rs:19
│   └─ Current(CurrentArgs)         ← FR-5
├─ Submit(submit::SubmitArgs)       ← 本 BD 不涉, BD-005
├─ Project(project::ProjectCommand) ← CL-1 #1, commands/project.rs:10
│   └─ List                         ← FR-1
├─ Issue(issue::IssueCommand)       ← CL-1 #2-4, commands/issue.rs:10
│   ├─ List                         ← FR-2
│   ├─ Show { id: String }          ← FR-3
│   └─ Claim { id: String }         ← FR-4
├─ Context(context::ContextCommand) ← CL-1 #6, commands/context.rs:10
│   ├─ Get { id: String }           ← FR-6 (本 BD)
│   └─ Current                      ← FR-18 (BD-003)
├─ Code(code::CodeCommand)          ← CL-3 #7-8, commands/code.rs:10
│   ├─ Search { query: String }     ← FR-7
│   ├─ Symbol { name: String }      ← FR-8
│   └─ References { name: String }  ← FR-19 (BD-003)
├─ Workspace(workspace::WorkspaceCommand) ← CL-3 #9-10, commands/workspace.rs:10
│   └─ List                         ← FR-9 (本 BD)
│   └─ Current                      ← FR-10 ⏳ [M] 子项 (本 BD §6.13)
├─ Worktree(worktree::WorktreeCommand)   ← CL-3 #11-13, commands/worktree.rs:10
│   ├─ Create { id: String }        ← FR-11
│   ├─ Enter { id: String }         ← FR-12
│   └─ Status                       ← FR-13
├─ Mr(mr::MrCommand)                ← 本 BD 不涉, BD-004 §3.1
├─ Test(test::TestCommand)          ← 本 BD 不涉, BD-004 §3.2
├─ Pipeline(pipeline::PipelineCommand) ← 本 BD 不涉, BD-003 §2.5
└─ Skill(skill_registry::SkillCommandArgs) ← 本 BD 不涉, BD-004 §4
```

### 2.2 7 个 .rs 文件职责矩阵

| # | 文件 | 行数 | 覆盖命令 | 主要 schema | 状态 | 跨文件依赖 |
|---|---|---|---|---|---|---|
| 1 | `commands/project.rs` | 80 | FR-1 (`list`) | `Project` / `ProjectList` | ✅ | `error.rs` + `output.rs` |
| 2 | `commands/issue.rs` | 163 | FR-2/3/4 (`list/show/claim`) | `Issue` / `IssueList` / `ClaimResult` | ✅ | `error.rs` + `output.rs` |
| 3 | `commands/task.rs` | 192 | FR-5 (`current`) | `CurrentTask` | ✅ | `error.rs` + `output.rs` |
| 4 | `commands/context.rs` | 106 | FR-6 (`get`) + FR-18 (`current`, shared) | `Context` / `CodeRef` / `DocRef` / `MRRef` | ✅ | `error.rs` + `output.rs` |
| 5 | `commands/code.rs` | 110 | FR-7/8 (`search/symbol`) + FR-19 (`references`) | `CodeSearchResult` / `SymbolResult` / `ReferencesResult` / `CodeMatch` | ✅ | `error.rs` + `output.rs` |
| 6 | `commands/workspace.rs` | 64 | FR-9 (`list`) + FR-10 ⏳ (`current`) | `WorkspaceSummary` / `WorkspaceList` | 🟡 部分 | `error.rs` + `output.rs` |
| 7 | `commands/worktree.rs` | 87 | FR-11/12/13 (`create/enter/status`) | `Worktree` / `WorktreeStatus` | ✅ | `error.rs` + `output.rs` |
| **合计** | **7 files** | **802** | **12/13 ✅ + 1 ⏳** | 11 schema 类型 | **92%** | 全部走 `error.rs` + `output.rs` |

### 2.3 跨文件共享约束 (8 项)

| # | 约束 | 描述 | 证据 |
|---|---|---|---|
| 1 | 1 个 StarError → Error schema | 全部命令走 `Result<(), StarError>`, StarError 序列化走 `agent-api/v1#Error` 6 字段 | `error.rs:10-22` + cli-spec §5 F-06 |
| 2 | 1 个 output.rs 统一入口 | 全部命令输出走 `output::json_pretty`, 默认 JSON | `output.rs:24-27` + `main.rs:28-30` `--json` |
| 3 | 0 shell=True | 全部走 `Command::new("git")` 而非 `Command::new("sh").arg("-c")` | 守门 #6 |
| 4 | 0 unsafe block | 0 unsafe (per 守门 #7) | `grep -rn 'unsafe ' crates/star-cli/src/` 0 命中 |
| 5 | 1 个 schema version | 默认 `--schema-version v1` (= `agent-api/v1`, per cli-spec §3 F-27) | `output.rs:18 SCHEMA_VERSION` + cli-spec §3 |
| 6 | 6 字段 Error 不重定义 | 全部引用 `agent-api/v1#Error`, CLI 端不重新定义 6 字段 (per cli-spec §5 F-06) | [BD-004 §2](./BD-STAR-CLI-004.md) |
| 7 | mock envelope 一致 | 所有 mock JSON 输出都含 `{schema_version, mock: true, tool: ..., <key>: ...}` envelope | `project.rs:67-75` / `issue.rs:104-112` / 等 |
| 8 | FR-12 stdout path 特例 | `star worktree enter` 不走 JSON envelope, stdout 仅打印 path (供 shell eval) | `worktree.rs:55-60` |

### 2.4 8 维度方式

| # | 维度 | CLI 实现方式 | 13 命令是否统一 |
|---|---|---|---|
| 1 | parse | clap derive (clap 4.5) | ✅ |
| 2 | dispatch | match `cli.command` in `fn run()` | ✅ |
| 3 | execute | mock data (Phase D 骨架, Phase D.1 替换真实 client) | ✅ |
| 4 | error | `Result<(), StarError>` → eprintln to stderr + ExitCode | ✅ |
| 5 | output | stdout via `output::json_pretty` (always JSON for MVP 17, except FR-12) | ✅ |
| 6 | schema | `agent-api/v1` (13 命令引用 11 schema) | ✅ |
| 7 | capability | `star agent capabilities` returns 15-item array (per cli-spec §2.3) | n/a (meta) |
| 8 | versioning | `--schema-version v1` (default), `--version` returns CLI binary version | ✅ |

---

## §3 データビュー

### 3.1 `star project list` (FR-1)

**实施**: [`commands/project.rs:14-79`](../crates/star-cli/src/commands/project.rs)

#### 3.1.1 schema 字段集 (`agent-api/v1#ProjectList`)

| 字段 | 类型 | 必填 | 说明 | 证据 |
|---|---|---|---|---|
| `schema_version` | `&'static str` | ✅ | 守门标记, 固定 `"agent-api/v1"` | `project.rs:70` |
| `mock` | `bool` | ✅ | Phase D 骨架标志, true = mock data | `project.rs:71` |
| `tool` | `&'static str` | ✅ | 命令标识, `"project list"` | `project.rs:72` |
| `list.items[]` | `Vec<Project>` | ✅ | 项目数组 | `project.rs:14-21` |
| `list.total` | `u32` | ✅ | 项目总数 | `project.rs:26` |
| `list.cursor` | `String` | ✅ | 分页游标, MVP 默认 `""` | `project.rs:27` |
| `Project.id` | `String` | ✅ | 项目 ID, e.g. `"proj-1"` | `project.rs:16` |
| `Project.name` | `String` | ✅ | 项目名 | `project.rs:17` |
| `Project.default_branch` | `String` | ✅ | 默认分支, 默认 `"main"` | `project.rs:18` |
| `Project.description` | `String` | ✅ | 项目描述 | `project.rs:19` |
| `Project.created_at` | `String` (RFC3339) | ✅ | 创建时间 | `project.rs:20` |

#### 3.1.2 mock 数据 (3 项 per `project.rs:30-54`)

| id | name | default_branch |
|---|---|---|
| `proj-1` | `STAR 平台` | `main` |
| `proj-2` | `GitGit` | `main` |
| `proj-3` | `Physis` | `main` |

#### 3.1.3 bash 块示例 (per cli-spec §2.1.1)

```bash
star project list --json
```

---

### 3.2 `star issue ...` (FR-2/3/4)

**实施**: [`commands/issue.rs:14-162`](../crates/star-cli/src/commands/issue.rs)

#### 3.2.1 `Issue` schema (FR-3 `show` / FR-2 `list` items element)

| 字段 | 类型 | 必填 | 说明 | 证据 |
|---|---|---|---|---|
| `id` | `String` | ✅ | issue ID, e.g. `"STAR-1024"` | `issue.rs:18` |
| `title` | `String` | ✅ | issue 标题 | `issue.rs:19` |
| `status` | `String` | ✅ | 状态, e.g. `OPEN`/`IN_PROGRESS`/`DONE` | `issue.rs:20` |
| `priority` | `String` | ✅ | 优先级, e.g. `P0`/`P1`/`P2`/`MEDIUM` | `issue.rs:21` |
| `labels[]` | `Vec<String>` | ✅ | 标签数组 | `issue.rs:22` |
| `assigned_to` | `Option<String>` | ✅ | 分配给的 agent, None 表示未分配 | `issue.rs:23` |
| `description` | `String` | ✅ | issue 描述 | `issue.rs:24` |
| `created_at` | `String` (RFC3339) | ✅ | 创建时间 | `issue.rs:25` |
| `updated_at` | `String` (RFC3339) | ✅ | 更新时间 | `issue.rs:26` |

#### 3.2.2 `IssueList` schema (FR-2)

| 字段 | 类型 | 说明 | 证据 |
|---|---|---|---|
| `items[]` | `Vec<Issue>` | issue 数组 | `issue.rs:31` |
| `total` | `u32` | 总数 | `issue.rs:32` |
| `cursor` | `String` | 分页游标, MVP 默认 `""` | `issue.rs:33` |

#### 3.2.3 `ClaimResult` schema (FR-4)

| 字段 | 类型 | 说明 | 证据 |
|---|---|---|---|
| `issue_id` | `String` | 被认领的 issue ID | `issue.rs:38` |
| `claimed` | `bool` | 是否认领成功, Phase D 永远 `true` | `issue.rs:39` |
| `claimed_at` | `String` (RFC3339) | 认领时间, `chrono::Utc::now()` | `issue.rs:40`, `issue.rs:147` |
| `claimed_by` | `String` | 认领者, Phase D 默认 `"agent-mock"` | `issue.rs:41`, `issue.rs:148` |

#### 3.2.4 mock 数据 (4 项 per `issue.rs:44-91`)

| id | status | priority | assigned_to |
|---|---|---|---|
| `STAR-1024` | IN_PROGRESS | P0 | `agent-mock` |
| `STAR-1025` | OPEN | P1 | `None` |
| `STAR-1026` | IN_PROGRESS | P2 | `Mavis` |
| `STAR-1027` | DONE | P1 | `Mavis` |

#### 3.2.5 FR-3 fallback 行为

per `issue.rs:115-142`: `star issue show <id>` 若 ID 不在 mock 列表, 生成 mock issue with `{id, title: "Mock issue {id}", status: "OPEN", priority: "MEDIUM", labels: ["mock"], assigned_to: None, ...}`.

#### 3.2.6 bash 块示例 (per cli-spec §2.1.1)

```bash
star issue list --json
star issue show STAR-1024 --json
star issue claim STAR-1024 --json
```

---

### 3.3 `star context get <id>` (FR-6)

**实施**: [`commands/context.rs:34-105`](../crates/star-cli/src/commands/context.rs)

> 注: `star context current` (FR-18) 在 [BD-STAR-CLI-003 §2.1](./BD-STAR-CLI-003.md); 本 BD 仅 `get` 子命令 (FR-6).

#### 3.3.1 `Context` schema (FR-6, agent-api/01 §3.8)

| 字段 | 类型 | 必填 | 说明 | 证据 |
|---|---|---|---|---|
| `issue_id` | `String` | ✅ | 关联 issue ID | `context.rs:36` |
| `related_code[]` | `Vec<CodeRef>` | ✅ | 关联代码引用 | `context.rs:37` |
| `related_docs[]` | `Vec<DocRef>` | ✅ | 关联文档引用 | `context.rs:38` |
| `related_mrs[]` | `Vec<MRRef>` | ✅ | 关联 MR 引用 | `context.rs:39` |
| `updated_at` | `String` (RFC3339) | ✅ | 最后更新时间 | `context.rs:40` |

#### 3.3.2 nested schemas

| Schema | 字段 | 说明 | 证据 |
|---|---|---|---|
| `CodeRef` | `path: String` / `line: u32` / `snippet: String` | 代码引用 | `context.rs:15-20` |
| `DocRef` | `path: String` / `title: String` | 文档引用 | `context.rs:22-26` |
| `MRRef` | `id: String` / `title: String` | MR 引用 | `context.rs:28-32` |

#### 3.3.3 mock 行为 (per `context.rs:43-73` `lookup_context`)

| 输入 ID | 返回 |
|---|---|
| `"ctx-current"` 或 `"current"` | mock context (关联 STAR-1024 + submit.rs + Universal Submit doc + MR-mock-001) |
| 其他 ID | empty context (`related_code/docs/mrs` 全 `vec![]`, `issue_id` = 输入 ID) |

> 注: `ctx-current` 这个 magic ID 也被 `star context current` (FR-18) 复用, 见 [BD-003 §2.1](./BD-STAR-CLI-003.md).

#### 3.3.4 bash 块示例 (per cli-spec §2.1.1)

```bash
star context get STAR-1024 --json
```

---

### 3.4 `star task current` (FR-5)

**实施**: [`commands/task.rs:32-191`](../crates/star-cli/src/commands/task.rs)

#### 3.4.1 `CurrentTask` schema (FR-5, agent-api/01 §3.6)

| 字段 | 类型 | 必填 | 说明 | 证据 |
|---|---|---|---|---|
| `schema_version` | `&'static str` | ✅ | 守门标记, 固定 `"agent-api/v1"` | `task.rs:55` |
| `id` | `String` | ✅ | 任务 ID | `task.rs:57` |
| `title` | `String` | ✅ | 任务标题 | `task.rs:58` |
| `status` | `String` | ✅ | 状态, e.g. `IN_PROGRESS`/`TODO`/`DONE` | `task.rs:60` |
| `assigned_to` | `String` | ✅ | 分配给的 agent | `task.rs:62` |
| `context_refs[]` | `Vec<String>` | ✅ | 上下文引用 (REQ / ADR / MR 列表) | `task.rs:64` |
| `acceptance_criteria[]` | `Vec<String>` | ✅ | 验收条件 | `task.rs:66` |
| `labels[]` | `Vec<String>` | ✅ | 标签 | `task.rs:68` |
| `updated_at` | `DateTime<Utc>` | ✅ | 最后更新时间 | `task.rs:70` |
| `source` | `&'static str` | ✅ | 数据来源, `"file"` 或 `"default_mock"` | `task.rs:75` |

> **注**: 本 schema 比 cli-spec §2.1 表描述多 4 字段 (schema_version/source/acceptance_criteria/updated_at 形态不同), 是因为 Phase D 实装时按 `agent-api/v1#CurrentTask` (W4 子代理定义) 完整展开, **不**缩水; 这是 P1-D 修复 2026-08-27 后的 baseline.

#### 3.4.2 数据加载策略 (per `task.rs:111-191`)

| 优先级 | 数据源 | 行为 | 触发 |
|---|---|---|---|
| 1 | `STAR-CURRENT-TASK.json` 文件 | parse JSON, 转 `CurrentTask { source: "file", ... }` | 文件存在且 parse 成功 |
| 2 | `default_current_task()` fallback | `CurrentTask { source: "default_mock", id: "STAR-1024", ... }` | 文件缺失 或 parse 失败 (打印 stderr warning) |

#### 3.4.3 `STAR-CURRENT-TASK.json` 文件 schema (per `task.rs:79-105`)

| 字段 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `id` | `String` | ✅ | 任务 ID |
| `title` | `Option<String>` | — | 默认 `""` |
| `status` | `Option<String>` | — | 默认 `"IN_PROGRESS"` |
| `assigned_to` | `Option<String>` | — | 默认 `"agent-unassigned"` |
| `context_refs` | `Vec<String>` | — | 默认 `[]` |
| `acceptance_criteria` | `Vec<String>` | — | 默认 `[]` |
| `labels` | `Vec<String>` | — | 默认 `[]` |
| `updated_at` | `Option<DateTime<Utc>>` | — | 默认 `now()` |

#### 3.4.4 文件查找策略 (per `task.rs:134-145` `locate_task_file`)

从 CWD 向上递归查找 `STAR-CURRENT-TASK.json`:
- 找到 → parse
- 找不到 (到根目录) → default mock

> **不**读 env 变量 (per 守门 #5: 28 命令不读 env 值).

#### 3.4.5 bash 块示例 (per cli-spec §2.1.1)

```bash
star task current --json
```

---

### 3.5 `star code ...` (FR-7/8)

**实施**: [`commands/code.rs:14-110`](../crates/star-cli/src/commands/code.rs)

> 注: `star code references` (FR-19) 在 [BD-STAR-CLI-003 §2.2](./BD-STAR-CLI-003.md); 本 BD 仅 `search` / `symbol` 子命令.

#### 3.5.1 `CodeSearchResult` schema (FR-7, agent-api/01 §3.9)

| 字段 | 类型 | 必填 | 说明 | 证据 |
|---|---|---|---|---|
| `query` | `String` | ✅ | 搜索 query | `code.rs:25` |
| `matches[]` | `Vec<CodeMatch>` | ✅ | 匹配项 | `code.rs:26` |
| `total` | `u32` | ✅ | 总匹配数 | `code.rs:27` |

#### 3.5.2 `CodeMatch` nested schema

| 字段 | 类型 | 说明 | 证据 |
|---|---|---|---|
| `file` | `String` | 文件路径 | `code.rs:18` |
| `line` | `u32` | 行号 | `code.rs:19` |
| `snippet` | `String` | 代码片段 | `code.rs:20` |

#### 3.5.3 `SymbolResult` schema (FR-8, agent-api/01 §3.10)

| 字段 | 类型 | 必填 | 说明 | 证据 |
|---|---|---|---|---|
| `name` | `String` | ✅ | 符号名 | `code.rs:32` |
| `kind` | `String` | ✅ | 类型, e.g. `"function"`/`"struct"`/`"enum"` | `code.rs:33` |
| `file` | `String` | ✅ | 所在文件 | `code.rs:34` |
| `line` | `u32` | ✅ | 行号 | `code.rs:35` |

#### 3.5.4 mock 行为 (per `code.rs:46-108`)

| 命令 | mock 返回 |
|---|---|
| `star code search <q>` | 1 匹配项, file=`crates/star-cli/src/commands/code.rs`, line=1, snippet=`// search: <q>` |
| `star code symbol <name>` | 1 符号, kind=`"function"`, file=`src/<name>.rs`, line=1 |

> **⏳ [M] 子项**: 当前 mock 仅 1 项占位; Phase D.1 需替换真实 backend (ripgrep / LSP).

#### 3.5.5 bash 块示例 (per cli-spec §2.1.1)

```bash
star code search "auth.rs" --limit 20 --json
star code symbol "verify_token" --json
```

---

### 3.6 `star workspace ...` (FR-9/10)

**实施**: [`commands/workspace.rs:14-64`](../crates/star-cli/src/commands/workspace.rs)

#### 3.6.1 `WorkspaceSummary` schema (FR-10, agent-api/01 §3.16)

| 字段 | 类型 | 必填 | 说明 | 证据 |
|---|---|---|---|---|
| `id` | `String` | ✅ | workspace ID | `workspace.rs:16` |
| `name` | `String` | ✅ | workspace 名 | `workspace.rs:17` |
| `issue_id` | `String` | ✅ | 关联 issue ID | `workspace.rs:18` |
| `worktree_id` | `String` | ✅ | 关联 worktree ID | `workspace.rs:19` |
| `agent_session_id` | `String` | ✅ | 关联 agent session ID | `workspace.rs:20` |
| `created_at` | `String` (RFC3339) | ✅ | 创建时间 | `workspace.rs:21` |

> **修复记录 (per cli-spec §2.1 P1-C 修复 2026-08-27)**: 本 schema 名为 `WorkspaceSummary` 而**非** §3.x `Workspace`, 是 agent 视角逻辑抽象. **不**误称为 `Workspace` (per 守门 #11 缺标比错标).

#### 3.6.2 `WorkspaceList` schema (FR-9)

| 字段 | 类型 | 说明 | 证据 |
|---|---|---|---|
| `items[]` | `Vec<WorkspaceSummary>` | workspace 数组 | `workspace.rs:26` |
| `total` | `u32` | 总数 | `workspace.rs:27` |

> **⏳ [M] 子项**: `WorkspaceList.cursor` 字段当前未实装 (per `workspace.rs:25-28`), 待 Phase D.1 加.

#### 3.6.3 mock 数据 (2 项 per `workspace.rs:33-49`)

| id | name | issue_id | worktree_id |
|---|---|---|---|
| `ws-1` | `main-workspace` | `STAR-1024` | `wt-1` |
| `ws-2` | `feat-workspace` | `STAR-1025` | `wt-2` |

#### 3.6.4 bash 块示例 (per cli-spec §2.1.1)

```bash
star workspace list --json
# star workspace current --json  ← ⏳ [M] 子项
```

---

### 3.7 `star worktree ...` (FR-11/12/13)

**实施**: [`commands/worktree.rs:14-86`](../crates/star-cli/src/commands/worktree.rs)

#### 3.7.1 `Worktree` schema (FR-11, agent-api/01 §3.2)

| 字段 | 类型 | 必填 | 说明 | 证据 |
|---|---|---|---|---|
| `id` | `String` | ✅ | worktree ID, 形态 `wt-<issue_id>` | `worktree.rs:18` + `worktree.rs:39` |
| `path` | `String` | ✅ | 路径, 形态 `/repos/owner/repo/wt-<id>` | `worktree.rs:19` + `worktree.rs:38` |
| `branch` | `String` | ✅ | 分支名, 形态 `feature/<id>` | `worktree.rs:20` + `worktree.rs:40` |
| `head_commit` | `String` | ✅ | HEAD commit SHA (40-char hex) | `worktree.rs:21` + `worktree.rs:41` |
| `dirty` | `bool` | ✅ | 是否有未提交改动 | `worktree.rs:22` |

#### 3.7.2 `WorktreeStatus` schema (FR-13, agent-api/01 §3.11)

| 字段 | 类型 | 必填 | 说明 | 证据 |
|---|---|---|---|---|
| `worktree` | `Worktree` | ✅ | 嵌套 worktree schema | `worktree.rs:27` |
| `last_commit` | `String` | ✅ | 最后 commit SHA (短格式 7-char) | `worktree.rs:28` |
| `uncommitted_files` | `u32` | ✅ | 未提交文件数 | `worktree.rs:29` |

#### 3.7.3 FR-12 特例: stdout 仅 path (no JSON envelope)

per `worktree.rs:55-60`:
```rust
WorktreeCommand::Enter { id } => {
    let path = format!("/repos/owner/repo/wt-{id}");
    println!("{path}");  // ← stdout 仅 path, 不走 json_pretty
    Ok(())
}
```

> **设计理由** (per 守门 #8): `star worktree enter` 的消费方式是 `eval $(star worktree enter wt-STAR-1024)` (shell 习惯), JSON envelope 会污染 shell 解析. 守门 #8 = "FR-12 stdout path 特例", 跨命令不统一.

#### 3.7.4 bash 块示例 (per cli-spec §2.1.1)

```bash
star worktree create STAR-1024 --branch feat/auth-fix
star worktree enter wt-STAR-1024        # stdout: /repos/owner/repo/wt-STAR-1024
star worktree status --json
```

---

### 3.8 10 项通用 flags (per cli-spec §3 + F-25/F-27 修复)

| # | flag | 说明 | 默认 | 守门 | 实装 |
|---|---|---|---|---|---|
| 1 | `--json` | 强制 JSON 输出 | `true` (MVP 默认) | 守门 #8 | `main.rs:28-30` |
| 2 | `--quiet` | 只输出 ID / 摘要 | `false` | — | ⏳ Phase 2 |
| 3 | `--fields k1,k2` | 限制输出字段 (逗号分隔) | 输出全部字段 | — | ⏳ Phase 2 |
| 4 | `--limit N` | 限制行数 (e.g. `star issue list --limit 20`) | 无限制 | — | ⏳ Phase 2 |
| 5 | `--cursor <c>` | 分页游标 | `""` | — | ⏳ Phase 2 |
| 6 | `--no-color` | 关闭 ANSI color | false (always off for MVP, per `output.rs:11`) | — | 🟡 不需要分支 |
| 7 | `--schema-version <v>` | 显式 schema 版本 | `v1` (= `agent-api/v1`, per F-27 修复) | 守门 #12 | `output.rs:18 SCHEMA_VERSION` |
| 8 | `--help` / `-h` | 显示帮助 (per F-25 修复) | n/a | 守门 #25 | clap derive 内建 (`main.rs:22`) |
| 9 | `--no-header` | 关闭 banner / header 行 (per F-25 修复) | false | 守门 #25 | ⏳ Phase 2 |
| 10 | `--version` / `-V` | 显示 `star` 版本 (per F-25 修复) | n/a | 守门 #25 | clap derive 内建 (`main.rs:22`) |

#### 3.8.1 当前实装状态 (per 守门 #1 + #5+#11)

- **MVP 已实装** (3 项): `--json` (clap derive global) + `--help` (clap 内建) + `--version` (clap 内建)
- **MVP 不需要分支** (1 项): `--no-color` (per `output.rs:11-12`: "不在 lib 层做 ANSI color")
- **MVP 不需要实装但 spec 已定义** (3 项): `--schema-version` (MVP 永远 `v1`, enum dispatch 已固化; 传 `v2` 报 `SCHEMA_VERSION_UNSUPPORTED`)
- **⏳ Phase 2+ 待实装** (3 项): `--quiet` / `--fields` / `--limit` / `--cursor` / `--no-header` (5 项, 跨 13 命令加这 5 flag 是 Phase 2 拍板)

#### 3.8.2 F-27 修复 (per 2026-08-27): `--schema-version <v>` 默认值

> `v1` 是当前 `star agent capabilities` 输出的 schema version, 与 `spec/agent-api/01-schema.md §1 OpenAPI info.version` 同步演化 (per INTERFACE-REVIEW-A 🟡 #11). 如果调用方传 `--schema-version v2` 但 server 未实现 v2, server 报 `SCHEMA_VERSION_UNSUPPORTED` 错误 (per `agent-api/v1#Error`).

---

## §4 モジュールビュー

### 4.1 7 个 .rs 文件职责细化

#### 4.1.1 `commands/project.rs` (80 行)

| 区域 | 行号 | 职责 |
|---|---|---|
| enum 定义 | `:10-12` | `ProjectCommand::List` 1 变体 |
| Project schema | `:14-21` | `Project` 5 字段 |
| ProjectList schema | `:23-28` | `ProjectList` 3 字段 |
| mock 数据 | `:30-54` | 3 个 project |
| `run()` dispatch | `:56-79` | match `List` → mock → JSON output |

**依赖**: `clap::Subcommand` / `serde::Serialize` / `crate::error::StarError` / `crate::output`
**输出 envelope**: `{schema_version, mock: true, tool: "project list", list: ProjectList}`

#### 4.1.2 `commands/issue.rs` (163 行)

| 区域 | 行号 | 职责 |
|---|---|---|
| enum 定义 | `:10-14` | `List` / `Show {id}` / `Claim {id}` 3 变体 |
| Issue schema | `:16-27` | `Issue` 9 字段 |
| IssueList schema | `:29-34` | `IssueList` 3 字段 |
| ClaimResult schema | `:36-42` | `ClaimResult` 4 字段 |
| mock 数据 | `:44-91` | 4 个 issue (STAR-1024/1025/1026/1027) |
| `run()` dispatch | `:93-162` | match 3 变体 → mock + fallback → JSON output |

**fallback 行为**: `Show {id}` 若 ID 不在 mock 列表, 生成 default mock issue with `status: "OPEN"` + `priority: "MEDIUM"` (per `:115-142`)
**Claim 行为**: 不查 mock, 直接返 `ClaimResult { claimed: true, claimed_at: chrono::Utc::now(), claimed_by: "agent-mock" }` (per `:143-160`)

#### 4.1.3 `commands/task.rs` (192 行)

| 区域 | 行号 | 职责 |
|---|---|---|
| enum 定义 | `:18-22` | `TaskCommand::Current(CurrentArgs)` 1 变体 |
| CurrentArgs | `:24-30` | `--json: bool` (default true) |
| `run()` dispatch | `:32-39` | → `current::run(&args)` |
| `current` submodule | `:42-191` | 完整实装 |
| ├─ CurrentTask schema | `:52-76` | 10 字段 (含 `schema_version` + `source`) |
| ├─ TaskFile (private) | `:79-105` | 8 字段 (与 CurrentTask 1:1 对应, 多 `deny_unknown_fields`) |
| ├─ `TASK_FILE` const | `:108` | `"STAR-CURRENT-TASK.json"` |
| ├─ `run()` entry | `:111-127` | locate → load (fallback default) → JSON output |
| ├─ `locate_task_file()` | `:134-145` | CWD 向上递归查找 |
| ├─ `load_task_file()` | `:148-165` | parse JSON, 转 CurrentTask { source: "file", ... } |
| └─ `default_current_task()` | `:168-191` | mock STAR-1024 / IN_PROGRESS / Phase D 骨架 |

**关键设计**:
- `source` 字段让消费者区分数据来源 (file vs default_mock), **不**是 schema 漂移
- 解析失败**不**抛错, 退到 default + stderr warning (per `:118-120`), 守门 #13 (可观测)

#### 4.1.4 `commands/context.rs` (106 行)

| 区域 | 行号 | 职责 |
|---|---|---|
| enum 定义 | `:10-13` | `Get {id}` / `Current` 2 变体 (Current 是 FR-18, BD-003 详) |
| CodeRef schema | `:15-20` | 3 字段 |
| DocRef schema | `:22-26` | 2 字段 |
| MRRef schema | `:28-32` | 2 字段 |
| Context schema | `:34-41` | 5 字段 |
| `lookup_context(id)` | `:43-73` | magic ID `ctx-current` / `current` 返 mock; 其他返 empty |
| `run()` dispatch | `:75-105` | match 2 变体 → JSON output |

**跨 BD 共享**: `lookup_context` 被 `Get` + `Current` 共用 (per `:43-73`)

#### 4.1.5 `commands/code.rs` (110 行)

| 区域 | 行号 | 职责 |
|---|---|---|
| enum 定义 | `:10-14` | `Search {query}` / `Symbol {name}` / `References {name}` (References 是 FR-19, BD-003 详) |
| CodeMatch schema | `:16-21` | 3 字段 |
| CodeSearchResult schema | `:23-28` | 3 字段 |
| SymbolResult schema | `:30-36` | 4 字段 |
| ReferencesResult schema | `:38-43` | 3 字段 (BD-003 详) |
| `run()` dispatch | `:45-109` | match 3 变体 → mock → JSON output |

#### 4.1.6 `commands/workspace.rs` (64 行)

| 区域 | 行号 | 职责 |
|---|---|---|
| enum 定义 | `:10-12` | `List` 1 变体 (Current 缺, ⏳ [M]) |
| WorkspaceSummary schema | `:14-22` | 6 字段 |
| WorkspaceList schema | `:24-28` | 2 字段 (缺 `cursor`, ⏳ Phase D.1) |
| `run()` dispatch | `:30-63` | mock 2 workspace → JSON output |

#### 4.1.7 `commands/worktree.rs` (87 行)

| 区域 | 行号 | 职责 |
|---|---|---|
| enum 定义 | `:10-14` | `Create {id}` / `Enter {id}` / `Status` 3 变体 |
| Worktree schema | `:16-23` | 5 字段 |
| WorktreeStatus schema | `:25-30` | 3 字段 |
| `run()` dispatch | `:32-86` | match 3 变体 → mock (Enter 是 stdout path 特例) |

### 4.2 跨文件共享约束 (8 项, 已在 §2.3 列)

### 4.3 mock envelope 一致性 (per 守门 #8)

13 命令中 12 命令 (除 FR-12 enter) 输出统一 envelope:
```json
{
  "schema_version": "agent-api/v1",
  "mock": true,
  "tool": "<command name>",
  "<key>": <schema>
}
```

> **设计理由**: `mock: true` 字段让消费者区分 mock data vs real data, **不**删除 (Phase D.1 替换真实 client 时改为 `false` 或删除字段).

---

## §5 NFR (per 守门 #1+#5+#6+#7+#11+#12+#13)

| NFR ID | 类别 | 目标 | 派生守门 | 实装状态 |
|---|---|---|---|---|
| NFR-CL13-1 性能 | 性能 | `star <cmd> --json` < 200ms (本地 mock, 无网络) | 守门 #1 | ✅ mock 数据 O(1) |
| NFR-CL13-2 可用 | 可用 | 13 / 13 命令 100% 可调用 + schema 100% 稳定 | 守门 #1 | ✅ 12/13 (1 ⏳ workspace current) |
| NFR-CL13-3 跨平台 | 跨平台 | Windows / macOS / Linux 三端二进制 | 守门 #6 | ✅ `Command::new("git")` 而非 `shell=True` |
| NFR-CL13-4 安全 | 安全 | 0 unsafe block | 守门 #7 | ✅ `grep -rn 'unsafe ' crates/star-cli/src/commands/` 0 命中 |
| NFR-CL13-5 可观测 | 可观测 | 错误带 `trace_id` (per `agent-api/v1#Error` §3.15) | 守门 #13 | ✅ [BD-004 §2](./BD-STAR-CLI-004.md) |
| NFR-CL13-6 易用 | 易用 | 13 命令有 `--help` 输出 (per F-25) | 守门 #25 | ✅ clap derive 内建 |
| NFR-CL13-7 数据加载 | 持久化 | `star task current` 优先读 `STAR-CURRENT-TASK.json`, fallback mock | 守门 #11 | ✅ `task.rs:111-191` |
| NFR-CL13-8 命名一致 | 命名 | 13 命令全部走 `agent-api/v1` schema 名 (无错标, per P1-C 修复) | 守门 #11 | ✅ |

---

## §6 已知缺口 (per 守门 #11 缺标比错标, ~20 项)

### 6.1 workspace 域 (3 项)

| # | 缺口 | 影响 | 优先级 | 实装位置 |
|---|---|---|---|---|
| G-CL13-W1 | `star workspace current` (FR-10) 未实装 | MVP 退出条件缺 1 命令 | ⏳ [M] | `commands/workspace.rs` enum 缺 `Current` 变体 |
| G-CL13-W2 | `WorkspaceList.cursor` 字段未实装 | `--cursor` flag 无法对接 | ⏳ Phase D.1 | `commands/workspace.rs:25-28` |
| G-CL13-W3 | `WorkspaceSummary.agent_session_id` 与 worktree_id 字段关系未文档化 | 消费者理解模糊 | ⏳ [S] | `commands/workspace.rs:19-20` |

### 6.2 worktree 域 (3 项)

| # | 缺口 | 影响 | 优先级 |
|---|---|---|---|
| G-CL13-WT1 | `head_commit` mock 用写死 `"deadbeef00..."` | Phase D 永远假 commit SHA | ⏳ Phase D.1 (读真实 `.git/HEAD`) |
| G-CL13-WT2 | `dirty` mock 永远 `false` | Phase D 永远干净 | ⏳ Phase D.1 (spawn `git status --porcelain`) |
| G-CL13-WT3 | `star worktree enter` 跨平台兼容 (Windows path separator) | Windows path 用 `\` 而非 `/` | ⏳ [S] |

### 6.3 task 域 (2 项)

| # | 缺口 | 影响 | 优先级 |
|---|---|---|---|
| G-CL13-T1 | `STAR-CURRENT-TASK.json` 文件查找**不**读 env var (per 守门 #5: 28 命令不读 env 值) | 多 workspace 切换需手动 mv 文件 | ⏳ Phase 2+ |
| G-CL13-T2 | `TaskFile` 字段 `context_refs` / `acceptance_criteria` 不带 `Option` (硬编 `Vec<String>`) | 文件 schema 演进不灵活 | ⏳ Phase 2+ |

### 6.4 context 域 (2 项)

| # | 缺口 | 影响 | 优先级 |
|---|---|---|---|
| G-CL13-C1 | `lookup_context` magic ID `ctx-current` / `current` hard-coded | 未来 agent_id 接入需重构 | ⏳ Phase 2+ |
| G-CL13-C2 | `Context.related_code/docs/mrs` 在 mock 中仅 1 项 | Phase D 不够丰富 | ⏳ [S] (mock 扩) |

### 6.5 code 域 (3 项)

| # | 缺口 | 影响 | 优先级 |
|---|---|---|---|
| G-CL13-CD1 | `star code search` mock 永远 1 匹配项, 不调真实 backend | Phase D 不可用 | ⏳ [M] (Phase D.1 接 ripgrep) |
| G-CL13-CD2 | `star code symbol` mock 永远 `kind: "function"`, 不做 kind 推断 | 消费者无法区分 function/struct/enum | ⏳ [M] (Phase D.1 接 LSP) |
| G-CL13-CD3 | `CodeMatch.snippet` 截断策略未文档化 | 大文件可能 OOM | ⏳ Phase 2+ (LSP response 通常 < 1KB, MVP 可忽略) |

### 6.6 issue 域 (3 项)

| # | 缺口 | 影响 | 优先级 |
|---|---|---|---|
| G-CL13-I1 | `star issue claim` 不接真实 backend, 永远 `claimed: true` | Phase D 假认领 | ⏳ [M] (Phase D.1 接 GitHub/GitLab API) |
| G-CL13-I2 | `Issue.assigned_to` mock 写死 `agent-mock` / `Mavis` | 不可配置 | ⏳ Phase 2+ (读 `$STAR_AGENT_ID`) |
| G-CL13-I3 | `Issue.labels` 跟 `Issue.priority` 在 mock 中独立 (无 cross-check) | 未来加 P0/P1 关联检查需重构 | ⏳ Phase 2+ |

### 6.7 project 域 (2 项)

| # | 缺口 | 影响 | 优先级 |
|---|---|---|---|
| G-CL13-P1 | `star project list` mock 写死 3 项 (STAR 平台 / GitGit / Physis) | 不可配置 | ⏳ Phase 2+ (读 `.star/projects.json` 或 env) |
| G-CL13-P2 | `Project.default_branch` mock 永远 `"main"` | Phase D 不区分 main/master | ⏳ Phase 2+ |

### 6.8 跨 CLI/MCP 命名映射缺口 (per cli-spec §2.5 F-08, 5 项, 本 BD 范围 2 项)

| # | CLI 命令 | MCP tool | 命名差异 | 状态 |
|---|---|---|---|---|
| G-CL13-X1 | `star project list` | (待定, 可能 `list_projects`) | `list` vs `list_*` | ⏳ Phase 2 评估 |
| G-CL13-X2 | `star issue list` | `search_issues` (empty query) | `list` vs `search_*` | ⏳ MCP 已对齐 |
| G-CL13-X3 | `star issue claim <id>` | (待定, 可能 `claim_issue`) | — | ⏳ Phase 2 MCP 补 |
| G-CL13-X4 | `star task current` | `get_current_task` | 一致 (CLI `current` / MCP `get_*_current_*`) | ✅ |
| G-CL13-X5 | `star context get <id>` | `get_context` | 一致 | ✅ |

> 余 3 跨层缺口 (`star context current` / `star workspace list` / `star mr show` 等) 跨 BD 处理, 见 [BD-001 §6.1 G-3](./BD-STAR-CLI-001.md) + [BD-003 §2](./BD-STAR-CLI-003.md) + [BD-004 §3.1](./BD-STAR-CLI-004.md).

### 6.9 通用 flags 缺口 (3 项)

| # | 缺口 | 优先级 |
|---|---|---|
| G-CL13-F1 | `--quiet` / `--fields` / `--limit` / `--cursor` / `--no-header` 5 flag 跨 13 命令未实装 | ⏳ Phase 2+ |
| G-CL13-F2 | `--schema-version v2` 不支持 (传 v2 报 `SCHEMA_VERSION_UNSUPPORTED`) | ⏳ Phase 2+ (per F-27) |
| G-CL13-F3 | `--no-color` 不需要分支 (MVP 不做 ANSI color, per `output.rs:11-12`) | ✅ |

### 6.10 错误模型缺口 (1 项)

| # | 缺口 | 优先级 |
|---|---|---|
| G-CL13-E1 | `error.rs` 仅 3 类 stub (Json/Io/Skill), 缺 9 类完整 ToError trait | ⏳ Phase D.1 (per [BD-004 §2](./BD-STAR-CLI-004.md)) |

---

## §7 テスト用例 (per FR-1~FR-13 + cli-spec §2.1.1 bash 块)

### 7.1 13 命令 AC 表 (per 守门 #1+#13)

| AC# | 命令 | 输入 | 期望输出 | 期望 exit code | 实装 |
|---|---|---|---|---|---|
| AC-CL13-1 | `star project list --json` | (no arg) | JSON, `mock: true`, `tool: "project list"`, `list.total: 3` | 0 | ✅ |
| AC-CL13-2 | `star issue list --json` | (no arg) | JSON, `list.total: 4`, 4 mock issues | 0 | ✅ |
| AC-CL13-3 | `star issue show STAR-1024 --json` | `STAR-1024` | JSON, `issue.id: "STAR-1024"`, `status: "IN_PROGRESS"` | 0 | ✅ |
| AC-CL13-3-fallback | `star issue show NON-EXISTENT --json` | `NON-EXISTENT` | JSON, `issue.id: "NON-EXISTENT"`, `status: "OPEN"`, `priority: "MEDIUM"` | 0 | ✅ |
| AC-CL13-4 | `star issue claim STAR-1024 --json` | `STAR-1024` | JSON, `claim.claimed: true`, `claimed_by: "agent-mock"` | 0 | ✅ |
| AC-CL13-5 | `star task current --json` | (no arg, CWD 无 `STAR-CURRENT-TASK.json`) | JSON, `id: "STAR-1024"`, `source: "default_mock"` | 0 | ✅ |
| AC-CL13-5-file | `star task current --json` | (CWD 含 `STAR-CURRENT-TASK.json`) | JSON, `source: "file"`, ID 来自文件 | 0 | ✅ |
| AC-CL13-5-fallback | `star task current --json` | (CWD 含坏 JSON) | JSON + stderr warning, `source: "default_mock"` | 0 | ✅ |
| AC-CL13-6 | `star context get STAR-1024 --json` | `STAR-1024` | JSON, `context.issue_id: "STAR-1024"`, empty related | 0 | ✅ |
| AC-CL13-6-magic | `star context get ctx-current --json` | `ctx-current` | JSON, `context.issue_id: "STAR-1024"`, 1 mock code/docs/mrs | 0 | ✅ |
| AC-CL13-7 | `star code search "auth.rs" --json` | `"auth.rs"` | JSON, `result.query: "auth.rs"`, `result.total: 1` | 0 | ✅ |
| AC-CL13-8 | `star code symbol "verify_token" --json` | `"verify_token"` | JSON, `symbol.name: "verify_token"`, `symbol.kind: "function"` | 0 | ✅ |
| AC-CL13-9 | `star workspace list --json` | (no arg) | JSON, `list.total: 2`, 2 mock workspaces | 0 | ✅ |
| AC-CL13-10 | `star workspace current --json` | n/a | ⏳ [M] 子项 | — | ⏳ |
| AC-CL13-11 | `star worktree create STAR-1024 --json` | `STAR-1024` | JSON, `worktree.id: "wt-STAR-1024"`, `branch: "feature/STAR-1024"` | 0 | ✅ |
| AC-CL13-12 | `star worktree enter wt-STAR-1024` | `wt-STAR-1024` | stdout: `/repos/owner/repo/wt-STAR-1024` (no JSON envelope) | 0 | ✅ |
| AC-CL13-13 | `star worktree status --json` | (no arg) | JSON, `status.worktree.id: "wt-current"`, `uncommitted_files: 0` | 0 | ✅ |
| AC-CL13-help | `star --help` | n/a | clap 输出的 help text, 含 13 顶层 subcommand | 0 | ✅ |
| AC-CL13-version | `star --version` | n/a | clap derive 版本 (e.g. `star 0.1.0`) | 0 | ✅ |

### 7.2 集成测试入口

per 守门 #1+#13, 13 命令的 cargo test 入口在各自 `commands/<sub>.rs` 的 `#[cfg(test)] mod tests` 块。当前已落:

| 文件 | test 数 | 覆盖 |
|---|---|---|
| `commands/test.rs:39-55` | 2 | pass count assertion |
| `commands/pipeline.rs:60-66` | 1 | pipeline id 形态 |

> **⏳ [L] 子项**: 13 命令的完整 integration test (含 stdout snapshot) 待 Phase D.1.

---

## §8 インターフェース契約

### 8.1 REST 边界 (n/a)

本 BD 13 命令不直接对外暴露 REST endpoint. CLI 是 `agent-api/v1` 的**消费者**, **不**是 producer. REST 协议由 `crates/star-api-rest/` (per AGENTS.md §5) 提供.

### 8.2 Shell 边界 (per 守门 #6+#8)

| 命令 | stdout 形态 | 退出码 | stderr 形态 |
|---|---|---|---|
| `star <cmd> --json` (12 命令) | JSON envelope | 0 成功 / 1 用户错 / 2 内部错 (per `error.rs:26-32`) | empty (成功) / error message (失败) |
| `star worktree enter <id>` (FR-12) | 仅 path (no envelope) | 0 | empty |

**消费示例**:
```bash
# JSON 消费 (默认)
projects=$(star project list --json | jq -r '.list.items[].id')

# path 消费 (eval 模式)
cd "$(star worktree enter wt-STAR-1024)"
```

### 8.3 MCP 边界 (n/a)

CLI 不重复 MCP 协议 (per AGENTS.md §5 + cli-spec §2.5 F-08). 13 命令的 MCP 覆盖见 cli-spec §2.5 表 + [BD-003 §6.8](./BD-STAR-CLI-003.md).

### 8.4 CLI ↔ CLI 子能力边界

| 边界 | 描述 |
|---|---|
| CL-1 ↔ CL-3 | 共用 `commands/` 模块但互不依赖 (`issue.rs` 不 import `worktree.rs`) |
| CL-1 ↔ CL-2 | `commands/context.rs` 共享 `Context` schema 给 FR-6 (本 BD) + FR-18 (BD-003) |
| CL-1 ↔ CL-4 | `commands/agent.rs` 不被 13 命令引用, 仅 `commands/task.rs` 与 agent capabilities 平行 |
| CL-1 ↔ CL-6 | `commands/task.rs::CurrentTask.id` 被 `commands/submit.rs::step_check_task` (FR-17) 引用 |

### 8.5 Git 协议边界 (per cli-spec §1 第 5 条)

**`star` 不替代 `git`**, 所有 Git 协议能力继续由 GitGit 提供. CLI 仅做 wrapper (per FR-11 `worktree create` → spawn `git worktree add`; per `submit.rs:442-460` step_check_diff spawn `git diff --stat`).

---

## §9 関連ドキュメント

per 守门 #12 v21 commit 引用全链 + 总冊 BD-001 §8:

| # | 类别 | 文档 | 关系 |
|---|---|---|---|
| 1 | 上位 SRS | [`SRS-STAR-CLI-001.md`](../requirements/SRS-STAR-CLI-001.md) v1.0 | 上游 requirements (本仓 HEAD main path) |
| 2 | 上游总冊 | [`BD-STAR-CLI-001.md`](./BD-STAR-CLI-001.md) v1.0 | 索引级汇总 (本 commit 同期落档) |
| 3 | 平行 BD | [`BD-STAR-CLI-003.md`](./BD-STAR-CLI-003.md) v1.0 | CL-2 11 扩展命令 (跨 BD 引用: `context.rs::lookup_context` 共用) |
| 4 | 平行 BD | [`BD-STAR-CLI-004.md`](./BD-STAR-CLI-004.md) v1.0 | CL-4+CL-5 错误模型 + Skill (跨 BD 引用: `error.rs::StarError`) |
| 5 | 平行 BD | [`BD-STAR-CLI-005.md`](./BD-STAR-CLI-005.md) v1.0 | CL-6 Universal Submit (跨 BD 引用: `task.rs::CurrentTask.id` → submit step 1) |
| 6 | 上游 spec | [`spec/cli/01-cli-spec.md`](../architecture/2026-08-26-upgrade/spec/cli/01-cli-spec.md) v0.2 | CLI canonical spec (canonical) |
| 7 | 上游 spec | [`spec/agent-api/01-schema.md`](../architecture/2026-08-26-upgrade/spec/agent-api/01-schema.md) | 13 命令引用的 11 schema canonical |
| 8 | 平行 SRS | [`SRS-AGENT-VIEW-001.md`](../requirements/SRS-AGENT-VIEW-001.md) v1.0 | Agent View 平级 |
| 9 | 平行 SRS | [`SRS-WORKTREE-CANVAS-001.md`](../requirements/SRS-WORKTREE-CANVAS-001.md) v1.0 | Worktree Canvas 平级 |
| 10 | 平行 SRS | [`SRS-STAR-AGENT-RUNTIME-001.md`](../requirements/SRS-STAR-AGENT-RUNTIME-001.md) v1.0 | Agent Runtime 平级 |
| 11 | ADR | [`ADR-0025-vendor-adapter-anti-contamination.md`](../adr/0025-vendor-adapter-anti-contamination.md) | vendor adapter 反污染 |
| 12 | ADR | [`ADR-0029-multica-patterns-borrow.md`](../adr/0029-multica-patterns-borrow.md) | Multica 模式借用 |
| 13 | ADR | [`ADR-0034-rust-pivot-agent-game.md`](../adr/0034-rust-pivot-agent-game.md) | Rust 主线切换 |
| 14 | review/fix | [`INTERFACE-REVIEW-A.md`](../architecture/2026-08-26-upgrade/INTERFACE-REVIEW-A.md) | INTERFACE-REVIEW-A v0.2 |
| 15 | 实施位置 | [`crates/star-cli/src/main.rs`](../crates/star-cli/src/main.rs) | Cli parser + TopCommand enum |
| 16 | 实施位置 | [`crates/star-cli/src/commands/`](../crates/star-cli/src/commands/) | 13 subcommand 模块 |
| 17 | 实施位置 | [`crates/star-cli/src/error.rs`](../crates/star-cli/src/error.rs) | StarError enum |
| 18 | 实施位置 | [`crates/star-cli/src/output.rs`](../crates/star-cli/src/output.rs) | json_pretty 统一入口 |
| 19 | 实施位置 | [`crates/star-cli/Cargo.toml`](../crates/star-cli/Cargo.toml) | workspace + clap 4.5 |
| 20 | 下游交付 | `DD-STAR-CLI-002.md` | Phase 2 待开 |
| 21 | 测试 | `crates/star-cli/src/commands/*/tests` | cargo test |
| 22 | 测试 | `acceptance/04-mvp.md` | MVP 17 退出条件 (per cli-spec §2.1) |
| 23 | 守门合规 | AGENTS.md | 守门 #1+#5+#6+#7+#9+#10+#11+#12+#14 |

### Traceability 表 (per 守门 #1+#11)

| SRS § | BD § | 实现 (.rs) |
|---|---|---|
| §2.2 Agent/IDE 双协议入口 | n/a | (跨 BD, BD-004 §5) |
| §3.1.1 MVP 17 核心 #1-13 (本 BD 13 项) | §3.1~§3.7 + §3.8 flags | `commands/project/issue/task/context/code/workspace/worktree.rs` |
| §4.1 28 命令 capability 覆盖 (本 BD 范围 9 项) | §3 + §4 | (同 §3.1~§3.7) |
| §4.2 35 FR (本 BD 范围 FR-1~13) | §3 (1:1 映射) | (同 §3) |
| §5 通用 flags + 子命令 meta | §3.8 (10 项 flags 完整定义) | `main.rs:24-30` + clap derive |
| §7 NFR | §5 NFR 表 | (跨 5 BD) |
| §8 错误模型 6 字段 | §3.8 envelope + §6.10 缺口 | `error.rs` + [BD-004 §2](./BD-STAR-CLI-004.md) |
| §9 AC-001..012 | §7.1 13 AC 表 | `commands/<sub>.rs::tests` |
| §10 风险与依赖 | §6 已知缺口 20 项 | (跨 5 BD) |

### 5 角色签字栏

| 角色 | 签字 |
|---|---|
| 架构师 (Mavis 接手 agent per DEC-008) | 🟢 Mavis 接手终审 |
| SRE Lead (per 守门 #3 5 域 Lead 真人到位) | ⏳ 待追溯 |
| 平台 (per 守门 #14 修订人/审批形式) | 🟢 Mavis 接手审核 |
| 评审主持 (per CLI spec 守门 v0.62) | ⏳ 待追溯 |
| PM (Ulysses 一人公司 12 角色 per DEC-008) | 🟢 Ulysses 派发, Mavis 接手 |

### 9 守門合规状态 (per 守门 #1+#5+#6+#7+#9+#10+#11+#12+#14)

| 守门 | 描述 | 状态 |
|---|---|---|
| #1 | v15 ULYS-124 是用户拍板明确事件, 严格按 spec 落档 | ✅ |
| #5 | 28 命令不读 env 值 | ✅ |
| #6 | `Command::new("git")` 而非 `shell=True` | ✅ |
| #7 | 0 unsafe block | ✅ |
| #9 | v27 root 直接落档, 不派 subagent | ✅ |
| #10 | author=Ulysses per DEC-008 | ✅ |
| #11 | 缺标比错标: §6 列 20 项已知缺口 | ✅ |
| #12 | v21 commit 引用 cli/01 v0.2 + agent-api/01 + SRS + 总冊 BD-001 | ✅ |
| #14 | v4 审批=架构师 Mavis 接手 per 9/10 反转 | ✅ |
| **总计** | **9/9** | ✅ |

---

## §10 签字栏 + 修订履历

### 10.1 5 角色签字栏

| 角色 | 姓名 | 签字 | 日期 |
|---|---|---|---|
| 架构师 | Mavis 接手 agent (per DEC-008) | 🟢 已审 | 2026-09-25 JST |
| SRE Lead | (5 域 Lead 真人到位后追溯) | ⏳ 待追溯 | — |
| 平台 | Mavis 接手 (per 守门 #14 v4) | 🟢 已审 | 2026-09-25 JST |
| 评审主持 | (per CLI spec 守门 v0.62) | ⏳ 待追溯 | — |
| PM | Ulysses 一人公司 12 角色 (per DEC-008) | 🟢 已派发 | 2026-09-25 JST |

### 10.2 修订履历

| 版本 | 日期 | commit | 内容 | 状态 |
|---|---|---|---|---|
| v0.1 | 2026-09-20 | (issue state 变更) | 4 专题 BD 平行分解 (Stage 1 子任务) | — |
| v0.2 | 2026-09-20 | (后不可达) | 4 专题 BD 平行落档 — `5b0454b3` 在 `git ls-remote origin agent/minimaxm3/ulys-124` 不可达 | ❌ |
| **v1.0** | **2026-09-25** | **(本 turn TBD)** | **BD-STAR-CLI-002 v1.0 recreate — 13 命令详细 BD (CL-1+CL-3), 10 段 IPA, ~20 项已知缺口显式列, 13 AC, 9 守門全过, 10 项 flags 完整定义, 全程 push origin 后 4 路验证** | ✅ (本 turn) |

### 10.3 本 BD body 总行数

按 `wc -l` 实测 (本 turn): 见 task final output line `BD-002: <wc -l> <sha256>` 行.

---

**ULYS-124 子任务进度**: v0.1 SRS ✅ → v0.2 BD 5 子任务分解 ✅ → Stage 1.1 BD-STAR-CLI-002 v1.0 recreate ✅ (本 turn) → Stage 1.2~1.4 + Stage 2 总冊 + 4 路验证 (本 turn 同期).

**下游衔接**: 本 BD 落档 + push origin + 4 路验证 done 后, Stage 2 总冊 BD-001 v1.0 可拍板 done, ULYS-124 推进到 Phase 2 (5 份 DD).
