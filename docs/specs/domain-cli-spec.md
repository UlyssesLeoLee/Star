# domain-cli 实施 spec

> **状态**: Draft v0.1 (2026-09-01)
> **触发**: per 2026-09-01 15:03 JST GAP-01 7 supporting crate 加 spec (PHASE-INTER-COLLAB-REFINE-REPORT §3 GAP-01)
> **下游交付**: Implementation team — Rust crate 路径 `crates/{display}/`

> **dual-use 警告** (per AGENTS.md §5 v0.6 + 2026-08-31 22:45 JST Q1-D 拍板):
> domain-cli 是 Star 仓 supporting domain crate (per [basic-design §6 22 logical domain + 7 supporting crate](../../../basic-design.md))
> 不在 22 bounded context 主域列表,与 5 域 (player/economy/match/social/admin) 历史治理命名**不建立映射**。

---

## 1. 职责与边界

`domain-cli` 负责 **Star CLI Agent Registry (per crates/domain-cli/src/lib.rs v0.1 实施实装)**。

**属于本 crate 的**:
- 6 种内置 CLI Agent 注册 (claude / codex / openclaw / hermes / gemini / aider) + 自定义 schema
- **双模式 API Key 存储** (per 2026-08-29 09:07 JST 用户拍板):
-   - `EncryptedRust`: AES-256-GCM 加密存储于 domain-cli
-   - `EnvironmentVar`: 运行时读 process env 即可,不存储
- API Agent 适配 (OpenClaw / Hermes HTTP API, 替代 CLI spawn)
- Agent Adapter 模式 (per ADR-0025 vendor adapter anti-contamination)

**不属于本 crate 的**:
- Agent Process 生命周期 (spawn/kill/lease) (属 `domain-agent`)
- Worktree 实装 (属 `domain-worktree`)
- Context Packet 编译 (属 `domain-context`)

## 2. 关键实体

- `CliAgent` (聚合根): agent_id / name / kind (内置 6 种 / Custom) / schema / api_key_ref (EncryptedRust | EnvironmentVar) / created_at
- `ApiKeyRef`: ref_type (EncryptedRust | EnvironmentVar) + ciphertext (EncryptedRust) | env_var_name (EnvironmentVar)
- `AgentAdapter`: provider / endpoint / auth_pattern / rate_limit

## 3. 关键不变量

| ID | 不变量 |
|---|---|
| INV-CLI-01 | 6 种内置 CLI Agent schema 不得修改 (向后兼容) |
| INV-CLI-02 | API Key 不得 plaintext 存储 (per REQ-SEC-002) |
| INV-CLI-03 | API Key EnvVar 模式不写入 domain-cli (运行时读取) |
| INV-CLI-04 | agent_id 全局唯一,跨 tenant 隔离 |

## 4. 接口契约

- `CliAgentCommandPort`: register / deregister / list / get
- `CliAgentQueryPort`: lookup by name / by kind / list-enabled
- `ApiKeyResolver`: resolve(ref) -> SecretString (注入式,调用方不接触)

## 5. 跨 domain 接触面 (v0.16 协作细化新增)

per [basic-design v0.16 §3.2.9 22 domain contact face 表](../../../basic-design.md) + [ADR-0039 §D26-D32 Worktree Orchestration 跨域协作](../../architecture/2026-08-26-upgrade/adr/0039-worktree-orchestration-cross-domain.md),`domain-cli` 作为 supporting crate 跨 22 bounded context + 6 supporting crate 的接触面。

**协作模式** (per [basic-design v0.16 §3.1 解耦机制](../../../basic-design.md) 8 种):

| 接触类型 | 目标 domain | 协作方式 | 引用 |
|---|---|---|---|
| `cli` 触发工单创建 | work-item | Customer-Supplier (Port) | per `cli` 提交触发 |
| `cli` 读取用户身份 | identity | Shared Kernel (UserId) | per User 引用 |
| `cli` 审计所有操作 | audit | Separate Ways (Append-only) | per AuditRecorder Port |
| `cli` 触发降噪通知 | notification | Separate Ways (异步) | per REQ-NOTIF-002 |

> 详细接触面待 [basic-design v0.16 §3.2.9](../../../basic-design.md) 后续 sweep 补充 (per GAP-01 后续 P3 阶段)。

## 6. 风险与缓解

| Risk | 影响 | 缓解 | 引用 |
|---|---|---|---|
| RISK-CLI-01: API Key 泄漏 | AES-256-GCM 加密 + 注入式 (per ADR-0025) | — | domain-cli §6 |
| RISK-CLI-02: 自定义 schema 兼容性 | schema version 字段 + 渐进式升级 | — | domain-cli §6 |
| RISK-CLI-03: EnvVar 模式 secret 误打印 | 引用 $env:VAR 不打印 (per AGENTS.md §4 #5 hard ban) | — | domain-cli §6 |

## 7. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | 架构师 (Mavis 接手 agent per DEC-008) | 初版：7 章简化模板 (职责 / 实体 / 不变量 / 接口 / 接触面 / 风险 / 修订历史) | 2026-09-01 15:03 JST GAP-01 7 supporting crate 加 spec (per PHASE-INTER-COLLAB-REFINE-REPORT §3 GAP-01) |

---

> **审批者**: 架构师 (Mavis 接手 agent per DEC-008) — 2026-09-01
> **per AGENTS.md §1.0 用户授权升级 v0.5 + 8/27 19:39/20:56/21:59 JST 三次强化**: Mavis 接手默认代签 Ulysses 无需再问
