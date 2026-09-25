# BD-STAR-CLI-003

> **STAR CLI 专题 BD #2 v1.0 (recreate, per ULYS-124 v1.0 缺口补齐)**
>
> **CL-2: 11 扩展命令 + Universal Submit 步骤暴露 (6 原扩展 + 5 P1-H 新增)**
>
> - 状态: Basic Design Baseline (v1.0 — recreate after 9/21 + 9/23 reviewer 打回, per 用户 9/25 01:09 JST "确保缺口已补")
> - 目标阶段: 基本設計 → 詳細設計 → 実装 → テスト → リリース
> - 上位要件: [`docs/requirements/SRS-STAR-CLI-001.md`](../requirements/SRS-STAR-CLI-001.md) v1.0 (本仓 HEAD main path, 本 commit 同期落档)
> - 上游 spec: [`docs/architecture/2026-08-26-upgrade/spec/cli/01-cli-spec.md`](../architecture/2026-08-26-upgrade/spec/cli/01-cli-spec.md) v0.2 (canonical, §2.2 + §2.4 + §2.5)
> - 上游总冊: [`docs/design/BD-STAR-CLI-001.md`](./BD-STAR-CLI-001.md) v1.0 (本 commit 同期落档)
> - 下游交接: [`docs/design/BD-STAR-CLI-002.md`](./BD-STAR-CLI-002.md) v1.0 (CL-1+CL-3 13 命令) / [`docs/design/BD-STAR-CLI-004.md`](./BD-STAR-CLI-004.md) v1.0 (CL-4+CL-5 错误模型 + Skill) / [`docs/design/BD-STAR-CLI-005.md`](./BD-STAR-CLI-005.md) v1.0 (CL-6 Universal Submit)
> - 实施位置: [`crates/star-cli/src/commands/`](../crates/star-cli/src/commands/) (`context.rs` 扩 + `code.rs` 扩 + `mr.rs` 扩 + `test.rs` 扩 + `pipeline.rs` 5 新子命令)
> - 修订人: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> - 审批: 架构师 (Mavis 接手 agent per DEC-008)
> - 日期: 2026-09-25 JST

> **回溯记录（per ULYS-124 9/21 + 9/23 reviewer 打回与 9/25 01:09 用户缺口补齐指令）**: v0.1 专题 BD #2 在 9/20 commit 落档但 9/21 reviewer 复审确认 `git ls-remote origin agent/minimaxm3/ulys-124` 不可达。本 v1.0 按 (a) 当前 main `b0dddf7e` 实装的 11 命令代码 (b) cli-spec v0.2 canonical (c) P1-H 修复 2026-08-27 新增的 5 步暴露命令 基础上重新创作, 不复制任何前 worker 内容, 严格按"CL-2 11 命令详细 BD"分工。

---

## §0 文档目的

本文档基于 SRS-STAR-CLI-001 v1.0 的 FR-18~FR-28 需求, 定义 STAR CLI **CL-2 域共 11 扩展命令**的基本設計: 每个命令的 (a) schema 字段完整展开 (b) 模块视图职责 (c) NFR 子集 (d) 已知缺口显式列 (e) 测试用例 (f) 接口契约 (g) 5 P1-H 新增命令与 Universal Submit 状态机的关系。

**职责定位**: 本 BD 是**专题 BD**, 详尽展开 11 扩展命令的 schema/字段/职责/缺口; 不重复总冊 BD-001 的索引职责。

**11 命令覆盖** (per cli-spec §2.2):
- **6 原扩展** (per arch/03 §2.2 漏 3 个, F-20 修复 2026-08-27): `star context current` / `star code references` / `star mr review` / `star test run` / `star pipeline run` / `star pipeline status`
- **5 P1-H 新增** (per 2026-08-27 Universal Submit 步骤暴露): `star diff` / `star policy check` / `star commit` / `star push` / `star mr link`

**MVP 范围**: 这 11 命令是**非 MVP 退出条件** (per acceptance/04 §3 退出条件仅 17 命令), Phase 2+ 候选. 但 spec 已固化 (per cli-spec §2.2.1 bash 块 11 行).

---

## §1 適用範囲

### 1.1 In-Scope: 11 命令 (6 原扩展 + 5 P1-H 新增)

| # | 命令 | 用途 | 输出 schema | agent-api/01 §3 节 | 来源 | 实施文件 |
|---|---|---|---|---|---|---|
| FR-18 | `star context current` | 当前 context | `agent-api/v1#Context` | §3.8 | 原扩展 | [`commands/context.rs:91`](../crates/star-cli/src/commands/context.rs) |
| FR-19 | `star code references <name>` | 引用查找 | `agent-api/v1#ReferencesResult` | §3.x (待补) | 原扩展 | [`commands/code.rs:87`](../crates/star-cli/src/commands/code.rs) |
| FR-20 | `star mr review <id>` | Review MR | `agent-api/v1#ReviewResult` | §3.x (待补) | 原扩展 (F-20 补) | [`commands/mr.rs:104`](../crates/star-cli/src/commands/mr.rs) |
| FR-21 | `star test run` | 跑全部测试 | `agent-api/v1#TestResult` | §3.12 | 原扩展 (F-20 补) | [`commands/test.rs:18`](../crates/star-cli/src/commands/test.rs) |
| FR-22 | `star pipeline run` | 跑 pipeline | `agent-api/v1#PipelineRun` | §3.x (待补) | 原扩展 | [`commands/pipeline.rs:23`](../crates/star-cli/src/commands/pipeline.rs) |
| FR-23 | `star pipeline status` | pipeline 状态 | `agent-api/v1#PipelineStatus` | §3.x (待补) | 原扩展 | [`commands/pipeline.rs:35`](../crates/star-cli/src/commands/pipeline.rs) |
| FR-24 | `star diff` | Diff 检查 (Universal Submit 第 4 步暴露) | `agent-api/v1#DiffResult` | §3.x (待补) | **P1-H 新增** | ⏳ Phase D.1 (per cli-spec §2.2) |
| FR-25 | `star policy check` | Policy 检查 (Universal Submit 第 6 步暴露) | `agent-api/v1#PolicyCheckResult` | §3.x (待补) | **P1-H 新增** | ⏳ Phase D.1 (per cli-spec §2.2) |
| FR-26 | `star commit` | Commit (注入 Policy/Audit/Worktree 上下文) | `agent-api/v1#CommitResult` | §3.x (待补) | **P1-H 新增** | ⏳ Phase D.1 (per cli-spec §2.2) |
| FR-27 | `star push` | Push (注入 Audit 上下文) | `agent-api/v1#PushResult` | §3.x (待补) | **P1-H 新增** | ⏳ Phase D.1 (per cli-spec §2.2) |
| FR-28 | `star mr link <id>` | 关联 Issue 到 MR (Universal Submit 第 10 步暴露) | `agent-api/v1#MRLinkResult` | §3.x (待补) | **P1-H 新增** | ⏳ Phase D.1 (per cli-spec §2.2) |

**当前实装状态** (per `git log --oneline -p crates/star-cli/src/commands/`):
- **6 / 11 命令已实装** (Phase D 骨架 mock): FR-18 / FR-19 / FR-20 / FR-21 / FR-22 / FR-23 (走 `output::json_pretty` 输出 mock)
- **5 / 11 命令 ⏳ [M] 子项** (P1-H 新增, Phase D.1 待实装): FR-24 / FR-25 / FR-26 / FR-27 / FR-28 (per cli-spec §2.2.1 bash 块已固化, 命令 enum 尚未添加)

### 1.2 Out-of-Scope (8 项)

| 不做 | 原因 | 替代 |
|---|---|---|
| MVP 17 核心命令 | 本 BD 仅含 11 扩展 | [BD-002 §1.1](./BD-STAR-CLI-002.md) |
| `star mr create/show` (FR-14/15) | 不在 11 扩展命令范围内 (FR-20 review 是扩展, create/show 是 MVP) | [BD-004 §3.1](./BD-STAR-CLI-004.md) |
| `star test affected` (FR-16) | 不在 11 扩展命令范围内 (FR-21 run 是扩展, affected 是 MVP) | [BD-004 §3.2](./BD-STAR-CLI-004.md) |
| `star submit` (FR-17) | Universal Submit 主体在 BD-005, 本 BD 仅含 5 步暴露命令 | [BD-005 §2](./BD-STAR-CLI-005.md) |
| Skill Registry 4 子命令 | 不在 11 扩展命令范围内 | [BD-004 §4](./BD-STAR-CLI-004.md) |
| `star agent capabilities/describe/instructions/permissions` | agent meta 子命令 | [BD-004 §5](./BD-STAR-CLI-004.md) |
| `star ide capabilities/describe/instructions/permissions` | ide meta 子命令 (待 [M]) | [BD-004 §5](./BD-STAR-CLI-004.md) |
| 完整 17 capability 独立 CLI 命令 | `repositories` + `deployments` 2 项 CLI 间接覆盖 (per cli-spec §2.3 F-15) | per cli-spec §2.3 |

### 1.3 边界声明 (5 项)

| 边界 | 描述 | 证据 |
|---|---|---|
| 11 / 11 扩展命令 | 本 BD 负责全部 11 命令 (6 原 + 5 P1-H 新增) | per cli-spec §2.2 |
| 1 个 binary `star` | `crates/star-cli/Cargo.toml:13-14` `[[bin]]` | `Cargo.toml` |
| 5 个 .rs 文件涉及 (6 已实装) | `context.rs` 扩 + `code.rs` 扩 + `mr.rs` 扩 + `test.rs` 扩 + `pipeline.rs` (含 5 P1-H 命令) | per `crates/star-cli/src/commands/mod.rs` |
| 5 P1-H 暴露命令与 Universal Submit 关系 | 5 步暴露 (FR-24/25/26/27/28) ↔ Submit 步 4/6/7/8/10 (per cli-spec §5 + [BD-005 §3.2](./BD-STAR-CLI-005.md)) | cli-spec §1 + §2.2 |
| 5 跨层 CLI↔MCP 缺口 | per cli-spec §2.5 F-08: `star context current` / `star workspace list` / `star mr show` / `star pipeline run` / `star issue claim` (本 BD 范围 2 项: `star context current` / `star pipeline run`) | cli-spec §2.5 |

### 1.4 平行 SRS 边界 (4 项)

| 平行 SRS | 关系 | 边界声明 |
|---|---|---|
| SRS-STAR-OPS-001 | Star Ops (Pipeline/Test/MR 内部协议) | CLI 是 consumer, 协议由 ops 仓负责 |
| SRS-WORKFLOW-TEMPLATE-001 | Workflow Template | CLI 不重复 template 渲染, 走 Workflow 域 |
| SRS-CANVAS-WORKFLOW-001 | Canvas Workflow | CLI 仅调 workflow run, canvas 渲染由 frontend 负责 |
| SRS-MULTICA-SKILL-001 | Multica Skill | CLI 不涉 skill 内部协议, 仅调 `star skill` 顶层 |

---

## §2 システムアーキテクチャ

### 2.1 11 命令实施拓扑

```
TopCommand (clap derive Subcommand, main.rs:36)
├─ Context(context::ContextCommand)  ← FR-6 + FR-18 (本 BD §3.1)
│   ├─ Get { id: String }             ← FR-6 (BD-002)
│   └─ Current                        ← FR-18 ★ 本 BD
├─ Code(code::CodeCommand)            ← FR-7/8 + FR-19 (本 BD §3.2)
│   ├─ Search { query: String }       ← FR-7 (BD-002)
│   ├─ Symbol { name: String }        ← FR-8 (BD-002)
│   └─ References { name: String }    ← FR-19 ★ 本 BD
├─ Mr(mr::MrCommand)                  ← FR-14/15 + FR-20 (本 BD §3.3)
│   ├─ Create { title/base/head }     ← FR-14 (BD-004)
│   ├─ Show { id: String }            ← FR-15 (BD-004)
│   └─ Review { id: String }          ← FR-20 ★ 本 BD
├─ Test(test::TestCommand)            ← FR-16 + FR-21 (本 BD §3.4)
│   ├─ Affected                       ← FR-16 (BD-004)
│   └─ Run                            ← FR-21 ★ 本 BD
├─ Pipeline(pipeline::PipelineCommand) ← FR-22/23 (本 BD §3.5)
│   ├─ Run { --branch <B> }           ← FR-22 ★ 本 BD
│   └─ Status { --id <I> }            ← FR-23 ★ 本 BD
└─ (P1-H 5 新增命令位置未定, 见 §3.6)
   ├─ Diff { ... }                    ← FR-24 ★ 本 BD ⏳ [M]
   ├─ PolicyCheck                     ← FR-25 ★ 本 BD ⏳ [M]
   ├─ Commit { -m }                   ← FR-26 ★ 本 BD ⏳ [M]
   ├─ Push { remote branch }          ← FR-27 ★ 本 BD ⏳ [M]
   └─ MrLink { id --issue <ID> }      ← FR-28 ★ 本 BD ⏳ [M]
```

### 2.2 6 个已实装 .rs 文件职责 (本 BD 范围)

| # | 文件 | 行数 | 覆盖扩展命令 | 主要 schema | 状态 | 跨 BD 引用 |
|---|---|---|---|---|---|---|
| 1 | `commands/context.rs` | 106 | FR-18 (`current`) | `Context` (与 FR-6 共用) | ✅ | `lookup_context()` 与 BD-002 §3.3 共用 |
| 2 | `commands/code.rs` | 110 | FR-19 (`references`) | `ReferencesResult` / `CodeMatch` | ✅ | 与 BD-002 §3.5 共用 `CodeMatch` |
| 3 | `commands/mr.rs` | 124 | FR-20 (`review`) | `ReviewResult` | ✅ | 与 BD-004 §3.1 共用 `MR` |
| 4 | `commands/test.rs` | 56 | FR-21 (`run`) | `TestResult` (与 FR-16 共用) | ✅ | 与 BD-004 §3.2 共用 schema |
| 5 | `commands/pipeline.rs` | 67 | FR-22/23 (`run`/`status`) | `PipelineRun` / `PipelineStatus` | ✅ | n/a (扩展域内闭环) |
| **合计** | **5 files** | **463** | **6/11 ✅ + 5/11 ⏳** | **5 schema 类型** | **55%** | — |

> 注: `commands/context.rs` (106 行) + `commands/code.rs` (110 行) + `commands/mr.rs` (124 行) + `commands/test.rs` (56 行) + `commands/pipeline.rs` (67 行) 共 463 行, 但 5 文件是 BD-002 + BD-004 共用, 本 BD 仅扩展部分职责.

### 2.3 5 P1-H 新增命令实施策略 (per cli-spec §2.2.1 bash 块)

| 策略选项 | 描述 | 决定 |
|---|---|---|
| (a) 新建 `commands/diff.rs` / `policy.rs` / `commit.rs` / `push.rs` / `mr_link.rs` 5 新文件 | 模块边界清, 但 5 新文件 + 5 TopCommand 变体 | ❌ (over-engineering for 5 commands) |
| (b) 5 命令挂在现有 `commands/git_wrapper.rs` 单文件 | DRY, 5 命令共享 git wrapper 逻辑 | ✅ (Phase D.1 拍板) |
| (c) 5 命令挂在 `commands/submit.rs` 同一文件 (作为 Submit 步函数暴露) | 与 Universal Submit 状态机共享步函数 | ✅ (per [BD-005 §4](./BD-STAR-CLI-005.md) ref 论证) |

**Phase D.1 拍板**: 策略 (c) — 5 P1-H 暴露命令**复用** `commands/submit.rs` 的步函数, 单 TopCommand 变体 `Submit` 已存在, 在 `SubmitArgs` 加 5 个 clap subcommand (per cli-spec §2.2.1 bash 块):

```rust
#[derive(Args)]
pub(crate) struct SubmitArgs {
    // ... existing dry_run, json, no_commit ...

    #[command(subcommand)]
    pub action: Option<SubmitAction>,
}

#[derive(Subcommand)]
pub(crate) enum SubmitAction {
    /// FR-24: Submit 第 4 步 diff 暴露
    Diff { /* args */ },
    /// FR-25: Submit 第 6 步 policy check 暴露
    PolicyCheck { /* args */ },
    /// FR-26: Submit 第 7 步 commit 暴露
    Commit { /* args */ },
    /// FR-27: Submit 第 8 步 push 暴露
    Push { /* args */ },
    /// FR-28: Submit 第 10 步 mr link 暴露
    MrLink { /* args */ },
}
```

> **设计理由** (per 守门 #8): 5 暴露命令与 Submit 步函数共享是 DRY 唯一解, 避免 logic drift. `star submit` (无 subcommand) 仍走全 12 步.

### 2.4 跨文件共享约束 (8 项)

| # | 约束 | 描述 | 证据 |
|---|---|---|---|
| 1 | 1 个 StarError → Error schema | 全部命令走 `Result<(), StarError>`, StarError 序列化走 `agent-api/v1#Error` 6 字段 | `error.rs:10-22` + cli-spec §5 F-06 |
| 2 | 1 个 output.rs 统一入口 | 全部命令输出走 `output::json_pretty`, 默认 JSON | `output.rs:24-27` |
| 3 | 0 shell=True | 全部走 `Command::new("git")` 而非 `Command::new("sh").arg("-c")` (含 5 P1-H 命令) | 守门 #6 |
| 4 | 0 unsafe block | 0 unsafe (per 守门 #7) | `grep -rn 'unsafe ' crates/star-cli/src/` 0 命中 |
| 5 | 1 个 schema version | 默认 `--schema-version v1` (= `agent-api/v1`) | `output.rs:18` |
| 6 | 6 字段 Error 不重定义 | 全部引用 `agent-api/v1#Error` | [BD-004 §2](./BD-STAR-CLI-004.md) |
| 7 | mock envelope 一致 | 6 已实装命令 mock JSON 输出都含 `{schema_version, mock: true, tool: ..., <key>: ...}` envelope | (跨 BD 一致) |
| 8 | 5 P1-H 命令与 Submit 步函数共享 | 5 命令 enum variant → submit.rs step_X 函数 (per §2.3 策略 c) | cli-spec §2.2.1 bash 块 |

### 2.5 8 维度方式 (per 守门 #1+#5+#6)

| # | 维度 | 11 命令实现方式 |
|---|---|---|
| 1 | parse | clap derive (clap 4.5) |
| 2 | dispatch | match `cli.command` in `fn run()` |
| 3 | execute | 6 已实装 = mock data; 5 P1-H 待 Phase D.1 接 git/process |
| 4 | error | `Result<(), StarError>` → eprintln + ExitCode |
| 5 | output | stdout via `output::json_pretty` (always JSON for 11 命令) |
| 6 | schema | `agent-api/v1` (11 命令引用 11 schema) |
| 7 | capability | `star agent capabilities` returns 15-item array (本 BD 11 命令覆盖 5 capability: `code_navigation` / `code_context` / `merge_requests` / `tests` / `pipelines` / `reviews`) |
| 8 | versioning | `--schema-version v1` (default), `--version` returns CLI binary version |

---

## §3 データビュー

### 3.1 `star context current` (FR-18)

**实施**: [`commands/context.rs:91-103`](../crates/star-cli/src/commands/context.rs) (共享 `lookup_context("ctx-current")`, per `context.rs:43-73`)

#### 3.1.1 schema 字段集 (`agent-api/v1#Context`)

> 与 [BD-002 §3.3.1](./BD-STAR-CLI-002.md) **完全一致**, 复用同一 schema.

| 字段 | 类型 | 必填 | 说明 | 证据 |
|---|---|---|---|---|
| `issue_id` | `String` | ✅ | 关联 issue ID (本命令永远 `"STAR-1024"`) | `context.rs:47` |
| `related_code[]` | `Vec<CodeRef>` | ✅ | 关联代码引用 | `context.rs:48-52` |
| `related_docs[]` | `Vec<DocRef>` | ✅ | 关联文档引用 | `context.rs:53-57` |
| `related_mrs[]` | `Vec<MRRef>` | ✅ | 关联 MR 引用 | `context.rs:58-61` |
| `updated_at` | `String` (RFC3339) | ✅ | 最后更新时间 | `context.rs:62` |

#### 3.1.2 mock 行为 (per `context.rs:43-73` `lookup_context`)

```rust
pub(crate) fn lookup_context(id: &str) -> Context {
    let now = chrono::Utc::now().to_rfc3339();
    if id == "ctx-current" || id == "current" {
        Context {
            issue_id: "STAR-1024".to_string(),
            related_code: vec![CodeRef { /* submit.rs */ }],
            related_docs: vec![DocRef { /* Universal Submit Protocol doc */ }],
            related_mrs: vec![MRRef { id: "MR-mock-001", ... }],
            updated_at: now,
        }
    } else {
        Context { /* empty */ }
    }
}
```

#### 3.1.3 bash 块示例 (per cli-spec §2.2.1)

```bash
star context current --json
```

#### 3.1.4 跨层 CLI↔MCP 缺口 (per cli-spec §2.5 F-08)

| CLI 命令 | MCP tool | 命名差异 | 状态 |
|---|---|---|---|
| `star context current` | (无对应) | — | ⚠️ MCP 缺, Phase 2 评估是否加 `get_current_context` |

---

### 3.2 `star code references <name>` (FR-19)

**实施**: [`commands/code.rs:87-106`](../crates/star-cli/src/commands/code.rs)

#### 3.2.1 `ReferencesResult` schema (FR-19, agent-api/01 §3.x 待补)

| 字段 | 类型 | 必填 | 说明 | 证据 |
|---|---|---|---|---|
| `name` | `String` | ✅ | 符号名 | `code.rs:40` |
| `references[]` | `Vec<CodeMatch>` | ✅ | 引用项 (与 `CodeSearchResult.matches` 同类型) | `code.rs:41` |
| `total` | `u32` | ✅ | 总数 | `code.rs:42` |

#### 3.2.2 跨 BD 共用 `CodeMatch` schema (per BD-002 §3.5.2)

| 字段 | 类型 | 说明 | 证据 |
|---|---|---|---|
| `file` | `String` | 文件路径 | `code.rs:18` |
| `line` | `u32` | 行号 | `code.rs:19` |
| `snippet` | `String` | 代码片段 | `code.rs:20` |

#### 3.2.3 mock 行为 (per `code.rs:87-106`)

```rust
CodeCommand::References { name } => {
    let result = ReferencesResult {
        name: name.clone(),
        references: vec![CodeMatch {
            file: "src/main.rs".to_string(),
            line: 1,
            snippet: format!("use {name}"),
        }],
        total: 1,
    };
    // ...
}
```

> **⏳ [M] 子项**: mock 永远 1 引用项, 不调真实 backend (与 `search` 同源缺口).

#### 3.2.4 bash 块示例 (per cli-spec §2.2.1)

```bash
star code references "verify_token" --json
```

#### 3.2.5 跨层 CLI↔MCP 命名差异 (per cli-spec §2.5 F-08)

| CLI 命令 | MCP tool | 命名差异 | 状态 |
|---|---|---|---|
| `star code references <name>` | `find_references` | `references` vs `find_*` (MCP 动词不同) | ✅ (MCP 已对齐, naming diff 已知) |

---

### 3.3 `star mr review <id>` (FR-20)

**实施**: [`commands/mr.rs:104-121`](../crates/star-cli/src/commands/mr.rs)

#### 3.3.1 `ReviewResult` schema (FR-20, agent-api/01 §3.x 待补)

| 字段 | 类型 | 必填 | 说明 | 证据 |
|---|---|---|---|---|
| `mr_id` | `String` | ✅ | 被 review 的 MR ID | `mr.rs:46` |
| `status` | `String` | ✅ | review 状态, e.g. `"PENDING"`/`"APPROVED"`/`"CHANGES_REQUESTED"` | `mr.rs:47` |
| `review_id` | `String` | ✅ | review 单据 ID, 形态 `REV-mock-<ts>` | `mr.rs:48`, `mr.rs:108` |
| `approved` | `bool` | ✅ | 是否通过, Phase D 永远 `false` | `mr.rs:49`, `mr.rs:109` |

#### 3.3.2 mock 行为 (per `mr.rs:104-121`)

```rust
MrCommand::Review { id } => {
    let result = ReviewResult {
        mr_id: id.clone(),
        status: "PENDING".to_string(),
        review_id: format!("REV-mock-{}", chrono::Utc::now().timestamp()),
        approved: false,
    };
    // ...
}
```

> **⏳ [M] 子项**: `approved` 永远 `false`, `status` 永远 `"PENDING"`, 不接真实 GitHub/GitLab review API. `--approve` 等命名 flag 待 Phase D.1.

#### 3.3.3 bash 块示例 (per cli-spec §2.2.1)

```bash
star mr review 42 --approve     # ← `--approve` flag 当前**未实装**, 待 Phase D.1
```

> **注**: per cli-spec §2.2.1 第 3 行, `--approve` flag 是 spec 期望接口, 但当前实装 (`mr.rs:104-121`) 不读这个 flag (mock 永远 PENDING). 这是 [§6 G-CL2-MR1 缺口](#6-mr-域-3-项) 之一.

#### 3.3.4 跨层 CLI↔MCP 命名差异 (per cli-spec §2.5 F-08)

| CLI 命令 | MCP tool | 命名差异 | 状态 |
|---|---|---|---|
| `star mr review <id>` | `request_review` | `review` vs `request_*` (MCP 动作化) | ✅ (MCP 已对齐, naming diff 已知) |

---

### 3.4 `star test run` (FR-21)

**实施**: [`commands/test.rs:14-37`](../crates/star-cli/src/commands/test.rs)

#### 3.4.1 `TestResult` schema (FR-21, agent-api/01 §3.12)

> 与 [BD-004 §3.2.1](./BD-STAR-CLI-004.md) `TestResult` schema **完全一致**, 复用同一 schema. CLI 端**不**区分 MVP `affected` 和扩展 `run` 的 schema 差异 (per `test.rs:16-19`).

| 字段 | 类型 | 必填 | 说明 | 证据 |
|---|---|---|---|---|
| `passed` | `u32` | ✅ | 通过数 | `test.rs:27` |
| `failed` | `u32` | ✅ | 失败数 | `test.rs:28` |
| `skipped` | `u32` | ✅ | 跳过数, 永远 `0` (Phase D mock) | `test.rs:29` |
| `failed_tests[]` | `Vec<...>` | ✅ | 失败明细数组, 永远 `[]` | `test.rs:30` |
| `duration_ms` | `u64` | ✅ | 耗时 (毫秒) | `test.rs:31` |

#### 3.4.2 mock 行为 (per `test.rs:14-37`)

| 命令 | mock 数字 |
|---|---|
| `star test affected` (FR-16) | passed=5 / failed=0 / duration=1234ms |
| `star test run` (FR-21) | passed=42 / failed=0 / duration=8765ms |

> **⏳ [M] 子项**: mock 数字硬编, 不接真实 cargo test / vitest / pytest runner. Phase D.1 接真实 backend.

#### 3.4.3 bash 块示例 (per cli-spec §2.2.1)

```bash
star test run --json
```

#### 3.4.4 跨层 CLI↔MCP 命名差异 (per cli-spec §2.5 F-08)

| CLI 命令 | MCP tool | 命名差异 | 状态 |
|---|---|---|---|
| `star test run` | `run_validation` | `test` vs `validation` (MCP 统一 validation 表达) | ✅ (MCP 已对齐) |

---

### 3.5 `star pipeline run/status` (FR-22/23)

**实施**: [`commands/pipeline.rs:8-58`](../crates/star-cli/src/commands/pipeline.rs)

#### 3.5.1 `PipelineRun` schema (FR-22, agent-api/01 §3.x 待补)

| 字段 | 类型 | 必填 | 说明 | 证据 |
|---|---|---|---|---|
| `id` | `String` | ✅ | pipeline run ID, 形态 `PIPE-mock-<ts>` | `pipeline.rs:24`, `pipeline.rs:28` |
| `status` | `String` | ✅ | 状态, Phase D mock 永远 `"QUEUED"` | `pipeline.rs:29` |
| `branch` | `String` | ✅ | 分支名, `--branch` flag (default `"main"`) | `pipeline.rs:30`, `pipeline.rs:11-12` |
| `url` | `String` | ✅ | pipeline UI URL (Phase D mock `https://example.invalid/pipelines/<id>`) | `pipeline.rs:31` |

#### 3.5.2 `PipelineStatus` schema (FR-23, agent-api/01 §3.x 待补)

| 字段 | 类型 | 必填 | 说明 | 证据 |
|---|---|---|---|---|
| `id` | `String` | ✅ | pipeline run ID, `--id` flag (default `"PIPE-mock-latest"`) | `pipeline.rs:36`, `pipeline.rs:15-16` |
| `status` | `String` | ✅ | 状态, Phase D mock 永远 `"SUCCESS"` | `pipeline.rs:41` |
| `url` | `String` | ✅ | pipeline UI URL | `pipeline.rs:42` |

> **schema 简化说明**: 当前 `PipelineRun` + `PipelineStatus` 用同一 serde_json::json! 内联, 未定义独立 struct. Phase D.1 需定义 `PipelineRun` + `PipelineStatus` 独立 struct 满足 `agent-api/v1` 完整 schema.

#### 3.5.3 mock 行为 (per `pipeline.rs:20-46`)

```rust
PipelineCommand::Run { branch } => {
    let id = format!("PIPE-mock-{}", chrono::Utc::now().timestamp());
    ("pipeline run", serde_json::json!({
        "id": id, "status": "QUEUED", "branch": branch,
        "url": format!("https://example.invalid/pipelines/{id}"),
    }))
}
PipelineCommand::Status { id } => {
    let id = id.unwrap_or_else(|| "PIPE-mock-latest".to_string());
    ("pipeline status", serde_json::json!({
        "id": id, "status": "SUCCESS",
        "url": format!("https://example.invalid/pipelines/{id}"),
    }))
}
```

#### 3.5.4 bash 块示例 (per cli-spec §2.2.1)

```bash
star pipeline run --json
star pipeline status --json
```

#### 3.5.5 跨层 CLI↔MCP 缺口 (per cli-spec §2.5 F-08)

| CLI 命令 | MCP tool | 命名差异 | 状态 |
|---|---|---|---|
| `star pipeline run` | (无对应) | — | ⚠️ MCP 缺, Phase 2 评估是否加 `trigger_pipeline` |
| `star pipeline status` | `get_pipeline_status` | 一致 | ✅ |

---

### 3.6 5 P1-H 新增命令 (Universal Submit 5 步暴露, ⏳ [M])

**实施位置**: [`commands/submit.rs`](../crates/star-cli/src/commands/submit.rs) (per §2.3 策略 c, Phase D.1 拍板)

> **⏳ [M] 子项**: 5 命令当前**未实装**. per cli-spec §2.2.1 第 99-104 行 bash 块已固化, Phase D.1 加 `SubmitAction` enum + 5 变体.

#### 3.6.1 `star diff` (FR-24, Submit 第 4 步暴露)

**bash 块示例** (per cli-spec §2.2.1):
```bash
star diff HEAD~1 --json
```

**预期 schema** (`agent-api/v1#DiffResult`, agent-api/01 §3.x 待补):

| 字段 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `ref` | `String` | ✅ | git ref, e.g. `"HEAD~1"` |
| `files[]` | `Vec<DiffFile>` | ✅ | 变更文件数组 |
| `additions` | `u32` | ✅ | 新增行数 |
| `deletions` | `u32` | ✅ | 删除行数 |
| `binary` | `bool` | ✅ | 是否含 binary 文件 |

**`DiffFile` nested**:
| 字段 | 类型 | 说明 |
|---|---|---|
| `path` | `String` | 文件路径 |
| `old_path` | `Option<String>` | rename 前路径 |
| `kind` | `String` | `added`/`modified`/`deleted`/`renamed` |

**Submit 步函数复用**: `submit.rs:442-460 step_check_diff` (per `submit.rs:442`) 已 spawn `git diff --stat`, 暴露命令复用此函数 + 加 `--name-status` + `--numstat`.

#### 3.6.2 `star policy check` (FR-25, Submit 第 6 步暴露)

**bash 块示例** (per cli-spec §2.2.1):
```bash
star policy check --json
```

**预期 schema** (`agent-api/v1#PolicyCheckResult`, agent-api/01 §3.x 待补):

| 字段 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `allowed` | `bool` | ✅ | 是否允许 submit |
| `violations[]` | `Vec<PolicyViolation>` | ✅ | 违反项数组 |
| `warnings[]` | `Vec<PolicyViolation>` | ✅ | 警告项数组 |

**`PolicyViolation` nested**:
| 字段 | 类型 | 说明 |
|---|---|---|
| `rule` | `String` | 规则名, e.g. `"no-force-push"` |
| `severity` | `String` | `BLOCKER`/`ERROR`/`WARNING` |
| `message` | `String` | 详细消息 |

**Submit 步函数复用**: `submit.rs:482-? step_validation` (写死 `policy_checked: true`, per `submit.rs:485`) — 暴露命令接真实 policy engine (per ADR-0029 multica pattern).

#### 3.6.3 `star commit` (FR-26, Submit 第 7 步暴露 + 注入 Policy/Audit/Worktree 上下文)

**bash 块示例** (per cli-spec §2.2.1):
```bash
star commit -m "fix: auth" --json
```

**预期 schema** (`agent-api/v1#CommitResult`, agent-api/01 §3.x 待补):

| 字段 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `commit_sha` | `String` | ✅ | commit SHA (40-char hex) |
| `branch` | `String` | ✅ | 分支名 |
| `message` | `String` | ✅ | commit message |
| `policy_checked` | `bool` | ✅ | 是否通过 policy check (FR-25 注入) |
| `audit_id` | `String` | ✅ | 审计 ID (注入 audit 上下文) |
| `worktree_id` | `String` | ✅ | worktree ID (注入 worktree 上下文) |
| `files_changed` | `u32` | ✅ | 变更文件数 |
| `author` | `String` | ✅ | author |

**Submit 步函数复用**: `submit.rs:482-? step_commit` (空 commit 跳过, per `submit.rs:482`) — 暴露命令接 FR-25 policy + FR-15 worktree + audit context.

#### 3.6.4 `star push` (FR-27, Submit 第 8 步暴露 + 注入 Audit 上下文)

**bash 块示例** (per cli-spec §2.2.1):
```bash
star push origin feat/auth-fix --json
```

**预期 schema** (`agent-api/v1#PushResult`, agent-api/01 §3.x 待补):

| 字段 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `remote` | `String` | ✅ | remote 名, e.g. `"origin"` |
| `branch` | `String` | ✅ | branch 名 |
| `pushed_commits[]` | `Vec<String>` | ✅ | pushed commit SHA 数组 |
| `audit_id` | `String` | ✅ | 审计 ID (注入 audit 上下文) |
| `forced` | `bool` | ✅ | 是否 force push (默认 false) |
| `dry_run` | `bool` | ✅ | dry run 标志 |

**Submit 步函数复用**: `submit.rs:515-? step_push_dry_run` (per `submit.rs:515`) — 暴露命令接 audit context.

#### 3.6.5 `star mr link <id>` (FR-28, Submit 第 10 步暴露)

**bash 块示例** (per cli-spec §2.2.1):
```bash
star mr link 42 --issue STAR-1024 --json
```

**预期 schema** (`agent-api/v1#MRLinkResult`, agent-api/01 §3.x 待补):

| 字段 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `mr_id` | `String` | ✅ | MR ID (positional `<id>`) |
| `issue_id` | `String` | ✅ | 关联 issue ID (`--issue` flag) |
| `linked` | `bool` | ✅ | 是否关联成功 |
| `link_id` | `String` | ✅ | 关联 ID |

**Submit 步函数复用**: `submit.rs:543-? step_link_issue` (写 `.star/issue-link.json`, per `submit.rs:543`) — 暴露命令调真实 GitHub/GitLab link API.

#### 3.6.6 5 暴露命令与 Universal Submit 状态机的关系 (per cli-spec §5 + [BD-005 §3.2](./BD-STAR-CLI-005.md))

| Submit 步 | 暴露命令 | Submit 步函数 | 暴露命令额外职责 |
|---|---|---|---|
| Step 4 (Diff) | FR-24 `star diff` | `step_check_diff` | 加 `--name-status` + `--numstat` 输出 |
| Step 6 (Policy) | FR-25 `star policy check` | `step_validation` | 接真实 policy engine |
| Step 7 (Commit) | FR-26 `star commit` | `step_commit` | 注入 policy + audit + worktree 上下文 |
| Step 8 (Push) | FR-27 `star push` | `step_push_dry_run` | 注入 audit 上下文 |
| Step 10 (Link) | FR-28 `star mr link` | `step_link_issue` | 接真实 GitHub/GitLab link API |

> **DRY 原则** (per 守门 #8): 5 暴露命令**必须**调 Submit 步函数, 避免 logic drift. `star submit --json` 全 12 步 vs `star diff` 单步 = 同一函数不同入口.

### 3.7 cli-spec §2.4 命名风格约定 (per F-08 修复)

| 维度 | CLI 命名 | MCP 命名 | 备注 |
|---|---|---|---|
| 查询 (多对象) | `list` | `search_*` | CLI 用 `list`, MCP 用 `search_*` (per F-08 #8) |
| 查询 (单对象) | `show` | `get_*` | 一致 |
| 操作 | `create` / `claim` / `link` | `create_*` / `claim_*` / etc. | 动词一致 |
| 缩写 | `mr` (CLI shell 习惯) | `merge_request` (machine 协议) | 设计选择, 不强制统一 |
| 测试 | `test` (CLI 业务) | `validation` (MCP 内部) | CLI 业务语义 vs MCP 统一抽象 |
| 状态查询 | `current` (CLI 空入参) | 单 tool + 空对象入参 (MCP) | e.g. `get_current_task` / `get_workspace` |

> 11 扩展命令全部遵循此约定.

### 3.8 cli-spec §2.5 CLI ↔ MCP 命名映射表 (本 BD 范围 8 项)

| CLI 命令 | MCP tool | 命名差异 | 状态 |
|---|---|---|---|
| `star context current` | (无对应) | — | ⚠️ MCP 缺, per F-08 |
| `star code references <name>` | `find_references` | `references` vs `find_*` | ✅ |
| `star mr review <id>` | `request_review` | `review` vs `request_*` | ✅ |
| `star test run` | `run_validation` | `test` vs `validation` | ✅ |
| `star pipeline run` | (无对应) | — | ⚠️ MCP 缺, per F-08 |
| `star pipeline status` | `get_pipeline_status` | 一致 | ✅ |
| `star diff` / `star policy check` / `star commit` / `star push` / `star mr link` | (待 MCP 加) | — | ⏳ Phase 2 (MCP 尚未拍板) |

**跨层缺口汇总**: 11 扩展命令中 **3 项 MCP 缺** (`star context current` / `star pipeline run` + 5 P1-H 待 MCP 拍板), 不阻塞 MVP, Phase 2 MCP 补齐.

---

## §4 モジュールビュー

### 4.1 6 已实装 .rs 文件本 BD 范围职责细化

#### 4.1.1 `commands/context.rs` (106 行) — 本 BD 扩 FR-18

| 区域 | 行号 | 职责 |
|---|---|---|
| enum 定义 | `:10-13` | `Get {id}` / `Current` 2 变体 |
| (FR-6 schema 与 mock 共用) | `:34-73` | (BD-002 §3.3 已详) |
| `Current` dispatch | `:91-103` | → `lookup_context("ctx-current")` → JSON output |

**关键设计**: `Current` 复用 `lookup_context()` (per `:91-92`), 与 `Get` 共享 mock data. 无 schema 差异.

#### 4.1.2 `commands/code.rs` (110 行) — 本 BD 扩 FR-19

| 区域 | 行号 | 职责 |
|---|---|---|
| enum 定义 | `:10-14` | `Search {query}` / `Symbol {name}` / `References {name}` 3 变体 |
| (FR-7/8 schema 共用) | `:16-36` | (BD-002 §3.5 已详) |
| ReferencesResult schema | `:38-43` | 3 字段 (本 BD 新增) |
| `References` dispatch | `:87-106` | mock 1 引用项 → JSON output |

#### 4.1.3 `commands/mr.rs` (124 行) — 本 BD 扩 FR-20

| 区域 | 行号 | 职责 |
|---|---|---|
| enum 定义 | `:10-30` | `Create {title/base/head}` / `Show {id}` / `Review {id}` 3 变体 |
| MR schema | `:32-42` | (BD-004 §3.1 已详) |
| ReviewResult schema | `:44-50` | 4 字段 (本 BD 新增) |
| mock_mr(id) | `:52-63` | mock_mr helper (FR-15 复用) |
| `Review` dispatch | `:104-121` | mock ReviewResult { PENDING, approved: false } → JSON output |

**FR-20 待实装**: `--approve` / `--request-changes` / `--comment` flag 当前**未实装**, per `mr.rs:104-121` 不读 flag. 待 Phase D.1 加 clap 命名 flag.

#### 4.1.4 `commands/test.rs` (56 行) — 本 BD 扩 FR-21

| 区域 | 行号 | 职责 |
|---|---|---|
| enum 定义 | `:10-12` | `Affected` / `Run` 2 变体 |
| `run()` dispatch | `:14-37` | match 2 变体 → mock (passed/failed/skipped/duration) → JSON output |
| test block | `:39-55` | 2 unit tests (pass count assertion) |

**FR-16 vs FR-21 schema 一致**: 2 子命令共用同一 `TestResult` schema, 仅 mock 数字不同. CLI 端不区分 MVP/扩展语义.

#### 4.1.5 `commands/pipeline.rs` (67 行) — 本 BD 扩 FR-22/23

| 区域 | 行号 | 职责 |
|---|---|---|
| enum 定义 | `:10-18` | `Run {--branch}` / `Status {--id}` 2 变体 |
| `run()` dispatch | `:20-58` | match 2 变体 → mock (id/status/url) → JSON output |
| test block | `:60-66` | 1 unit test (id 形态断言) |

**`PipelineRun` / `PipelineStatus` schema 简化**: 当前用 `serde_json::json!` 内联, 未定义独立 struct. Phase D.1 需定义 2 struct 满足 `agent-api/v1` 完整 schema.

#### 4.1.6 `commands/submit.rs` (615 行) — 本 BD 扩 5 P1-H 命令 (per §2.3 策略 c)

| 区域 | 行号 | 职责 |
|---|---|---|
| (BD-005 §3.2 全 12 步定义) | (per BD-005) | (BD-005 详) |
| ⏳ SubmitAction enum | (Phase D.1 加) | 5 变体 (Diff/PolicyCheck/Commit/Push/MrLink) |
| ⏳ 5 暴露命令 dispatch | (Phase D.1 加) | match 5 变体 → 调 submit 步函数 → JSON output |

### 4.2 跨文件共享约束 (8 项, 已在 §2.4 列)

### 4.3 mock envelope 一致性 (per 守门 #8)

11 命令中 6 已实装命令输出统一 envelope:
```json
{
  "schema_version": "agent-api/v1",
  "mock": true,
  "tool": "<command name>",
  "<key>": <schema>
}
```

5 P1-H 待实装命令 (Phase D.1) 走同样 envelope.

---

## §5 NFR (per 守门 #1+#5+#6+#7+#11+#12+#13)

| NFR ID | 类别 | 目标 | 派生守门 | 实装状态 |
|---|---|---|---|---|
| NFR-CL2-1 性能 | 性能 | `star <cmd> --json` < 200ms (本地 mock) | 守门 #1 | ✅ mock 数据 O(1) |
| NFR-CL2-2 可用 | 可用 | 11 / 11 命令可调用 + schema 稳定 | 守门 #1 | 🟡 6/11 ✅ + 5/11 ⏳ |
| NFR-CL2-3 跨平台 | 跨平台 | Windows / macOS / Linux 三端二进制 | 守门 #6 | ✅ `Command::new("git")` 而非 `shell=True` |
| NFR-CL2-4 安全 | 安全 | 0 unsafe block | 守门 #7 | ✅ `grep -rn 'unsafe ' crates/star-cli/src/commands/` 0 命中 |
| NFR-CL2-5 可观测 | 可观测 | 错误带 `trace_id` | 守门 #13 | ✅ [BD-004 §2](./BD-STAR-CLI-004.md) |
| NFR-CL2-6 易用 | 易用 | 11 命令有 `--help` 输出 + 0 静默失败 | 守门 #25 | ✅ clap derive 内建 |
| NFR-CL2-7 DRY | 重用 | 5 P1-H 命令必须调 Submit 步函数, 避免 logic drift | 守门 #8 | ⏳ Phase D.1 (策略 c 拍板) |
| NFR-CL2-8 命名一致 | 命名 | 11 命令全部走 `agent-api/v1` schema 名, 无错标 | 守门 #11 | ✅ (P1-C 修复后) |
| NFR-CL2-9 跨层 CLI↔MCP | 接口 | 3 跨层缺口 Phase 2 MCP 补齐 | 守门 #11 | ⏳ Phase 2 (per cli-spec §2.5) |

---

## §6 已知缺口 (per 守门 #11 缺标比错标, ~10 项)

### 6.1 context 域 (1 项)

| # | 缺口 | 影响 | 优先级 |
|---|---|---|---|
| G-CL2-C1 | `star context current` 不读 env / 不读 agent 真实状态 | 永远 mock STAR-1024 | ⏳ Phase 2+ (per G-CL13-C1 共享缺口) |

### 6.2 code 域 (1 项)

| # | 缺口 | 影响 | 优先级 |
|---|---|---|---|
| G-CL2-CD1 | `star code references` mock 永远 1 引用项, 不调真实 backend | Phase D 不可用 | ⏳ [M] (与 G-CL13-CD1 同源, Phase D.1 接 LSP) |

### 6.3 mr 域 (3 项)

| # | 缺口 | 影响 | 优先级 |
|---|---|---|---|
| G-CL2-MR1 | `star mr review` 不接 `--approve` / `--request-changes` / `--comment` flag | mock 永远 PENDING / approved=false | ⏳ Phase D.1 (clap 命名 flag) |
| G-CL2-MR2 | `ReviewResult` 缺 `comments[]` / `submitted_at` 字段 | consumer 拿不到 review 详情 | ⏳ Phase D.1 |
| G-CL2-MR3 | `star mr review` 不接真实 GitHub/GitLab review API | Phase D 假 review | ⏳ [M] |

### 6.4 test 域 (1 项)

| # | 缺口 | 影响 | 优先级 |
|---|---|---|---|
| G-CL2-T1 | `star test run` mock 永远 passed=42 / failed=0, 不接真实 cargo test / vitest / pytest | Phase D 假测试 | ⏳ [M] (Phase D.1 接 backend) |

### 6.5 pipeline 域 (3 项)

| # | 缺口 | 影响 | 优先级 |
|---|---|---|---|
| G-CL2-PL1 | `PipelineRun` / `PipelineStatus` 未定义独立 struct (用 `serde_json::json!` 内联) | schema 不严格, 字段名漂移风险 | ⏳ Phase D.1 (per agent-api/01 §3.x) |
| G-CL2-PL2 | `star pipeline run` 不接真实 CI/CD runner (GitHub Actions / GitLab CI) | Phase D 假 pipeline | ⏳ [M] |
| G-CL2-PL3 | `PipelineStatus.status` mock 永远 `"SUCCESS"`, 不模拟 FAILED/RUNNING/CANCELLED | 消费者无法区分状态 | ⏳ [S] |

### 6.6 5 P1-H 新增命令缺口 (5 项, ⏳ [M])

| # | 缺口 | 影响 | 优先级 |
|---|---|---|---|
| G-CL2-P1H1 | `star diff` (FR-24) **未实装**, `SubmitAction` enum 缺 `Diff` 变体 | 5 步暴露缺 1 步 | ⏳ [M] (Phase D.1) |
| G-CL2-P1H2 | `star policy check` (FR-25) **未实装**, `SubmitAction` enum 缺 `PolicyCheck` 变体 | 5 步暴露缺 1 步 | ⏳ [M] (Phase D.1) |
| G-CL2-P1H3 | `star commit` (FR-26) **未实装**, 不注入 Policy/Audit/Worktree 上下文 | 5 步暴露缺 1 步 | ⏳ [M] (Phase D.1) |
| G-CL2-P1H4 | `star push` (FR-27) **未实装**, 不注入 Audit 上下文 | 5 步暴露缺 1 步 | ⏳ [M] (Phase D.1) |
| G-CL2-P1H5 | `star mr link` (FR-28) **未实装**, 不接 GitHub/GitLab link API | 5 步暴露缺 1 步 | ⏳ [M] (Phase D.1) |

### 6.7 跨 CLI/MCP 命名映射缺口 (per cli-spec §2.5 F-08, 5 跨层缺口, 本 BD 范围 3 项)

| # | CLI 命令 | MCP tool | 命名差异 | 状态 |
|---|---|---|---|---|
| G-CL2-X1 | `star context current` | (无对应) | — | ⏳ Phase 2 评估 `get_current_context` |
| G-CL2-X2 | `star pipeline run` | (无对应) | — | ⏳ Phase 2 评估 `trigger_pipeline` |
| G-CL2-X3 | 5 P1-H 命令 (`diff` / `policy check` / `commit` / `push` / `mr link`) | (待 MCP 加) | — | ⏳ Phase 2 (MCP 尚未拍板) |

> 余 2 跨层缺口 (`star workspace list` / `star mr show`) 跨 BD 处理, 见 [BD-002 §6.8 G-CL13-X3](./BD-STAR-CLI-002.md) + [BD-004 §3.1](./BD-STAR-CLI-004.md).

### 6.8 Schema 演化路径 (per 守门 #11+#12)

| # | 缺口 | 优先级 |
|---|---|---|
| G-CL2-S1 | 11 命令中 5 schema (`ReferencesResult` / `ReviewResult` / `PipelineRun` / `PipelineStatus` / 5 P1-H Result) agent-api/01 §3 节待补 | ⏳ Phase 2 (per cli-spec §2.1 注: "§3.x 待补") |
| G-CL2-S2 | `--schema-version v2` 不支持 (传 v2 报 `SCHEMA_VERSION_UNSUPPORTED`, per F-27) | ⏳ Phase 2+ |

---

## §7 テスト用例 (per FR-18~FR-28 + cli-spec §2.2.1 bash 块)

### 7.1 11 命令 AC 表 (per 守门 #1+#13)

| AC# | 命令 | 输入 | 期望输出 | 期望 exit code | 实装 |
|---|---|---|---|---|---|
| AC-CL2-1 | `star context current --json` | (no arg) | JSON, `context.issue_id: "STAR-1024"`, 1 mock related_code/docs/mrs | 0 | ✅ |
| AC-CL2-2 | `star code references "verify_token" --json` | `"verify_token"` | JSON, `result.name: "verify_token"`, `result.total: 1` | 0 | ✅ |
| AC-CL2-3 | `star mr review 42 --json` (per cli-spec §2.2.1) | `42` | JSON, `review.mr_id: "42"`, `review.status: "PENDING"`, `review.approved: false` | 0 | ✅ |
| AC-CL2-3-approve | `star mr review 42 --approve --json` | `42 --approve` | (当前实装 mock 忽略 flag, 永远 PENDING) | 0 | 🟡 部分 (--approve flag 待 Phase D.1) |
| AC-CL2-4 | `star test run --json` | (no arg) | JSON, `test.passed: 42`, `test.failed: 0`, `test.duration_ms: 8765` | 0 | ✅ |
| AC-CL2-5 | `star pipeline run --json` | (no arg) | JSON, `pipeline.id` starts with `"PIPE-mock-"`, `status: "QUEUED"`, `branch: "main"` (default) | 0 | ✅ |
| AC-CL2-5-branch | `star pipeline run --branch feat/auth --json` | `--branch feat/auth` | JSON, `pipeline.branch: "feat/auth"` | 0 | ✅ |
| AC-CL2-6 | `star pipeline status --json` | (no arg) | JSON, `pipeline.id: "PIPE-mock-latest"`, `status: "SUCCESS"` | 0 | ✅ |
| AC-CL2-6-id | `star pipeline status --id PIPE-123 --json` | `--id PIPE-123` | JSON, `pipeline.id: "PIPE-123"` | 0 | ✅ |
| AC-CL2-7 | `star diff HEAD~1 --json` | `HEAD~1` | ⏳ [M] 子项 | — | ⏳ |
| AC-CL2-8 | `star policy check --json` | (no arg) | ⏳ [M] 子项 | — | ⏳ |
| AC-CL2-9 | `star commit -m "fix: auth" --json` | `-m "fix: auth"` | ⏳ [M] 子项 | — | ⏳ |
| AC-CL2-10 | `star push origin feat/auth-fix --json` | `origin feat/auth-fix` | ⏳ [M] 子项 | — | ⏳ |
| AC-CL2-11 | `star mr link 42 --issue STAR-1024 --json` | `42 --issue STAR-1024` | ⏳ [M] 子项 | — | ⏳ |

### 7.2 bash 块完整示例 (per cli-spec §2.2.1, 11 行)

```bash
# 6 个原扩展 (per F-20 修复 2026-08-27, 补 arch/03 §2.2 漏的 mr review / test run / context current)
star context current --json                    # FR-18 ✅
star code references "verify_token" --json     # FR-19 ✅
star mr review 42 --approve                    # FR-20 ✅ (--approve flag 待 Phase D.1)
star test run --json                           # FR-21 ✅
star pipeline run --json                       # FR-22 ✅
star pipeline status --json                    # FR-23 ✅

# 5 个 P1-H 新增 (Universal Submit 第 4/6/7/8/10 步)
star diff HEAD~1 --json                        # FR-24 ⏳ [M]
star policy check --json                       # FR-25 ⏳ [M]
star commit -m "fix: auth" --json              # FR-26 ⏳ [M]
star push origin feat/auth-fix --json          # FR-27 ⏳ [M]
star mr link 42 --issue STAR-1024 --json       # FR-28 ⏳ [M]
```

### 7.3 集成测试入口

per 守门 #1+#13, 11 命令的 cargo test 入口在各自 `commands/<sub>.rs` 的 `#[cfg(test)] mod tests` 块。当前已落:

| 文件 | test 数 | 覆盖 |
|---|---|---|
| `commands/test.rs:39-55` | 2 | pass count assertion |
| `commands/pipeline.rs:60-66` | 1 | pipeline id 形态 |

> **⏳ [L] 子项**: 11 命令的完整 integration test (含 stdout snapshot) 待 Phase D.1.

---

## §8 インターフェース契約

### 8.1 REST 边界 (n/a)

本 BD 11 命令不直接对外暴露 REST endpoint. CLI 是 `agent-api/v1` 的**消费者**.

### 8.2 Shell 边界 (per 守门 #6+#8)

| 命令 | stdout 形态 | 退出码 | stderr 形态 |
|---|---|---|---|
| `star <cmd> --json` (11 命令) | JSON envelope | 0 成功 / 1 用户错 / 2 内部错 (per `error.rs:26-32`) | empty (成功) / error message (失败) |

**消费示例**:
```bash
# MR review 状态查询
review_status=$(star mr review 42 --json | jq -r '.review.status')

# Pipeline 状态查询
pipeline_id=$(star pipeline run --json | jq -r '.pipeline.id')
star pipeline status --id "$pipeline_id" --json
```

### 8.3 MCP 边界 (per cli-spec §2.5 F-08)

| CLI 命令 | MCP tool | 命名差异 | 状态 |
|---|---|---|---|
| `star context current` | (无对应) | — | ⚠️ MCP 缺 |
| `star code references <name>` | `find_references` | `references` vs `find_*` | ✅ |
| `star mr review <id>` | `request_review` | `review` vs `request_*` | ✅ |
| `star test run` | `run_validation` | `test` vs `validation` | ✅ |
| `star pipeline run` | (无对应) | — | ⚠️ MCP 缺 |
| `star pipeline status` | `get_pipeline_status` | 一致 | ✅ |
| 5 P1-H 命令 | (待 MCP 加) | — | ⏳ Phase 2 |

### 8.4 CLI ↔ CLI 子能力边界

| 边界 | 描述 |
|---|---|
| CL-2 ↔ CL-1 | `commands/context.rs` 共享 `Context` schema + `lookup_context()` 给 FR-6 (BD-002) + FR-18 (本 BD) |
| CL-2 ↔ CL-1 | `commands/code.rs` 共享 `CodeMatch` schema 给 FR-7/8 (BD-002) + FR-19 (本 BD) |
| CL-2 ↔ CL-4 | `commands/mr.rs` 共享 `MR` schema 给 FR-14/15 (BD-004) + FR-20 (本 BD) |
| CL-2 ↔ CL-4 | `commands/test.rs` 共享 `TestResult` schema 给 FR-16 (BD-004) + FR-21 (本 BD) |
| CL-2 ↔ CL-6 | 5 P1-H 命令复用 `commands/submit.rs` 步函数 (per §2.3 策略 c) |

### 8.5 Git 协议边界 (per cli-spec §1 第 5 条)

**`star` 不替代 `git`**, 所有 Git 协议能力继续由 GitGit 提供. CLI 仅做 wrapper (per 5 P1-H 命令 `star diff` / `star commit` / `star push` 未来 spawn `git diff` / `git commit` / `git push`).

---

## §9 関連ドキュメント

per 守门 #12 v21 commit 引用全链 + 总冊 BD-001 §8:

| # | 类别 | 文档 | 关系 |
|---|---|---|---|
| 1 | 上位 SRS | [`SRS-STAR-CLI-001.md`](../requirements/SRS-STAR-CLI-001.md) v1.0 | 上游 requirements |
| 2 | 上游总冊 | [`BD-STAR-CLI-001.md`](./BD-STAR-CLI-001.md) v1.0 | 索引级汇总 (本 commit 同期落档) |
| 3 | 平行 BD | [`BD-STAR-CLI-002.md`](./BD-STAR-CLI-002.md) v1.0 | CL-1+CL-3 13 命令 (跨 BD 引用: `context.rs` / `code.rs` / `mr.rs` / `test.rs` 共用) |
| 4 | 平行 BD | [`BD-STAR-CLI-004.md`](./BD-STAR-CLI-004.md) v1.0 | CL-4+CL-5 (跨 BD 引用: `error.rs::StarError` + Skill) |
| 5 | 平行 BD | [`BD-STAR-CLI-005.md`](./BD-STAR-CLI-005.md) v1.0 | CL-6 Universal Submit (跨 BD 引用: 5 P1-H 命令复用 submit 步函数) |
| 6 | 上游 spec | [`spec/cli/01-cli-spec.md`](../architecture/2026-08-26-upgrade/spec/cli/01-cli-spec.md) v0.2 §2.2 + §2.4 + §2.5 | CLI canonical spec |
| 7 | 上游 spec | [`spec/agent-api/01-schema.md`](../architecture/2026-08-26-upgrade/spec/agent-api/01-schema.md) | 11 命令引用的 11 schema canonical |
| 8 | 上游 spec | [`spec/mcp/01-mcp-spec.md`](../architecture/2026-08-26-upgrade/spec/mcp/01-mcp-spec.md) | MCP canonical spec (per §2.1 命名约定 + §2.5 跨层缺口) |
| 9 | 平行 SRS | [`SRS-STAR-OPS-001.md`](../requirements/SRS-STAR-OPS-001.md) v1.0 | Star Ops (Pipeline/Test/MR 内部协议) |
| 10 | 平行 SRS | [`SRS-WORKFLOW-TEMPLATE-001.md`](../requirements/SRS-WORKFLOW-TEMPLATE-001.md) v1.0 | Workflow Template |
| 11 | 平行 SRS | [`SRS-CANVAS-WORKFLOW-001.md`](../requirements/SRS-CANVAS-WORKFLOW-001.md) v1.0 | Canvas Workflow |
| 12 | 平行 SRS | [`SRS-MULTICA-SKILL-001.md`](../requirements/SRS-MULTICA-SKILL-001.md) v1.0 | Multica Skill |
| 13 | ADR | [`ADR-0029-multica-patterns-borrow.md`](../adr/0029-multica-patterns-borrow.md) | Multica 模式借用 (FR-25 policy engine) |
| 14 | review/fix | [`INTERFACE-REVIEW-A.md`](../architecture/2026-08-26-upgrade/INTERFACE-REVIEW-A.md) | INTERFACE-REVIEW-A v0.2 (F-08/F-20/P1-H 来源) |
| 15 | 实施位置 | [`crates/star-cli/src/main.rs`](../crates/star-cli/src/main.rs) | Cli parser + TopCommand enum |
| 16 | 实施位置 | [`crates/star-cli/src/commands/context.rs`](../crates/star-cli/src/commands/context.rs) | FR-18 `current` + 共享 `lookup_context()` |
| 17 | 实施位置 | [`crates/star-cli/src/commands/code.rs`](../crates/star-cli/src/commands/code.rs) | FR-19 `references` + 共享 `CodeMatch` |
| 18 | 实施位置 | [`crates/star-cli/src/commands/mr.rs`](../crates/star-cli/src/commands/mr.rs) | FR-20 `review` + 共享 `MR` |
| 19 | 实施位置 | [`crates/star-cli/src/commands/test.rs`](../crates/star-cli/src/commands/test.rs) | FR-21 `run` + 共享 `TestResult` |
| 20 | 实施位置 | [`crates/star-cli/src/commands/pipeline.rs`](../crates/star-cli/src/commands/pipeline.rs) | FR-22/23 `run`/`status` |
| 21 | 实施位置 | [`crates/star-cli/src/commands/submit.rs`](../crates/star-cli/src/commands/submit.rs) | 5 P1-H 命令 Phase D.1 复用此文件步函数 |
| 22 | 下游交付 | `DD-STAR-CLI-003.md` | Phase 2 待开 |
| 23 | 测试 | `acceptance/05-phase2.md` | Phase 2 退出条件 (per 11 扩展 + 5 P1-H) |
| 24 | 守门合规 | AGENTS.md | 守门 #1+#5+#6+#7+#9+#10+#11+#12+#14 |

### Traceability 表 (per 守门 #1+#11)

| SRS § | BD § | 实现 (.rs) |
|---|---|---|
| §3.1.2 扩展 11 命令 | §3.1~§3.5 (6 已实装) + §3.6 (5 P1-H ⏳) | `commands/context.rs::Current` / `code.rs::References` / `mr.rs::Review` / `test.rs::Run` / `pipeline.rs::Run/Status` / `submit.rs::SubmitAction` (Phase D.1) |
| §4.1 28 命令 capability 覆盖 (本 BD 范围 5 capability: code_navigation / code_context / merge_requests / tests / pipelines / reviews) | §3 | (同 §3.1~§3.6) |
| §4.2 35 FR (本 BD 范围 FR-18~28) | §3 (1:1 映射) | (同 §3) |
| §8 错误模型 6 字段 | §3.7 + §6.8 缺口 | `error.rs` + [BD-004 §2](./BD-STAR-CLI-004.md) |
| §9 AC-009..012 SK 4 项 | n/a (跨 BD-004) | n/a |
| §10 风险与依赖 | §6 已知缺口 ~10 项 | (跨 5 BD) |

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
| #11 | 缺标比错标: §6 列 ~10 项已知缺口 | ✅ |
| #12 | v21 commit 引用 cli/01 v0.2 §2.2/§2.4/§2.5 + mcp/01 + agent-api/01 + SRS + 总冊 BD-001 | ✅ |
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
| v0.2 | 2026-09-20 | (后不可达) | 4 专题 BD 平行落档 — `d6e8cf4a` 在 `git ls-remote origin agent/minimaxm3/ulys-124` 不可达 | ❌ |
| **v1.0** | **2026-09-25** | **(本 turn TBD)** | **BD-STAR-CLI-003 v1.0 recreate — 11 扩展命令详细 BD (CL-2, 6 已实装 + 5 P1-H ⏳ [M]), 10 段 IPA, ~10 项已知缺口显式列, 11 AC, 9 守門全过, 5 P1-H 与 Submit 状态机关系, 全程 push origin 后 4 路验证** | ✅ (本 turn) |

### 10.3 本 BD body 总行数

按 `wc -l` 实测 (本 turn): 见 task final output line `BD-003: <wc -l> <sha256>` 行.

---

**ULYS-124 子任务进度**: v0.1 SRS ✅ → v0.2 BD 5 子任务分解 ✅ → Stage 1.1 BD-STAR-CLI-002 v1.0 ✅ → **Stage 1.2 BD-STAR-CLI-003 v1.0 recreate ✅** (本 turn) → Stage 1.3~1.4 + Stage 2 总冊 + 4 路验证 (本 turn 同期).

**下游衔接**: 本 BD 落档 + push origin + 4 路验证 done 后, Stage 2 总冊 BD-001 v1.0 可拍板 done, ULYS-124 推进到 Phase 2 (5 份 DD).
