# domain-form 实施 spec

> **状态**: Draft v0.1 (2026-09-01)
> **触发**: per 2026-09-01 15:03 JST GAP-01 7 supporting crate 加 spec (PHASE-INTER-COLLAB-REFINE-REPORT §3 GAP-01)
> **下游交付**: Implementation team — Rust crate 路径 `crates/{display}/`

> **dual-use 警告** (per AGENTS.md §5 v0.6 + 2026-08-31 22:45 JST Q1-D 拍板):
> domain-form 是 Star 仓 supporting domain crate (per [basic-design §6 22 logical domain + 7 supporting crate](../../../basic-design.md))
> 不在 22 bounded context 主域列表,与 5 域 (player/economy/match/social/admin) 历史治理命名**不建立映射**。

---

## 1. 职责与边界

`domain-form` 负责 **Star Form Engine (per crates/domain-form/src/lib.rs v0.1 实施实装)**。

**属于本 crate 的**:
- 12 字段类型 (text / number / date / select / multi-select / file / user / work-item / ...)
- 条件逻辑 (show_if / require_if / hide_if, JSON Logic 风格)
- 提交触发 (工单创建 / 字段更新 / 发邮件 / 调 Webhook)
- 公开 URL 表单 (匿名提交)
- 表单版本管理

**不属于本 crate 的**:
- WorkItem 聚合根 (属 `domain-work-item`,form 触发工单创建后由 work-item 接管)
- Notification 投递 (属 `domain-notification`,form 触发后调用 notification Port)

## 2. 关键实体

- `FormDefinition` (聚合根): form_id / tenant_id / project_id / title / version / fields[] / triggers[] / public_url_slug?
- `FormField`: field_id / type (12 种) / label / required / default_value? / options? (select) / validation?
- `FormCondition`: rule (show_if / require_if / hide_if) + expression (JSON Logic)
- `FormSubmission`: submission_id / form_id / submitted_by / submitted_at / field_values{}

## 3. 关键不变量

| ID | 不变量 |
|---|---|
| INV-FORM-01 | 表单 schema 不可变,变更需新建 version (per 缺标比错标) |
| INV-FORM-02 | 公开 URL 表单不可要求登录 (匿名提交场景) |
| INV-FORM-03 | 条件逻辑不得循环引用 (静态分析检测) |
| INV-FORM-04 | 提交触发必走 Workflow Guard 校验,不可绕过 (per REQ-WF-003) |

## 4. 接口契约

- `FormDefinitionCommandPort`: create / update / publish (锁定 schema) / archive
- `FormDefinitionQueryPort`: get / list-by-project / get-by-public-slug
- `FormSubmissionPort`: submit (含条件逻辑执行 + 触发动作) / list-by-form

## 5. 跨 domain 接触面 (v0.16 协作细化新增)

per [basic-design v0.16 §3.2.9 22 domain contact face 表](../../../basic-design.md) + [ADR-0039 §D26-D32 Worktree Orchestration 跨域协作](../../architecture/2026-08-26-upgrade/adr/0039-worktree-orchestration-cross-domain.md),`domain-form` 作为 supporting crate 跨 22 bounded context + 6 supporting crate 的接触面。

**协作模式** (per [basic-design v0.16 §3.1 解耦机制](../../../basic-design.md) 8 种):

| 接触类型 | 目标 domain | 协作方式 | 引用 |
|---|---|---|---|
| `form` 触发工单创建 | work-item | Customer-Supplier (Port) | per `form` 提交触发 |
| `form` 读取用户身份 | identity | Shared Kernel (UserId) | per User 引用 |
| `form` 审计所有操作 | audit | Separate Ways (Append-only) | per AuditRecorder Port |
| `form` 触发降噪通知 | notification | Separate Ways (异步) | per REQ-NOTIF-002 |

> 详细接触面待 [basic-design v0.16 §3.2.9](../../../basic-design.md) 后续 sweep 补充 (per GAP-01 后续 P3 阶段)。

## 6. 风险与缓解

| Risk | 影响 | 缓解 | 引用 |
|---|---|---|---|
| RISK-FORM-01: 公开 URL 表单被滥用 | 限流 + Captcha + 提交审核 (per integration-design §8 Rate Limit) | — | domain-form §6 |
| RISK-FORM-02: 条件逻辑 bug 导致字段绕过 | 单元测试覆盖 show_if/require_if | — | domain-form §6 |
| RISK-FORM-03: 提交触发失败导致工单半成品 | Saga 编排 (per spec/saga/01 v0.2 §4) | — | domain-form §6 |

## 7. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | 架构师 (Mavis 接手 agent per DEC-008) | 初版：7 章简化模板 (职责 / 实体 / 不变量 / 接口 / 接触面 / 风险 / 修订历史) | 2026-09-01 15:03 JST GAP-01 7 supporting crate 加 spec (per PHASE-INTER-COLLAB-REFINE-REPORT §3 GAP-01) |

---

> **审批者**: 架构师 (Mavis 接手 agent per DEC-008) — 2026-09-01
> **per AGENTS.md §1.0 用户授权升级 v0.5 + 8/27 19:39/20:56/21:59 JST 三次强化**: Mavis 接手默认代签 Ulysses 无需再问
