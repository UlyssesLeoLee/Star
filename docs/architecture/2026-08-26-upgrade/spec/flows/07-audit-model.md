# 32. Audit Model

> **状态**：🟡 草案 v0.1
> **依赖**：[arch/06 §1.2 Threat Model](../../arch/06-threat-model-nfr.md)

## 1. 统一 Audit Trail（per §35 任务原文）

Human / AI Agent / IDE / Automation 走同一 Audit Trail。

## 2. 必含字段

```
Actor
ActorType
Session
Workspace
Task
Action
Resource
Timestamp
Permission
Result
TraceID
```

## 3. ActorType

```
Human
Agent
IDE
Service
Automation
```

## 4. 关键约束

- ❌ 不得因为 AI 是通过 CLI 操作就失去审计
- ❌ 不得因为 IDE 是通过 Git 操作就失去审计
- ❌ 不得因为 Automation 跑后台就失去审计

## 5. 存储

- Append-only log
- HMAC chain 防篡改（per [arch/06 §1.2 T-10](../../arch/06-threat-model-nfr.md)）
- 7 年保留（合规）

## 6. 实施位置

- `crates/star-audit/` — Audit service
- `crates/star-audit/src/append_only.rs` — Append-only 存储
- `crates/star-audit/src/hmac.rs` — HMAC chain

## 7. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。
