# 40. Unknown IDE Test

> **状态**：🟡 草案 v0.1
> **依赖**：[spec/acceptance/01-unknown-agent-test.md](01-unknown-agent-test.md)

## 1. 目标（per §44 任务原文）

测试一个没有 STAR 专用插件的 IDE 是否可以通过标准能力接入 STAR。

## 2. 测试条件

只提供：
- Git
- Shell
- Repository
- AGENTS.md
- star CLI
- OpenAPI

## 3. 测试场景

```text
打开 Repository
   ↓
发现 STAR (读 AGENTS.md)
   ↓
获取当前 Task
   ↓
获取 Context
   ↓
搜索代码
   ↓
定位符号
   ↓
修改文件
   ↓
运行测试
   ↓
创建 Commit
   ↓
创建 MR
```

## 4. 通过条件

- 10 步全部完成
- IDE 不需要 STAR 专用 plugin
- 通过 Git + Shell + OpenAPI 标准能力完成

## 5. 如果必须等 IDE 厂商开发 STAR Plugin

测试失败 → 架构设计失败。

## 6. 实施位置

- `tests/unknown-ide/` — Test harness
- 至少 3 轮测试（每轮 1 个不同 IDE）

## 7. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。
