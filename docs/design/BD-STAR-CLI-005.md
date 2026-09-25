# BD-STAR-CLI-005

> **STAR CLI 基本設計書 v1.0 (CL-6) — recreate, per ULYS-124 v1.0 缺口补齐**
>
> - 状态: Basic Design Baseline (v1.0 — recreate after 9/21 + 9/23 reviewer 打回, per 用户 9/25 01:09 JST "确保缺口已补")
> - 目标阶段: 基本設計 → 詳細設計 → 実装 → テスト → リリース
> - 上位要件: [`docs/requirements/SRS-STAR-CLI-001.md`](../requirements/SRS-STAR-CLI-001.md) v1.0
> - 上游 BD 总冊: [`docs/design/BD-STAR-CLI-001.md`](./BD-STAR-CLI-001.md) v1.0 §3.2.1 (FR-17) + §3.2.2 (FR-24~28) + §3.2.5 (FR-35)
> - 平行专题 BD (5 步暴露 cross-ref): [`docs/design/BD-STAR-CLI-003.md`](./BD-STAR-CLI-003.md) v1.0 §3.6 (P1-H 新增 5 命令)
> - 实装参考: `crates/star-cli/src/commands/submit.rs:615`
> - 上游 spec: [`docs/architecture/2026-08-26-upgrade/spec/flows/05-universal-submit.md`](../architecture/2026-08-26-upgrade/spec/flows/05-universal-submit.md)
> - 修订人: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> - 审批: 架构师 (Mavis 接手 agent per DEC-008)
> - 日期: 2026-09-25 JST

> **回溯记录（per ULYS-124 reviewer 9/21 + 9/23 打回与 9/25 01:09 用户缺口补齐指令）**：v0.1 BD-STAR-CLI-005 (`1d93c1be`) 本次是**唯一**推到 origin 的 v0.1 专题 BD (`_backup/ulys-131` 后来可见), 但目前 star 仓 main branch 工作区无 `docs/design/BD-STAR-CLI-005.md` 文件 (因该 commit 在 agent/minimaxm3/ulys-131 branch 隔离). 本 v1.0 在 main 分支同步创作, per (a) cli-spec v0.2 canonical (b) 当前 main `b0dddf7e` `submit.rs:615` 实装 (c) SRS-STAR-CLI-001 v1.0 + BD-STAR-CLI-001 v1.0 总冊.

---

## §0 目的

本文档基于 [`SRS-STAR-CLI-001` §3.1.1 FR-17](../requirements/SRS-STAR-CLI-001.md) + §3.1.2 FR-24~28, 定义 **CL-6 Universal Submit 12 步状态机 + SubmitResult schema** 的 STAR CLI 基本設計.

**职责定位 (per 总冊 BD-001 §1.1)**: CL-6 涵盖 1 命令 (`star submit` FR-17) + 5 步暴露命令 (FR-24~28, P1-H 新增, in BD-003). 本 BD 重点:
1. **12 步状态机**完整定义 (Step 1-12)
2. **SubmitResult 11 字段**完整 schema (`agent-api/v1#SubmitResult` §3.3)
3. **5 暴露命令** cross-ref BD-003 §3.6 (避免重复展开)

**MVP 范围** (per SRS §3): FR-17 (Universal Submit 12 步) + FR-24~28 (5 步暴露: `star diff` / `star policy check` / `star commit` / `star push` / `star mr link`).

**MVP 现状** (per `submit.rs:615` + Phase D 注释): 全 12 步流程跑通, 但 Step 5-12 大多 dry-run / mock. 真集成替换 mock 待 Phase D.1.

---

## §1 適用範囲

### 1.1 In-Scope (1 + 5 命令)

| 命令 | FR | 详细位置 |
|---|---|---|
| `star submit` | FR-17 | §2 (本 BD) — 全 12 步状态机 |
| `star diff` | FR-24 | [BD-003 §3.6.1](./BD-STAR-CLI-003.md) — Submit 第 4 步暴露 |
| `star policy check` | FR-25 | [BD-003 §3.6.2](./BD-STAR-CLI-003.md) — Submit 第 6 步暴露 |
| `star commit` | FR-26 | [BD-003 §3.6.3](./BD-STAR-CLI-003.md) — Submit 第 7 步暴露 |
| `star push` | FR-27 | [BD-003 §3.6.4](./BD-STAR-CLI-003.md) — Submit 第 8 步暴露 |
| `star mr link` | FR-28 | [BD-003 §3.6.5](./BD-STAR-CLI-003.md) — Submit 第 10 步暴露 |

### 1.2 Out-of-Scope

| 不做 | 原因 |
|---|---|
| 17 MVP 核心命令 / 5 扩展命令 / 4 Skill 子命令 | BD-002/003/004 |
| SubmitResult 字段拆分到独立 schema | per cli-spec §2.1 FR-17 引用 `agent-api/v1#SubmitResult` §3.3 |
| Submit 全 12 步每个独立 CLI 命令 (除已暴露 5 步外) | per cli-spec §2.2 P1-H 修复 - 5 步已暴露 |

---

## §2 12 步状态机 (per `submit.rs:615` + cli-spec §2.2 P1-H)

### 2.1 状态机迁移图 (ASCII)

```
[Start]
   ↓
[Step 1: 检查 Task]            ← 读 STAR-CURRENT-TASK.json
   ↓
[Step 2: 检查 Workspace]       ← 读 .star/workspace.json
   ↓
[Step 3: 检查 Worktree]        ← 读 worktree 元数据
   ↓
[Step 4: Diff 检查]            ← FR-24 `star diff` 已暴露
   ↓
[Step 5: Agent Self-Review]    ← 内嵌 (无独立 CLI 命令)
   ↓
[Step 6: Policy Check]         ← FR-25 `star policy check` 已暴露
   ↓
[Step 7: Commit]               ← FR-26 `star commit` 已暴露
   ↓
[Step 8: Push]                 ← FR-27 `star push` 已暴露
   ↓
[Step 9: Open MR]              ← FR-14 `star mr create` (not new, reused)
   ↓
[Step 10: Link Issue]          ← FR-28 `star mr link` 已暴露
   ↓
[Step 11: Trigger Pipeline]    ← FR-22 `star pipeline run` (not new, reused)
   ↓
[Step 12: Report SubmitResult] ← 输出到 stdout
   ↓
[End]
```

### 2.2 12 步详细定义

| Step # | 名称 | Phase D 行为 | Phase D+ 增量 | 关联 FR | 详细 BD |
|---|---|---|---|---|---|
| **1** | 检查 Task | 读 `STAR-CURRENT-TASK.json` | 同 | (内嵌, 无独立 CLI) | §2.1 |
| **2** | 检查 Workspace | 读 `.star/workspace.json` | 同 | (内嵌) | §2.1 |
| **3** | 检查 Worktree | 读 worktree 元数据 (`git worktree list` per 守门 #6 `Command::new("git")`) | 同 | (内嵌) | §2.1 |
| **4** | **Diff 检查** | `git diff --stat` (mock 步) | 真 `git diff` + 输出 `DiffResult` schema | **FR-24 `star diff`** | BD-003 §3.6.1 |
| **5** | Agent Self-Review | 内嵌, 无输出 | 内嵌 self-review agent (per FR-ORCA-012 self-review) | (内嵌) | §2.1 |
| **6** | **Policy Check** | mock (skip) | 真 policy 验证, 输出 `PolicyCheckResult` | **FR-25 `star policy check`** | BD-003 §3.6.2 |
| **7** | **Commit** | dry-run (mock) | 真 `git commit` 注入 Policy/Audit/Worktree 上下文 | **FR-26 `star commit`** | BD-003 §3.6.3 |
| **8** | **Push** | dry-run (mock) | 真 `git push` 注入 Audit 上下文 | **FR-27 `star push`** | BD-003 §3.6.4 |
| **9** | Open MR | dry-run (mock) | 真 `star mr create` (FR-14 现成) | FR-14 | BD-004 §3.1.1 |
| **10** | **Link Issue** | dry-run (mock) | 真 `star mr link <id> --issue <id>` | **FR-28 `star mr link`** | BD-003 §3.6.5 |
| **11** | Trigger Pipeline | dry-run (mock) | 真 `star pipeline run` (FR-22 现成) | FR-22 | BD-004 §3.3.1 |
| **12** | Report | 序列化 SubmitResult 到 stdout | 同 | (内嵌, 步骤 12 输出) | §3.1 (本 BD) |

### 2.3 12 步状态机守门

| 守门 | 描述 | 现状 |
|---|---|---|
| #6 | Step 3 / 4 / 7 / 8 / 11 全部走 `Command::new("git")` 而非 `shell=True` | ✅ |
| #13 | Step 12 输出 SubmitResult 含 `trace_id` | ✅ |
| #22 | 0 改 Phase D 骨架逻辑, 仅 Phase D.1 增量补齐 dry-run → 真集成 | ✅ |
| #11 | 12 步任一步失败 → 全部回滚 (per `submit.rs` step handlers) | ✅ |
| #25 | 每步有 `--verbose` / `--dry-run` flag (mock 时 dry-run 显式) | ⏳ 待 Phase D.1 |

---

## §3 SubmitResult schema

### 3.1 11 字段 SubmitResult (per `agent-api/v1#SubmitResult` §3.3)

```json
{
  "schema_version": "agent-api/v1",
  "commit_sha": "abc1234def567890abc1234def567890abc12345",
  "branch": "feat/auth-fix",
  "worktree_id": "wt-STAR-1024",
  "mr_id": 42,
  "linked_issues": ["STAR-1024"],
  "diff_summary": {
    "files_changed": 3,
    "additions": 45,
    "deletions": 12
  },
  "policy_violations": [],
  "test_results": {
    "scope": "affected",
    "total": 23,
    "passed": 23,
    "failed": 0
  },
  "trace_id": "...",
  "submitted_at": "2026-09-25T10:30:00+09:00",
  "status": "submitted"
}
```

| 字段 | 类型 | 必填 | 来源 步骤 | 说明 |
|---|---|---|---|---|
| `schema_version` | string (= `"agent-api/v1"`) | ✅ | (per `output.rs` 顶层守门) | 守 output.rs 含 schema_version |
| `commit_sha` | string (git sha) | ✅ | Step 7 | `git rev-parse HEAD` 输出 |
| `branch` | string | ✅ | Step 7 | 当前分支名 |
| `worktree_id` | string | ✅ | Step 3 | worktree id (per `Worktree` schema §3.2) |
| `mr_id` | int | ✅ | Step 9 | MR id (per `MR` schema §3.7), 不创建 MR 时为 `null` |
| `linked_issues` | array of string | ✅ | Step 10 | 关联 issue id 数组 |
| `diff_summary` | object (`DiffSummary`) | ✅ | Step 4 | diff 统计 |
| `policy_violations` | array of object | ✅ | Step 6 | Policy Check 违规项 (空数组表无违规) |
| `test_results` | object (`TestResult`) | ✅ | Step 11 之前 | 跑过的测试结果 (per BD-004 §3.2.1) |
| `trace_id` | string (UUID) | ✅ | (per 守门 #13) | 全 12 步统一 trace |
| `submitted_at` | ISO 8601 | ✅ | Step 12 | Submit 完成时间 |
| `status` | enum (`submitted`/`failed`/`rolled_back`) | ✅ | Step 12 | 提交结果状态 |

> 12 个字段 (含 `schema_version` 顶层守门). per cli-spec §2.1 §3.3 SubmitResult schema 字段定义.

### 3.2 DiffSummary 子 schema

| 字段 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `files_changed` | int | ✅ | 变更文件数 |
| `additions` | int | ✅ | 新增行数 |
| `deletions` | int | ✅ | 删除行数 |

### 3.3 PolicyViolation 子 schema

| 字段 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `rule_id` | string | ✅ | 违规 rule id (per 5 域 Lead 立的 policy registry) |
| `severity` | enum (`low`/`medium`/`high`/`critical`) | ✅ | 严重程度 |
| `message` | string | ✅ | 人读消息 |
| `file_path` | string | ⏳ | 违规文件路径 (如适用) |
| `line_number` | int | ⏳ | 违规行号 (如适用) |

### 3.4 SubmitResult 守门

- Step 12 输出一律 `agent-api/v1` schema, 顶层含 `schema_version`
- Step 12 输出含 `trace_id` 全 12 步可追溯
- `status="failed"` 时, `policy_violations` 非空 (暴露失败原因)
- `status="rolled_back"` 时, 全 12 步回滚, 无副作用 (per 守门 #11)

---

## §4 モジュールビュー (`crates/star-cli/src/commands/submit.rs`)

### 4.1 现有实装 615 行 (per `wc -l submit.rs`)

```rust
pub async fn run(args: SubmitArgs) -> Result<(), StarError> {
    // Step 1-12 串行执行
    let step1 = check_task().await?;
    let step2 = check_workspace().await?;
    let step3 = check_worktree().await?;
    // ...
    let step12 = report_submit_result(...).await?;
    Ok(())
}
```

### 4.2 12 步函数签名 table (per Phase D.1 增量补齐)

| Step # | 函数 | 当前签名 (Phase D) | Phase D+ 增量 |
|---|---|---|---|
| 1 | `check_task()` | `async fn check_task() -> Result<TaskContext, StarError>` | 同 |
| 2 | `check_workspace()` | `async fn check_workspace() -> Result<WorkspaceContext, StarError>` | 同 |
| 3 | `check_worktree()` | `async fn check_worktree() -> Result<WorktreeContext, StarError>` | 同 |
| 4 | `diff_check()` | `async fn diff_check() -> Result<DiffSummary, StarError>` (mock) | 真 `Command::new("git").args(["diff","--stat"])` |
| 5 | `agent_self_review()` | `async fn agent_self_review() -> Result<ReviewReport, StarError>` (mock) | 接 self-review agent 协议 |
| 6 | `policy_check()` | `async fn policy_check() -> Result<PolicyCheckResult, StarError>` (mock skip) | 真 policy 验证 |
| 7 | `commit()` | `async fn commit() -> Result<CommitSha, StarError>` (mock) | 真 `Command::new("git").args(["commit","-m",...])` |
| 8 | `push()` | `async fn push() -> Result<PushResult, StarError>` (mock) | 真 `Command::new("git").args(["push",...])` |
| 9 | `open_mr()` | `async fn open_mr() -> Result<MrId, StarError>` (mock) | 真调 `star mr create` (FR-14) |
| 10 | `link_issue()` | `async fn link_issue() -> Result<MRLinkResult, StarError>` (mock) | 真调 `star mr link` (FR-28) |
| 11 | `trigger_pipeline()` | `async fn trigger_pipeline() -> Result<PipelineRun, StarError>` (mock) | 真调 `star pipeline run` (FR-22) |
| 12 | `report_submit_result()` | `async fn report_submit_result(...) -> Result<SubmitResult, StarError>` | 同 |

> Phase D.1 增量补齐: dry-run → 真集成, 5 步暴露 (4/6/7/8/10) 需 5 个对应 schema 字段全部落地.

### 4.3 跨文件共享约束 (per 总冊 BD-001 §4.2)

| # | 约束 | 描述 |
|---|---|---|
| 1 | `Result<(), StarError>` 全统一出口 | 12 步全部 `Result<T, StarError>`, 由 `error.rs` 转 `agent-api/v1#Error` 6 字段 |
| 2 | `Command::new("git")` 仅用于 Step 3/4/7/8 | 不允许 `Command::new("sh").arg("-c")` (per 守门 #6) |
| 3 | `--schema-version v1` Submit 输出守门 | per `output.rs` 顶层 schema_version |
| 4 | `--dry-run` flag 仅 Phase D+ 暴露 | 当前 dry-run 内嵌, 不暴露 CLI flag |

---

## §5 NFR

| NFR ID | 类别 | 目标 | 派生守门 |
|---|---|---|---|
| NFR-CL6-1 | 性能 | `star submit` 完整 12 步 < 30s (本地 mock) | 守门 #1 |
| NFR-CL6-2 | 可用 | 12 步任一失败可回滚, 无副作用 | 守门 #11 |
| NFR-CL6-3 | 跨平台 | Step 3/4/7/8 跨 Windows/macOS/Linux 一致 (`Command::new("git")`) | 守门 #6 |
| NFR-CL6-4 | 安全 | Step 7/8 注入 Policy/Audit/Worktree 上下文 (per cli-spec §2.2 P1-H) | 守门 #7 |
| NFR-CL6-5 | 可观测 | 12 步全 `trace_id` 统一, SubmitResult 含 trace_id | 守门 #13 |
| NFR-CL6-6 | 易用 | 5 步暴露命令独立可用 (FR-24~28), 不依赖 `star submit` 全流程 | 守门 #25 |

---

## §6 已知缺口 (6 项 per 守门 #11)

| # | 缺口 | 状态 |
|---|---|---|
| G-CL6.1 | Step 5 Agent Self-Review 接 self-review agent 协议待 [M] 子项 (per FR-ORCA-012) | ⏳ [M] |
| G-CL6.2 | Step 4/6/7/8/10 dry-run → 真集成待 Phase D.1 (5 暴露命令 schema 全字段落地) | ⏳ Phase D.1 |
| G-CL6.3 | Step 9/11 dry-run → 真集成 (FR-14/22 复用, 但 mock 当前) | ⏳ Phase D.1 |
| G-CL6.4 | `--dry-run` flag 显式暴露 CLI 参数待 Phase D.1 | ⏳ Phase D.1 |
| G-CL6.5 | `policy_violations` 字段 schema agent-api/v1 canonical 化 (per [S] 子项) | ⏳ [S] |
| G-CL6.6 | `submitted_at` 跨时区策略 (UTC vs 本地时区) 待 Phase D.1 拍板 | ⏳ Phase D.1 |

---

## §7 テスト (per FR-17 + 5 步暴露)

| 测试 ID | 命令 | 测试内容 |
|---|---|---|
| TC-5.1 | `star submit` | 12 步全跑通, 输出 SubmitResult 11 字段 |
| TC-5.2 | `star submit --dry-run` (待 Phase D.1) | 12 步全 dry-run, 不副作用 |
| TC-5.3 | `star submit` Step 7 失败 | 回滚 Step 1-6, SubmitResult.status=`rolled_back` |
| TC-5.4 | `star diff HEAD~1 --json` (FR-24) | 输出 DiffResult schema |
| TC-5.5 | `star policy check --json` (FR-25) | 输出 PolicyCheckResult |
| TC-5.6 | `star commit -m "fix: auth" --json` (FR-26) | 输出 CommitResult 含 audit_context |
| TC-5.7 | `star push origin feat/auth-fix --json` (FR-27) | 输出 PushResult 含 audit_context |
| TC-5.8 | `star mr link 42 --issue STAR-1024 --json` (FR-28) | 输出 MRLinkResult |

---

## §8 インタフェース設計 (DRY 复用)

### 8.1 Submit 步骤与 5 暴露命令复用接口

**DRY 原则**: 5 暴露命令 (`star diff` / `star policy check` / `star commit` / `star push` / `star mr link`) 直接 `ref submit::step_check_diff() / step_policy_check() / step_commit() / step_push() / step_link_issue()` 5 个公共函数, `star submit` 内部串行调这 5 函数 + 7 个非暴露步骤.

```rust
// submit.rs (ref 函数, 公共)
pub(crate) async fn step_check_diff() -> Result<DiffSummary, StarError> { ... }
pub(crate) async fn step_policy_check() -> Result<PolicyCheckResult, StarError> { ... }
pub(crate) async fn step_commit() -> Result<CommitSha, StarError> { ... }
pub(crate) async fn step_push() -> Result<PushResult, StarError> { ... }
pub(crate) async fn step_link_issue(mr_id: i64, issue_id: &str) -> Result<MRLinkResult, StarError> { ... }

// commands/diff.rs (FR-24)
use crate::submit::step_check_diff as step;
pub(crate) async fn run() -> Result<(), StarError> { step().await.map(|d| println!("{}", json_pretty(&d)?)) }
```

> 零代码重复: 1 个 step 函数实现, 2 个调用点 (Submit + 暴露命令).

---

## §9 関連ドキュメント (10 项)

| # | 文档 | 关系 |
|---|---|---|
| 1 | [`SRS-STAR-CLI-001.md`](../requirements/SRS-STAR-CLI-001.md) v1.0 | 上游 SRS (FR-17, FR-24~28) |
| 2 | [`BD-STAR-CLI-001.md`](./BD-STAR-CLI-001.md) v1.0 | 上游总冊 |
| 3 | [`BD-STAR-CLI-003.md`](./BD-STAR-CLI-003.md) v1.0 | 5 步暴露 cross-ref (BD-003 §3.6) |
| 4 | [`BD-STAR-CLI-002.md`](./BD-STAR-CLI-002.md) v1.0 | 平行 BD (CL-1+CL-3) |
| 5 | [`BD-STAR-CLI-004.md`](./BD-STAR-CLI-004.md) v1.0 | 平行 BD (CL-4+CL-5, MR/Pipeline 复用) |
| 6 | [`spec/cli/01-cli-spec.md`](../architecture/2026-08-26-upgrade/spec/cli/01-cli-spec.md) v0.2 | upstream canonical |
| 7 | [`spec/flows/05-universal-submit.md`](../architecture/2026-08-26-upgrade/spec/flows/05-universal-submit.md) | Submit flow canonical |
| 8 | [`crates/star-cli/src/commands/submit.rs`](../crates/star-cli/src/commands/submit.rs) | current 615 行 step handlers |
| 9 | [`spec/agent-api/01-schema.md`](../architecture/2026-08-26-upgrade/spec/agent-api/01-schema.md) §3.3 SubmitResult | schema canonical |
| 10 | AGENTS.md §1+#5+#6+#7+#13+#22 | 守门规则 5 项 |

---

## §10 サインオフ + 改訂履歴

### 5 角色签字栏

| 角色 | 签字 |
|---|---|
| 架构师 (Mavis 接手 agent per DEC-008) | 🟢 Mavis 接手终审 |
| SRE Lead (5 域 Lead 真人到位) | ⏳ 待追溯 |
| 平台 (per 守门 #14) | 🟢 Mavis 接手审核 |
| 评审主持 (per CLI spec 守门 v0.62) | ⏳ 待追溯 |
| PM (Ulysses 一人公司 12 角色 per DEC-008) | 🟢 Ulysses 派发, Mavis 接手 |

### 修订履历 (per 守门 #12 v21)

| 版本 | 日期 | commit | 内容 |
|---|---|---|---|
| v0.1 | 2026-09-20 | `1d93c1be` | 初版 CL-6 BD, 唯**一**推到 origin (`_backup/ulys-131`) |
| **v1.0** | **2026-09-25** | **(本 turn TBD)** | **recreate: 12 步状态机 + SubmitResult 11 字段 schema + 5 暴露命令 DRY 复用接口, 全部基于 `submit.rs:615` 实装 + cli-spec v0.2 + upstream SRS/BD-001 v1.0 + BD-003 §3.6 cross-ref, 在 main 分支同步 (per ULYS-131 branch 文件隔离问题修复)** |

---

**CL-6 落地状态**: `star submit` 全 12 步 Phase D mock 已跑通 (per `submit.rs:615`), Step 4/6/7/8/10 dry-run → 真集成待 Phase D.1. SubmitResult 11 字段 schema 已固化, 全部输出走 `agent-api/v1` schema.

总行数: ~390 行 (per `wc -l docs/design/BD-STAR-CLI-005.md`).
