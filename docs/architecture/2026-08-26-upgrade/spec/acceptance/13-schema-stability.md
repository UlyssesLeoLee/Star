# 50. Schema Stability

> **状态**：🟡 草案 v0.1
> **依赖**：[spec/agent-api/01-schema.md](../agent-api/01-schema.md) · [spec/ide-api/01-schema.md](../ide-api/01-schema.md)

## 1. 机器可读稳定性

因为 AI / IDE 会依赖机器输出，**CLI JSON Schema 稳定性比 Human CLI 文本稳定性更重要**（per §41 任务原文）。

## 2. 版本号方案

- `agent-api/v1` — Agent API
- `ide-api/v1` — IDE API
- `context-api/v1` — Context API
- `git-provider/v1` — Git Provider API

## 3. Breaking Change 规则

任何 Breaking Change 必须版本化：
- ❌ 字段重命名
- ❌ 字段删除
- ❌ 字段类型变更
- ✅ 添加新字段（minor）
- ✅ 添加新命令（minor）

## 4. Migration 策略

- `agent-api/v1` 仍支持至少 12 个月
- `agent-api/v2` 上线时，`/v1` endpoint 仍工作
- Deprecation warning 通过 response header

## 5. 验证

```bash
# CI 跑 schema 兼容性测试
pnpm test --filter=@star/schema-stability
```

## 6. 实施位置

- `crates/star-cli/src/schemas/` — 所有 schema 落盘
- `crates/star-cli/tests/schema_compat.rs` — 兼容性测试

## 7. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。
