# 37. Fallback Strategy

> **状态**：🟡 草案 v0.2
> **依赖**：[arch/03 STAR AI Compat Arch §3](../../arch/03-star-ai-compat-arch.md)

## 1. Fallback Ladder（per §38 任务原文）

```
Level 1
MCP + CLI + Git
   ↓
Level 2
CLI + Git
   ↓
Level 3
REST + Git
   ↓
Level 4
Git Only
```

## 2. 每级必须能跑通的核心闭环

```text
Unknown Coding Agent / Unknown IDE
   ↓
Clone GitGit Repository (or other Git provider)
   ↓
读 Repository Instructions (AGENTS.md / README)
   ↓
发现 STAR CLI
   ↓
Capability Discovery
   ↓
获取 Assigned Issue
   ↓
获取 Context
   ↓
搜索相关代码
   ↓
定位相关符号
   ↓
创建 Workspace
   ↓
创建 Worktree
   ↓
修改代码
   ↓
运行测试
   ↓
Commit
   ↓
star submit
   ↓
创建 MR
   ↓
STAR 更新 Issue 状态
```

## 3. 每级的最低可用子集

| Level | 必须 | 不得依赖 |
|---|---|---|
| 1 | MCP + CLI + Git + AGENTS.md | IDE 专用 plugin |
| 2 | CLI + Git + AGENTS.md | MCP server |
| 3 | REST + Git + AGENTS.md | CLI binary |
| 4 | Git Only（通过 `star` 实际是 shell 脚本 or 远程 REST 都不依赖） | CLI binary / REST server / MCP server |

## 4. 关键约束

- 任何 Level 必须能跑通
- 测试覆盖 4 级（per Phase D）
- Git Only 是兜底终极底线

## 5. 实施位置

- `crates/star-cli/` — 17 个核心命令（per [spec/cli/01-cli-spec.md §2.1](../cli/01-cli-spec.md)）
- `crates/star-mcp/` — 13 个 MVP tools（per [arch/03 §2.3 MVP 13 tools 子集边界](../../arch/03-star-ai-compat-arch.md)）
- `crates/star-rest/` — REST API
- 全部 4 级的 conformance 测试在 `tests/`（per P1-L 修复 2026-08-27） — `tests/unknown-agent/` / `tests/zero-knowledge-agent/` / `tests/unknown-ide/` / `tests/fallback-conformance/`
  - 4 套具体目录对应 acceptance/01（Unknown Agent）/ 02（Zero-Knowledge）/ 03（Unknown IDE）+ Level 2/3/4 conformance 单独测试
  - 原表述"全部 4 级的 conformance 测试在 `crates/star-cli/tests/`"不准确 — 实际 acceptance/01-03 实施位置 = `tests/<test-name>/`，与 vcs/04 §5 现统一为 `tests/`

## 6. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-26 | Mavis（per DEC-008）| 初版：4 级 Ladder + 16 步核心闭环 | Phase C 54 份 spec 草案 |
| v0.2 | 2026-08-27 | Ulysses（一人公司 12 角色 per DEC-008）| P1-L：§5 实施位置从 `crates/star-cli/tests/` 改 `tests/`（与 acceptance/01-03 一致，4 级 conformance 测试在 `tests/<level>/`） | 8 子代理 INTERFACE-REVIEW-C P1-2 + P1-BLOCKERS-SUMMARY v0.2 |
