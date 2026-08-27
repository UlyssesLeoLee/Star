# 47. IDE Instructions Specification

> **状态**：🟡 草案 v0.1
> **依赖**：[spec/acceptance/09-agent-instructions-spec.md](09-agent-instructions-spec.md)

## 1. IDE Instructions（per §15 任务原文）

4 类 Instructions：
- Bootstrap Instructions
- Context Instructions
- Policy Instructions
- Task Instructions

## 2. Bootstrap Instructions

告诉 IDE / Agent：
- STAR 是否存在
- 如何发现 CLI
- 如何获取当前任务
- 如何获取权限
- 如何获取 Context

## 3. Context Instructions

告诉 Agent：
- 当前 Issue
- 当前 Task
- 相关文件
- 相关符号
- 相关测试
- 相关 ADR
- 相关 MR

## 4. Policy Instructions

告诉 Agent：
- 哪些操作允许
- 哪些操作需要审批
- 哪些分支受保护
- 哪些测试必须通过
- 哪些文件禁止修改

## 5. Task Instructions

告诉 Agent：
- 当前任务目标
- 验收标准
- 约束
- 依赖
- 交付方式

## 6. 实施位置

- `crates/star-cli/src/commands/ide/instructions.rs`

## 7. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。
