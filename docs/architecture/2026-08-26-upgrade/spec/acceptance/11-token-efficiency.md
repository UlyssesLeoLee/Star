# 48. Token Efficiency Requirements

> **状态**：🟡 草案 v0.1
> **依赖**：[arch/06 §4 Token Efficiency](../../arch/06-threat-model-nfr.md)

## 1. 禁止模式

- ❌ 每次发送整个 Issue 历史
- ❌ 每次发送整个 Repository
- ❌ 每次发送所有文档
- ❌ 每次发送所有 Pipeline 日志
- ❌ 每次发送所有代码文件

## 2. 强制模式

- ✅ Progressive Disclosure
- ✅ Graph-based Context
- ✅ Semantic Search
- ✅ Symbol-level Retrieval
- ✅ Incremental Diff
- ✅ Context Cache
- ✅ Context Snapshot
- ✅ Context Fingerprint
- ✅ Task-aware Retrieval
- ✅ IDE Viewport Context（per IDE Session 状态）
- ✅ Open File Context
- ✅ Selection Context
- ✅ Diagnostic Context

## 3. 测量

每次 tool call 输出必须带：

```json
{
  "result": {...},
  "token_count": {
    "input": 1234,
    "output": 56,
    "context_fetched": 789
  }
}
```

## 4. 预算（per Project 可调）

| 操作 | Token 上限 |
|---|---|
| `star task current` | < 500 |
| `star context get` (minimal) | < 5K |
| `star context get` (normal) | < 20K |
| `star code search` | < 10K (result) |
| `star submit` 端到端 | < 50K |

## 5. 实施位置

- `crates/star-context/src/token_budget.rs`
- `crates/star-cli/src/output.rs` — token 计数

## 6. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。
