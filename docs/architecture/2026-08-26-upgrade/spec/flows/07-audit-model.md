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
Human       # 人（开发 / reviewer / 终端用户）
Agent       # AI Agent（Claude / Codex / Gemini / Local LLM / Cursor / JetBrains）
System      # 系统层（IDE Session / GitGit / Platform Event Bus，per B-07/B-16 修复 2026-08-27）
Service     # STAR 内部服务（鉴权 / 审计 / NATS consumer / NFR 平台层）
Automation  # 外部脚本（CI / cron / 定时任务 / webhook）
```

> **5 ActorType 边界澄清（per B-07 / B-16 修复 2026-08-27）**：
> - **Human** ↔ [resources/05](../../resources/05-agent-permission-model.md) 隐式一致（User = Human）
> - **Agent** ↔ [resources/03 §3](../../resources/03-agent-identity.md) Agent Schema
> - **System** ↔ IDE Session + GitGit + Platform 内部事件（per [resources/04](../../resources/04-ide-session-identity.md) + [arch/05](../../arch/05-gitgit-compat-arch.md)，System 是合并抽象 — 区别于具体 IDE/具体 GitGit 客户端）
> - **Service** ↔ STAR 平台层服务（鉴权 / 审计 / 事件总线 / NATS consumer）— **是 STAR 一等对象**，与 Automation 区分
> - **Automation** ↔ 外部脚本（CI / cron / 定时任务 / webhook）— **不是 STAR 一等对象**，是外部 trigger

## 4. 关键约束

- ❌ 不得因为 AI 是通过 CLI 操作就失去审计
- ❌ 不得因为 IDE 是通过 Git 操作就失去审计
- ❌ 不得因为 Automation 跑后台就失去审计

## 5. 存储

- Append-only log
- HMAC chain 防篡改（per [arch/06 §1.2 T-10](../../arch/06-threat-model-nfr.md)）
- 7 年保留（合规）

## 6. Audit Schema（per B-27 修复 2026-08-27）

> **7 字段权威定义**（`AuditEntry` schema）：
>
> - `audit_id` (string, e.g. `"audit-2026-08-27-uuid-v7-..."` — UUID v7 时间排序)
> - `actor` (string, e.g. `"user-uuid"` / `"agent-session-id"` / `"service-auth"`)
> - `action` (string, e.g. `"workspace.create"` / `"agent.heartbeat"` / `"policy.deny"`)
> - `target` (object, `{type, id}` — e.g. `{type: "workspace", id: "ws-abc"}`)
> - `timestamp` (timestamp, ISO 8601 with timezone)
> - `trace_id` (string, 关联 [agent-api/v1 §3.15 Error.trace_id](../agent-api/01-schema.md#315-error))
> - `payload` (object?, 领域特定 payload — e.g. `{from_state: "Implementing", to_state: "Failed"}` / `{policy_violation: "..."}`)
>
> **权威来源（per B-27 修复 2026-08-27）**：本 spec 定义 7 字段；权威 schema 由 W4 子代理落 [agent-api/v1 §3.X AuditEntry](../agent-api/01-schema.md)（预计 §3.18+ — W4 起草中，本 spec §6 字段定义需与 §3.X 1:1 对齐）。CLI / MCP / REST / Audit 写入 **全部**统一引用本 schema，per Error 模型 6 字段 1 源原则（[agent-api/v1 §3.15](../agent-api/01-schema.md#315-error)）。
>
> **与 §2 必含字段（11 个）的关系**：§2 列 11 个必含字段是"必含最小集"（Actor / ActorType / Session / Workspace / Task / Action / Resource / Timestamp / Permission / Result / TraceID）；§6 是 schema 形式 — 7 字段是合并抽象版（Actor 包含 ActorType；Workspace/Task/Resource 合并到 target；Permission 合并到 payload；Result 合并到 payload；Session 合并到 actor 或 payload）。**两份都是权威**，§2 11 字段是"业务必含"；§6 7 字段是"schema 形式"。

> v0.2 fix: 2026-08-27 per B-27 (Audit 7 字段 schema)

## 7. 实施位置

- `crates/star-audit/` — Audit service
- `crates/star-audit/src/append_only.rs` — Append-only 存储
- `crates/star-audit/src/hmac.rs` — HMAC chain
- `crates/star-audit/src/audit_entry.rs` — AuditEntry schema 落地（per B-27 修复 2026-08-27）

## 8. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-26 | Mavis（per DEC-008）| 初版：5 ActorType + 11 必含字段 + Append-only | Phase C 54 份 spec 草案 |
| v0.2 | 2026-08-27 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 agent（per 2026-08-27 07:16 JST 代签规则反转）| 🟡 B-07/B-16：§3 5 ActorType 把 "IDE" 改 "System"（合并 IDE Session + GitGit + Platform Event Bus）+ 5 ActorType 边界澄清（Service = STAR 一等对象 / Automation = 外部脚本）· 🟡 B-27：§6 增 AuditEntry 7 字段 schema（`audit_id` / `actor` / `action` / `target` / `timestamp` / `trace_id` / `payload`），引用 [agent-api/v1 §3.15 Error.trace_id](../agent-api/01-schema.md#315-error)，W4 子代理落 [agent-api/v1 §3.X AuditEntry](../agent-api/01-schema.md) 后 1:1 对齐 + §7 增 `audit_entry.rs` 实施位置 | worker 子代理修 INTERFACE-REVIEW-B 8 子代理报告 follow-up |

> v0.2 fix: 2026-08-27 per B-07 / B-16 / B-27
