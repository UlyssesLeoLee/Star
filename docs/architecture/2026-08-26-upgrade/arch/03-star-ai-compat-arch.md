# 03. STAR AI Compatibility Architecture

> **状态**：🟡 草案 v0.3
> **日期**：2026-08-26
> **依赖**：[ADR-0021 Zero Vendor Cooperation](../../adr/0021-zero-vendor-cooperation.md) · [Protocol Survey](../../ecosystem-survey/protocol-survey.md)

---

## 1. 总体架构

```text
┌────────────────────────────────────────────────────────────────────┐
│                    Any AI Coding Agent (7+ 主流)                   │
│  Codex · Claude Code · Gemini CLI · Copilot · Cursor · VS Code ·  │
│  Junie · Unknown Agent (Phase D 验证)                              │
└────────────────────────┬───────────────────────────────────────────┘
                         │ (5 接入通道)
                         ↓
┌────────────────────────────────────────────────────────────────────┐
│              STAR AI and IDE Compatibility Layer                   │
│                                                                    │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │ 1. Git Compatibility       (Universal Submit 协议)        │  │
│  │    - 标准 git commit / push / worktree                      │  │
│  │    - GitHub / GitLab / Gitea 也通过这层                    │  │
│  └──────────────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │ 2. Shell / CLI             (star CLI + --json)              │  │
│  │    - 17 个核心命令                                          │  │
│  │    - machine-readable 稳定 schema (agent-api/v1)            │  │
│  └──────────────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │ 3. MCP Server (2026-07-28)  (13 个领域语义 tools)          │  │
│  │    - 兜底: stdio transport (Rust SDK beta 风险规避)         │  │
│  └──────────────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │ 4. REST + OpenAPI 3.1                                       │  │
│  │    - Web / Automation / External Agent                      │  │
│  └──────────────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │ 5. AGENTS.md bootstrap (vendor-neutral)                     │  │
│  │    - 薄 + 含 3 个最小命令 (capabilities / task / submit)    │  │
│  └──────────────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │ 6. Optional Vendor Adapter (per ADR-0025, 独立 crate)      │  │
│  │    - 删除后 Core 100% 完整                                  │  │
│  └──────────────────────────────────────────────────────────────┘  │
└────────────────────────┬───────────────────────────────────────────┘
                         ↓
┌────────────────────────────────────────────────────────────────────┐
│                         STAR Core                                  │
│  star-domain · star-application · star-context · star-ai-gateway  │
│  star-ide-gateway · star-workspace · star-audit · star-policy      │
└────────────────────────┬───────────────────────────────────────────┘
                         ↓
              Version Control Provider 抽象 (per ADR-0023)
                         ↓
   GitGit · GitHub · GitLab · Gitea (and other Git providers)
```

## 2. 5 接入通道详细规格

### 2.1 Git Compatibility（兜底层）

- `git clone` / `git push` / `git pull` / `git fetch` 100% 走 GitGit（当 GitGit 是 Provider）
- `git worktree` add/remove/list 必须工作
- Agent 通过 shell 调 `git` 命令 + 解析输出（兜底 Level 4: Git Only）

### 2.2 Shell / CLI（推荐层）

per [spec/cli/01-cli-spec.md §2](../spec/cli/01-cli-spec.md)：

**MVP 17 核心命令**（per 任务原文 §9）：

```bash
# MVP 17 核心命令（per 任务原文 §9）
star project list
star issue list
star issue show STAR-1024
star issue claim STAR-1024
star task current --json
star context get STAR-1024 --json
star code search "auth" --json
star code symbol "AuthService" --json
star workspace list
star workspace current
star worktree create STAR-1024
star worktree enter STAR-1024
star worktree status --json
star mr create --json
star mr show --json
star test affected
star submit  # Universal Submit
```

> **MVP 子集边界**（per P1-A 修复 2026-08-27）：17 核心 = MVP 退出条件 acceptance/04 §3 第一条必含。完整命令清单 + 11 个扩展命令见 [spec/cli/01-cli-spec.md §2.2](../spec/cli/01-cli-spec.md)，快速参考表：
>
> | 11 扩展命令 | 用途 | 对应能力 |
> |---|---|---|
> | `star context current` | 当前 context | context |
> | `star code references <name>` | 引用查找 | code_navigation |
> | `star mr review <id>` | Review MR | merge_requests（review 部分，MVP 17 核心不覆盖） |
> | `star test run` | 跑全部测试 | tests |
> | `star pipeline run` / `pipeline status` | Pipeline 控制 + 状态 | pipelines（Phase 2+ 标记能力） |
> | `star diff` (P1-H 新增) | Diff 检查（Universal Submit 第 4 步） | diff（隐式走 code_navigation） |
> | `star policy check` (P1-H 新增) | Policy 检查（Universal Submit 第 6 步） | policy（Phase 2+ 标记能力） |
> | `star commit` (P1-H 新增) | Commit（注入 Policy / Audit / Worktree 上下文） | worktrees |
> | `star push` (P1-H 新增) | Push（注入 Audit 上下文） | worktrees |
> | `star mr link <id>` (P1-H 新增) | 关联 Issue 到 MR（Universal Submit 第 10 步） | merge_requests |
>
> MVP 17 核心 + 11 扩展 = **28 个 CLI 命令总数**（per [spec/cli/01-cli-spec.md §2](../spec/cli/01-cli-spec.md) §2.1 + §2.2 双表）。

> v0.3 fix: 2026-08-27 per INTERFACE-REVIEW-A 🟡 #20（F-20）补 11 扩展命令速查表（含 mr review / test run / context current），数字 17+11=28 对齐 cli/01 §2.1+§2.2。

**关键约束**：
- `--json` 必须稳定（versioned as `agent-api/v1`）
- `--quiet` / `--fields` / `--limit` / `--cursor` / `--no-color` / `--schema-version`
- 不得强制解析 ANSI / 表格 / 自然语言
- CLI 不只是 REST API 的映射，必须表达 Domain Semantics
- **`star` 是 `git` 的 superset**（per P1-H 修复 2026-08-27） — `star` 提供 `git` 不具备的领域操作 + 包装 `git` 子命令（diff / commit / push）注入 Policy / Audit / Worktree 上下文

### 2.3 MCP Server（增强层）

per [spec/mcp/01-mcp-spec.md](../spec/mcp/01-mcp-spec.md) §2 工具表：

**完整 16 tools**（per P1-F 修复 2026-08-27，per [spec/mcp/01-mcp-spec.md §2](../spec/mcp/01-mcp-spec.md) 工具表）：15 原工具 + 1 新增 `submit` = **16 tools 完整集合**。MVP 退出条件 acceptance/04 §3 第三条要求实现其中 MVP 13 子集：

| MVP 13（必实现，per 任务原文 §17） | 扩展 3（MVP 退出条件可选） |
|---|---|
| `get_issue` | `get_workspace`（P1-C 修复后独立 agent 视角） |
| `search_issues` | `request_review`（Phase 2+ 协作） |
| `get_current_task` | `submit`（P1-F 新增，per 2026-08-27；MVP 必实现，Universal Submit 12 步入口） |
| `get_worktree` |  |
| `create_worktree` |  |
| `search_code` |  |
| `get_symbol` |  |
| `find_references` |  |
| `get_code_context` |  |
| `get_context` |  |
| `create_merge_request` |  |
| `run_validation` |  |
| `get_pipeline_status` |  |

> **MVP 实际必实现 14 tools** = 13 MVP 子集 + `submit`（per acceptance/04 §3 退出条件 #3 "MCP server 13 tools" + #6 "Universal Submit 12 步" 双重约束）。完整 16 = 14 MVP 必实现 + 2 扩展（`get_workspace` / `request_review`，per Phase 2+）。本节标题 "MCP 13 tools" 沿用任务原文 §17 数字，但正文描述 = 完整 16（per mcp/01 §2 工具表，per F-13 / INTERFACE-REVIEW-A 🔴 #3 + 🟡 #13）。

**关键约束**：
- 不暴露 `update_issue_table` / `insert_worktree_row` 等内部表操作
- 必须表达领域操作
- 2026-07-28 规范 + stdio transport（Rust SDK beta 风险规避）
- tool list 排序按 name 字典序 + metadata 含 `ttlMs` / `cacheScope`（per MCP §1.2）

**MCP Resources / Prompts MVP 范围**（per F-28 / INTERFACE-REVIEW-A 🟢 #28）：MVP 阶段（Level 1-2）**不实现** Resources / Prompts，仅 Phase 2+ 评估。spec/mcp/01 §4-§5 措辞"可选 / 不强制"在 arch 层明确为"**MVP 不实现**"。含义：MVP 退出条件 acceptance/04 §3 第三条 "MCP server 13 tools" **不**包含 Resources/Prompts 数量；IDE 客户端通过 `tools/call` 即可获得全部 MVP 能力，Resources/Prompts 缺失**不**影响 IDE 接入。

> v0.3 fix: 2026-08-27 per INTERFACE-REVIEW-C P1-4 / INTERFACE-REVIEW-A 🔴 #3 + 🟡 #13（F-13）数字统一 16（per mcp/01 §2 工具表）；MVP 13 子集 + submit = MVP 14 必实现。F-28 显式标 Resources/Prompts MVP 不实现。

### 2.4 REST + OpenAPI 3.1（远程 / 集成层）

- 用于：Web UI、Automation、External Agent、IDE Plugin
- 跟 MCP 共享同一 Domain API
- OpenAPI 3.1（不是 3.0）— 完整对齐 JSON Schema 2020-12
- 必须版本化、文档化、机器可读、稳定、权限感知、可审计

### 2.5 AGENTS.md Bootstrap（vendor-neutral）

**最小 3 命令**（per F-26 / INTERFACE-REVIEW-A 🟢 #26 / protocol-survey §2）— AGENTS.md bootstrap **必含**：

```markdown
# This repository is managed by STAR.

Discover available capabilities:
    star agent capabilities

Retrieve your current task:
    star task current --json

Submit your work:
    star submit
```

**可选 3 命令**（Phase 2+ 视项目情况追加，不算 MVP 必含）：

```markdown
# 以下命令在 Minimal 3 命令之外按需追加；不算 MVP 必含。

# Retrieve relevant context
star context current --json

# Search code
star code search "your query" --json

# Run affected tests before submit
star test affected
```

**关键约束**：
- 极薄（不超过 50 行）
- 不得塞企业知识
- 是 Bootstrap 不是 Knowledge Base
- **Minimal 3 命令 = MVP 必含**（acceptance/04 §3 退出条件之一 = "AGENTS.md bootstrap 含 3 个最小命令"）
- Optional 3 命令仅作为 "如果项目鼓励" 的提示，不强制出现
- per 任务原文 §14

> v0.3 fix: 2026-08-27 per INTERFACE-REVIEW-A 🟢 #26（F-26）6 命令拆 Minimal 3（必含）+ Optional 3（Phase 2+ 提示）。

## 3. Fallback Ladder（per §38）

```
Level 1: MCP + CLI + Git + AGENTS.md      (推荐入口)
   ↓
Level 2: CLI + Git + AGENTS.md            (MCP 不可用)
   ↓
Level 3: REST + Git + AGENTS.md           (CLI 不可用)
   ↓
Level 4: Git Only                         (所有抽象都不可用)
```

每一级都**必须**能跑通 Unknown Agent Test（per §42-§44）。

## 4. Capability Discovery（per §12）

```bash
star agent capabilities
star ide capabilities
star capabilities
```

返回结构化列表（machine-readable + human-readable 双格式）：

```json
{
  "schema_version": "agent-api/v1",
  "agent": {
    "commands": [
      {"name": "task current", "schema_ref": "..."},
      {"name": "submit", "schema_ref": "..."}
    ]
  },
  "ide": {
    "commands": [
      {"name": "workspace current", "schema_ref": "..."}
    ]
  },
  "capabilities": ["code_context", "code_navigation", "code_search", "context", "issues", "merge_requests", "projects", "tasks", "tests", "worktrees", "workspaces"]
}
```

> **capabilities 数组 = 11 个**（per F-15 / INTERFACE-REVIEW-A 🔴 #15 + INTERFACE-REVIEW-C P1-3 修复）：跟 [spec/cli/01-cli-spec.md §2.1](../spec/cli/01-cli-spec.md) MVP 17 核心命令一一对应，删除项：
>
> | 删除项 | 理由 |
> |---|---|
> | `repositories` | 隐式走 `workspace.repository`（per 🔴 #15 方案 B） |
> | `deployments` | 隐式走 `pipeline run`（per 🔴 #15 方案 B） |
> | `pipelines` | 走 cli/01 §2.2 扩展 `star pipeline run/status`（Phase 2+ 标记能力） |
> | `reviews` | 走 cli/01 §2.2 扩展 `star mr review`（Phase 2+ 标记能力） |
> | `code_context` | 与 `context` 重复（per 🔴 #15 警告） |
>
> 共 15 - 5 + 1（保留 code_context，删 context 重复）= **11 个**；按字母序排序。

> v0.3 fix: 2026-08-27 per INTERFACE-REVIEW-A 🟡 #15（F-15）capabilities 数组 15 → 11，匹配 cli/01 §2.1 MVP 17 核心命令双表。

## 5. Agent Instructions（per §13）

```bash
star agent instructions
star ide instructions
```

动态输出（基于 user / agent / IDE / project / workspace / env / permission）：

```text
You may:
- read repository
- read issues
- search code
- navigate symbols
- create worktrees
- modify current worktree
- run tests
- create merge requests

You may not:
- merge protected branches
- deploy production
- delete repositories
```

### 5.1 IDE 接入通道与 MCP transport 约束（per F-22 / INTERFACE-REVIEW-A 🟢 #22）

IDE 接入 STAR 时使用的接入通道（per §2 + §3）跟 transport 类型**显式绑定**：

| 接入通道 | MCP transport | 备注 |
|---|---|---|
| Local process（IDE 内 STAR CLI + MCP server 进程） | **stdio** | MVP 唯一支持；Rust MCP SDK 当前稳定 target（per [spec/mcp/01-mcp-spec.md §1](../spec/mcp/01-mcp-spec.md) "stdio transport"） |
| Remote（IDE 远程接入 STAR server） | **Streamable HTTP** | **Phase 2+**；MVP **不**实现；MCP 2026-07-28 已规范但 Rust SDK beta 风险规避 |
| Local 但无 MCP（IDE 只有 Git / Shell） | n/a | 走 Fallback Ladder L2（CLI + Git + AGENTS.md，per §3） |

> v0.3 fix: 2026-08-27 per INTERFACE-REVIEW-A 🟢 #22（F-22）§5 加 IDE MCP transport 约束（stdio MVP / Streamable HTTP Phase 2+ 二选一）。

## 6. Progressive Discovery（per §16）

```
Repository
   ↓ (读 README + AGENTS.md)
Agent / IDE Instructions
   ↓ (知道 STAR 存在)
star capabilities
   ↓ (知道可用命令)
star task current
   ↓ (拿到 Issue)
star context current
   ↓ (拿到相关代码/文档/符号)
执行任务
```

**关键**：Agent **不需要提前知道 STAR**。它通过 Repository 自身发现。

## 7. 验收（per P1-K 修复 2026-08-27）

- **Unknown Agent Test 跑 Level 1**（per acceptance/01 §3 16 步实际用 star CLI） — 跟 acceptance/01 §3 实际跑通能力兼容
- **Level 2 / 3 / 4 单独跑 conformance 测试**（per [spec/vcs/04 §5](../spec/vcs/04-fallback-strategy.md) 4 级分别跑通） — 跟 vcs/04 §3 L1/L2/L3/L4 表一一对应
- Phase D 的 Unknown IDE Test 必须**只**用 Level 3 (REST + Git) 通过
- 真实 Coding Agent 接入（per AI Compatibility Matrix）实测 7 款中至少 4 款

**Acceptance ↔ Level 边界表**（per INTERFACE-REVIEW-C P1-1 修复补表）：

| 测试 | 跑哪一级 | 跑通条件 | spec 引用 |
|---|---|---|---|
| Unknown Agent Test | **Level 1**（MCP + CLI + Git + AGENTS.md）| 16 步全跑通，含 star CLI 4-15 步 | [acceptance/01 §3](../spec/acceptance/01-unknown-agent-test.md) |
| Zero-Knowledge Agent Test | **Level 1** | 12 步子集（去掉 MR 自动创建 + Issue 状态更新） | [acceptance/02 §2](../spec/acceptance/02-zero-knowledge-agent-test.md) |
| Unknown IDE Test | **Level 3**（REST + Git + AGENTS.md）| 10 步全跑通，**只**用 OpenAPI + Git（不用 star CLI / MCP） | [acceptance/03 §3](../spec/acceptance/03-unknown-ide-test.md) |
| Level 2 conformance | **Level 2**（CLI + Git + AGENTS.md）| 单独跑通，per [vcs/04 §3 L2 表](../spec/vcs/04-fallback-strategy.md) | [spec/vcs/04 §3](../spec/vcs/04-fallback-strategy.md) |
| Level 4 conformance（Git Only） | **Level 4**（Git Only）| 单独跑通，per [vcs/04 §3 L4 表](../spec/vcs/04-fallback-strategy.md) | [spec/vcs/04 §3](../spec/vcs/04-fallback-strategy.md) |

> **冲突来源**（per 子代理 C P1-1）：原 §7 写"必须只用 Level 4 通过"但 acceptance/01 §3 16 步用了大量 `star` CLI（步骤 4-15），`star` CLI 属 Level 2+ 能力，**不可同时成立**。修法：Unknown Agent Test 跑 Level 1（用 star CLI），Level 2/3/4 单独跑 conformance。

> v0.3 fix: 2026-08-27 per INTERFACE-REVIEW-C P1-1 边界表（Acceptance ↔ Level 映射）。

## 8. Event Naming Boundary — GitGit vs STAR Domain Events（per B-17 / INTERFACE-REVIEW-B 🟡 #17）

GitGit 原生事件（per [arch/05 §6](05-gitgit-compat-arch.md)）与 STAR Domain Events（per [spec/flows/08 §1.1](../spec/flows/08-event-model.md)）**重名**事件（如 `WorktreeCreated`）必须用命名空间区分，避免跨层数据语义混淆：

| 事件名（带命名空间） | 出处层 | 语义 | 触发时机 |
|---|---|---|---|
| `WorktreeCreated.gitgit` | GitGit 物理层 | git worktree 实际创建（OS 级） | `git worktree add` 成功 |
| `WorktreeCreated.star` | STAR 业务层 | Workspace ↔ Worktree 绑定完成 | `star worktree create <id>` 成功 |
| `MergeCompleted.gitgit` | GitGit 物理层 | git merge 物理完成 | git merge 退出码 0 |
| `MergeCompleted.star` | STAR 业务层 | MR 状态变 MERGED | STAR MR 状态机迁移 |
| `ConflictDetected.gitgit` | GitGit 物理层 | git merge / rebase 检测到 conflict | merge / rebase 退出码非 0 |
| `ConflictDetected.star` | STAR 业务层 | 9 类冲突（per [flows/04 §2](../spec/flows/04-multi-agent-conflict.md)）之一被识别 | STAR 冲突检测器判定 |

> **命名约定**（per B-17 修复 2026-08-27）：GitGit 物理层事件 = `<EventName>.gitgit`；STAR 业务层事件 = `<EventName>.star`。同一动词名（如 `WorktreeCreated`）跨层时**必须**带命名空间后缀。
>
> **触发链路**：GitGit 物理层事件触发 → STAR 业务层在 Application Service 内重发业务层事件（如 `WorktreeCreated.gitgit` → `WorktreeCreated.star`），保持 STAR 上层逻辑只看 `.star` 后缀事件。STAR 上层（[flows/08 §1.1](../spec/flows/08-event-model.md) 13 个 STAR Domain Events）**不**含 `.gitgit` 后缀（隐式 `.star` 默认）。

> v0.3 fix: 2026-08-27 per INTERFACE-REVIEW-B 🟡 #17（B-17）加 GitGit/STAR 事件命名表，跨层重名（`WorktreeCreated`）用 `.gitgit` / `.star` 命名空间后缀区分。arch/05 §6 同步。

## 9. IDE Level ↔ 接入通道判别矩阵（per P2-11 / INTERFACE-REVIEW-C 🟡 P2-11）

IDE 接入 STAR 时，按 IDE 能力判定走哪个 Fallback Level（per §3）。**判别矩阵**：

| IDE 能力 | 走哪一级 | 接入通道组合 | 备注 |
|---|---|---|---|
| IDE 完整支持 MCP client + 有 star CLI binary | **Level 1** | MCP + CLI + Git + AGENTS.md | 推荐入口；Cursor / Junie / Claude Code 这类有 MCP + CLI 支持的 IDE |
| IDE 有 Git / Shell + 跑 star CLI，但无 MCP client | **Level 2** | CLI + Git + AGENTS.md | VS Code 装 shell extension；Copilot Chat 调 CLI |
| IDE 只有 HTTP client / 浏览器（无 CLI binary） | **Level 3** | REST + Git + AGENTS.md | Web UI / 远程 IDE / 简化客户端；OpenAPI 3.1 spec 必含（per [spec/rest/01](../spec/rest/01-rest-strategy.md)）|
| IDE 只支持 Git 命令（无 CLI / REST / MCP） | **Level 4** | Git Only | 纯 Git 客户端；走 AGENTS.md + git 命令拼 Universal Submit |
| IDE 啥都不支持（连 Git 命令都没） | **不可达** | n/a | 不在兼容范围；建议换 IDE |

> **判别依据**：IDE 自报能力（`ide capabilities` per §4） + STAR 端探测（`star agent capabilities` + MCP `tools/list` + REST `/api/v1/ide/capabilities`）；IDE 端能力缺失时**降级**到下一级，**不**报错。
>
> **cross-ref**（per P2-11 修复 2026-08-27）：此矩阵跟 §3 Fallback Ladder + §5.1 transport 约束 + §7 Acceptance ↔ Level 边界表**联动**——同一组测试 (Unknown IDE Test) 在不同 IDE 能力下自动降级到对应 Level。

> v0.3 fix: 2026-08-27 per INTERFACE-REVIEW-C P2-11 加 IDE Level ↔ 接入通道判别矩阵（5 类 IDE 能力 → 4 级 Fallback）。

## 10. 签字栏 / 修订历史

per [arch/01-current-architecture-analysis.md](01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-26 | Mavis（per DEC-008）| 初版：5 接入通道 + Fallback Ladder 4 级 | Phase C 54 份 spec 草案 |
| v0.2 | 2026-08-27 | Ulysses（一人公司 12 角色 per DEC-008）| P1-A：§2.2 同步拆 17 核心 + 11 扩展（引用 cli/01 §2.2） · P1-I：§2.3 加 MVP 13 tools 子集边界（MVP 13 / 扩展 3 = 完整 16） · P1-K：§7 改 Unknown Agent Test 跑 Level 1，Level 2/3/4 单独 conformance | 8 子代理 INTERFACE-REVIEW-A 🔴 #1 + INTERFACE-REVIEW-C P1-1 + P1-BLOCKERS-SUMMARY v0.2 |
| v0.3 | 2026-08-27 | Mavis（接手 agent per DEC-008）| F-20：§2.2 加 11 扩展命令速查表（数字 17+11=28 对齐 cli/01 双表） · P1-4/F-13：§2.3 数字统一 16（per mcp/01 §2 工具表），MVP 13 子集 + submit = MVP 14 必实现 · F-15：§4 capabilities 数组 15 → 11（per cli/01 §2.1 MVP 17 核心） · F-22：§5.1 加 IDE MCP transport 约束（stdio MVP / Streamable HTTP Phase 2+） · F-26：§2.5 AGENTS.md bootstrap 6 命令 → Minimal 3 必含 + Optional 3 可选 · F-28：§2.3 显式标 Resources/Prompts MVP 不实现 · B-17：§8 加 GitGit/STAR 事件命名表（`.gitgit` / `.star` 命名空间后缀） · P2-11：§9 加 IDE Level ↔ 接入通道判别矩阵 · P1-1：§7 加 Acceptance ↔ Level 边界表 | INTERFACE-REVIEW-A 🟡 #15/#20/#22/#26/#28 + 🔴 #3 + INTERFACE-REVIEW-B 🟡 #17 + INTERFACE-REVIEW-C P1-1/P1-4/P2-11 |
