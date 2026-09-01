# domain-kms 实施 spec

> **状态**: Draft v0.1 (2026-09-01)
> **触发**: per 2026-09-01 15:03 JST GAP-01 7 supporting crate 加 spec (PHASE-INTER-COLLAB-REFINE-REPORT §3 GAP-01)
> **下游交付**: Implementation team — Rust crate 路径 `crates/{display}/`

> **dual-use 警告** (per AGENTS.md §5 v0.6 + 2026-08-31 22:45 JST Q1-D 拍板):
> domain-kms 是 Star 仓 supporting domain crate (per [basic-design §6 22 logical domain + 7 supporting crate](../../../basic-design.md))
> 不在 22 bounded context 主域列表,与 5 域 (player/economy/match/social/admin) 历史治理命名**不建立映射**。

---

## 1. 职责与边界

`domain-kms` 负责 **KMS 抽象 (Vault / AWS KMS 凭据) — E.4 mock 备选 (per 29692a7 mock 备选路径, GAP-01 提前补 spec)**。

**属于本 crate 的**:
- KMS Provider 抽象 (Vault / AWS KMS / Mock / Future)
- 凭据 CRUD: encrypt / decrypt / sign / verify (per use case)
- 凭据轮转: schedule / force / grace-period
- 凭据审计: 谁在何时访问哪个凭据

**不属于本 crate 的**:
- 具体 Provider 实现 (Mock / Vault / AWS KMS, 由 adapter 实现)
- 凭据内容 (调用方通过 `SecretString` 引用,不接触 plaintext)

## 2. 关键实体

- `KmsKey` (聚合根): key_id / tenant_id / alias / kind (Encryption | Signing) / algorithm / rotation_policy / status (Active | PendingDeletion | Disabled)
- `KmsAuditEvent`: event_id / key_id / actor_id / operation / timestamp / success
- `SecretString` (值对象): 持有 plaintext, 0 处打印 (per AGENTS.md §4 #5 hard ban)

## 3. 关键不变量

| ID | 不变量 |
|---|---|
| INV-KMS-01 | SecretString 不得离开 domain-kms 边界 (调用方通过 API 注入, 不接触 plaintext) |
| INV-KMS-02 | 凭据轮转期间旧凭据 grace-period 仍可用 (per use case) |
| INV-KMS-03 | 凭据删除前必 force-rotate 一次 (防残留) |
| INV-KMS-04 | 凭据审计必 Append-only (per REQ-AUDIT-001) |

## 4. 接口契约

- `KmsCommandPort`: create-key / rotate / schedule-deletion / force-rotate
- `KmsQueryPort`: get-key (不返回 plaintext) / list-by-tenant / get-rotation-status
- `SecretResolver`: resolve(ref) -> SecretString (注入式,调用方不接触)

## 5. 跨 domain 接触面 (v0.16 协作细化新增)

per [basic-design v0.16 §3.2.9 22 domain contact face 表](../../../basic-design.md) + [ADR-0039 §D26-D32 Worktree Orchestration 跨域协作](../../architecture/2026-08-26-upgrade/adr/0039-worktree-orchestration-cross-domain.md),`domain-kms` 作为 supporting crate 跨 22 bounded context + 6 supporting crate 的接触面。

**协作模式** (per [basic-design v0.16 §3.1 解耦机制](../../../basic-design.md) 8 种):

| 接触类型 | 目标 domain | 协作方式 | 引用 |
|---|---|---|---|
| `kms` 触发工单创建 | work-item | Customer-Supplier (Port) | per `kms` 提交触发 |
| `kms` 读取用户身份 | identity | Shared Kernel (UserId) | per User 引用 |
| `kms` 审计所有操作 | audit | Separate Ways (Append-only) | per AuditRecorder Port |
| `kms` 触发降噪通知 | notification | Separate Ways (异步) | per REQ-NOTIF-002 |

> 详细接触面待 [basic-design v0.16 §3.2.9](../../../basic-design.md) 后续 sweep 补充 (per GAP-01 后续 P3 阶段)。

## 6. 风险与缓解

| Risk | 影响 | 缓解 | 引用 |
|---|---|---|---|
| RISK-KMS-01: 凭据泄漏 | SecretString 0 打印 + 引用 SecretResolver 注入式 | — | domain-kms §6 |
| RISK-KMS-02: 凭据轮转失败 | grace-period + 自动 retry + 人工介入 | — | domain-kms §6 |
| RISK-KMS-03: Provider 不可用 | fallback 到 Mock (per 29692a7 mock 备选路径) | — | domain-kms §6 |
| RISK-KMS-04: 删除残留 | force-rotate 前置 + PendingDeletion 状态保留 grace-period | — | domain-kms §6 |

## 7. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | 架构师 (Mavis 接手 agent per DEC-008) | 初版：7 章简化模板 (职责 / 实体 / 不变量 / 接口 / 接触面 / 风险 / 修订历史) | 2026-09-01 15:03 JST GAP-01 7 supporting crate 加 spec (per PHASE-INTER-COLLAB-REFINE-REPORT §3 GAP-01) |

---

> **审批者**: 架构师 (Mavis 接手 agent per DEC-008) — 2026-09-01
> **per AGENTS.md §1.0 用户授权升级 v0.5 + 8/27 19:39/20:56/21:59 JST 三次强化**: Mavis 接手默认代签 Ulysses 无需再问
