# 03. STAR AI Compatibility Architecture

> **状态**：🟡 草案 v0.1
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

```bash
# 17 个核心命令
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
star pipeline run
star submit  # Universal Submit
```

**关键约束**：
- `--json` 必须稳定（versioned as `agent-api/v1`）
- `--quiet` / `--fields` / `--limit` / `--cursor` / `--no-color` / `--schema-version`
- 不得强制解析 ANSI / 表格 / 自然语言
- CLI 不只是 REST API 的映射，必须表达 Domain Semantics

### 2.3 MCP Server（增强层）

per [spec/mcp/spec.md](../spec/mcp/spec.md)：

```
get_issue / search_issues / get_current_task / get_workspace /
get_worktree / create_worktree / search_code / get_symbol /
find_references / get_code_context / get_context / create_merge_request /
request_review / run_validation / get_pipeline_status
```

**关键约束**：
- 不暴露 `update_issue_table` / `insert_worktree_row` 等内部表操作
- 必须表达领域操作
- 2026-07-28 规范 + stdio transport（Rust SDK beta 风险规避）

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

## 7. 验收

- Phase D 的 Unknown Agent Test 必须**只**用 Level 4 (Git Only) 通过
- Phase D 的 Unknown IDE Test 必须**只**用 Level 3 (REST + Git) 通过
- 真实 Coding Agent 接入（per AI Compatibility Matrix）实测 7 款中至少 4 款

## 8. 签字栏 / 修订历史

per [arch/01-current-architecture-analysis.md](01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。
