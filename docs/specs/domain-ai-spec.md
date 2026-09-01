# domain-ai 实施 spec

> **状态**: Draft v0.1 (2026-09-01)
> **触发**: per 2026-09-01 15:03 JST GAP-01 7 supporting crate 加 spec (PHASE-INTER-COLLAB-REFINE-REPORT §3 GAP-01)
> **下游交付**: Implementation team — Rust crate 路径 `crates/{display}/`

> **dual-use 警告** (per AGENTS.md §5 v0.6 + 2026-08-31 22:45 JST Q1-D 拍板):
> domain-ai 是 Star 仓 supporting domain crate (per [basic-design §6 22 logical domain + 7 supporting crate](../../../basic-design.md))
> 不在 22 bounded context 主域列表,与 5 域 (player/economy/match/social/admin) 历史治理命名**不建立映射**。

---

## 1. 职责与边界

`domain-ai` 负责 **Star AI Engine (per crates/domain-ai/src/lib.rs v0.1 实施实装)**。

**属于本 crate 的**:
- 3 类 Rovo-like Agent:
-   1. Workflow Builder (自然语言 → Workflow JSON)
-   2. Work Readiness Checker (开工前 AI 自检: AC 覆盖 / 依赖 / Conflict)
-   3. Report Insight (报表 → 自然语言洞察)
- + JQL AI (自然语言 → JQL, per requirements §12 REQ-SEARCH-002 V1 候选)

**不属于本 crate 的**:
- Coding Agent 进程 (属 `domain-agent`,本 crate 只生成 Workflow JSON 等元数据,不 spawn agent)
- AI Provider 凭据 (属 `domain-kms`,本 crate 通过 Adapter 抽象调用)

## 2. 关键实体

- `AiAgent` (聚合根): agent_id / kind (4 类) / input_schema / output_schema / model_preference / prompt_template
- `AiInvocation` (Projection): invocation_id / agent_id / input{} / output{} / latency_ms / token_used / created_at
- `AiFeedback` (Feedback): invocation_id / user_id / rating (1-5) / comment?

## 3. 关键不变量

| ID | 不变量 |
|---|---|
| INV-AI-01 | AI 输出必须可解释 (per requirements §28 AI Extension) |
| INV-AI-02 | AI 操作必须可审计 (per REQ-AUDIT-002) |
| INV-AI-03 | AI Token 预算受控 (per basic-design §4.4.4 Token Budget P0/P1/P2/P3/P4) |
| INV-AI-04 | Workflow Builder 输出必须 JSON Schema 验证 (拒绝幻觉结构) |

## 4. 接口契约

- `AiAgentCommandPort`: register / deregister / list / get
- `AiInvocationPort`: invoke (sync) / invoke-async (fire-and-forget) / get-result / list-by-user
- `AiFeedbackPort`: submit / list-by-agent / aggregate-rating

## 5. 跨 domain 接触面 (v0.16 协作细化新增)

per [basic-design v0.16 §3.2.9 22 domain contact face 表](../../../basic-design.md) + [ADR-0039 §D26-D32 Worktree Orchestration 跨域协作](../../architecture/2026-08-26-upgrade/adr/0039-worktree-orchestration-cross-domain.md),`domain-ai` 作为 supporting crate 跨 22 bounded context + 6 supporting crate 的接触面。

**协作模式** (per [basic-design v0.16 §3.1 解耦机制](../../../basic-design.md) 8 种):

| 接触类型 | 目标 domain | 协作方式 | 引用 |
|---|---|---|---|
| `ai` 触发工单创建 | work-item | Customer-Supplier (Port) | per `ai` 提交触发 |
| `ai` 读取用户身份 | identity | Shared Kernel (UserId) | per User 引用 |
| `ai` 审计所有操作 | audit | Separate Ways (Append-only) | per AuditRecorder Port |
| `ai` 触发降噪通知 | notification | Separate Ways (异步) | per REQ-NOTIF-002 |

> 详细接触面待 [basic-design v0.16 §3.2.9](../../../basic-design.md) 后续 sweep 补充 (per GAP-01 后续 P3 阶段)。

## 6. 风险与缓解

| Risk | 影响 | 缓解 | 引用 |
|---|---|---|---|
| RISK-AI-01: AI 幻觉 | 强制 JSON Schema 验证 + 用户 confirm gate | — | domain-ai §6 |
| RISK-AI-02: Token 超支 | Token Budget P0 不可裁剪 (per basic-design §4.4.4) | — | domain-ai §6 |
| RISK-AI-03: Workflow Builder 生成危险操作 | dry-run 强制 + 显式 confirm | — | domain-ai §6 |
| RISK-AI-04: AI Provider 数据泄漏 | ProviderDataBoundary (per domain-tenant SecurityPolicy) | — | domain-ai §6 |

## 7. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | 架构师 (Mavis 接手 agent per DEC-008) | 初版：7 章简化模板 (职责 / 实体 / 不变量 / 接口 / 接触面 / 风险 / 修订历史) | 2026-09-01 15:03 JST GAP-01 7 supporting crate 加 spec (per PHASE-INTER-COLLAB-REFINE-REPORT §3 GAP-01) |

---

> **审批者**: 架构师 (Mavis 接手 agent per DEC-008) — 2026-09-01
> **per AGENTS.md §1.0 用户授权升级 v0.5 + 8/27 19:39/20:56/21:59 JST 三次强化**: Mavis 接手默认代签 Ulysses 无需再问
