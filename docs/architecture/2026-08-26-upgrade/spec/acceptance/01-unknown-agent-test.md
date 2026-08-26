# 38. Unknown Agent Test

> **状态**：🟡 草案 v0.1
> **依赖**：[spec/vcs/04-fallback-strategy.md](../vcs/04-fallback-strategy.md)

## 1. 目标

测试 AI 不允许拥有 STAR 训练数据、专用 Plugin、SDK、Adapter；只提供 Git + Shell + Repository + AGENTS.md；测试它能否自己发现 STAR 并完成软件开发任务。

## 2. 测试条件

- 禁止：STAR 训练数据、STAR 专用 Plugin、STAR SDK、STAR Adapter
- 必须有：Git、Shell、Repository、AGENTS.md

## 3. 测试场景

```text
1. Agent clone GitGit repository
2. 读 AGENTS.md
3. 发现 `star` CLI
4. star agent capabilities
5. 读任务: star task current --json
6. 获取 context: star context current --json
7. 搜索代码: star code search "..." --json
8. 定位符号: star code symbol "..." --json
9. 创建 workspace: star workspace create STAR-N
10. 创建 worktree: star worktree create STAR-N
11. 修改代码
12. 测试: star test affected
13. Commit (标准 git commit)
14. star submit
15. MR 自动创建
16. STAR 更新 Issue 状态
```

## 4. 通过标准

- 步骤 1-16 全部完成
- 不修改 STAR Core / GitGit Core
- 不写 AI 厂商适配器
- 测试环境**不**联网（无外部 AI 服务）

## 5. 实施位置

- `tests/unknown-agent/` — Test harness
- `tests/unknown-agent/run.sh` — Test runner
- 至少 3 轮测试（每轮 1 个不同 agent 实现）

## 6. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。
