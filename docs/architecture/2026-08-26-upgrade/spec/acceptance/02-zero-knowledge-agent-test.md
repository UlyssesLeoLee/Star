# 39. Zero-Knowledge Agent Test

> **状态**：🟡 草案 v0.1
> **依赖**：[spec/acceptance/01-unknown-agent-test.md](01-unknown-agent-test.md)

## 1. 更严格的版本（per §43 任务原文）

Agent 初始 Prompt 只能：

```text
Fix the assigned issue in this repository.
```

**除此之外不给 STAR 使用说明**。

## 2. 成功标准

```text
发现 AGENTS.md
   ↓
发现 star CLI
   ↓
Capability Discovery
   ↓
读取 Task
   ↓
获取 Context
   ↓
建立 / 进入 Workspace
   ↓
建立 / 进入 Worktree
   ↓
搜索代码
   ↓
导航符号
   ↓
修改
   ↓
测试
   ↓
提交
```

## 3. 通过条件

- 12 步全部完成
- Agent 没有"提示"（除初始 prompt 外）
- 测试环境**不**联网

## 4. 与 Unknown Agent Test 的区别

| 维度 | Unknown Agent | Zero-Knowledge |
|---|---|---|
| 初始 prompt | 详细 STAR 提示 | 极简 "Fix the issue" |
| 测试严格度 | 中 | 高 |
| 难度 | 中 | 高 |

## 5. 实施位置

- `tests/zero-knowledge-agent/` — Test harness
- 至少 3 轮测试

## 6. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。
