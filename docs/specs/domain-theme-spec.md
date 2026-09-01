# domain-theme 实施 spec

> **状态**: Draft v0.1 (2026-09-01)
> **触发**: per 2026-09-01 15:03 JST GAP-01 7 supporting crate 加 spec (PHASE-INTER-COLLAB-REFINE-REPORT §3 GAP-01)
> **下游交付**: Implementation team — Rust crate 路径 `crates/{display}/`

> **dual-use 警告** (per AGENTS.md §5 v0.6 + 2026-08-31 22:45 JST Q1-D 拍板):
> domain-theme 是 Star 仓 supporting domain crate (per [basic-design §6 22 logical domain + 7 supporting crate](../../../basic-design.md))
> 不在 22 bounded context 主域列表,与 5 域 (player/economy/match/social/admin) 历史治理命名**不建立映射**。

---

## 1. 职责与边界

`domain-theme` 负责 **Star 主题系统 (per crates/domain-theme/src/lib.rs 实施实装, per 2026-08-29 04:09 JST 用户拍板)**。

**属于本 crate 的**:
- 前端: next-themes (React 主题切换)
- 结构: 三元组 enum (Light + Dark + 扩展位)
- 优先级: 3 层覆盖 (Personal > Tenant > Global)
- 拓展: 主题 + 主题色系, 后续可扩展 (动画 / 字体)

**不属于本 crate 的**:
- 用户身份 (从 `domain-identity` 读 user_id)
- Tenant 隔离 (从 `domain-tenant` 读 tenant_id)

## 2. 关键实体

- `ThemePreference` (值对象): user_id? (Personal) / tenant_id? (Tenant) / global (bool) / theme (light | dark) / accent_color?
- `ThemeRegistry`: theme_id / name / accent_colors[] / preview_image_url

## 3. 关键不变量

| ID | 不变量 |
|---|---|
| INV-THEME-01 | Personal > Tenant > Global 优先级严格生效 |
| INV-THEME-02 | 主题切换不刷新页面 (前端 only) |
| INV-THEME-03 | 跨设备同步 Personal 主题 (经 user 身份) |

## 4. 接口契约

- `ThemePreferencePort`: get (resolve 3 层) / set (per layer) / reset
- `ThemeRegistryPort`: list-available / get-default

## 5. 跨 domain 接触面 (v0.16 协作细化新增)

per [basic-design v0.16 §3.2.9 22 domain contact face 表](../../../basic-design.md) + [ADR-0039 §D26-D32 Worktree Orchestration 跨域协作](../../architecture/2026-08-26-upgrade/adr/0039-worktree-orchestration-cross-domain.md),`domain-theme` 作为 supporting crate 跨 22 bounded context + 6 supporting crate 的接触面。

**协作模式** (per [basic-design v0.16 §3.1 解耦机制](../../../basic-design.md) 8 种):

| 接触类型 | 目标 domain | 协作方式 | 引用 |
|---|---|---|---|
| `theme` 触发工单创建 | work-item | Customer-Supplier (Port) | per `theme` 提交触发 |
| `theme` 读取用户身份 | identity | Shared Kernel (UserId) | per User 引用 |
| `theme` 审计所有操作 | audit | Separate Ways (Append-only) | per AuditRecorder Port |
| `theme` 触发降噪通知 | notification | Separate Ways (异步) | per REQ-NOTIF-002 |

> 详细接触面待 [basic-design v0.16 §3.2.9](../../../basic-design.md) 后续 sweep 补充 (per GAP-01 后续 P3 阶段)。

## 6. 风险与缓解

| Risk | 影响 | 缓解 | 引用 |
|---|---|---|---|
| RISK-THEME-01: 主题不兼容 (色盲 / 高对比) | WCAG 2.1 AA 合规检测 | — | domain-theme §6 |
| RISK-THEME-02: 跨设备同步冲突 | last-write-wins + Audit | — | domain-theme §6 |

## 7. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | 架构师 (Mavis 接手 agent per DEC-008) | 初版：7 章简化模板 (职责 / 实体 / 不变量 / 接口 / 接触面 / 风险 / 修订历史) | 2026-09-01 15:03 JST GAP-01 7 supporting crate 加 spec (per PHASE-INTER-COLLAB-REFINE-REPORT §3 GAP-01) |

---

> **审批者**: 架构师 (Mavis 接手 agent per DEC-008) — 2026-09-01
> **per AGENTS.md §1.0 用户授权升级 v0.5 + 8/27 19:39/20:56/21:59 JST 三次强化**: Mavis 接手默认代签 Ulysses 无需再问
