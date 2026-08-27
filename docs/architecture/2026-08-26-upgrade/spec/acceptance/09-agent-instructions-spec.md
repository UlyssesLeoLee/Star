# 46. Agent Instructions Specification

> **状态**：🟡 草案 v0.1
> **依赖**：[arch/03 STAR AI Compat Arch §5](../../arch/03-star-ai-compat-arch.md)

## 1. AGENTS.md Bootstrap（per §14 任务原文）

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

## 2. 必须薄

- 不超过 50 行
- 不塞企业知识
- 是 Bootstrap 不是 Knowledge Base

## 3. 动态 Instructions（per §13）

```bash
star agent instructions
```

输出根据 user / agent / IDE / project / workspace / env / permission 动态生成：

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

## 4. 不依赖静态 Prompt

- 不要把所有内容都写进 system prompt
- 每次任务前调 `star agent instructions` 获取最新状态

## 5. Repository-level vs Project-level vs Task-level

| 层 | 写入位置 | 内容 |
|---|---|---|
| Bootstrap | `AGENTS.md` | 通用 STAR 入口 |
| Context | `docs/context/<id>.md` (per Issue) | 当前任务相关 |
| Policy | `.star/policy.md` (per Project) | 项目级策略 |
| Task | `tasks/<id>.md` (per Task) | 任务级详情 |

**不要全塞 AGENTS.md**。

## 6. 实施位置

- `crates/star-cli/src/commands/agent/instructions.rs`
- AGENTS.md 生成器在 `crates/star-cli/src/commands/agent/generate_bootstrap.rs`

## 7. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。
