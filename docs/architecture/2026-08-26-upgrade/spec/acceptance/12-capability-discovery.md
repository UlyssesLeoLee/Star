# 49. Capability Discovery Protocol

> **状态**：🟡 草案 v0.1
> **依赖**：[arch/03 §4](../../arch/03-star-ai-compat-arch.md)

## 1. 核心能力（per §12 任务原文）

未知 AI / IDE 不应提前学习 STAR。它应主动发现 STAR 能力。

## 2. 命令

```bash
star agent capabilities
star ide capabilities
star capabilities
```

## 3. 返回 Schema

```json
{
  "schema_version": "agent-api/v1",
  "capabilities": [
    "projects", "issues", "tasks", "workspaces", "worktrees",
    "repositories", "code_search", "code_navigation", "code_context",
    "merge_requests", "context", "tests", "pipelines", "reviews", "deployments"
  ],
  "commands": {
    "agent": [
      {"name": "task current", "schema_ref": "agent-api/v1#CurrentTask", "description": "..."}
    ],
    "ide": [
      {"name": "workspace current", "schema_ref": "ide-api/v1#Workspace", "description": "..."}
    ]
  },
  "resources": {
    "agent": [
      {"uri_template": "issue://{id}", "description": "..."}
    ]
  },
  "permissions": {
    "read_repository": "ALLOW",
    "create_worktree": "ALLOW",
    "deploy_production": "DENY"
  }
}
```

## 4. Describe 命令

```bash
star agent describe issue
star agent describe worktree
star agent describe merge-request
star ide describe workspace
star ide describe code-navigation
star ide describe code-context
```

返回详细描述：Command / Input Schema / Output Schema / Permission / Side Effects / Preconditions / Examples / Event Types / Error Codes

## 5. 实施位置

- `crates/star-cli/src/commands/agent/capabilities.rs`
- `crates/star-cli/src/commands/agent/describe.rs`
- `crates/star-cli/src/commands/ide/capabilities.rs`
- `crates/star-cli/src/commands/ide/describe.rs`

## 6. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。
