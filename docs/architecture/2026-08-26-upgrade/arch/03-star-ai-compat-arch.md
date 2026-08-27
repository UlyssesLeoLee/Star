# 03. STAR AI Compatibility Architecture

> **状态**：🟡 草案 v0.2
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

> **MVP 子集边界**（per P1-A 修复 2026-08-27）：17 核心 = MVP 退出条件 acceptance/04 §3 第一条必含。完整命令清单 + 11 个扩展命令（context current / code references / mr review / test run / pipeline run / pipeline status + 新增 diff / policy check / commit / push / mr link）见 [spec/cli/01-cli-spec.md §2.2](../spec/cli/01-cli-spec.md)。

**关键约束**：
- `--json` 必须稳定（versioned as `agent-api/v1`）
- `--quiet` / `--fields` / `--limit` / `--cursor` / `--no-color` / `--schema-version`
- 不得强制解析 ANSI / 表格 / 自然语言
- CLI 不只是 REST API 的映射，必须表达 Domain Semantics
- **`star` 是 `git` 的 superset**（per P1-H 修复 2026-08-27） — `star` 提供 `git` 不具备的领域操作 + 包装 `git` 子命令（diff / commit / push）注入 Policy / Audit / Worktree 上下文

### 2.3 MCP Server（增强层）

per [spec/mcp/01-mcp-spec.md](../spec/mcp/01-mcp-spec.md)：

**MVP 13 tools 子集边界**（per P1-I 修复 2026-08-27）：MVP 退出条件 acceptance/04 §3 第三条 = "MCP server 13 tools"。完整 16 tools = 13 MVP + 3 扩展（get_workspace / request_review / submit 三个 Phase 2+ 或 P1-F 后置）：

| MVP 13（必实现） | 扩展 3（per P1-F / Phase 2+） |
|---|---|
| `get_issue` | `get_workspace`（P1-C 修复后独立） |
| `search_issues` | `request_review`（Phase 2+ 协作） |
| `get_current_task` | `submit`（P1-F 新增，per 2026-08-27） |
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

> `submit` 虽属 P1-F 新增，但 MVP 阶段必实现（acceptance/04 §3 退出条件"Universal Submit 12 步"必跑通）；故 MVP 实际为 14（13 + submit），完整 16 = 14 MVP + 2 扩展（get_workspace / request_review）。本节按 13 = 任务原文 §17 列出，不变更 §2 整体描述。

**关键约束**：
- 不暴露 `update_issue_table` / `insert_worktree_row` 等内部表操作
- 必须表达领域操作
- 2026-07-28 规范 + stdio transport（Rust SDK beta 风险规避）
- tool list 排序按 name 字典序 + metadata 含 `ttlMs` / `cacheScope`（per MCP §1.2）

### 2.4 REST + OpenAPI 3.1（远程 / 集成层）

- 用于：Web UI、Automation、External Agent、IDE Plugin
- 跟 MCP 共享同一 Domain API
- OpenAPI 3.1（不是 3.0）— 完整对齐 JSON Schema 2020-12
- 必须版本化、文档化、机器可读、稳定、权限感知、可审计

### 2.5 AGENTS.md Bootstrap（vendor-neutral）

```markdown
# This repository is managed by STAR.

Discover available capabilities:
    star agent capabilities

Retrieve your current task:
    star task current --json

Retrieve relevant context:
    star context current --json

Search code:
    star code search "your query" --json

Before submitting:
    star test affected

Submit:
    star submit
```

**关键约束**：
- 极薄（不超过 50 行）
- 不得塞企业知识
- 是 Bootstrap 不是 Knowledge Base
- per 任务原文 §14

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
  "capabilities": ["projects", "issues", "tasks", "workspaces", "worktrees", "repositories", "code_search", "code_navigation", "code_context", "merge_requests", "context", "tests", "pipelines", "reviews", "deployments"]
}
```

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

> **冲突来源**（per 子代理 C P1-1）：原 §7 写"必须只用 Level 4 通过"但 acceptance/01 §3 16 步用了大量 `star` CLI（步骤 4-15），`star` CLI 属 Level 2+ 能力，**不可同时成立**。修法：Unknown Agent Test 跑 Level 1（用 star CLI），Level 2/3/4 单独跑 conformance。

## 8. 签字栏 / 修订历史

per [arch/01-current-architecture-analysis.md](01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-26 | Mavis（per DEC-008）| 初版：5 接入通道 + Fallback Ladder 4 级 | Phase C 54 份 spec 草案 |
| v0.2 | 2026-08-27 | Ulysses（一人公司 12 角色 per DEC-008）| P1-A：§2.2 同步拆 17 核心 + 11 扩展（引用 cli/01 §2.2） · P1-I：§2.3 加 MVP 13 tools 子集边界（MVP 13 / 扩展 3 = 完整 16） · P1-K：§7 改 Unknown Agent Test 跑 Level 1，Level 2/3/4 单独 conformance | 8 子代理 INTERFACE-REVIEW-A 🔴 #1 + INTERFACE-REVIEW-C P1-1 + P1-BLOCKERS-SUMMARY v0.2 |
